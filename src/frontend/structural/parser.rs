use std::collections::{HashMap, VecDeque};

use crate::events::EventLog;
use crate::frontend::formulation::{
    ParseError as FormulationParseError, parse_author_header, parse_command_header,
    parse_expression, parse_expression_alias, parse_form_or_declaration, parse_is_via_statement,
    parse_label_header, parse_ordinary_declaration_statement, parse_refined_declaration_statement,
    parse_resource_header, parse_spec_operator_alias, parse_writing_alias,
};
use crate::frontend::proto::Parser as ProtoParser;
use crate::frontend::proto::ast::{
    Argument as ProtoArgument, Formulation as ProtoFormulation, Group as ProtoGroup,
    Section as ProtoSection, TextLiteral as ProtoText,
};

use super::ast::*;

const ORIGIN: &str = "structural_parser";

/// Parses raw MathLingua source into the strongly typed structural AST.
///
/// This function composes the proto parser with structural recognition.  Proto
/// groups that cannot be recognized are diagnosed and skipped, allowing valid
/// neighboring groups to continue into backend checks and rendering.
pub fn parse_document(input: &str, tracker: &mut EventLog) -> Document {
    let groups = {
        let mut proto_parser = ProtoParser::new(input, tracker);
        proto_parser.parse()
    };

    let mut items = Vec::new();
    for group in &groups {
        if let Some(item) = parse_top_level_group(group, tracker) {
            items.push(item);
        }
    }

    Document {
        items: ZeroOrMore::from(items),
    }
}

// ===============================[ sections ]=====================================

/// Returns the first section label of a proto group.
///
/// Structural group dispatch is label-first, so groups without sections cannot
/// be recognized.
pub(in crate::frontend::structural::parser) fn first_section_label(
    group: &ProtoGroup,
) -> Option<&str> {
    group.sections.first().map(|section| section.label.as_str())
}

/// Looks up an identified section by label.
///
/// The section map stores borrowed proto sections keyed by their normalized
/// expected label.
pub(in crate::frontend::structural::parser) fn section<'a>(
    sections: &'a HashMap<String, &'a ProtoSection>,
    label: &str,
) -> Option<&'a ProtoSection> {
    sections.get(label).copied()
}

/// Validates section order and presence for a structural group.
///
/// The `expected` slice is an ordered pattern where labels ending in `?` are
/// optional.  The returned map contains only sections that were present and
/// accepted.  Diagnostics include the full expected pattern to make authoring
/// mistakes easier to repair.
pub(in crate::frontend::structural::parser) fn identify_sections<'a>(
    name: &str,
    sections: &'a [ProtoSection],
    tracker: &mut EventLog,
    expected: &[&str],
) -> Option<HashMap<String, &'a ProtoSection>> {
    let mut section_queue: VecDeque<&ProtoSection> = sections.iter().collect();
    let mut expected_queue: VecDeque<&str> = expected.iter().copied().collect();
    let mut result = HashMap::new();

    let pattern = expected
        .iter()
        .map(|value| format!("{value}:"))
        .collect::<Vec<_>>()
        .join("\n");

    while let (Some(next_section), Some(maybe_name)) = (
        section_queue.front().copied(),
        expected_queue.front().copied(),
    ) {
        let is_optional = maybe_name.ends_with('?');
        let true_name = maybe_name.trim_end_matches('?');

        if next_section.label == true_name {
            result.insert(true_name.to_owned(), next_section);
            section_queue.pop_front();
            expected_queue.pop_front();
        } else if is_optional {
            expected_queue.pop_front();
        } else {
            tracker.user_error_at_row(
                Some(ORIGIN),
                next_section.metadata.row,
                format!(
                    "For {name} pattern:\n\n{pattern}\n\nExpected `{true_name}` but found `{}`",
                    next_section.label
                ),
            );
            return None;
        }
    }

    if let Some(unexpected) = section_queue.front() {
        tracker.user_error_at_row(
            Some(ORIGIN),
            unexpected.metadata.row,
            format!(
                "For {name} pattern:\n\n{pattern}\n\nUnexpected section `{}`",
                unexpected.label
            ),
        );
        return None;
    }

    if let Some(missing) = expected_queue
        .iter()
        .find(|name| !name.ends_with('?'))
        .copied()
    {
        let row = sections
            .first()
            .map(|section| section.metadata.row)
            .unwrap_or(0);
        tracker.user_error_at_row(
            Some(ORIGIN),
            row,
            format!(
                "For {name} pattern:\n\n{pattern}\n\nExpected section `{}`",
                missing.trim_end_matches('?')
            ),
        );
        return None;
    }

    Some(result)
}

/// One flattened entry from a proto section body.
///
/// Section entries unify inline arguments with body arguments so helper parsers
/// can apply the same validation logic regardless of how the author chose to
/// place the section content.
pub(in crate::frontend::structural::parser) enum SectionEntry<'a> {
    /// Inline text after the section colon.
    Inline { text: &'a str, row: usize },
    /// A formulation body argument.
    Formulation { text: &'a str, row: usize },
    /// A quoted text body argument.
    Text { text: &'a str, row: usize },
    /// A nested proto group body argument.
    Group { group: &'a ProtoGroup, row: usize },
}

/// Flattens a section's inline and body arguments into parseable entries.
///
/// Inline arguments are yielded first using the section row, followed by body
/// arguments in source order with their own rows.
pub(in crate::frontend::structural::parser) fn section_entries(
    section: &ProtoSection,
) -> Vec<SectionEntry<'_>> {
    let mut entries = Vec::new();
    if let Some(argument) = section.inline_argument.as_deref() {
        entries.push(SectionEntry::Inline {
            text: argument,
            row: section.metadata.row,
        });
    }

    for argument in &section.arguments {
        match argument {
            ProtoArgument::Formulation(ProtoFormulation { text, metadata }) => {
                entries.push(SectionEntry::Formulation {
                    text,
                    row: metadata.row,
                });
            }
            ProtoArgument::Text(ProtoText { text, metadata }) => {
                entries.push(SectionEntry::Text {
                    text,
                    row: metadata.row,
                });
            }
            ProtoArgument::Group(group) => {
                entries.push(SectionEntry::Group {
                    group,
                    row: group.metadata.row,
                });
            }
        }
    }

    entries
}

// ===============================[ groups ]=====================================

/// Parses one or more required nested groups from a section.
///
/// The supplied parser determines which nested group kinds are legal in the
/// section, letting this helper centralize cardinality and non-group diagnostics.
pub(in crate::frontend::structural::parser) fn parse_required_groups<T>(
    section: &ProtoSection,
    label: &str,
    tracker: &mut EventLog,
    parser: fn(&ProtoGroup, &mut EventLog) -> Option<T>,
) -> Option<OneOrMore<T>> {
    let starting_issue_count = tracker.issue_count();
    let items = parse_optional_groups(Some(section), label, tracker, parser);
    one_or_more(items, || {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(
                Some(ORIGIN),
                section.metadata.row,
                format!("Expected nested groups in `{label}`"),
            );
        }
    })
}

/// Parses zero or more nested groups from an optional section.
///
/// Non-group entries are reported because this helper is used only for sections
/// whose grammar requires group-shaped items.
pub(in crate::frontend::structural::parser) fn parse_optional_groups<T>(
    section: Option<&ProtoSection>,
    label: &str,
    tracker: &mut EventLog,
    parser: fn(&ProtoGroup, &mut EventLog) -> Option<T>,
) -> ZeroOrMore<T> {
    let Some(section) = section else {
        return ZeroOrMore::default();
    };

    let mut items = Vec::new();
    for entry in section_entries(section) {
        match entry {
            SectionEntry::Group { group, .. } => {
                if let Some(item) = parser(group, tracker) {
                    items.push(item);
                }
            }
            SectionEntry::Inline { row, .. }
            | SectionEntry::Formulation { row, .. }
            | SectionEntry::Text { row, .. } => {
                tracker.user_error_at_row(
                    Some(ORIGIN),
                    row,
                    format!("Expected nested group in section `{label}`"),
                );
            }
        }
    }

    items.into()
}

// ===============================[ headings ]=====================================

/// Parses an alias body into the supported alias variants.
///
/// Expression aliases are attempted first because their left-hand side grammar
/// is broader; if that fails the body is parsed as a specification-operator
/// alias.
pub(in crate::frontend::structural::parser) fn parse_alias_kind(
    input: &str,
) -> Result<AliasKind, FormulationParseError> {
    if let Ok(alias) = parse_expression_alias(input) {
        return Ok(AliasKind::Expression(alias));
    }
    parse_spec_operator_alias(input).map(AliasKind::SpecOperator)
}

/// Parses an item accepted by `extends:` and related sections.
///
/// `is ... via ...` is more specific, so it is attempted before the broader
/// `is`/spec parser.
pub(in crate::frontend::structural::parser) fn parse_is_or_via_item(
    input: &str,
) -> Result<IsOrViaItem, FormulationParseError> {
    if let Ok(item) = parse_is_via_statement(input) {
        return Ok(IsOrViaItem::IsVia(item));
    }
    parse_ordinary_declaration_statement(input).map(IsOrViaItem::Declaration)
}

/// Parses a quantifier binding or ordinary specification.
pub(in crate::frontend::structural::parser) fn parse_binding_or_spec(
    input: &str,
) -> Result<BindingOrSpec, FormulationParseError> {
    parse_ordinary_declaration_statement(input).map(BindingOrSpec::Declaration)
}

/// Parses a required command heading from a proto group.
///
/// Missing or malformed headings are reported at the group row because headings
/// live on the group header line rather than in a section body.
pub(in crate::frontend::structural::parser) fn parse_required_command_heading(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<crate::frontend::formulation::ast::CommandHeader> {
    let Some(heading) = group.heading.as_deref() else {
        tracker.user_error_at_row(Some(ORIGIN), group.metadata.row, "Expected command heading");
        return None;
    };
    match parse_command_header(heading) {
        Ok(heading) => Some(heading),
        Err(error) => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Invalid command heading: {error}"),
            );
            None
        }
    }
}

/// Parses an optional command heading from a proto group.
///
/// `None` means the group has no heading; an invalid present heading prevents
/// construction of the enclosing structural group.
pub(in crate::frontend::structural::parser) fn parse_optional_command_heading(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<Option<crate::frontend::formulation::ast::CommandHeader>> {
    match group.heading.as_deref() {
        Some(heading) => match parse_command_header(heading) {
            Ok(heading) => Some(Some(heading)),
            Err(error) => {
                tracker.user_error_at_row(
                    Some(ORIGIN),
                    group.metadata.row,
                    format!("Invalid command heading: {error}"),
                );
                None
            }
        },
        None => Some(None),
    }
}

/// Parses an optional label heading from a nested proto group.
///
/// Label headings are used for local documentation/proof notes and are
/// syntactically distinct from command headings.
pub(in crate::frontend::structural::parser) fn parse_optional_label_heading(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<Option<crate::frontend::formulation::ast::LabelHeader>> {
    match group.heading.as_deref() {
        Some(heading) => match parse_label_header(heading) {
            Ok(heading) => Some(Some(heading)),
            Err(error) => {
                tracker.user_error_at_row(
                    Some(ORIGIN),
                    group.metadata.row,
                    format!("Invalid label heading: {error}"),
                );
                None
            }
        },
        None => Some(None),
    }
}

/// Parses a required author heading from a `Person:` group.
pub(in crate::frontend::structural::parser) fn parse_required_author_heading(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<crate::frontend::formulation::ast::AuthorHeader> {
    let Some(heading) = group.heading.as_deref() else {
        tracker.user_error_at_row(Some(ORIGIN), group.metadata.row, "Expected author heading");
        return None;
    };
    match parse_author_header(heading) {
        Ok(heading) => Some(heading),
        Err(error) => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Invalid author heading: {error}"),
            );
            None
        }
    }
}

/// Parses a required resource heading from a `Resource:` group.
pub(in crate::frontend::structural::parser) fn parse_required_resource_heading(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<crate::frontend::formulation::ast::ResourceHeader> {
    let Some(heading) = group.heading.as_deref() else {
        tracker.user_error_at_row(
            Some(ORIGIN),
            group.metadata.row,
            "Expected resource heading",
        );
        return None;
    };
    match parse_resource_header(heading) {
        Ok(heading) => Some(heading),
        Err(error) => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Invalid resource heading: {error}"),
            );
            None
        }
    }
}

/// Ensures a group has no bracket heading.
///
/// Outline and metadata groups derive their identity from sections only; a
/// heading on those groups is almost certainly an authoring mistake.
pub(in crate::frontend::structural::parser) fn ensure_no_heading(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<()> {
    if let Some(heading) = &group.heading {
        tracker.user_error_at_row(
            Some(ORIGIN),
            group.metadata.row,
            format!("Unexpected heading `{heading}`"),
        );
        None
    } else {
        Some(())
    }
}

// ===============================[ clauses ]=====================================

/// Parses exactly one required clause from a section.
///
/// This is used for section shapes such as `expresses:` and `not:` where the
/// language grammar expects one logical clause.
pub(in crate::frontend::structural::parser) fn parse_required_clause(
    section: &ProtoSection,
    label: &str,
    tracker: &mut EventLog,
) -> Option<Clause> {
    let starting_issue_count = tracker.issue_count();
    let clauses = parse_optional_clauses(Some(section), label, tracker);
    if clauses.is_empty() {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(
                Some(ORIGIN),
                section.metadata.row,
                format!("Expected a clause in `{label}`"),
            );
        }
        None
    } else {
        Some(clauses.into_iter().next().expect("non-empty clauses"))
    }
}

/// Parses one or more required clauses from a section.
pub(in crate::frontend::structural::parser) fn parse_required_clauses(
    section: &ProtoSection,
    label: &str,
    tracker: &mut EventLog,
) -> Option<OneOrMore<Clause>> {
    let starting_issue_count = tracker.issue_count();
    let clauses = parse_optional_clauses(Some(section), label, tracker);
    one_or_more(clauses, || {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(
                Some(ORIGIN),
                section.metadata.row,
                format!("Expected clauses in `{label}`"),
            );
        }
    })
}

/// Parses zero or more clauses from an optional section.
///
/// Inline formulations are parsed first as declaration statements, then as
/// expressions, while nested groups are dispatched through clause-group parsers.
pub(in crate::frontend::structural::parser) fn parse_optional_clauses(
    section: Option<&ProtoSection>,
    label: &str,
    tracker: &mut EventLog,
) -> ZeroOrMore<Clause> {
    let Some(section) = section else {
        return ZeroOrMore::default();
    };

    let mut result = Vec::new();
    for entry in section_entries(section) {
        match entry {
            SectionEntry::Inline { text, row } | SectionEntry::Formulation { text, row } => {
                if let Ok(statement) = parse_ordinary_declaration_statement(text) {
                    result.push(Clause::Declaration(statement));
                    continue;
                }

                match parse_expression(text) {
                    Ok(expression) => result.push(Clause::Expression(expression)),
                    Err(expression_error) => tracker.user_error_at_row(
                        Some(ORIGIN),
                        row,
                        format!("Invalid clause expression in `{label}`: {expression_error}"),
                    ),
                }
            }
            SectionEntry::Group { group, .. } => {
                if let Some(clause) = parse_clause_group(group, tracker) {
                    result.push(clause);
                }
            }
            SectionEntry::Text { row, .. } => {
                tracker.user_error_at_row(
                    Some(ORIGIN),
                    row,
                    format!("Expected clause in section `{label}`"),
                );
            }
        }
    }

    result.into()
}

// ===============================[ formulations ]=====================================

/// Parses exactly one required formulation from a section.
///
/// The parser function is supplied by the caller so this helper can parse
/// expressions, command headings, labels, resources, or other formulation
/// fragments while sharing diagnostics and cardinality checks.
pub(in crate::frontend::structural::parser) fn parse_required_formulation<T>(
    section: &ProtoSection,
    label: &str,
    tracker: &mut EventLog,
    parser: fn(&str) -> Result<T, FormulationParseError>,
) -> Option<T> {
    let starting_issue_count = tracker.issue_count();
    let items = parse_optional_formulations(Some(section), label, tracker, parser);
    if items.is_empty() {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(
                Some(ORIGIN),
                section.metadata.row,
                format!("Expected a {label} formulation"),
            );
        }
        None
    } else {
        Some(items.into_iter().next().expect("non-empty formulations"))
    }
}

/// Parses one or more required formulations from a section.
///
/// If parsing produced no items and no more specific issue was emitted, this
/// helper reports a missing-content diagnostic for the whole section.
pub(in crate::frontend::structural::parser) fn parse_required_formulations<T>(
    section: &ProtoSection,
    label: &str,
    tracker: &mut EventLog,
    parser: fn(&str) -> Result<T, FormulationParseError>,
) -> Option<OneOrMore<T>> {
    let starting_issue_count = tracker.issue_count();
    let items = parse_optional_formulations(Some(section), label, tracker, parser);
    one_or_more(items, || {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(
                Some(ORIGIN),
                section.metadata.row,
                format!("Expected {label} formulations"),
            );
        }
    })
}

/// Parses zero or more formulations from an optional section.
///
/// Inline section arguments and formulation arguments are accepted.  Text and
/// nested groups are diagnosed because callers requested formulation content.
pub(in crate::frontend::structural::parser) fn parse_optional_formulations<T>(
    section: Option<&ProtoSection>,
    label: &str,
    tracker: &mut EventLog,
    parser: fn(&str) -> Result<T, FormulationParseError>,
) -> ZeroOrMore<T> {
    let Some(section) = section else {
        return ZeroOrMore::default();
    };

    let mut result = Vec::new();
    for entry in section_entries(section) {
        match entry {
            SectionEntry::Inline { text, row } | SectionEntry::Formulation { text, row } => {
                match parser(text) {
                    Ok(value) => result.push(value),
                    Err(error) => tracker.user_error_at_row(
                        Some(ORIGIN),
                        row,
                        format!("Invalid {label} formulation: {error}"),
                    ),
                }
            }
            SectionEntry::Text { row, .. } => {
                tracker.user_error_at_row(
                    Some(ORIGIN),
                    row,
                    format!("Expected formulation in section `{label}`"),
                );
            }
            SectionEntry::Group { row, .. } => {
                tracker.user_error_at_row(
                    Some(ORIGIN),
                    row,
                    format!("Expected formulation in section `{label}`"),
                );
            }
        }
    }

    result.into()
}

// ===============================[ text ]=====================================

/// Parses exactly one required quoted open-text entry.
///
/// Open-text sections accept inline quoted arguments and quoted text arguments,
/// with quote stripping handled by the shared text parser.
pub(in crate::frontend::structural::parser) fn parse_required_open_text(
    section: &ProtoSection,
    label: &str,
    tracker: &mut EventLog,
) -> Option<OpenText> {
    let starting_issue_count = tracker.issue_count();
    let texts = parse_optional_open_texts(Some(section), tracker);
    if texts.is_empty() {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(
                Some(ORIGIN),
                section.metadata.row,
                format!("Expected text in `{label}`"),
            );
        }
        None
    } else {
        Some(texts.into_iter().next().expect("non-empty texts"))
    }
}

/// Parses one or more required quoted open-text entries.
pub(in crate::frontend::structural::parser) fn parse_required_open_texts(
    section: &ProtoSection,
    label: &str,
    tracker: &mut EventLog,
) -> Option<OneOrMore<OpenText>> {
    let starting_issue_count = tracker.issue_count();
    let texts = parse_optional_open_texts(Some(section), tracker);
    one_or_more(texts, || {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(
                Some(ORIGIN),
                section.metadata.row,
                format!("Expected text entries in `{label}`"),
            );
        }
    })
}

/// Parses zero or more open-text entries from an optional section.
///
/// Missing sections become an empty wrapper, which lets callers model optional
/// prose without conflating it with malformed text in a present section.
pub(in crate::frontend::structural::parser) fn parse_optional_open_texts(
    section: Option<&ProtoSection>,
    tracker: &mut EventLog,
) -> ZeroOrMore<OpenText> {
    parse_optional_texts(section, tracker, OpenText)
}

/// Parses one or more required `WrittenText` entries.
///
/// The structural parser only validates the quoted text shape; LaTeX mode and
/// substitution semantics are handled later by the view/backend layers.
pub(in crate::frontend::structural::parser) fn parse_required_written_texts(
    section: &ProtoSection,
    tracker: &mut EventLog,
) -> Option<OneOrMore<WrittenText>> {
    let starting_issue_count = tracker.issue_count();
    let texts = parse_optional_texts(Some(section), tracker, WrittenText);
    one_or_more(texts, || {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(Some(ORIGIN), section.metadata.row, "Expected written text");
        }
    })
}

/// Parses one or more required `CalledText` entries.
///
/// Called text is plain-text rendering metadata, but at this stage it is just
/// quote-stripped and wrapped.
pub(in crate::frontend::structural::parser) fn parse_required_called_texts(
    section: &ProtoSection,
    tracker: &mut EventLog,
) -> Option<OneOrMore<CalledText>> {
    let starting_issue_count = tracker.issue_count();
    let texts = parse_optional_texts(Some(section), tracker, CalledText);
    one_or_more(texts, || {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(Some(ORIGIN), section.metadata.row, "Expected called text");
        }
    })
}

/// Parses one or more required `WritingText` entries.
pub(in crate::frontend::structural::parser) fn parse_required_writing_texts(
    section: &ProtoSection,
    tracker: &mut EventLog,
) -> Option<OneOrMore<WritingText>> {
    let starting_issue_count = tracker.issue_count();
    let texts = parse_optional_texts(Some(section), tracker, WritingText);
    one_or_more(texts, || {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(Some(ORIGIN), section.metadata.row, "Expected writing text");
        }
    })
}

/// Parses quoted text entries from an optional section and wraps them.
///
/// Inline and text arguments are accepted; formulations and nested groups are
/// diagnosed because text sections are intentionally non-formula content.
pub(in crate::frontend::structural::parser) fn parse_optional_texts<T>(
    section: Option<&ProtoSection>,
    tracker: &mut EventLog,
    wrap: fn(String) -> T,
) -> ZeroOrMore<T> {
    let Some(section) = section else {
        return ZeroOrMore::default();
    };

    let mut result = Vec::new();
    for entry in section_entries(section) {
        match entry {
            SectionEntry::Inline { text, row } | SectionEntry::Text { text, row } => {
                if let Some(value) = strip_quoted_text(text) {
                    result.push(wrap(value));
                } else {
                    tracker.user_error_at_row(
                        Some(ORIGIN),
                        row,
                        format!("Expected quoted text, found `{text}`"),
                    );
                }
            }
            SectionEntry::Formulation { row, .. } => {
                tracker.user_error_at_row(Some(ORIGIN), row, "Expected text, found formulation");
            }
            SectionEntry::Group { row, .. } => {
                tracker.user_error_at_row(Some(ORIGIN), row, "Expected text, found nested group");
            }
        }
    }

    result.into()
}

/// Converts a repeated wrapper into a nonempty wrapper or emits a caller-supplied error.
///
/// This keeps the "did a more specific error already happen?" logic at the call
/// site while centralizing the `OneOrMore` conversion.
pub(in crate::frontend::structural::parser) fn one_or_more<T>(
    items: ZeroOrMore<T>,
    on_empty: impl FnOnce(),
) -> Option<OneOrMore<T>> {
    match OneOrMore::try_from(items) {
        Ok(items) => Some(items),
        Err(_) => {
            on_empty();
            None
        }
    }
}

/// Strips one layer of double quotes from text.
///
/// Escapes are not interpreted at this layer; the current structural syntax
/// treats quoted text as the raw inner substring between the first and last
/// quote.
pub(in crate::frontend::structural::parser) fn strip_quoted_text(input: &str) -> Option<String> {
    let input = input.trim();
    let inner = input.strip_prefix('"')?.strip_suffix('"')?;
    Some(inner.to_owned())
}

// ===============================[ clauses ]=====================================

/// Parses a `not:` clause group.
///
/// The nested `not:` section must contain exactly one clause, which is boxed to
/// avoid making the recursive [`Clause`] enum infinitely sized.
pub(super) fn parse_not_clause(group: &ProtoGroup, tracker: &mut EventLog) -> Option<NotGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("not", &group.sections, tracker, &["not"])?;
    Some(NotGroup {
        heading,
        not: NotSection {
            argument: Box::new(parse_required_clause(
                section(&sections, "not")?,
                "not",
                tracker,
            )?),
        },
    })
}

/// Parses an `allOf:` clause group.
pub(super) fn parse_all_of_clause(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<AllOfGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("allOf", &group.sections, tracker, &["allOf"])?;
    Some(AllOfGroup {
        heading,
        all_of: AllOfSection {
            arguments: parse_required_clauses(section(&sections, "allOf")?, "allOf", tracker)?,
        },
    })
}

/// Parses an `anyOf:` clause group.
pub(super) fn parse_any_of_clause(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<AnyOfGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("anyOf", &group.sections, tracker, &["anyOf"])?;
    Some(AnyOfGroup {
        heading,
        any_of: AnyOfSection {
            arguments: parse_required_clauses(section(&sections, "anyOf")?, "anyOf", tracker)?,
        },
    })
}

/// Parses a `oneOf:` clause group.
pub(super) fn parse_one_of_clause(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<OneOfGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("oneOf", &group.sections, tracker, &["oneOf"])?;
    Some(OneOfGroup {
        heading,
        one_of: OneOfSection {
            arguments: parse_required_clauses(section(&sections, "oneOf")?, "oneOf", tracker)?,
        },
    })
}

/// Parses an `exists:` clause group.
///
/// The bound value is parsed as `is`/spec syntax and the required `suchThat:`
/// section supplies predicate clauses.
pub(super) fn parse_exists_clause(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ExistsGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("exists", &group.sections, tracker, &["exists", "suchThat"])?;
    Some(ExistsGroup {
        heading,
        exists: ExistsSection {
            argument: parse_required_formulation(
                section(&sections, "exists")?,
                "exists",
                tracker,
                parse_binding_or_spec,
            )?,
        },
        such_that: SuchThatSection {
            arguments: parse_required_clauses(
                section(&sections, "suchThat")?,
                "suchThat",
                tracker,
            )?,
        },
    })
}

/// Parses an `existsUnique:` clause group.
pub(super) fn parse_exists_unique_clause(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ExistsUniqueGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "existsUnique",
        &group.sections,
        tracker,
        &["existsUnique", "suchThat"],
    )?;
    Some(ExistsUniqueGroup {
        heading,
        exists_unique: ExistsUniqueSection {
            argument: parse_required_formulation(
                section(&sections, "existsUnique")?,
                "existsUnique",
                tracker,
                parse_binding_or_spec,
            )?,
        },
        such_that: SuchThatSection {
            arguments: parse_required_clauses(
                section(&sections, "suchThat")?,
                "suchThat",
                tracker,
            )?,
        },
    })
}

/// Parses a `forAll:` clause group.
///
/// The optional `where:` section acts as a guard and the required `then:`
/// section carries the quantified conclusion.
pub(super) fn parse_for_all_clause(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ForAllGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "forAll",
        &group.sections,
        tracker,
        &["forAll", "where?", "then"],
    )?;
    Some(ForAllGroup {
        heading,
        for_all: ForAllSection {
            argument: parse_required_formulation(
                section(&sections, "forAll")?,
                "forAll",
                tracker,
                parse_binding_or_spec,
            )?,
        },
        where_: sections.get("where").copied().and_then(|section| {
            parse_required_clauses(section, "where", tracker)
                .map(|arguments| WhereSection { arguments })
        }),
        then: ThenSection {
            arguments: parse_required_clauses(section(&sections, "then")?, "then", tracker)?,
        },
    })
}

/// Parses an `if:` clause group.
pub(super) fn parse_if_clause(group: &ProtoGroup, tracker: &mut EventLog) -> Option<IfGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("if", &group.sections, tracker, &["if", "then"])?;
    Some(IfGroup {
        heading,
        if_: IfSection {
            arguments: parse_required_clauses(section(&sections, "if")?, "if", tracker)?,
        },
        then: ThenSection {
            arguments: parse_required_clauses(section(&sections, "then")?, "then", tracker)?,
        },
    })
}

/// Parses an `iff:` clause group.
pub(super) fn parse_iff_clause(group: &ProtoGroup, tracker: &mut EventLog) -> Option<IffGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("iff", &group.sections, tracker, &["iff", "then"])?;
    Some(IffGroup {
        heading,
        iff: IffSection {
            arguments: parse_required_clauses(section(&sections, "iff")?, "iff", tracker)?,
        },
        then: ThenSection {
            arguments: parse_required_clauses(section(&sections, "then")?, "then", tracker)?,
        },
    })
}

/// Parses a `piecewise:` clause group.
///
/// The leading section may hold descriptive text while `if:` and `then:` are
/// required and `else:` is optional.
pub(super) fn parse_piecewise_clause(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<PiecewiseGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "piecewise",
        &group.sections,
        tracker,
        &["piecewise", "if", "then", "else?"],
    )?;
    Some(PiecewiseGroup {
        heading,
        piecewise: PiecewiseSection {
            arguments: parse_optional_open_texts(sections.get("piecewise").copied(), tracker),
        },
        if_: IfSection {
            arguments: parse_required_clauses(section(&sections, "if")?, "if", tracker)?,
        },
        then: ThenSection {
            arguments: parse_required_clauses(section(&sections, "then")?, "then", tracker)?,
        },
        else_: sections.get("else").copied().and_then(|section| {
            parse_required_clauses(section, "else", tracker)
                .map(|arguments| ElseSection { arguments })
        }),
    })
}

/// Parses a nested `given:` clause group.
///
/// This group is used inside clause lists to introduce a local assumption,
/// optional context, and required consequence.
pub(super) fn parse_given_clause(group: &ProtoGroup, tracker: &mut EventLog) -> Option<GivenGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "given",
        &group.sections,
        tracker,
        &["given", "where?", "then"],
    )?;
    Some(GivenGroup {
        heading,
        given: GivenClauseSection {
            argument: parse_required_formulation(
                section(&sections, "given")?,
                "given",
                tracker,
                parse_refined_declaration_statement,
            )?,
        },
        where_: sections.get("where").copied().and_then(|section| {
            parse_required_clauses(section, "where", tracker)
                .map(|arguments| WhereSection { arguments })
        }),
        then: ThenSection {
            arguments: parse_required_clauses(section(&sections, "then")?, "then", tracker)?,
        },
    })
}

/// Adapts an `alias:` group into an [`AliasItem`].
pub(super) fn parse_alias_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<AliasItem> {
    parse_alias_group(group, tracker).map(AliasItem::Alias)
}

/// Dispatches nested `Provides:` groups to symbol or connection parsers.
pub(super) fn parse_provides_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ProvidesItem> {
    match first_section_label(group)? {
        "symbol" => parse_symbol(group, tracker)
            .map(Box::new)
            .map(ProvidesItem::Symbol),
        "connection" => parse_connection(group, tracker).map(ProvidesItem::Connection),
        other => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Unexpected provides group `{other}`"),
            );
            None
        }
    }
}

/// Dispatches nested `Documented:` groups to documentation item parsers.
///
/// Unknown documentation group labels are reported and skipped so other
/// documentation entries in the same section can still be used.
pub(super) fn parse_documented_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<DocumentedItem> {
    match first_section_label(group)? {
        "written" => parse_written(group, tracker).map(DocumentedItem::Written),
        "called" => parse_called(group, tracker).map(DocumentedItem::Called),
        "writing" => parse_writing(group, tracker).map(DocumentedItem::Writing),
        "overview" => parse_overview(group, tracker).map(DocumentedItem::Overview),
        "related" => parse_related(group, tracker).map(DocumentedItem::Related),
        "discoverer" => parse_discoverer(group, tracker).map(DocumentedItem::Discoverer),
        other => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Unexpected documented group `{other}`"),
            );
            None
        }
    }
}

/// Dispatches nested `Justified:` groups to justification item parsers.
pub(super) fn parse_justified_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<JustifiedItem> {
    match first_section_label(group)? {
        "label" => parse_label_note(group, tracker).map(JustifiedItem::Label),
        "by" => parse_by_note(group, tracker).map(JustifiedItem::By),
        other => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Unexpected justified group `{other}`"),
            );
            None
        }
    }
}

/// Dispatches nested `Metadata:` groups to metadata item parsers.
pub(super) fn parse_metadata_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<MetadataItem> {
    match first_section_label(group)? {
        "id" => parse_id(group, tracker).map(MetadataItem::Id),
        "version" => parse_version(group, tracker).map(MetadataItem::Version),
        other => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Unexpected metadata group `{other}`"),
            );
            None
        }
    }
}

/// Dispatches nested `Specify:` groups to numeric-domain item parsers.
///
/// Positive/negative groups are distinguished by the presence of an `int:`
/// section; otherwise they are parsed as decimal specification items.
pub(super) fn parse_specify_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<SpecifyItem> {
    match first_section_label(group)? {
        "positive" => {
            if group.sections.iter().any(|section| section.label == "int") {
                parse_positive_int(group, tracker).map(SpecifyItem::PositiveInt)
            } else {
                parse_positive_decimal(group, tracker).map(SpecifyItem::PositiveDecimal)
            }
        }
        "negative" => {
            if group.sections.iter().any(|section| section.label == "int") {
                parse_negative_int(group, tracker).map(SpecifyItem::NegativeInt)
            } else {
                parse_negative_decimal(group, tracker).map(SpecifyItem::NegativeDecimal)
            }
        }
        "zero" => parse_zero(group, tracker).map(SpecifyItem::Zero),
        other => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Unexpected specify group `{other}`"),
            );
            None
        }
    }
}

/// Dispatches nested `Resource:` groups to resource field parsers.
pub(super) fn parse_resource_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ResourceItem> {
    match first_section_label(group)? {
        "title" => parse_resource_title(group, tracker).map(ResourceItem::Title),
        "author" => parse_resource_author(group, tracker).map(ResourceItem::Author),
        "offset" => parse_resource_offset(group, tracker).map(ResourceItem::Offset),
        "url" => parse_resource_url(group, tracker).map(ResourceItem::Url),
        "homepage" => parse_resource_homepage(group, tracker).map(ResourceItem::Homepage),
        "type" => parse_resource_type(group, tracker).map(ResourceItem::Type),
        "edition" => parse_resource_edition(group, tracker).map(ResourceItem::Edition),
        "editor" => parse_resource_editor(group, tracker).map(ResourceItem::Editor),
        "institution" => parse_resource_institution(group, tracker).map(ResourceItem::Institution),
        "journal" => parse_resource_journal(group, tracker).map(ResourceItem::Journal),
        "publisher" => parse_resource_publisher(group, tracker).map(ResourceItem::Publisher),
        "volume" => parse_resource_volume(group, tracker).map(ResourceItem::Volume),
        "month" => parse_resource_month(group, tracker).map(ResourceItem::Month),
        "year" => parse_resource_year(group, tracker).map(ResourceItem::Year),
        "description" => parse_resource_description(group, tracker).map(ResourceItem::Description),
        other => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Unexpected resource group `{other}`"),
            );
            None
        }
    }
}

/// Dispatches a nested clause group to the corresponding clause parser.
///
/// Inline formulation clauses are handled by [`parse_optional_clauses`]; this
/// function only handles group-shaped clauses.
pub(super) fn parse_clause_group(group: &ProtoGroup, tracker: &mut EventLog) -> Option<Clause> {
    match first_section_label(group)? {
        "not" => parse_not_clause(group, tracker).map(Clause::Not),
        "allOf" => parse_all_of_clause(group, tracker).map(Clause::AllOf),
        "anyOf" => parse_any_of_clause(group, tracker).map(Clause::AnyOf),
        "oneOf" => parse_one_of_clause(group, tracker).map(Clause::OneOf),
        "exists" => parse_exists_clause(group, tracker).map(Clause::Exists),
        "existsUnique" => parse_exists_unique_clause(group, tracker).map(Clause::ExistsUnique),
        "forAll" => parse_for_all_clause(group, tracker).map(Clause::ForAll),
        "if" => parse_if_clause(group, tracker).map(Clause::If),
        "iff" => parse_iff_clause(group, tracker).map(Clause::Iff),
        "piecewise" => parse_piecewise_clause(group, tracker).map(Clause::Piecewise),
        "given" => parse_given_clause(group, tracker).map(Clause::Given),
        other => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Unexpected clause group `{other}`"),
            );
            None
        }
    }
}

// ===============================[ nested ]=====================================

// ===============================[ metadata ]=====================================

/// Parses an `id:` metadata group.
///
/// Metadata groups do not accept headings because their meaning is determined
/// entirely by the nested section label.
pub(in crate::frontend::structural::parser) fn parse_id(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<IdGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("id", &group.sections, tracker, &["id"])?;
    Some(IdGroup {
        id: IdSection {
            argument: parse_required_open_text(section(&sections, "id")?, "id", tracker)?,
        },
    })
}

/// Parses a `version:` metadata group.
pub(in crate::frontend::structural::parser) fn parse_version(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<VersionGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("version", &group.sections, tracker, &["version"])?;
    Some(VersionGroup {
        version: VersionSection {
            argument: parse_required_open_text(section(&sections, "version")?, "version", tracker)?,
        },
    })
}

// ===============================[ documentation ]=====================================

/// Parses an `alias:` nested group.
///
/// Aliases may optionally include a label heading and a `written:` rendering
/// section.  The alias body accepts either expression or specification-operator
/// alias syntax.
pub(in crate::frontend::structural::parser) fn parse_alias_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<AliasGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("alias", &group.sections, tracker, &["alias", "written?"])?;
    Some(AliasGroup {
        heading,
        alias: AliasSection {
            argument: parse_required_formulation(
                section(&sections, "alias")?,
                "alias",
                tracker,
                parse_alias_kind,
            )?,
        },
        written: sections.get("written").copied().and_then(|section| {
            parse_required_written_texts(section, tracker)
                .map(|arguments| WrittenSection { arguments })
        }),
    })
}

/// Parses a `symbol:` nested group inside `Provides:`.
///
/// Symbols reuse alias-kind parsing because provided symbols can stand for
/// expression aliases or specification-operator aliases.
pub(in crate::frontend::structural::parser) fn parse_symbol(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<SymbolGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("symbol", &group.sections, tracker, &["symbol", "written?"])?;
    Some(SymbolGroup {
        heading,
        symbol: SymbolSection {
            argument: parse_required_formulation(
                section(&sections, "symbol")?,
                "symbol",
                tracker,
                parse_alias_kind,
            )?,
        },
        written: sections.get("written").copied().and_then(|section| {
            parse_required_written_texts(section, tracker)
                .map(|arguments| WrittenSection { arguments })
        }),
    })
}

/// Parses a `connection:` nested group inside `Provides:`.
///
/// Connection groups capture prose and optional formulation constraints that
/// describe how one documented concept connects to another.
pub(in crate::frontend::structural::parser) fn parse_connection(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ConnectionGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "connection",
        &group.sections,
        tracker,
        &[
            "connection",
            "to",
            "using?",
            "means",
            "signifies?",
            "viewable?",
            "through?",
        ],
    )?;

    Some(ConnectionGroup {
        heading,
        connection: ConnectionSection {
            arguments: parse_optional_open_texts(sections.get("connection").copied(), tracker),
        },
        to: ToSection {
            arguments: parse_optional_open_texts(sections.get("to").copied(), tracker),
        },
        using: sections.get("using").copied().and_then(|section| {
            parse_required_formulations(
                section,
                "using",
                tracker,
                parse_ordinary_declaration_statement,
            )
            .map(|arguments| UsingSection { arguments })
        }),
        means: MeansSection {
            arguments: parse_optional_open_texts(sections.get("means").copied(), tracker),
        },
        signifies: sections
            .get("signifies")
            .copied()
            .map(|section| SignifiesSection {
                arguments: parse_optional_open_texts(Some(section), tracker),
            }),
        viewable: sections
            .get("viewable")
            .copied()
            .map(|section| ViewableSection {
                arguments: parse_optional_open_texts(Some(section), tracker),
            }),
        through: sections
            .get("through")
            .copied()
            .map(|section| ThroughSection {
                arguments: parse_optional_open_texts(Some(section), tracker),
            }),
    })
}

/// Parses a `written:` documentation group.
///
/// The text entries are stored as math-mode rendering templates and validated
/// only for quoted-text shape at this structural layer.
pub(in crate::frontend::structural::parser) fn parse_written(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<WrittenGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("written", &group.sections, tracker, &["written"])?;
    Some(WrittenGroup {
        heading,
        written: WrittenSection {
            arguments: parse_required_written_texts(section(&sections, "written")?, tracker)?,
        },
    })
}

/// Parses a `called:` documentation group.
///
/// A `called:` group may bundle an optional `written:` section, which lets a
/// definition provide both prose and math-mode renderings in one nested group.
pub(in crate::frontend::structural::parser) fn parse_called(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<CalledGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("called", &group.sections, tracker, &["called", "written?"])?;
    Some(CalledGroup {
        heading,
        called: CalledSection {
            arguments: parse_required_called_texts(section(&sections, "called")?, tracker)?,
        },
        written: sections.get("written").copied().and_then(|section| {
            parse_required_written_texts(section, tracker)
                .map(|arguments| WrittenSection { arguments })
        }),
    })
}

/// Parses a `writing:` documentation group.
///
/// The `writing:` section defines the alias and the `as:` section stores the
/// quoted rendering text associated with that alias.
pub(in crate::frontend::structural::parser) fn parse_writing(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<WritingGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("writing", &group.sections, tracker, &["writing", "as"])?;
    Some(WritingGroup {
        heading,
        writing: WritingSection {
            argument: parse_required_formulation(
                section(&sections, "writing")?,
                "writing",
                tracker,
                parse_writing_alias,
            )?,
        },
        as_: AsSection {
            arguments: parse_required_writing_texts(section(&sections, "as")?, tracker)?,
        },
    })
}

/// Parses an `overview:` documentation group.
pub(in crate::frontend::structural::parser) fn parse_overview(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<OverviewGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("overview", &group.sections, tracker, &["overview"])?;
    Some(OverviewGroup {
        heading,
        overview: OverviewSection {
            argument: parse_required_open_text(
                section(&sections, "overview")?,
                "overview",
                tracker,
            )?,
        },
    })
}

/// Parses a `related:` documentation group.
///
/// Related groups require at least one quoted text entry so empty related
/// sections are reported as authoring mistakes.
pub(in crate::frontend::structural::parser) fn parse_related(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<RelatedGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("related", &group.sections, tracker, &["related"])?;
    Some(RelatedGroup {
        heading,
        related: RelatedSection {
            arguments: parse_required_open_texts(
                section(&sections, "related")?,
                "related",
                tracker,
            )?,
        },
    })
}

/// Parses a `discoverer:` documentation group.
///
/// Discoverer text is optional/open because the section may be used as a marker
/// before richer metadata is available.
pub(in crate::frontend::structural::parser) fn parse_discoverer(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<DiscovererGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("discoverer", &group.sections, tracker, &["discoverer"])?;
    Some(DiscovererGroup {
        heading,
        discoverer: DiscovererSection {
            arguments: parse_optional_open_texts(sections.get("discoverer").copied(), tracker),
        },
    })
}

// ===============================[ justification ]=====================================

/// Parses a `label:` justification note.
///
/// The note may contain optional label/by prose but must include a comment so
/// the justification has useful explanatory content.
pub(in crate::frontend::structural::parser) fn parse_label_note(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<LabelGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "label",
        &group.sections,
        tracker,
        &["label", "by", "comment"],
    )?;
    Some(LabelGroup {
        heading,
        label: LabelSection {
            arguments: parse_optional_open_texts(sections.get("label").copied(), tracker),
        },
        by: BySection {
            arguments: parse_optional_open_texts(sections.get("by").copied(), tracker),
        },
        comment: CommentSection {
            argument: parse_required_open_text(section(&sections, "comment")?, "comment", tracker)?,
        },
    })
}

/// Parses a `by:` justification note.
///
/// This mirrors label notes but starts from a `by:` section for author/source
/// oriented justification entries.
pub(in crate::frontend::structural::parser) fn parse_by_note(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ByGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("by", &group.sections, tracker, &["by", "comment"])?;
    Some(ByGroup {
        heading,
        by: BySection {
            arguments: parse_optional_open_texts(sections.get("by").copied(), tracker),
        },
        comment: CommentSection {
            argument: parse_required_open_text(section(&sections, "comment")?, "comment", tracker)?,
        },
    })
}

// ===============================[ resource_items ]=====================================

/// Parses a resource `title:` item.
pub(in crate::frontend::structural::parser) fn parse_resource_title(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ResourceTitleGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("title", &group.sections, tracker, &["title"])?;
    Some(ResourceTitleGroup {
        title: ResourceTitleSection {
            argument: parse_required_open_text(section(&sections, "title")?, "title", tracker)?,
        },
    })
}

/// Parses a resource `author:` item.
///
/// Resource authors require at least one quoted text entry.
pub(in crate::frontend::structural::parser) fn parse_resource_author(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ResourceAuthorGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("author", &group.sections, tracker, &["author"])?;
    Some(ResourceAuthorGroup {
        author: ResourceAuthorSection {
            arguments: parse_required_open_texts(section(&sections, "author")?, "author", tracker)?,
        },
    })
}

/// Parses a resource `offset:` item.
pub(in crate::frontend::structural::parser) fn parse_resource_offset(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ResourceOffsetGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("offset", &group.sections, tracker, &["offset"])?;
    Some(ResourceOffsetGroup {
        offset: ResourceOffsetSection {
            argument: parse_required_open_text(section(&sections, "offset")?, "offset", tracker)?,
        },
    })
}

/// Parses a resource `url:` item.
pub(in crate::frontend::structural::parser) fn parse_resource_url(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ResourceUrlGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("url", &group.sections, tracker, &["url"])?;
    Some(ResourceUrlGroup {
        url: ResourceUrlSection {
            argument: parse_required_open_text(section(&sections, "url")?, "url", tracker)?,
        },
    })
}

/// Parses a resource `homepage:` item.
pub(in crate::frontend::structural::parser) fn parse_resource_homepage(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ResourceHomepageGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("homepage", &group.sections, tracker, &["homepage"])?;
    Some(ResourceHomepageGroup {
        homepage: ResourceHomepageSection {
            argument: parse_required_open_text(
                section(&sections, "homepage")?,
                "homepage",
                tracker,
            )?,
        },
    })
}

/// Parses a resource `type:` item.
pub(in crate::frontend::structural::parser) fn parse_resource_type(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ResourceTypeGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("type", &group.sections, tracker, &["type"])?;
    Some(ResourceTypeGroup {
        type_: ResourceTypeSection {
            argument: parse_required_open_text(section(&sections, "type")?, "type", tracker)?,
        },
    })
}

/// Parses a resource `edition:` item.
pub(in crate::frontend::structural::parser) fn parse_resource_edition(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ResourceEditionGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("edition", &group.sections, tracker, &["edition"])?;
    Some(ResourceEditionGroup {
        edition: ResourceEditionSection {
            argument: parse_required_open_text(section(&sections, "edition")?, "edition", tracker)?,
        },
    })
}

/// Parses a resource `editor:` item.
pub(in crate::frontend::structural::parser) fn parse_resource_editor(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ResourceEditorGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("editor", &group.sections, tracker, &["editor"])?;
    Some(ResourceEditorGroup {
        editor: ResourceEditorSection {
            argument: parse_required_open_text(section(&sections, "editor")?, "editor", tracker)?,
        },
    })
}

/// Parses a resource `institution:` item.
pub(in crate::frontend::structural::parser) fn parse_resource_institution(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ResourceInstitutionGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("institution", &group.sections, tracker, &["institution"])?;
    Some(ResourceInstitutionGroup {
        institution: ResourceInstitutionSection {
            argument: parse_required_open_text(
                section(&sections, "institution")?,
                "institution",
                tracker,
            )?,
        },
    })
}

/// Parses a resource `journal:` item.
pub(in crate::frontend::structural::parser) fn parse_resource_journal(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ResourceJournalGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("journal", &group.sections, tracker, &["journal"])?;
    Some(ResourceJournalGroup {
        journal: ResourceJournalSection {
            argument: parse_required_open_text(section(&sections, "journal")?, "journal", tracker)?,
        },
    })
}

/// Parses a resource `publisher:` item.
pub(in crate::frontend::structural::parser) fn parse_resource_publisher(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ResourcePublisherGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("publisher", &group.sections, tracker, &["publisher"])?;
    Some(ResourcePublisherGroup {
        publisher: ResourcePublisherSection {
            argument: parse_required_open_text(
                section(&sections, "publisher")?,
                "publisher",
                tracker,
            )?,
        },
    })
}

/// Parses a resource `volume:` item.
pub(in crate::frontend::structural::parser) fn parse_resource_volume(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ResourceVolumeGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("volume", &group.sections, tracker, &["volume"])?;
    Some(ResourceVolumeGroup {
        volume: ResourceVolumeSection {
            argument: parse_required_open_text(section(&sections, "volume")?, "volume", tracker)?,
        },
    })
}

/// Parses a resource `month:` item.
pub(in crate::frontend::structural::parser) fn parse_resource_month(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ResourceMonthGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("month", &group.sections, tracker, &["month"])?;
    Some(ResourceMonthGroup {
        month: ResourceMonthSection {
            argument: parse_required_open_text(section(&sections, "month")?, "month", tracker)?,
        },
    })
}

/// Parses a resource `year:` item.
pub(in crate::frontend::structural::parser) fn parse_resource_year(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ResourceYearGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("year", &group.sections, tracker, &["year"])?;
    Some(ResourceYearGroup {
        year: ResourceYearSection {
            argument: parse_required_open_text(section(&sections, "year")?, "year", tracker)?,
        },
    })
}

/// Parses a resource `description:` item.
pub(in crate::frontend::structural::parser) fn parse_resource_description(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ResourceDescriptionGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("description", &group.sections, tracker, &["description"])?;
    Some(ResourceDescriptionGroup {
        description: ResourceDescriptionSection {
            argument: parse_required_open_text(
                section(&sections, "description")?,
                "description",
                tracker,
            )?,
        },
    })
}

// ===============================[ spec_items ]=====================================

/// Parses a positive-integer specification item.
///
/// The structural shape is identified by a `positive:` section together with an
/// `int:` section; all prose fields are open and may be empty.
pub(in crate::frontend::structural::parser) fn parse_positive_int(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<PositiveIntGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "positive_int",
        &group.sections,
        tracker,
        &["positive", "int", "is"],
    )?;
    Some(PositiveIntGroup {
        heading,
        positive: PositiveSection {
            arguments: parse_optional_open_texts(sections.get("positive").copied(), tracker),
        },
        int: IntSection {
            arguments: parse_optional_open_texts(sections.get("int").copied(), tracker),
        },
        is_: IsSection {
            arguments: parse_optional_open_texts(sections.get("is").copied(), tracker),
        },
    })
}

/// Parses a negative-integer specification item.
pub(in crate::frontend::structural::parser) fn parse_negative_int(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<NegativeIntGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "negative_int",
        &group.sections,
        tracker,
        &["negative", "int", "is"],
    )?;
    Some(NegativeIntGroup {
        heading,
        negative: NegativeSection {
            arguments: parse_optional_open_texts(sections.get("negative").copied(), tracker),
        },
        int: IntSection {
            arguments: parse_optional_open_texts(sections.get("int").copied(), tracker),
        },
        is_: IsSection {
            arguments: parse_optional_open_texts(sections.get("is").copied(), tracker),
        },
    })
}

/// Parses a zero specification item.
pub(in crate::frontend::structural::parser) fn parse_zero(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ZeroGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("zero", &group.sections, tracker, &["zero", "is"])?;
    Some(ZeroGroup {
        heading,
        zero: ZeroSection {
            arguments: parse_optional_open_texts(sections.get("zero").copied(), tracker),
        },
        is_: IsSection {
            arguments: parse_optional_open_texts(sections.get("is").copied(), tracker),
        },
    })
}

/// Parses a positive-decimal specification item.
pub(in crate::frontend::structural::parser) fn parse_positive_decimal(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<PositiveDecimalGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "positive_decimal",
        &group.sections,
        tracker,
        &["positive", "decimal", "is"],
    )?;
    Some(PositiveDecimalGroup {
        heading,
        positive: PositiveSection {
            arguments: parse_optional_open_texts(sections.get("positive").copied(), tracker),
        },
        decimal: DecimalSection {
            arguments: parse_optional_open_texts(sections.get("decimal").copied(), tracker),
        },
        is_: IsSection {
            arguments: parse_optional_open_texts(sections.get("is").copied(), tracker),
        },
    })
}

/// Parses a negative-decimal specification item.
pub(in crate::frontend::structural::parser) fn parse_negative_decimal(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<NegativeDecimalGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "negative_decimal",
        &group.sections,
        tracker,
        &["negative", "decimal", "is"],
    )?;
    Some(NegativeDecimalGroup {
        heading,
        negative: NegativeSection {
            arguments: parse_optional_open_texts(sections.get("negative").copied(), tracker),
        },
        decimal: DecimalSection {
            arguments: parse_optional_open_texts(sections.get("decimal").copied(), tracker),
        },
        is_: IsSection {
            arguments: parse_optional_open_texts(sections.get("is").copied(), tracker),
        },
    })
}

// ===============================[ top_level ]=====================================

// ===============================[ dispatch ]=====================================

/// Dispatches one proto group to the top-level structural parser matching its first section.
///
/// The first section label determines the group kind.  Unknown labels are
/// reported at the group start and omitted from the resulting document.
pub(in crate::frontend::structural::parser) fn parse_top_level_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<TopLevelItem> {
    let label = first_section_label(group)?;
    match label {
        "Title" => parse_title(group, tracker).map(TopLevelItem::Title),
        "Section" => parse_section(group, tracker).map(TopLevelItem::Section),
        "Subsection" => parse_subsection(group, tracker).map(TopLevelItem::Subsection),
        "Subsubsection" => parse_subsubsection(group, tracker).map(TopLevelItem::Subsubsection),
        "Describes" => parse_describes(group, tracker).map(TopLevelItem::Describes),
        "Defines" => parse_defines(group, tracker).map(TopLevelItem::Defines),
        "Refines" => parse_refines(group, tracker).map(TopLevelItem::Refines),
        "States" => parse_states(group, tracker).map(TopLevelItem::States),
        "Axiom" => parse_axiom(group, tracker).map(TopLevelItem::Axiom),
        "Theorem" => parse_theorem(group, tracker).map(TopLevelItem::Theorem),
        "Corollary" => parse_corollary(group, tracker).map(TopLevelItem::Corollary),
        "Lemma" => parse_lemma(group, tracker).map(TopLevelItem::Lemma),
        "Conjecture" => parse_conjecture(group, tracker).map(TopLevelItem::Conjecture),
        "Person" => parse_person(group, tracker).map(TopLevelItem::Person),
        "Resource" => parse_resource(group, tracker).map(TopLevelItem::Resource),
        "Specify" => parse_specify(group, tracker).map(TopLevelItem::Specify),
        other => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Unexpected top-level group `{other}`"),
            );
            None
        }
    }
}

// ===============================[ outline ]=====================================

/// Parses a top-level `Title:` group.
///
/// Title groups cannot have bracket headings and must contain exactly the
/// `Title:` section shape.
pub(in crate::frontend::structural::parser) fn parse_title(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<TitleGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("Title", &group.sections, tracker, &["Title"])?;
    Some(TitleGroup {
        title: TitleSection {
            argument: parse_required_open_text(section(&sections, "Title")?, "Title", tracker)?,
        },
    })
}

/// Parses a top-level `Section:` group.
///
/// This represents a first-level document outline heading rather than a
/// definition or theorem block.
pub(in crate::frontend::structural::parser) fn parse_section(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<SectionGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("Section", &group.sections, tracker, &["Section"])?;
    Some(SectionGroup {
        section: SectionSection {
            argument: parse_required_open_text(section(&sections, "Section")?, "Section", tracker)?,
        },
    })
}

/// Parses a top-level `Subsection:` group.
///
/// Subsections share the simple outline shape with `Section:` but carry their
/// own wrapper so rendering can preserve hierarchy.
pub(in crate::frontend::structural::parser) fn parse_subsection(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<SubsectionGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("Subsection", &group.sections, tracker, &["Subsection"])?;
    Some(SubsectionGroup {
        subsection: SubsectionSection {
            argument: parse_required_open_text(
                section(&sections, "Subsection")?,
                "Subsection",
                tracker,
            )?,
        },
    })
}

/// Parses a top-level `Subsubsection:` group.
///
/// The parser enforces the expected section label and rejects accidental command
/// headings on outline-only groups.
pub(in crate::frontend::structural::parser) fn parse_subsubsection(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<SubsubsectionGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections(
        "Subsubsection",
        &group.sections,
        tracker,
        &["Subsubsection"],
    )?;
    Some(SubsubsectionGroup {
        subsubsection: SubsubsectionSection {
            argument: parse_required_open_text(
                section(&sections, "Subsubsection")?,
                "Subsubsection",
                tracker,
            )?,
        },
    })
}

// ===============================[ definitions ]=====================================

/// Parses a command-backed `Describes:` group.
///
/// This enforces the full `Describes` section order and converts each optional
/// nested section into its typed representation.  Formulation sections are
/// delegated to the formulation parser while clause/nested sections recurse
/// through structural helpers.
pub(in crate::frontend::structural::parser) fn parse_describes(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<DescribesGroup> {
    let heading = parse_required_command_heading(group, tracker)?;
    let sections = identify_sections(
        "Describes",
        &group.sections,
        tracker,
        &[
            "Describes",
            "using?",
            "when?",
            "extends?",
            "specifies?",
            "satisfies?",
            "Provides?",
            "Justified?",
            "Documented?",
            "Aliases?",
            "References?",
            "Metadata?",
        ],
    )?;

    Some(DescribesGroup {
        heading,
        describes: DescribesSection {
            argument: parse_required_formulation(
                section(&sections, "Describes")?,
                "Describes",
                tracker,
                parse_form_or_declaration,
            )?,
        },
        using: sections.get("using").copied().and_then(|section| {
            parse_required_formulations(
                section,
                "using",
                tracker,
                parse_ordinary_declaration_statement,
            )
            .map(|arguments| UsingSection { arguments })
        }),
        when: sections.get("when").copied().and_then(|section| {
            parse_required_clauses(section, "when", tracker)
                .map(|arguments| WhenSection { arguments })
        }),
        extends: sections.get("extends").copied().and_then(|section| {
            parse_required_formulation(section, "extends", tracker, parse_is_or_via_item)
                .map(|argument| ExtendsSection { argument })
        }),
        specifies: sections.get("specifies").copied().and_then(|section| {
            parse_required_formulations(section, "specifies", tracker, parse_is_or_via_item)
                .map(|arguments| DescribesSpecifiesSection { arguments })
        }),
        satisfies: sections.get("satisfies").copied().and_then(|section| {
            parse_required_clauses(section, "satisfies", tracker)
                .map(|arguments| SatisfiesSection { arguments })
        }),
        provides: sections.get("Provides").copied().and_then(|section| {
            parse_required_groups(section, "Provides", tracker, parse_provides_item_group)
                .map(|arguments| ProvidesSection { arguments })
        }),
        justified: sections.get("Justified").copied().and_then(|section| {
            parse_required_groups(section, "Justified", tracker, parse_justified_item_group)
                .map(|arguments| JustifiedSection { arguments })
        }),
        documented: sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(section, "Documented", tracker, parse_documented_item_group)
                .map(|arguments| DocumentedSection { arguments })
        }),
        aliases: sections.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", tracker, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        references: sections.get("References").copied().and_then(|section| {
            parse_required_formulations(section, "References", tracker, parse_resource_header)
                .map(|arguments| ReferencesSection { arguments })
        }),
        metadata: sections.get("Metadata").copied().and_then(|section| {
            parse_required_groups(section, "Metadata", tracker, parse_metadata_item_group)
                .map(|arguments| MetadataSection { arguments })
        }),
    })
}

/// Parses a command-backed `Defines:` group.
///
/// `Defines` groups introduce command signatures for specification/type-like
/// statements and support the same auxiliary sections as `Describes`, except
/// for the `expresses:` clause in place of form-specific sections.
pub(in crate::frontend::structural::parser) fn parse_defines(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<DefinesGroup> {
    let heading = parse_required_command_heading(group, tracker)?;
    let sections = identify_sections(
        "Defines",
        &group.sections,
        tracker,
        &[
            "Defines",
            "using?",
            "when?",
            "expresses?",
            "Provides?",
            "Justified?",
            "Documented?",
            "Aliases?",
            "References?",
            "Metadata?",
        ],
    )?;

    Some(DefinesGroup {
        heading,
        defines: DefinesSection {
            argument: parse_required_formulation(
                section(&sections, "Defines")?,
                "Defines",
                tracker,
                parse_ordinary_declaration_statement,
            )?,
        },
        using: sections.get("using").copied().and_then(|section| {
            parse_required_formulations(
                section,
                "using",
                tracker,
                parse_ordinary_declaration_statement,
            )
            .map(|arguments| UsingSection { arguments })
        }),
        when: sections.get("when").copied().and_then(|section| {
            parse_required_clauses(section, "when", tracker)
                .map(|arguments| WhenSection { arguments })
        }),
        expresses: sections.get("expresses").copied().and_then(|section| {
            parse_required_clause(section, "expresses", tracker)
                .map(|argument| ExpressesSection { argument })
        }),
        provides: sections.get("Provides").copied().and_then(|section| {
            parse_required_groups(section, "Provides", tracker, parse_provides_item_group)
                .map(|arguments| ProvidesSection { arguments })
        }),
        justified: sections.get("Justified").copied().and_then(|section| {
            parse_required_groups(section, "Justified", tracker, parse_justified_item_group)
                .map(|arguments| JustifiedSection { arguments })
        }),
        documented: sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(section, "Documented", tracker, parse_documented_item_group)
                .map(|arguments| DocumentedSection { arguments })
        }),
        aliases: sections.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", tracker, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        references: sections.get("References").copied().and_then(|section| {
            parse_required_formulations(section, "References", tracker, parse_resource_header)
                .map(|arguments| ReferencesSection { arguments })
        }),
        metadata: sections.get("Metadata").copied().and_then(|section| {
            parse_required_groups(section, "Metadata", tracker, parse_metadata_item_group)
                .map(|arguments| MetadataSection { arguments })
        }),
    })
}

/// Parses a command-backed `Refines:` group.
///
/// Refines groups define a refined command signature and validate their
/// `Refines:`/`specifies:` bodies with the parser variant that accepts refined
/// command references.
pub(in crate::frontend::structural::parser) fn parse_refines(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<RefinesGroup> {
    let heading = parse_required_command_heading(group, tracker)?;
    let sections = identify_sections(
        "Refines",
        &group.sections,
        tracker,
        &[
            "Refines",
            "using?",
            "when?",
            "specifies?",
            "satisfies?",
            "Provides?",
            "Justified?",
            "Documented?",
            "Aliases?",
            "References?",
            "Metadata?",
        ],
    )?;

    Some(RefinesGroup {
        heading,
        refines: RefinesSection {
            argument: parse_required_formulation(
                section(&sections, "Refines")?,
                "Refines",
                tracker,
                parse_refined_declaration_statement,
            )?,
        },
        using: sections.get("using").copied().and_then(|section| {
            parse_required_formulations(
                section,
                "using",
                tracker,
                parse_ordinary_declaration_statement,
            )
            .map(|arguments| UsingSection { arguments })
        }),
        when: sections.get("when").copied().and_then(|section| {
            parse_required_clauses(section, "when", tracker)
                .map(|arguments| WhenSection { arguments })
        }),
        specifies: sections.get("specifies").copied().and_then(|section| {
            parse_required_formulation(
                section,
                "specifies",
                tracker,
                parse_refined_declaration_statement,
            )
            .map(|argument| RefinesSpecifiesSection { argument })
        }),
        satisfies: sections.get("satisfies").copied().and_then(|section| {
            parse_required_clauses(section, "satisfies", tracker)
                .map(|arguments| SatisfiesSection { arguments })
        }),
        provides: sections.get("Provides").copied().and_then(|section| {
            parse_required_groups(section, "Provides", tracker, parse_provides_item_group)
                .map(|arguments| ProvidesSection { arguments })
        }),
        justified: sections.get("Justified").copied().and_then(|section| {
            parse_required_groups(section, "Justified", tracker, parse_justified_item_group)
                .map(|arguments| JustifiedSection { arguments })
        }),
        documented: sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(section, "Documented", tracker, parse_documented_item_group)
                .map(|arguments| DocumentedSection { arguments })
        }),
        aliases: sections.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", tracker, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        references: sections.get("References").copied().and_then(|section| {
            parse_required_formulations(section, "References", tracker, parse_resource_header)
                .map(|arguments| ReferencesSection { arguments })
        }),
        metadata: sections.get("Metadata").copied().and_then(|section| {
            parse_required_groups(section, "Metadata", tracker, parse_metadata_item_group)
                .map(|arguments| MetadataSection { arguments })
        }),
    })
}

/// Parses a command-backed `States:` group.
///
/// The `that:` section is required and supplies the statement body.  Optional
/// prose in `States:` is retained for documentation/rendering contexts.
pub(in crate::frontend::structural::parser) fn parse_states(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<StatesGroup> {
    let heading = parse_required_command_heading(group, tracker)?;
    let sections = identify_sections(
        "States",
        &group.sections,
        tracker,
        &[
            "States",
            "using?",
            "when?",
            "that",
            "Provides?",
            "Justified?",
            "Documented?",
            "Aliases?",
            "References?",
            "Metadata?",
        ],
    )?;

    Some(StatesGroup {
        heading,
        states: StatesSection {
            arguments: parse_optional_open_texts(sections.get("States").copied(), tracker),
        },
        using: sections.get("using").copied().and_then(|section| {
            parse_required_formulations(
                section,
                "using",
                tracker,
                parse_ordinary_declaration_statement,
            )
            .map(|arguments| UsingSection { arguments })
        }),
        when: sections.get("when").copied().and_then(|section| {
            parse_required_clauses(section, "when", tracker)
                .map(|arguments| WhenSection { arguments })
        }),
        that: ThatSection {
            arguments: parse_required_clauses(section(&sections, "that")?, "that", tracker)?,
        },
        provides: sections.get("Provides").copied().and_then(|section| {
            parse_required_groups(section, "Provides", tracker, parse_provides_item_group)
                .map(|arguments| ProvidesSection { arguments })
        }),
        justified: sections.get("Justified").copied().and_then(|section| {
            parse_required_groups(section, "Justified", tracker, parse_justified_item_group)
                .map(|arguments| JustifiedSection { arguments })
        }),
        documented: sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(section, "Documented", tracker, parse_documented_item_group)
                .map(|arguments| DocumentedSection { arguments })
        }),
        aliases: sections.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", tracker, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        references: sections.get("References").copied().and_then(|section| {
            parse_required_formulations(section, "References", tracker, parse_resource_header)
                .map(|arguments| ReferencesSection { arguments })
        }),
        metadata: sections.get("Metadata").copied().and_then(|section| {
            parse_required_groups(section, "Metadata", tracker, parse_metadata_item_group)
                .map(|arguments| MetadataSection { arguments })
        }),
    })
}

// ===============================[ theorems ]=====================================

/// Parses an `Axiom:` group using the shared theorem-like parser.
///
/// The returned shared tuple is adapted to the axiom-specific section wrapper.
pub(in crate::frontend::structural::parser) fn parse_axiom(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<AxiomGroup> {
    parse_argument_theorem_like(group, tracker, "Axiom").map(
        |(
            heading,
            text,
            given,
            where_,
            then,
            iff,
            justified,
            documented,
            aliases,
            references,
            metadata,
        )| {
            AxiomGroup {
                heading,
                axiom: AxiomSection { arguments: text },
                given,
                where_,
                then,
                iff,
                justified,
                documented,
                aliases,
                references,
                metadata,
            }
        },
    )
}

/// Parses a `Theorem:` group using the shared theorem-like parser.
pub(in crate::frontend::structural::parser) fn parse_theorem(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<TheoremGroup> {
    parse_argument_theorem_like(group, tracker, "Theorem").map(
        |(
            heading,
            text,
            given,
            where_,
            then,
            iff,
            justified,
            documented,
            aliases,
            references,
            metadata,
        )| {
            TheoremGroup {
                heading,
                theorem: TheoremSection { arguments: text },
                given,
                where_,
                then,
                iff,
                justified,
                documented,
                aliases,
                references,
                metadata,
            }
        },
    )
}

/// Parses a `Lemma:` group using the shared theorem-like parser.
pub(in crate::frontend::structural::parser) fn parse_lemma(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<LemmaGroup> {
    parse_argument_theorem_like(group, tracker, "Lemma").map(
        |(
            heading,
            text,
            given,
            where_,
            then,
            iff,
            justified,
            documented,
            aliases,
            references,
            metadata,
        )| {
            LemmaGroup {
                heading,
                lemma: LemmaSection { arguments: text },
                given,
                where_,
                then,
                iff,
                justified,
                documented,
                aliases,
                references,
                metadata,
            }
        },
    )
}

/// Parses a `Conjecture:` group using the shared theorem-like parser.
pub(in crate::frontend::structural::parser) fn parse_conjecture(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ConjectureGroup> {
    parse_argument_theorem_like(group, tracker, "Conjecture").map(
        |(
            heading,
            text,
            given,
            where_,
            then,
            iff,
            justified,
            documented,
            aliases,
            references,
            metadata,
        )| {
            ConjectureGroup {
                heading,
                conjecture: ConjectureSection { arguments: text },
                given,
                where_,
                then,
                iff,
                justified,
                documented,
                aliases,
                references,
                metadata,
            }
        },
    )
}

#[allow(clippy::type_complexity)]
/// Parses the common shape shared by axiom/theorem/lemma/conjecture groups.
///
/// These groups all allow optional command headings, optional prose on their
/// primary section, optional assumptions/context, a required `then:` section,
/// and the same auxiliary documentation/reference sections.  Returning a tuple
/// keeps each concrete wrapper parser small while preserving exact section
/// types at the call site.
pub(in crate::frontend::structural::parser) fn parse_argument_theorem_like(
    group: &ProtoGroup,
    tracker: &mut EventLog,
    name: &str,
) -> Option<(
    Option<crate::frontend::formulation::ast::CommandHeader>,
    ZeroOrMore<OpenText>,
    Option<GivenSection>,
    Option<WhereSection>,
    ThenSection,
    Option<IffSection>,
    Option<JustifiedSection>,
    Option<DocumentedSection>,
    Option<AliasesSection>,
    Option<ReferencesSection>,
    Option<MetadataSection>,
)> {
    let heading = parse_optional_command_heading(group, tracker)?;
    let section_name = name;
    let sections = identify_sections(
        name,
        &group.sections,
        tracker,
        &[
            section_name,
            "given?",
            "where?",
            "then",
            "iff?",
            "Justified?",
            "Documented?",
            "Aliases?",
            "References?",
            "Metadata?",
        ],
    )?;

    Some((
        heading,
        parse_optional_open_texts(sections.get(section_name).copied(), tracker),
        sections.get("given").copied().and_then(|section| {
            parse_required_formulations(
                section,
                "given",
                tracker,
                parse_refined_declaration_statement,
            )
            .map(|arguments| GivenSection { arguments })
        }),
        sections.get("where").copied().and_then(|section| {
            parse_required_clauses(section, "where", tracker)
                .map(|arguments| WhereSection { arguments })
        }),
        ThenSection {
            arguments: parse_required_clauses(section(&sections, "then")?, "then", tracker)?,
        },
        sections.get("iff").copied().and_then(|section| {
            parse_required_clauses(section, "iff", tracker)
                .map(|arguments| IffSection { arguments })
        }),
        sections.get("Justified").copied().and_then(|section| {
            parse_required_groups(section, "Justified", tracker, parse_justified_item_group)
                .map(|arguments| JustifiedSection { arguments })
        }),
        sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(section, "Documented", tracker, parse_documented_item_group)
                .map(|arguments| DocumentedSection { arguments })
        }),
        sections.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", tracker, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        sections.get("References").copied().and_then(|section| {
            parse_required_formulations(section, "References", tracker, parse_resource_header)
                .map(|arguments| ReferencesSection { arguments })
        }),
        sections.get("Metadata").copied().and_then(|section| {
            parse_required_groups(section, "Metadata", tracker, parse_metadata_item_group)
                .map(|arguments| MetadataSection { arguments })
        }),
    ))
}

/// Parses a `Corollary:` group.
///
/// Corollaries mostly share theorem-like structure but additionally require an
/// `of:` section that records what theorem or statement they follow from.
pub(in crate::frontend::structural::parser) fn parse_corollary(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<CorollaryGroup> {
    let heading = parse_optional_command_heading(group, tracker)?;
    let sections = identify_sections(
        "Corollary",
        &group.sections,
        tracker,
        &[
            "Corollary",
            "of",
            "given?",
            "where?",
            "then",
            "iff?",
            "Justified?",
            "Documented?",
            "Aliases?",
            "References?",
            "Metadata?",
        ],
    )?;

    Some(CorollaryGroup {
        heading,
        corollary: CorollarySection {
            arguments: parse_optional_open_texts(sections.get("Corollary").copied(), tracker),
        },
        of: OfSection {
            arguments: parse_optional_open_texts(sections.get("of").copied(), tracker),
        },
        given: sections.get("given").copied().and_then(|section| {
            parse_required_formulations(
                section,
                "given",
                tracker,
                parse_refined_declaration_statement,
            )
            .map(|arguments| GivenSection { arguments })
        }),
        where_: sections.get("where").copied().and_then(|section| {
            parse_required_clauses(section, "where", tracker)
                .map(|arguments| WhereSection { arguments })
        }),
        then: ThenSection {
            arguments: parse_required_clauses(section(&sections, "then")?, "then", tracker)?,
        },
        iff: sections.get("iff").copied().and_then(|section| {
            parse_required_clauses(section, "iff", tracker)
                .map(|arguments| IffSection { arguments })
        }),
        justified: sections.get("Justified").copied().and_then(|section| {
            parse_required_groups(section, "Justified", tracker, parse_justified_item_group)
                .map(|arguments| JustifiedSection { arguments })
        }),
        documented: sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(section, "Documented", tracker, parse_documented_item_group)
                .map(|arguments| DocumentedSection { arguments })
        }),
        aliases: sections.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", tracker, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        references: sections.get("References").copied().and_then(|section| {
            parse_required_formulations(section, "References", tracker, parse_resource_header)
                .map(|arguments| ReferencesSection { arguments })
        }),
        metadata: sections.get("Metadata").copied().and_then(|section| {
            parse_required_groups(section, "Metadata", tracker, parse_metadata_item_group)
                .map(|arguments| MetadataSection { arguments })
        }),
    })
}

// ===============================[ metadata ]=====================================

/// Parses a `Person:` metadata group.
///
/// Person groups require an author-style heading and carry required `name:` and
/// `biography:` sections, with optional prose on the leading `Person:` section.
pub(in crate::frontend::structural::parser) fn parse_person(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<PersonGroup> {
    let heading = parse_required_author_heading(group, tracker)?;
    let sections = identify_sections(
        "Person",
        &group.sections,
        tracker,
        &["Person", "name", "biography"],
    )?;

    Some(PersonGroup {
        heading,
        person: PersonSection {
            arguments: parse_optional_open_texts(sections.get("Person").copied(), tracker),
        },
        name: NameSection {
            arguments: parse_required_open_texts(section(&sections, "name")?, "name", tracker)?,
        },
        biography: BiographySection {
            argument: parse_required_open_text(
                section(&sections, "biography")?,
                "biography",
                tracker,
            )?,
        },
    })
}

/// Parses a `Resource:` metadata group.
///
/// Resource groups require a resource heading and then delegate each nested
/// resource field to [`parse_resource_item_group`].
pub(in crate::frontend::structural::parser) fn parse_resource(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ResourceGroup> {
    let heading = parse_required_resource_heading(group, tracker)?;
    let sections = identify_sections("Resource", &group.sections, tracker, &["Resource"])?;

    Some(ResourceGroup {
        heading,
        resource: ResourceSection {
            arguments: parse_required_groups(
                section(&sections, "Resource")?,
                "Resource",
                tracker,
                parse_resource_item_group,
            )?,
        },
    })
}

/// Parses a top-level `Specify:` group.
///
/// Specify groups do not take headings and contain nested numeric-domain
/// specification items.
pub(in crate::frontend::structural::parser) fn parse_specify(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<SpecifyGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("Specify", &group.sections, tracker, &["Specify"])?;
    Some(SpecifyGroup {
        specify: SpecifySection {
            arguments: parse_required_groups(
                section(&sections, "Specify")?,
                "Specify",
                tracker,
                parse_specify_item_group,
            )?,
        },
    })
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    //! Integration tests for the structural parser.
    //!
    //! These tests exercise [`parse_document`] end-to-end and assert on the
    //! resulting [`Document`] or on diagnostic events emitted via the event log.

    use std::collections::BTreeSet;
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::parse_document;
    use crate::events::{Event, EventLog};
    use crate::frontend::structural::ast::{
        AliasItem, AliasKind, Clause, Document, DocumentedItem, IsOrViaItem, JustifiedItem,
        MetadataItem, ProvidesItem, ResourceItem, SpecifyItem, TopLevelItem,
    };

    fn split_test_chunks(text: &str) -> Vec<String> {
        text.replace("\r\n", "\n")
            .split("\n\n")
            .filter_map(|entry| {
                let entry = entry.trim();
                (!entry.is_empty()).then(|| entry.to_owned())
            })
            .collect()
    }

    fn read_test_chunks(path: &Path) -> Vec<String> {
        let text = fs::read_to_string(path).unwrap_or_else(|error| {
            panic!(
                "expected structural golden file {}: {error}",
                path.display()
            )
        });
        split_test_chunks(&text)
    }

    fn read_test_files(directory: &Path, extension: &str) -> Vec<PathBuf> {
        let mut files = fs::read_dir(directory)
            .unwrap_or_else(|error| panic!("expected directory {}: {error}", directory.display()))
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().and_then(|value| value.to_str()) == Some(extension))
            .collect::<Vec<_>>();
        files.sort();
        files
    }

    fn file_name(path: &Path) -> String {
        path.file_name()
            .and_then(|value| value.to_str())
            .expect("expected valid utf-8 file name")
            .to_owned()
    }

    fn parse_ok(text: &str) -> Document {
        let mut tracker = EventLog::new();
        let document = parse_document(text, &mut tracker);

        assert!(!tracker.has_errors(), "{:#?}", tracker.events());

        document
    }

    fn parse_with_diagnostics(text: &str) -> (Document, Vec<Event>) {
        let mut tracker = EventLog::new();
        let document = parse_document(text, &mut tracker);
        let messages = tracker.events().to_vec();

        (document, messages)
    }

    // ===============================[ definitions ]=====================================

    #[test]
    fn parses_definition_like_groups_with_nested_sections_and_items() {
        let document = parse_ok(
            r#"
[\structure]
Describes: S ::= (X, *)
using:
. X is \set
. X "contains" Element
when:
. [logic.when]
  allOf:
  . x = x
  . y = y
extends: X is \set via (X, Y)
specifies:
. Y is \set via (X, Y)
. y "contains" Y
satisfies:
. [logic.satisfies]
  not:
  . x = y
Provides:
. [symbol.plus]
  symbol: plus(x_, y_) :=> x + y
  written:
  . "+"
Justified:
. [proof.label]
  label:
  . "Closure"
  by:
  . "Definition"
  comment: "standard"
Documented:
. [docs.written]
  written:
  . "plus"
Aliases:
. [alias.expr]
  alias: plus(x_, y_) :=> x + y
  written:
  . "+"
References:
. $book.plus
Metadata:
. id: "desc-1"

[\structure.connection]
Describes: T
Provides:
. [conn.plus]
  connection:
  . "addition"
  to:
  . "binary operation"
  using:
  . X is \set
  means:
  . "adds elements"
  signifies:
  . "closure"
  viewable:
  . "as a table"
  through:
  . "worked examples"
Justified:
. [proof.by]
  by:
  . "Convention"
  comment: "accepted"
Documented:
. [docs.called]
  called:
  . "addition"
Aliases:
. [alias.spec]
  alias: x_ "in" X :-> x is \element
Metadata:
. version: "1.0"

[\structure.writing]
Describes: W
Documented:
. [docs.writing]
  writing: plus(x_, y_) :~> x + y
  as:
  . "inline notation"

[\structure.overview]
Describes: O
Documented:
. [docs.overview]
  overview: "Binary operation on X"

[\structure.related]
Describes: R
Documented:
. [docs.related]
  related:
  . "group"
  . "ring"

[\structure.discoverer]
Describes: D
Documented:
. [docs.discoverer]
  discoverer:
  . "Gauss"

[\constant]
Defines: zero is \element
using:
. X is \set
expresses:
. [logic.expr]
  piecewise:
  . "choose a representative"
  if:
  . x = x
  then:
  . x = x
  else:
  . y = y

[\transform]
Refines: x is \(f)::[[g]]
using:
. X is \set
when:
. [logic.exists]
  existsUnique: x is \element
  suchThat:
  . x = x
specifies: y is \(f)::[[g]]
satisfies:
. [logic.given]
  given: x is \element
  where:
  . x = x
  then:
  . y = y

[\statement]
States:
. "Closure law"
. "Associativity"
using:
. X is \set
that:
. [logic.exists]
  exists: y is \element
  suchThat:
  . y = y

[\statement.expr]
States:
that:
. y = y
"#,
        );

        assert_eq!(document.items.len(), 10);

        match &document.items[0] {
            TopLevelItem::Describes(group) => {
                assert_eq!(
                    group
                        .using
                        .as_ref()
                        .expect("expected using")
                        .arguments
                        .len(),
                    2
                );
                assert!(matches!(
                    group.when.as_ref().expect("expected when").arguments[0],
                    Clause::AllOf(_)
                ));
                assert!(matches!(
                    group.extends.as_ref().expect("expected extends").argument,
                    IsOrViaItem::IsVia(_)
                ));
                assert_eq!(
                    group
                        .specifies
                        .as_ref()
                        .expect("expected specifies")
                        .arguments
                        .len(),
                    2
                );
                assert!(matches!(
                    group
                        .satisfies
                        .as_ref()
                        .expect("expected satisfies")
                        .arguments[0],
                    Clause::Not(_)
                ));
                assert!(matches!(
                    group
                        .provides
                        .as_ref()
                        .expect("expected provides")
                        .arguments[0],
                    ProvidesItem::Symbol(_)
                ));
                assert!(matches!(
                    group
                        .justified
                        .as_ref()
                        .expect("expected justified")
                        .arguments[0],
                    JustifiedItem::Label(_)
                ));
                assert!(matches!(
                    group
                        .documented
                        .as_ref()
                        .expect("expected documented")
                        .arguments[0],
                    DocumentedItem::Written(_)
                ));
                match &group.aliases.as_ref().expect("expected aliases").arguments[0] {
                    AliasItem::Alias(alias) => {
                        assert!(matches!(alias.alias.argument, AliasKind::Expression(_)))
                    }
                }
                assert!(matches!(
                    group
                        .metadata
                        .as_ref()
                        .expect("expected metadata")
                        .arguments[0],
                    MetadataItem::Id(_)
                ));
            }
            other => panic!("expected describes group, got {other:?}"),
        }

        match &document.items[1] {
            TopLevelItem::Describes(group) => {
                assert!(matches!(
                    group
                        .provides
                        .as_ref()
                        .expect("expected provides")
                        .arguments[0],
                    ProvidesItem::Connection(_)
                ));
                assert!(matches!(
                    group
                        .justified
                        .as_ref()
                        .expect("expected justified")
                        .arguments[0],
                    JustifiedItem::By(_)
                ));
                assert!(matches!(
                    group
                        .documented
                        .as_ref()
                        .expect("expected documented")
                        .arguments[0],
                    DocumentedItem::Called(_)
                ));
                match &group.aliases.as_ref().expect("expected aliases").arguments[0] {
                    AliasItem::Alias(alias) => {
                        assert!(matches!(alias.alias.argument, AliasKind::SpecOperator(_)))
                    }
                }
                assert!(matches!(
                    group
                        .metadata
                        .as_ref()
                        .expect("expected metadata")
                        .arguments[0],
                    MetadataItem::Version(_)
                ));
            }
            other => panic!("expected describes group, got {other:?}"),
        }

        match &document.items[2] {
            TopLevelItem::Describes(group) => {
                assert!(matches!(
                    group
                        .documented
                        .as_ref()
                        .expect("expected documented")
                        .arguments[0],
                    DocumentedItem::Writing(_)
                ));
            }
            other => panic!("expected describes group, got {other:?}"),
        }

        match &document.items[3] {
            TopLevelItem::Describes(group) => {
                assert!(matches!(
                    group
                        .documented
                        .as_ref()
                        .expect("expected documented")
                        .arguments[0],
                    DocumentedItem::Overview(_)
                ));
            }
            other => panic!("expected describes group, got {other:?}"),
        }

        match &document.items[4] {
            TopLevelItem::Describes(group) => {
                assert!(matches!(
                    group
                        .documented
                        .as_ref()
                        .expect("expected documented")
                        .arguments[0],
                    DocumentedItem::Related(_)
                ));
            }
            other => panic!("expected describes group, got {other:?}"),
        }

        match &document.items[5] {
            TopLevelItem::Describes(group) => {
                assert!(matches!(
                    group
                        .documented
                        .as_ref()
                        .expect("expected documented")
                        .arguments[0],
                    DocumentedItem::Discoverer(_)
                ));
            }
            other => panic!("expected describes group, got {other:?}"),
        }

        match &document.items[6] {
            TopLevelItem::Defines(group) => {
                assert!(matches!(
                    group
                        .expresses
                        .as_ref()
                        .expect("expected expresses")
                        .argument,
                    Clause::Piecewise(_)
                ));
            }
            other => panic!("expected defines group, got {other:?}"),
        }

        match &document.items[7] {
            TopLevelItem::Refines(group) => {
                assert!(matches!(
                    &group.refines.argument.relation,
                    Some(crate::frontend::formulation::ast::DeclarationRelation::Is(
                        _
                    ))
                ));
                assert!(group.specifies.is_some());
                assert!(matches!(
                    group.when.as_ref().expect("expected when").arguments[0],
                    Clause::ExistsUnique(_)
                ));
                assert!(matches!(
                    group
                        .satisfies
                        .as_ref()
                        .expect("expected satisfies")
                        .arguments[0],
                    Clause::Given(_)
                ));
            }
            other => panic!("expected refines group, got {other:?}"),
        }

        match &document.items[8] {
            TopLevelItem::States(group) => {
                assert_eq!(group.states.arguments.len(), 2);
                assert_eq!(group.states.arguments[0].0, "Closure law");
                assert_eq!(group.states.arguments[1].0, "Associativity");
                assert!(matches!(group.that.arguments[0], Clause::Exists(_)));
            }
            other => panic!("expected states group, got {other:?}"),
        }

        match &document.items[9] {
            TopLevelItem::States(group) => {
                assert!(matches!(group.that.arguments[0], Clause::Expression(_)));
            }
            other => panic!("expected states group, got {other:?}"),
        }
    }

    #[test]
    fn parses_provided_symbol_with_builtin_spec_operator_target() {
        let document = parse_ok(
            r#"
[\set]
Describes: X
Provides:
. symbol: x_ "in" X :-> \\abstract
Documented:
. called: "set"
"#,
        );

        let TopLevelItem::Describes(group) = &document.items[0] else {
            panic!("expected describes group");
        };

        assert!(matches!(
            group
                .provides
                .as_ref()
                .expect("expected provides")
                .arguments[0],
            ProvidesItem::Symbol(_)
        ));
    }

    // ===============================[ diagnostics ]=====================================

    #[test]
    fn reports_section_order_errors_and_recovers() {
        let (document, diagnostics) = parse_with_diagnostics(
            r#"
[\statement]
States:
References:
. $bad.ref
that:
. x = x

Section: "Recovered"
"#,
        );

        assert_eq!(document.items.len(), 1);
        assert!(matches!(document.items[0], TopLevelItem::Section(_)));
        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics[0]
                .as_message()
                .expect("expected message event")
                .message
                .contains("Expected `that` but found `References`")
        );
    }

    #[test]
    fn parses_structural_golden_directory() {
        let directory = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/goldens/structural"));
        let files = read_test_files(directory, "text");
        let expected_names = BTreeSet::from([
            "axioms.text".to_owned(),
            "conjectures.text".to_owned(),
            "corollaries.text".to_owned(),
            "defines.text".to_owned(),
            "describes.text".to_owned(),
            "lemmas.text".to_owned(),
            "outline.text".to_owned(),
            "persons.text".to_owned(),
            "refines.text".to_owned(),
            "resources.text".to_owned(),
            "specify.text".to_owned(),
            "states.text".to_owned(),
            "theorems.text".to_owned(),
        ]);

        assert!(!files.is_empty(), "expected structural golden files");

        let actual_names = files
            .iter()
            .map(|path| file_name(path))
            .collect::<BTreeSet<_>>();
        assert_eq!(
            actual_names, expected_names,
            "unexpected structural golden files"
        );

        for path in files {
            let name = file_name(&path);
            let entries = read_test_chunks(&path);

            assert!(!entries.is_empty(), "expected cases in {}", path.display());

            for (index, entry) in entries.iter().enumerate() {
                let mut tracker = EventLog::new();
                let parse_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    parse_document(entry, &mut tracker)
                }));

                if let Err(payload) = parse_result {
                    let message = if let Some(message) = payload.downcast_ref::<&str>() {
                        *message
                    } else if let Some(message) = payload.downcast_ref::<String>() {
                        message.as_str()
                    } else {
                        "unknown panic"
                    };
                    panic!(
                        "structural golden case {} chunk {} panicked: {}\n\n{}",
                        name,
                        index + 1,
                        message,
                        entry
                    );
                }

                assert!(
                    !tracker.has_errors(),
                    "failed to parse structural golden case {} chunk {}:\n{}\n\n{:#?}",
                    name,
                    index + 1,
                    entry,
                    tracker.events()
                );
            }
        }
    }

    // ===============================[ metadata ]=====================================

    #[test]
    fn parses_person_resource_and_specify_variants() {
        let document = parse_ok(
            r#"
[@euclid]
Person:
. "Ancient mathematician"
name:
. "Euclid"
. "Euclides"
biography: "Greek mathematician"

[$book.title]
Resource:
. title: "Elements"

[$book.author]
Resource:
. author:
  . "Euclid"
  . "Translator"

[$book.offset]
Resource:
. offset: "Book I"

[$book.url]
Resource:
. url: "https://example.com/elements"

[$book.homepage]
Resource:
. homepage: "https://example.com"

[$book.type]
Resource:
. type: "book"

[$book.edition]
Resource:
. edition: "second"

[$book.editor]
Resource:
. editor: "Editor Name"

[$book.institution]
Resource:
. institution: "Library"

[$book.journal]
Resource:
. journal: "Journal Name"

[$book.publisher]
Resource:
. publisher: "Publisher Name"

[$book.volume]
Resource:
. volume: "I"

[$book.month]
Resource:
. month: "January"

[$book.year]
Resource:
. year: "300BC"

[$book.description]
Resource:
. description: "Classic text"

Specify:
. [numbers.positive.int]
  positive:
  . "positive"
  int:
  . "integer"
  is:
  . "greater than zero"

Specify:
. [numbers.negative.int]
  negative:
  . "negative"
  int:
  . "integer"
  is:
  . "less than zero"

Specify:
. [numbers.zero]
  zero:
  . "zero"
  is:
  . "additive identity"

Specify:
. [numbers.positive.decimal]
  positive:
  . "positive"
  decimal:
  . "decimal"
  is:
  . "greater than zero"

Specify:
. [numbers.negative.decimal]
  negative:
  . "negative"
  decimal:
  . "decimal"
  is:
  . "less than zero"
"#,
        );

        assert_eq!(document.items.len(), 21);

        match &document.items[0] {
            TopLevelItem::Person(group) => {
                assert_eq!(group.person.arguments.len(), 1);
                assert_eq!(group.name.arguments.len(), 2);
                assert_eq!(group.biography.argument.0, "Greek mathematician");
            }
            other => panic!("expected person group, got {other:?}"),
        }

        match &document.items[1] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Title(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[2] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Author(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[3] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Offset(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[4] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(group.resource.arguments[0], ResourceItem::Url(_)));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[5] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Homepage(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[6] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(group.resource.arguments[0], ResourceItem::Type(_)));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[7] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Edition(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[8] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Editor(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[9] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Institution(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[10] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Journal(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[11] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Publisher(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[12] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Volume(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[13] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Month(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[14] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(group.resource.arguments[0], ResourceItem::Year(_)));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[15] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Description(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[16] {
            TopLevelItem::Specify(group) => {
                assert!(matches!(
                    group.specify.arguments[0],
                    SpecifyItem::PositiveInt(_)
                ));
            }
            other => panic!("expected specify group, got {other:?}"),
        }

        match &document.items[17] {
            TopLevelItem::Specify(group) => {
                assert!(matches!(
                    group.specify.arguments[0],
                    SpecifyItem::NegativeInt(_)
                ));
            }
            other => panic!("expected specify group, got {other:?}"),
        }

        match &document.items[18] {
            TopLevelItem::Specify(group) => {
                assert!(matches!(group.specify.arguments[0], SpecifyItem::Zero(_)));
            }
            other => panic!("expected specify group, got {other:?}"),
        }

        match &document.items[19] {
            TopLevelItem::Specify(group) => {
                assert!(matches!(
                    group.specify.arguments[0],
                    SpecifyItem::PositiveDecimal(_)
                ));
            }
            other => panic!("expected specify group, got {other:?}"),
        }

        match &document.items[20] {
            TopLevelItem::Specify(group) => {
                assert!(matches!(
                    group.specify.arguments[0],
                    SpecifyItem::NegativeDecimal(_)
                ));
            }
            other => panic!("expected specify group, got {other:?}"),
        }
    }

    // ===============================[ overview ]=====================================

    #[test]
    fn parses_mixed_structural_document() {
        let text = r#"
[\function]
Describes: f(x_)
using:
. x is \type{A}
when:
. x = x
Provides:
. [symbol]
  symbol: f(x_) :=> x
Aliases:
. [alias]
  alias: f(x_) :=> x
References:
. $elements
Metadata:
. id: "desc-1"

[\statement]
States:
that:
. if:
  . x = x
  then:
  . x = x

[@euclid]
Person:
name:
. "Euclid"
biography: "Greek mathematician"

[$elements]
Resource:
. title: "Elements"
"#;

        let mut tracker = EventLog::new();
        let document = parse_document(text, &mut tracker);

        assert!(!tracker.has_errors(), "{:#?}", tracker.events());
        assert_eq!(document.items.len(), 4);
        assert!(matches!(document.items[0], TopLevelItem::Describes(_)));
        assert!(matches!(document.items[1], TopLevelItem::States(_)));
        assert!(matches!(document.items[2], TopLevelItem::Person(_)));
        assert!(matches!(document.items[3], TopLevelItem::Resource(_)));
    }

    #[test]
    fn recovers_after_invalid_group() {
        let text = r#"
[\function]
Describes: f(x_)
that:
. x = x

Title: "Valid Title"
"#;

        let mut tracker = EventLog::new();
        let document = parse_document(text, &mut tracker);

        assert!(tracker.has_errors());
        assert_eq!(document.items.len(), 1);
        assert!(matches!(document.items[0], TopLevelItem::Title(_)));
    }

    #[test]
    fn parses_clause_groups_as_clauses() {
        let text = r#"
[\property]
States:
that:
. exists: x is \type{A}
  suchThat:
  . x = x
"#;

        let mut tracker = EventLog::new();
        let document = parse_document(text, &mut tracker);

        assert!(!tracker.has_errors(), "{:#?}", tracker.events());
        match &document.items[0] {
            TopLevelItem::States(states) => {
                assert!(matches!(states.that.arguments[0], Clause::Exists(_)));
            }
            other => panic!("expected states item, got {other:?}"),
        }
    }

    #[test]
    fn parses_is_statements_as_inline_clauses() {
        let text = r#"
[\function:on{A}:to{B}]
Describes: f(x__)
when:
. A, B is \set
satisfies:
. forAll: x "in" A
  then:
  . existsUnique: y "in" B
    suchThat:
    . f(x) = y
"#;

        let mut tracker = EventLog::new();
        let document = parse_document(text, &mut tracker);

        assert!(!tracker.has_errors(), "{:#?}", tracker.events());
        match &document.items[0] {
            TopLevelItem::Describes(group) => {
                assert!(matches!(
                    group.when.as_ref().expect("expected when").arguments[0],
                    Clause::Declaration(_)
                ));
                assert!(matches!(
                    group
                        .satisfies
                        .as_ref()
                        .expect("expected satisfies")
                        .arguments[0],
                    Clause::ForAll(_)
                ));
            }
            other => panic!("expected describes item, got {other:?}"),
        }
    }

    #[test]
    fn parses_outline_groups() {
        let document = parse_ok(
            r#"
Title: "Foundations"

Section: "Sets"

Subsection: "Membership"

Subsubsection: "Examples"
"#,
        );

        assert_eq!(document.items.len(), 4);

        match &document.items[0] {
            TopLevelItem::Title(group) => assert_eq!(group.title.argument.0, "Foundations"),
            other => panic!("expected title group, got {other:?}"),
        }
        match &document.items[1] {
            TopLevelItem::Section(group) => assert_eq!(group.section.argument.0, "Sets"),
            other => panic!("expected section group, got {other:?}"),
        }
        match &document.items[2] {
            TopLevelItem::Subsection(group) => {
                assert_eq!(group.subsection.argument.0, "Membership")
            }
            other => panic!("expected subsection group, got {other:?}"),
        }
        match &document.items[3] {
            TopLevelItem::Subsubsection(group) => {
                assert_eq!(group.subsubsection.argument.0, "Examples")
            }
            other => panic!("expected subsubsection group, got {other:?}"),
        }
    }

    // ===============================[ theorems ]=====================================

    #[test]
    fn parses_theorem_like_groups_and_clause_variants() {
        let document = parse_ok(
            r#"
[\axiom]
Axiom:
. "Every element equals itself"
given:
. X is \set
where:
. [logic.not]
  not:
  . x = y
then:
. [logic.if]
  if:
  . x = x
  then:
  . y = y
iff:
. [logic.iff]
  iff:
  . x = x
  then:
  . y = y
Justified:
. [axiom.justified]
  by:
  . "Definition"
  comment: "classical"
Documented:
. [axiom.written]
  written:
  . "axiom"
Aliases:
. [axiom.alias]
  alias: axiom(x_) :=> x
References:
. $axiom.ref
Metadata:
. id: "ax-1"

Theorem:
then:
. [logic.any]
  anyOf:
  . x = x
  . y = y

[\corollary]
Corollary:
. "Immediate consequence"
of:
. "Previous theorem"
then:
. [logic.one]
  oneOf:
  . x = x
  . y = y

Lemma:
then:
. [logic.forall]
  forAll: x is \element
  where:
  . x = x
  then:
  . x = x

[\conjecture]
Conjecture:
. "Open question"
then:
. [logic.exists]
  exists: x is \element
  suchThat:
  . x = x
"#,
        );

        assert_eq!(document.items.len(), 5);

        match &document.items[0] {
            TopLevelItem::Axiom(group) => {
                assert!(group.heading.is_some());
                assert_eq!(group.axiom.arguments.len(), 1);
                assert!(group.given.is_some());
                assert!(matches!(
                    group.where_.as_ref().expect("expected where").arguments[0],
                    Clause::Not(_)
                ));
                assert!(matches!(group.then.arguments[0], Clause::If(_)));
                assert!(matches!(
                    group.iff.as_ref().expect("expected iff").arguments[0],
                    Clause::Iff(_)
                ));
                assert!(group.justified.is_some());
                assert!(group.documented.is_some());
                assert!(group.aliases.is_some());
                assert!(group.references.is_some());
                assert!(group.metadata.is_some());
            }
            other => panic!("expected axiom group, got {other:?}"),
        }

        match &document.items[1] {
            TopLevelItem::Theorem(group) => {
                assert!(group.heading.is_none());
                assert!(matches!(group.then.arguments[0], Clause::AnyOf(_)));
            }
            other => panic!("expected theorem group, got {other:?}"),
        }

        match &document.items[2] {
            TopLevelItem::Corollary(group) => {
                assert!(group.heading.is_some());
                assert_eq!(group.of.arguments[0].0, "Previous theorem");
                assert!(matches!(group.then.arguments[0], Clause::OneOf(_)));
            }
            other => panic!("expected corollary group, got {other:?}"),
        }

        match &document.items[3] {
            TopLevelItem::Lemma(group) => {
                assert!(group.heading.is_none());
                assert!(matches!(group.then.arguments[0], Clause::ForAll(_)));
            }
            other => panic!("expected lemma group, got {other:?}"),
        }

        match &document.items[4] {
            TopLevelItem::Conjecture(group) => {
                assert!(group.heading.is_some());
                assert!(matches!(group.then.arguments[0], Clause::Exists(_)));
            }
            other => panic!("expected conjecture group, got {other:?}"),
        }
    }
}

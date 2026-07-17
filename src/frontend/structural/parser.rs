use std::collections::{HashMap, VecDeque};

use crate::events::EventLog;
use crate::frontend::formulation::ast::{ExpressionKind, FormOrDeclaration, FormOrDeclarationKind};
use crate::frontend::formulation::{
    ParseError as FormulationParseError, parse_author_header, parse_command_header,
    parse_expression, parse_expression_alias, parse_expression_binding, parse_form_or_declaration,
    parse_hard_cast_statement, parse_is_via_statement, parse_label_header,
    parse_ordinary_declaration_statement, parse_refined_declaration_statement,
    parse_resource_header, parse_spec_operator_alias, parse_topic_header, parse_writing_alias,
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

fn parse_describes_target(input: &str) -> Result<DescribesTarget, FormulationParseError> {
    if let Ok(form) = parse_form_or_declaration(input) {
        return Ok(DescribesTarget::Form(form));
    }

    parse_ordinary_declaration_statement(input).map(DescribesTarget::Declaration)
}

/// Parses a quantifier binding or ordinary/refined specification.
pub(in crate::frontend::structural::parser) fn parse_binding_or_spec(
    input: &str,
) -> Result<BindingOrSpec, FormulationParseError> {
    parse_refined_declaration_statement(input).map(BindingOrSpec::Declaration)
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

/// Parses a required topic heading from a `Topic:` group.
///
/// Topic headings are `#`-sigil dotted paths (for example `#real.analysis`) that
/// name a documentation topic and, absent a `Documented:called:`, render as a
/// human title.
pub(in crate::frontend::structural::parser) fn parse_required_topic_heading(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<crate::frontend::formulation::ast::TopicHeader> {
    let Some(heading) = group.heading.as_deref() else {
        tracker.user_error_at_row(Some(ORIGIN), group.metadata.row, "Expected topic heading");
        return None;
    };
    match parse_topic_header(heading) {
        Ok(heading) => Some(heading),
        Err(error) => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Invalid topic heading: {error}"),
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

/// Parses one or more required `AdjectiveText` entries.
pub(in crate::frontend::structural::parser) fn parse_required_adjective_texts(
    section: &ProtoSection,
    tracker: &mut EventLog,
) -> Option<OneOrMore<AdjectiveText>> {
    let starting_issue_count = tracker.issue_count();
    let texts = parse_optional_texts(Some(section), tracker, AdjectiveText);
    one_or_more(texts, || {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(
                Some(ORIGIN),
                section.metadata.row,
                "Expected adjective text",
            );
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
/// Only quote and backslash escapes are interpreted here, so prose can contain
/// escaped string delimiters without changing LaTeX commands such as `\alpha`.
pub(in crate::frontend::structural::parser) fn strip_quoted_text(input: &str) -> Option<String> {
    let input = input.trim();
    let inner = input.strip_prefix('"')?.strip_suffix('"')?;
    Some(crate::frontend::unescape_quoted_text(inner))
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

pub(super) fn parse_equivalently_clause(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<EquivalentlyGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("equivalently", &group.sections, tracker, &["equivalently"])?;
    Some(EquivalentlyGroup {
        heading,
        equivalently: EquivalentlySection {
            arguments: parse_required_clauses(
                section(&sections, "equivalently")?,
                "equivalently",
                tracker,
            )?,
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
/// The bound value is parsed as `is`/spec syntax and the optional `suchThat:`
/// section supplies predicate clauses.
pub(super) fn parse_exists_clause(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ExistsGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("exists", &group.sections, tracker, &["exists", "suchThat?"])?;
    let such_that = match section(&sections, "suchThat") {
        Some(section) => Some(SuchThatSection {
            arguments: parse_required_clauses(section, "suchThat", tracker)?,
        }),
        None => None,
    };
    Some(ExistsGroup {
        heading,
        exists: ExistsSection {
            arguments: parse_required_formulations(
                section(&sections, "exists")?,
                "exists",
                tracker,
                parse_binding_or_spec,
            )?,
        },
        such_that,
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
        &["existsUnique", "suchThat?"],
    )?;
    let such_that = match section(&sections, "suchThat") {
        Some(section) => Some(SuchThatSection {
            arguments: parse_required_clauses(section, "suchThat", tracker)?,
        }),
        None => None,
    };
    Some(ExistsUniqueGroup {
        heading,
        exists_unique: ExistsUniqueSection {
            arguments: parse_required_formulations(
                section(&sections, "existsUnique")?,
                "existsUnique",
                tracker,
                parse_binding_or_spec,
            )?,
        },
        such_that,
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
            arguments: parse_required_formulations(
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

/// Parses a `have:` clause group with an `iff:` condition.
pub(super) fn parse_have_clause(group: &ProtoGroup, tracker: &mut EventLog) -> Option<IffGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("have", &group.sections, tracker, &["have", "iff"])?;
    Some(IffGroup {
        heading,
        iff: IffSection {
            arguments: parse_required_clauses(section(&sections, "iff")?, "iff", tracker)?,
        },
        then: ThenSection {
            arguments: parse_required_clauses(section(&sections, "have")?, "have", tracker)?,
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
            arguments: parse_required_formulations(
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

/// Dispatches nested `Requires:` groups to capability or definition parsers.
pub(super) fn parse_requires_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<RequiresItem> {
    match first_section_label(group)? {
        "capability" => parse_capability(group, tracker)
            .map(Box::new)
            .map(RequiresItem::Capability),
        "definition" => {
            parse_definition_requirement_group(group, tracker).map(RequiresItem::Definition)
        }
        other => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Unexpected requires group `{other}`"),
            );
            None
        }
    }
}

/// Dispatches nested `Enables:` groups to capability, from, or relation parsers.
pub(super) fn parse_enables_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<EnablesItem> {
    match first_section_label(group)? {
        "capability" => parse_capability(group, tracker)
            .map(Box::new)
            .map(EnablesItem::Capability),
        "from" => parse_from_group(group, tracker),
        "relation" => parse_enables_relation_group(group, tracker)
            .map(Box::new)
            .map(EnablesItem::Relation),
        other => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Unexpected enables group `{other}`"),
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
        "description" => parse_description(group, tracker).map(DocumentedItem::Description),
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

/// Dispatches nested `Documented:` groups for `Refines:` entries.
///
/// Refinements are named by adjectives, so `called:` is intentionally rejected
/// here even though it remains valid for ordinary definitions.
pub(super) fn parse_refines_documented_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<DocumentedItem> {
    match first_section_label(group)? {
        "written" => parse_written(group, tracker).map(DocumentedItem::Written),
        "adjective" => parse_adjective(group, tracker).map(DocumentedItem::Adjective),
        "called" => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                "`Refines` documentation does not accept `called:`; use `adjective:`",
            );
            None
        }
        "writing" => parse_writing(group, tracker).map(DocumentedItem::Writing),
        "overview" => parse_overview(group, tracker).map(DocumentedItem::Overview),
        "description" => parse_description(group, tracker).map(DocumentedItem::Description),
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
        "have" => parse_have_clause(group, tracker).map(Clause::Iff),
        "piecewise" => parse_piecewise_clause(group, tracker).map(Clause::Piecewise),
        "given" => parse_given_clause(group, tracker).map(Clause::Given),
        "equivalently" => parse_equivalently_clause(group, tracker).map(Clause::Equivalently),
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

/// Parses a `capability:` nested group inside `Enables:`.
///
/// Capabilities reuse alias-kind parsing because enabled capabilities can stand for
/// expression aliases or specification-operator aliases.
pub(in crate::frontend::structural::parser) fn parse_capability(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<CapabilityGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "capability",
        &group.sections,
        tracker,
        &["capability", "written?"],
    )?;
    Some(CapabilityGroup {
        heading,
        capability: CapabilitySection {
            argument: parse_required_formulation(
                section(&sections, "capability")?,
                "capability",
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

/// Parses a `definition:` nested group inside `Requires:`.
pub(in crate::frontend::structural::parser) fn parse_definition_requirement_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<DefinitionGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("definition", &group.sections, tracker, &["definition"])?;
    Some(DefinitionGroup {
        heading,
        definition: DefinitionSection {
            argument: parse_required_formulation(
                section(&sections, "definition")?,
                "definition",
                tracker,
                parse_definition_requirement,
            )?,
        },
    })
}

fn parse_definition_requirement(
    input: &str,
) -> Result<DefinitionRequirement, FormulationParseError> {
    let expression = parse_expression(input)?;
    let ExpressionKind::IsType { subject, ty } = expression.kind else {
        return Err(FormulationParseError::Custom(
            "`definition:` must have the form `\\command is <spec>`".to_owned(),
        ));
    };
    let ExpressionKind::Command(command) = subject.kind else {
        return Err(FormulationParseError::Custom(
            "`definition:` subject must be a command expression".to_owned(),
        ));
    };
    Ok(DefinitionRequirement { command, ty })
}

/// Parses a cast-backed `from:` nested group inside `Enables:`.
pub(in crate::frontend::structural::parser) fn parse_from_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<EnablesItem> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "from",
        &group.sections,
        tracker,
        &["from", "capability?", "as?", "written?"],
    )?;
    let from = FromSection {
        argument: parse_required_formulation(
            section(&sections, "from")?,
            "from",
            tracker,
            parse_ordinary_declaration_statement,
        )?,
    };
    let capability = sections.get("capability").copied();
    let as_ = sections.get("as").copied();

    match (capability, as_) {
        (Some(capability), None) => {
            Some(EnablesItem::FromCapability(Box::new(FromCapabilityGroup {
                heading,
                from,
                capability: CapabilitySection {
                    argument: parse_required_formulation(
                        capability,
                        "capability",
                        tracker,
                        parse_alias_kind,
                    )?,
                },
                written: sections.get("written").copied().and_then(|section| {
                    parse_required_written_texts(section, tracker)
                        .map(|arguments| WrittenSection { arguments })
                }),
            })))
        }
        (None, Some(as_)) => {
            if sections.contains_key("written") {
                tracker.user_error_at_row(
                    Some(ORIGIN),
                    group.metadata.row,
                    "`from:` groups with `as:` do not accept `written:`",
                );
            }
            Some(EnablesItem::FromAs(Box::new(FromAsGroup {
                heading,
                from,
                as_: CastAsSection {
                    argument: parse_required_formulation(
                        as_,
                        "as",
                        tracker,
                        parse_expression_binding,
                    )?,
                },
            })))
        }
        (Some(_), Some(_)) => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                "`from:` groups must contain either `capability:` or `as:`, not both",
            );
            None
        }
        (None, None) => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                "`from:` groups require either `capability:` or `as:`",
            );
            None
        }
    }
}

/// Parses a `relation:` nested group inside `Enables:`.
pub(in crate::frontend::structural::parser) fn parse_enables_relation_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<EnablesRelationGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "relation",
        &group.sections,
        tracker,
        &["relation", "to", "when?", "means?", "represents?", "by?"],
    )?;

    Some(EnablesRelationGroup {
        heading,
        relation: EnablesRelationSection {
            arguments: parse_optional_open_texts(sections.get("relation").copied(), tracker),
        },
        to: RelationToSection {
            argument: parse_required_formulation(
                section(&sections, "to")?,
                "to",
                tracker,
                parse_relationship_declaration,
            )?,
        },
        when: sections.get("when").copied().and_then(|section| {
            parse_required_formulations(section, "when", tracker, parse_relation_when_item)
                .map(|arguments| RelationWhenSection { arguments })
        }),
        means: sections.get("means").copied().and_then(|section| {
            parse_required_clause(section, "means", tracker)
                .map(|argument| RelationshipMeansSection { argument })
        }),
        represents: sections.get("represents").copied().and_then(|section| {
            parse_required_formulations(section, "represents", tracker, parse_relation_kind)
                .map(|arguments| RelationRepresentsSection { arguments })
        }),
        by: parse_optional_by_relationship_section(&sections, tracker),
    })
}

fn parse_relation_when_item(input: &str) -> Result<RelationWhenItem, FormulationParseError> {
    if let Ok(statement) = parse_hard_cast_statement(input) {
        return Ok(RelationWhenItem::HardCast(statement));
    }
    parse_ordinary_declaration_statement(input).map(RelationWhenItem::Declaration)
}

fn parse_relation_kind(input: &str) -> Result<RelationKind, FormulationParseError> {
    match input.trim() {
        r#"\\coercion"# => Ok(RelationKind::Coercion),
        r#"\\encoding"# => Ok(RelationKind::Encoding),
        _ => Err(FormulationParseError::Custom(
            "`represents:` entries must be `\\\\coercion` or `\\\\encoding`".to_owned(),
        )),
    }
}

fn parse_relationship_declaration(
    input: &str,
) -> Result<RelationshipDeclaration, FormulationParseError> {
    if let Ok(expression) = parse_expression(input) {
        if let ExpressionKind::Command(command) = expression.kind {
            return Ok(RelationshipDeclaration::Command(command));
        }
    }
    parse_ordinary_declaration_statement(input).map(RelationshipDeclaration::Declaration)
}

fn parse_optional_by_relationship_section(
    sections: &HashMap<String, &ProtoSection>,
    tracker: &mut EventLog,
) -> Option<BySection> {
    sections.get("by").copied().and_then(|section| {
        parse_required_open_texts(section, "by", tracker).map(|arguments| BySection {
            arguments: arguments.into(),
        })
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

/// Parses an `adjective:` documentation group for `Refines:` entries.
pub(in crate::frontend::structural::parser) fn parse_adjective(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<AdjectiveGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("adjective", &group.sections, tracker, &["adjective"])?;
    Some(AdjectiveGroup {
        heading,
        adjective: AdjectiveSection {
            arguments: parse_required_adjective_texts(section(&sections, "adjective")?, tracker)?,
        },
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

/// Parses a `description:` documentation group.
pub(in crate::frontend::structural::parser) fn parse_description(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<DescriptionGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("description", &group.sections, tracker, &["description"])?;
    Some(DescriptionGroup {
        heading,
        description: DescriptionSection {
            argument: parse_required_open_text(
                section(&sections, "description")?,
                "description",
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
        "SectionTitle" => parse_section_title(group, tracker).map(TopLevelItem::SectionTitle),
        "SubsectionTitle" => {
            parse_subsection_title(group, tracker).map(TopLevelItem::SubsectionTitle)
        }
        "Text" => parse_text_group(group, tracker).map(TopLevelItem::Text),
        "Writing" => parse_top_level_writing(group, tracker).map(TopLevelItem::Writing),
        "Disambiguates" => parse_disambiguates(group, tracker).map(TopLevelItem::Disambiguates),
        "Describes" => parse_describes(group, tracker).map(TopLevelItem::Describes),
        "Defines" => parse_defines(group, tracker).map(TopLevelItem::Defines),
        "Refines" => parse_refines(group, tracker).map(TopLevelItem::Refines),
        "States" => parse_states(group, tracker).map(TopLevelItem::States),
        "Axiom" => parse_axiom(group, tracker).map(TopLevelItem::Axiom),
        "Theorem" => parse_theorem(group, tracker).map(TopLevelItem::Theorem),
        "Corollary" => parse_corollary(group, tracker).map(TopLevelItem::Corollary),
        "Person" => parse_person(group, tracker).map(TopLevelItem::Person),
        "Resource" => parse_resource(group, tracker).map(TopLevelItem::Resource),
        "Specify" => parse_specify(group, tracker).map(TopLevelItem::Specify),
        "Relation" => parse_relation(group, tracker).map(TopLevelItem::Relation),
        "Equivalent" => parse_equivalent(group, tracker).map(TopLevelItem::Equivalent),
        "Topic" => parse_topic(group, tracker).map(TopLevelItem::Topic),
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
    let sections = identify_sections("Title", &group.sections, tracker, &["Title", "Id?"])?;
    Some(TitleGroup {
        title: TitleSection {
            argument: parse_required_open_text(section(&sections, "Title")?, "Title", tracker)?,
        },
    })
}

/// Parses a top-level `SectionTitle:` group.
///
/// This represents a first-level document outline heading rather than a
/// definition or theorem block.
pub(in crate::frontend::structural::parser) fn parse_section_title(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<SectionTitleGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections(
        "SectionTitle",
        &group.sections,
        tracker,
        &["SectionTitle", "Id?"],
    )?;
    Some(SectionTitleGroup {
        section_title: SectionTitleSection {
            argument: parse_required_open_text(
                section(&sections, "SectionTitle")?,
                "SectionTitle",
                tracker,
            )?,
        },
    })
}

/// Parses a top-level `SubsectionTitle:` group.
///
/// Subsections share the simple outline shape with `SectionTitle:` but carry their
/// own wrapper so rendering can preserve hierarchy.
pub(in crate::frontend::structural::parser) fn parse_subsection_title(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<SubsectionTitleGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections(
        "SubsectionTitle",
        &group.sections,
        tracker,
        &["SubsectionTitle", "Id?"],
    )?;
    Some(SubsectionTitleGroup {
        subsection_title: SubsectionTitleSection {
            argument: parse_required_open_text(
                section(&sections, "SubsectionTitle")?,
                "SubsectionTitle",
                tracker,
            )?,
        },
    })
}

/// Parses a top-level `Text:` group.
///
/// Text groups are document prose blocks that render directly in the page
/// flow, with Markdown/LaTeX interpretation handled by the viewer.
pub(in crate::frontend::structural::parser) fn parse_text_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<TextGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("Text", &group.sections, tracker, &["Text", "Id?"])?;
    Some(TextGroup {
        text: TextSection {
            argument: parse_required_open_text(section(&sections, "Text")?, "Text", tracker)?,
        },
    })
}

/// Parses a collection-wide `Writing:` group.
///
/// These aliases are intentionally narrower than documented `writing:` groups:
/// each entry must map one plain name to the LaTeX used to render that name.
pub(in crate::frontend::structural::parser) fn parse_top_level_writing(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<TopLevelWritingGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("Writing", &group.sections, tracker, &["Writing", "Id?"])?;
    let writing_section = section(&sections, "Writing")?;
    let arguments =
        parse_required_formulations(writing_section, "Writing", tracker, parse_writing_alias)?;

    let mut all_names = true;
    for alias in &arguments {
        if !matches!(alias.form.kind, FormOrDeclarationKind::Name(_)) {
            tracker.user_error_at_row(
                Some(ORIGIN),
                writing_section.metadata.row,
                "Writing aliases must use a name on the left of `:~>`",
            );
            all_names = false;
        }
    }
    if !all_names {
        return None;
    }

    Some(TopLevelWritingGroup {
        writing: TopLevelWritingSection { arguments },
    })
}

// ===============================[ definitions ]=====================================

/// Parses a global operator/function disambiguation table.
///
/// Unlike ordinary section patterns, `Disambiguates` permits zero or more
/// ordered `when:`/`to:` pairs plus an optional `else:`, so this parser walks
/// the section list directly.
pub(in crate::frontend::structural::parser) fn parse_disambiguates(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<DisambiguatesGroup> {
    let heading = parse_required_disambiguates_heading(group, tracker)?;
    let first = group.sections.first()?;
    if first.label != "Disambiguates" {
        tracker.user_error_at_row(
            Some(ORIGIN),
            first.metadata.row,
            format!("Expected `Disambiguates` but found `{}`", first.label),
        );
        return None;
    }

    ensure_empty_section(first, "Disambiguates", tracker);

    let mut index = 1;
    let mut branches = Vec::new();
    while let Some(section) = group.sections.get(index) {
        if section.label != "when" {
            break;
        }

        let when = parse_required_clauses(section, "when", tracker)
            .map(|arguments| WhenSection { arguments })?;
        index += 1;

        let Some(to_section) = group.sections.get(index) else {
            tracker.user_error_at_row(
                Some(ORIGIN),
                section.metadata.row,
                "Expected `to` section after `when`",
            );
            return None;
        };
        if to_section.label != "to" {
            tracker.user_error_at_row(
                Some(ORIGIN),
                to_section.metadata.row,
                format!(
                    "Expected `to` section after `when` but found `{}`",
                    to_section.label
                ),
            );
            return None;
        }

        let to = parse_required_formulation(to_section, "to", tracker, parse_expression)
            .map(|argument| DisambiguatesToSection { argument })?;
        branches.push(DisambiguatesBranch { when, to });
        index += 1;
    }

    let else_ = match group.sections.get(index) {
        Some(section) if section.label == "else" => {
            index += 1;
            parse_required_formulation(section, "else", tracker, parse_expression)
                .map(|argument| DisambiguatesElseSection { argument })
        }
        _ => None,
    };

    if branches.is_empty() && else_.is_none() {
        tracker.user_error_at_row(
            Some(ORIGIN),
            first.metadata.row,
            "Expected at least one `when`/`to` branch or an `else` section in `Disambiguates`",
        );
        return None;
    }

    let trailing = identify_sections(
        "Disambiguates",
        &group.sections[index..],
        tracker,
        &[
            "Justified?",
            "Documented?",
            "Aliases?",
            "References?",
            "Metadata?",
            "Id?",
        ],
    )?;

    Some(DisambiguatesGroup {
        heading,
        branches,
        else_,
        justified: trailing.get("Justified").copied().and_then(|section| {
            parse_required_groups(section, "Justified", tracker, parse_justified_item_group)
                .map(|arguments| JustifiedSection { arguments })
        }),
        documented: trailing.get("Documented").copied().and_then(|section| {
            parse_required_groups(section, "Documented", tracker, parse_documented_item_group)
                .map(|arguments| DocumentedSection { arguments })
        }),
        aliases: trailing.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", tracker, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        references: trailing.get("References").copied().and_then(|section| {
            parse_required_formulations(section, "References", tracker, parse_resource_header)
                .map(|arguments| ReferencesSection { arguments })
        }),
        metadata: trailing.get("Metadata").copied().and_then(|section| {
            parse_required_groups(section, "Metadata", tracker, parse_metadata_item_group)
                .map(|arguments| MetadataSection { arguments })
        }),
    })
}

fn parse_required_disambiguates_heading(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<FormOrDeclaration> {
    let Some(raw_heading) = group.heading.as_deref() else {
        tracker.user_error_at_row(
            Some(ORIGIN),
            group.metadata.row,
            "Expected disambiguation heading",
        );
        return None;
    };

    if raw_heading.contains(':') {
        tracker.user_error_at_row(
            Some(ORIGIN),
            group.metadata.row,
            "Disambiguates headings cannot use colon-directed operators",
        );
        return None;
    }

    let heading = match parse_form_or_declaration(raw_heading) {
        Ok(heading) => heading,
        Err(error) => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Invalid disambiguation heading: {error}"),
            );
            return None;
        }
    };

    match &heading.kind {
        FormOrDeclarationKind::FunctionDeclaration { name: None, .. }
        | FormOrDeclarationKind::InfixOperator { .. }
        | FormOrDeclarationKind::PrefixOperator { .. }
        | FormOrDeclarationKind::PostfixOperator { .. } => Some(heading),
        FormOrDeclarationKind::FunctionDeclaration { name: Some(_), .. } => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                "Disambiguates function headings cannot use declaration aliases",
            );
            None
        }
        FormOrDeclarationKind::Name(_)
        | FormOrDeclarationKind::TupleDeclaration { .. }
        | FormOrDeclarationKind::SetDeclaration { .. } => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                "Disambiguates headings must be operator or function forms",
            );
            None
        }
    }
}

fn ensure_empty_section(section: &ProtoSection, label: &str, tracker: &mut EventLog) {
    for entry in section_entries(section) {
        let row = match entry {
            SectionEntry::Inline { row, .. }
            | SectionEntry::Formulation { row, .. }
            | SectionEntry::Text { row, .. }
            | SectionEntry::Group { row, .. } => row,
        };
        tracker.user_error_at_row(
            Some(ORIGIN),
            row,
            format!("Section `{label}` does not accept content"),
        );
    }
}

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
            "Requires?",
            "Enables?",
            "Justified?",
            "Documented?",
            "Aliases?",
            "References?",
            "Metadata?",
            "Id?",
        ],
    )?;

    Some(DescribesGroup {
        heading,
        describes: DescribesSection {
            argument: parse_required_formulation(
                section(&sections, "Describes")?,
                "Describes",
                tracker,
                parse_describes_target,
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
        requires: sections.get("Requires").copied().and_then(|section| {
            parse_required_groups(section, "Requires", tracker, parse_requires_item_group)
                .map(|arguments| RequiresSection { arguments })
        }),
        enables: sections.get("Enables").copied().and_then(|section| {
            parse_required_groups(section, "Enables", tracker, parse_enables_item_group)
                .map(|arguments| EnablesSection { arguments })
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
            "Requires?",
            "Enables?",
            "Justified?",
            "Documented?",
            "Aliases?",
            "References?",
            "Metadata?",
            "Id?",
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
            parse_required_clauses(section, "expresses", tracker)
                .map(|arguments| ExpressesSection { arguments })
        }),
        requires: sections.get("Requires").copied().and_then(|section| {
            parse_required_groups(section, "Requires", tracker, parse_requires_item_group)
                .map(|arguments| RequiresSection { arguments })
        }),
        enables: sections.get("Enables").copied().and_then(|section| {
            parse_required_groups(section, "Enables", tracker, parse_enables_item_group)
                .map(|arguments| EnablesSection { arguments })
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
/// `Refines:`/`extends:` bodies with the parser variant that accepts refined
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
            "extends?",
            "satisfies?",
            "Requires?",
            "Enables?",
            "Justified?",
            "Documented?",
            "Aliases?",
            "References?",
            "Metadata?",
            "Id?",
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
        extends: sections.get("extends").copied().and_then(|section| {
            parse_required_formulation(
                section,
                "extends",
                tracker,
                parse_refined_declaration_statement,
            )
            .map(|argument| RefinesExtendsSection { argument })
        }),
        satisfies: sections.get("satisfies").copied().and_then(|section| {
            parse_required_clauses(section, "satisfies", tracker)
                .map(|arguments| SatisfiesSection { arguments })
        }),
        requires: sections.get("Requires").copied().and_then(|section| {
            parse_required_groups(section, "Requires", tracker, parse_requires_item_group)
                .map(|arguments| RequiresSection { arguments })
        }),
        enables: sections.get("Enables").copied().and_then(|section| {
            parse_required_groups(section, "Enables", tracker, parse_enables_item_group)
                .map(|arguments| EnablesSection { arguments })
        }),
        justified: sections.get("Justified").copied().and_then(|section| {
            parse_required_groups(section, "Justified", tracker, parse_justified_item_group)
                .map(|arguments| JustifiedSection { arguments })
        }),
        documented: sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(
                section,
                "Documented",
                tracker,
                parse_refines_documented_item_group,
            )
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
            "Requires?",
            "Enables?",
            "Justified?",
            "Documented?",
            "Aliases?",
            "References?",
            "Metadata?",
            "Id?",
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
        requires: sections.get("Requires").copied().and_then(|section| {
            parse_required_groups(section, "Requires", tracker, parse_requires_item_group)
                .map(|arguments| RequiresSection { arguments })
        }),
        enables: sections.get("Enables").copied().and_then(|section| {
            parse_required_groups(section, "Enables", tracker, parse_enables_item_group)
                .map(|arguments| EnablesSection { arguments })
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

// ===============================[ equivalent ]=====================================

/// Parses a top-level `Equivalent:` item.
///
/// The required `[...]` command heading names the equivalence class, and the
/// required `to:` section lists one or more `\command` expressions asserted to be
/// interchangeable. It has no `Enables:`/`Requires:`/`Aliases:`/`Metadata:`.
pub(in crate::frontend::structural::parser) fn parse_equivalent(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<EquivalentGroup> {
    let heading = parse_required_command_heading(group, tracker)?;
    let sections = identify_sections(
        "Equivalent",
        &group.sections,
        tracker,
        &[
            "Equivalent",
            "using?",
            "when?",
            "to",
            "Justified?",
            "Documented?",
            "References?",
            "Id?",
        ],
    )?;

    Some(EquivalentGroup {
        heading,
        equivalent: EquivalentSection {
            arguments: parse_optional_open_texts(sections.get("Equivalent").copied(), tracker),
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
        to: EquivalentToSection {
            arguments: parse_required_formulations(
                section(&sections, "to")?,
                "to",
                tracker,
                parse_expression,
            )?,
        },
        justified: sections.get("Justified").copied().and_then(|section| {
            parse_required_groups(section, "Justified", tracker, parse_justified_item_group)
                .map(|arguments| JustifiedSection { arguments })
        }),
        documented: sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(section, "Documented", tracker, parse_documented_item_group)
                .map(|arguments| DocumentedSection { arguments })
        }),
        references: sections.get("References").copied().and_then(|section| {
            parse_required_formulations(section, "References", tracker, parse_resource_header)
                .map(|arguments| ReferencesSection { arguments })
        }),
    })
}

// ===============================[ relation ]=====================================

/// Parses a top-level `Relation:` item.
///
/// A `Relation:` states a bidirectional relationship between the two concepts
/// declared in the required `between:` and `and:` sections. It takes no command
/// heading and — unlike the `Enables: relation:` group — registers no cast rule.
pub(in crate::frontend::structural::parser) fn parse_relation(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<RelationGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections(
        "Relation",
        &group.sections,
        tracker,
        &[
            "Relation",
            "using?",
            "between",
            "and",
            "when?",
            "means?",
            "Justified?",
            "Documented?",
            "Aliases?",
            "References?",
            "Metadata?",
            "Id?",
        ],
    )?;

    Some(RelationGroup {
        relation: RelationSection {
            arguments: parse_optional_open_texts(sections.get("Relation").copied(), tracker),
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
        between: RelationBetweenSection {
            argument: parse_required_relation_subject(
                section(&sections, "between")?,
                "between",
                tracker,
            )?,
        },
        and_: RelationAndSection {
            argument: parse_required_relation_subject(section(&sections, "and")?, "and", tracker)?,
        },
        when: sections.get("when").copied().and_then(|section| {
            parse_required_clauses(section, "when", tracker)
                .map(|arguments| WhenSection { arguments })
        }),
        means: sections.get("means").copied().and_then(|section| {
            parse_required_relation_means(section, tracker)
                .map(|argument| RelationMeansSection { argument })
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

/// Reports whether a section's single argument is quoted text.
///
/// Both top-level `Relation:` subjects and its `means:` accept either quoted text
/// (a `"#topic"`/`"\signature"` reference, or a prose description) or an unquoted
/// formulation; this routes the section to the matching parser. The inline
/// argument carries its quotes verbatim, so quoting is detected with
/// [`strip_quoted_text`].
fn section_is_quoted_text(section: &ProtoSection) -> bool {
    match section_entries(section).first() {
        Some(SectionEntry::Text { .. }) => true,
        Some(SectionEntry::Inline { text, .. }) => strip_quoted_text(text).is_some(),
        _ => false,
    }
}

/// Parses one side of a `Relation:` (`between:`/`and:`).
///
/// A quoted `"#topic"` or `"\signature"` is a reference (to a topic or a
/// definition); anything unquoted is parsed as an ordinary refined declaration
/// such as `a is \real`.
fn parse_required_relation_subject(
    section: &ProtoSection,
    label: &str,
    tracker: &mut EventLog,
) -> Option<RelationSubject> {
    if section_is_quoted_text(section) {
        parse_required_open_text(section, label, tracker).map(RelationSubject::Reference)
    } else {
        parse_required_formulation(section, label, tracker, parse_refined_declaration_statement)
            .map(|declaration| RelationSubject::Declaration(Box::new(declaration)))
    }
}

/// Parses the `means:` of a `Relation:`.
///
/// A quoted string is a prose description; anything unquoted is parsed as a
/// logical clause (a statement of what the relationship means).
fn parse_required_relation_means(
    section: &ProtoSection,
    tracker: &mut EventLog,
) -> Option<RelationMeans> {
    if section_is_quoted_text(section) {
        parse_required_open_text(section, "means", tracker).map(RelationMeans::Text)
    } else {
        parse_required_clause(section, "means", tracker)
            .map(|clause| RelationMeans::Statement(Box::new(clause)))
    }
}

// ===============================[ topic ]=====================================

/// Parses a top-level `Topic:` item.
///
/// A `Topic:` names a documentation topic via a required `#`-sigil heading. The
/// optional `within:` names a parent topic (making this a sub-topic) as a quoted
/// `"#..."` reference; the optional `Related:` records relationships to other
/// topics or definitions; and the optional `Documented:` accepts only `called:`,
/// which overrides how the topic title is rendered.
pub(in crate::frontend::structural::parser) fn parse_topic(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<TopicGroup> {
    let heading = parse_required_topic_heading(group, tracker)?;
    let sections = identify_sections(
        "Topic",
        &group.sections,
        tracker,
        &["Topic", "within?", "Related?", "Documented?", "Id?"],
    )?;

    Some(TopicGroup {
        heading,
        topic: TopicSection {
            arguments: parse_optional_open_texts(sections.get("Topic").copied(), tracker),
        },
        within: sections.get("within").copied().and_then(|section| {
            parse_required_open_text(section, "within", tracker)
                .map(|argument| TopicWithinSection { argument })
        }),
        related: sections.get("Related").copied().and_then(|section| {
            parse_required_groups(section, "Related", tracker, parse_related_item_group)
                .map(|arguments| TopicRelatedSection { arguments })
        }),
        documented: sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(
                section,
                "Documented",
                tracker,
                parse_topic_documented_item_group,
            )
            .map(|arguments| DocumentedSection { arguments })
        }),
    })
}

/// Parses one entry of a `Topic:`'s `Related:` section.
///
/// Each entry lists one or more `to:` references (quoted `"#topic"` or
/// `"\signature"` strings) and a required `means:` description. References are
/// quoted text so a bare `\signature` reads as a reference to a definition rather
/// than a usage; they are recorded, not resolved.
pub(super) fn parse_related_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<TopicRelatedItem> {
    let sections = identify_sections("related", &group.sections, tracker, &["to", "means"])?;
    Some(TopicRelatedItem {
        to: TopicRelatedToSection {
            arguments: parse_required_open_texts(section(&sections, "to")?, "to", tracker)?,
        },
        means: TopicRelatedMeansSection {
            argument: parse_required_open_text(section(&sections, "means")?, "means", tracker)?,
        },
    })
}

/// Dispatches nested `Documented:` groups for `Topic:` entries.
///
/// A topic's documentation only controls how its title renders, so `called:` is
/// the sole accepted field.
pub(super) fn parse_topic_documented_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<DocumentedItem> {
    match first_section_label(group)? {
        "called" => parse_called(group, tracker).map(DocumentedItem::Called),
        other => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("`Topic` documentation only accepts `called:`, not `{other}:`"),
            );
            None
        }
    }
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

/// Rejects a name/argument on a theorem-like head section.
///
/// `Axiom:`/`Theorem:`/`Corollary:` no longer accept a name;
/// a result's name belongs in `Documented:` `called:`, matching the definition items.
fn ensure_no_named_result_arg(section: Option<&ProtoSection>, name: &str, tracker: &mut EventLog) {
    let Some(section) = section else {
        return;
    };
    if let Some(entry) = section_entries(section).first() {
        let row = match entry {
            SectionEntry::Inline { row, .. }
            | SectionEntry::Formulation { row, .. }
            | SectionEntry::Text { row, .. }
            | SectionEntry::Group { row, .. } => *row,
        };
        tracker.user_error_at_row(
            Some(ORIGIN),
            row,
            format!(
                "`{name}:` does not take a name; put the result's name in `Documented:` `called:`"
            ),
        );
    }
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
            "Id?",
        ],
    )?;
    ensure_no_named_result_arg(sections.get(section_name).copied(), name, tracker);

    Some((
        heading,
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
            "Id?",
        ],
    )?;
    ensure_no_named_result_arg(sections.get("Corollary").copied(), "Corollary", tracker);

    Some(CorollaryGroup {
        heading,
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
/// Person groups require an author-style heading and carry required name text on
/// the leading `Person:` section plus an optional `biography:` section.
pub(in crate::frontend::structural::parser) fn parse_person(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<PersonGroup> {
    let heading = parse_required_author_heading(group, tracker)?;
    let sections = identify_sections(
        "Person",
        &group.sections,
        tracker,
        &["Person", "biography?", "Id?"],
    )?;

    Some(PersonGroup {
        heading,
        person: PersonSection {
            arguments: parse_required_open_texts(section(&sections, "Person")?, "Person", tracker)?,
        },
        biography: sections.get("biography").copied().and_then(|section| {
            parse_required_open_text(section, "biography", tracker)
                .map(|argument| BiographySection { argument })
        }),
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
    let sections = identify_sections("Resource", &group.sections, tracker, &["Resource", "Id?"])?;

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
    let sections = identify_sections("Specify", &group.sections, tracker, &["Specify", "Id?"])?;
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
    use crate::frontend::formulation::ast::{
        FormOrDeclaration, FormOrDeclarationKind, IsSubjectForm, IsSubjectKind,
    };
    use crate::frontend::structural::ast::{
        AliasItem, AliasKind, Clause, DescribesTarget, Document, DocumentedItem, EnablesItem,
        IsOrViaItem, JustifiedItem, MetadataItem, RelationKind, RelationMeans, RelationSubject,
        RelationshipDeclaration, RequiresItem, ResourceItem, SpecifyItem, TopLevelItem,
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
    fn parses_disambiguates_groups_with_ordered_branches() {
        let document = parse_ok(
            r#"
[x_ + y_]
Disambiguates:
when:
. x_ is \real
. y_ is \complex
to: x_ \.complex.+./ y_
when:
. x_ is \real
. y_ is \integer
to: x_ \.real.+./ y_
else: x_
Documented:
. written: "x_? + y_?"
"#,
        );

        match &document.items[0] {
            TopLevelItem::Disambiguates(group) => {
                assert_eq!(group.branches.len(), 2);
                assert!(group.else_.is_some());
                assert!(group.documented.is_some());
                assert!(matches!(
                    group.heading.kind,
                    FormOrDeclarationKind::InfixOperator { ref operator, .. }
                        if operator.text == "+"
                ));
            }
            other => panic!("expected Disambiguates item, got {other:?}"),
        }
    }

    #[test]
    fn parses_disambiguates_groups_with_else_only() {
        let document = parse_ok(
            r#"
[x_ + y_]
Disambiguates:
else: x_ :-: y_
Documented:
. written: "x_? + y_?"
"#,
        );

        match &document.items[0] {
            TopLevelItem::Disambiguates(group) => {
                assert!(group.branches.is_empty());
                assert!(group.else_.is_some());
                assert!(group.documented.is_some());
            }
            other => panic!("expected Disambiguates item, got {other:?}"),
        }
    }

    #[test]
    fn parses_requires_capabilities_and_definitions() {
        let document = parse_ok(
            r#"
[\natural]
Describes: n
Requires:
. capability: n_ + m_ :=> n_ \.natural.+./ m_
. definition: \natural.0 is \natural
Documented:
. called: "natural"
"#,
        );

        assert_eq!(document.items.len(), 1);
        match &document.items[0] {
            TopLevelItem::Describes(group) => {
                let requires = group.requires.as_ref().expect("expected Requires section");
                assert_eq!(requires.arguments.len(), 2);
                assert!(matches!(requires.arguments[0], RequiresItem::Capability(_)));
                assert!(matches!(requires.arguments[1], RequiresItem::Definition(_)));
            }
            other => panic!("expected Describes item, got {other:?}"),
        }
    }

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
Enables:
. [symbol.plus]
  capability: plus(x_, y_) :=> x + y
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
Enables:
. [conn.plus]
  relation:
  to: y := \foo{X} is \bar
  represents: \\coercion
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

[\structure.description]
Describes: P
Documented:
. [docs.description]
  description: "Longer prose for readers"

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
Refines: x
using:
. X is \set
when:
. [logic.exists]
  existsUnique: x is \element
  suchThat:
  . x = x
extends: y is \(f)::[[g]]
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

        assert_eq!(document.items.len(), 11);

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
                    group.enables.as_ref().expect("expected enables").arguments[0],
                    EnablesItem::Capability(_)
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
                    group.enables.as_ref().expect("expected enables").arguments[0],
                    EnablesItem::Relation(_)
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
                    DocumentedItem::Description(_)
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
                    DocumentedItem::Related(_)
                ));
            }
            other => panic!("expected describes group, got {other:?}"),
        }

        match &document.items[6] {
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

        match &document.items[7] {
            TopLevelItem::Defines(group) => {
                assert!(matches!(
                    group
                        .expresses
                        .as_ref()
                        .expect("expected expresses")
                        .arguments[0],
                    Clause::Piecewise(_)
                ));
            }
            other => panic!("expected defines group, got {other:?}"),
        }

        match &document.items[8] {
            TopLevelItem::Refines(group) => {
                assert!(group.refines.argument.relation.is_none());
                assert!(group.extends.is_some());
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

        match &document.items[9] {
            TopLevelItem::States(group) => {
                assert_eq!(group.states.arguments.len(), 2);
                assert_eq!(group.states.arguments[0].0, "Closure law");
                assert_eq!(group.states.arguments[1].0, "Associativity");
                assert!(matches!(group.that.arguments[0], Clause::Exists(_)));
            }
            other => panic!("expected states group, got {other:?}"),
        }

        match &document.items[10] {
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
Enables:
. capability: x_ "in" X :-> \\abstract
Documented:
. called: "set"
"#,
        );

        let TopLevelItem::Describes(group) = &document.items[0] else {
            panic!("expected describes group");
        };

        assert!(matches!(
            group.enables.as_ref().expect("expected enables").arguments[0],
            EnablesItem::Capability(_)
        ));
    }

    #[test]
    fn parses_from_capability_from_as_and_relation_enables_items() {
        let document = parse_ok(
            r#"
[\set]
Describes: X
Enables:
. from: Y ::= {y__ : ...}
  capability: x_ "in" X :-> x_ member_of Y
. from: P ::= {(p_, q_) : ...}
  as: f(p_) := q_
. relation:
  to: r := \as.rational{X} is \rational
  when: X is \set
  means: X \.embedded.to./ r
  represents:
  . \\coercion
Documented:
. called: "set"
"#,
        );

        let TopLevelItem::Describes(group) = &document.items[0] else {
            panic!("expected describes group");
        };
        let enables = group.enables.as_ref().expect("expected enables");
        assert!(matches!(
            enables.arguments[0],
            EnablesItem::FromCapability(_)
        ));
        assert!(matches!(enables.arguments[1], EnablesItem::FromAs(_)));
        assert!(matches!(enables.arguments[2], EnablesItem::Relation(_)));
    }

    #[test]
    fn parses_relationship_enables_items() {
        let document = parse_ok(
            r#"
[\pair]
Defines: P is \pair
Enables:
. relation:
  to: x := \pair:of{a0}:and{b0}
  when:
  . a0 := a is! \set
  . b0 := b is \foo
  means: x \:isomorphic.to?:/ p
  represents:
  . \\coercion
  . \\encoding
  by: "\some.theorem"
. relation:
  to: x is \group
  when:
  . x is \set
  means: x is \group
Documented:
. written: "P?"
"#,
        );

        let TopLevelItem::Defines(group) = &document.items[0] else {
            panic!("expected defines group");
        };
        let enables = group.enables.as_ref().expect("expected enables");
        assert!(matches!(enables.arguments[0], EnablesItem::Relation(_)));
        assert!(matches!(enables.arguments[1], EnablesItem::Relation(_)));

        let EnablesItem::Relation(relation) = &enables.arguments[0] else {
            panic!("expected relation");
        };
        assert_eq!(
            relation
                .when
                .as_ref()
                .expect("expected when")
                .arguments
                .len(),
            2
        );
        assert!(matches!(
            relation.to.argument,
            RelationshipDeclaration::Declaration(_)
        ));
        let represents = relation
            .represents
            .as_ref()
            .expect("expected represents markers");
        assert_eq!(represents.arguments.len(), 2);
        assert!(represents.arguments.contains(&RelationKind::Coercion));
        assert!(represents.arguments.contains(&RelationKind::Encoding));
        assert!(relation.by.is_some());
        assert!(relation.means.is_some());
    }

    #[test]
    fn rejects_legacy_viewable_enables_group() {
        let (_document, diagnostics) = parse_with_diagnostics(
            r#"
[\integer]
Describes: n
Enables:
. viewable:
  as: r is \rational
Documented:
. written: "\operatorname{integer}"
"#,
        );

        assert!(diagnostics.iter().any(|event| {
            matches!(event, Event::Message(message) if
                message.message.contains("Unexpected enables group `viewable`")
            )
        }));
    }

    #[test]
    fn rejects_legacy_connection_enables_group() {
        let (_document, diagnostics) = parse_with_diagnostics(
            r#"
[\integer]
Describes: n
Enables:
. connection:
  to: s := \as.set{n} is \set
  represents: \\encoding
Documented:
. written: "\operatorname{integer}"
"#,
        );

        assert!(diagnostics.iter().any(|event| {
            matches!(event, Event::Message(message) if
                message.message.contains("Unexpected enables group `connection`")
            )
        }));
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

SectionTitle: "Recovered"
"#,
        );

        assert_eq!(document.items.len(), 1);
        assert!(matches!(document.items[0], TopLevelItem::SectionTitle(_)));
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
    fn parses_relation_item_with_using_between_and_when_means() {
        let document = parse_ok(
            r#"
Relation:
using:
. n is \integer
between: a is \real
and: b is \real
when:
. a = b
means: a = b
Documented:
. description: "a and b name the same value."
"#,
        );

        match &document.items[0] {
            TopLevelItem::Relation(group) => {
                assert!(group.using.is_some());
                assert!(group.when.is_some());
                assert!(group.means.is_some());
                assert!(group.documented.is_some());
            }
            other => panic!("expected Relation item, got {other:?}"),
        }
    }

    #[test]
    fn relation_requires_between_and_and_sections() {
        let (_, diagnostics) = parse_with_diagnostics(
            r#"
Relation:
between: a is \real
means: a = a
"#,
        );

        assert!(
            diagnostics.iter().any(|event| event
                .as_message()
                .is_some_and(|message| message.message.contains("and"))),
            "expected a diagnostic about the missing `and:` section: {diagnostics:#?}"
        );
    }

    #[test]
    fn parses_topic_item_with_within_related_and_documented_called() {
        let document = parse_ok(
            r##"
[#real.analysis]
Topic: "Analysis over the real numbers."
within: "#analysis"
Related:
. to: "#complex.analysis"
  . "\sin"
  means: "Closely connected subjects."
. to: "\function:on:to"
  means: "Functions studied here."
Documented:
. called: "Real Analysis"
"##,
        );

        match &document.items[0] {
            TopLevelItem::Topic(group) => {
                assert_eq!(group.heading.parts.len(), 2);
                assert_eq!(group.heading.parts[0], "real");
                assert_eq!(group.heading.parts[1], "analysis");
                assert_eq!(group.topic.arguments.len(), 1);
                let within = group.within.as_ref().expect("expected a within section");
                assert_eq!(within.argument.0, "#analysis");
                let related = group.related.as_ref().expect("expected a Related section");
                assert_eq!(related.arguments.len(), 2);
                let first = &related.arguments[0];
                assert_eq!(first.to.arguments.len(), 2);
                assert_eq!(first.to.arguments[0].0, "#complex.analysis");
                assert_eq!(first.to.arguments[1].0, r"\sin");
                assert_eq!(first.means.argument.0, "Closely connected subjects.");
                assert_eq!(related.arguments[1].to.arguments[0].0, r"\function:on:to");
                assert!(group.documented.is_some());
            }
            other => panic!("expected Topic item, got {other:?}"),
        }
    }

    #[test]
    fn topic_related_item_requires_means() {
        let (_, diagnostics) = parse_with_diagnostics(
            r##"
[#real.analysis]
Topic: "Analysis over the real numbers."
Related:
. to: "#complex.analysis"
"##,
        );

        assert!(
            diagnostics.iter().any(|event| event
                .as_message()
                .is_some_and(|message| message.message.contains("means"))),
            "expected a diagnostic about the missing `means:` section: {diagnostics:#?}"
        );
    }

    #[test]
    fn topic_requires_a_heading() {
        let (_, diagnostics) = parse_with_diagnostics(
            r#"
Topic: "A topic with no heading."
"#,
        );

        assert!(
            diagnostics.iter().any(|event| event
                .as_message()
                .is_some_and(|message| message.message.contains("topic heading"))),
            "expected a diagnostic about the missing topic heading: {diagnostics:#?}"
        );
    }

    #[test]
    fn topic_documented_rejects_fields_other_than_called() {
        let (_, diagnostics) = parse_with_diagnostics(
            r#"
[#real.analysis]
Topic: "Analysis over the real numbers."
Documented:
. description: "Not allowed here."
"#,
        );

        assert!(
            diagnostics.iter().any(|event| event
                .as_message()
                .is_some_and(|message| message.message.contains("only accepts `called:`"))),
            "expected a diagnostic rejecting non-`called:` topic documentation: {diagnostics:#?}"
        );
    }

    #[test]
    fn parses_relation_between_quoted_topic_and_signature_with_text_means() {
        let document = parse_ok(
            r##"
Relation:
between: "#real.analysis"
and: "\sin"
means: "The sine function is studied within real analysis."
"##,
        );

        match &document.items[0] {
            TopLevelItem::Relation(group) => {
                match &group.between.argument {
                    RelationSubject::Reference(text) => assert_eq!(text.0, "#real.analysis"),
                    other => panic!("expected a reference subject, got {other:?}"),
                }
                match &group.and_.argument {
                    RelationSubject::Reference(text) => assert_eq!(text.0, r"\sin"),
                    other => panic!("expected a reference subject, got {other:?}"),
                }
                match &group.means.as_ref().expect("means").argument {
                    RelationMeans::Text(text) => {
                        assert_eq!(text.0, "The sine function is studied within real analysis.")
                    }
                    other => panic!("expected a text means, got {other:?}"),
                }
            }
            other => panic!("expected Relation item, got {other:?}"),
        }
    }

    #[test]
    fn parses_relation_between_declaration_with_statement_means() {
        let document = parse_ok(
            r#"
Relation:
between: a is \real
and: b is \real
means: a = b
"#,
        );

        match &document.items[0] {
            TopLevelItem::Relation(group) => {
                assert!(matches!(
                    &group.between.argument,
                    RelationSubject::Declaration(_)
                ));
                assert!(matches!(
                    &group.means.as_ref().expect("means").argument,
                    RelationMeans::Statement(_)
                ));
            }
            other => panic!("expected Relation item, got {other:?}"),
        }
    }

    #[test]
    fn parses_equivalent_item_with_using_when_to() {
        let document = parse_ok(
            r#"
[\foo:of{a}:with{b}]
Equivalent:
using:
. n is \integer
when:
. a is \real
. b is \real
to:
. \bar{a, b}
. \baz:with{b}:and{a}
References:
. $book.foo
Id: "11111111-1111-4111-8111-111111111111"
"#,
        );

        match &document.items[0] {
            TopLevelItem::Equivalent(group) => {
                assert!(group.using.is_some());
                assert!(group.when.is_some());
                assert_eq!(group.to.arguments.len(), 2);
                assert!(group.references.is_some());
            }
            other => panic!("expected Equivalent item, got {other:?}"),
        }
    }

    #[test]
    fn parses_equivalent_item_with_head_text() {
        let document = parse_ok(
            r#"
[\eq{a, b}]
Equivalent:
. "a and b are interchangeable"
to:
. \bar{a, b}
. \baz{a, b}
"#,
        );

        match &document.items[0] {
            TopLevelItem::Equivalent(group) => {
                assert_eq!(group.equivalent.arguments.len(), 1);
                assert_eq!(group.to.arguments.len(), 2);
                assert!(group.using.is_none());
            }
            other => panic!("expected Equivalent item, got {other:?}"),
        }
    }

    #[test]
    fn equivalent_requires_a_to_section() {
        let (_, diagnostics) = parse_with_diagnostics(
            r#"
[\foo{a}]
Equivalent:
when:
. a is \real
"#,
        );

        assert!(
            diagnostics.iter().any(|event| event
                .as_message()
                .is_some_and(|message| message.message.contains("to"))),
            "expected a diagnostic about the missing `to:` section: {diagnostics:#?}"
        );
    }

    #[test]
    fn parses_equivalently_clause_inside_theorem() {
        let document = parse_ok(
            r#"
Theorem:
then:
. equivalently:
  . a = b
  . b = a
"#,
        );

        let TopLevelItem::Theorem(theorem) = &document.items[0] else {
            panic!("expected Theorem item, got {:?}", document.items[0]);
        };
        match &theorem.then.arguments[0] {
            Clause::Equivalently(group) => {
                assert_eq!(group.equivalently.arguments.len(), 2);
            }
            other => panic!("expected an equivalently clause, got {other:?}"),
        }
    }

    #[test]
    fn parses_structural_golden_directory() {
        let directory = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/goldens/structural"));
        let files = read_test_files(directory, "text");
        let expected_names = BTreeSet::from([
            "axioms.text".to_owned(),
            "corollaries.text".to_owned(),
            "defines.text".to_owned(),
            "describes.text".to_owned(),
            "equivalent.text".to_owned(),
            "outline.text".to_owned(),
            "persons.text".to_owned(),
            "refines.text".to_owned(),
            "relations.text".to_owned(),
            "resources.text".to_owned(),
            "specify.text".to_owned(),
            "states.text".to_owned(),
            "theorems.text".to_owned(),
            "topics.text".to_owned(),
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
                assert_eq!(group.person.arguments.len(), 2);
                assert_eq!(
                    group
                        .biography
                        .as_ref()
                        .map(|section| section.argument.0.as_str()),
                    Some("Greek mathematician")
                );
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
Enables:
. [symbol]
  capability: f(x_) :=> x
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
Person: "Euclid"
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
    fn parses_person_group_without_biography() {
        let document = parse_ok(
            r#"
[@ada.lovelace]
Person: "Ada Lovelace"
"#,
        );

        match &document.items[0] {
            TopLevelItem::Person(group) => {
                assert_eq!(group.person.arguments[0].0, "Ada Lovelace");
                assert!(group.biography.is_none());
            }
            other => panic!("expected person group, got {other:?}"),
        }
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
    fn parses_exists_groups_without_such_that_sections() {
        let text = r#"
[\property]
States:
that:
. exists: x is \type{A}
. existsUnique: y is \type{A}
"#;

        let mut tracker = EventLog::new();
        let document = parse_document(text, &mut tracker);

        assert!(!tracker.has_errors(), "{:#?}", tracker.events());
        match &document.items[0] {
            TopLevelItem::States(states) => {
                match &states.that.arguments[0] {
                    Clause::Exists(group) => assert!(group.such_that.is_none()),
                    other => panic!("expected exists clause, got {other:?}"),
                }
                match &states.that.arguments[1] {
                    Clause::ExistsUnique(group) => assert!(group.such_that.is_none()),
                    other => panic!("expected existsUnique clause, got {other:?}"),
                }
            }
            other => panic!("expected states item, got {other:?}"),
        }
    }

    #[test]
    fn parses_refined_bindings_in_quantifier_clause_groups() {
        let text = r#"
Axiom:
then:
. exists: A is \(inductive)::set
"#;

        let mut tracker = EventLog::new();
        let document = parse_document(text, &mut tracker);

        assert!(!tracker.has_errors(), "{:#?}", tracker.events());
        match &document.items[0] {
            TopLevelItem::Axiom(group) => {
                assert!(matches!(group.then.arguments[0], Clause::Exists(_)));
            }
            other => panic!("expected axiom item, got {other:?}"),
        }
    }

    #[test]
    fn theorem_like_head_rejects_a_name() {
        for head in ["Axiom", "Theorem"] {
            let (_, diagnostics) =
                parse_with_diagnostics(&format!("{head}: \"Some Result\"\nthen: x = x\n"));
            assert!(
                diagnostics
                    .iter()
                    .any(|event| event.as_message().is_some_and(|message| {
                        message.message.contains("does not take a name")
                            && message.message.contains(head)
                    })),
                "expected `{head}:` to reject a name: {diagnostics:#?}"
            );
        }

        let (_, diagnostics) =
            parse_with_diagnostics("Corollary: \"Some Result\"\nof: \"A theorem\"\nthen: x = x\n");
        assert!(
            diagnostics.iter().any(|event| event
                .as_message()
                .is_some_and(|message| message.message.contains("does not take a name")
                    && message.message.contains("Corollary"))),
            "expected `Corollary:` to reject a name: {diagnostics:#?}"
        );
    }

    #[test]
    fn parses_quantifier_clause_groups_with_multiple_bindings() {
        let text = r#"
[\property]
States:
that:
. exists:
  . a "in" A
  . b "in" B
  suchThat:
  . a = b
. existsUnique:
  . x is \type{A}
  . y is \type{B}
  suchThat:
  . x = y
. forAll:
  . m is \type{A}
  . n is \type{B}
  then:
  . m = n
. given:
  . p is \type{A}
  . q is \type{B}
  then:
  . p = q
"#;

        let mut tracker = EventLog::new();
        let document = parse_document(text, &mut tracker);

        assert!(!tracker.has_errors(), "{:#?}", tracker.events());
        match &document.items[0] {
            TopLevelItem::States(states) => {
                match &states.that.arguments[0] {
                    Clause::Exists(group) => assert_eq!(group.exists.arguments.len(), 2),
                    other => panic!("expected exists clause, got {other:?}"),
                }
                match &states.that.arguments[1] {
                    Clause::ExistsUnique(group) => {
                        assert_eq!(group.exists_unique.arguments.len(), 2)
                    }
                    other => panic!("expected existsUnique clause, got {other:?}"),
                }
                match &states.that.arguments[2] {
                    Clause::ForAll(group) => assert_eq!(group.for_all.arguments.len(), 2),
                    other => panic!("expected forAll clause, got {other:?}"),
                }
                match &states.that.arguments[3] {
                    Clause::Given(group) => assert_eq!(group.given.arguments.len(), 2),
                    other => panic!("expected given clause, got {other:?}"),
                }
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
    fn parses_describes_function_declaration_target_with_specifies() {
        let text = r#"
[\function:on{A}:to{B}]
Describes: f(x__) ::= y_
when:
. A, B is \set
specifies:
. x__ "in" A
. y_ "in" B
"#;

        let mut tracker = EventLog::new();
        let document = parse_document(text, &mut tracker);

        assert!(!tracker.has_errors(), "{:#?}", tracker.events());
        match &document.items[0] {
            TopLevelItem::Describes(group) => {
                let DescribesTarget::Declaration(statement) = &group.describes.argument else {
                    panic!("expected declaration target");
                };
                assert!(statement.expansion.is_some());
                assert!(matches!(
                    statement.subject.kind,
                    IsSubjectKind::Forms(ref forms)
                        if matches!(
                            forms.as_slice(),
                            [IsSubjectForm::Form(FormOrDeclaration {
                                kind: FormOrDeclarationKind::FunctionDeclaration { form, .. },
                                ..
                            })] if form.magnetic_placeholder.is_some()
                        )
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
            }
            other => panic!("expected describes item, got {other:?}"),
        }
    }

    #[test]
    fn parses_outline_groups() {
        let document = parse_ok(
            r#"
Title: "Foundations"

SectionTitle: "Sets"

SubsectionTitle: "Membership"

Text: "This is prose

with another paragraph."
"#,
        );

        assert_eq!(document.items.len(), 4);

        match &document.items[0] {
            TopLevelItem::Title(group) => assert_eq!(group.title.argument.0, "Foundations"),
            other => panic!("expected title group, got {other:?}"),
        }
        match &document.items[1] {
            TopLevelItem::SectionTitle(group) => {
                assert_eq!(group.section_title.argument.0, "Sets")
            }
            other => panic!("expected section group, got {other:?}"),
        }
        match &document.items[2] {
            TopLevelItem::SubsectionTitle(group) => {
                assert_eq!(group.subsection_title.argument.0, "Membership")
            }
            other => panic!("expected subsection group, got {other:?}"),
        }
        match &document.items[3] {
            TopLevelItem::Text(group) => {
                assert_eq!(
                    group.text.argument.0,
                    "This is prose\n\nwith another paragraph."
                )
            }
            other => panic!("expected text group, got {other:?}"),
        }
    }

    #[test]
    fn parses_top_level_writing_groups() {
        let document = parse_ok(
            r#"
Writing:
. alpha :~> \alpha
. beta :~> \beta
"#,
        );

        let TopLevelItem::Writing(group) = &document.items[0] else {
            panic!("expected writing group, got {:?}", document.items[0]);
        };
        assert_eq!(group.writing.arguments.len(), 2);
        assert_eq!(group.writing.arguments[0].body, r#"\alpha"#);
        assert!(matches!(
            group.writing.arguments[0].form.kind,
            FormOrDeclarationKind::Name(ref name) if name == "alpha"
        ));
    }

    #[test]
    fn unescapes_quotes_in_top_level_text() {
        let document = parse_ok(
            r#"
Text: "A \"quoted\" word and \alpha."
"#,
        );

        let TopLevelItem::Text(group) = &document.items[0] else {
            panic!("expected text group, got {:?}", document.items[0]);
        };
        assert_eq!(group.text.argument.0, r#"A "quoted" word and \alpha."#);
    }

    #[test]
    fn rejects_top_level_writing_aliases_with_non_name_lhs() {
        let (document, messages) = parse_with_diagnostics(
            r#"
Writing:
. f(x_) :~> x
"#,
        );

        assert!(document.items.is_empty());
        assert!(messages.iter().any(|event| {
            event
                .as_message()
                .is_some_and(|message| message.message.contains("Writing aliases must use a name"))
        }));
    }

    // ===============================[ theorems ]=====================================

    #[test]
    fn parses_theorem_like_groups_and_clause_variants() {
        let document = parse_ok(
            r#"
[\axiom]
Axiom:
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
. [logic.have]
  have:
  . y = y
  iff:
  . x = x
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
of:
. "Previous theorem"
then:
. [logic.one]
  oneOf:
  . x = x
  . y = y
"#,
        );

        assert_eq!(document.items.len(), 3);

        match &document.items[0] {
            TopLevelItem::Axiom(group) => {
                assert!(group.heading.is_some());
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
    }

    #[test]
    fn rejects_legacy_iff_then_clause_groups() {
        let (_document, messages) = parse_with_diagnostics(
            r#"
Theorem:
then:
. iff:
  . x = x
  then:
  . y = y
"#,
        );

        assert!(messages.iter().any(|event| {
            event
                .as_message()
                .is_some_and(|message| message.message.contains("Unexpected clause group `iff`"))
        }));
    }
}

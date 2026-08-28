use std::collections::{HashMap, VecDeque};

use crate::events::EventLog;
use crate::frontend::formulation::ast::{
    DeclarationRelation, ExpressionKind, FormOrDeclaration, FormOrDeclarationKind, WritingAlias,
};
use crate::frontend::formulation::{
    ParseError as FormulationParseError, parse_author_header, parse_command_header,
    parse_expression, parse_expression_alias, parse_expression_binding, parse_form_or_declaration,
    parse_is_via_statement, parse_label_header, parse_refined_declaration_statement,
    parse_resource_header, parse_spec_operator_alias, parse_topic_header, parse_type_expression,
    parse_writing_alias, split_via_view,
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

    parse_document_from_groups(&groups, tracker)
}

/// Recognizes an already proto-parsed source document.
///
/// Collection commands need both the proto groups (for ids and rendering) and
/// the structural document. Keeping this entry point separate lets those
/// callers pay for the proto parse only once.
pub(crate) fn parse_document_from_groups(
    groups: &[ProtoGroup],
    tracker: &mut EventLog,
) -> Document {
    let mut items = Vec::new();
    for group in groups {
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

/// Parses an item accepted by a `Declares:` group's `specifies:` and related sections.
///
/// `is ... via ...` is more specific, so it is attempted before the broader
/// `is`/spec parser. The `is` relation may name a refined command
/// (`* is \(associative)::binary.operation:on{X}`), so a value can be specified
/// or extended to a refinement of a type, not just a bare command.
pub(in crate::frontend::structural::parser) fn parse_is_or_via_item(
    input: &str,
) -> Result<IsOrViaItem, FormulationParseError> {
    if let Some((label, inner)) = split_labeled_specification(input) {
        let item = parse_is_or_via_item(inner)?;
        return Ok(IsOrViaItem::Labeled {
            label,
            item: Box::new(item),
        });
    }
    if let Ok(item) = parse_is_via_statement(input) {
        return Ok(IsOrViaItem::IsVia(item));
    }
    parse_refined_declaration_statement(input).map(IsOrViaItem::Declaration)
}

/// Recognizes a `[:label:]`-labeled grouped specification such as
/// `(.*_1 is \foo.)[:1:]`. Returns the label parts and the source text of the
/// grouped inner specification (e.g. `*_1 is \foo`) so the caller can re-parse it
/// with the declaration parser (which, unlike the expression parser, accepts an
/// operator subject like `*_1`). Returns `None` when `input` is not a labeled
/// grouped specification.
fn split_labeled_specification(input: &str) -> Option<(Vec<String>, &str)> {
    // Strip the trailing `[:label.parts:]` token.
    let body = input.trim().strip_suffix(":]")?;
    let label_start = body.rfind("[:")?;
    let label_body = &body[label_start + 2..];
    if label_body.is_empty()
        || !label_body.split('.').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        })
    {
        return None;
    }
    let parts = label_body.split('.').map(str::to_owned).collect();
    // Strip the enclosing `(. .)` (or `( )`) grouping that carries the label.
    let grouped = body[..label_start].trim();
    let inner = grouped
        .strip_prefix("(.")
        .and_then(|rest| rest.strip_suffix(".)"))
        .or_else(|| {
            grouped
                .strip_prefix('(')
                .and_then(|rest| rest.strip_suffix(')'))
        })?;
    Some((parts, inner.trim()))
}

/// Parses the argument of a `Declares:` section.
///
/// The target may state the type the definition extends with an `is`/spec
/// relation (`A is \set`), and an `is` relation may be followed by the `via`
/// view used to regard the defined item as that type
/// (`G ::= (X, *, e) is \monoid via (X, *)`).
fn parse_declares_section(input: &str) -> Result<DeclaresSection, FormulationParseError> {
    let (target_text, via_text) = split_via_view(input);
    let via = via_text.map(parse_form_or_declaration).transpose()?;
    let argument = parse_declares_target(target_text)?;
    if via.is_some() && !declares_target_states_is(&argument) {
        return Err(FormulationParseError::custom(
            "`via` requires the `Declares:` target to name the type it extends, \
             as in `G ::= (X, *, e) is \\monoid via (X, *)`",
        ));
    }

    Ok(DeclaresSection { argument, via })
}

/// Whether a `Declares:` target names an extended type with an `is` relation.
fn declares_target_states_is(target: &DeclaresTarget) -> bool {
    matches!(
        target,
        DeclaresTarget::Declaration(statement)
            if matches!(statement.relation, Some(DeclarationRelation::Is(_)))
    )
}

/// Whether a `Declares:` target states the type it extends at all, by an `is` or
/// a specification relation. Such a target may not also have an `extends:`
/// section: the two spellings say the same thing.
fn declares_target_states_extends(target: &DeclaresTarget) -> bool {
    matches!(
        target,
        DeclaresTarget::Declaration(statement) if statement.relation.is_some()
    )
}

/// Parses one `extends:` clause: the type extended, with an optional `via` view.
///
/// This is the same syntax a `Declares:` target uses for a single clause, minus
/// the bare-form case, since a clause always states a type.
fn parse_extends_item(input: &str) -> Result<ExtendsItem, FormulationParseError> {
    let (statement_text, via_text) = split_via_view(input);
    let via = via_text.map(parse_form_or_declaration).transpose()?;
    let statement = parse_refined_declaration_statement(statement_text)?;
    if via.is_some() && !matches!(statement.relation, Some(DeclarationRelation::Is(_))) {
        return Err(FormulationParseError::custom(
            "`via` requires an `is` clause to name the extended type, \
             as in `G is \\monoid via (X, *)`",
        ));
    }

    Ok(ExtendsItem { statement, via })
}

/// Parses a `Declares:` target: the described form, or a declaration naming the
/// type it extends. The `is` relation may name a refined command, so a
/// definition can extend a refinement of a type and not only a bare command.
fn parse_declares_target(input: &str) -> Result<DeclaresTarget, FormulationParseError> {
    if let Ok(form) = parse_form_or_declaration(input) {
        return Ok(DeclaresTarget::Form(form));
    }

    parse_refined_declaration_statement(input).map(DeclaresTarget::Declaration)
}

/// Parses the restricted target of a documented mapping `writing:` rule.
fn parse_mapping_writing_target(
    input: &str,
) -> Result<MappingWritingTarget, FormulationParseError> {
    if let Ok(form) = parse_form_or_declaration(input)
        && matches!(form.kind, FormOrDeclarationKind::FunctionDeclaration { .. })
    {
        return Ok(MappingWritingTarget::Mapping(form));
    }

    if let Ok(expression) = parse_expression(input)
        && matches!(expression.kind, ExpressionKind::FunctionCall { .. })
    {
        return Ok(MappingWritingTarget::Invocation(expression));
    }

    Err(FormulationParseError::custom(
        "expected a mapping form such as `x(i_)` or its named invocation form such as `x(i)`",
    ))
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
                if let Ok(statement) = parse_refined_declaration_statement(text) {
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

/// Parses one or more resource citations from a `References:` section.
///
/// References historically used formulation entries (`. $book.name`). Quoted
/// entries are accepted as well because citations are identifiers rather than
/// mathematical expressions and are commonly authored as strings.
fn parse_required_resource_references(
    section: &ProtoSection,
    tracker: &mut EventLog,
) -> Option<OneOrMore<crate::frontend::formulation::ast::ResourceHeader>> {
    let starting_issue_count = tracker.issue_count();
    let mut references = Vec::new();

    for entry in section_entries(section) {
        let (text, row) = match entry {
            SectionEntry::Inline { text, row } | SectionEntry::Formulation { text, row } => {
                (text.to_owned(), row)
            }
            SectionEntry::Text { text, row } => (
                strip_quoted_text(text).unwrap_or_else(|| text.to_owned()),
                row,
            ),
            SectionEntry::Group { row, .. } => {
                tracker.user_error_at_row(
                    Some(ORIGIN),
                    row,
                    "Expected a resource reference in section `References`",
                );
                continue;
            }
        };

        let text = strip_quoted_text(&text).unwrap_or(text);
        match parse_resource_header(&text) {
            Ok(reference) => references.push(reference),
            Err(error) => tracker.user_error_at_row(
                Some(ORIGIN),
                row,
                format!("Invalid References formulation: {error}"),
            ),
        }
    }

    one_or_more(references.into(), || {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(
                Some(ORIGIN),
                section.metadata.row,
                "Expected References formulations",
            );
        }
    })
}

/// Parses zero or more formulations from an optional section.
///
/// Inline section arguments and formulation arguments are accepted.  Text and
/// nested groups are diagnosed because callers requested formulation content.
/// Parses `specifies:` items: inline `is`/`is … via …` specifications, plus
/// `have:` groups, optionally with `asserting:`, standing in for a specification.
fn parse_required_specify_items(
    section: &ProtoSection,
    tracker: &mut EventLog,
) -> Option<OneOrMore<IsOrViaItem>> {
    let starting_issue_count = tracker.issue_count();
    let mut result = Vec::new();
    for entry in section_entries(section) {
        match entry {
            SectionEntry::Inline { text, row } | SectionEntry::Formulation { text, row } => {
                match parse_is_or_via_item(text) {
                    Ok(item) => result.push(item),
                    Err(error) => tracker.user_error_at_row(
                        Some(ORIGIN),
                        row,
                        format!("Invalid specifies formulation: {error}"),
                    ),
                }
            }
            SectionEntry::Group { group, row } => {
                if group.sections.first().map(|section| section.label.as_str()) == Some("have") {
                    if let Some(have) = parse_have_group(group, tracker) {
                        result.push(IsOrViaItem::Have(Box::new(have)));
                    }
                } else {
                    tracker.user_error_at_row(
                        Some(ORIGIN),
                        row,
                        "Expected a specification or a `have:` group in `specifies`".to_owned(),
                    );
                }
            }
            SectionEntry::Text { row, .. } => {
                tracker.user_error_at_row(
                    Some(ORIGIN),
                    row,
                    "Expected formulation in section `specifies`".to_owned(),
                );
            }
        }
    }
    one_or_more(result.into(), || {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(
                Some(ORIGIN),
                section.metadata.row,
                "Expected specifies formulations".to_owned(),
            );
        }
    })
}

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

/// Parses a `let:` clause group.
///
/// The leading section introduces local bindings that are available only while
/// checking the optional `where:` guard and required `then:` section. Facts
/// established by `where:` are available while checking `then:`.
pub(super) fn parse_let_clause(group: &ProtoGroup, tracker: &mut EventLog) -> Option<LetGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("let", &group.sections, tracker, &["let", "where?", "then"])?;
    Some(LetGroup {
        heading,
        let_: LetSection {
            arguments: parse_required_formulations(
                section(&sections, "let")?,
                "let",
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

/// Dispatches a `have:` clause group. `have:`/`iff:` is the shorthand iff
/// clause; every other `have:` shape is the optionally-asserting escape hatch.
fn parse_have_or_assertion(group: &ProtoGroup, tracker: &mut EventLog) -> Option<Clause> {
    let has_assertion_section = group
        .sections
        .iter()
        .any(|section| matches!(section.label.as_str(), "asserting" | "because" | "by"));
    let has_iff = group.sections.iter().any(|section| section.label == "iff");
    if has_assertion_section || !has_iff {
        parse_have_group(group, tracker).map(|group| Clause::Have(Box::new(group)))
    } else {
        parse_have_clause(group, tracker).map(Clause::Iff)
    }
}

/// Parses a `have:`/`asserting:`/`because?:`/`by?:` group.
pub(super) fn parse_have_group(group: &ProtoGroup, tracker: &mut EventLog) -> Option<HaveGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "have",
        &group.sections,
        tracker,
        &["have", "asserting?", "because?", "by?"],
    )?;
    Some(HaveGroup {
        heading,
        have: HaveSection {
            arguments: parse_required_clauses(section(&sections, "have")?, "have", tracker)?,
        },
        asserting: sections.get("asserting").copied().and_then(|section| {
            parse_required_clauses(section, "asserting", tracker)
                .map(|arguments| AssertingSection { arguments })
        }),
        because: sections.get("because").copied().and_then(|section| {
            parse_required_clauses(section, "because", tracker)
                .map(|arguments| BecauseSection { arguments })
        }),
        by: sections.get("by").copied().and_then(|section| {
            parse_required_formulations(section, "by", tracker, parse_expression)
                .map(|arguments| HaveBySection { arguments })
        }),
    })
}

/// Parses a `piecewise:` clause group.
///
/// The leading `piecewise:` section takes no arguments, followed by required `if:`
/// and `then:` sections, zero or more `(elseIf: then:)` section pairs, and an
/// optional `else:` section.
pub(super) fn parse_piecewise_clause(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<PiecewiseGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let first = group.sections.first()?;
    if first.label != "piecewise" {
        tracker.user_error_at_row(
            Some(ORIGIN),
            first.metadata.row,
            format!("Expected `piecewise` section but found `{}`", first.label),
        );
        return None;
    }
    ensure_empty_section(first, "piecewise", tracker);

    let pattern = "piecewise:\nif:\nthen:\n(elseIf:\nthen:)*\nelse?:";

    let mut index = 1;
    let Some(if_section) = group.sections.get(index) else {
        tracker.user_error_at_row(
            Some(ORIGIN),
            first.metadata.row,
            format!("For piecewise pattern:\n\n{pattern}\n\nExpected section `if`"),
        );
        return None;
    };
    if if_section.label != "if" {
        tracker.user_error_at_row(
            Some(ORIGIN),
            if_section.metadata.row,
            format!(
                "For piecewise pattern:\n\n{pattern}\n\nExpected `if` but found `{}`",
                if_section.label
            ),
        );
        return None;
    }
    let if_ = IfSection {
        arguments: parse_required_clauses(if_section, "if", tracker)?,
    };
    index += 1;

    let Some(then_section) = group.sections.get(index) else {
        tracker.user_error_at_row(
            Some(ORIGIN),
            if_section.metadata.row,
            format!("For piecewise pattern:\n\n{pattern}\n\nExpected section `then`"),
        );
        return None;
    };
    if then_section.label != "then" {
        tracker.user_error_at_row(
            Some(ORIGIN),
            then_section.metadata.row,
            format!(
                "For piecewise pattern:\n\n{pattern}\n\nExpected `then` but found `{}`",
                then_section.label
            ),
        );
        return None;
    }
    let then = ThenSection {
        arguments: parse_required_clauses(then_section, "then", tracker)?,
    };
    index += 1;

    let mut else_if = Vec::new();
    while let Some(section) = group.sections.get(index) {
        if section.label != "elseIf" {
            break;
        }
        let else_if_section = ElseIfSection {
            arguments: parse_required_clauses(section, "elseIf", tracker)?,
        };
        index += 1;
        let Some(branch_then_section) = group.sections.get(index) else {
            tracker.user_error_at_row(
                Some(ORIGIN),
                section.metadata.row,
                format!("For piecewise pattern:\n\n{pattern}\n\nExpected `then` section after `elseIf`"),
            );
            return None;
        };
        if branch_then_section.label != "then" {
            tracker.user_error_at_row(
                Some(ORIGIN),
                branch_then_section.metadata.row,
                format!(
                    "For piecewise pattern:\n\n{pattern}\n\nExpected `then` section after `elseIf` but found `{}`",
                    branch_then_section.label
                ),
            );
            return None;
        }
        let branch_then = ThenSection {
            arguments: parse_required_clauses(branch_then_section, "then", tracker)?,
        };
        index += 1;
        else_if.push(PiecewiseElseIf {
            else_if: else_if_section,
            then: branch_then,
        });
    }

    let else_ = match group.sections.get(index) {
        Some(section) if section.label == "else" => {
            index += 1;
            parse_required_clauses(section, "else", tracker)
                .map(|arguments| ElseSection { arguments })
        }
        _ => None,
    };

    if let Some(unexpected) = group.sections.get(index) {
        tracker.user_error_at_row(
            Some(ORIGIN),
            unexpected.metadata.row,
            format!(
                "For piecewise pattern:\n\n{pattern}\n\nUnexpected section `{}`",
                unexpected.label
            ),
        );
        return None;
    }

    Some(PiecewiseGroup {
        heading,
        if_,
        then,
        else_if,
        else_,
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

/// Dispatches nested `Enables:` groups to capability, from, or view parsers.
pub(super) fn parse_enables_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<EnablesItem> {
    match first_section_label(group)? {
        "capability" => parse_capability(group, tracker)
            .map(Box::new)
            .map(EnablesItem::Capability),
        "from" => parse_from_group(group, tracker),
        "view" => parse_enables_view_group(group, tracker)
            .map(Box::new)
            .map(EnablesItem::View),
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

/// Dispatches the four global numeric literal/index categories in `Specify:`.
pub(super) fn parse_specify_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<SpecifyItem> {
    match first_section_label(group)? {
        "decimal" => {
            parse_numeric_specification(group, "decimal", tracker).map(SpecifyItem::Decimal)
        }
        "zeroOrPositiveInt" => parse_numeric_specification(group, "zeroOrPositiveInt", tracker)
            .map(SpecifyItem::ZeroOrPositiveInt),
        "positiveInt" => {
            parse_numeric_specification(group, "positiveInt", tracker).map(SpecifyItem::PositiveInt)
        }
        "int" => parse_numeric_specification(group, "int", tracker).map(SpecifyItem::Int),
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
        "let" => parse_let_clause(group, tracker).map(Clause::Let),
        "if" => parse_if_clause(group, tracker).map(Clause::If),
        "have" => parse_have_or_assertion(group, tracker),
        "piecewise" => parse_piecewise_clause(group, tracker).map(Clause::Piecewise),
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
            parse_refined_declaration_statement,
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

/// Parses a zero-argument `view:` group inside `Enables:`.
pub(in crate::frontend::structural::parser) fn parse_enables_view_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<EnablesViewGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "view",
        &group.sections,
        tracker,
        &["view", "as", "signifies?"],
    )?;
    parse_marker_section(&sections, "view", tracker);

    Some(EnablesViewGroup {
        heading,
        as_: ViewAsSection {
            argument: parse_required_formulation(
                section(&sections, "as")?,
                "as",
                tracker,
                parse_refined_declaration_statement,
            )?,
        },
        signifies: sections.get("signifies").copied().and_then(|section| {
            parse_required_clause(section, "signifies", tracker)
                .map(|argument| ViewSignifiesSection { argument })
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
/// The `writing:` section identifies either a mapping definition form or its
/// named invocation form; `as:` stores its rendering template.
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
                parse_mapping_writing_target,
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

/// Parses a `notes:` documentation item (one or more prose reminders).
pub(in crate::frontend::structural::parser) fn parse_notes(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<NotesGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("notes", &group.sections, tracker, &["notes"])?;
    Some(NotesGroup {
        heading,
        notes: NotesSection {
            arguments: parse_required_open_texts(section(&sections, "notes")?, "notes", tracker)?,
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

// ===============================[ specify items ]=====================================

fn parse_numeric_specification(
    group: &ProtoGroup,
    label: &str,
    tracker: &mut EventLog,
) -> Option<NumericSpecificationGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections(label, &group.sections, tracker, &[label, "is"])?;
    ensure_empty_section(section(&sections, label)?, label, tracker);
    Some(NumericSpecificationGroup {
        is_: NumericSpecificationIsSection {
            argument: parse_required_formulation(
                section(&sections, "is")?,
                "is",
                tracker,
                |input| parse_type_expression(input, false),
            )?,
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
        "Declares" => parse_declares(group, tracker).map(TopLevelItem::Declares),
        "Defines" => parse_defines(group, tracker).map(TopLevelItem::Defines),
        "Realizes" => parse_realizes(group, tracker).map(TopLevelItem::Realizes),
        "Refines" => parse_refines(group, tracker).map(TopLevelItem::Refines),
        "States" => parse_states(group, tracker).map(TopLevelItem::States),
        "Axiom" => parse_axiom(group, tracker).map(TopLevelItem::Axiom),
        "Theorem" => parse_theorem(group, tracker).map(TopLevelItem::Theorem),
        "Conjecture" => parse_conjecture(group, tracker).map(TopLevelItem::Conjecture),
        "Person" => parse_person(group, tracker).map(TopLevelItem::Person),
        "Resource" => parse_resource(group, tracker).map(TopLevelItem::Resource),
        "Specify" => parse_specify(group, tracker).map(TopLevelItem::Specify),
        "Relation" => parse_relation(group, tracker).map(TopLevelItem::Relation),
        "Equivalent" => parse_equivalent(group, tracker).map(TopLevelItem::Equivalent),
        "Topic" => parse_topic(group, tracker).map(TopLevelItem::Topic),
        "TextTheorem" => {
            parse_text_item(group, tracker, TextItemKind::Theorem).map(TopLevelItem::TextItem)
        }
        "TextAxiom" => {
            parse_text_item(group, tracker, TextItemKind::Axiom).map(TopLevelItem::TextItem)
        }
        "TextConjecture" => {
            parse_text_item(group, tracker, TextItemKind::Conjecture).map(TopLevelItem::TextItem)
        }
        "TextDefinition" => {
            parse_text_item(group, tracker, TextItemKind::Definition).map(TopLevelItem::TextItem)
        }
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

/// Parses one or more required quoted writing aliases.
///
/// Each entry is a double-quoted string whose contents form a `name :~> body`
/// rule, for example `. "pi :~> \pi"`. Quotes are required because the body to
/// the right of `:~>` may be arbitrary LaTeX that would otherwise be misparsed
/// as formulation syntax, and the left-hand side must be a plain `Name`. Shared
/// by the collection-wide `Writing:` group and each item-level `Writing:` section.
///
/// Returns `None` (after reporting the offending entries) if any entry is
/// unquoted, fails to parse, or uses a non-name left-hand side.
fn parse_required_writing_aliases(
    section: &ProtoSection,
    tracker: &mut EventLog,
) -> Option<OneOrMore<WritingAlias>> {
    let starting_issue_count = tracker.issue_count();
    let mut result = Vec::new();
    let mut all_valid = true;
    for entry in section_entries(section) {
        match entry {
            SectionEntry::Inline { text, row } | SectionEntry::Text { text, row } => {
                let Some(inner) = strip_quoted_text(text) else {
                    tracker.user_error_at_row(
                        Some(ORIGIN),
                        row,
                        format!("Expected a quoted Writing alias, found `{text}`"),
                    );
                    all_valid = false;
                    continue;
                };
                match parse_writing_alias(&inner) {
                    Ok(alias) if matches!(alias.form.kind, FormOrDeclarationKind::Name(_)) => {
                        result.push(alias);
                    }
                    Ok(_) => {
                        tracker.user_error_at_row(
                            Some(ORIGIN),
                            row,
                            "Writing aliases must use a name on the left of `:~>`",
                        );
                        all_valid = false;
                    }
                    Err(error) => {
                        tracker.user_error_at_row(
                            Some(ORIGIN),
                            row,
                            format!("Invalid Writing alias: {error}"),
                        );
                        all_valid = false;
                    }
                }
            }
            SectionEntry::Formulation { row, .. } => {
                tracker.user_error_at_row(
                    Some(ORIGIN),
                    row,
                    "Expected a quoted Writing alias, found formulation",
                );
                all_valid = false;
            }
            SectionEntry::Group { row, .. } => {
                tracker.user_error_at_row(
                    Some(ORIGIN),
                    row,
                    "Expected a quoted Writing alias, found nested group",
                );
                all_valid = false;
            }
        }
    }
    if !all_valid {
        return None;
    }
    one_or_more(result.into(), || {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(
                Some(ORIGIN),
                section.metadata.row,
                "Expected Writing aliases",
            );
        }
    })
}

/// Parses an optional item-level `Writing:` section.
///
/// The section appears after `Aliases:` on definition- and result-like items and
/// overrides, for that item only, how the collection-wide `Writing:` group renders
/// the named identifiers.
fn parse_optional_item_writing(
    sections: &HashMap<String, &ProtoSection>,
    tracker: &mut EventLog,
) -> Option<ItemWritingSection> {
    sections.get("Writing").copied().and_then(|section| {
        parse_required_writing_aliases(section, tracker)
            .map(|arguments| ItemWritingSection { arguments })
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
    let arguments = parse_required_writing_aliases(writing_section, tracker)?;

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
            "Documented?",
            "Justification?",
            "Aliases?",
            "Writing?",
            "References?",
            "Metadata?",
            "Id?",
        ],
    )?;

    Some(DisambiguatesGroup {
        heading,
        branches,
        else_,
        justification: trailing.get("Justification").copied().and_then(|section| {
            parse_required_groups(section, "Justification", tracker, parse_have_group)
                .map(|arguments| JustificationSection { arguments })
        }),
        documented: trailing.get("Documented").copied().and_then(|section| {
            parse_required_groups(section, "Documented", tracker, parse_documented_item_group)
                .map(|arguments| DocumentedSection { arguments })
        }),
        aliases: trailing.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", tracker, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        writing: parse_optional_item_writing(&trailing, tracker),
        references: trailing.get("References").copied().and_then(|section| {
            parse_required_resource_references(section, tracker)
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
        | FormOrDeclarationKind::MappingParameter { .. }
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

/// Parses a command-backed `Declares:` group.
///
/// This enforces the full `Declares` section order and converts each optional
/// nested section into its typed representation.  Formulation sections are
/// delegated to the formulation parser while clause/nested sections recurse
/// through structural helpers.
pub(in crate::frontend::structural::parser) fn parse_declares(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<DeclaresGroup> {
    let heading = parse_required_command_heading(group, tracker)?;
    let sections = identify_sections(
        "Declares",
        &group.sections,
        tracker,
        &[
            "Declares",
            "using?",
            "when?",
            "extends?",
            "specifies?",
            "satisfies?",
            "Requires?",
            "Enables?",
            "Documented?",
            "Justification?",
            "Aliases?",
            "Writing?",
            "References?",
            "Metadata?",
            "Id?",
        ],
    )?;

    let declares = parse_required_formulation(
        section(&sections, "Declares")?,
        "Declares",
        tracker,
        parse_declares_section,
    )?;
    let extends = sections.get("extends").copied().and_then(|section| {
        if declares_target_states_extends(&declares.argument) {
            tracker.user_error_at_row(
                Some(ORIGIN),
                section.metadata.row,
                "A `Declares:` target that names the type it extends cannot also have an \
                 `extends:` section; use one or the other"
                    .to_owned(),
            );
            return None;
        }
        parse_required_formulations(section, "extends", tracker, parse_extends_item)
            .map(|arguments| ExtendsSection { arguments })
    });

    Some(DeclaresGroup {
        heading,
        declares,
        extends,
        using: sections.get("using").copied().and_then(|section| {
            parse_required_formulations(
                section,
                "using",
                tracker,
                parse_refined_declaration_statement,
            )
            .map(|arguments| UsingSection { arguments })
        }),
        when: sections.get("when").copied().and_then(|section| {
            parse_required_clauses(section, "when", tracker)
                .map(|arguments| WhenSection { arguments })
        }),
        specifies: sections.get("specifies").copied().and_then(|section| {
            parse_required_specify_items(section, tracker)
                .map(|arguments| DeclaresSpecifiesSection { arguments })
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
        justification: sections.get("Justification").copied().and_then(|section| {
            parse_required_groups(section, "Justification", tracker, parse_have_group)
                .map(|arguments| JustificationSection { arguments })
        }),
        documented: sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(section, "Documented", tracker, parse_documented_item_group)
                .map(|arguments| DocumentedSection { arguments })
        }),
        aliases: sections.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", tracker, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        writing: parse_optional_item_writing(&sections, tracker),
        references: sections.get("References").copied().and_then(|section| {
            parse_required_resource_references(section, tracker)
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
/// statements and support the same auxiliary sections as `Declares`, except
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
            "abstractly?",
            "using?",
            "when?",
            "specifies?",
            "expresses?",
            "Requires?",
            "Enables?",
            "Documented?",
            "Justification?",
            "Aliases?",
            "Writing?",
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
                parse_refined_declaration_statement,
            )?,
        },
        abstractly: parse_marker_section(&sections, "abstractly", tracker),
        using: sections.get("using").copied().and_then(|section| {
            parse_required_formulations(
                section,
                "using",
                tracker,
                parse_refined_declaration_statement,
            )
            .map(|arguments| UsingSection { arguments })
        }),
        when: sections.get("when").copied().and_then(|section| {
            parse_required_clauses(section, "when", tracker)
                .map(|arguments| WhenSection { arguments })
        }),
        specifies: sections.get("specifies").copied().and_then(|section| {
            parse_required_specify_items(section, tracker)
                .map(|arguments| DefinesSpecifiesSection { arguments })
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
        justification: sections.get("Justification").copied().and_then(|section| {
            parse_required_groups(section, "Justification", tracker, parse_have_group)
                .map(|arguments| JustificationSection { arguments })
        }),
        documented: sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(section, "Documented", tracker, parse_documented_item_group)
                .map(|arguments| DocumentedSection { arguments })
        }),
        aliases: sections.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", tracker, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        writing: parse_optional_item_writing(&sections, tracker),
        references: sections.get("References").copied().and_then(|section| {
            parse_required_resource_references(section, tracker)
                .map(|arguments| ReferencesSection { arguments })
        }),
        metadata: sections.get("Metadata").copied().and_then(|section| {
            parse_required_groups(section, "Metadata", tracker, parse_metadata_item_group)
                .map(|arguments| MetadataSection { arguments })
        }),
    })
}

/// Extracts the optional `implicitly:`/`explicitly:` marker from a `Refines:`
/// group's sections.
///
/// Both are zero-argument marker sections and are mutually exclusive; violations
/// are reported to `tracker` (and the first-declared marker is still returned so
/// that later semantic checks can proceed).
fn parse_refinement_kind(
    sections: &HashMap<String, &ProtoSection>,
    tracker: &mut EventLog,
) -> Option<RefinementKind> {
    let implicitly = section(sections, "implicitly");
    let explicitly = section(sections, "explicitly");

    for (label, marker) in [("implicitly", implicitly), ("explicitly", explicitly)] {
        if let Some(marker) = marker
            && (marker.inline_argument.is_some() || !marker.arguments.is_empty())
        {
            tracker.user_error_at_row(
                Some(ORIGIN),
                marker.metadata.row,
                format!("`{label}:` is a marker section and takes no arguments"),
            );
        }
    }

    match (implicitly, explicitly) {
        (Some(_), Some(explicitly)) => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                explicitly.metadata.row,
                "A `Refines:` may specify at most one of `implicitly:` or `explicitly:`".to_owned(),
            );
            Some(RefinementKind::Implicit)
        }
        (Some(_), None) => Some(RefinementKind::Implicit),
        (None, Some(_)) => Some(RefinementKind::Explicit),
        (None, None) => None,
    }
}

/// Reads a zero-argument marker section such as `abstractly:`, reporting any
/// content it was given.
fn parse_marker_section(
    sections: &HashMap<String, &ProtoSection>,
    label: &str,
    tracker: &mut EventLog,
) -> bool {
    let Some(marker) = section(sections, label) else {
        return false;
    };
    if marker.inline_argument.is_some() || !marker.arguments.is_empty() {
        tracker.user_error_at_row(
            Some(ORIGIN),
            marker.metadata.row,
            format!("`{label}:` is a marker section and takes no arguments"),
        );
    }
    true
}

/// Parses a command-backed `Realizes:` group.
///
/// A `Realizes:` supplies concrete values for the symbols an abstract
/// `Defines:` left open. Its target names the declaration being realized
/// (`Realizes: Nb := \naturals`), and it shares the rest of its sections with
/// `Defines:`.
pub(in crate::frontend::structural::parser) fn parse_realizes(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<RealizesGroup> {
    let heading = parse_required_command_heading(group, tracker)?;
    let sections = identify_sections(
        "Realizes",
        &group.sections,
        tracker,
        &[
            "Realizes",
            "using?",
            "when?",
            "specifies?",
            "expresses?",
            "Requires?",
            "Enables?",
            "Documented?",
            "Justification?",
            "Aliases?",
            "Writing?",
            "References?",
            "Metadata?",
            "Id?",
        ],
    )?;

    Some(RealizesGroup {
        heading,
        realizes: RealizesSection {
            argument: parse_required_formulation(
                section(&sections, "Realizes")?,
                "Realizes",
                tracker,
                parse_refined_declaration_statement,
            )?,
        },
        using: sections.get("using").copied().and_then(|section| {
            parse_required_formulations(
                section,
                "using",
                tracker,
                parse_refined_declaration_statement,
            )
            .map(|arguments| UsingSection { arguments })
        }),
        when: sections.get("when").copied().and_then(|section| {
            parse_required_clauses(section, "when", tracker)
                .map(|arguments| WhenSection { arguments })
        }),
        specifies: sections.get("specifies").copied().and_then(|section| {
            parse_required_specify_items(section, tracker)
                .map(|arguments| DefinesSpecifiesSection { arguments })
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
        justification: sections.get("Justification").copied().and_then(|section| {
            parse_required_groups(section, "Justification", tracker, parse_have_group)
                .map(|arguments| JustificationSection { arguments })
        }),
        documented: sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(section, "Documented", tracker, parse_documented_item_group)
                .map(|arguments| DocumentedSection { arguments })
        }),
        aliases: sections.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", tracker, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        writing: parse_optional_item_writing(&sections, tracker),
        references: sections.get("References").copied().and_then(|section| {
            parse_required_resource_references(section, tracker)
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
            "implicitly?",
            "explicitly?",
            "using?",
            "when?",
            "specifies?",
            "satisfies?",
            "Requires?",
            "Enables?",
            "Documented?",
            "Justification?",
            "Aliases?",
            "Writing?",
            "References?",
            "Metadata?",
            "Id?",
        ],
    )?;

    let refinement_kind = parse_refinement_kind(&sections, tracker);

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
        refinement_kind,
        using: sections.get("using").copied().and_then(|section| {
            parse_required_formulations(
                section,
                "using",
                tracker,
                parse_refined_declaration_statement,
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
        requires: sections.get("Requires").copied().and_then(|section| {
            parse_required_groups(section, "Requires", tracker, parse_requires_item_group)
                .map(|arguments| RequiresSection { arguments })
        }),
        enables: sections.get("Enables").copied().and_then(|section| {
            parse_required_groups(section, "Enables", tracker, parse_enables_item_group)
                .map(|arguments| EnablesSection { arguments })
        }),
        justification: sections.get("Justification").copied().and_then(|section| {
            parse_required_groups(section, "Justification", tracker, parse_have_group)
                .map(|arguments| JustificationSection { arguments })
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
        writing: parse_optional_item_writing(&sections, tracker),
        references: sections.get("References").copied().and_then(|section| {
            parse_required_resource_references(section, tracker)
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
            "Documented?",
            "Justification?",
            "Aliases?",
            "Writing?",
            "References?",
            "Metadata?",
            "Id?",
        ],
    )?;

    ensure_empty_section(section(&sections, "States")?, "States", tracker);

    Some(StatesGroup {
        heading,
        using: sections.get("using").copied().and_then(|section| {
            parse_required_formulations(
                section,
                "using",
                tracker,
                parse_refined_declaration_statement,
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
        justification: sections.get("Justification").copied().and_then(|section| {
            parse_required_groups(section, "Justification", tracker, parse_have_group)
                .map(|arguments| JustificationSection { arguments })
        }),
        documented: sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(section, "Documented", tracker, parse_documented_item_group)
                .map(|arguments| DocumentedSection { arguments })
        }),
        aliases: sections.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", tracker, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        writing: parse_optional_item_writing(&sections, tracker),
        references: sections.get("References").copied().and_then(|section| {
            parse_required_resource_references(section, tracker)
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
            "Documented?",
            "Justification?",
            "References?",
            "Id?",
        ],
    )?;

    ensure_empty_section(section(&sections, "Equivalent")?, "Equivalent", tracker);

    Some(EquivalentGroup {
        heading,
        using: sections.get("using").copied().and_then(|section| {
            parse_required_formulations(
                section,
                "using",
                tracker,
                parse_refined_declaration_statement,
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
        justification: sections.get("Justification").copied().and_then(|section| {
            parse_required_groups(section, "Justification", tracker, parse_have_group)
                .map(|arguments| JustificationSection { arguments })
        }),
        documented: sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(section, "Documented", tracker, parse_documented_item_group)
                .map(|arguments| DocumentedSection { arguments })
        }),
        references: sections.get("References").copied().and_then(|section| {
            parse_required_resource_references(section, tracker)
                .map(|arguments| ReferencesSection { arguments })
        }),
    })
}

// ===============================[ relation ]=====================================

/// Parses a top-level `Relation:` item.
///
/// A `Relation:` states a bidirectional relationship between the two concepts
/// declared in the required `between:` and `and:` sections. It takes no command
/// heading and registers no view rule.
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
            "specifies?",
            "Documented?",
            "Justification?",
            "Aliases?",
            "Writing?",
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
                parse_refined_declaration_statement,
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
        specifies: sections.get("specifies").copied().and_then(|section| {
            parse_required_relation_specifies(section, tracker)
                .map(|argument| RelationSpecifiesSection { argument })
        }),
        justification: sections.get("Justification").copied().and_then(|section| {
            parse_required_groups(section, "Justification", tracker, parse_have_group)
                .map(|arguments| JustificationSection { arguments })
        }),
        documented: sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(section, "Documented", tracker, parse_documented_item_group)
                .map(|arguments| DocumentedSection { arguments })
        }),
        aliases: sections.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", tracker, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        writing: parse_optional_item_writing(&sections, tracker),
        references: sections.get("References").copied().and_then(|section| {
            parse_required_resource_references(section, tracker)
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
/// Both top-level `Relation:` subjects and its `specifies:` accept either quoted text
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

/// Parses the `specifies:` of a `Relation:`.
///
/// A quoted string is a prose description; anything unquoted is parsed as a
/// logical clause (a statement of what the relationship means).
fn parse_required_relation_specifies(
    section: &ProtoSection,
    tracker: &mut EventLog,
) -> Option<RelationSpecifies> {
    if section_is_quoted_text(section) {
        parse_required_open_text(section, "specifies", tracker).map(RelationSpecifies::Text)
    } else {
        parse_required_clause(section, "specifies", tracker)
            .map(|clause| RelationSpecifies::Statement(Box::new(clause)))
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
/// `"\signature"` strings) and a required `specifies:` description. References are
/// quoted text so a bare `\signature` reads as a reference to a definition rather
/// than a usage; they are recorded, not resolved.
pub(super) fn parse_related_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<TopicRelatedItem> {
    let sections = identify_sections("related", &group.sections, tracker, &["to", "specifies"])?;
    Some(TopicRelatedItem {
        to: TopicRelatedToSection {
            arguments: parse_required_open_texts(section(&sections, "to")?, "to", tracker)?,
        },
        specifies: TopicRelatedSpecifiesSection {
            argument: parse_required_open_text(
                section(&sections, "specifies")?,
                "specifies",
                tracker,
            )?,
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

/// Dispatches the `Documented:` items allowed on a `Text*` placeholder group:
/// `called:`, `written:`, `description:`, and `notes:`.
pub(super) fn parse_text_documented_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<DocumentedItem> {
    match first_section_label(group)? {
        "called" => parse_called(group, tracker).map(DocumentedItem::Called),
        "written" => parse_written(group, tracker).map(DocumentedItem::Written),
        "description" => parse_description(group, tracker).map(DocumentedItem::Description),
        "notes" => parse_notes(group, tracker).map(DocumentedItem::Notes),
        other => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!(
                    "`Text*` documentation only accepts `called:`, `written:`, `description:`, \
                     and `notes:`, not `{other}:`"
                ),
            );
            None
        }
    }
}

/// Parses one of the opaque `Text*` placeholder groups (`TextTheorem:`,
/// `TextAxiom:`, `TextConjecture:`, `TextDefinition:`). The leading section holds
/// a markdown-with-LaTeX body; `Documented?:`/`References?:` are optional and
/// `Id:` is required.
pub(in crate::frontend::structural::parser) fn parse_text_item(
    group: &ProtoGroup,
    tracker: &mut EventLog,
    kind: TextItemKind,
) -> Option<TextItemGroup> {
    ensure_no_heading(group, tracker)?;
    let label = kind.label();
    let sections = identify_sections(
        label,
        &group.sections,
        tracker,
        &[label, "Documented?", "References?", "Id"],
    )?;
    Some(TextItemGroup {
        kind,
        text: TextItemSection {
            argument: parse_required_open_text(section(&sections, label)?, label, tracker)?,
        },
        documented: sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(
                section,
                "Documented",
                tracker,
                parse_text_documented_item_group,
            )
            .map(|arguments| DocumentedSection { arguments })
        }),
        references: sections.get("References").copied().and_then(|section| {
            parse_required_resource_references(section, tracker)
                .map(|arguments| ReferencesSection { arguments })
        }),
        id: IdSection {
            argument: parse_required_open_text(section(&sections, "Id")?, "Id", tracker)?,
        },
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
            given,
            where_,
            then,
            iff,
            justification,
            documented,
            aliases,
            writing,
            references,
            metadata,
        )| {
            AxiomGroup {
                heading,
                given,
                where_,
                then,
                iff,
                justification,
                documented,
                aliases,
                writing,
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
            justification,
            documented,
            aliases,
            writing,
            references,
            metadata,
        )| {
            TheoremGroup {
                heading,
                given,
                where_,
                then,
                iff,
                justification,
                documented,
                aliases,
                writing,
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
            given,
            where_,
            then,
            iff,
            justification,
            documented,
            aliases,
            writing,
            references,
            metadata,
        )| {
            ConjectureGroup {
                heading,
                given,
                where_,
                then,
                iff,
                justification,
                documented,
                aliases,
                writing,
                references,
                metadata,
            }
        },
    )
}

/// Rejects a name/argument on a theorem-like head section.
///
/// `Axiom:`/`Theorem:`/`Conjecture:` do not accept a name;
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
    Option<JustificationSection>,
    Option<DocumentedSection>,
    Option<AliasesSection>,
    Option<ItemWritingSection>,
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
            "Documented?",
            "Justification?",
            "Aliases?",
            "Writing?",
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
        sections.get("Justification").copied().and_then(|section| {
            parse_required_groups(section, "Justification", tracker, parse_have_group)
                .map(|arguments| JustificationSection { arguments })
        }),
        sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(section, "Documented", tracker, parse_documented_item_group)
                .map(|arguments| DocumentedSection { arguments })
        }),
        sections.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", tracker, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        parse_optional_item_writing(&sections, tracker),
        sections.get("References").copied().and_then(|section| {
            parse_required_resource_references(section, tracker)
                .map(|arguments| ReferencesSection { arguments })
        }),
        sections.get("Metadata").copied().and_then(|section| {
            parse_required_groups(section, "Metadata", tracker, parse_metadata_item_group)
                .map(|arguments| MetadataSection { arguments })
        }),
    ))
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
        DeclarationRelation, FormOrDeclaration, FormOrDeclarationKind, IsSubjectForm, IsSubjectKind,
    };
    use crate::frontend::structural::ast::{
        AliasItem, AliasKind, Clause, DeclaresTarget, Document, DocumentedItem, EnablesItem,
        MetadataItem, RelationSpecifies, RelationSubject, RequiresItem, ResourceItem, SpecifyItem,
        TextItemKind, TopLevelItem,
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
Declares: n
Requires:
. capability: n_ + m_ :=> n_ \.natural.+./ m_
. definition: \natural.0 is \natural
Documented:
. called: "natural"
"#,
        );

        assert_eq!(document.items.len(), 1);
        match &document.items[0] {
            TopLevelItem::Declares(group) => {
                let requires = group.requires.as_ref().expect("expected Requires section");
                assert_eq!(requires.arguments.len(), 2);
                assert!(matches!(requires.arguments[0], RequiresItem::Capability(_)));
                assert!(matches!(requires.arguments[1], RequiresItem::Definition(_)));
            }
            other => panic!("expected Declares item, got {other:?}"),
        }
    }

    #[test]
    fn rejects_the_extends_section_label_for_refines() {
        let source = r#"[\(special)::thing]
Refines: X
extends: X is \thing
"#;

        let (_, diagnostics) = parse_with_diagnostics(source);

        assert!(
            diagnostics.iter().any(|event| {
                event.as_message().is_some_and(|message| {
                    message.message.contains("Unexpected section `extends`")
                        && message.message.contains("specifies?:")
                })
            }),
            "{diagnostics:#?}"
        );
    }

    #[test]
    fn rejects_documented_writing_targets_that_are_not_mapping_forms_or_invocations() {
        let (_, diagnostics) = parse_with_diagnostics(
            r#"[\sequence]
Declares: x(i_)
Documented:
. writing: x
  as: "x"
"#,
        );

        assert!(diagnostics.iter().any(|event| {
            event.as_message().is_some_and(|message| {
                message
                    .message
                    .contains("expected a mapping form such as `x(i_)`")
            })
        }));
    }

    #[test]
    fn parses_definition_like_groups_with_nested_sections_and_items() {
        let document = parse_ok(
            r#"
[\structure]
Declares: S ::= (X, *) is \set via (X, Y)
using:
. X is \set
. X "contains" Element
when:
. [logic.when]
  allOf:
  . x = x
  . y = y
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
Documented:
. [docs.written]
  written:
  . "plus"
Justification:
. [proof.label]
  have:
  . x = x
  asserting:
  . x = x
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
Declares: T
Enables:
. [conn.plus]
  view:
  as: y := X is \bar
Documented:
. [docs.called]
  called:
  . "addition"
Justification:
. [proof.by]
  have:
  . y = y
  asserting:
  . y = y
Aliases:
. [alias.spec]
  alias: x_ "in" X :-> x is \element
Metadata:
. version: "1.0"

[\structure.writing]
Declares: plus(x_, y_)
Documented:
. [docs.writing]
  writing: plus(x, y)
  as: "x? + y?"
. writing: plus(x_, y_)
  as: "\operatorname{plus}(x?, y?)"

[\structure.overview]
Declares: O
Documented:
. [docs.overview]
  overview: "Binary operation on X"

[\structure.description]
Declares: P
Documented:
. [docs.description]
  description: "Longer prose for readers"

[\structure.related]
Declares: R
Documented:
. [docs.related]
  related:
  . "group"
  . "ring"

[\structure.discoverer]
Declares: D
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
specifies: y is \(f)::[[g]]
satisfies:
. [logic.let]
  let: x is \element
  where:
  . x = x
  then:
  . y = y

[\statement]
States:
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
            TopLevelItem::Declares(group) => {
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
                assert!(group.declares.via.is_some());
                assert!(matches!(
                    group.declares.argument,
                    DeclaresTarget::Declaration(_)
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
                assert_eq!(
                    group
                        .justification
                        .as_ref()
                        .expect("expected justification")
                        .arguments[0]
                        .heading
                        .as_ref()
                        .expect("expected justification heading")
                        .parts,
                    vec!["proof".to_owned(), "label".to_owned()]
                );
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
            other => panic!("expected declares group, got {other:?}"),
        }

        match &document.items[1] {
            TopLevelItem::Declares(group) => {
                assert!(matches!(
                    group.enables.as_ref().expect("expected enables").arguments[0],
                    EnablesItem::View(_)
                ));
                assert_eq!(
                    group
                        .justification
                        .as_ref()
                        .expect("expected justification")
                        .arguments[0]
                        .heading
                        .as_ref()
                        .expect("expected justification heading")
                        .parts,
                    vec!["proof".to_owned(), "by".to_owned()]
                );
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
            other => panic!("expected declares group, got {other:?}"),
        }

        match &document.items[2] {
            TopLevelItem::Declares(group) => {
                assert!(matches!(
                    group
                        .documented
                        .as_ref()
                        .expect("expected documented")
                        .arguments[0],
                    DocumentedItem::Writing(_)
                ));
            }
            other => panic!("expected declares group, got {other:?}"),
        }

        match &document.items[3] {
            TopLevelItem::Declares(group) => {
                assert!(matches!(
                    group
                        .documented
                        .as_ref()
                        .expect("expected documented")
                        .arguments[0],
                    DocumentedItem::Overview(_)
                ));
            }
            other => panic!("expected declares group, got {other:?}"),
        }

        match &document.items[4] {
            TopLevelItem::Declares(group) => {
                assert!(matches!(
                    group
                        .documented
                        .as_ref()
                        .expect("expected documented")
                        .arguments[0],
                    DocumentedItem::Description(_)
                ));
            }
            other => panic!("expected declares group, got {other:?}"),
        }

        match &document.items[5] {
            TopLevelItem::Declares(group) => {
                assert!(matches!(
                    group
                        .documented
                        .as_ref()
                        .expect("expected documented")
                        .arguments[0],
                    DocumentedItem::Related(_)
                ));
            }
            other => panic!("expected declares group, got {other:?}"),
        }

        match &document.items[6] {
            TopLevelItem::Declares(group) => {
                assert!(matches!(
                    group
                        .documented
                        .as_ref()
                        .expect("expected documented")
                        .arguments[0],
                    DocumentedItem::Discoverer(_)
                ));
            }
            other => panic!("expected declares group, got {other:?}"),
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
                    Clause::Let(_)
                ));
            }
            other => panic!("expected refines group, got {other:?}"),
        }

        match &document.items[9] {
            TopLevelItem::States(group) => {
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
    fn parses_labeled_declaration_in_satisfies() {
        let document = parse_ok(
            r#"
[\thing]
Declares: t
satisfies:
. (.*' := `*`.)[:1:]
Documented:
. called: "thing"
Justification:
. [1]
  have: *' := `*`
  asserting: t = t
"#,
        );

        let TopLevelItem::Declares(group) = &document.items[0] else {
            panic!("expected Declares item");
        };
        let satisfies = group.satisfies.as_ref().expect("expected satisfies");
        let Clause::Declaration(statement) = &satisfies.arguments[0] else {
            panic!("expected labeled declaration clause");
        };
        assert_eq!(statement.labels.len(), 1);
        assert_eq!(statement.labels[0].parts, vec!["1".to_string()]);
    }

    #[test]
    fn parses_provided_symbol_with_builtin_spec_operator_target() {
        let document = parse_ok(
            r#"
[\set]
Declares: X
Enables:
. capability: x_ "in" X :-> \\abstract
Documented:
. called: "set"
"#,
        );

        let TopLevelItem::Declares(group) = &document.items[0] else {
            panic!("expected declares group");
        };

        assert!(matches!(
            group.enables.as_ref().expect("expected enables").arguments[0],
            EnablesItem::Capability(_)
        ));
    }

    #[test]
    fn parses_from_capability_from_as_and_view_enables_items() {
        let document = parse_ok(
            r#"
[\set]
Declares: X
Enables:
. from: Y ::= {y__ : ...}
  capability: x_ "in" X :-> x_ member_of Y
. from: P ::= {(p_, q_) : ...}
  as: f(p_) := q_
. view:
  as: r := X is \rational
  signifies: X \.embedded.to./ r
Documented:
. called: "set"
"#,
        );

        let TopLevelItem::Declares(group) = &document.items[0] else {
            panic!("expected declares group");
        };
        let enables = group.enables.as_ref().expect("expected enables");
        assert!(matches!(
            enables.arguments[0],
            EnablesItem::FromCapability(_)
        ));
        assert!(matches!(enables.arguments[1], EnablesItem::FromAs(_)));
        assert!(matches!(enables.arguments[2], EnablesItem::View(_)));
    }

    #[test]
    fn parses_view_enables_items() {
        let document = parse_ok(
            r#"
[\pair]
Defines: P is \pair
Enables:
. view:
  as: x := a is \set
  signifies: x \:isomorphic.to?:/ p
. view:
  as: y := b is \group
Documented:
. written: "P?"
"#,
        );

        let TopLevelItem::Defines(group) = &document.items[0] else {
            panic!("expected defines group");
        };
        let enables = group.enables.as_ref().expect("expected enables");
        assert!(matches!(enables.arguments[0], EnablesItem::View(_)));
        assert!(matches!(enables.arguments[1], EnablesItem::View(_)));

        let EnablesItem::View(view) = &enables.arguments[0] else {
            panic!("expected view");
        };
        assert!(view.as_.argument.definition.is_some());
        assert!(matches!(
            view.as_.argument.relation,
            Some(DeclarationRelation::Is(_))
        ));
        assert!(view.signifies.is_some());
    }

    #[test]
    fn view_marker_rejects_arguments() {
        let (_document, diagnostics) = parse_with_diagnostics(
            r#"
[\integer]
Declares: n
Enables:
. view: unexpected
  as: k := n is \integer
Documented:
. called: "integer"
"#,
        );

        assert!(diagnostics.iter().any(|event| {
            matches!(event, Event::Message(message) if
                message.message.contains("`view:` is a marker section and takes no arguments")
            )
        }));
    }

    #[test]
    fn rejects_legacy_viewable_enables_group() {
        let (_document, diagnostics) = parse_with_diagnostics(
            r#"
[\integer]
Declares: n
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
Declares: n
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
    fn parses_relation_item_with_using_between_and_when_specifies() {
        let document = parse_ok(
            r#"
Relation:
using:
. n is \integer
between: a is \real
and: b is \real
when:
. a = b
specifies: a = b
Documented:
. description: "a and b name the same value."
"#,
        );

        match &document.items[0] {
            TopLevelItem::Relation(group) => {
                assert!(group.using.is_some());
                assert!(group.when.is_some());
                assert!(group.specifies.is_some());
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
specifies: a = a
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
  specifies: "Closely connected subjects."
. to: "\function:on:to"
  specifies: "Functions studied here."
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
                assert_eq!(first.specifies.argument.0, "Closely connected subjects.");
                assert_eq!(related.arguments[1].to.arguments[0].0, r"\function:on:to");
                assert!(group.documented.is_some());
            }
            other => panic!("expected Topic item, got {other:?}"),
        }
    }

    #[test]
    fn parses_text_item_placeholders() {
        let document = parse_ok(
            r#"
TextTheorem: "For every group $G$, the identity is **unique**."
Documented:
. called: "Uniqueness of identity"
. written: "\text{Uniqueness}"
. description: "A placeholder."
. notes: "Use \group once defined."
. notes: "Cross-reference the monoid version."
References:
. $book.algebra
Id: "11111111-1111-4111-8111-111111111111"

TextDefinition: "A **prime** is a natural number with exactly two divisors."
Id: "22222222-2222-4222-8222-222222222222"
"#,
        );

        match &document.items[0] {
            TopLevelItem::TextItem(group) => {
                assert_eq!(group.kind, TextItemKind::Theorem);
                assert!(group.text.argument.0.contains("identity is"));
                let documented = group.documented.as_ref().expect("expected Documented");
                assert!(matches!(documented.arguments[0], DocumentedItem::Called(_)));
                let notes: Vec<_> = documented
                    .arguments
                    .iter()
                    .filter_map(|item| match item {
                        DocumentedItem::Notes(notes) => Some(notes),
                        _ => None,
                    })
                    .collect();
                assert_eq!(notes.len(), 2);
                assert_eq!(notes[0].notes.arguments.len(), 1);
                assert!(group.references.is_some());
                assert_eq!(group.id.argument.0, "11111111-1111-4111-8111-111111111111");
            }
            other => panic!("expected TextItem, got {other:?}"),
        }

        match &document.items[1] {
            TopLevelItem::TextItem(group) => {
                assert_eq!(group.kind, TextItemKind::Definition);
                assert!(group.documented.is_none());
                assert_eq!(group.id.argument.0, "22222222-2222-4222-8222-222222222222");
            }
            other => panic!("expected TextItem, got {other:?}"),
        }
    }

    #[test]
    fn topic_related_item_requires_specifies() {
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
                .is_some_and(|message| message.message.contains("specifies"))),
            "expected a diagnostic about the missing `specifies:` section: {diagnostics:#?}"
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
    fn parses_relation_between_quoted_topic_and_signature_with_text_specifies() {
        let document = parse_ok(
            r##"
Relation:
between: "#real.analysis"
and: "\sin"
specifies: "The sine function is studied within real analysis."
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
                match &group.specifies.as_ref().expect("specifies").argument {
                    RelationSpecifies::Text(text) => {
                        assert_eq!(text.0, "The sine function is studied within real analysis.")
                    }
                    other => panic!("expected a text specifies, got {other:?}"),
                }
            }
            other => panic!("expected Relation item, got {other:?}"),
        }
    }

    #[test]
    fn parses_relation_between_declaration_with_statement_specifies() {
        let document = parse_ok(
            r#"
Relation:
between: a is \real
and: b is \real
specifies: a = b
"#,
        );

        match &document.items[0] {
            TopLevelItem::Relation(group) => {
                assert!(matches!(
                    &group.between.argument,
                    RelationSubject::Declaration(_)
                ));
                assert!(matches!(
                    &group.specifies.as_ref().expect("specifies").argument,
                    RelationSpecifies::Statement(_)
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
to:
. \bar{a, b}
. \baz{a, b}
"#,
        );

        match &document.items[0] {
            TopLevelItem::Equivalent(group) => {
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
            "conjectures.text".to_owned(),
            "defines.text".to_owned(),
            "declares.text".to_owned(),
            "equivalent.text".to_owned(),
            "outline.text".to_owned(),
            "persons.text".to_owned(),
            "realizes.text".to_owned(),
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
. decimal:
  is: \real
. zeroOrPositiveInt:
  is: \whole
. positiveInt:
  is: \natural
. int:
  is: \integer
"#,
        );

        assert_eq!(document.items.len(), 17);

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
                    SpecifyItem::Decimal(_)
                ));
                assert!(matches!(
                    group.specify.arguments[1],
                    SpecifyItem::ZeroOrPositiveInt(_)
                ));
                assert!(matches!(
                    group.specify.arguments[2],
                    SpecifyItem::PositiveInt(_)
                ));
                assert!(matches!(group.specify.arguments[3], SpecifyItem::Int(_)));
            }
            other => panic!("expected specify group, got {other:?}"),
        }
    }

    // ===============================[ overview ]=====================================

    #[test]
    fn parses_mixed_structural_document() {
        let text = r#"
[\function]
Declares: f(x_)
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
        assert!(matches!(document.items[0], TopLevelItem::Declares(_)));
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
Declares: f(x_)
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
    fn parses_let_clause_groups() {
        let text = r#"
[\property]
States:
that:
. let: n "in" X
  where: n = n
  then: n = n
"#;

        let mut tracker = EventLog::new();
        let document = parse_document(text, &mut tracker);

        assert!(!tracker.has_errors(), "{:#?}", tracker.events());
        match &document.items[0] {
            TopLevelItem::States(states) => match &states.that.arguments[0] {
                Clause::Let(group) => {
                    assert_eq!(group.let_.arguments.len(), 1);
                    assert_eq!(
                        group
                            .where_
                            .as_ref()
                            .expect("expected where")
                            .arguments
                            .len(),
                        1
                    );
                    assert_eq!(group.then.arguments.len(), 1);
                }
                other => panic!("expected let clause, got {other:?}"),
            },
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
        for head in ["Axiom", "Theorem", "Conjecture"] {
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
    }

    #[test]
    fn rejects_corollary_as_a_top_level_group() {
        let (document, diagnostics) =
            parse_with_diagnostics("Corollary:\nof: \"A theorem\"\nthen: x = x\n");
        assert!(document.items.is_empty());
        assert!(
            diagnostics
                .iter()
                .any(|event| event.as_message().is_some_and(|message| message
                    .message
                    .contains("Unexpected top-level group `Corollary`")))
        );
    }

    #[test]
    fn parses_binding_clause_groups_with_multiple_bindings() {
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
. let:
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
                    Clause::Let(group) => assert_eq!(group.let_.arguments.len(), 2),
                    other => panic!("expected let clause, got {other:?}"),
                }
            }
            other => panic!("expected states item, got {other:?}"),
        }
    }

    #[test]
    fn parses_is_statements_as_inline_clauses() {
        let text = r#"
[\function:on{A}:to{B}]
Declares: f(x__)
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
            TopLevelItem::Declares(group) => {
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
            other => panic!("expected declares item, got {other:?}"),
        }
    }

    #[test]
    fn parses_a_declares_target_that_names_the_type_it_extends() {
        let text = r#"
[\group]
Declares: G ::= (X, *, e) is \monoid via (X, *)
specifies:
. e "in" X
"#;

        let mut tracker = EventLog::new();
        let document = parse_document(text, &mut tracker);

        assert!(!tracker.has_errors(), "{:#?}", tracker.events());
        match &document.items[0] {
            TopLevelItem::Declares(group) => {
                let DeclaresTarget::Declaration(statement) = &group.declares.argument else {
                    panic!("expected declaration target");
                };
                assert!(statement.expansion.is_some());
                assert!(matches!(
                    statement.relation,
                    Some(DeclarationRelation::Is(_))
                ));
                assert!(matches!(
                    group.declares.via.as_ref().expect("expected via").kind,
                    FormOrDeclarationKind::TupleDeclaration { .. }
                ));
            }
            other => panic!("expected declares item, got {other:?}"),
        }
    }

    #[test]
    fn parses_a_declares_target_that_states_a_spec_relation() {
        let text = r#"
[\element:of{X}]
Declares: x "in" X
when: X is \set
"#;

        let mut tracker = EventLog::new();
        let document = parse_document(text, &mut tracker);

        assert!(!tracker.has_errors(), "{:#?}", tracker.events());
        match &document.items[0] {
            TopLevelItem::Declares(group) => {
                let DeclaresTarget::Declaration(statement) = &group.declares.argument else {
                    panic!("expected declaration target");
                };
                assert!(matches!(
                    statement.relation,
                    Some(DeclarationRelation::Spec { .. })
                ));
                assert!(group.declares.via.is_none());
            }
            other => panic!("expected declares item, got {other:?}"),
        }
    }

    #[test]
    fn rejects_a_declares_via_without_an_extended_type() {
        let text = r#"
[\group]
Declares: G ::= (X, *, e) via (X, *)
"#;

        let (_, diagnostics) = parse_with_diagnostics(text);

        assert!(
            diagnostics.iter().any(|event| {
                event.as_message().is_some_and(|message| {
                    message.message.contains(
                        "`via` requires the `Declares:` target to name the type it extends",
                    )
                })
            }),
            "{diagnostics:#?}"
        );
    }

    #[test]
    fn parses_a_declares_extends_section_with_several_clauses() {
        let text = r#"
[\foo]
Declares: X ::= (A, B, C)
extends:
. X is \bar via (A, B)
. X is \baz via (B, C)
"#;

        let mut tracker = EventLog::new();
        let document = parse_document(text, &mut tracker);

        assert!(!tracker.has_errors(), "{:#?}", tracker.events());
        match &document.items[0] {
            TopLevelItem::Declares(group) => {
                assert!(matches!(group.declares.argument, DeclaresTarget::Form(_)));
                assert!(group.declares.via.is_none());
                let extends = group.extends.as_ref().expect("expected extends");
                assert_eq!(extends.arguments.len(), 2);
                for item in &extends.arguments {
                    assert!(matches!(
                        item.statement.relation,
                        Some(DeclarationRelation::Is(_))
                    ));
                    assert!(item.via.is_some());
                }
            }
            other => panic!("expected declares item, got {other:?}"),
        }
    }

    #[test]
    fn rejects_a_declares_target_relation_together_with_an_extends_section() {
        let text = r#"
[\nonempty.set]
Declares: X is \set
extends: X is \set
"#;

        let (_, diagnostics) = parse_with_diagnostics(text);

        assert!(
            diagnostics.iter().any(|event| {
                event.as_message().is_some_and(|message| {
                    message.message.contains(
                        "A `Declares:` target that names the type it extends cannot also have an \
                         `extends:` section",
                    )
                })
            }),
            "{diagnostics:#?}"
        );
    }

    #[test]
    fn accepts_an_extends_section_when_the_declares_target_states_no_relation() {
        let text = r#"
[\nonempty.set]
Declares: X
extends: X is \set
"#;

        let mut tracker = EventLog::new();
        let document = parse_document(text, &mut tracker);

        assert!(!tracker.has_errors(), "{:#?}", tracker.events());
        match &document.items[0] {
            TopLevelItem::Declares(group) => {
                assert_eq!(
                    group
                        .extends
                        .as_ref()
                        .expect("expected extends")
                        .arguments
                        .len(),
                    1
                );
            }
            other => panic!("expected declares item, got {other:?}"),
        }
    }

    #[test]
    fn parses_declares_function_declaration_target_with_specifies() {
        let text = r#"
[\function:on{A}:to{B}]
Declares: f(x__) ::= y_
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
            TopLevelItem::Declares(group) => {
                let DeclaresTarget::Declaration(statement) = &group.declares.argument else {
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
            other => panic!("expected declares item, got {other:?}"),
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
. "alpha :~> \alpha"
. "beta :~> \beta"
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
    fn rejects_unquoted_top_level_writing_aliases() {
        let (document, messages) = parse_with_diagnostics(
            r#"
Writing:
. alpha :~> \alpha
"#,
        );

        assert!(document.items.is_empty());
        assert!(messages.iter().any(|event| {
            event
                .as_message()
                .is_some_and(|message| message.message.contains("Expected a quoted Writing alias"))
        }));
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
. "f(x_) :~> x"
"#,
        );

        assert!(document.items.is_empty());
        assert!(messages.iter().any(|event| {
            event
                .as_message()
                .is_some_and(|message| message.message.contains("Writing aliases must use a name"))
        }));
    }

    #[test]
    fn parses_item_level_writing_section_after_aliases() {
        let document = parse_ok(
            r#"
[\natural]
Declares: n
Documented:
. called: "natural"
Writing:
. "pi :~> \varpi"
. "e :~> \mathrm{e}"
"#,
        );

        let TopLevelItem::Declares(group) = &document.items[0] else {
            panic!("expected Declares item, got {:?}", document.items[0]);
        };
        let writing = group.writing.as_ref().expect("expected item-level Writing");
        assert_eq!(writing.arguments.len(), 2);
        assert_eq!(writing.arguments[0].body, r#"\varpi"#);
        assert!(matches!(
            writing.arguments[0].form.kind,
            FormOrDeclarationKind::Name(ref name) if name == "pi"
        ));
    }

    #[test]
    fn rejects_unquoted_item_level_writing_aliases() {
        let (_, messages) = parse_with_diagnostics(
            r#"
[\natural]
Declares: n
Documented:
. called: "natural"
Writing:
. pi :~> \varpi
"#,
        );

        assert!(messages.iter().any(|event| {
            event
                .as_message()
                .is_some_and(|message| message.message.contains("Expected a quoted Writing alias"))
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
Documented:
. [axiom.written]
  written:
  . "axiom"
Justification:
. [axiom.justified]
  have:
  . y = y
  asserting:
  . y = y
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

[\conjecture]
Conjecture:
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
                assert!(group.justification.is_some());
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
            TopLevelItem::Conjecture(group) => {
                assert!(group.heading.is_some());
                assert!(matches!(group.then.arguments[0], Clause::OneOf(_)));
            }
            other => panic!("expected conjecture group, got {other:?}"),
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

    #[test]
    fn rejects_nested_given_clause_groups() {
        let (_document, messages) = parse_with_diagnostics(
            r#"
Theorem:
then:
. given: x is \set
  then: x is? \set
"#,
        );

        assert!(messages.iter().any(|event| {
            event
                .as_message()
                .is_some_and(|message| message.message.contains("Unexpected clause group `given`"))
        }));
    }

    #[test]
    fn parses_quoted_and_paged_resource_references() {
        let document = parse_ok(
            r#"
Theorem:
then: x = x
References:
. "$royden.real.analysis"
. "$royden.real.analysis:page{4}"
"#,
        );

        let TopLevelItem::Theorem(group) = &document.items[0] else {
            panic!("expected theorem group");
        };
        let references = group.references.as_ref().expect("expected references");
        assert_eq!(
            references.arguments[0].parts.join("."),
            "royden.real.analysis"
        );
        assert_eq!(references.arguments[0].page, None);
        assert_eq!(references.arguments[1].page, Some(4));
    }

    #[test]
    fn parses_refined_declaration_statement_in_clauses_and_have() {
        let document = parse_ok(
            r#"
[\von.neumann.omega]
Defines: omega is \set
expresses:
. let:
  . (.A is \(inductive)::set.)[:1:]
  . X is \set
  then:
  . have: X "in"? omega
    iff:
    . X "in"? A
    . forAll: I is \(inductive)::set
      then: X "in"? I
Enables:
. capability: X_ "in" omega :-> X_ is \set
Documented:
. written: "\omega"
Justification:
. [1]
  have: A is \(inductive)::set
  by: \axiom.of.infinity
Id: "c13f4641-0ed5-4ad7-b309-8ec13b4c6b77"
"#,
        );

        let TopLevelItem::Defines(group) = &document.items[0] else {
            panic!("expected defines group");
        };
        let crate::frontend::formulation::ast::CommandHeader::Command(ref node) = group.heading else {
            panic!("expected command header");
        };
        assert_eq!(
            node.chain.parts.iter().map(|p| match p {
                crate::frontend::formulation::ast::ChainPart::Name(name) => name.as_str(),
                _ => "",
            }).collect::<Vec<_>>().join("."),
            "von.neumann.omega"
        );
    }

    #[test]
    fn parses_piecewise_with_else_if_sections() {
        let document = parse_ok(
            r#"
[\foo]
Defines: f(n_, m_)
expresses:
. piecewise:
  if: n_ = 0
  then: n_ + m_ := m_
  elseIf: n_ = 1
  then: n_ + m_ := m_ - 1
  else:
  . let: k is \natural
    where: n_ = \naturals..S(k)
    then: n_ + m_ := \naturals..S(n_ \.natural.+./ k)
"#,
        );
        assert_eq!(document.items.len(), 1);
        let TopLevelItem::Defines(group) = &document.items[0] else {
            panic!("expected defines group");
        };
        let expresses = group.expresses.as_ref().expect("expected expresses");
        let Clause::Piecewise(piecewise) = &expresses.arguments[0] else {
            panic!("expected piecewise clause");
        };
        assert_eq!(piecewise.if_.arguments.len(), 1);
        assert_eq!(piecewise.then.arguments.len(), 1);
        assert_eq!(piecewise.else_if.len(), 1);
        assert_eq!(piecewise.else_if[0].else_if.arguments.len(), 1);
        assert_eq!(piecewise.else_if[0].then.arguments.len(), 1);
        assert!(piecewise.else_.is_some());
    }

    #[test]
    fn rejects_piecewise_with_arguments() {
        let (_, diagnostics) = parse_with_diagnostics(
            r#"
[\foo]
Defines: f(n_)
expresses:
. piecewise:
  . "invalid text"
  if: n_ = 0
  then: n_ := 0
"#,
        );
        assert!(!diagnostics.is_empty(), "expected error for piecewise with arguments");
        assert!(
            diagnostics
                .iter()
                .any(|e| matches!(e, Event::Message(m) if m.message.contains("Section `piecewise` does not accept content")))
        );
    }

    #[test]
    fn rejects_piecewise_else_if_without_then() {
        let (_, diagnostics) = parse_with_diagnostics(
            r#"
[\foo]
Defines: f(n_)
expresses:
. piecewise:
  if: n_ = 0
  then: n_ := 0
  elseIf: n_ = 1
  else: n_ := 2
"#,
        );
        assert!(!diagnostics.is_empty(), "expected error for elseIf without then");
        assert!(
            diagnostics
                .iter()
                .any(|e| matches!(e, Event::Message(m) if m.message.contains("Expected `then` section after `elseIf`")))
        );
    }
}

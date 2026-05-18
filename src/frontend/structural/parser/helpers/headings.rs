use super::*;

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
    parse_is_or_spec(input).map(IsOrViaItem::IsOrSpec)
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

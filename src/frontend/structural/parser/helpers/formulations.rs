use super::*;

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

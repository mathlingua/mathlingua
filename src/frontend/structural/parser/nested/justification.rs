use super::*;

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

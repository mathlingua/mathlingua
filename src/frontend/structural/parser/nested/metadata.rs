/// Parses an `id:` metadata group.
///
/// Metadata groups do not accept headings because their meaning is determined
/// entirely by the nested section label.
fn parse_id(group: &ProtoGroup, tracker: &mut EventLog) -> Option<IdGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("id", &group.sections, tracker, &["id"])?;
    Some(IdGroup {
        id: IdSection {
            argument: parse_required_open_text(section(&sections, "id")?, "id", tracker)?,
        },
    })
}

/// Parses a `version:` metadata group.
fn parse_version(group: &ProtoGroup, tracker: &mut EventLog) -> Option<VersionGroup> {
    ensure_no_heading(group, tracker)?;
    let sections = identify_sections("version", &group.sections, tracker, &["version"])?;
    Some(VersionGroup {
        version: VersionSection {
            argument: parse_required_open_text(section(&sections, "version")?, "version", tracker)?,
        },
    })
}


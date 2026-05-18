/// Parses a top-level `Title:` group.
///
/// Title groups cannot have bracket headings and must contain exactly the
/// `Title:` section shape.
fn parse_title(group: &ProtoGroup, tracker: &mut EventLog) -> Option<TitleGroup> {
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
fn parse_section(group: &ProtoGroup, tracker: &mut EventLog) -> Option<SectionGroup> {
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
fn parse_subsection(group: &ProtoGroup, tracker: &mut EventLog) -> Option<SubsectionGroup> {
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
fn parse_subsubsection(group: &ProtoGroup, tracker: &mut EventLog) -> Option<SubsubsectionGroup> {
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


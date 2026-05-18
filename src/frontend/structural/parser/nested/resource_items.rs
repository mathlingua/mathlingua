use super::*;

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

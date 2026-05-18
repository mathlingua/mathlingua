/// Parses a `Person:` metadata group.
///
/// Person groups require an author-style heading and carry required `name:` and
/// `biography:` sections, with optional prose on the leading `Person:` section.
fn parse_person(group: &ProtoGroup, tracker: &mut EventLog) -> Option<PersonGroup> {
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
fn parse_resource(group: &ProtoGroup, tracker: &mut EventLog) -> Option<ResourceGroup> {
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
fn parse_specify(group: &ProtoGroup, tracker: &mut EventLog) -> Option<SpecifyGroup> {
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


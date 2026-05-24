use super::*;

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
            parse_required_formulations(section, "using", tracker, parse_is_or_spec)
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

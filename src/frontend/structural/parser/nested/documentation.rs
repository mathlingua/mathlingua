/// Parses an `alias:` nested group.
///
/// Aliases may optionally include a label heading and a `written:` rendering
/// section.  The alias body accepts either expression or specification-operator
/// alias syntax.
fn parse_alias_group(group: &ProtoGroup, tracker: &mut EventLog) -> Option<AliasGroup> {
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
fn parse_symbol(group: &ProtoGroup, tracker: &mut EventLog) -> Option<SymbolGroup> {
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
fn parse_connection(group: &ProtoGroup, tracker: &mut EventLog) -> Option<ConnectionGroup> {
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
fn parse_written(group: &ProtoGroup, tracker: &mut EventLog) -> Option<WrittenGroup> {
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
fn parse_called(group: &ProtoGroup, tracker: &mut EventLog) -> Option<CalledGroup> {
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
fn parse_writing(group: &ProtoGroup, tracker: &mut EventLog) -> Option<WritingGroup> {
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
fn parse_overview(group: &ProtoGroup, tracker: &mut EventLog) -> Option<OverviewGroup> {
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
fn parse_related(group: &ProtoGroup, tracker: &mut EventLog) -> Option<RelatedGroup> {
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
fn parse_discoverer(group: &ProtoGroup, tracker: &mut EventLog) -> Option<DiscovererGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("discoverer", &group.sections, tracker, &["discoverer"])?;
    Some(DiscovererGroup {
        heading,
        discoverer: DiscovererSection {
            arguments: parse_optional_open_texts(sections.get("discoverer").copied(), tracker),
        },
    })
}


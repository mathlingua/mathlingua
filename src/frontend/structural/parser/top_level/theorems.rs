/// Parses an `Axiom:` group using the shared theorem-like parser.
///
/// The returned shared tuple is adapted to the axiom-specific section wrapper.
fn parse_axiom(group: &ProtoGroup, tracker: &mut EventLog) -> Option<AxiomGroup> {
    parse_argument_theorem_like(group, tracker, "Axiom").map(
        |(
            heading,
            text,
            given,
            where_,
            then,
            iff,
            justified,
            documented,
            aliases,
            references,
            metadata,
        )| {
            AxiomGroup {
                heading,
                axiom: AxiomSection { arguments: text },
                given,
                where_,
                then,
                iff,
                justified,
                documented,
                aliases,
                references,
                metadata,
            }
        },
    )
}

/// Parses a `Theorem:` group using the shared theorem-like parser.
fn parse_theorem(group: &ProtoGroup, tracker: &mut EventLog) -> Option<TheoremGroup> {
    parse_argument_theorem_like(group, tracker, "Theorem").map(
        |(
            heading,
            text,
            given,
            where_,
            then,
            iff,
            justified,
            documented,
            aliases,
            references,
            metadata,
        )| {
            TheoremGroup {
                heading,
                theorem: TheoremSection { arguments: text },
                given,
                where_,
                then,
                iff,
                justified,
                documented,
                aliases,
                references,
                metadata,
            }
        },
    )
}

/// Parses a `Lemma:` group using the shared theorem-like parser.
fn parse_lemma(group: &ProtoGroup, tracker: &mut EventLog) -> Option<LemmaGroup> {
    parse_argument_theorem_like(group, tracker, "Lemma").map(
        |(
            heading,
            text,
            given,
            where_,
            then,
            iff,
            justified,
            documented,
            aliases,
            references,
            metadata,
        )| {
            LemmaGroup {
                heading,
                lemma: LemmaSection { arguments: text },
                given,
                where_,
                then,
                iff,
                justified,
                documented,
                aliases,
                references,
                metadata,
            }
        },
    )
}

/// Parses a `Conjecture:` group using the shared theorem-like parser.
fn parse_conjecture(group: &ProtoGroup, tracker: &mut EventLog) -> Option<ConjectureGroup> {
    parse_argument_theorem_like(group, tracker, "Conjecture").map(
        |(
            heading,
            text,
            given,
            where_,
            then,
            iff,
            justified,
            documented,
            aliases,
            references,
            metadata,
        )| {
            ConjectureGroup {
                heading,
                conjecture: ConjectureSection { arguments: text },
                given,
                where_,
                then,
                iff,
                justified,
                documented,
                aliases,
                references,
                metadata,
            }
        },
    )
}

#[allow(clippy::type_complexity)]
/// Parses the common shape shared by axiom/theorem/lemma/conjecture groups.
///
/// These groups all allow optional command headings, optional prose on their
/// primary section, optional assumptions/context, a required `then:` section,
/// and the same auxiliary documentation/reference sections.  Returning a tuple
/// keeps each concrete wrapper parser small while preserving exact section
/// types at the call site.
fn parse_argument_theorem_like(
    group: &ProtoGroup,
    tracker: &mut EventLog,
    name: &str,
) -> Option<(
    Option<crate::frontend::formulation::ast::CommandHeader>,
    ZeroOrMore<OpenText>,
    Option<GivenSection>,
    Option<WhereSection>,
    ThenSection,
    Option<IffSection>,
    Option<JustifiedSection>,
    Option<DocumentedSection>,
    Option<AliasesSection>,
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
            "Justified?",
            "Documented?",
            "Aliases?",
            "References?",
            "Metadata?",
        ],
    )?;

    Some((
        heading,
        parse_optional_open_texts(sections.get(section_name).copied(), tracker),
        sections.get("given").copied().and_then(|section| {
            parse_required_formulations(
                section,
                "given",
                tracker,
                parse_is_or_refined_statement_spec,
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
        sections.get("Justified").copied().and_then(|section| {
            parse_required_groups(section, "Justified", tracker, parse_justified_item_group)
                .map(|arguments| JustifiedSection { arguments })
        }),
        sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(section, "Documented", tracker, parse_documented_item_group)
                .map(|arguments| DocumentedSection { arguments })
        }),
        sections.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", tracker, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        sections.get("References").copied().and_then(|section| {
            parse_required_formulations(section, "References", tracker, parse_resource_header)
                .map(|arguments| ReferencesSection { arguments })
        }),
        sections.get("Metadata").copied().and_then(|section| {
            parse_required_groups(section, "Metadata", tracker, parse_metadata_item_group)
                .map(|arguments| MetadataSection { arguments })
        }),
    ))
}

/// Parses a `Corollary:` group.
///
/// Corollaries mostly share theorem-like structure but additionally require an
/// `of:` section that records what theorem or statement they follow from.
fn parse_corollary(group: &ProtoGroup, tracker: &mut EventLog) -> Option<CorollaryGroup> {
    let heading = parse_optional_command_heading(group, tracker)?;
    let sections = identify_sections(
        "Corollary",
        &group.sections,
        tracker,
        &[
            "Corollary",
            "of",
            "given?",
            "where?",
            "then",
            "iff?",
            "Justified?",
            "Documented?",
            "Aliases?",
            "References?",
            "Metadata?",
        ],
    )?;

    Some(CorollaryGroup {
        heading,
        corollary: CorollarySection {
            arguments: parse_optional_open_texts(sections.get("Corollary").copied(), tracker),
        },
        of: OfSection {
            arguments: parse_optional_open_texts(sections.get("of").copied(), tracker),
        },
        given: sections.get("given").copied().and_then(|section| {
            parse_required_formulations(
                section,
                "given",
                tracker,
                parse_is_or_refined_statement_spec,
            )
            .map(|arguments| GivenSection { arguments })
        }),
        where_: sections.get("where").copied().and_then(|section| {
            parse_required_clauses(section, "where", tracker)
                .map(|arguments| WhereSection { arguments })
        }),
        then: ThenSection {
            arguments: parse_required_clauses(section(&sections, "then")?, "then", tracker)?,
        },
        iff: sections.get("iff").copied().and_then(|section| {
            parse_required_clauses(section, "iff", tracker)
                .map(|arguments| IffSection { arguments })
        }),
        justified: sections.get("Justified").copied().and_then(|section| {
            parse_required_groups(section, "Justified", tracker, parse_justified_item_group)
                .map(|arguments| JustifiedSection { arguments })
        }),
        documented: sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(section, "Documented", tracker, parse_documented_item_group)
                .map(|arguments| DocumentedSection { arguments })
        }),
        aliases: sections.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", tracker, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        references: sections.get("References").copied().and_then(|section| {
            parse_required_formulations(section, "References", tracker, parse_resource_header)
                .map(|arguments| ReferencesSection { arguments })
        }),
        metadata: sections.get("Metadata").copied().and_then(|section| {
            parse_required_groups(section, "Metadata", tracker, parse_metadata_item_group)
                .map(|arguments| MetadataSection { arguments })
        }),
    })
}


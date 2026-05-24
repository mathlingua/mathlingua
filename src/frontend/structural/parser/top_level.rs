use super::*;

// ===============================[ dispatch ]=====================================

/// Dispatches one proto group to the top-level structural parser matching its first section.
///
/// The first section label determines the group kind.  Unknown labels are
/// reported at the group start and omitted from the resulting document.
pub(in crate::frontend::structural::parser) fn parse_top_level_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<TopLevelItem> {
    let label = first_section_label(group)?;
    match label {
        "Title" => parse_title(group, tracker).map(TopLevelItem::Title),
        "Section" => parse_section(group, tracker).map(TopLevelItem::Section),
        "Subsection" => parse_subsection(group, tracker).map(TopLevelItem::Subsection),
        "Subsubsection" => parse_subsubsection(group, tracker).map(TopLevelItem::Subsubsection),
        "Describes" => parse_describes(group, tracker).map(TopLevelItem::Describes),
        "Defines" => parse_defines(group, tracker).map(TopLevelItem::Defines),
        "Refines" => parse_refines(group, tracker).map(TopLevelItem::Refines),
        "States" => parse_states(group, tracker).map(TopLevelItem::States),
        "Axiom" => parse_axiom(group, tracker).map(TopLevelItem::Axiom),
        "Theorem" => parse_theorem(group, tracker).map(TopLevelItem::Theorem),
        "Corollary" => parse_corollary(group, tracker).map(TopLevelItem::Corollary),
        "Lemma" => parse_lemma(group, tracker).map(TopLevelItem::Lemma),
        "Conjecture" => parse_conjecture(group, tracker).map(TopLevelItem::Conjecture),
        "Person" => parse_person(group, tracker).map(TopLevelItem::Person),
        "Resource" => parse_resource(group, tracker).map(TopLevelItem::Resource),
        "Specify" => parse_specify(group, tracker).map(TopLevelItem::Specify),
        other => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Unexpected top-level group `{other}`"),
            );
            None
        }
    }
}

// ===============================[ outline ]=====================================

/// Parses a top-level `Title:` group.
///
/// Title groups cannot have bracket headings and must contain exactly the
/// `Title:` section shape.
pub(in crate::frontend::structural::parser) fn parse_title(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<TitleGroup> {
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
pub(in crate::frontend::structural::parser) fn parse_section(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<SectionGroup> {
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
pub(in crate::frontend::structural::parser) fn parse_subsection(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<SubsectionGroup> {
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
pub(in crate::frontend::structural::parser) fn parse_subsubsection(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<SubsubsectionGroup> {
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

// ===============================[ definitions ]=====================================

/// Parses a command-backed `Describes:` group.
///
/// This enforces the full `Describes` section order and converts each optional
/// nested section into its typed representation.  Formulation sections are
/// delegated to the formulation parser while clause/nested sections recurse
/// through structural helpers.
pub(in crate::frontend::structural::parser) fn parse_describes(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<DescribesGroup> {
    let heading = parse_required_command_heading(group, tracker)?;
    let sections = identify_sections(
        "Describes",
        &group.sections,
        tracker,
        &[
            "Describes",
            "using?",
            "when?",
            "extends?",
            "specifies?",
            "satisfies?",
            "Provides?",
            "Justified?",
            "Documented?",
            "Aliases?",
            "References?",
            "Metadata?",
        ],
    )?;

    Some(DescribesGroup {
        heading,
        describes: DescribesSection {
            argument: parse_required_formulation(
                section(&sections, "Describes")?,
                "Describes",
                tracker,
                parse_form_or_declaration,
            )?,
        },
        using: sections.get("using").copied().and_then(|section| {
            parse_required_formulations(section, "using", tracker, parse_is_or_spec)
                .map(|arguments| UsingSection { arguments })
        }),
        when: sections.get("when").copied().and_then(|section| {
            parse_required_clauses(section, "when", tracker)
                .map(|arguments| WhenSection { arguments })
        }),
        extends: sections.get("extends").copied().and_then(|section| {
            parse_required_formulation(section, "extends", tracker, parse_is_or_via_item)
                .map(|argument| ExtendsSection { argument })
        }),
        specifies: sections.get("specifies").copied().and_then(|section| {
            parse_required_formulations(section, "specifies", tracker, parse_is_or_via_item)
                .map(|arguments| DescribesSpecifiesSection { arguments })
        }),
        satisfies: sections.get("satisfies").copied().and_then(|section| {
            parse_required_clauses(section, "satisfies", tracker)
                .map(|arguments| SatisfiesSection { arguments })
        }),
        provides: sections.get("Provides").copied().and_then(|section| {
            parse_required_groups(section, "Provides", tracker, parse_provides_item_group)
                .map(|arguments| ProvidesSection { arguments })
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

/// Parses a command-backed `Defines:` group.
///
/// `Defines` groups introduce command signatures for specification/type-like
/// statements and support the same auxiliary sections as `Describes`, except
/// for the `expresses:` clause in place of form-specific sections.
pub(in crate::frontend::structural::parser) fn parse_defines(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<DefinesGroup> {
    let heading = parse_required_command_heading(group, tracker)?;
    let sections = identify_sections(
        "Defines",
        &group.sections,
        tracker,
        &[
            "Defines",
            "using?",
            "when?",
            "expresses?",
            "Provides?",
            "Justified?",
            "Documented?",
            "Aliases?",
            "References?",
            "Metadata?",
        ],
    )?;

    Some(DefinesGroup {
        heading,
        defines: DefinesSection {
            argument: parse_required_formulation(
                section(&sections, "Defines")?,
                "Defines",
                tracker,
                parse_is_or_spec,
            )?,
        },
        using: sections.get("using").copied().and_then(|section| {
            parse_required_formulations(section, "using", tracker, parse_is_or_spec)
                .map(|arguments| UsingSection { arguments })
        }),
        when: sections.get("when").copied().and_then(|section| {
            parse_required_clauses(section, "when", tracker)
                .map(|arguments| WhenSection { arguments })
        }),
        expresses: sections.get("expresses").copied().and_then(|section| {
            parse_required_clause(section, "expresses", tracker)
                .map(|argument| ExpressesSection { argument })
        }),
        provides: sections.get("Provides").copied().and_then(|section| {
            parse_required_groups(section, "Provides", tracker, parse_provides_item_group)
                .map(|arguments| ProvidesSection { arguments })
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

/// Parses a command-backed `Refines:` group.
///
/// Refines groups define a refined command signature and validate their
/// `Refines:`/`specifies:` bodies with the parser variant that accepts refined
/// command references.
pub(in crate::frontend::structural::parser) fn parse_refines(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<RefinesGroup> {
    let heading = parse_required_command_heading(group, tracker)?;
    let sections = identify_sections(
        "Refines",
        &group.sections,
        tracker,
        &[
            "Refines",
            "using?",
            "when?",
            "specifies?",
            "satisfies?",
            "Provides?",
            "Justified?",
            "Documented?",
            "Aliases?",
            "References?",
            "Metadata?",
        ],
    )?;

    Some(RefinesGroup {
        heading,
        refines: RefinesSection {
            argument: parse_required_formulation(
                section(&sections, "Refines")?,
                "Refines",
                tracker,
                parse_is_or_refined_statement_spec,
            )?,
        },
        using: sections.get("using").copied().and_then(|section| {
            parse_required_formulations(section, "using", tracker, parse_is_or_spec)
                .map(|arguments| UsingSection { arguments })
        }),
        when: sections.get("when").copied().and_then(|section| {
            parse_required_clauses(section, "when", tracker)
                .map(|arguments| WhenSection { arguments })
        }),
        specifies: sections.get("specifies").copied().and_then(|section| {
            parse_required_formulation(
                section,
                "specifies",
                tracker,
                parse_is_or_refined_statement_spec,
            )
            .map(|argument| RefinesSpecifiesSection { argument })
        }),
        satisfies: sections.get("satisfies").copied().and_then(|section| {
            parse_required_clauses(section, "satisfies", tracker)
                .map(|arguments| SatisfiesSection { arguments })
        }),
        provides: sections.get("Provides").copied().and_then(|section| {
            parse_required_groups(section, "Provides", tracker, parse_provides_item_group)
                .map(|arguments| ProvidesSection { arguments })
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

/// Parses a command-backed `States:` group.
///
/// The `that:` section is required and supplies the statement body.  Optional
/// prose in `States:` is retained for documentation/rendering contexts.
pub(in crate::frontend::structural::parser) fn parse_states(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<StatesGroup> {
    let heading = parse_required_command_heading(group, tracker)?;
    let sections = identify_sections(
        "States",
        &group.sections,
        tracker,
        &[
            "States",
            "using?",
            "when?",
            "that",
            "Provides?",
            "Justified?",
            "Documented?",
            "Aliases?",
            "References?",
            "Metadata?",
        ],
    )?;

    Some(StatesGroup {
        heading,
        states: StatesSection {
            arguments: parse_optional_open_texts(sections.get("States").copied(), tracker),
        },
        using: sections.get("using").copied().and_then(|section| {
            parse_required_formulations(section, "using", tracker, parse_is_or_spec)
                .map(|arguments| UsingSection { arguments })
        }),
        when: sections.get("when").copied().and_then(|section| {
            parse_required_clauses(section, "when", tracker)
                .map(|arguments| WhenSection { arguments })
        }),
        that: ThatSection {
            arguments: parse_required_clauses(section(&sections, "that")?, "that", tracker)?,
        },
        provides: sections.get("Provides").copied().and_then(|section| {
            parse_required_groups(section, "Provides", tracker, parse_provides_item_group)
                .map(|arguments| ProvidesSection { arguments })
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

// ===============================[ theorems ]=====================================

/// Parses an `Axiom:` group using the shared theorem-like parser.
///
/// The returned shared tuple is adapted to the axiom-specific section wrapper.
pub(in crate::frontend::structural::parser) fn parse_axiom(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<AxiomGroup> {
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
pub(in crate::frontend::structural::parser) fn parse_theorem(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<TheoremGroup> {
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
pub(in crate::frontend::structural::parser) fn parse_lemma(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<LemmaGroup> {
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
pub(in crate::frontend::structural::parser) fn parse_conjecture(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ConjectureGroup> {
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
pub(in crate::frontend::structural::parser) fn parse_argument_theorem_like(
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
pub(in crate::frontend::structural::parser) fn parse_corollary(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<CorollaryGroup> {
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

// ===============================[ metadata ]=====================================

/// Parses a `Person:` metadata group.
///
/// Person groups require an author-style heading and carry required `name:` and
/// `biography:` sections, with optional prose on the leading `Person:` section.
pub(in crate::frontend::structural::parser) fn parse_person(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<PersonGroup> {
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
pub(in crate::frontend::structural::parser) fn parse_resource(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ResourceGroup> {
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
pub(in crate::frontend::structural::parser) fn parse_specify(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<SpecifyGroup> {
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

use std::collections::{HashMap, VecDeque};

use crate::diagnostics::DiagnosticTracker;
use crate::frontend::formulation::{
    ParseError as FormulationParseError, parse_author_header, parse_command_header,
    parse_expression, parse_expression_alias, parse_form_or_declaration,
    parse_is_or_refined_statement_spec, parse_is_or_spec, parse_is_via_statement,
    parse_label_header, parse_resource_header, parse_spec_operator_alias, parse_writing_alias,
};
use crate::frontend::proto::Parser as ProtoParser;
use crate::frontend::proto::ast::{
    Argument as ProtoArgument, Formulation as ProtoFormulation, Group as ProtoGroup,
    Section as ProtoSection, TextLiteral as ProtoText,
};

use super::ast::*;

pub fn parse_document(input: &str, diagnostics: &mut DiagnosticTracker) -> Document {
    let groups = {
        let mut proto_parser = ProtoParser::new(input, diagnostics);
        proto_parser.parse()
    };

    let mut items = Vec::new();
    for group in &groups {
        if let Some(item) = parse_top_level_group(group, diagnostics) {
            items.push(item);
        }
    }

    Document {
        items: ZeroOrMore::from(items),
    }
}

fn parse_top_level_group(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<TopLevelItem> {
    let label = first_section_label(group)?;
    match label {
        "Title" => parse_title(group, diagnostics).map(TopLevelItem::Title),
        "Section" => parse_section(group, diagnostics).map(TopLevelItem::Section),
        "Subsection" => parse_subsection(group, diagnostics).map(TopLevelItem::Subsection),
        "Subsubsection" => parse_subsubsection(group, diagnostics).map(TopLevelItem::Subsubsection),
        "Describes" => parse_describes(group, diagnostics).map(TopLevelItem::Describes),
        "Defines" => parse_defines(group, diagnostics).map(TopLevelItem::Defines),
        "Refines" => parse_refines(group, diagnostics).map(TopLevelItem::Refines),
        "States" => parse_states(group, diagnostics).map(TopLevelItem::States),
        "Axiom" => parse_axiom(group, diagnostics).map(TopLevelItem::Axiom),
        "Theorem" => parse_theorem(group, diagnostics).map(TopLevelItem::Theorem),
        "Corollary" => parse_corollary(group, diagnostics).map(TopLevelItem::Corollary),
        "Lemma" => parse_lemma(group, diagnostics).map(TopLevelItem::Lemma),
        "Conjecture" => parse_conjecture(group, diagnostics).map(TopLevelItem::Conjecture),
        "Person" => parse_person(group, diagnostics).map(TopLevelItem::Person),
        "Resource" => parse_resource(group, diagnostics).map(TopLevelItem::Resource),
        "Specify" => parse_specify(group, diagnostics).map(TopLevelItem::Specify),
        other => {
            diagnostics.error(
                group.metadata.row,
                format!("Unexpected top-level group `{other}`"),
            );
            None
        }
    }
}

fn parse_title(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<TitleGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections("Title", &group.sections, diagnostics, &["Title"])?;
    Some(TitleGroup {
        title: TitleSection {
            argument: parse_required_open_text(section(&sections, "Title")?, "Title", diagnostics)?,
        },
    })
}

fn parse_section(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<SectionGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections("Section", &group.sections, diagnostics, &["Section"])?;
    Some(SectionGroup {
        section: SectionSection {
            argument: parse_required_open_text(
                section(&sections, "Section")?,
                "Section",
                diagnostics,
            )?,
        },
    })
}

fn parse_subsection(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<SubsectionGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections("Subsection", &group.sections, diagnostics, &["Subsection"])?;
    Some(SubsectionGroup {
        subsection: SubsectionSection {
            argument: parse_required_open_text(
                section(&sections, "Subsection")?,
                "Subsection",
                diagnostics,
            )?,
        },
    })
}

fn parse_subsubsection(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<SubsubsectionGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections(
        "Subsubsection",
        &group.sections,
        diagnostics,
        &["Subsubsection"],
    )?;
    Some(SubsubsectionGroup {
        subsubsection: SubsubsectionSection {
            argument: parse_required_open_text(
                section(&sections, "Subsubsection")?,
                "Subsubsection",
                diagnostics,
            )?,
        },
    })
}

fn parse_describes(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<DescribesGroup> {
    let heading = parse_required_command_heading(group, diagnostics)?;
    let sections = identify_sections(
        "Describes",
        &group.sections,
        diagnostics,
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
                diagnostics,
                parse_form_or_declaration,
            )?,
        },
        using: sections.get("using").copied().and_then(|section| {
            parse_required_formulations(section, "using", diagnostics, parse_is_or_spec)
                .map(|arguments| UsingSection { arguments })
        }),
        when: sections.get("when").copied().and_then(|section| {
            parse_required_clauses(section, "when", diagnostics)
                .map(|arguments| WhenSection { arguments })
        }),
        extends: sections.get("extends").copied().and_then(|section| {
            parse_required_formulation(section, "extends", diagnostics, parse_is_or_via_item)
                .map(|argument| ExtendsSection { argument })
        }),
        specifies: sections.get("specifies").copied().and_then(|section| {
            parse_required_formulations(section, "specifies", diagnostics, parse_is_or_via_item)
                .map(|arguments| DescribesSpecifiesSection { arguments })
        }),
        satisfies: sections.get("satisfies").copied().and_then(|section| {
            parse_required_clauses(section, "satisfies", diagnostics)
                .map(|arguments| SatisfiesSection { arguments })
        }),
        provides: sections.get("Provides").copied().and_then(|section| {
            parse_required_groups(section, "Provides", diagnostics, parse_provides_item_group)
                .map(|arguments| ProvidesSection { arguments })
        }),
        justified: sections.get("Justified").copied().and_then(|section| {
            parse_required_groups(
                section,
                "Justified",
                diagnostics,
                parse_justified_item_group,
            )
            .map(|arguments| JustifiedSection { arguments })
        }),
        documented: sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(
                section,
                "Documented",
                diagnostics,
                parse_documented_item_group,
            )
            .map(|arguments| DocumentedSection { arguments })
        }),
        aliases: sections.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", diagnostics, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        references: sections.get("References").copied().and_then(|section| {
            parse_required_formulations(section, "References", diagnostics, parse_resource_header)
                .map(|arguments| ReferencesSection { arguments })
        }),
        metadata: sections.get("Metadata").copied().and_then(|section| {
            parse_required_groups(section, "Metadata", diagnostics, parse_metadata_item_group)
                .map(|arguments| MetadataSection { arguments })
        }),
    })
}

fn parse_defines(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<DefinesGroup> {
    let heading = parse_required_command_heading(group, diagnostics)?;
    let sections = identify_sections(
        "Defines",
        &group.sections,
        diagnostics,
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
                diagnostics,
                parse_is_or_spec,
            )?,
        },
        using: sections.get("using").copied().and_then(|section| {
            parse_required_formulations(section, "using", diagnostics, parse_is_or_spec)
                .map(|arguments| UsingSection { arguments })
        }),
        when: sections.get("when").copied().and_then(|section| {
            parse_required_clauses(section, "when", diagnostics)
                .map(|arguments| WhenSection { arguments })
        }),
        expresses: sections.get("expresses").copied().and_then(|section| {
            parse_required_clause(section, "expresses", diagnostics)
                .map(|argument| ExpressesSection { argument })
        }),
        provides: sections.get("Provides").copied().and_then(|section| {
            parse_required_groups(section, "Provides", diagnostics, parse_provides_item_group)
                .map(|arguments| ProvidesSection { arguments })
        }),
        justified: sections.get("Justified").copied().and_then(|section| {
            parse_required_groups(
                section,
                "Justified",
                diagnostics,
                parse_justified_item_group,
            )
            .map(|arguments| JustifiedSection { arguments })
        }),
        documented: sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(
                section,
                "Documented",
                diagnostics,
                parse_documented_item_group,
            )
            .map(|arguments| DocumentedSection { arguments })
        }),
        aliases: sections.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", diagnostics, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        references: sections.get("References").copied().and_then(|section| {
            parse_required_formulations(section, "References", diagnostics, parse_resource_header)
                .map(|arguments| ReferencesSection { arguments })
        }),
        metadata: sections.get("Metadata").copied().and_then(|section| {
            parse_required_groups(section, "Metadata", diagnostics, parse_metadata_item_group)
                .map(|arguments| MetadataSection { arguments })
        }),
    })
}

fn parse_refines(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<RefinesGroup> {
    let heading = parse_required_command_heading(group, diagnostics)?;
    let sections = identify_sections(
        "Refines",
        &group.sections,
        diagnostics,
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
                diagnostics,
                parse_is_or_refined_statement_spec,
            )?,
        },
        using: sections.get("using").copied().and_then(|section| {
            parse_required_formulations(section, "using", diagnostics, parse_is_or_spec)
                .map(|arguments| UsingSection { arguments })
        }),
        when: sections.get("when").copied().and_then(|section| {
            parse_required_clauses(section, "when", diagnostics)
                .map(|arguments| WhenSection { arguments })
        }),
        specifies: sections.get("specifies").copied().and_then(|section| {
            parse_required_formulation(
                section,
                "specifies",
                diagnostics,
                parse_is_or_refined_statement_spec,
            )
            .map(|argument| RefinesSpecifiesSection { argument })
        }),
        satisfies: sections.get("satisfies").copied().and_then(|section| {
            parse_required_clauses(section, "satisfies", diagnostics)
                .map(|arguments| SatisfiesSection { arguments })
        }),
        provides: sections.get("Provides").copied().and_then(|section| {
            parse_required_groups(section, "Provides", diagnostics, parse_provides_item_group)
                .map(|arguments| ProvidesSection { arguments })
        }),
        justified: sections.get("Justified").copied().and_then(|section| {
            parse_required_groups(
                section,
                "Justified",
                diagnostics,
                parse_justified_item_group,
            )
            .map(|arguments| JustifiedSection { arguments })
        }),
        documented: sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(
                section,
                "Documented",
                diagnostics,
                parse_documented_item_group,
            )
            .map(|arguments| DocumentedSection { arguments })
        }),
        aliases: sections.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", diagnostics, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        references: sections.get("References").copied().and_then(|section| {
            parse_required_formulations(section, "References", diagnostics, parse_resource_header)
                .map(|arguments| ReferencesSection { arguments })
        }),
        metadata: sections.get("Metadata").copied().and_then(|section| {
            parse_required_groups(section, "Metadata", diagnostics, parse_metadata_item_group)
                .map(|arguments| MetadataSection { arguments })
        }),
    })
}

fn parse_states(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<StatesGroup> {
    let heading = parse_required_command_heading(group, diagnostics)?;
    let sections = identify_sections(
        "States",
        &group.sections,
        diagnostics,
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
            arguments: parse_optional_open_texts(sections.get("States").copied(), diagnostics),
        },
        using: sections.get("using").copied().and_then(|section| {
            parse_required_formulations(section, "using", diagnostics, parse_is_or_spec)
                .map(|arguments| UsingSection { arguments })
        }),
        when: sections.get("when").copied().and_then(|section| {
            parse_required_clauses(section, "when", diagnostics)
                .map(|arguments| WhenSection { arguments })
        }),
        that: ThatSection {
            arguments: parse_required_clauses(section(&sections, "that")?, "that", diagnostics)?,
        },
        provides: sections.get("Provides").copied().and_then(|section| {
            parse_required_groups(section, "Provides", diagnostics, parse_provides_item_group)
                .map(|arguments| ProvidesSection { arguments })
        }),
        justified: sections.get("Justified").copied().and_then(|section| {
            parse_required_groups(
                section,
                "Justified",
                diagnostics,
                parse_justified_item_group,
            )
            .map(|arguments| JustifiedSection { arguments })
        }),
        documented: sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(
                section,
                "Documented",
                diagnostics,
                parse_documented_item_group,
            )
            .map(|arguments| DocumentedSection { arguments })
        }),
        aliases: sections.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", diagnostics, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        references: sections.get("References").copied().and_then(|section| {
            parse_required_formulations(section, "References", diagnostics, parse_resource_header)
                .map(|arguments| ReferencesSection { arguments })
        }),
        metadata: sections.get("Metadata").copied().and_then(|section| {
            parse_required_groups(section, "Metadata", diagnostics, parse_metadata_item_group)
                .map(|arguments| MetadataSection { arguments })
        }),
    })
}

fn parse_axiom(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<AxiomGroup> {
    parse_argument_theorem_like(group, diagnostics, "Axiom").map(
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

fn parse_theorem(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<TheoremGroup> {
    parse_argument_theorem_like(group, diagnostics, "Theorem").map(
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

fn parse_lemma(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<LemmaGroup> {
    parse_argument_theorem_like(group, diagnostics, "Lemma").map(
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

fn parse_conjecture(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ConjectureGroup> {
    parse_argument_theorem_like(group, diagnostics, "Conjecture").map(
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
fn parse_argument_theorem_like(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
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
    let heading = parse_optional_command_heading(group, diagnostics)?;
    let section_name = name;
    let sections = identify_sections(
        name,
        &group.sections,
        diagnostics,
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
        parse_optional_open_texts(sections.get(section_name).copied(), diagnostics),
        sections.get("given").copied().and_then(|section| {
            parse_required_formulations(section, "given", diagnostics, parse_is_or_spec)
                .map(|arguments| GivenSection { arguments })
        }),
        sections.get("where").copied().and_then(|section| {
            parse_required_clauses(section, "where", diagnostics)
                .map(|arguments| WhereSection { arguments })
        }),
        ThenSection {
            arguments: parse_required_clauses(section(&sections, "then")?, "then", diagnostics)?,
        },
        sections.get("iff").copied().and_then(|section| {
            parse_required_clauses(section, "iff", diagnostics)
                .map(|arguments| IffSection { arguments })
        }),
        sections.get("Justified").copied().and_then(|section| {
            parse_required_groups(
                section,
                "Justified",
                diagnostics,
                parse_justified_item_group,
            )
            .map(|arguments| JustifiedSection { arguments })
        }),
        sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(
                section,
                "Documented",
                diagnostics,
                parse_documented_item_group,
            )
            .map(|arguments| DocumentedSection { arguments })
        }),
        sections.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", diagnostics, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        sections.get("References").copied().and_then(|section| {
            parse_required_formulations(section, "References", diagnostics, parse_resource_header)
                .map(|arguments| ReferencesSection { arguments })
        }),
        sections.get("Metadata").copied().and_then(|section| {
            parse_required_groups(section, "Metadata", diagnostics, parse_metadata_item_group)
                .map(|arguments| MetadataSection { arguments })
        }),
    ))
}

fn parse_corollary(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<CorollaryGroup> {
    let heading = parse_optional_command_heading(group, diagnostics)?;
    let sections = identify_sections(
        "Corollary",
        &group.sections,
        diagnostics,
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
            arguments: parse_optional_open_texts(sections.get("Corollary").copied(), diagnostics),
        },
        of: OfSection {
            arguments: parse_optional_open_texts(sections.get("of").copied(), diagnostics),
        },
        given: sections.get("given").copied().and_then(|section| {
            parse_required_formulations(section, "given", diagnostics, parse_is_or_spec)
                .map(|arguments| GivenSection { arguments })
        }),
        where_: sections.get("where").copied().and_then(|section| {
            parse_required_clauses(section, "where", diagnostics)
                .map(|arguments| WhereSection { arguments })
        }),
        then: ThenSection {
            arguments: parse_required_clauses(section(&sections, "then")?, "then", diagnostics)?,
        },
        iff: sections.get("iff").copied().and_then(|section| {
            parse_required_clauses(section, "iff", diagnostics)
                .map(|arguments| IffSection { arguments })
        }),
        justified: sections.get("Justified").copied().and_then(|section| {
            parse_required_groups(
                section,
                "Justified",
                diagnostics,
                parse_justified_item_group,
            )
            .map(|arguments| JustifiedSection { arguments })
        }),
        documented: sections.get("Documented").copied().and_then(|section| {
            parse_required_groups(
                section,
                "Documented",
                diagnostics,
                parse_documented_item_group,
            )
            .map(|arguments| DocumentedSection { arguments })
        }),
        aliases: sections.get("Aliases").copied().and_then(|section| {
            parse_required_groups(section, "Aliases", diagnostics, parse_alias_item_group)
                .map(|arguments| AliasesSection { arguments })
        }),
        references: sections.get("References").copied().and_then(|section| {
            parse_required_formulations(section, "References", diagnostics, parse_resource_header)
                .map(|arguments| ReferencesSection { arguments })
        }),
        metadata: sections.get("Metadata").copied().and_then(|section| {
            parse_required_groups(section, "Metadata", diagnostics, parse_metadata_item_group)
                .map(|arguments| MetadataSection { arguments })
        }),
    })
}

fn parse_person(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<PersonGroup> {
    let heading = parse_required_author_heading(group, diagnostics)?;
    let sections = identify_sections(
        "Person",
        &group.sections,
        diagnostics,
        &["Person", "name", "biography"],
    )?;

    Some(PersonGroup {
        heading,
        person: PersonSection {
            arguments: parse_optional_open_texts(sections.get("Person").copied(), diagnostics),
        },
        name: NameSection {
            arguments: parse_required_open_texts(section(&sections, "name")?, "name", diagnostics)?,
        },
        biography: BiographySection {
            argument: parse_required_open_text(
                section(&sections, "biography")?,
                "biography",
                diagnostics,
            )?,
        },
    })
}

fn parse_resource(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ResourceGroup> {
    let heading = parse_required_resource_heading(group, diagnostics)?;
    let sections = identify_sections("Resource", &group.sections, diagnostics, &["Resource"])?;

    Some(ResourceGroup {
        heading,
        resource: ResourceSection {
            arguments: parse_required_groups(
                section(&sections, "Resource")?,
                "Resource",
                diagnostics,
                parse_resource_item_group,
            )?,
        },
    })
}

fn parse_specify(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<SpecifyGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections("Specify", &group.sections, diagnostics, &["Specify"])?;
    Some(SpecifyGroup {
        specify: SpecifySection {
            arguments: parse_required_groups(
                section(&sections, "Specify")?,
                "Specify",
                diagnostics,
                parse_specify_item_group,
            )?,
        },
    })
}

fn parse_alias_group(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<AliasGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections(
        "alias",
        &group.sections,
        diagnostics,
        &["alias", "written?"],
    )?;
    Some(AliasGroup {
        heading,
        alias: AliasSection {
            argument: parse_required_formulation(
                section(&sections, "alias")?,
                "alias",
                diagnostics,
                parse_alias_kind,
            )?,
        },
        written: sections.get("written").copied().and_then(|section| {
            parse_required_written_texts(section, diagnostics)
                .map(|arguments| WrittenSection { arguments })
        }),
    })
}

fn parse_symbol(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<SymbolGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections(
        "symbol",
        &group.sections,
        diagnostics,
        &["symbol", "written?"],
    )?;
    Some(SymbolGroup {
        heading,
        symbol: SymbolSection {
            argument: parse_required_formulation(
                section(&sections, "symbol")?,
                "symbol",
                diagnostics,
                parse_alias_kind,
            )?,
        },
        written: sections.get("written").copied().and_then(|section| {
            parse_required_written_texts(section, diagnostics)
                .map(|arguments| WrittenSection { arguments })
        }),
    })
}

fn parse_connection(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ConnectionGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections(
        "connection",
        &group.sections,
        diagnostics,
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
            arguments: parse_optional_open_texts(sections.get("connection").copied(), diagnostics),
        },
        to: ToSection {
            arguments: parse_optional_open_texts(sections.get("to").copied(), diagnostics),
        },
        using: sections.get("using").copied().and_then(|section| {
            parse_required_formulations(section, "using", diagnostics, parse_is_or_spec)
                .map(|arguments| UsingSection { arguments })
        }),
        means: MeansSection {
            arguments: parse_optional_open_texts(sections.get("means").copied(), diagnostics),
        },
        signifies: sections
            .get("signifies")
            .copied()
            .map(|section| SignifiesSection {
                arguments: parse_optional_open_texts(Some(section), diagnostics),
            }),
        viewable: sections
            .get("viewable")
            .copied()
            .map(|section| ViewableSection {
                arguments: parse_optional_open_texts(Some(section), diagnostics),
            }),
        through: sections
            .get("through")
            .copied()
            .map(|section| ThroughSection {
                arguments: parse_optional_open_texts(Some(section), diagnostics),
            }),
    })
}

fn parse_written(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<WrittenGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections("written", &group.sections, diagnostics, &["written"])?;
    Some(WrittenGroup {
        heading,
        written: WrittenSection {
            arguments: parse_required_written_texts(section(&sections, "written")?, diagnostics)?,
        },
    })
}

fn parse_called(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<CalledGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections("called", &group.sections, diagnostics, &["called"])?;
    Some(CalledGroup {
        heading,
        called: CalledSection {
            arguments: parse_required_called_texts(section(&sections, "called")?, diagnostics)?,
        },
    })
}

fn parse_writing(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<WritingGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections("writing", &group.sections, diagnostics, &["writing", "as"])?;
    Some(WritingGroup {
        heading,
        writing: WritingSection {
            argument: parse_required_formulation(
                section(&sections, "writing")?,
                "writing",
                diagnostics,
                parse_writing_alias,
            )?,
        },
        as_: AsSection {
            arguments: parse_required_writing_texts(section(&sections, "as")?, diagnostics)?,
        },
    })
}

fn parse_overview(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<OverviewGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections("overview", &group.sections, diagnostics, &["overview"])?;
    Some(OverviewGroup {
        heading,
        overview: OverviewSection {
            argument: parse_required_open_text(
                section(&sections, "overview")?,
                "overview",
                diagnostics,
            )?,
        },
    })
}

fn parse_related(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<RelatedGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections("related", &group.sections, diagnostics, &["related"])?;
    Some(RelatedGroup {
        heading,
        related: RelatedSection {
            arguments: parse_required_open_texts(
                section(&sections, "related")?,
                "related",
                diagnostics,
            )?,
        },
    })
}

fn parse_discoverer(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<DiscovererGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections("discoverer", &group.sections, diagnostics, &["discoverer"])?;
    Some(DiscovererGroup {
        heading,
        discoverer: DiscovererSection {
            arguments: parse_optional_open_texts(sections.get("discoverer").copied(), diagnostics),
        },
    })
}

fn parse_label_note(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<LabelGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections(
        "label",
        &group.sections,
        diagnostics,
        &["label", "by", "comment"],
    )?;
    Some(LabelGroup {
        heading,
        label: LabelSection {
            arguments: parse_optional_open_texts(sections.get("label").copied(), diagnostics),
        },
        by: BySection {
            arguments: parse_optional_open_texts(sections.get("by").copied(), diagnostics),
        },
        comment: CommentSection {
            argument: parse_required_open_text(
                section(&sections, "comment")?,
                "comment",
                diagnostics,
            )?,
        },
    })
}

fn parse_by_note(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<ByGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections("by", &group.sections, diagnostics, &["by", "comment"])?;
    Some(ByGroup {
        heading,
        by: BySection {
            arguments: parse_optional_open_texts(sections.get("by").copied(), diagnostics),
        },
        comment: CommentSection {
            argument: parse_required_open_text(
                section(&sections, "comment")?,
                "comment",
                diagnostics,
            )?,
        },
    })
}

fn parse_id(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<IdGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections("id", &group.sections, diagnostics, &["id"])?;
    Some(IdGroup {
        id: IdSection {
            argument: parse_required_open_text(section(&sections, "id")?, "id", diagnostics)?,
        },
    })
}

fn parse_version(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<VersionGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections("version", &group.sections, diagnostics, &["version"])?;
    Some(VersionGroup {
        version: VersionSection {
            argument: parse_required_open_text(
                section(&sections, "version")?,
                "version",
                diagnostics,
            )?,
        },
    })
}

fn parse_positive_int(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<PositiveIntGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections(
        "positive_int",
        &group.sections,
        diagnostics,
        &["positive", "int", "is"],
    )?;
    Some(PositiveIntGroup {
        heading,
        positive: PositiveSection {
            arguments: parse_optional_open_texts(sections.get("positive").copied(), diagnostics),
        },
        int: IntSection {
            arguments: parse_optional_open_texts(sections.get("int").copied(), diagnostics),
        },
        is_: IsSection {
            arguments: parse_optional_open_texts(sections.get("is").copied(), diagnostics),
        },
    })
}

fn parse_negative_int(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<NegativeIntGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections(
        "negative_int",
        &group.sections,
        diagnostics,
        &["negative", "int", "is"],
    )?;
    Some(NegativeIntGroup {
        heading,
        negative: NegativeSection {
            arguments: parse_optional_open_texts(sections.get("negative").copied(), diagnostics),
        },
        int: IntSection {
            arguments: parse_optional_open_texts(sections.get("int").copied(), diagnostics),
        },
        is_: IsSection {
            arguments: parse_optional_open_texts(sections.get("is").copied(), diagnostics),
        },
    })
}

fn parse_zero(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<ZeroGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections("zero", &group.sections, diagnostics, &["zero", "is"])?;
    Some(ZeroGroup {
        heading,
        zero: ZeroSection {
            arguments: parse_optional_open_texts(sections.get("zero").copied(), diagnostics),
        },
        is_: IsSection {
            arguments: parse_optional_open_texts(sections.get("is").copied(), diagnostics),
        },
    })
}

fn parse_positive_decimal(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<PositiveDecimalGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections(
        "positive_decimal",
        &group.sections,
        diagnostics,
        &["positive", "decimal", "is"],
    )?;
    Some(PositiveDecimalGroup {
        heading,
        positive: PositiveSection {
            arguments: parse_optional_open_texts(sections.get("positive").copied(), diagnostics),
        },
        decimal: DecimalSection {
            arguments: parse_optional_open_texts(sections.get("decimal").copied(), diagnostics),
        },
        is_: IsSection {
            arguments: parse_optional_open_texts(sections.get("is").copied(), diagnostics),
        },
    })
}

fn parse_negative_decimal(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<NegativeDecimalGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections(
        "negative_decimal",
        &group.sections,
        diagnostics,
        &["negative", "decimal", "is"],
    )?;
    Some(NegativeDecimalGroup {
        heading,
        negative: NegativeSection {
            arguments: parse_optional_open_texts(sections.get("negative").copied(), diagnostics),
        },
        decimal: DecimalSection {
            arguments: parse_optional_open_texts(sections.get("decimal").copied(), diagnostics),
        },
        is_: IsSection {
            arguments: parse_optional_open_texts(sections.get("is").copied(), diagnostics),
        },
    })
}

fn parse_resource_title(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ResourceTitleGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections("title", &group.sections, diagnostics, &["title"])?;
    Some(ResourceTitleGroup {
        title: ResourceTitleSection {
            argument: parse_required_open_text(section(&sections, "title")?, "title", diagnostics)?,
        },
    })
}

fn parse_resource_author(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ResourceAuthorGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections("author", &group.sections, diagnostics, &["author"])?;
    Some(ResourceAuthorGroup {
        author: ResourceAuthorSection {
            arguments: parse_required_open_texts(
                section(&sections, "author")?,
                "author",
                diagnostics,
            )?,
        },
    })
}

fn parse_resource_offset(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ResourceOffsetGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections("offset", &group.sections, diagnostics, &["offset"])?;
    Some(ResourceOffsetGroup {
        offset: ResourceOffsetSection {
            argument: parse_required_open_text(
                section(&sections, "offset")?,
                "offset",
                diagnostics,
            )?,
        },
    })
}

fn parse_resource_url(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ResourceUrlGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections("url", &group.sections, diagnostics, &["url"])?;
    Some(ResourceUrlGroup {
        url: ResourceUrlSection {
            argument: parse_required_open_text(section(&sections, "url")?, "url", diagnostics)?,
        },
    })
}

fn parse_resource_homepage(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ResourceHomepageGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections("homepage", &group.sections, diagnostics, &["homepage"])?;
    Some(ResourceHomepageGroup {
        homepage: ResourceHomepageSection {
            argument: parse_required_open_text(
                section(&sections, "homepage")?,
                "homepage",
                diagnostics,
            )?,
        },
    })
}

fn parse_resource_type(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ResourceTypeGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections("type", &group.sections, diagnostics, &["type"])?;
    Some(ResourceTypeGroup {
        type_: ResourceTypeSection {
            argument: parse_required_open_text(section(&sections, "type")?, "type", diagnostics)?,
        },
    })
}

fn parse_resource_edition(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ResourceEditionGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections("edition", &group.sections, diagnostics, &["edition"])?;
    Some(ResourceEditionGroup {
        edition: ResourceEditionSection {
            argument: parse_required_open_text(
                section(&sections, "edition")?,
                "edition",
                diagnostics,
            )?,
        },
    })
}

fn parse_resource_editor(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ResourceEditorGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections("editor", &group.sections, diagnostics, &["editor"])?;
    Some(ResourceEditorGroup {
        editor: ResourceEditorSection {
            argument: parse_required_open_text(
                section(&sections, "editor")?,
                "editor",
                diagnostics,
            )?,
        },
    })
}

fn parse_resource_institution(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ResourceInstitutionGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections(
        "institution",
        &group.sections,
        diagnostics,
        &["institution"],
    )?;
    Some(ResourceInstitutionGroup {
        institution: ResourceInstitutionSection {
            argument: parse_required_open_text(
                section(&sections, "institution")?,
                "institution",
                diagnostics,
            )?,
        },
    })
}

fn parse_resource_journal(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ResourceJournalGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections("journal", &group.sections, diagnostics, &["journal"])?;
    Some(ResourceJournalGroup {
        journal: ResourceJournalSection {
            argument: parse_required_open_text(
                section(&sections, "journal")?,
                "journal",
                diagnostics,
            )?,
        },
    })
}

fn parse_resource_publisher(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ResourcePublisherGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections("publisher", &group.sections, diagnostics, &["publisher"])?;
    Some(ResourcePublisherGroup {
        publisher: ResourcePublisherSection {
            argument: parse_required_open_text(
                section(&sections, "publisher")?,
                "publisher",
                diagnostics,
            )?,
        },
    })
}

fn parse_resource_volume(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ResourceVolumeGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections("volume", &group.sections, diagnostics, &["volume"])?;
    Some(ResourceVolumeGroup {
        volume: ResourceVolumeSection {
            argument: parse_required_open_text(
                section(&sections, "volume")?,
                "volume",
                diagnostics,
            )?,
        },
    })
}

fn parse_resource_month(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ResourceMonthGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections("month", &group.sections, diagnostics, &["month"])?;
    Some(ResourceMonthGroup {
        month: ResourceMonthSection {
            argument: parse_required_open_text(section(&sections, "month")?, "month", diagnostics)?,
        },
    })
}

fn parse_resource_year(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ResourceYearGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections("year", &group.sections, diagnostics, &["year"])?;
    Some(ResourceYearGroup {
        year: ResourceYearSection {
            argument: parse_required_open_text(section(&sections, "year")?, "year", diagnostics)?,
        },
    })
}

fn parse_resource_description(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ResourceDescriptionGroup> {
    ensure_no_heading(group, diagnostics)?;
    let sections = identify_sections(
        "description",
        &group.sections,
        diagnostics,
        &["description"],
    )?;
    Some(ResourceDescriptionGroup {
        description: ResourceDescriptionSection {
            argument: parse_required_open_text(
                section(&sections, "description")?,
                "description",
                diagnostics,
            )?,
        },
    })
}

fn parse_not_clause(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<NotGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections("not", &group.sections, diagnostics, &["not"])?;
    Some(NotGroup {
        heading,
        not: NotSection {
            argument: Box::new(parse_required_clause(
                section(&sections, "not")?,
                "not",
                diagnostics,
            )?),
        },
    })
}

fn parse_all_of_clause(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<AllOfGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections("allOf", &group.sections, diagnostics, &["allOf"])?;
    Some(AllOfGroup {
        heading,
        all_of: AllOfSection {
            arguments: parse_required_clauses(section(&sections, "allOf")?, "allOf", diagnostics)?,
        },
    })
}

fn parse_any_of_clause(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<AnyOfGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections("anyOf", &group.sections, diagnostics, &["anyOf"])?;
    Some(AnyOfGroup {
        heading,
        any_of: AnyOfSection {
            arguments: parse_required_clauses(section(&sections, "anyOf")?, "anyOf", diagnostics)?,
        },
    })
}

fn parse_one_of_clause(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<OneOfGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections("oneOf", &group.sections, diagnostics, &["oneOf"])?;
    Some(OneOfGroup {
        heading,
        one_of: OneOfSection {
            arguments: parse_required_clauses(section(&sections, "oneOf")?, "oneOf", diagnostics)?,
        },
    })
}

fn parse_exists_clause(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ExistsGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections(
        "exists",
        &group.sections,
        diagnostics,
        &["exists", "suchThat"],
    )?;
    Some(ExistsGroup {
        heading,
        exists: ExistsSection {
            argument: parse_required_formulation(
                section(&sections, "exists")?,
                "exists",
                diagnostics,
                parse_is_or_spec,
            )?,
        },
        such_that: SuchThatSection {
            arguments: parse_required_clauses(
                section(&sections, "suchThat")?,
                "suchThat",
                diagnostics,
            )?,
        },
    })
}

fn parse_exists_unique_clause(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ExistsUniqueGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections(
        "existsUnique",
        &group.sections,
        diagnostics,
        &["existsUnique", "suchThat"],
    )?;
    Some(ExistsUniqueGroup {
        heading,
        exists_unique: ExistsUniqueSection {
            argument: parse_required_formulation(
                section(&sections, "existsUnique")?,
                "existsUnique",
                diagnostics,
                parse_is_or_spec,
            )?,
        },
        such_that: SuchThatSection {
            arguments: parse_required_clauses(
                section(&sections, "suchThat")?,
                "suchThat",
                diagnostics,
            )?,
        },
    })
}

fn parse_for_all_clause(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ForAllGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections(
        "forAll",
        &group.sections,
        diagnostics,
        &["forAll", "where?", "then"],
    )?;
    Some(ForAllGroup {
        heading,
        for_all: ForAllSection {
            argument: parse_required_formulation(
                section(&sections, "forAll")?,
                "forAll",
                diagnostics,
                parse_is_or_spec,
            )?,
        },
        where_: sections.get("where").copied().and_then(|section| {
            parse_required_clauses(section, "where", diagnostics)
                .map(|arguments| WhereSection { arguments })
        }),
        then: ThenSection {
            arguments: parse_required_clauses(section(&sections, "then")?, "then", diagnostics)?,
        },
    })
}

fn parse_if_clause(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<IfGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections("if", &group.sections, diagnostics, &["if", "then"])?;
    Some(IfGroup {
        heading,
        if_: IfSection {
            arguments: parse_required_clauses(section(&sections, "if")?, "if", diagnostics)?,
        },
        then: ThenSection {
            arguments: parse_required_clauses(section(&sections, "then")?, "then", diagnostics)?,
        },
    })
}

fn parse_iff_clause(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<IffGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections("iff", &group.sections, diagnostics, &["iff", "then"])?;
    Some(IffGroup {
        heading,
        iff: IffSection {
            arguments: parse_required_clauses(section(&sections, "iff")?, "iff", diagnostics)?,
        },
        then: ThenSection {
            arguments: parse_required_clauses(section(&sections, "then")?, "then", diagnostics)?,
        },
    })
}

fn parse_piecewise_clause(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<PiecewiseGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections(
        "piecewise",
        &group.sections,
        diagnostics,
        &["piecewise", "if", "then", "else?"],
    )?;
    Some(PiecewiseGroup {
        heading,
        piecewise: PiecewiseSection {
            arguments: parse_optional_open_texts(sections.get("piecewise").copied(), diagnostics),
        },
        if_: IfSection {
            arguments: parse_required_clauses(section(&sections, "if")?, "if", diagnostics)?,
        },
        then: ThenSection {
            arguments: parse_required_clauses(section(&sections, "then")?, "then", diagnostics)?,
        },
        else_: sections.get("else").copied().and_then(|section| {
            parse_required_clauses(section, "else", diagnostics)
                .map(|arguments| ElseSection { arguments })
        }),
    })
}

fn parse_given_clause(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<GivenGroup> {
    let heading = parse_optional_label_heading(group, diagnostics)?;
    let sections = identify_sections(
        "given",
        &group.sections,
        diagnostics,
        &["given", "where?", "then"],
    )?;
    Some(GivenGroup {
        heading,
        given: GivenClauseSection {
            argument: parse_required_formulation(
                section(&sections, "given")?,
                "given",
                diagnostics,
                parse_is_or_spec,
            )?,
        },
        where_: sections.get("where").copied().and_then(|section| {
            parse_required_clauses(section, "where", diagnostics)
                .map(|arguments| WhereSection { arguments })
        }),
        then: ThenSection {
            arguments: parse_required_clauses(section(&sections, "then")?, "then", diagnostics)?,
        },
    })
}

fn parse_alias_item_group(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<AliasItem> {
    parse_alias_group(group, diagnostics).map(AliasItem::Alias)
}

fn parse_provides_item_group(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ProvidesItem> {
    match first_section_label(group)? {
        "symbol" => parse_symbol(group, diagnostics).map(ProvidesItem::Symbol),
        "connection" => parse_connection(group, diagnostics).map(ProvidesItem::Connection),
        other => {
            diagnostics.error(
                group.metadata.row,
                format!("Unexpected provides group `{other}`"),
            );
            None
        }
    }
}

fn parse_documented_item_group(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<DocumentedItem> {
    match first_section_label(group)? {
        "written" => parse_written(group, diagnostics).map(DocumentedItem::Written),
        "called" => parse_called(group, diagnostics).map(DocumentedItem::Called),
        "writing" => parse_writing(group, diagnostics).map(DocumentedItem::Writing),
        "overview" => parse_overview(group, diagnostics).map(DocumentedItem::Overview),
        "related" => parse_related(group, diagnostics).map(DocumentedItem::Related),
        "discoverer" => parse_discoverer(group, diagnostics).map(DocumentedItem::Discoverer),
        other => {
            diagnostics.error(
                group.metadata.row,
                format!("Unexpected documented group `{other}`"),
            );
            None
        }
    }
}

fn parse_justified_item_group(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<JustifiedItem> {
    match first_section_label(group)? {
        "label" => parse_label_note(group, diagnostics).map(JustifiedItem::Label),
        "by" => parse_by_note(group, diagnostics).map(JustifiedItem::By),
        other => {
            diagnostics.error(
                group.metadata.row,
                format!("Unexpected justified group `{other}`"),
            );
            None
        }
    }
}

fn parse_metadata_item_group(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<MetadataItem> {
    match first_section_label(group)? {
        "id" => parse_id(group, diagnostics).map(MetadataItem::Id),
        "version" => parse_version(group, diagnostics).map(MetadataItem::Version),
        other => {
            diagnostics.error(
                group.metadata.row,
                format!("Unexpected metadata group `{other}`"),
            );
            None
        }
    }
}

fn parse_specify_item_group(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<SpecifyItem> {
    match first_section_label(group)? {
        "positive" => {
            if group.sections.iter().any(|section| section.label == "int") {
                parse_positive_int(group, diagnostics).map(SpecifyItem::PositiveInt)
            } else {
                parse_positive_decimal(group, diagnostics).map(SpecifyItem::PositiveDecimal)
            }
        }
        "negative" => {
            if group.sections.iter().any(|section| section.label == "int") {
                parse_negative_int(group, diagnostics).map(SpecifyItem::NegativeInt)
            } else {
                parse_negative_decimal(group, diagnostics).map(SpecifyItem::NegativeDecimal)
            }
        }
        "zero" => parse_zero(group, diagnostics).map(SpecifyItem::Zero),
        other => {
            diagnostics.error(
                group.metadata.row,
                format!("Unexpected specify group `{other}`"),
            );
            None
        }
    }
}

fn parse_resource_item_group(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<ResourceItem> {
    match first_section_label(group)? {
        "title" => parse_resource_title(group, diagnostics).map(ResourceItem::Title),
        "author" => parse_resource_author(group, diagnostics).map(ResourceItem::Author),
        "offset" => parse_resource_offset(group, diagnostics).map(ResourceItem::Offset),
        "url" => parse_resource_url(group, diagnostics).map(ResourceItem::Url),
        "homepage" => parse_resource_homepage(group, diagnostics).map(ResourceItem::Homepage),
        "type" => parse_resource_type(group, diagnostics).map(ResourceItem::Type),
        "edition" => parse_resource_edition(group, diagnostics).map(ResourceItem::Edition),
        "editor" => parse_resource_editor(group, diagnostics).map(ResourceItem::Editor),
        "institution" => {
            parse_resource_institution(group, diagnostics).map(ResourceItem::Institution)
        }
        "journal" => parse_resource_journal(group, diagnostics).map(ResourceItem::Journal),
        "publisher" => parse_resource_publisher(group, diagnostics).map(ResourceItem::Publisher),
        "volume" => parse_resource_volume(group, diagnostics).map(ResourceItem::Volume),
        "month" => parse_resource_month(group, diagnostics).map(ResourceItem::Month),
        "year" => parse_resource_year(group, diagnostics).map(ResourceItem::Year),
        "description" => {
            parse_resource_description(group, diagnostics).map(ResourceItem::Description)
        }
        other => {
            diagnostics.error(
                group.metadata.row,
                format!("Unexpected resource group `{other}`"),
            );
            None
        }
    }
}

fn parse_clause_group(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<Clause> {
    match first_section_label(group)? {
        "not" => parse_not_clause(group, diagnostics).map(Clause::Not),
        "allOf" => parse_all_of_clause(group, diagnostics).map(Clause::AllOf),
        "anyOf" => parse_any_of_clause(group, diagnostics).map(Clause::AnyOf),
        "oneOf" => parse_one_of_clause(group, diagnostics).map(Clause::OneOf),
        "exists" => parse_exists_clause(group, diagnostics).map(Clause::Exists),
        "existsUnique" => parse_exists_unique_clause(group, diagnostics).map(Clause::ExistsUnique),
        "forAll" => parse_for_all_clause(group, diagnostics).map(Clause::ForAll),
        "if" => parse_if_clause(group, diagnostics).map(Clause::If),
        "iff" => parse_iff_clause(group, diagnostics).map(Clause::Iff),
        "piecewise" => parse_piecewise_clause(group, diagnostics).map(Clause::Piecewise),
        "given" => parse_given_clause(group, diagnostics).map(Clause::Given),
        other => {
            diagnostics.error(
                group.metadata.row,
                format!("Unexpected clause group `{other}`"),
            );
            None
        }
    }
}

fn parse_alias_kind(input: &str) -> Result<AliasKind, FormulationParseError> {
    if let Ok(alias) = parse_expression_alias(input) {
        return Ok(AliasKind::Expression(alias));
    }
    parse_spec_operator_alias(input).map(AliasKind::SpecOperator)
}

fn parse_is_or_via_item(input: &str) -> Result<IsOrViaItem, FormulationParseError> {
    if let Ok(item) = parse_is_via_statement(input) {
        return Ok(IsOrViaItem::IsVia(item));
    }
    parse_is_or_spec(input).map(IsOrViaItem::IsOrSpec)
}

fn parse_required_command_heading(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<crate::frontend::formulation::ast::CommandHeader> {
    let Some(heading) = group.heading.as_deref() else {
        diagnostics.error(group.metadata.row, "Expected command heading");
        return None;
    };
    match parse_command_header(heading) {
        Ok(heading) => Some(heading),
        Err(error) => {
            diagnostics.error(
                group.metadata.row,
                format!("Invalid command heading: {error}"),
            );
            None
        }
    }
}

fn parse_optional_command_heading(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<Option<crate::frontend::formulation::ast::CommandHeader>> {
    match group.heading.as_deref() {
        Some(heading) => match parse_command_header(heading) {
            Ok(heading) => Some(Some(heading)),
            Err(error) => {
                diagnostics.error(
                    group.metadata.row,
                    format!("Invalid command heading: {error}"),
                );
                None
            }
        },
        None => Some(None),
    }
}

fn parse_optional_label_heading(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<Option<crate::frontend::formulation::ast::LabelHeader>> {
    match group.heading.as_deref() {
        Some(heading) => match parse_label_header(heading) {
            Ok(heading) => Some(Some(heading)),
            Err(error) => {
                diagnostics.error(
                    group.metadata.row,
                    format!("Invalid label heading: {error}"),
                );
                None
            }
        },
        None => Some(None),
    }
}

fn parse_required_author_heading(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<crate::frontend::formulation::ast::AuthorHeader> {
    let Some(heading) = group.heading.as_deref() else {
        diagnostics.error(group.metadata.row, "Expected author heading");
        return None;
    };
    match parse_author_header(heading) {
        Ok(heading) => Some(heading),
        Err(error) => {
            diagnostics.error(
                group.metadata.row,
                format!("Invalid author heading: {error}"),
            );
            None
        }
    }
}

fn parse_required_resource_heading(
    group: &ProtoGroup,
    diagnostics: &mut DiagnosticTracker,
) -> Option<crate::frontend::formulation::ast::ResourceHeader> {
    let Some(heading) = group.heading.as_deref() else {
        diagnostics.error(group.metadata.row, "Expected resource heading");
        return None;
    };
    match parse_resource_header(heading) {
        Ok(heading) => Some(heading),
        Err(error) => {
            diagnostics.error(
                group.metadata.row,
                format!("Invalid resource heading: {error}"),
            );
            None
        }
    }
}

fn ensure_no_heading(group: &ProtoGroup, diagnostics: &mut DiagnosticTracker) -> Option<()> {
    if let Some(heading) = &group.heading {
        diagnostics.error(
            group.metadata.row,
            format!("Unexpected heading `{heading}`"),
        );
        None
    } else {
        Some(())
    }
}

fn parse_required_formulation<T>(
    section: &ProtoSection,
    label: &str,
    diagnostics: &mut DiagnosticTracker,
    parser: fn(&str) -> Result<T, FormulationParseError>,
) -> Option<T> {
    let items = parse_optional_formulations(Some(section), label, diagnostics, parser);
    if items.is_empty() {
        diagnostics.error(
            section.metadata.row,
            format!("Expected a {label} formulation"),
        );
        None
    } else {
        Some(items.into_iter().next().expect("non-empty formulations"))
    }
}

fn parse_required_formulations<T>(
    section: &ProtoSection,
    label: &str,
    diagnostics: &mut DiagnosticTracker,
    parser: fn(&str) -> Result<T, FormulationParseError>,
) -> Option<OneOrMore<T>> {
    let items = parse_optional_formulations(Some(section), label, diagnostics, parser);
    one_or_more(items, || {
        diagnostics.error(
            section.metadata.row,
            format!("Expected {label} formulations"),
        );
    })
}

fn parse_optional_formulations<T>(
    section: Option<&ProtoSection>,
    label: &str,
    diagnostics: &mut DiagnosticTracker,
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
                    Err(error) => {
                        diagnostics.error(row, format!("Invalid {label} formulation: {error}"))
                    }
                }
            }
            SectionEntry::Text { row, .. } => {
                diagnostics.error(row, format!("Expected formulation in section `{label}`"));
            }
            SectionEntry::Group { row, .. } => {
                diagnostics.error(row, format!("Expected formulation in section `{label}`"));
            }
        }
    }

    result.into()
}

fn parse_required_clause(
    section: &ProtoSection,
    label: &str,
    diagnostics: &mut DiagnosticTracker,
) -> Option<Clause> {
    let clauses = parse_optional_clauses(Some(section), label, diagnostics);
    if clauses.is_empty() {
        diagnostics.error(
            section.metadata.row,
            format!("Expected a clause in `{label}`"),
        );
        None
    } else {
        Some(clauses.into_iter().next().expect("non-empty clauses"))
    }
}

fn parse_required_clauses(
    section: &ProtoSection,
    label: &str,
    diagnostics: &mut DiagnosticTracker,
) -> Option<OneOrMore<Clause>> {
    let clauses = parse_optional_clauses(Some(section), label, diagnostics);
    one_or_more(clauses, || {
        diagnostics.error(
            section.metadata.row,
            format!("Expected clauses in `{label}`"),
        );
    })
}

fn parse_optional_clauses(
    section: Option<&ProtoSection>,
    label: &str,
    diagnostics: &mut DiagnosticTracker,
) -> ZeroOrMore<Clause> {
    let Some(section) = section else {
        return ZeroOrMore::default();
    };

    let mut result = Vec::new();
    for entry in section_entries(section) {
        match entry {
            SectionEntry::Inline { text, row } | SectionEntry::Formulation { text, row } => {
                match parse_expression(text) {
                    Ok(expression) => result.push(Clause::Expression(expression)),
                    Err(error) => diagnostics.error(
                        row,
                        format!("Invalid clause expression in `{label}`: {error}"),
                    ),
                }
            }
            SectionEntry::Group { group, .. } => {
                if let Some(clause) = parse_clause_group(group, diagnostics) {
                    result.push(clause);
                }
            }
            SectionEntry::Text { row, .. } => {
                diagnostics.error(row, format!("Expected clause in section `{label}`"));
            }
        }
    }

    result.into()
}

fn parse_required_groups<T>(
    section: &ProtoSection,
    label: &str,
    diagnostics: &mut DiagnosticTracker,
    parser: fn(&ProtoGroup, &mut DiagnosticTracker) -> Option<T>,
) -> Option<OneOrMore<T>> {
    let items = parse_optional_groups(Some(section), label, diagnostics, parser);
    one_or_more(items, || {
        diagnostics.error(
            section.metadata.row,
            format!("Expected nested groups in `{label}`"),
        );
    })
}

fn parse_optional_groups<T>(
    section: Option<&ProtoSection>,
    label: &str,
    diagnostics: &mut DiagnosticTracker,
    parser: fn(&ProtoGroup, &mut DiagnosticTracker) -> Option<T>,
) -> ZeroOrMore<T> {
    let Some(section) = section else {
        return ZeroOrMore::default();
    };

    let mut items = Vec::new();
    for entry in section_entries(section) {
        match entry {
            SectionEntry::Group { group, .. } => {
                if let Some(item) = parser(group, diagnostics) {
                    items.push(item);
                }
            }
            SectionEntry::Inline { row, .. }
            | SectionEntry::Formulation { row, .. }
            | SectionEntry::Text { row, .. } => {
                diagnostics.error(row, format!("Expected nested group in section `{label}`"));
            }
        }
    }

    items.into()
}

fn parse_required_open_text(
    section: &ProtoSection,
    label: &str,
    diagnostics: &mut DiagnosticTracker,
) -> Option<OpenText> {
    let texts = parse_optional_open_texts(Some(section), diagnostics);
    if texts.is_empty() {
        diagnostics.error(section.metadata.row, format!("Expected text in `{label}`"));
        None
    } else {
        Some(texts.into_iter().next().expect("non-empty texts"))
    }
}

fn parse_required_open_texts(
    section: &ProtoSection,
    label: &str,
    diagnostics: &mut DiagnosticTracker,
) -> Option<OneOrMore<OpenText>> {
    let texts = parse_optional_open_texts(Some(section), diagnostics);
    one_or_more(texts, || {
        diagnostics.error(
            section.metadata.row,
            format!("Expected text entries in `{label}`"),
        );
    })
}

fn parse_optional_open_texts(
    section: Option<&ProtoSection>,
    diagnostics: &mut DiagnosticTracker,
) -> ZeroOrMore<OpenText> {
    parse_optional_texts(section, diagnostics, OpenText)
}

fn parse_required_written_texts(
    section: &ProtoSection,
    diagnostics: &mut DiagnosticTracker,
) -> Option<OneOrMore<WrittenText>> {
    let texts = parse_optional_texts(Some(section), diagnostics, WrittenText);
    one_or_more(texts, || {
        diagnostics.error(section.metadata.row, "Expected written text");
    })
}

fn parse_required_called_texts(
    section: &ProtoSection,
    diagnostics: &mut DiagnosticTracker,
) -> Option<OneOrMore<CalledText>> {
    let texts = parse_optional_texts(Some(section), diagnostics, CalledText);
    one_or_more(texts, || {
        diagnostics.error(section.metadata.row, "Expected called text");
    })
}

fn parse_required_writing_texts(
    section: &ProtoSection,
    diagnostics: &mut DiagnosticTracker,
) -> Option<OneOrMore<WritingText>> {
    let texts = parse_optional_texts(Some(section), diagnostics, WritingText);
    one_or_more(texts, || {
        diagnostics.error(section.metadata.row, "Expected writing text");
    })
}

fn parse_optional_texts<T>(
    section: Option<&ProtoSection>,
    diagnostics: &mut DiagnosticTracker,
    wrap: fn(String) -> T,
) -> ZeroOrMore<T> {
    let Some(section) = section else {
        return ZeroOrMore::default();
    };

    let mut result = Vec::new();
    for entry in section_entries(section) {
        match entry {
            SectionEntry::Inline { text, row } | SectionEntry::Text { text, row } => {
                if let Some(value) = strip_quoted_text(text) {
                    result.push(wrap(value));
                } else {
                    diagnostics.error(row, format!("Expected quoted text, found `{text}`"));
                }
            }
            SectionEntry::Formulation { row, .. } => {
                diagnostics.error(row, "Expected text, found formulation");
            }
            SectionEntry::Group { row, .. } => {
                diagnostics.error(row, "Expected text, found nested group");
            }
        }
    }

    result.into()
}

fn one_or_more<T>(items: ZeroOrMore<T>, on_empty: impl FnOnce()) -> Option<OneOrMore<T>> {
    match OneOrMore::try_from(items) {
        Ok(items) => Some(items),
        Err(_) => {
            on_empty();
            None
        }
    }
}

fn strip_quoted_text(input: &str) -> Option<String> {
    let input = input.trim();
    let inner = input.strip_prefix('"')?.strip_suffix('"')?;
    Some(inner.to_owned())
}

fn first_section_label(group: &ProtoGroup) -> Option<&str> {
    group.sections.first().map(|section| section.label.as_str())
}

fn section<'a>(
    sections: &'a HashMap<String, &'a ProtoSection>,
    label: &str,
) -> Option<&'a ProtoSection> {
    sections.get(label).copied()
}

fn identify_sections<'a>(
    name: &str,
    sections: &'a [ProtoSection],
    diagnostics: &mut DiagnosticTracker,
    expected: &[&str],
) -> Option<HashMap<String, &'a ProtoSection>> {
    let mut section_queue: VecDeque<&ProtoSection> = sections.iter().collect();
    let mut expected_queue: VecDeque<&str> = expected.iter().copied().collect();
    let mut result = HashMap::new();

    let pattern = expected
        .iter()
        .map(|value| format!("{value}:"))
        .collect::<Vec<_>>()
        .join("\n");

    while let (Some(next_section), Some(maybe_name)) = (
        section_queue.front().copied(),
        expected_queue.front().copied(),
    ) {
        let is_optional = maybe_name.ends_with('?');
        let true_name = maybe_name.trim_end_matches('?');

        if next_section.label == true_name {
            result.insert(true_name.to_owned(), next_section);
            section_queue.pop_front();
            expected_queue.pop_front();
        } else if is_optional {
            expected_queue.pop_front();
        } else {
            diagnostics.error(
                next_section.metadata.row,
                format!(
                    "For {name} pattern:\n\n{pattern}\n\nExpected `{true_name}` but found `{}`",
                    next_section.label
                ),
            );
            return None;
        }
    }

    if let Some(unexpected) = section_queue.front() {
        diagnostics.error(
            unexpected.metadata.row,
            format!(
                "For {name} pattern:\n\n{pattern}\n\nUnexpected section `{}`",
                unexpected.label
            ),
        );
        return None;
    }

    if let Some(missing) = expected_queue
        .iter()
        .find(|name| !name.ends_with('?'))
        .copied()
    {
        let row = sections
            .first()
            .map(|section| section.metadata.row)
            .unwrap_or(0);
        diagnostics.error(
            row,
            format!(
                "For {name} pattern:\n\n{pattern}\n\nExpected section `{}`",
                missing.trim_end_matches('?')
            ),
        );
        return None;
    }

    Some(result)
}

enum SectionEntry<'a> {
    Inline { text: &'a str, row: usize },
    Formulation { text: &'a str, row: usize },
    Text { text: &'a str, row: usize },
    Group { group: &'a ProtoGroup, row: usize },
}

fn section_entries(section: &ProtoSection) -> Vec<SectionEntry<'_>> {
    let mut entries = Vec::new();
    if let Some(argument) = section.inline_argument.as_deref() {
        entries.push(SectionEntry::Inline {
            text: argument,
            row: section.metadata.row,
        });
    }

    for argument in &section.arguments {
        match argument {
            ProtoArgument::Formulation(ProtoFormulation { text, metadata }) => {
                entries.push(SectionEntry::Formulation {
                    text,
                    row: metadata.row,
                });
            }
            ProtoArgument::Text(ProtoText { text, metadata }) => {
                entries.push(SectionEntry::Text {
                    text,
                    row: metadata.row,
                });
            }
            ProtoArgument::Group(group) => {
                entries.push(SectionEntry::Group {
                    group,
                    row: group.metadata.row,
                });
            }
        }
    }

    entries
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::parse_document;
    use crate::diagnostics::{Diagnostic, DiagnosticTracker};
    use crate::frontend::proto::Parser as ProtoParser;
    use crate::frontend::structural::ast::{
        AliasItem, AliasKind, Clause, Document, DocumentedItem, IsOrViaItem, JustifiedItem,
        MetadataItem, ProvidesItem, ResourceItem, SpecifyItem, TopLevelItem,
    };

    fn split_golden_entries(text: &str) -> Vec<String> {
        text.replace("\r\n", "\n")
            .trim_matches('\n')
            .split("\n\n\n")
            .filter(|entry| !entry.trim().is_empty())
            .map(str::to_owned)
            .collect()
    }

    fn read_golden_entries(path: &str) -> Vec<String> {
        let text = fs::read_to_string(path).expect("expected structural golden file");
        split_golden_entries(&text)
    }

    fn render_proto_groups(text: &str) -> String {
        let mut diagnostics = DiagnosticTracker::new();
        let groups = {
            let mut parser = ProtoParser::new(text, &mut diagnostics);
            parser.parse()
        };

        assert!(
            !diagnostics.has_errors(),
            "{:#?}",
            diagnostics.diagnostics()
        );

        groups
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn parse_ok(text: &str) -> Document {
        let mut diagnostics = DiagnosticTracker::new();
        let document = parse_document(text, &mut diagnostics);

        assert!(
            !diagnostics.has_errors(),
            "{:#?}",
            diagnostics.diagnostics()
        );

        document
    }

    fn parse_with_diagnostics(text: &str) -> (Document, Vec<Diagnostic>) {
        let mut diagnostics = DiagnosticTracker::new();
        let document = parse_document(text, &mut diagnostics);
        let messages = diagnostics.diagnostics().to_vec();

        (document, messages)
    }

    #[test]
    fn parses_mixed_structural_document() {
        let text = r#"
[\function]
Describes: f(x_)
using:
. x is \type{A}
when:
. x = x
Provides:
. [symbol]
  symbol: f(x_) :=> x
Aliases:
. [alias]
  alias: f(x_) :=> x
References:
. $elements
Metadata:
. id: "desc-1"

[\statement]
States:
that:
. if:
  . x = x
  then:
  . x = x

[@euclid]
Person:
name:
. "Euclid"
biography: "Greek mathematician"

[$elements]
Resource:
. title: "Elements"
"#;

        let mut diagnostics = DiagnosticTracker::new();
        let document = parse_document(text, &mut diagnostics);

        assert!(
            !diagnostics.has_errors(),
            "{:#?}",
            diagnostics.diagnostics()
        );
        assert_eq!(document.items.len(), 4);
        assert!(matches!(document.items[0], TopLevelItem::Describes(_)));
        assert!(matches!(document.items[1], TopLevelItem::States(_)));
        assert!(matches!(document.items[2], TopLevelItem::Person(_)));
        assert!(matches!(document.items[3], TopLevelItem::Resource(_)));
    }

    #[test]
    fn recovers_after_invalid_group() {
        let text = r#"
[\function]
Describes: f(x_)
that:
. x = x

Title: "Valid Title"
"#;

        let mut diagnostics = DiagnosticTracker::new();
        let document = parse_document(text, &mut diagnostics);

        assert!(diagnostics.has_errors());
        assert_eq!(document.items.len(), 1);
        assert!(matches!(document.items[0], TopLevelItem::Title(_)));
    }

    #[test]
    fn parses_clause_groups_as_clauses() {
        let text = r#"
[\property]
States:
that:
. exists: x is \type{A}
  suchThat:
  . x = x
"#;

        let mut diagnostics = DiagnosticTracker::new();
        let document = parse_document(text, &mut diagnostics);

        assert!(
            !diagnostics.has_errors(),
            "{:#?}",
            diagnostics.diagnostics()
        );
        match &document.items[0] {
            TopLevelItem::States(states) => {
                assert!(matches!(states.that.arguments[0], Clause::Exists(_)));
            }
            other => panic!("expected states item, got {other:?}"),
        }
    }

    #[test]
    fn parses_outline_groups() {
        let document = parse_ok(
            r#"
Title: "Foundations"

Section: "Sets"

Subsection: "Membership"

Subsubsection: "Examples"
"#,
        );

        assert_eq!(document.items.len(), 4);

        match &document.items[0] {
            TopLevelItem::Title(group) => assert_eq!(group.title.argument.0, "Foundations"),
            other => panic!("expected title group, got {other:?}"),
        }
        match &document.items[1] {
            TopLevelItem::Section(group) => assert_eq!(group.section.argument.0, "Sets"),
            other => panic!("expected section group, got {other:?}"),
        }
        match &document.items[2] {
            TopLevelItem::Subsection(group) => {
                assert_eq!(group.subsection.argument.0, "Membership")
            }
            other => panic!("expected subsection group, got {other:?}"),
        }
        match &document.items[3] {
            TopLevelItem::Subsubsection(group) => {
                assert_eq!(group.subsubsection.argument.0, "Examples")
            }
            other => panic!("expected subsubsection group, got {other:?}"),
        }
    }

    #[test]
    fn parses_definition_like_groups_with_nested_sections_and_items() {
        let document = parse_ok(
            r#"
[\structure]
Describes: S := (X, *)
using:
. X is \set
. X "contains" Element
when:
. [logic.when]
  allOf:
  . x = x
  . y = y
extends: X is \set via (X, Y)
specifies:
. Y is \set via (X, Y)
. y "contains" Y
satisfies:
. [logic.satisfies]
  not:
  . x = y
Provides:
. [symbol.plus]
  symbol: plus(x_, y_) :=> x + y
  written:
  . "+"
Justified:
. [proof.label]
  label:
  . "Closure"
  by:
  . "Definition"
  comment: "standard"
Documented:
. [docs.written]
  written:
  . "plus"
Aliases:
. [alias.expr]
  alias: plus(x_, y_) :=> x + y
  written:
  . "+"
References:
. $book.plus
Metadata:
. id: "desc-1"

[\structure.connection]
Describes: T
Provides:
. [conn.plus]
  connection:
  . "addition"
  to:
  . "binary operation"
  using:
  . X is \set
  means:
  . "adds elements"
  signifies:
  . "closure"
  viewable:
  . "as a table"
  through:
  . "worked examples"
Justified:
. [proof.by]
  by:
  . "Convention"
  comment: "accepted"
Documented:
. [docs.called]
  called:
  . "addition"
Aliases:
. [alias.spec]
  alias: x_ "in" X :-> x is \element
Metadata:
. version: "1.0"

[\structure.writing]
Describes: W
Documented:
. [docs.writing]
  writing: plus(x_, y_) :~> x + y
  as:
  . "inline notation"

[\structure.overview]
Describes: O
Documented:
. [docs.overview]
  overview: "Binary operation on X"

[\structure.related]
Describes: R
Documented:
. [docs.related]
  related:
  . "group"
  . "ring"

[\structure.discoverer]
Describes: D
Documented:
. [docs.discoverer]
  discoverer:
  . "Gauss"

[\constant]
Defines: zero is \element
using:
. X is \set
expresses:
. [logic.expr]
  piecewise:
  . "choose a representative"
  if:
  . x = x
  then:
  . x = x
  else:
  . y = y

[\transform]
Refines: x is \(f)::[[g]]
using:
. X is \set
when:
. [logic.exists]
  existsUnique: x is \element
  suchThat:
  . x = x
specifies: y is \(f)::[[g]]
satisfies:
. [logic.given]
  given: x is \element
  where:
  . x = x
  then:
  . y = y

[\statement]
States:
. "Closure law"
. "Associativity"
using:
. X is \set
that:
. [logic.exists]
  exists: y is \element
  suchThat:
  . y = y

[\statement.expr]
States:
that:
. y = y
"#,
        );

        assert_eq!(document.items.len(), 10);

        match &document.items[0] {
            TopLevelItem::Describes(group) => {
                assert_eq!(
                    group
                        .using
                        .as_ref()
                        .expect("expected using")
                        .arguments
                        .len(),
                    2
                );
                assert!(matches!(
                    group.when.as_ref().expect("expected when").arguments[0],
                    Clause::AllOf(_)
                ));
                assert!(matches!(
                    group.extends.as_ref().expect("expected extends").argument,
                    IsOrViaItem::IsVia(_)
                ));
                assert_eq!(
                    group
                        .specifies
                        .as_ref()
                        .expect("expected specifies")
                        .arguments
                        .len(),
                    2
                );
                assert!(matches!(
                    group
                        .satisfies
                        .as_ref()
                        .expect("expected satisfies")
                        .arguments[0],
                    Clause::Not(_)
                ));
                assert!(matches!(
                    group
                        .provides
                        .as_ref()
                        .expect("expected provides")
                        .arguments[0],
                    ProvidesItem::Symbol(_)
                ));
                assert!(matches!(
                    group
                        .justified
                        .as_ref()
                        .expect("expected justified")
                        .arguments[0],
                    JustifiedItem::Label(_)
                ));
                assert!(matches!(
                    group
                        .documented
                        .as_ref()
                        .expect("expected documented")
                        .arguments[0],
                    DocumentedItem::Written(_)
                ));
                match &group.aliases.as_ref().expect("expected aliases").arguments[0] {
                    AliasItem::Alias(alias) => {
                        assert!(matches!(alias.alias.argument, AliasKind::Expression(_)))
                    }
                }
                assert!(matches!(
                    group
                        .metadata
                        .as_ref()
                        .expect("expected metadata")
                        .arguments[0],
                    MetadataItem::Id(_)
                ));
            }
            other => panic!("expected describes group, got {other:?}"),
        }

        match &document.items[1] {
            TopLevelItem::Describes(group) => {
                assert!(matches!(
                    group
                        .provides
                        .as_ref()
                        .expect("expected provides")
                        .arguments[0],
                    ProvidesItem::Connection(_)
                ));
                assert!(matches!(
                    group
                        .justified
                        .as_ref()
                        .expect("expected justified")
                        .arguments[0],
                    JustifiedItem::By(_)
                ));
                assert!(matches!(
                    group
                        .documented
                        .as_ref()
                        .expect("expected documented")
                        .arguments[0],
                    DocumentedItem::Called(_)
                ));
                match &group.aliases.as_ref().expect("expected aliases").arguments[0] {
                    AliasItem::Alias(alias) => {
                        assert!(matches!(alias.alias.argument, AliasKind::SpecOperator(_)))
                    }
                }
                assert!(matches!(
                    group
                        .metadata
                        .as_ref()
                        .expect("expected metadata")
                        .arguments[0],
                    MetadataItem::Version(_)
                ));
            }
            other => panic!("expected describes group, got {other:?}"),
        }

        match &document.items[2] {
            TopLevelItem::Describes(group) => {
                assert!(matches!(
                    group
                        .documented
                        .as_ref()
                        .expect("expected documented")
                        .arguments[0],
                    DocumentedItem::Writing(_)
                ));
            }
            other => panic!("expected describes group, got {other:?}"),
        }

        match &document.items[3] {
            TopLevelItem::Describes(group) => {
                assert!(matches!(
                    group
                        .documented
                        .as_ref()
                        .expect("expected documented")
                        .arguments[0],
                    DocumentedItem::Overview(_)
                ));
            }
            other => panic!("expected describes group, got {other:?}"),
        }

        match &document.items[4] {
            TopLevelItem::Describes(group) => {
                assert!(matches!(
                    group
                        .documented
                        .as_ref()
                        .expect("expected documented")
                        .arguments[0],
                    DocumentedItem::Related(_)
                ));
            }
            other => panic!("expected describes group, got {other:?}"),
        }

        match &document.items[5] {
            TopLevelItem::Describes(group) => {
                assert!(matches!(
                    group
                        .documented
                        .as_ref()
                        .expect("expected documented")
                        .arguments[0],
                    DocumentedItem::Discoverer(_)
                ));
            }
            other => panic!("expected describes group, got {other:?}"),
        }

        match &document.items[6] {
            TopLevelItem::Defines(group) => {
                assert!(matches!(
                    group
                        .expresses
                        .as_ref()
                        .expect("expected expresses")
                        .argument,
                    Clause::Piecewise(_)
                ));
            }
            other => panic!("expected defines group, got {other:?}"),
        }

        match &document.items[7] {
            TopLevelItem::Refines(group) => {
                assert!(matches!(
                    group.refines.argument,
                    crate::frontend::formulation::ast::IsOrRefinedStatementSpec::Is(_)
                ));
                assert!(group.specifies.is_some());
                assert!(matches!(
                    group.when.as_ref().expect("expected when").arguments[0],
                    Clause::ExistsUnique(_)
                ));
                assert!(matches!(
                    group
                        .satisfies
                        .as_ref()
                        .expect("expected satisfies")
                        .arguments[0],
                    Clause::Given(_)
                ));
            }
            other => panic!("expected refines group, got {other:?}"),
        }

        match &document.items[8] {
            TopLevelItem::States(group) => {
                assert_eq!(group.states.arguments.len(), 2);
                assert_eq!(group.states.arguments[0].0, "Closure law");
                assert_eq!(group.states.arguments[1].0, "Associativity");
                assert!(matches!(group.that.arguments[0], Clause::Exists(_)));
            }
            other => panic!("expected states group, got {other:?}"),
        }

        match &document.items[9] {
            TopLevelItem::States(group) => {
                assert!(matches!(group.that.arguments[0], Clause::Expression(_)));
            }
            other => panic!("expected states group, got {other:?}"),
        }
    }

    #[test]
    fn parses_theorem_like_groups_and_clause_variants() {
        let document = parse_ok(
            r#"
[\axiom]
Axiom:
. "Every element equals itself"
given:
. X is \set
where:
. [logic.not]
  not:
  . x = y
then:
. [logic.if]
  if:
  . x = x
  then:
  . y = y
iff:
. [logic.iff]
  iff:
  . x = x
  then:
  . y = y
Justified:
. [axiom.justified]
  by:
  . "Definition"
  comment: "classical"
Documented:
. [axiom.written]
  written:
  . "axiom"
Aliases:
. [axiom.alias]
  alias: axiom(x_) :=> x
References:
. $axiom.ref
Metadata:
. id: "ax-1"

Theorem:
then:
. [logic.any]
  anyOf:
  . x = x
  . y = y

[\corollary]
Corollary:
. "Immediate consequence"
of:
. "Previous theorem"
then:
. [logic.one]
  oneOf:
  . x = x
  . y = y

Lemma:
then:
. [logic.forall]
  forAll: x is \element
  where:
  . x = x
  then:
  . x = x

[\conjecture]
Conjecture:
. "Open question"
then:
. [logic.exists]
  exists: x is \element
  suchThat:
  . x = x
"#,
        );

        assert_eq!(document.items.len(), 5);

        match &document.items[0] {
            TopLevelItem::Axiom(group) => {
                assert!(group.heading.is_some());
                assert_eq!(group.axiom.arguments.len(), 1);
                assert!(group.given.is_some());
                assert!(matches!(
                    group.where_.as_ref().expect("expected where").arguments[0],
                    Clause::Not(_)
                ));
                assert!(matches!(group.then.arguments[0], Clause::If(_)));
                assert!(matches!(
                    group.iff.as_ref().expect("expected iff").arguments[0],
                    Clause::Iff(_)
                ));
                assert!(group.justified.is_some());
                assert!(group.documented.is_some());
                assert!(group.aliases.is_some());
                assert!(group.references.is_some());
                assert!(group.metadata.is_some());
            }
            other => panic!("expected axiom group, got {other:?}"),
        }

        match &document.items[1] {
            TopLevelItem::Theorem(group) => {
                assert!(group.heading.is_none());
                assert!(matches!(group.then.arguments[0], Clause::AnyOf(_)));
            }
            other => panic!("expected theorem group, got {other:?}"),
        }

        match &document.items[2] {
            TopLevelItem::Corollary(group) => {
                assert!(group.heading.is_some());
                assert_eq!(group.of.arguments[0].0, "Previous theorem");
                assert!(matches!(group.then.arguments[0], Clause::OneOf(_)));
            }
            other => panic!("expected corollary group, got {other:?}"),
        }

        match &document.items[3] {
            TopLevelItem::Lemma(group) => {
                assert!(group.heading.is_none());
                assert!(matches!(group.then.arguments[0], Clause::ForAll(_)));
            }
            other => panic!("expected lemma group, got {other:?}"),
        }

        match &document.items[4] {
            TopLevelItem::Conjecture(group) => {
                assert!(group.heading.is_some());
                assert!(matches!(group.then.arguments[0], Clause::Exists(_)));
            }
            other => panic!("expected conjecture group, got {other:?}"),
        }
    }

    #[test]
    fn parses_person_resource_and_specify_variants() {
        let document = parse_ok(
            r#"
[@euclid]
Person:
. "Ancient mathematician"
name:
. "Euclid"
. "Euclides"
biography: "Greek mathematician"

[$book.title]
Resource:
. title: "Elements"

[$book.author]
Resource:
. author:
  . "Euclid"
  . "Translator"

[$book.offset]
Resource:
. offset: "Book I"

[$book.url]
Resource:
. url: "https://example.com/elements"

[$book.homepage]
Resource:
. homepage: "https://example.com"

[$book.type]
Resource:
. type: "book"

[$book.edition]
Resource:
. edition: "second"

[$book.editor]
Resource:
. editor: "Editor Name"

[$book.institution]
Resource:
. institution: "Library"

[$book.journal]
Resource:
. journal: "Journal Name"

[$book.publisher]
Resource:
. publisher: "Publisher Name"

[$book.volume]
Resource:
. volume: "I"

[$book.month]
Resource:
. month: "January"

[$book.year]
Resource:
. year: "300BC"

[$book.description]
Resource:
. description: "Classic text"

Specify:
. [numbers.positive.int]
  positive:
  . "positive"
  int:
  . "integer"
  is:
  . "greater than zero"

Specify:
. [numbers.negative.int]
  negative:
  . "negative"
  int:
  . "integer"
  is:
  . "less than zero"

Specify:
. [numbers.zero]
  zero:
  . "zero"
  is:
  . "additive identity"

Specify:
. [numbers.positive.decimal]
  positive:
  . "positive"
  decimal:
  . "decimal"
  is:
  . "greater than zero"

Specify:
. [numbers.negative.decimal]
  negative:
  . "negative"
  decimal:
  . "decimal"
  is:
  . "less than zero"
"#,
        );

        assert_eq!(document.items.len(), 21);

        match &document.items[0] {
            TopLevelItem::Person(group) => {
                assert_eq!(group.person.arguments.len(), 1);
                assert_eq!(group.name.arguments.len(), 2);
                assert_eq!(group.biography.argument.0, "Greek mathematician");
            }
            other => panic!("expected person group, got {other:?}"),
        }

        match &document.items[1] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Title(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[2] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Author(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[3] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Offset(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[4] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(group.resource.arguments[0], ResourceItem::Url(_)));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[5] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Homepage(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[6] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(group.resource.arguments[0], ResourceItem::Type(_)));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[7] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Edition(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[8] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Editor(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[9] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Institution(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[10] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Journal(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[11] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Publisher(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[12] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Volume(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[13] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Month(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[14] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(group.resource.arguments[0], ResourceItem::Year(_)));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[15] {
            TopLevelItem::Resource(group) => {
                assert!(matches!(
                    group.resource.arguments[0],
                    ResourceItem::Description(_)
                ));
            }
            other => panic!("expected resource group, got {other:?}"),
        }

        match &document.items[16] {
            TopLevelItem::Specify(group) => {
                assert!(matches!(
                    group.specify.arguments[0],
                    SpecifyItem::PositiveInt(_)
                ));
            }
            other => panic!("expected specify group, got {other:?}"),
        }

        match &document.items[17] {
            TopLevelItem::Specify(group) => {
                assert!(matches!(
                    group.specify.arguments[0],
                    SpecifyItem::NegativeInt(_)
                ));
            }
            other => panic!("expected specify group, got {other:?}"),
        }

        match &document.items[18] {
            TopLevelItem::Specify(group) => {
                assert!(matches!(group.specify.arguments[0], SpecifyItem::Zero(_)));
            }
            other => panic!("expected specify group, got {other:?}"),
        }

        match &document.items[19] {
            TopLevelItem::Specify(group) => {
                assert!(matches!(
                    group.specify.arguments[0],
                    SpecifyItem::PositiveDecimal(_)
                ));
            }
            other => panic!("expected specify group, got {other:?}"),
        }

        match &document.items[20] {
            TopLevelItem::Specify(group) => {
                assert!(matches!(
                    group.specify.arguments[0],
                    SpecifyItem::NegativeDecimal(_)
                ));
            }
            other => panic!("expected specify group, got {other:?}"),
        }
    }

    #[test]
    fn reports_section_order_errors_and_recovers() {
        let (document, diagnostics) = parse_with_diagnostics(
            r#"
[\statement]
States:
References:
. $bad.ref
that:
. x = x

Section: "Recovered"
"#,
        );

        assert_eq!(document.items.len(), 1);
        assert!(matches!(document.items[0], TopLevelItem::Section(_)));
        assert_eq!(diagnostics.len(), 1);
        assert!(
            diagnostics[0]
                .message
                .contains("Expected `that` but found `References`")
        );
    }

    #[test]
    fn parses_structural_golden_file() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/goldens/structural.golden.txt");
        let entries = read_golden_entries(path);

        assert!(!entries.is_empty(), "expected structural golden entries");

        for (index, entry) in entries.iter().enumerate() {
            let mut diagnostics = DiagnosticTracker::new();
            let parse_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                parse_document(entry, &mut diagnostics)
            }));

            if let Err(payload) = parse_result {
                let message = if let Some(message) = payload.downcast_ref::<&str>() {
                    *message
                } else if let Some(message) = payload.downcast_ref::<String>() {
                    message.as_str()
                } else {
                    "unknown panic"
                };
                panic!(
                    "structural golden case {} panicked: {}\n\n{}",
                    index + 1,
                    message,
                    entry
                );
            }

            assert!(
                !diagnostics.has_errors(),
                "failed to parse structural golden case {}:\n{}\n\n{:#?}",
                index + 1,
                entry,
                diagnostics.diagnostics()
            );

            let rendered = render_proto_groups(entry);
            assert_eq!(
                rendered,
                *entry,
                "structural golden case {} did not round-trip",
                index + 1
            );
        }
    }
}

use super::*;

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

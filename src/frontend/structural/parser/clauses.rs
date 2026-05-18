use super::*;

/// Parses a `not:` clause group.
///
/// The nested `not:` section must contain exactly one clause, which is boxed to
/// avoid making the recursive [`Clause`] enum infinitely sized.
pub(super) fn parse_not_clause(group: &ProtoGroup, tracker: &mut EventLog) -> Option<NotGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("not", &group.sections, tracker, &["not"])?;
    Some(NotGroup {
        heading,
        not: NotSection {
            argument: Box::new(parse_required_clause(
                section(&sections, "not")?,
                "not",
                tracker,
            )?),
        },
    })
}

/// Parses an `allOf:` clause group.
pub(super) fn parse_all_of_clause(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<AllOfGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("allOf", &group.sections, tracker, &["allOf"])?;
    Some(AllOfGroup {
        heading,
        all_of: AllOfSection {
            arguments: parse_required_clauses(section(&sections, "allOf")?, "allOf", tracker)?,
        },
    })
}

/// Parses an `anyOf:` clause group.
pub(super) fn parse_any_of_clause(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<AnyOfGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("anyOf", &group.sections, tracker, &["anyOf"])?;
    Some(AnyOfGroup {
        heading,
        any_of: AnyOfSection {
            arguments: parse_required_clauses(section(&sections, "anyOf")?, "anyOf", tracker)?,
        },
    })
}

/// Parses a `oneOf:` clause group.
pub(super) fn parse_one_of_clause(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<OneOfGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("oneOf", &group.sections, tracker, &["oneOf"])?;
    Some(OneOfGroup {
        heading,
        one_of: OneOfSection {
            arguments: parse_required_clauses(section(&sections, "oneOf")?, "oneOf", tracker)?,
        },
    })
}

/// Parses an `exists:` clause group.
///
/// The bound value is parsed as `is`/spec syntax and the required `suchThat:`
/// section supplies predicate clauses.
pub(super) fn parse_exists_clause(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ExistsGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("exists", &group.sections, tracker, &["exists", "suchThat"])?;
    Some(ExistsGroup {
        heading,
        exists: ExistsSection {
            argument: parse_required_formulation(
                section(&sections, "exists")?,
                "exists",
                tracker,
                parse_is_or_spec,
            )?,
        },
        such_that: SuchThatSection {
            arguments: parse_required_clauses(
                section(&sections, "suchThat")?,
                "suchThat",
                tracker,
            )?,
        },
    })
}

/// Parses an `existsUnique:` clause group.
pub(super) fn parse_exists_unique_clause(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ExistsUniqueGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "existsUnique",
        &group.sections,
        tracker,
        &["existsUnique", "suchThat"],
    )?;
    Some(ExistsUniqueGroup {
        heading,
        exists_unique: ExistsUniqueSection {
            argument: parse_required_formulation(
                section(&sections, "existsUnique")?,
                "existsUnique",
                tracker,
                parse_is_or_spec,
            )?,
        },
        such_that: SuchThatSection {
            arguments: parse_required_clauses(
                section(&sections, "suchThat")?,
                "suchThat",
                tracker,
            )?,
        },
    })
}

/// Parses a `forAll:` clause group.
///
/// The optional `where:` section acts as a guard and the required `then:`
/// section carries the quantified conclusion.
pub(super) fn parse_for_all_clause(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ForAllGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "forAll",
        &group.sections,
        tracker,
        &["forAll", "where?", "then"],
    )?;
    Some(ForAllGroup {
        heading,
        for_all: ForAllSection {
            argument: parse_required_formulation(
                section(&sections, "forAll")?,
                "forAll",
                tracker,
                parse_is_or_spec,
            )?,
        },
        where_: sections.get("where").copied().and_then(|section| {
            parse_required_clauses(section, "where", tracker)
                .map(|arguments| WhereSection { arguments })
        }),
        then: ThenSection {
            arguments: parse_required_clauses(section(&sections, "then")?, "then", tracker)?,
        },
    })
}

/// Parses an `if:` clause group.
pub(super) fn parse_if_clause(group: &ProtoGroup, tracker: &mut EventLog) -> Option<IfGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("if", &group.sections, tracker, &["if", "then"])?;
    Some(IfGroup {
        heading,
        if_: IfSection {
            arguments: parse_required_clauses(section(&sections, "if")?, "if", tracker)?,
        },
        then: ThenSection {
            arguments: parse_required_clauses(section(&sections, "then")?, "then", tracker)?,
        },
    })
}

/// Parses an `iff:` clause group.
pub(super) fn parse_iff_clause(group: &ProtoGroup, tracker: &mut EventLog) -> Option<IffGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections("iff", &group.sections, tracker, &["iff", "then"])?;
    Some(IffGroup {
        heading,
        iff: IffSection {
            arguments: parse_required_clauses(section(&sections, "iff")?, "iff", tracker)?,
        },
        then: ThenSection {
            arguments: parse_required_clauses(section(&sections, "then")?, "then", tracker)?,
        },
    })
}

/// Parses a `piecewise:` clause group.
///
/// The leading section may hold descriptive text while `if:` and `then:` are
/// required and `else:` is optional.
pub(super) fn parse_piecewise_clause(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<PiecewiseGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "piecewise",
        &group.sections,
        tracker,
        &["piecewise", "if", "then", "else?"],
    )?;
    Some(PiecewiseGroup {
        heading,
        piecewise: PiecewiseSection {
            arguments: parse_optional_open_texts(sections.get("piecewise").copied(), tracker),
        },
        if_: IfSection {
            arguments: parse_required_clauses(section(&sections, "if")?, "if", tracker)?,
        },
        then: ThenSection {
            arguments: parse_required_clauses(section(&sections, "then")?, "then", tracker)?,
        },
        else_: sections.get("else").copied().and_then(|section| {
            parse_required_clauses(section, "else", tracker)
                .map(|arguments| ElseSection { arguments })
        }),
    })
}

/// Parses a nested `given:` clause group.
///
/// This group is used inside clause lists to introduce a local assumption,
/// optional context, and required consequence.
pub(super) fn parse_given_clause(group: &ProtoGroup, tracker: &mut EventLog) -> Option<GivenGroup> {
    let heading = parse_optional_label_heading(group, tracker)?;
    let sections = identify_sections(
        "given",
        &group.sections,
        tracker,
        &["given", "where?", "then"],
    )?;
    Some(GivenGroup {
        heading,
        given: GivenClauseSection {
            argument: parse_required_formulation(
                section(&sections, "given")?,
                "given",
                tracker,
                parse_is_or_refined_statement_spec,
            )?,
        },
        where_: sections.get("where").copied().and_then(|section| {
            parse_required_clauses(section, "where", tracker)
                .map(|arguments| WhereSection { arguments })
        }),
        then: ThenSection {
            arguments: parse_required_clauses(section(&sections, "then")?, "then", tracker)?,
        },
    })
}

/// Adapts an `alias:` group into an [`AliasItem`].
pub(super) fn parse_alias_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<AliasItem> {
    parse_alias_group(group, tracker).map(AliasItem::Alias)
}

/// Dispatches nested `Provides:` groups to symbol or connection parsers.
pub(super) fn parse_provides_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ProvidesItem> {
    match first_section_label(group)? {
        "symbol" => parse_symbol(group, tracker).map(ProvidesItem::Symbol),
        "connection" => parse_connection(group, tracker).map(ProvidesItem::Connection),
        other => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Unexpected provides group `{other}`"),
            );
            None
        }
    }
}

/// Dispatches nested `Documented:` groups to documentation item parsers.
///
/// Unknown documentation group labels are reported and skipped so other
/// documentation entries in the same section can still be used.
pub(super) fn parse_documented_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<DocumentedItem> {
    match first_section_label(group)? {
        "written" => parse_written(group, tracker).map(DocumentedItem::Written),
        "called" => parse_called(group, tracker).map(DocumentedItem::Called),
        "writing" => parse_writing(group, tracker).map(DocumentedItem::Writing),
        "overview" => parse_overview(group, tracker).map(DocumentedItem::Overview),
        "related" => parse_related(group, tracker).map(DocumentedItem::Related),
        "discoverer" => parse_discoverer(group, tracker).map(DocumentedItem::Discoverer),
        other => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Unexpected documented group `{other}`"),
            );
            None
        }
    }
}

/// Dispatches nested `Justified:` groups to justification item parsers.
pub(super) fn parse_justified_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<JustifiedItem> {
    match first_section_label(group)? {
        "label" => parse_label_note(group, tracker).map(JustifiedItem::Label),
        "by" => parse_by_note(group, tracker).map(JustifiedItem::By),
        other => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Unexpected justified group `{other}`"),
            );
            None
        }
    }
}

/// Dispatches nested `Metadata:` groups to metadata item parsers.
pub(super) fn parse_metadata_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<MetadataItem> {
    match first_section_label(group)? {
        "id" => parse_id(group, tracker).map(MetadataItem::Id),
        "version" => parse_version(group, tracker).map(MetadataItem::Version),
        other => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Unexpected metadata group `{other}`"),
            );
            None
        }
    }
}

/// Dispatches nested `Specify:` groups to numeric-domain item parsers.
///
/// Positive/negative groups are distinguished by the presence of an `int:`
/// section; otherwise they are parsed as decimal specification items.
pub(super) fn parse_specify_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<SpecifyItem> {
    match first_section_label(group)? {
        "positive" => {
            if group.sections.iter().any(|section| section.label == "int") {
                parse_positive_int(group, tracker).map(SpecifyItem::PositiveInt)
            } else {
                parse_positive_decimal(group, tracker).map(SpecifyItem::PositiveDecimal)
            }
        }
        "negative" => {
            if group.sections.iter().any(|section| section.label == "int") {
                parse_negative_int(group, tracker).map(SpecifyItem::NegativeInt)
            } else {
                parse_negative_decimal(group, tracker).map(SpecifyItem::NegativeDecimal)
            }
        }
        "zero" => parse_zero(group, tracker).map(SpecifyItem::Zero),
        other => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Unexpected specify group `{other}`"),
            );
            None
        }
    }
}

/// Dispatches nested `Resource:` groups to resource field parsers.
pub(super) fn parse_resource_item_group(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<ResourceItem> {
    match first_section_label(group)? {
        "title" => parse_resource_title(group, tracker).map(ResourceItem::Title),
        "author" => parse_resource_author(group, tracker).map(ResourceItem::Author),
        "offset" => parse_resource_offset(group, tracker).map(ResourceItem::Offset),
        "url" => parse_resource_url(group, tracker).map(ResourceItem::Url),
        "homepage" => parse_resource_homepage(group, tracker).map(ResourceItem::Homepage),
        "type" => parse_resource_type(group, tracker).map(ResourceItem::Type),
        "edition" => parse_resource_edition(group, tracker).map(ResourceItem::Edition),
        "editor" => parse_resource_editor(group, tracker).map(ResourceItem::Editor),
        "institution" => parse_resource_institution(group, tracker).map(ResourceItem::Institution),
        "journal" => parse_resource_journal(group, tracker).map(ResourceItem::Journal),
        "publisher" => parse_resource_publisher(group, tracker).map(ResourceItem::Publisher),
        "volume" => parse_resource_volume(group, tracker).map(ResourceItem::Volume),
        "month" => parse_resource_month(group, tracker).map(ResourceItem::Month),
        "year" => parse_resource_year(group, tracker).map(ResourceItem::Year),
        "description" => parse_resource_description(group, tracker).map(ResourceItem::Description),
        other => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Unexpected resource group `{other}`"),
            );
            None
        }
    }
}

/// Dispatches a nested clause group to the corresponding clause parser.
///
/// Inline formulation clauses are handled by [`parse_optional_clauses`]; this
/// function only handles group-shaped clauses.
pub(super) fn parse_clause_group(group: &ProtoGroup, tracker: &mut EventLog) -> Option<Clause> {
    match first_section_label(group)? {
        "not" => parse_not_clause(group, tracker).map(Clause::Not),
        "allOf" => parse_all_of_clause(group, tracker).map(Clause::AllOf),
        "anyOf" => parse_any_of_clause(group, tracker).map(Clause::AnyOf),
        "oneOf" => parse_one_of_clause(group, tracker).map(Clause::OneOf),
        "exists" => parse_exists_clause(group, tracker).map(Clause::Exists),
        "existsUnique" => parse_exists_unique_clause(group, tracker).map(Clause::ExistsUnique),
        "forAll" => parse_for_all_clause(group, tracker).map(Clause::ForAll),
        "if" => parse_if_clause(group, tracker).map(Clause::If),
        "iff" => parse_iff_clause(group, tracker).map(Clause::Iff),
        "piecewise" => parse_piecewise_clause(group, tracker).map(Clause::Piecewise),
        "given" => parse_given_clause(group, tracker).map(Clause::Given),
        other => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Unexpected clause group `{other}`"),
            );
            None
        }
    }
}

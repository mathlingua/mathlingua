/// Dispatches one proto group to the top-level structural parser matching its first section.
///
/// The first section label determines the group kind.  Unknown labels are
/// reported at the group start and omitted from the resulting document.
fn parse_top_level_group(group: &ProtoGroup, tracker: &mut EventLog) -> Option<TopLevelItem> {
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


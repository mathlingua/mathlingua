/// Traverses every command reference that can appear inside a top-level item.
///
/// The visitor receives only signature shapes; it does not need to know the
/// structural context that produced each reference.  This keeps registry
/// validation separate from AST traversal.
fn walk_top_level_item(item: &TopLevelItem, visit: &mut impl FnMut(&SignatureShape)) {
    match item {
        TopLevelItem::Describes(group) => {
            walk_form_or_declaration(&group.describes.argument, visit);
            walk_optional_is_or_specs(&group.using, visit);
            walk_optional_clauses(&group.when, visit);
            if let Some(section) = &group.extends {
                walk_is_or_via_item(&section.argument, visit);
            }
            if let Some(section) = &group.specifies {
                for item in &section.arguments {
                    walk_is_or_via_item(item, visit);
                }
            }
            walk_optional_clauses(&group.satisfies, visit);
            walk_optional_provides(&group.provides, visit);
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::Defines(group) => {
            walk_is_or_spec(&group.defines.argument, visit);
            walk_optional_is_or_specs(&group.using, visit);
            walk_optional_clauses(&group.when, visit);
            if let Some(section) = &group.expresses {
                walk_clause(&section.argument, visit);
            }
            walk_optional_provides(&group.provides, visit);
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::Refines(group) => {
            walk_is_or_refined_spec(&group.refines.argument, visit);
            walk_optional_is_or_specs(&group.using, visit);
            walk_optional_clauses(&group.when, visit);
            if let Some(section) = &group.specifies {
                walk_is_or_refined_spec(&section.argument, visit);
            }
            walk_optional_clauses(&group.satisfies, visit);
            walk_optional_provides(&group.provides, visit);
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::States(group) => {
            walk_optional_is_or_specs(&group.using, visit);
            walk_optional_clauses(&group.when, visit);
            for clause in &group.that.arguments {
                walk_clause(clause, visit);
            }
            walk_optional_provides(&group.provides, visit);
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::Axiom(group) => {
            walk_theorem_like(
                group.given.as_ref(),
                group.where_.as_ref(),
                &group.then,
                group.iff.as_ref(),
                visit,
            );
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::Theorem(group) => {
            walk_theorem_like(
                group.given.as_ref(),
                group.where_.as_ref(),
                &group.then,
                group.iff.as_ref(),
                visit,
            );
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::Corollary(group) => {
            walk_theorem_like(
                group.given.as_ref(),
                group.where_.as_ref(),
                &group.then,
                group.iff.as_ref(),
                visit,
            );
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::Lemma(group) => {
            walk_theorem_like(
                group.given.as_ref(),
                group.where_.as_ref(),
                &group.then,
                group.iff.as_ref(),
                visit,
            );
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::Conjecture(group) => {
            walk_theorem_like(
                group.given.as_ref(),
                group.where_.as_ref(),
                &group.then,
                group.iff.as_ref(),
                visit,
            );
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::Specify(group) => {
            for item in &group.specify.arguments {
                walk_specify_item(item, visit);
            }
        }
        TopLevelItem::Title(_)
        | TopLevelItem::Section(_)
        | TopLevelItem::Subsection(_)
        | TopLevelItem::Subsubsection(_)
        | TopLevelItem::Person(_)
        | TopLevelItem::Resource(_) => {}
    }
}

/// Traverses the common sections shared by theorem-like groups.
///
/// `Axiom`, `Theorem`, `Corollary`, `Lemma`, and `Conjecture` all expose the
/// same logical proof/result sections, so this helper centralizes reference
/// discovery for those group kinds.
fn walk_theorem_like(
    given: Option<&GivenSection>,
    where_: Option<&WhereSection>,
    then: &ThenSection,
    iff: Option<&IffSection>,
    visit: &mut impl FnMut(&SignatureShape),
) {
    if let Some(section) = given {
        for spec in &section.arguments {
            walk_is_or_refined_spec(spec, visit);
        }
    }
    if let Some(section) = where_ {
        for clause in &section.arguments {
            walk_clause(clause, visit);
        }
    }
    for clause in &then.arguments {
        walk_clause(clause, visit);
    }
    if let Some(section) = iff {
        for clause in &section.arguments {
            walk_clause(clause, visit);
        }
    }
}


//! Pulling a top-level item and everything it depends on out of a collection.
//!
//! `mlg extract` and `mlg report` use this to turn "the item with this id" into a
//! self-contained snippet: the requested item plus the transitive closure of the
//! definitions it uses, ordered so that nothing appears before what it depends
//! on. Pasted into a fresh collection, the result checks cleanly whenever the
//! collection it came from did.

use crate::backend::release::ReleaseItem;
use std::collections::HashMap;

/// Why an extraction could not be performed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ExtractError {
    /// No top-level item in the collection carries this `Id:` value.
    UnknownId(String),
}

/// The requested items and everything they transitively depend on, in
/// dependencies-first order.
///
/// Each returned item appears after every definition it uses, so the output can
/// be read as a buildup to the requested item. Items reachable from more than
/// one root appear once, at their first (deepest) position. Dependency edges
/// come from [`ReleaseItem::uses`], which resolves command occurrences through
/// the same signature registry go-to-definition uses.
///
/// A `uses` id with no matching item is skipped rather than reported: the graph
/// is built from a collection that already checked cleanly, so a dangling edge
/// means the owning item simply had no `Id:` to attribute the definition to, and
/// there is nothing to emit for it.
pub(crate) fn extract_items<'a>(
    items: &'a [ReleaseItem],
    ids: &[String],
) -> Result<Vec<&'a ReleaseItem>, ExtractError> {
    let index_by_id = items
        .iter()
        .enumerate()
        .map(|(index, item)| (item.id.as_str(), index))
        .collect::<HashMap<_, _>>();

    let mut visited = vec![false; items.len()];
    let mut ordered = Vec::new();
    for id in ids {
        let Some(&root) = index_by_id.get(id.as_str()) else {
            return Err(ExtractError::UnknownId(id.clone()));
        };
        visit(items, &index_by_id, root, &mut visited, &mut ordered);
    }

    Ok(ordered.into_iter().map(|index| &items[index]).collect())
}

/// Post-order depth-first walk: push `index` only after every dependency it can
/// reach has been pushed. `visited` doubles as the cycle guard, since it is set
/// on entry rather than on push — a cycle among definitions therefore yields
/// each member once instead of recursing forever.
fn visit(
    items: &[ReleaseItem],
    index_by_id: &HashMap<&str, usize>,
    index: usize,
    visited: &mut [bool],
    ordered: &mut Vec<usize>,
) {
    if visited[index] {
        return;
    }
    visited[index] = true;

    for used in &items[index].uses {
        if let Some(&dependency) = index_by_id.get(used.as_str()) {
            visit(items, index_by_id, dependency, visited, ordered);
        }
    }

    ordered.push(index);
}

/// Render extracted items as pasteable MathLingua source: each item's exact
/// source slice, separated by two blank lines.
///
/// Two blank lines is what `mlg format` normalizes to between top-level items,
/// so extracted output is already in canonical form and survives a format pass
/// unchanged. [`ReleaseItem::source`] has had its trailing blank lines trimmed,
/// so joining on `"\n\n\n"` produces exactly that gap.
pub(crate) fn render_extracted_source(items: &[&ReleaseItem]) -> String {
    items
        .iter()
        .map(|item| item.source.as_str())
        .collect::<Vec<_>>()
        .join("\n\n\n")
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn item(id: &str, uses: &[&str]) -> ReleaseItem {
        ReleaseItem {
            id: id.to_string(),
            path: PathBuf::from("content/x.mlg"),
            kind: "Describes".to_string(),
            header: None,
            preview: None,
            source: format!("source of {id}"),
            is_definition: true,
            uses: uses.iter().map(|value| value.to_string()).collect(),
        }
    }

    fn extracted_ids<'a>(items: &'a [ReleaseItem], ids: &[&str]) -> Vec<&'a str> {
        let ids = ids.iter().map(|id| id.to_string()).collect::<Vec<_>>();
        extract_items(items, &ids)
            .expect("ids are present")
            .into_iter()
            .map(|item| item.id.as_str())
            .collect()
    }

    #[test]
    fn extracts_a_lone_item() {
        let items = vec![item("A", &[])];

        assert_eq!(extracted_ids(&items, &["A"]), vec!["A"]);
    }

    #[test]
    fn orders_dependencies_before_the_item_that_uses_them() {
        // A uses B; B uses C.
        let items = vec![item("A", &["B"]), item("B", &["C"]), item("C", &[])];

        assert_eq!(extracted_ids(&items, &["A"]), vec!["C", "B", "A"]);
    }

    #[test]
    fn omits_items_the_request_does_not_reach() {
        let items = vec![item("A", &["B"]), item("B", &[]), item("unrelated", &[])];

        assert_eq!(extracted_ids(&items, &["A"]), vec!["B", "A"]);
    }

    #[test]
    fn includes_a_shared_dependency_once() {
        // A uses {B, C}; B and C both use D.
        let items = vec![
            item("A", &["B", "C"]),
            item("B", &["D"]),
            item("C", &["D"]),
            item("D", &[]),
        ];

        assert_eq!(extracted_ids(&items, &["A"]), vec!["D", "B", "C", "A"]);
    }

    #[test]
    fn extracts_several_roots_without_repeating_shared_dependencies() {
        let items = vec![item("A", &["C"]), item("B", &["C"]), item("C", &[])];

        assert_eq!(extracted_ids(&items, &["A", "B"]), vec!["C", "A", "B"]);
    }

    #[test]
    fn terminates_on_a_dependency_cycle() {
        // Mutually recursive definitions still yield each member exactly once.
        let items = vec![item("A", &["B"]), item("B", &["A"])];

        assert_eq!(extracted_ids(&items, &["A"]), vec!["B", "A"]);
    }

    #[test]
    fn skips_uses_that_name_no_known_item() {
        let items = vec![item("A", &["missing"])];

        assert_eq!(extracted_ids(&items, &["A"]), vec!["A"]);
    }

    #[test]
    fn reports_an_unknown_requested_id() {
        let items = vec![item("A", &[])];
        let ids = vec!["B".to_string()];

        assert_eq!(
            extract_items(&items, &ids),
            Err(ExtractError::UnknownId("B".to_string()))
        );
    }

    #[test]
    fn separates_rendered_items_by_two_blank_lines() {
        // Two blank lines is the gap `mlg format` normalizes to, so extracted
        // output is already canonical.
        let items = vec![item("A", &["B"]), item("B", &[])];
        let extracted = extract_items(&items, &["A".to_string()]).expect("A is present");

        assert_eq!(
            render_extracted_source(&extracted),
            "source of B\n\n\nsource of A"
        );
    }
}

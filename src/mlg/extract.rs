//! `mlg extract` — print a top-level item and everything it depends on.
//!
//! Hidden command, used to turn a problem found while authoring into a
//! reproducible test case: give it the `Id:` of the item that misbehaves and it
//! prints that item plus the transitive closure of the definitions it uses, in
//! dependencies-first order. The output is pasteable into a fresh collection and
//! checks cleanly whenever the collection it came from did; when it did not, the
//! check errors follow the source on stdout so that piping the command to a
//! clipboard captures the problem along with the code.

use crate::backend::collection::{SourceCollection, find_collection_root};
use crate::backend::extract::{ExtractError, extract_items, render_extracted_source};
use crate::backend::release::build_release_items;
use crate::events::{
    Audience, ColorMode, EventConsoleWriter, EventFilter, EventLog, EventLogListener, Level,
};
use crate::mlg::util::no_errors_since;
use std::path::Path;

const ORIGIN: &str = "mlg_extract";

pub struct ExtractResult {
    pub event_log: EventLog,
    pub successful: bool,
}

/// What `mlg extract` found: the pasteable source, plus whatever `mlg check`
/// had to say about the collection it came from.
pub(super) struct Extracted {
    pub source: String,
    /// The collection's check errors, rendered as `mlg check` prints them.
    /// Empty when the collection checks cleanly.
    pub check_errors: Vec<String>,
}

pub fn extract(
    cwd: &Path,
    ids: &[String],
    listener: Option<Box<dyn EventLogListener>>,
) -> ExtractResult {
    let mut event_log = EventLog::new();
    if let Some(listener) = listener {
        event_log.add_boxed_listener(listener);
    }

    let starting_event_count = event_log.events().len();
    if let Some(extracted) = extract_source(cwd, ids, &mut event_log) {
        println!("{}", extracted.source);
        if !extracted.check_errors.is_empty() {
            println!("{}", render_check_errors(&extracted.check_errors));
        }
    }
    let successful = no_errors_since(&event_log, starting_event_count);

    ExtractResult {
        event_log,
        successful,
    }
}

/// The pasteable source for `ids` and their transitive dependencies, or `None`
/// when the collection could not be loaded or does not contain one of the
/// requested ids. Both failures are reported to `event_log`.
///
/// A collection that does not check cleanly is *not* a failure: reproducing a
/// case the checker mishandles is the whole reason to reach for this. The check
/// pass still runs — it is what resolves command uses into dependency edges, and
/// what fills in any missing `Id:` sections — but its diagnostics are returned
/// alongside the source rather than logged, so they neither mix into the report
/// nor fail the command. Each caller decides where they belong.
///
/// `mlg report` shares this so a reported issue always quotes exactly what
/// `mlg extract` would have printed.
pub(super) fn extract_source(
    cwd: &Path,
    ids: &[String],
    event_log: &mut EventLog,
) -> Option<Extracted> {
    let start = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
    let Some(root) = find_collection_root(&start) else {
        event_log.user_error(
            Some(ORIGIN),
            "Could not find an mlg.json; run this inside a Mathlingua collection",
        );
        return None;
    };

    let mut check_log = EventLog::new();
    let mut collection = SourceCollection::load(&root, &mut check_log, ORIGIN);
    collection.run_check_passes(&mut check_log, ORIGIN);
    let check_errors = rendered_check_errors(&check_log, &start);

    let items = build_release_items(collection.parsed_files());
    let extracted = match extract_items(&items, ids) {
        Ok(extracted) => extracted,
        Err(ExtractError::UnknownId(id)) => {
            event_log.user_error(
                Some(ORIGIN),
                format!("No top-level item in the collection has the id \"{id}\""),
            );
            return None;
        }
    };

    Some(Extracted {
        source: render_extracted_source(&extracted),
        check_errors,
    })
}

/// The error-level messages in `check_log`, each rendered the way `mlg check`
/// prints it: a path, a line and column, and the message, relative to `base`.
fn rendered_check_errors(check_log: &EventLog, base: &Path) -> Vec<String> {
    let writer = EventConsoleWriter::new()
        .with_filter(
            EventFilter::new()
                .with_audiences(vec![Audience::User, Audience::System])
                .with_levels(vec![Level::Error]),
        )
        .with_color_mode(ColorMode::Never)
        .with_base_path(base);

    check_log
        .events()
        .iter()
        .filter_map(|event| writer.render_to_string(event))
        .collect()
}

/// The check errors as a trailing section of the extracted output.
///
/// This rides on stdout with the source rather than going to stderr as a
/// warning, so that one `mlg extract <id> | pbcopy` carries both the code and
/// what is wrong with it — enough to hand to someone (or something) that can
/// fix it. It only ever appears when the collection does not check cleanly,
/// which is exactly when the source above it would not have pasted cleanly
/// either.
fn render_check_errors(check_errors: &[String]) -> String {
    let count = check_errors.len();
    let mut rendered = format!(
        "\nThe collection has {count} check {}; the extracted items may be incomplete:\n",
        if count == 1 { "error" } else { "errors" }
    );
    for check_error in check_errors {
        rendered.push('\n');
        rendered.push_str(check_error);
    }

    rendered
}

/// The warning `mlg report` shows when the collection it extracted from does
/// not check cleanly. `mlg extract` prints the errors themselves instead.
pub(super) fn check_error_warning(count: usize) -> String {
    format!(
        "The collection has {count} check {}; the extracted items may be incomplete \
         (run `mlg check` for detail)",
        if count == 1 { "error" } else { "errors" }
    )
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::{check_error_warning, render_check_errors, rendered_check_errors};
    use crate::events::{Audience, Event, EventLocation, EventLog, EventSpan, Level};
    use std::path::Path;

    #[test]
    fn a_single_error_is_counted_in_the_singular() {
        let rendered = render_check_errors(&["a.mlg:1:1: error: broken".to_string()]);

        assert!(rendered.contains("has 1 check error;"));
        assert!(rendered.ends_with("a.mlg:1:1: error: broken"));
    }

    #[test]
    fn every_error_is_listed_not_just_counted() {
        let rendered = render_check_errors(&[
            "a.mlg:1:1: error: first".to_string(),
            "b.mlg:2:1: error: second".to_string(),
        ]);

        assert!(rendered.contains("has 2 check errors;"));
        assert!(rendered.contains("a.mlg:1:1: error: first"));
        assert!(rendered.contains("b.mlg:2:1: error: second"));
    }

    #[test]
    fn errors_are_rendered_the_way_mlg_check_prints_them() {
        let mut check_log = EventLog::new();
        check_log.push(
            Event::message(
                "Undefined command signature `\\set`",
                Level::Error,
                Audience::User,
                Some(EventLocation::file(
                    "/repo/content/sets/example.mlg",
                    Some(EventSpan::row_and_column(3, 6)),
                )),
            )
            .with_origin("mlg_extract"),
        );

        assert_eq!(
            rendered_check_errors(&check_log, Path::new("/repo")),
            ["sets/example.mlg:4:7: error: Undefined command signature `\\set`".to_string()]
        );
    }

    #[test]
    fn warnings_and_logs_are_not_reported_as_errors() {
        let mut check_log = EventLog::new();
        check_log.user_warning(Some("mlg_extract"), "a warning");
        check_log.user_log(Some("mlg_extract"), "Checked 1 file");

        assert!(rendered_check_errors(&check_log, Path::new("/repo")).is_empty());
    }

    #[test]
    fn the_report_warning_points_at_mlg_check_for_the_detail() {
        assert!(check_error_warning(1).contains("1 check error;"));
        assert!(check_error_warning(2).contains("2 check errors;"));
        assert!(check_error_warning(1).contains("run `mlg check` for detail"));
    }
}

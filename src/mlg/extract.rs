//! `mlg extract` — print a top-level item and everything it depends on.
//!
//! Hidden command, used to turn a problem found while authoring into a
//! reproducible test case: give it the `Id:` of the item that misbehaves and it
//! prints that item plus the transitive closure of the definitions it uses, in
//! dependencies-first order. The output is pasteable into a fresh collection and
//! checks cleanly whenever the collection it came from did.

use crate::backend::collection::{SourceCollection, find_collection_root};
use crate::backend::extract::{ExtractError, extract_items, render_extracted_source};
use crate::backend::release::build_release_items;
use crate::events::{EventLog, EventLogListener, Level};
use crate::mlg::util::no_errors_since;
use std::path::Path;

const ORIGIN: &str = "mlg_extract";

pub struct ExtractResult {
    pub event_log: EventLog,
    pub successful: bool,
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
    if let Some(source) = extracted_source(cwd, ids, &mut event_log) {
        println!("{source}");
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
/// what fills in any missing `Id:` sections — but its diagnostics are collected
/// separately so they neither mix into the report nor fail the command. Only a
/// count is surfaced, pointing at `mlg check` for the detail.
///
/// `mlg report` shares this so a reported issue always quotes exactly what
/// `mlg extract` would have printed.
pub(super) fn extracted_source(
    cwd: &Path,
    ids: &[String],
    event_log: &mut EventLog,
) -> Option<String> {
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
    let check_errors = error_count(&check_log);

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

    if check_errors > 0 {
        event_log.user_warning(
            Some(ORIGIN),
            format!(
                "The collection has {check_errors} check {}; \
                 the extracted items may be incomplete (run `mlg check` for detail)",
                if check_errors == 1 { "error" } else { "errors" }
            ),
        );
    }

    Some(render_extracted_source(&extracted))
}

/// The number of error-level messages in `event_log`.
fn error_count(event_log: &EventLog) -> usize {
    event_log
        .events()
        .iter()
        .filter_map(|event| event.as_message())
        .filter(|message| message.level == Level::Error)
        .count()
}

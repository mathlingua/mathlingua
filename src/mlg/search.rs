use crate::backend::collection::SourceCollection;
use crate::backend::view::CollectionView;
use crate::events::{EventLog, EventLogListener};
use crate::mlg::util::no_errors_since;
use serde::Serialize;
use std::path::Path;

const ORIGIN: &str = "mlg_search";

pub struct SearchResult {
    pub event_log: EventLog,
    pub collection_view: Option<CollectionView>,
    pub search_report: Option<SearchReport>,
    pub successful: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchReport {
    pub query: String,
    pub total_matches: usize,
    pub matches: Vec<SearchMatch>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchMatch {
    pub file_path: String,
    pub id: String,
    pub kind: String,
    pub heading: Option<String>,
    pub definition_keys: Vec<String>,
    pub match_reasons: Vec<String>,
    pub snippet: Option<String>,
}

pub fn search(
    cwd: &Path,
    query: &str,
    json: bool,
    listener: Option<Box<dyn EventLogListener>>,
) -> SearchResult {
    let mut event_log = EventLog::new();
    if let Some(listener) = listener {
        event_log.add_boxed_listener(listener);
    }

    let starting_event_count = event_log.events().len();
    let mut collection = SourceCollection::load(cwd, &mut event_log, ORIGIN);
    collection.run_view_check_passes(&mut event_log, ORIGIN);

    let view = collection.build_view(&mut event_log);
    let report = view.as_ref().map(|v| perform_search(v, query));

    if let Some(ref report) = report {
        if !json {
            print_text_search(report);
        }
    }

    let successful = report.is_some() && no_errors_since(&event_log, starting_event_count);

    SearchResult {
        event_log,
        collection_view: view,
        search_report: report,
        successful,
    }
}

pub fn perform_search(view: &CollectionView, query: &str) -> SearchReport {
    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    for file in &view.files {
        for item in &file.items {
            let mut reasons = Vec::new();

            // Check definition keys
            for key in &item.definition_keys {
                if key.to_lowercase().contains(&query_lower) {
                    reasons.push(format!("definition key: {key}"));
                }
            }

            // Check heading
            if let Some(ref heading) = item.heading {
                if heading.to_lowercase().contains(&query_lower) {
                    reasons.push(format!("heading: \"{heading}\""));
                }
            }

            // Check kind
            if item.kind.to_lowercase().contains(&query_lower) {
                reasons.push(format!("kind: {}", item.kind));
            }

            // Check ID
            if item.id.to_lowercase().contains(&query_lower) {
                reasons.push(format!("id: {}", item.id));
            }

            // Check source snippet if not already matched
            let source_matched = item.source.to_lowercase().contains(&query_lower);
            if source_matched && reasons.is_empty() {
                reasons.push("source content".to_string());
            }

            if !reasons.is_empty() {
                let snippet = extract_snippet(&item.source, &query_lower);
                matches.push(SearchMatch {
                    file_path: file.path.clone(),
                    id: item.id.clone(),
                    kind: item.kind.clone(),
                    heading: item.heading.clone(),
                    definition_keys: item.definition_keys.clone(),
                    match_reasons: reasons,
                    snippet,
                });
            }
        }
    }

    let total_matches = matches.len();
    SearchReport {
        query: query.to_string(),
        total_matches,
        matches,
    }
}

fn extract_snippet(source: &str, query_lower: &str) -> Option<String> {
    for line in source.lines() {
        if line.to_lowercase().contains(query_lower) {
            return Some(line.trim().to_string());
        }
    }
    source.lines().next().map(|l| l.trim().to_string())
}

fn print_text_search(report: &SearchReport) {
    println!(
        "Search results for \"{}\" ({} match(es)):",
        report.query, report.total_matches
    );
    if report.matches.is_empty() {
        println!("  No matching items found in collection.");
        return;
    }

    let mut current_file = "";
    for item in &report.matches {
        if item.file_path != current_file {
            current_file = &item.file_path;
            println!("\nIn {}:", current_file);
        }
        let heading = item.heading.as_deref().unwrap_or("<no heading>");
        let keys = if item.definition_keys.is_empty() {
            String::new()
        } else {
            format!(" [keys: {}]", item.definition_keys.join(", "))
        };
        println!("  * [{}] {}{} (Id: {})", item.kind, heading, keys, item.id);
        println!("    Matched by: {}", item.match_reasons.join(", "));
        if let Some(ref snippet) = item.snippet {
            println!("    Snippet: {}", snippet);
        }
    }
}

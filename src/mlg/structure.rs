use crate::backend::collection::SourceCollection;
use crate::backend::view::CollectionView;
use crate::events::{EventLog, EventLogListener};
use crate::mlg::util::no_errors_since;
use serde::Serialize;
use std::path::Path;

const ORIGIN: &str = "mlg_structure";

pub struct StructureResult {
    pub event_log: EventLog,
    pub collection_view: Option<CollectionView>,
    pub structure_report: Option<StructureReport>,
    pub successful: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructureReport {
    pub title: String,
    pub directories: Vec<DirectoryStructure>,
    pub files: Vec<FileStructure>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryStructure {
    pub path: String,
    pub title: Option<String>,
    pub has_preface: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileStructure {
    pub path: String,
    pub title: Option<String>,
    pub defined_commands: Vec<String>,
    pub items: Vec<ItemStructure>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemStructure {
    pub id: String,
    pub kind: String,
    pub heading: Option<String>,
    pub definition_keys: Vec<String>,
    pub commands: Vec<String>,
}

pub fn structure(
    cwd: &Path,
    json: bool,
    listener: Option<Box<dyn EventLogListener>>,
) -> StructureResult {
    let mut event_log = EventLog::new();
    if let Some(listener) = listener {
        event_log.add_boxed_listener(listener);
    }

    let starting_event_count = event_log.events().len();
    let mut collection = SourceCollection::load(cwd, &mut event_log, ORIGIN);
    collection.run_view_check_passes(&mut event_log, ORIGIN);

    let view = collection.build_view(&mut event_log);
    let report = view.as_ref().map(build_structure_report);

    if let Some(ref report) = report {
        if !json {
            print_text_structure(report);
        }
    }

    let successful = report.is_some() && no_errors_since(&event_log, starting_event_count);

    StructureResult {
        event_log,
        collection_view: view,
        structure_report: report,
        successful,
    }
}

pub fn decode_hex_command(hex_str: &str) -> Option<String> {
    if hex_str.len() % 2 != 0 {
        return None;
    }
    let mut bytes = Vec::with_capacity(hex_str.len() / 2);
    for i in (0..hex_str.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex_str[i..i + 2], 16).ok()?;
        bytes.push(byte);
    }
    String::from_utf8(bytes).ok()
}

pub fn build_structure_report(view: &CollectionView) -> StructureReport {
    let directories = view
        .directories
        .iter()
        .map(|d| DirectoryStructure {
            path: d.path.clone(),
            title: d.title.clone(),
            has_preface: !d.preface.is_empty(),
        })
        .collect();

    let files = view
        .files
        .iter()
        .map(|f| {
            let mut defined_commands = Vec::new();
            let items: Vec<ItemStructure> = f
                .items
                .iter()
                .map(|item| {
                    let commands: Vec<String> = item
                        .definition_keys
                        .iter()
                        .filter_map(|k| decode_hex_command(k))
                        .collect();

                    for cmd in &commands {
                        if !defined_commands.contains(cmd) {
                            defined_commands.push(cmd.clone());
                        }
                    }

                    ItemStructure {
                        id: item.id.clone(),
                        kind: item.kind.clone(),
                        heading: item.heading.clone(),
                        definition_keys: item.definition_keys.clone(),
                        commands,
                    }
                })
                .collect();

            FileStructure {
                path: f.path.clone(),
                title: f.title.clone(),
                defined_commands,
                items,
            }
        })
        .collect();

    StructureReport {
        title: view.title.clone(),
        directories,
        files,
    }
}

fn print_text_structure(report: &StructureReport) {
    println!("Collection: {}", report.title);
    if !report.directories.is_empty() {
        println!("\nDirectories:");
        for dir in &report.directories {
            let title = dir.title.as_deref().unwrap_or("<untitled>");
            let preface_indicator = if dir.has_preface { " (has preface)" } else { "" };
            println!("  - {}: {}{}", dir.path, title, preface_indicator);
        }
    }

    if !report.files.is_empty() {
        println!("\nFiles:");
        for file in &report.files {
            let title = file.title.as_deref().unwrap_or("<untitled>");
            println!("  - {} ({}): {} item(s)", file.path, title, file.items.len());
            if !file.defined_commands.is_empty() {
                println!("    Defined Commands: {}", file.defined_commands.join(", "));
            }
            for item in &file.items {
                let heading = item.heading.as_deref().unwrap_or("");
                let keys = if item.commands.is_empty() {
                    String::new()
                } else {
                    format!(" [commands: {}]", item.commands.join(", "))
                };
                let heading_str = if heading.is_empty() { "" } else { " " };
                println!("      * [{}] {}{}{} (Id: {})", item.kind, heading, heading_str, keys, item.id);
            }
        }
    }
}


use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::parse_document;
use crate::events::{Event, EventLog};
use crate::frontend::structural::ast::{
    AliasItem, AliasKind, Clause, Document, DocumentedItem, IsOrViaItem, JustifiedItem,
    MetadataItem, ProvidesItem, ResourceItem, SpecifyItem, TopLevelItem,
};

fn split_test_chunks(text: &str) -> Vec<String> {
    text.replace("\r\n", "\n")
        .split("\n\n")
        .filter_map(|entry| {
            let entry = entry.trim();
            (!entry.is_empty()).then(|| entry.to_owned())
        })
        .collect()
}

fn read_test_chunks(path: &Path) -> Vec<String> {
    let text = fs::read_to_string(path).unwrap_or_else(|error| {
        panic!(
            "expected structural golden file {}: {error}",
            path.display()
        )
    });
    split_test_chunks(&text)
}

fn read_test_files(directory: &Path, extension: &str) -> Vec<PathBuf> {
    let mut files = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("expected directory {}: {error}", directory.display()))
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some(extension))
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .expect("expected valid utf-8 file name")
        .to_owned()
}

fn parse_ok(text: &str) -> Document {
    let mut tracker = EventLog::new();
    let document = parse_document(text, &mut tracker);

    assert!(!tracker.has_errors(), "{:#?}", tracker.events());

    document
}

fn parse_with_diagnostics(text: &str) -> (Document, Vec<Event>) {
    let mut tracker = EventLog::new();
    let document = parse_document(text, &mut tracker);
    let messages = tracker.events().to_vec();

    (document, messages)
}


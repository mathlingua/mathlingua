use crate::backend::semantic::check_documents;
use crate::backend::view::{CollectionView, build_collection_view};
use crate::events::EventLog;
use crate::frontend::{ParsedSourceFile, parse_source_file};
use std::path::PathBuf;

/// A resolved MathLingua file collection as it moves through backend passes.
///
/// Command code owns collection discovery and path resolution. Once source
/// files have been selected, this type becomes the shared handoff point for the
/// structural parser, semantic checker, and optional viewer rendering pass.
pub(crate) struct MlgFileCollection {
    /// Root used for collection-relative output such as viewer file paths.
    root: PathBuf,
    /// Stable list of selected MathLingua source files.
    source_files: Vec<PathBuf>,
    /// Files successfully read and parsed by the structural parser pass.
    parsed_files: Vec<ParsedSourceFile>,
}

impl MlgFileCollection {
    /// Creates a collection from already-resolved source files.
    pub(crate) fn new(root: PathBuf, source_files: Vec<PathBuf>) -> Self {
        Self {
            root,
            source_files,
            parsed_files: Vec::new(),
        }
    }

    /// Returns the selected source files.
    pub(crate) fn source_files(&self) -> &[PathBuf] {
        &self.source_files
    }

    /// Returns the structurally parsed source files.
    pub(crate) fn parsed_files(&self) -> &[ParsedSourceFile] {
        &self.parsed_files
    }

    /// Runs the common checking passes in their intended order.
    pub(crate) fn run_check_passes(&mut self, event_log: &mut EventLog, origin: &str) {
        self.parse_structural(event_log, origin);
        self.check_semantics(event_log);
    }

    /// Reads each source file and parses it into the structural AST.
    fn parse_structural(&mut self, event_log: &mut EventLog, origin: &str) {
        self.parsed_files.clear();

        for file in &self.source_files {
            if let Some(parsed_file) = parse_source_file(file, event_log, origin) {
                self.parsed_files.push(parsed_file);
            }
        }
    }

    /// Runs backend semantic checks across all parsed files.
    fn check_semantics(&self, event_log: &mut EventLog) {
        check_documents(&self.parsed_files, event_log);
    }

    /// Runs the optional view-model generation pass.
    pub(crate) fn build_view(&self, event_log: &mut EventLog) -> Option<CollectionView> {
        build_collection_view(&self.root, &self.parsed_files, event_log)
    }
}

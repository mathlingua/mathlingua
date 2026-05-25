use crate::backend::semantic::{ParsedSourceFile, check_documents};
use crate::backend::view::{CollectionView, build_collection_view};
use crate::events::EventLog;
use crate::frontend::structural::parse_document;
use std::fs;
use std::path::{Path, PathBuf};

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

/// Reads and structurally parses one source file.
///
/// Parser diagnostics are rewritten with the file path so downstream console
/// output can report precise file locations.
fn parse_source_file(
    path: &Path,
    event_log: &mut EventLog,
    origin: &str,
) -> Option<ParsedSourceFile> {
    event_log.system_debug(Some(origin), format!("Parsing {}", path.display()));

    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            event_log.user_error_at_path(
                Some(origin),
                path.to_path_buf(),
                format!("Failed to read file: {error}"),
            );
            return None;
        }
    };

    let mut file_event_log = EventLog::new();
    let document = parse_document(&source, &mut file_event_log);

    for event in file_event_log.events() {
        event_log.push(event.clone().with_file_path(path.to_path_buf()));
    }

    Some(ParsedSourceFile {
        path: path.to_path_buf(),
        source,
        document,
    })
}

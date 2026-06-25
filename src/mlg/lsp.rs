use crate::mlg::check::{check, check_diagnostics_report, CheckDiagnostic};
use lsp_server::{Connection, Message, Notification};
use lsp_types::{
    notification::{
        DidOpenTextDocument, DidSaveTextDocument, Notification as _, PublishDiagnostics,
    },
    Diagnostic, DiagnosticSeverity, InitializeParams, Position, PublishDiagnosticsParams, Range,
    SaveOptions, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextDocumentSyncOptions, TextDocumentSyncSaveOptions, Url,
};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub struct LspResult {
    pub successful: bool,
}

pub fn lsp() -> LspResult {
    let (connection, io_threads) = Connection::stdio();

    let server_capabilities = serde_json::to_value(ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::NONE),
                save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                    include_text: Some(false),
                })),
                ..Default::default()
            },
        )),
        ..Default::default()
    })
    .expect("server capabilities serialize");

    let init_params: Value = match connection.initialize(server_capabilities) {
        Ok(v) => v,
        Err(_) => {
            let _ = io_threads.join();
            return LspResult { successful: false };
        }
    };

    let workspace_root = initial_workspace_root(&init_params);
    let mut state = ServerState::new(workspace_root);

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req).unwrap_or(false) {
                    break;
                }
            }
            Message::Notification(note) => state.handle_notification(&connection, note),
            Message::Response(_) => {}
        }
    }

    let _ = io_threads.join();
    LspResult { successful: true }
}

struct ServerState {
    workspace_root: Option<PathBuf>,
    last_diagnostic_files: HashSet<Url>,
}

impl ServerState {
    fn new(workspace_root: Option<PathBuf>) -> Self {
        Self {
            workspace_root,
            last_diagnostic_files: HashSet::new(),
        }
    }

    fn handle_notification(&mut self, connection: &Connection, note: Notification) {
        let uri = match note.method.as_str() {
            DidOpenTextDocument::METHOD => note
                .params
                .get("textDocument")
                .and_then(|td| td.get("uri"))
                .and_then(|u| u.as_str())
                .and_then(|s| Url::parse(s).ok()),
            DidSaveTextDocument::METHOD => note
                .params
                .get("textDocument")
                .and_then(|td| td.get("uri"))
                .and_then(|u| u.as_str())
                .and_then(|s| Url::parse(s).ok()),
            _ => return,
        };

        let Some(uri) = uri else { return };
        let Ok(file_path) = uri.to_file_path() else { return };

        let root = project_root_for(&file_path, self.workspace_root.as_deref());
        self.refresh_diagnostics(connection, &root);
    }

    fn refresh_diagnostics(&mut self, connection: &Connection, root: &Path) {
        let result = check(root, &[], None);
        let report = check_diagnostics_report(&result, root);

        let mut grouped: HashMap<Url, Vec<Diagnostic>> = HashMap::new();
        for diag in report.diagnostics {
            let Some(uri) = uri_for_diagnostic(&diag, root) else { continue };
            grouped.entry(uri).or_default().push(to_lsp_diagnostic(diag));
        }

        let new_files: HashSet<Url> = grouped.keys().cloned().collect();
        for stale in self.last_diagnostic_files.difference(&new_files) {
            publish(connection, stale.clone(), Vec::new());
        }
        for (uri, diagnostics) in grouped {
            publish(connection, uri, diagnostics);
        }
        self.last_diagnostic_files = new_files;
    }
}

fn publish(connection: &Connection, uri: Url, diagnostics: Vec<Diagnostic>) {
    let params = PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: None,
    };
    let note = Notification {
        method: PublishDiagnostics::METHOD.to_string(),
        params: serde_json::to_value(params).unwrap_or(Value::Null),
    };
    let _ = connection.sender.send(Message::Notification(note));
}

fn initial_workspace_root(init_params: &Value) -> Option<PathBuf> {
    let parsed: InitializeParams = serde_json::from_value(init_params.clone()).ok()?;
    if let Some(folders) = parsed.workspace_folders {
        if let Some(folder) = folders.into_iter().next() {
            if let Ok(p) = folder.uri.to_file_path() {
                return Some(p);
            }
        }
    }
    #[allow(deprecated)]
    if let Some(root_uri) = parsed.root_uri {
        if let Ok(p) = root_uri.to_file_path() {
            return Some(p);
        }
    }
    None
}

fn project_root_for(file: &Path, workspace_root: Option<&Path>) -> PathBuf {
    let start = file.parent().unwrap_or(file);
    let mut cur = Some(start);
    while let Some(dir) = cur {
        if dir.join("mlg.json").exists() {
            return dir.to_path_buf();
        }
        cur = dir.parent();
    }
    workspace_root
        .map(Path::to_path_buf)
        .unwrap_or_else(|| start.to_path_buf())
}

fn uri_for_diagnostic(diag: &CheckDiagnostic, root: &Path) -> Option<Url> {
    let loc = diag.location.as_ref()?;
    let path = if let Some(abs) = loc.absolute_path.as_deref() {
        PathBuf::from(abs)
    } else if let Some(rel) = loc.path.as_deref() {
        root.join(rel)
    } else {
        return None;
    };
    Url::from_file_path(path).ok()
}

fn to_lsp_diagnostic(diag: CheckDiagnostic) -> Diagnostic {
    let severity = match diag.level.as_str() {
        "error" => DiagnosticSeverity::ERROR,
        "warning" => DiagnosticSeverity::WARNING,
        _ => DiagnosticSeverity::INFORMATION,
    };

    let range = diag
        .location
        .as_ref()
        .and_then(|loc| loc.span.as_ref())
        .map(|span| {
            let start = position_from(span.start.line, span.start.column);
            let end = span
                .end
                .as_ref()
                .map(|p| position_from(p.line, p.column))
                .unwrap_or(start);
            Range { start, end }
        })
        .unwrap_or_else(|| Range {
            start: Position::new(0, 0),
            end: Position::new(0, 0),
        });

    Diagnostic {
        range,
        severity: Some(severity),
        source: diag.origin.clone().or_else(|| Some("mlg".to_string())),
        message: diag.message,
        ..Default::default()
    }
}

fn position_from(line: Option<usize>, column: Option<usize>) -> Position {
    // mlg reports 1-based line/column; LSP uses 0-based.
    let line = line.unwrap_or(1).saturating_sub(1) as u32;
    let character = column.unwrap_or(1).saturating_sub(1) as u32;
    Position { line, character }
}

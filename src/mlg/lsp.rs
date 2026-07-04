use crate::mlg::check::{CheckDiagnostic, check, check_diagnostics_report};
use crate::mlg::completion::{
    CandidateKind, CompletionCandidate, Signature, collect_signatures, complete_with_signatures,
};
use lsp_server::{Connection, Message, Notification, Response};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, CompletionResponse,
    CompletionTextEdit, Diagnostic, DiagnosticSeverity, InitializeParams, InsertTextFormat,
    Position, PublishDiagnosticsParams, Range, SaveOptions, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
    TextDocumentSyncSaveOptions, TextEdit, Url,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument,
        Notification as _, PublishDiagnostics,
    },
    request::{Completion, Request as _},
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
                // Full sync so the server always has the current buffer text for
                // completion (diagnostics still only refresh on open/save).
                change: Some(TextDocumentSyncKind::FULL),
                save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                    include_text: Some(false),
                })),
                ..Default::default()
            },
        )),
        completion_provider: Some(CompletionOptions {
            // Pop up command completions as soon as a `\` is typed.
            trigger_characters: Some(vec!["\\".to_string()]),
            ..Default::default()
        }),
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
    let snippets = snippet_support(&init_params);
    let mut state = ServerState::new(workspace_root, snippets);

    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req).unwrap_or(false) {
                    break;
                }
                if req.method == Completion::METHOD {
                    let response = state.handle_completion(req.id.clone(), &req.params);
                    let _ = connection.sender.send(Message::Response(response));
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
    /// Current text of open documents, keyed by URI. Used for completion.
    documents: HashMap<Url, String>,
    /// Whether the client can render completion snippets (tab stops); when it
    /// cannot, command completions fall back to inserting the plain signature.
    snippets: bool,
}

impl ServerState {
    fn new(workspace_root: Option<PathBuf>, snippets: bool) -> Self {
        Self {
            workspace_root,
            last_diagnostic_files: HashSet::new(),
            documents: HashMap::new(),
            snippets,
        }
    }

    fn handle_notification(&mut self, connection: &Connection, note: Notification) {
        // Keep the in-memory document text current for completion.
        match note.method.as_str() {
            DidOpenTextDocument::METHOD => {
                if let (Some(uri), Some(text)) = (
                    note_uri(&note.params),
                    note.params
                        .get("textDocument")
                        .and_then(|td| td.get("text"))
                        .and_then(|t| t.as_str()),
                ) {
                    self.documents.insert(uri, text.to_string());
                }
            }
            DidChangeTextDocument::METHOD => {
                // Full sync: the last content change carries the whole document.
                if let (Some(uri), Some(text)) = (
                    note_uri(&note.params),
                    note.params
                        .get("contentChanges")
                        .and_then(|c| c.as_array())
                        .and_then(|c| c.last())
                        .and_then(|c| c.get("text"))
                        .and_then(|t| t.as_str()),
                ) {
                    self.documents.insert(uri, text.to_string());
                }
                return; // diagnostics refresh on save, not on every edit
            }
            DidCloseTextDocument::METHOD => {
                if let Some(uri) = note_uri(&note.params) {
                    self.documents.remove(&uri);
                }
                return;
            }
            _ => {}
        }

        // Diagnostics refresh happens on open and save.
        let uri = match note.method.as_str() {
            DidOpenTextDocument::METHOD | DidSaveTextDocument::METHOD => note_uri(&note.params),
            _ => return,
        };

        let Some(uri) = uri else { return };
        let Ok(file_path) = uri.to_file_path() else {
            return;
        };

        let root = project_root_for(&file_path, self.workspace_root.as_deref());
        self.refresh_diagnostics(connection, &root);
    }

    fn handle_completion(&self, id: lsp_server::RequestId, params: &Value) -> Response {
        let items = self.completion_items(params).unwrap_or_default();
        let result = serde_json::to_value(CompletionResponse::Array(items))
            .unwrap_or(Value::Null);
        Response {
            id,
            result: Some(result),
            error: None,
        }
    }

    fn completion_items(&self, params: &Value) -> Option<Vec<CompletionItem>> {
        let params: CompletionParams = serde_json::from_value(params.clone()).ok()?;
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let text = self.documents.get(&uri)?;
        let signatures = self.all_signatures();
        let snippets = self.snippets;
        let items = complete_with_signatures(
            text,
            position.line as usize,
            position.character as usize,
            &signatures,
        )
        .into_iter()
        .map(|candidate| completion_item(candidate, position, snippets))
        .collect();
        Some(items)
    }

    /// Command signatures from every open document, deduplicated by text, so a
    /// command declared in one file can be completed while editing another.
    fn all_signatures(&self) -> Vec<Signature> {
        let mut seen = HashSet::new();
        let mut signatures = Vec::new();
        for text in self.documents.values() {
            for signature in collect_signatures(text) {
                if seen.insert(signature.text.clone()) {
                    signatures.push(signature);
                }
            }
        }
        signatures
    }

    fn refresh_diagnostics(&mut self, connection: &Connection, root: &Path) {
        let result = check(root, &[], None);
        let report = check_diagnostics_report(&result, root);

        let mut grouped: HashMap<Url, Vec<Diagnostic>> = HashMap::new();
        for diag in report.diagnostics {
            let Some(uri) = uri_for_diagnostic(&diag, root) else {
                continue;
            };
            grouped
                .entry(uri)
                .or_default()
                .push(to_lsp_diagnostic(diag));
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

/// Extract the `textDocument.uri` from a notification's params.
fn note_uri(params: &Value) -> Option<Url> {
    params
        .get("textDocument")
        .and_then(|td| td.get("uri"))
        .and_then(|u| u.as_str())
        .and_then(|s| Url::parse(s).ok())
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

/// Build an LSP completion item from a candidate. Command candidates carry
/// snippet placeholders and replace the typed `\`-prefix via an explicit edit;
/// section candidates insert at the cursor as before. When the client lacks
/// snippet support, the plain signature (the label) is inserted instead.
fn completion_item(
    candidate: CompletionCandidate,
    position: Position,
    snippets: bool,
) -> CompletionItem {
    let kind = match candidate.kind {
        CandidateKind::Section => CompletionItemKind::KEYWORD,
        CandidateKind::Command => CompletionItemKind::FUNCTION,
    };
    let use_snippet = candidate.snippet && snippets;
    let new_text = if candidate.snippet && !snippets {
        candidate.label.clone()
    } else {
        candidate.insert
    };

    let mut item = CompletionItem {
        kind: Some(kind),
        detail: Some(candidate.detail),
        filter_text: Some(candidate.label.clone()),
        insert_text_format: use_snippet.then_some(InsertTextFormat::SNIPPET),
        label: candidate.label,
        ..Default::default()
    };

    if candidate.replace_chars > 0 {
        let start = Position::new(
            position.line,
            position
                .character
                .saturating_sub(candidate.replace_chars as u32),
        );
        item.text_edit = Some(CompletionTextEdit::Edit(TextEdit {
            range: Range { start, end: position },
            new_text,
        }));
    } else {
        item.insert_text = Some(new_text);
    }

    item
}

/// Whether the client advertised support for completion snippets.
fn snippet_support(init_params: &Value) -> bool {
    let Ok(parsed) = serde_json::from_value::<InitializeParams>(init_params.clone()) else {
        return false;
    };
    parsed
        .capabilities
        .text_document
        .and_then(|text_document| text_document.completion)
        .and_then(|completion| completion.completion_item)
        .and_then(|item| item.snippet_support)
        .unwrap_or(false)
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

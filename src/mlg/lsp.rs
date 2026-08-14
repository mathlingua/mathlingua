use crate::backend::rename::{RenameEdit, RenameError, RenamePreparation, RenameSpan};
use crate::backend::semantic::{DocumentTypeInfo, TypeEntry};
use crate::mlg::check::{CheckDiagnostic, check_collecting_type_info, check_diagnostics_report};
use crate::mlg::completion::{
    CandidateKind, CompletionCandidate, Signature, collect_signatures, complete_with_signatures,
};
use lsp_server::{Connection, ErrorCode, Message, Notification, Response};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, CompletionResponse,
    CompletionTextEdit, Diagnostic, DiagnosticSeverity, GotoDefinitionParams,
    GotoDefinitionResponse, Hover, HoverContents, HoverParams, HoverProviderCapability,
    InitializeParams, InsertTextFormat, Location, MarkupContent, MarkupKind, OneOf, Position,
    PrepareRenameResponse, PublishDiagnosticsParams, Range, RenameOptions, RenameParams,
    SaveOptions, ServerCapabilities, TextDocumentPositionParams, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextDocumentSyncOptions, TextDocumentSyncSaveOptions, TextEdit, Url,
    WorkDoneProgressOptions, WorkspaceEdit,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument,
        Notification as _, PublishDiagnostics,
    },
    request::{
        Completion, GotoDefinition, HoverRequest, PrepareRenameRequest, Rename, Request as _,
    },
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
        // Jump from a `\`-command usage to the top-level item that defines it.
        definition_provider: Some(OneOf::Left(true)),
        // Rename a top-level item's command heading and every use of it. The
        // prepare step restricts renames to heading signatures and seeds the
        // edit box with the current signature.
        rename_provider: Some(OneOf::Right(RenameOptions {
            prepare_provider: Some(true),
            work_done_progress_options: WorkDoneProgressOptions::default(),
        })),
        // Show the type of every expression on a line, as resolved by the last
        // check. Hovering is the only way a Zed extension can surface this — the
        // extension API has no panel of its own.
        hover_provider: Some(HoverProviderCapability::Simple(true)),
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
                } else if req.method == GotoDefinition::METHOD {
                    let response = state.handle_definition(req.id.clone(), &req.params);
                    let _ = connection.sender.send(Message::Response(response));
                } else if req.method == PrepareRenameRequest::METHOD {
                    let response = state.handle_prepare_rename(req.id.clone(), &req.params);
                    let _ = connection.sender.send(Message::Response(response));
                } else if req.method == Rename::METHOD {
                    let response = state.handle_rename(req.id.clone(), &req.params);
                    let _ = connection.sender.send(Message::Response(response));
                } else if req.method == HoverRequest::METHOD {
                    let response = state.handle_hover(req.id.clone(), &req.params);
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
    /// Types resolved per line by the last check of each document. Checking is
    /// expensive, so this is refreshed only on open and save — never on edit —
    /// and served as-is until the next save.
    type_info: HashMap<Url, DocumentTypeInfo>,
    /// Documents edited since their type information was resolved. Their line
    /// numbers may have shifted, so hovers say so rather than pretending to be
    /// current.
    edited_since_check: HashSet<Url>,
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
            type_info: HashMap::new(),
            edited_since_check: HashSet::new(),
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
                    self.edited_since_check.insert(uri.clone());
                    self.documents.insert(uri, text.to_string());
                }
                return; // diagnostics refresh on save, not on every edit
            }
            DidCloseTextDocument::METHOD => {
                if let Some(uri) = note_uri(&note.params) {
                    self.documents.remove(&uri);
                    self.type_info.remove(&uri);
                    self.edited_since_check.remove(&uri);
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
        self.refresh_diagnostics(connection, &root, &uri, &file_path);
    }

    fn handle_completion(&self, id: lsp_server::RequestId, params: &Value) -> Response {
        let items = self.completion_items(params).unwrap_or_default();
        let result = serde_json::to_value(CompletionResponse::Array(items)).unwrap_or(Value::Null);
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

    fn handle_definition(&self, id: lsp_server::RequestId, params: &Value) -> Response {
        let result = self
            .definition_location(params)
            .map(|location| {
                serde_json::to_value(GotoDefinitionResponse::Scalar(location))
                    .unwrap_or(Value::Null)
            })
            .unwrap_or(Value::Null);
        Response {
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Resolve the `\`-command under the cursor to the top-level item that
    /// defines it, searching the whole collection. The current buffer supplies
    /// both the cursor's byte offset and the target file's text, so navigation
    /// reflects unsaved edits.
    fn definition_location(&self, params: &Value) -> Option<Location> {
        let params: GotoDefinitionParams = serde_json::from_value(params.clone()).ok()?;
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let text = self.documents.get(&uri)?;
        let file_path = uri.to_file_path().ok()?;
        let root = project_root_for(&file_path, self.workspace_root.as_deref());
        let offset = byte_offset_at(text, position.line, position.character)?;

        let site = crate::backend::definition::resolve_definition(&root, &file_path, text, offset)?;

        let target_uri = Url::from_file_path(&site.path).ok()?;
        let target = Position::new(site.row as u32, site.column as u32);
        Some(Location {
            uri: target_uri,
            range: Range {
                start: target,
                end: target,
            },
        })
    }

    fn handle_prepare_rename(&self, id: lsp_server::RequestId, params: &Value) -> Response {
        let result = self
            .rename_preparation(params)
            .map(|prep| {
                serde_json::to_value(PrepareRenameResponse::RangeWithPlaceholder {
                    range: span_to_range(&prep.span),
                    placeholder: prep.placeholder,
                })
                .unwrap_or(Value::Null)
            })
            .unwrap_or(Value::Null);
        Response {
            id,
            result: Some(result),
            error: None,
        }
    }

    /// The signature span the cursor rests on, if it is on a top-level item's
    /// command heading; `None` (reported as `null`) means "cannot rename here".
    fn rename_preparation(&self, params: &Value) -> Option<RenamePreparation> {
        let params: TextDocumentPositionParams = serde_json::from_value(params.clone()).ok()?;
        let uri = params.text_document.uri;
        let position = params.position;
        let text = self.documents.get(&uri)?;
        let file_path = uri.to_file_path().ok()?;
        let root = project_root_for(&file_path, self.workspace_root.as_deref());
        let offset = byte_offset_at(text, position.line, position.character)?;
        crate::backend::rename::prepare_rename(&root, &file_path, text, offset)
    }

    fn handle_rename(&self, id: lsp_server::RequestId, params: &Value) -> Response {
        match self.rename_workspace_edit(params) {
            Ok(edit) => Response {
                id,
                result: Some(serde_json::to_value(edit).unwrap_or(Value::Null)),
                error: None,
            },
            // A rejected rename must come back as an error so the editor shows
            // the reason instead of silently applying nothing.
            Err(message) => Response::new_err(id, ErrorCode::InvalidParams as i32, message),
        }
    }

    fn rename_workspace_edit(&self, params: &Value) -> Result<WorkspaceEdit, String> {
        let params: RenameParams =
            serde_json::from_value(params.clone()).map_err(|error| error.to_string())?;
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = params.new_name;
        let text = self.documents.get(&uri).ok_or("The document is not open")?;
        let file_path = uri
            .to_file_path()
            .map_err(|_| "The document has no file path".to_string())?;
        let root = project_root_for(&file_path, self.workspace_root.as_deref());
        let offset = byte_offset_at(text, position.line, position.character)
            .ok_or("The cursor is out of range")?;

        let edits =
            crate::backend::rename::resolve_rename(&root, &file_path, text, offset, &new_name)
                .map_err(rename_error_message)?;

        Ok(workspace_edit_from_edits(edits))
    }

    fn handle_hover(&self, id: lsp_server::RequestId, params: &Value) -> Response {
        let result = self
            .line_types_hover(params)
            .map(|hover| serde_json::to_value(hover).unwrap_or(Value::Null))
            .unwrap_or(Value::Null);
        Response {
            id,
            result: Some(result),
            error: None,
        }
    }

    /// The types of every expression, sub-expression, and statement on the
    /// hovered line, as the last check resolved them.
    fn line_types_hover(&self, params: &Value) -> Option<Hover> {
        let params: HoverParams = serde_json::from_value(params.clone()).ok()?;
        let uri = params.text_document_position_params.text_document.uri;
        let row = params.text_document_position_params.position.line as usize;
        let entries = self.type_info.get(&uri)?.get(&row)?;

        let stale = self.edited_since_check.contains(&uri);
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: render_line_types(row, entries, stale),
            }),
            range: self.line_range(&uri, row),
        })
    }

    /// The full extent of `row` in the open document, so the hover highlights the
    /// whole line its types describe.
    fn line_range(&self, uri: &Url, row: usize) -> Option<Range> {
        let text = self.documents.get(uri)?;
        let line = text.split('\n').nth(row)?;
        Some(Range {
            start: Position::new(row as u32, 0),
            end: Position::new(row as u32, line.chars().count() as u32),
        })
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

    /// Re-checks the collection, republishing diagnostics and refreshing the type
    /// information for `file` — the document that was just opened or saved.
    ///
    /// This is the only place a check runs. Editing a document deliberately does
    /// not trigger one: a check walks the whole collection, and doing that per
    /// keystroke would be unusable.
    fn refresh_diagnostics(
        &mut self,
        connection: &Connection,
        root: &Path,
        uri: &Url,
        file: &Path,
    ) {
        let result = check_collecting_type_info(root, &[], None, Some(file));
        self.type_info.insert(uri.clone(), result.type_info.clone());
        self.edited_since_check.remove(uri);

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

/// The widest expression column a hover will pad to. Beyond this the type moves
/// to its own line rather than pushing the popup off the side of the editor.
const MAX_EXPRESSION_COLUMN: usize = 56;

/// Renders a line's entries as a hover popup: one row per expression, indented
/// under the expression that contains it, with its resolved types alongside.
fn render_line_types(row: usize, entries: &[TypeEntry], stale: bool) -> String {
    // Width of the widest expression that fits; anything longer wraps instead of
    // widening the whole popup to accommodate it.
    let width = entries
        .iter()
        .map(|entry| entry.depth * 2 + entry.text.chars().count())
        .filter(|length| *length <= MAX_EXPRESSION_COLUMN)
        .max()
        .unwrap_or(0);

    let mut body = String::new();
    for entry in entries {
        let indent = entry_indent(entry);
        let label = format!("{indent}{}", entry.text);
        let types = if entry.types.is_empty() {
            "(no type resolved)".to_owned()
        } else {
            entry.types.join(", ")
        };

        if label.chars().count() > width {
            body.push_str(&format!(
                "{label}\n{:width$}    {types}\n",
                "",
                width = width
            ));
        } else {
            let padding = width - label.chars().count();
            body.push_str(&format!("{label}{:padding$}    {types}\n", ""));
        }
    }

    let note = if stale {
        " — this file has been edited since; save to refresh"
    } else {
        ""
    };
    format!(
        "Types on line {} (from the last `mlg check`{note})\n\n```\n{body}```",
        row + 1
    )
}

fn entry_indent(entry: &TypeEntry) -> String {
    "  ".repeat(entry.depth)
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

fn span_to_range(span: &RenameSpan) -> Range {
    Range {
        start: Position::new(span.start_row as u32, span.start_column as u32),
        end: Position::new(span.end_row as u32, span.end_column as u32),
    }
}

/// Group per-file rename edits into a single workspace edit. Files that cannot
/// be expressed as a `file://` URL are dropped.
fn workspace_edit_from_edits(edits: Vec<RenameEdit>) -> WorkspaceEdit {
    let mut changes: HashMap<Url, Vec<TextEdit>> = HashMap::new();
    for edit in edits {
        let Ok(uri) = Url::from_file_path(&edit.path) else {
            continue;
        };
        changes.entry(uri).or_default().push(TextEdit {
            range: span_to_range(&edit.span),
            new_text: edit.new_text,
        });
    }
    WorkspaceEdit {
        changes: Some(changes),
        document_changes: None,
        change_annotations: None,
    }
}

fn rename_error_message(error: RenameError) -> String {
    match error {
        RenameError::NotOnHeading => {
            "Place the cursor on a top-level item's command heading to rename it".to_string()
        }
        RenameError::Unsupported(message)
        | RenameError::InvalidNewName(message)
        | RenameError::ParametersChanged(message) => message,
    }
}

/// Byte offset within `text` of the character at zero-based `line` /
/// `character` (character counted in Unicode scalar values, matching the
/// completion path). A position past a line's end clamps to the line's end; an
/// out-of-range line yields `None`.
fn byte_offset_at(text: &str, line: u32, character: u32) -> Option<usize> {
    let mut offset = 0usize;
    let mut lines = text.split('\n');
    for _ in 0..line {
        offset += lines.next()?.len() + 1; // + the '\n' that `split` consumed
    }
    let current = lines.next()?;
    let column = current
        .char_indices()
        .nth(character as usize)
        .map(|(byte, _)| byte)
        .unwrap_or(current.len());
    Some(offset + column)
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
            range: Range {
                start,
                end: position,
            },
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

#[cfg(test)]
mod tests {
    use super::render_line_types;
    use crate::backend::semantic::TypeEntry;

    fn entry(depth: usize, text: &str, types: &[&str]) -> TypeEntry {
        TypeEntry {
            depth,
            text: text.to_owned(),
            types: types.iter().map(|ty| (*ty).to_owned()).collect(),
        }
    }

    #[test]
    fn renders_sub_expressions_indented_with_their_types_aligned() {
        let entries = vec![
            entry(0, "x = y", &[r"is \\statement"]),
            entry(1, "x", &[r"is \real"]),
            entry(1, "y", &[r"is \real", r#""in" \reals"#]),
        ];

        assert_eq!(
            render_line_types(24, &entries, false),
            concat!(
                "Types on line 25 (from the last `mlg check`)\n",
                "\n",
                "```\n",
                "x = y    is \\\\statement\n",
                "  x      is \\real\n",
                "  y      is \\real, \"in\" \\reals\n",
                "```",
            )
        );
    }

    #[test]
    fn renders_an_expression_the_checker_could_not_type() {
        let entries = vec![entry(0, "f(x)", &[])];

        assert!(render_line_types(0, &entries, false).contains("f(x)    (no type resolved)"));
    }

    /// An expression too wide to align against keeps its own line, and the
    /// narrower entries stay aligned with each other rather than being pushed out
    /// to accommodate it.
    #[test]
    fn wraps_an_over_wide_expression_instead_of_widening_the_popup() {
        let wide = "a".repeat(super::MAX_EXPRESSION_COLUMN + 1);
        let entries = vec![
            entry(0, &wide, &[r"is \real"]),
            entry(1, "a", &[r"is \real"]),
        ];

        assert_eq!(
            render_line_types(0, &entries, false),
            format!(
                "Types on line 1 (from the last `mlg check`)\n\n```\n{wide}\n       is \\real\n  a    is \\real\n```"
            )
        );
    }

    #[test]
    fn says_when_the_types_predate_the_current_edits() {
        let entries = vec![entry(0, "x", &[r"is \real"])];

        assert!(
            render_line_types(0, &entries, true)
                .starts_with("Types on line 1 (from the last `mlg check` — this file has been edited since; save to refresh)")
        );
    }
}

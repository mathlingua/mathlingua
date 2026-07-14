//! Syntax checking of embedded ` ```mlg ` fenced code blocks.
//!
//! Quoted text values (prose, `description:`, `Text:` Markdown, and so on) may
//! embed MathLingua examples in ` ```mlg ` Markdown fences. `mlg check` parses each
//! such fence and reports any *syntax* errors it finds. Semantics are deliberately
//! not checked: the fenced code is only structurally parsed, never type-checked, so
//! an example may freely reference commands that are not defined in the collection.

use std::path::Path;

use crate::events::{Audience, EventLocation, EventLog, Level, MessageEvent};
use crate::frontend::{parse_document, unescape_quoted_text};

/// A ` ```mlg ` fenced code block extracted from a source file.
struct MlgFence {
    /// 0-based file row of the first fenced content line (the line after the
    /// opening ` ```mlg ` marker).
    content_start_row: usize,
    /// The dedented, unescaped MathLingua source inside the fence.
    code: String,
}

/// Reports MathLingua *syntax* errors in every ` ```mlg ` fenced block in `source`.
///
/// Each fence is structurally parsed on its own; the resulting parse diagnostics
/// are re-emitted against `path` at the corresponding source row. No semantic
/// checking is performed.
pub(crate) fn check_text_fence_syntax(
    path: &Path,
    source: &str,
    event_log: &mut EventLog,
    origin: &str,
) {
    for fence in extract_mlg_fences(source) {
        let mut fence_log = EventLog::new();
        parse_document(&fence.code, &mut fence_log);

        for event in fence_log.events() {
            let Some(message) = event.as_message() else {
                continue;
            };
            if message.level != Level::Error || message.audience != Audience::User {
                continue;
            }

            let file_row = fence.content_start_row + message_row(message).unwrap_or(0);
            event_log.user_error_at_file_row(
                Some(origin),
                path.to_path_buf(),
                file_row,
                format!("Syntax error in `mlg` code block: {}", message.message),
            );
        }
    }
}

/// Extracts the 0-based start row of the parse diagnostic within its fence.
fn message_row(message: &MessageEvent) -> Option<usize> {
    let span = match message.location.as_ref()? {
        EventLocation::InMemory { span, .. } | EventLocation::File { span, .. } => span.as_ref()?,
    };
    span.start.row
}

/// Collects every ` ```mlg ` fenced block from a source file.
fn extract_mlg_fences(source: &str) -> Vec<MlgFence> {
    // Split on `\n` (not `lines()`) so row indices match the proto lexer's.
    let lines: Vec<&str> = source.split('\n').collect();
    let mut fences = Vec::new();
    let mut row = 0;

    while row < lines.len() {
        let Some(indent) = mlg_fence_open(lines[row]) else {
            row += 1;
            continue;
        };

        let content_start_row = row + 1;
        let mut end = content_start_row;
        while end < lines.len() && !is_fence_close(lines[end]) {
            end += 1;
        }

        // Dedent by the fence's own indentation (Markdown semantics) and undo the
        // `\"`/`\\` escaping that the enclosing quoted text required.
        let code = lines[content_start_row..end]
            .iter()
            .map(|line| unescape_quoted_text(&dedent(line, indent)))
            .collect::<Vec<_>>()
            .join("\n");
        fences.push(MlgFence {
            content_start_row,
            code,
        });

        // Resume after the closing fence (or at end of input if unterminated).
        row = end + 1;
    }

    fences
}

/// Returns the leading-space indentation of a ` ```mlg ` opening fence, or `None`
/// when `line` does not open one.
fn mlg_fence_open(line: &str) -> Option<usize> {
    let line = line.trim_end_matches('\r');
    let body = line.trim_start_matches(' ');
    let indent = line.len() - body.len();
    let info = body.strip_prefix("```")?.trim_start_matches('`').trim();
    (info == "mlg").then_some(indent)
}

/// Returns whether a line is a closing code fence (only backticks, at least three).
fn is_fence_close(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.len() >= 3 && trimmed.bytes().all(|byte| byte == b'`')
}

/// Removes up to `indent` leading spaces, mirroring how Markdown dedents fenced
/// content by the fence's own indentation.
fn dedent(line: &str, indent: usize) -> String {
    let line = line.trim_end_matches('\r');
    let leading = line.len() - line.trim_start_matches(' ').len();
    line[leading.min(indent)..].to_string()
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::*;

    fn fence_error_messages(source: &str) -> Vec<String> {
        let mut event_log = EventLog::new();
        check_text_fence_syntax(Path::new("test.mlg"), source, &mut event_log, "test");
        event_log
            .events()
            .iter()
            .filter_map(|event| event.as_message())
            .map(|message| message.message.clone())
            .collect()
    }

    #[test]
    fn accepts_syntactically_valid_fenced_mathlingua() {
        // The example references `\function:on:to`, which is not defined anywhere;
        // that is a semantic concern and must not be reported here.
        let source = "Text: \"Example:\n```mlg\n[\\function:on{A}:to{B}]\nDescribes: f(x__) ::= y_\nDocumented:\n. called: \\\"function\\\"\n```\n\"\n";
        assert!(
            fence_error_messages(source).is_empty(),
            "valid fenced code should produce no diagnostics"
        );
    }

    #[test]
    fn reports_syntax_error_in_fenced_mathlingua() {
        // `Bogus:` is not a valid top-level item, so the structural parser rejects it.
        let source = "Text: \"Example:\n```mlg\nBogus: \\\"nope\\\"\n```\n\"\n";
        let messages = fence_error_messages(source);
        assert!(
            messages
                .iter()
                .any(|message| message.contains("mlg` code block")),
            "expected a fenced-code syntax error, got: {messages:?}"
        );
    }

    #[test]
    fn ignores_non_mlg_fences() {
        let source = "Text: \"Example:\n```rust\nBogus nonsense that is not mlg\n```\n\"\n";
        assert!(
            fence_error_messages(source).is_empty(),
            "non-`mlg` fences must not be parsed as MathLingua"
        );
    }

    #[test]
    fn ignores_mlg_fragment_fences() {
        // `mlg-fragment` blocks are highlighted but never checked — even a bare
        // formulation (not a valid standalone item) must not be reported.
        let source = "Text: \"Example:\n```mlg-fragment\nf(x) := y\n```\n\"\n";
        assert!(
            fence_error_messages(source).is_empty(),
            "mlg-fragment fences must not be syntax-checked"
        );
    }

    #[test]
    fn dedents_indented_fences() {
        // A fence indented four spaces still parses once dedented.
        let source = "    Text: \"\n    ```mlg\n    [\\real]\n    Describes: X\n    ```\n    \"\n";
        assert!(
            fence_error_messages(source).is_empty(),
            "indented fenced code should dedent and parse cleanly"
        );
    }
}

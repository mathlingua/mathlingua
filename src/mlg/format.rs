use crate::backend::collection::{collection_source_files, find_collection_root};
use crate::backend::config::{
    CONFIG_FILE, Config, DEFAULT_MARGIN, legacy_margin_field_message, uses_legacy_margin_field,
};
use crate::events::{EventLog, EventLogListener};
use crate::frontend::{ProtoArgument, ProtoGroup, ProtoParser, ProtoSection};
use crate::mlg::util::no_errors_since;
use std::fs;
use std::io;
use std::path::Path;

const ORIGIN: &str = "mlg_format";

pub struct FormatResult {
    pub event_log: EventLog,
    pub successful: bool,
}

/// Normalize `.mlg` source formatting for the collection rooted at (or above)
/// `cwd`: ensure exactly two blank lines between top-level items, and reflow inline
/// `"..."` text values to the configured print margin.
pub fn format(cwd: &Path, listener: Option<Box<dyn EventLogListener>>) -> FormatResult {
    let mut event_log = EventLog::new();
    if let Some(listener) = listener {
        event_log.add_boxed_listener(listener);
    }

    let starting_event_count = event_log.events().len();
    let io_ok = format_in(cwd, &mut event_log).is_ok();
    let successful = io_ok && no_errors_since(&event_log, starting_event_count);

    FormatResult {
        event_log,
        successful,
    }
}

fn format_in(cwd: &Path, event_log: &mut EventLog) -> io::Result<()> {
    let start = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
    let Some(root) = find_collection_root(&start) else {
        event_log.user_error(
            Some(ORIGIN),
            "Could not find an mlg.json; run `mlg format` inside a Mathlingua collection",
        );
        return Err(io::Error::other("no collection root"));
    };

    let Some(formatted) = format_collection(&root, event_log, ORIGIN) else {
        return Err(io::Error::other("stale margin field"));
    };

    event_log.user_log(
        Some(ORIGIN),
        match formatted {
            0 => "Nothing to format".to_string(),
            1 => "Formatted 1 file".to_string(),
            n => format!("Formatted {n} files"),
        },
    );
    Ok(())
}

/// Format every source file of the collection rooted at `root`, returning how
/// many files were rewritten. `None` means the config still uses the pre-rename
/// `print_margin` key, so nothing was formatted.
///
/// The count is returned rather than logged so that each caller words its own
/// summary — `mlg format` always reports, while `mlg check` stays quiet unless
/// it actually changed something.
pub(super) fn format_collection(
    root: &Path,
    event_log: &mut EventLog,
    origin: &str,
) -> Option<usize> {
    // A stale `print_margin` aborts rather than falling back: formatting every
    // file to the default width would rewrap exactly the files the author set a
    // narrower margin for.
    let margin = load_margin(root, event_log, origin)?;
    let files = collection_source_files(root, event_log, origin);
    let mut formatted = 0usize;

    for file in files {
        let Ok(source) = fs::read_to_string(&file) else {
            continue;
        };
        if let Some(updated) = format_source(&source, margin) {
            if let Err(error) = fs::write(&file, updated) {
                event_log.user_error_at_path(
                    Some(origin),
                    file.clone(),
                    format!("Failed to write formatted source: {error}"),
                );
                continue;
            }
            formatted += 1;
        }
    }

    Some(formatted)
}

/// The `margin` from `mlg.json`, or the default when unset/unreadable.
///
/// `None` means the config still uses the pre-rename `print_margin` key, which
/// is reported rather than ignored: silently formatting to the default width
/// would rewrap every file the author had set a narrower margin for.
fn load_margin(root: &Path, event_log: &mut EventLog, origin: &str) -> Option<usize> {
    let path = root.join(CONFIG_FILE);
    let Ok(contents) = fs::read_to_string(&path) else {
        return Some(DEFAULT_MARGIN);
    };

    if uses_legacy_margin_field(&contents) {
        event_log.user_error_at_path(Some(origin), path, legacy_margin_field_message());
        return None;
    }

    Some(serde_json::from_str::<Config>(&contents).map_or(DEFAULT_MARGIN, |config| config.margin()))
}

/// Applies the formatting rules to a single file's source, returning the rewritten
/// source when anything changed.
fn format_source(source: &str, margin: usize) -> Option<String> {
    let mut lines: Vec<String> = source.split('\n').map(str::to_owned).collect();

    let mut event_log = EventLog::new();
    let groups = ProtoParser::new(source, &mut event_log).parse();

    // Collect all edits as (start_row, end_row_inclusive, replacement_lines). Text
    // reflows and blank-line normalization never overlap (one edits an item's
    // interior, the other the blank gap between items), so they compose cleanly.
    let mut edits: Vec<(usize, usize, Vec<String>)> = Vec::new();
    for group in &groups {
        collect_text_reflow_edits(group, &lines, margin, &mut edits);
        collect_item_boundary_edits(group, &lines, &mut edits);
    }
    collect_blank_line_edits(&groups, &lines, &mut edits);

    if edits.is_empty() {
        return None;
    }

    // Apply bottom-up so earlier indices stay valid.
    edits.sort_by(|a, b| b.0.cmp(&a.0));
    let mut changed = false;
    for (start, end, replacement) in edits {
        if end >= lines.len() || start > end {
            continue;
        }
        if lines[start..=end] != replacement[..] {
            changed = true;
        }
        lines.splice(start..=end, replacement);
    }

    changed.then(|| lines.join("\n"))
}

/// Records reflow edits for every inline text value in a group (recursing into
/// nested groups).
fn collect_text_reflow_edits(
    group: &ProtoGroup,
    lines: &[String],
    margin: usize,
    edits: &mut Vec<(usize, usize, Vec<String>)>,
) {
    for section in &group.sections {
        collect_section_text_reflow_edits(section, lines, margin, edits);
    }
}

fn collect_section_text_reflow_edits(
    section: &ProtoSection,
    lines: &[String],
    margin: usize,
    edits: &mut Vec<(usize, usize, Vec<String>)>,
) {
    if let Some(edit) = section_inline_text_edit(section, lines, margin) {
        edits.push(edit);
    }

    for argument in &section.arguments {
        if let ProtoArgument::Group(group) = argument {
            collect_text_reflow_edits(group, lines, margin, edits);
        }
    }
}

/// The reflow edit for a section whose inline argument is a text value, if any.
fn section_inline_text_edit(
    section: &ProtoSection,
    lines: &[String],
    margin: usize,
) -> Option<(usize, usize, Vec<String>)> {
    let argument = section.inline_argument.as_deref()?;
    if !argument.starts_with('"') {
        return None;
    }
    reflow_inline_text(section.metadata.row, lines, margin)
}

/// Builds the reflow edit for the text value that begins on `row` (either inline —
/// content on the `label: "` line — or block form, content on the following lines),
/// reflowing both into the canonical inline form. `None` if malformed.
fn reflow_inline_text(
    row: usize,
    lines: &[String],
    margin: usize,
) -> Option<(usize, usize, Vec<String>)> {
    let open_line = lines.get(row)?;
    // The opening quote is the first quote on the line (labels never contain quotes).
    let quote = open_line.find('"')?;
    let is_triple = open_line[quote..].starts_with("\"\"\"");
    let delim_len = if is_triple { 3 } else { 1 };
    let content_col = quote + delim_len;

    // Find the closing line (the first line from `row` that ends with an unescaped
    // `"`, or `"""` if triple-quoted, considering the opening quote itself for a
    // single-line value).
    let close_row = (row..lines.len()).find(|&index| {
        let line = &lines[index];
        let end = if index == row { content_col } else { 0 };
        let slice = &line[end.min(line.len())..];
        if is_triple {
            line_closes_triple_quoted_text(slice)
        } else {
            closes_quoted_text(slice)
        }
    })?;

    // Assemble the raw content lines (between the quotes).
    let mut content_lines: Vec<&str> = Vec::new();
    for (index, line) in lines[row..=close_row].iter().enumerate() {
        let index = row + index;
        let from = if index == row { content_col } else { 0 };
        let mut slice = &line[from.min(line.len())..];
        if index == close_row {
            if is_triple {
                let trimmed = slice.trim_end();
                slice = trimmed.strip_suffix("\"\"\"").unwrap_or(trimmed);
            } else {
                slice = slice.strip_suffix('"').unwrap_or(slice);
            }
        }
        content_lines.push(slice);
    }

    let first_prefix = &open_line[..content_col];
    let replacement = reflow_text(&content_lines, first_prefix, content_col, margin, is_triple)?;
    Some((row, close_row, replacement))
}

/// One unit of a text value for reflow: a whitespace-delimited word (with math
/// blobs kept whole, so their internal spaces don't split it), a paragraph break,
/// a Markdown code fence, or a Markdown list item. A `Fence` carries its lines
/// dedented to the fence's own base indentation, so the reflow re-indents them to the
/// value's content column while preserving the relative layout the author gave the
/// fenced content. A `ListItem` carries its nesting (extra indent past the content
/// column, so nested lists keep their level), its normalized marker (e.g. `* ` or
/// `1. `), and its content words, so the reflow wraps the item to the margin with
/// continuation lines hanging-indented past the marker.
enum Piece {
    Word(String),
    Paragraph,
    Fence(Vec<String>),
    ListItem {
        nesting: usize,
        marker: String,
        words: Vec<String>,
    },
}

/// Reflows text content into lines that fit within `margin`, treating math blobs
/// (`$…$`, `$$…$$`, `\(…\)`, `\[…\]`, `{. … .}`, `{{. … .}}`) as atomic tokens that
/// are never split or modified. The first line keeps the verbatim `first_prefix`
/// (`…label: "`); every other line is indented to `content_col`; a blank line
/// separates paragraphs; the closing `"` is appended to the final line.
///
/// Markdown code fences (```` ``` ````, with or without an info string) are emitted
/// verbatim on their own lines — their internal spacing and line breaks are never
/// wrapped or altered — while the prose around them is still reflowed to `margin`.
/// Markdown list items keep their own line and marker, wrapping under a hanging
/// indent so their structure is preserved.
///
/// Returns `None` (leave the value unchanged) when a math blob spans multiple
/// lines or is too wide to fit on a line — the author has laid such content out by
/// hand and it must not be reflowed.
fn reflow_text(
    content_lines: &[&str],
    first_prefix: &str,
    content_col: usize,
    margin: usize,
    is_triple: bool,
) -> Option<Vec<String>> {
    let content = content_lines.join("\n");
    let pieces = tokenize_reflow_pieces(&content, content_col);
    let available = margin.saturating_sub(content_col);

    for piece in &pieces {
        let words: &[String] = match piece {
            Piece::Word(word) => std::slice::from_ref(word),
            Piece::ListItem { words, .. } => words,
            Piece::Paragraph | Piece::Fence(_) => continue,
        };
        for word in words {
            let has_math = word_contains_math(word);
            // A multi-line math blob, or a blob too wide to sit on a line, means the
            // author has already laid this out — leave the whole value untouched.
            if word.contains('\n') || (has_math && word.chars().count() > available) {
                return None;
            }
        }
    }

    let indent = " ".repeat(content_col);
    let mut out: Vec<String> = Vec::new();
    let mut current = first_prefix.to_string();
    let mut has_word = false;
    // Whether `current` holds a prose line not yet flushed into `out`. A `Paragraph`
    // or `Fence` closes it; the next word opens a fresh, indented line.
    let mut open_line = true;

    for piece in pieces {
        match piece {
            Piece::Paragraph => {
                if open_line {
                    out.push(std::mem::take(&mut current));
                }
                out.push(String::new());
                has_word = false;
                open_line = false;
            }
            Piece::Word(word) => {
                // The first word after a paragraph break or fence opens a fresh line.
                if !open_line {
                    current = indent.clone();
                    has_word = false;
                    open_line = true;
                }
                let word_len = word.chars().count();
                if has_word && current.chars().count() + 1 + word_len > margin {
                    out.push(std::mem::take(&mut current));
                    current = indent.clone();
                    current.push_str(&word);
                } else {
                    if has_word {
                        current.push(' ');
                    }
                    current.push_str(&word);
                }
                has_word = true;
            }
            Piece::Fence(fence_lines) => {
                // Flush the open prose line (the last wrapped line, or the opening
                // prefix when the value starts with a fence). A fence right after a
                // paragraph break must not emit a stray indent-only line, so an open
                // line with no word is dropped unless it is the very first line.
                if open_line && (has_word || out.is_empty()) {
                    out.push(std::mem::take(&mut current));
                }
                for line in fence_lines {
                    if line.is_empty() {
                        out.push(String::new());
                    } else {
                        out.push(format!("{indent}{line}"));
                    }
                }
                current = String::new();
                has_word = false;
                open_line = false;
            }
            Piece::ListItem {
                nesting,
                marker,
                words,
            } => {
                // Flush the open prose line (or opening prefix), then wrap the item:
                // the first line carries the marker at the content column (plus any
                // nesting); wrapped continuation lines hang-indent past the marker.
                if open_line && (has_word || out.is_empty()) {
                    out.push(std::mem::take(&mut current));
                }
                let item_indent = " ".repeat(content_col + nesting);
                let hang = " ".repeat(content_col + nesting + marker.chars().count());
                let mut line = format!("{item_indent}{marker}");
                let mut item_has_word = false;
                for word in words {
                    let word_len = word.chars().count();
                    if item_has_word && line.chars().count() + 1 + word_len > margin {
                        out.push(std::mem::take(&mut line));
                        line = hang.clone();
                        line.push_str(&word);
                    } else {
                        if item_has_word {
                            line.push(' ');
                        }
                        line.push_str(&word);
                    }
                    item_has_word = true;
                }
                // A bare marker (no content) must not leave trailing whitespace.
                if !item_has_word {
                    line = format!("{item_indent}{}", marker.trim_end());
                }
                out.push(line);
                current = String::new();
                has_word = false;
                open_line = false;
            }
        }
    }
    if open_line {
        out.push(current);
    }

    if let Some(last) = out.last_mut() {
        if is_triple {
            last.push_str("\"\"\"");
        } else {
            last.push('"');
        }
    }
    Some(out)
}

/// Splits text content into pieces, working one line at a time so a Markdown code
/// fence can be lifted out as a verbatim block. The content is a sequence of blocks —
/// prose paragraphs and code fences — separated by blank lines; each blank-line gap
/// between two blocks becomes a `Paragraph` piece (leading and trailing gaps are
/// dropped). A prose paragraph is tokenized into `Word` pieces (math blobs kept
/// whole); a fence becomes a verbatim `Fence` piece dedented to its own indentation.
///
/// This mirrors the "split into chunks between fences, reflow each chunk" approach:
/// prose reflows to the margin while the fenced blocks pass through untouched.
fn tokenize_reflow_pieces(content: &str, content_col: usize) -> Vec<Piece> {
    let lines: Vec<&str> = content.split('\n').collect();
    let mut pieces: Vec<Piece> = Vec::new();
    let mut row = 0;
    // A blank-line gap has been seen; emit a `Paragraph` separator before the next
    // block. Gaps before the first block and after the last one are simply ignored.
    let mut pending_separator = false;

    while row < lines.len() {
        if lines[row].trim().is_empty() {
            pending_separator = !pieces.is_empty();
            row += 1;
            continue;
        }

        if pending_separator {
            pieces.push(Piece::Paragraph);
            pending_separator = false;
        }

        if let Some((base_indent, open_ticks)) = fence_open(lines[row]) {
            // Capture the fence verbatim through its closing line (or to the end if it
            // is never closed), dedented by its own indentation so its relative layout
            // survives re-indentation to the content column.
            let mut fence = vec![dedent(lines[row], base_indent)];
            row += 1;
            while row < lines.len() {
                let line = lines[row];
                fence.push(dedent(line, base_indent));
                row += 1;
                if is_closing_fence(line, open_ticks) {
                    break;
                }
            }
            pieces.push(Piece::Fence(fence));
        } else {
            // A run of consecutive non-blank, non-fence lines. When it contains a
            // Markdown list item its structure must survive (each item on its own
            // line); otherwise it is one prose paragraph that reflows freely.
            let start = row;
            while row < lines.len()
                && !lines[row].trim().is_empty()
                && fence_open(lines[row]).is_none()
            {
                row += 1;
            }
            let run = &lines[start..row];
            if run.iter().any(|line| split_list_marker(line).is_some()) {
                tokenize_list_run(run, content_col, &mut pieces);
            } else {
                tokenize_prose_words(&run.join("\n"), &mut pieces);
            }
        }
    }

    pieces
}

/// Tokenizes a run of lines that contains at least one Markdown list item. Any prose
/// before the first marker becomes `Word` pieces (a lead-in paragraph); then each
/// marker line, together with the non-marker lines that continue it, becomes one
/// `ListItem` piece whose words the reflow re-wraps under a hanging indent.
fn tokenize_list_run(run: &[&str], content_col: usize, pieces: &mut Vec<Piece>) {
    let first_marker = run
        .iter()
        .position(|line| split_list_marker(line).is_some())
        .unwrap_or(run.len());
    if first_marker > 0 {
        tokenize_prose_words(&run[..first_marker].join("\n"), pieces);
    }

    let mut row = first_marker;
    while row < run.len() {
        let (marker, first) = split_list_marker(run[row]).expect("run[row] opens a list item");
        // Indentation past the content column marks a nested item; preserve it so the
        // list level survives. (A list item on the value's first line has no leading
        // indent, so it clamps to the content column.)
        let leading = run[row].len() - run[row].trim_start_matches(' ').len();
        let nesting = leading.saturating_sub(content_col);
        let mut content = first.to_string();
        row += 1;
        // Non-marker lines lazily continue the current item.
        while row < run.len() && split_list_marker(run[row]).is_none() {
            content.push('\n');
            content.push_str(run[row]);
            row += 1;
        }
        pieces.push(Piece::ListItem {
            nesting,
            marker,
            words: tokenize_words(&content),
        });
    }
}

/// If `line` opens a Markdown list item, returns its normalized marker (the bullet or
/// number followed by a single space, e.g. `* ` or `10. `) and the content after the
/// marker. Recognizes unordered (`*`/`-`/`+`) and ordered (`N.`/`N)`) markers.
fn split_list_marker(line: &str) -> Option<(String, &str)> {
    let trimmed = line.trim_start();
    let first = trimmed.chars().next()?;

    if matches!(first, '*' | '-' | '+') {
        let rest = &trimmed[first.len_utf8()..];
        let content = rest.strip_prefix([' ', '\t'])?;
        return Some((format!("{first} "), content));
    }

    if first.is_ascii_digit() {
        let digits_len = trimmed.chars().take_while(char::is_ascii_digit).count();
        let after = &trimmed[digits_len..];
        let delim = after.chars().next()?;
        if matches!(delim, '.' | ')') {
            let content = after[delim.len_utf8()..].strip_prefix([' ', '\t'])?;
            return Some((format!("{}{delim} ", &trimmed[..digits_len]), content));
        }
    }

    None
}

/// Splits `content` into words, keeping each math blob whole so its internal
/// whitespace and newlines do not split it — a multi-line blob thus yields a word
/// containing `\n`, which the caller uses to detect hand-laid-out content and leave
/// the whole value unchanged.
fn tokenize_words(content: &str) -> Vec<String> {
    let chars: Vec<char> = content.chars().collect();
    let count = chars.len();
    let mut index = 0;
    let mut words = Vec::new();

    while index < count {
        if chars[index].is_whitespace() {
            index += 1;
            continue;
        }

        let mut word = String::new();
        while index < count {
            if let Some(end) = math_blob_end(&chars, index) {
                word.extend(&chars[index..end]);
                index = end;
                continue;
            }
            if chars[index].is_whitespace() {
                break;
            }
            word.push(chars[index]);
            index += 1;
        }
        words.push(word);
    }

    words
}

/// Tokenizes one prose paragraph (which contains no blank lines) into `Word` pieces.
fn tokenize_prose_words(content: &str, pieces: &mut Vec<Piece>) {
    pieces.extend(tokenize_words(content).into_iter().map(Piece::Word));
}

/// If a math blob opens at `start`, returns the char index just past its close
/// (an unclosed blob runs to the end). Handles `$…$`, `$$…$$`, `\(…\)`, `\[…\]`,
/// `{. … .}`, and `{{. … .}}`.
fn math_blob_end(chars: &[char], start: usize) -> Option<usize> {
    let count = chars.len();
    if chars[start] == '$' {
        if chars.get(start + 1) == Some(&'$') {
            let mut index = start + 2;
            while index + 1 < count {
                if chars[index] == '$' && chars[index + 1] == '$' {
                    return Some(index + 2);
                }
                index += 1;
            }
            return Some(count);
        }
        let mut index = start + 1;
        while index < count {
            if chars[index] == '$' {
                return Some(index + 1);
            }
            index += 1;
        }
        return Some(count);
    }

    if chars[start] == '\\' {
        let close = match chars.get(start + 1) {
            Some('(') => ')',
            Some('[') => ']',
            _ => return None,
        };
        let mut index = start + 2;
        while index + 1 < count {
            if chars[index] == '\\' && chars[index + 1] == close {
                return Some(index + 2);
            }
            index += 1;
        }
        return Some(count);
    }

    if chars[start] == '{' {
        // Display math: `{{. ... .}}`
        if chars.get(start + 1) == Some(&'{') && chars.get(start + 2) == Some(&'.') {
            // Variadic writing templates use `{{...`; those are not math fragments.
            if chars.get(start + 3) == Some(&'.') {
                return None;
            }
            let mut index = start + 3;
            while index + 2 < count {
                if chars[index] == '.'
                    && chars[index + 1] == '}'
                    && chars[index + 2] == '}'
                    && (index == 0 || chars[index - 1] != '.')
                {
                    return Some(index + 3);
                }
                index += 1;
            }
            return Some(count);
        }

        // Inline math: `{. ... .}`
        if chars.get(start + 1) == Some(&'.') {
            // Variadic writing templates use `{...`; those are not math fragments.
            if chars.get(start + 2) == Some(&'.') {
                return None;
            }
            let mut index = start + 2;
            while index + 1 < count {
                if chars[index] == '.'
                    && chars[index + 1] == '}'
                    && chars.get(index + 2) != Some(&'}')
                    && (index == 0 || chars[index - 1] != '.')
                {
                    return Some(index + 2);
                }
                index += 1;
            }
            return Some(count);
        }
    }

    None
}

/// If `line` opens a Markdown code fence — its first non-space content is a run of
/// three or more backticks — returns `(leading-space indent, backtick count)`. An
/// info string after the backticks (e.g. ```` ```mlg ````) is allowed on an opener.
fn fence_open(line: &str) -> Option<(usize, usize)> {
    let line = line.trim_end_matches('\r');
    let body = line.trim_start_matches(' ');
    let indent = line.len() - body.len();
    let ticks = body.chars().take_while(|&ch| ch == '`').count();
    (ticks >= 3).then_some((indent, ticks))
}

/// Whether `line` closes a fence opened with `open_ticks` backticks: its only
/// non-whitespace content is a run of at least `open_ticks` backticks.
fn is_closing_fence(line: &str, open_ticks: usize) -> bool {
    let trimmed = line.trim();
    trimmed.len() >= open_ticks && trimmed.bytes().all(|byte| byte == b'`')
}

/// Removes up to `indent` leading spaces, mirroring how Markdown dedents fenced
/// content by the fence's own indentation.
fn dedent(line: &str, indent: usize) -> String {
    let line = line.trim_end_matches('\r');
    let leading = line.len() - line.trim_start_matches(' ').len();
    line[leading.min(indent)..].to_string()
}

/// Whether a word contains any math delimiter (LaTeX or Mathlingua).
fn word_contains_math(word: &str) -> bool {
    word.contains('$')
        || word.contains("\\(")
        || word.contains("\\[")
        || word_contains_mathlingua_fragment(word)
}

fn word_contains_mathlingua_fragment(word: &str) -> bool {
    let chars: Vec<char> = word.chars().collect();
    for i in 0..chars.len() {
        if chars[i] == '{' {
            if chars.get(i + 1) == Some(&'{') && chars.get(i + 2) == Some(&'.') {
                if chars.get(i + 3) != Some(&'.') {
                    return true;
                }
            } else if chars.get(i + 1) == Some(&'.') {
                if chars.get(i + 2) != Some(&'.') {
                    return true;
                }
            }
        }
    }
    false
}

/// Records edits that normalize the blank-line gap between consecutive top-level
/// items to exactly two blank lines. Only gaps that are entirely blank are touched.
fn collect_blank_line_edits(
    groups: &[ProtoGroup],
    lines: &[String],
    edits: &mut Vec<(usize, usize, Vec<String>)>,
) {
    for pair in groups.windows(2) {
        let current_end = group_last_row(&pair[0]);
        let next_start = pair[1].metadata.row;
        if next_start <= current_end + 1 || next_start > lines.len() {
            // Adjacent (no gap) or overlapping — inserting is handled below only for
            // the no-gap case.
        }

        let gap_start = current_end + 1;
        let gap_end = next_start; // exclusive
        if gap_end <= gap_start {
            continue;
        }
        let gap = &lines[gap_start..gap_end];
        // Only normalize pure-blank gaps (leave comments/dividers untouched).
        if gap.iter().all(|line| line.trim().is_empty()) && gap.len() != 2 {
            edits.push((gap_start, gap_end - 1, vec![String::new(), String::new()]));
        }
    }
}

/// Inserts two blank lines after each top-level `Id:` section that is not the last
/// section of its group — i.e. between top-level items the proto parser merged into
/// one group because no blank line separated them.
fn collect_item_boundary_edits(
    group: &ProtoGroup,
    lines: &[String],
    edits: &mut Vec<(usize, usize, Vec<String>)>,
) {
    let last_index = group.sections.len().saturating_sub(1);
    for (index, section) in group.sections.iter().enumerate() {
        if section.label == "Id" && index < last_index {
            let row = section_last_row(section);
            if row < lines.len() {
                edits.push((
                    row,
                    row,
                    vec![lines[row].clone(), String::new(), String::new()],
                ));
            }
        }
    }
}

/// The last source row occupied by a top-level group.
fn group_last_row(group: &ProtoGroup) -> usize {
    group
        .sections
        .iter()
        .map(section_last_row)
        .max()
        .unwrap_or(group.metadata.row)
}

fn section_last_row(section: &ProtoSection) -> usize {
    let own = section.metadata.row
        + section
            .inline_argument
            .as_deref()
            .map(|arg| arg.matches('\n').count())
            .unwrap_or(0);
    let arguments = section
        .arguments
        .iter()
        .map(argument_last_row)
        .max()
        .unwrap_or(own);
    own.max(arguments)
}

fn argument_last_row(argument: &ProtoArgument) -> usize {
    match argument {
        ProtoArgument::Formulation(formulation) => {
            formulation.metadata.row + formulation.text.matches('\n').count()
        }
        ProtoArgument::Text(text) => text.metadata.row + text.text.matches('\n').count(),
        ProtoArgument::Group(group) => group_last_row(group),
    }
}

/// Whether `text` (with leading content already trimmed) ends with an unescaped `"`.
fn closes_quoted_text(text: &str) -> bool {
    let text = text.trim_end();
    text.ends_with('"') && !trailing_quote_is_escaped(text)
}

fn line_closes_triple_quoted_text(text: &str) -> bool {
    text.trim_end().ends_with("\"\"\"")
}

fn trailing_quote_is_escaped(text: &str) -> bool {
    let mut backslashes = 0;
    for character in text[..text.len() - 1].chars().rev() {
        if character != '\\' {
            break;
        }
        backslashes += 1;
    }
    backslashes % 2 == 1
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::{CONFIG_FILE, DEFAULT_MARGIN, EventLog, ORIGIN, format_source, load_margin};
    use std::fs;

    /// Formats until a fixed point, asserting it is reached within a few passes.
    fn format_stable(source: &str, margin: usize) -> String {
        let mut current = source.to_string();
        for _ in 0..5 {
            match format_source(&current, margin) {
                Some(next) => current = next,
                None => return current,
            }
        }
        panic!("formatting did not stabilize:\n{current}");
    }

    #[test]
    fn reflows_over_long_inline_description() {
        let source = "[\\foo]\nDeclares: X\nDocumented:\n. description: \"The primitive object of the theory. Belonging is here.\"\nId: \"x\"\n";
        let formatted = format_source(source, 60).expect("expected a change");
        let lines: Vec<&str> = formatted.split('\n').collect();
        // Wrapped onto two lines, continuation indented to the content column (16).
        assert_eq!(
            lines[3],
            ". description: \"The primitive object of the theory."
        );
        assert_eq!(lines[4], "                Belonging is here.\"");
        // Idempotent.
        assert_eq!(format_source(&formatted, 60), None);
    }

    #[test]
    fn repacks_already_wrapped_inline_text_to_margin() {
        let source = "Text: \"Before beginning with the axioms,\n       it is worthwhile to engage the reader.\"\nId: \"x\"\n";
        // At a wide margin the two author lines repack onto one.
        let formatted = format_source(source, 120).expect("expected a change");
        assert!(formatted.contains(
            "Text: \"Before beginning with the axioms, it is worthwhile to engage the reader.\""
        ));
    }

    #[test]
    fn reflows_block_form_text_into_inline_form() {
        // Opening `"` at end of line, content on following lines (block form) is
        // reflowed into the canonical inline form.
        let source = "Text: \"\nBefore beginning with the axioms of Zermelo-Fraenkel set theory, it is\nworthwhile to engage with the reader's intuitive notion of a set and to justify\nthe axiomatic approach to set theory.\"\nId: \"x\"\n";
        let formatted = format_stable(source, 80);
        let lines: Vec<&str> = formatted.split('\n').collect();
        assert!(
            lines[0].starts_with("Text: \"Before beginning"),
            "first line should be inline: {:?}",
            lines[0]
        );
        // Continuation lines indented to the content column (7).
        assert!(
            lines[1].starts_with("       ") && !lines[1].trim().is_empty(),
            "continuation should be indented to col 7: {:?}",
            lines[1]
        );
        // Closing quote at the end, Id preserved.
        assert!(formatted.contains("set theory.\""));
        assert!(formatted.contains("Id: \"x\""));
    }

    /// Block form is reflowed to inline and repacked to the margin (fuller lines
    /// than the author's original breaks), and is idempotent.
    #[test]
    fn reflows_block_form_example_repacked_to_margin() {
        let source = "Text: \"\nBefore beginning with the axioms of Zermelo-Fraenkel set theory, it is\nworthwhile to engage with the reader's intuitive notion of a set and to justify\nthe axiomatic approach to set theory.\"\nId: \"8f66079c-6e4d-47d1-bb13-9798c5a9d36a\"\n";
        let expected = "Text: \"Before beginning with the axioms of Zermelo-Fraenkel set theory, it is worthwhile to engage\n       with the reader's intuitive notion of a set and to justify the axiomatic approach to set\n       theory.\"\nId: \"8f66079c-6e4d-47d1-bb13-9798c5a9d36a\"\n";
        assert_eq!(format_stable(source, 100), expected);
        // Every content line stays within the margin.
        for line in expected.split('\n') {
            assert!(line.len() <= 100, "line exceeds margin: {line:?}");
        }
        // Idempotent.
        assert_eq!(format_source(expected, 100), None);
    }

    #[test]
    fn preserves_paragraph_breaks() {
        let source = "Text: \"First paragraph here.\n\nSecond paragraph here.\"\nId: \"x\"\n";
        let formatted = format_stable(source, 120);
        let lines: Vec<&str> = formatted.split('\n').collect();
        assert_eq!(lines[0], "Text: \"First paragraph here.");
        assert_eq!(lines[1], "");
        assert_eq!(lines[2], "       Second paragraph here.\"");
    }

    #[test]
    fn normalizes_blank_lines_between_top_level_items_to_two() {
        let source = "Title: \"A\"\nId: \"1\"\n\n\n\n\nTitle: \"B\"\nId: \"2\"\n";
        let formatted = format_source(source, 120).expect("expected a change");
        assert_eq!(
            formatted,
            "Title: \"A\"\nId: \"1\"\n\n\nTitle: \"B\"\nId: \"2\"\n"
        );
    }

    #[test]
    fn inserts_missing_blank_lines_between_top_level_items() {
        let source = "Title: \"A\"\nId: \"1\"\nTitle: \"B\"\nId: \"2\"\n";
        let formatted = format_source(source, 120).expect("expected a change");
        assert_eq!(
            formatted,
            "Title: \"A\"\nId: \"1\"\n\n\nTitle: \"B\"\nId: \"2\"\n"
        );
    }

    #[test]
    fn honors_custom_margin() {
        let source = "Text: \"one two three four five six\"\nId: \"x\"\n";
        // A tiny margin forces wrapping; content column is 7 (`Text: "`).
        let formatted = format_source(source, 15).expect("expected a change");
        let lines: Vec<&str> = formatted.split('\n').collect();
        assert!(lines[0].starts_with("Text: \""));
        // At least one continuation line, indented to the content column (7).
        assert!(
            lines[1..]
                .iter()
                .any(|line| line.starts_with("       ") && !line.trim().is_empty()),
            "expected an indented continuation line in {lines:?}"
        );
        // The `Id:` line is preserved verbatim.
        assert!(formatted.contains("Id: \"x\""));
        // Idempotent.
        assert_eq!(format_source(&formatted, 15), None);
    }

    #[test]
    fn a_stale_print_margin_aborts_instead_of_reformatting() {
        // Falling back to the default width here would rewrap exactly the files
        // whose author had chosen a narrower margin, so formatting must not run.
        let dir = std::env::temp_dir().join(format!(
            "mlg-format-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&dir).unwrap();
        fs::write(
            dir.join(CONFIG_FILE),
            r#"{"name": "a", "version": "1", "print_margin": 80}"#,
        )
        .unwrap();

        let mut event_log = EventLog::new();
        let margin = load_margin(&dir, &mut event_log, ORIGIN);

        assert_eq!(margin, None, "a stale key must not yield a usable margin");
        assert!(
            event_log
                .events()
                .iter()
                .filter_map(crate::events::Event::as_message)
                .any(|message| message.message.contains("was renamed to \"margin\"")),
            "expected the rename to be reported"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_config_uses_the_default_margin() {
        let dir = std::env::temp_dir().join(format!(
            "mlg-format-test-missing-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&dir).unwrap();

        let mut event_log = EventLog::new();

        assert_eq!(
            load_margin(&dir, &mut event_log, ORIGIN),
            Some(DEFAULT_MARGIN)
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn leaves_text_with_multiline_latex_unchanged() {
        // The exact example from the feature request: a description containing
        // `$$…$$` and `\[…\]` display-math blocks must be left untouched.
        let source = "[\\foo]\nDeclares: x\nDocumented:\n. called: \"family indexed by $I?$\"\n. written: \"\\{A?_i\\}_{i \\in I?}\"\n. description: \"A family of sets is a function $A$ with domain $I$. When $A$ is\n                a family over $I$ one writes $\\{A_i\\}_{i \\in I}$ and $A_i$ for\n                $A(i)$.\n                $$\n                  \\int f(x) \\: dx\n                $$\n                Some more text\n                \\[\n                  \\int f(x) \\: dx\n                \\]\"\nId: \"a2451abb-cfc3-4655-a641-ff6826592e7d\"\n";
        assert_eq!(format_source(source, 100), None);
    }

    #[test]
    fn reflows_markdown_list_items_with_a_hanging_indent() {
        // Each `* ` item keeps its own line and marker; a long item wraps with its
        // continuation lines hanging-indented to align past the marker. The lead-in
        // sentence and the earlier paragraph still reflow as ordinary prose.
        let source = "Text: \"So far, our definitions have used `Declares: X`. That is, our\n       definitions do not specify a shape the entity being defined could have.\n\n       Mathlingua supports the following structures:\n       * `f(x_)` for mappings, i.e. entities that map and input to an output\n       * `(x, f(x_), (a, b)) for tuples, i.e. entities that are a fixed named sequence of items\n       * `{x_ : ...}` for collections, i.e. entities that are a collection of items\"\nId: \"c4eaa2cf-c945-47b8-a7bd-4d88b7834ba8\"\n";
        let expected = "Text: \"So far, our definitions have used `Declares: X`. That is, our definitions\n       do not specify a shape the entity being defined could have.\n\n       Mathlingua supports the following structures:\n       * `f(x_)` for mappings, i.e. entities that map and input to an output\n       * `(x, f(x_), (a, b)) for tuples, i.e. entities that are a fixed named\n         sequence of items\n       * `{x_ : ...}` for collections, i.e. entities that are a collection of\n         items\"\nId: \"c4eaa2cf-c945-47b8-a7bd-4d88b7834ba8\"\n";
        assert_eq!(format_source(source, 80).as_deref(), Some(expected));
        // Idempotent.
        assert_eq!(format_source(expected, 80), None);
    }

    #[test]
    fn preserves_nested_list_indentation_when_wrapping() {
        // A nested item (indented past the content column) keeps its level, and each
        // level wraps under its own hanging indent.
        let source = "Documented:\n. description: \"Outline:\n                * outer item that is reasonably long so that it wraps across the margin here\n                  * inner nested item that is also long enough to wrap across the print margin\n                * second outer\"\nId: \"x\"\n";
        let expected = "Documented:\n. description: \"Outline:\n                * outer item that is reasonably long so that it wraps\n                  across the margin here\n                  * inner nested item that is also long enough to wrap\n                    across the print margin\n                * second outer\"\nId: \"x\"\n";
        assert_eq!(format_source(source, 70).as_deref(), Some(expected));
        assert_eq!(format_source(expected, 70), None);
    }

    #[test]
    fn reflows_ordered_list_items_and_preserves_the_marker() {
        // Ordered markers (`1. `) survive and wrap with a hanging indent sized to the
        // marker width.
        let source = "Documented:\n. description: \"Steps:\n                1. do the first thing and then keep going until the line is quite long indeed\n                2. done\"\nId: \"x\"\n";
        let formatted = format_source(source, 60).expect("expected reflow");
        assert!(formatted.contains("                1. do the first thing and then keep going"));
        // The wrapped continuation hangs under the marker (content column + `1. `).
        assert!(formatted.contains("\n                   until"));
        assert!(formatted.contains("                2. done"));
        assert_eq!(format_source(&formatted, 60), None);
    }

    #[test]
    fn reflows_prose_paragraphs_around_a_fence_keeping_one_blank_line_on_each_side() {
        // A `Text:` value with prose paragraphs before and after a fence: every prose
        // paragraph reflows to the margin, the fence passes through verbatim, and each
        // blank-line separator collapses to exactly one blank line (no stray indent
        // line before the fence, no lost blank line after it).
        let source = "Text: \"The Declares: construct is used to specify an abstract concept, called a\n       type in other languages. To start, we specify that a set is an type.\n       We'll expand on it as we continue.\n\n       Here is the minimal content needed to define an abstract concept in Mathlingua.\n\n       ```mlg\n       [\\set]\n       Declares: X\n       Documented:\n       . called: \\\"set\\\"\n       Id: \\\"a0759217-e1f6-412c-982a-0038cd17a3a1\\\"\n       ```\n\n       The `\\set` declares the name used to identify this type.  The `Declares:`\n       construct is used to define abstract concepts, i.e. types.\"\nId: \"e798d1a3-1029-44f3-8b92-d794cbb6596c\"\n";
        let expected = "Text: \"The Declares: construct is used to specify an abstract concept, called a\n       type in other languages. To start, we specify that a set is an type.\n       We'll expand on it as we continue.\n\n       Here is the minimal content needed to define an abstract concept in\n       Mathlingua.\n\n       ```mlg\n       [\\set]\n       Declares: X\n       Documented:\n       . called: \\\"set\\\"\n       Id: \\\"a0759217-e1f6-412c-982a-0038cd17a3a1\\\"\n       ```\n\n       The `\\set` declares the name used to identify this type. The `Declares:`\n       construct is used to define abstract concepts, i.e. types.\"\nId: \"e798d1a3-1029-44f3-8b92-d794cbb6596c\"\n";
        assert_eq!(format_source(source, 80).as_deref(), Some(expected));
        // Idempotent.
        assert_eq!(format_source(expected, 80), None);
    }

    #[test]
    fn reflows_prose_after_a_code_fence_and_attaches_the_closing_quote() {
        // Prose following a fence starts a fresh indented line and reflows; a value
        // that ends at the fence keeps the closing `"` on the closing fence line.
        let source = "Text: \"```\nx = 1\n```\nthen some words after the fence here\"\nId: \"y\"\n";
        let expected = "Text: \"\n       ```\n       x = 1\n       ```\n       then some words after the fence here\"\nId: \"y\"\n";
        assert_eq!(format_source(source, 55).as_deref(), Some(expected));
        assert_eq!(format_source(expected, 55), None);
    }

    #[test]
    fn reflows_prose_around_a_code_fence() {
        // The example from the bug report: the surrounding prose is reflowed to the
        // margin, but the embedded ```` ```mlg-fragment ```` fence is emitted verbatim
        // — its spacing and line breaks are significant and must never be collapsed.
        let source = "Text: \"We don't want to require a set to have a particular shape. We want it to\n       be abstract in that regard, but we want to allow write\n       ```mlg-fragment\n          X := {x__ : x_ is \\real} is \\set\n       ```\"\nId: \"8d820a04-5252-4ef2-9177-b1b328328197\"\n";
        let expected = "Text: \"We don't want to require a set to have a\n       particular shape. We want it to be abstract in\n       that regard, but we want to allow write\n       ```mlg-fragment\n          X := {x__ : x_ is \\real} is \\set\n       ```\"\nId: \"8d820a04-5252-4ef2-9177-b1b328328197\"\n";
        assert_eq!(format_source(source, 55).as_deref(), Some(expected));
        // Idempotent: the reflowed form is a fixed point.
        assert_eq!(format_source(expected, 55), None);
    }

    #[test]
    fn preserves_interior_spacing_of_a_code_fence() {
        // Significant interior whitespace (`a    b`) and relative indentation (`  c`)
        // inside the fence survive verbatim while the prose before it is reflowed.
        let source = "Documented:\n. description: \"Consider the example below where spacing matters a great deal here:\n                ```\n                a    b\n                  c\n                ```\"\nId: \"x\"\n";
        let expected = "Documented:\n. description: \"Consider the example below where spacing\n                matters a great deal here:\n                ```\n                a    b\n                  c\n                ```\"\nId: \"x\"\n";
        assert_eq!(format_source(source, 60).as_deref(), Some(expected));
        assert_eq!(format_source(expected, 60), None);
    }

    #[test]
    fn keeps_inline_math_with_internal_spaces_whole() {
        // `$\{A_i\}_{i \in I}$` has internal spaces but must never be split.
        let source = "Documented:\n. description: \"aaaa bbbb cccc dddd eeee ffff $\\{A_i\\}_{i \\in I}$ gggg hhhh\"\nId: \"x\"\n";
        let formatted = format_source(source, 40).expect("expected wrapping at margin 40");
        assert!(
            formatted.contains("$\\{A_i\\}_{i \\in I}$"),
            "inline math blob was split: {formatted}"
        );
        // The blob sits entirely on one line (no line contains only part of it).
        for line in formatted.split('\n') {
            let opens = line.matches('$').count();
            assert!(opens % 2 == 0, "unbalanced `$` on a line: {line:?}");
        }
    }

    #[test]
    fn leaves_text_with_overwide_latex_blob_unchanged() {
        // A single-line blob too wide to fit on a line → the author laid it out.
        let source = "Documented:\n. description: \"text $blobbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb$ more\"\nId: \"x\"\n";
        assert_eq!(format_source(source, 30), None);
    }

    #[test]
    fn still_reflows_descriptions_with_short_inline_math() {
        // Short inline math does not block reflow.
        let source = "Documented:\n. description: \"The value $x$ satisfies $x \\in A$ and also $x \\notin B$ under the stated hypotheses here.\"\nId: \"x\"\n";
        let formatted = format_source(source, 60).expect("expected reflow");
        assert!(formatted.split('\n').count() > 3, "expected wrapping");
        assert!(formatted.contains("$x \\in A$"));
        assert_eq!(format_source(&formatted, 60), None); // idempotent
    }

    #[test]
    fn leaves_already_formatted_source_unchanged() {
        let source = "Title: \"A\"\nId: \"1\"\n\n\nText: \"short enough\"\nId: \"2\"\n";
        assert_eq!(format_source(source, 120), None);
    }

    #[test]
    fn keeps_inline_mathlingua_fragments_whole() {
        // `{. x is \real .}` has internal spaces but must never be split across lines.
        let source = "Documented:\n. description: \"aaaa bbbb cccc dddd eeee ffff {. x is \\real .} gggg hhhh\"\nId: \"x\"\n";
        let formatted = format_source(source, 40).expect("expected wrapping at margin 40");
        assert!(
            formatted.contains("{. x is \\real .}"),
            "inline mathlingua fragment was split: {formatted}"
        );
        // The fragment sits entirely on one line.
        for line in formatted.split('\n') {
            if line.contains("{.") {
                assert!(line.contains(".}"), "unclosed fragment on a line: {line:?}");
            }
        }
    }

    #[test]
    fn keeps_display_mathlingua_fragments_whole() {
        // `{{. x^2 + y^2 = z^2 .}}` has internal spaces and must never be split across lines.
        let source = "Documented:\n. description: \"aaaa bbbb cccc dddd eeee {{. x^2 + y^2 = z^2 .}} ffff gggg\"\nId: \"x\"\n";
        let formatted = format_source(source, 45).expect("expected wrapping at margin 45");
        assert!(
            formatted.contains("{{. x^2 + y^2 = z^2 .}}"),
            "display mathlingua fragment was split: {formatted}"
        );
        for line in formatted.split('\n') {
            if line.contains("{{.") {
                assert!(line.contains(".}}"), "unclosed display fragment on a line: {line:?}");
            }
        }
    }

    #[test]
    fn leaves_text_with_multiline_mathlingua_fragment_unchanged() {
        // A multi-line display fragment must be left untouched.
        let source = "Documented:\n. description: \"Some text before\n                {{.\n                  x + y = z\n                .}}\n                and some text after\"\nId: \"x\"\n";
        assert_eq!(format_source(source, 80), None);
    }

    #[test]
    fn leaves_text_with_overwide_mathlingua_fragment_unchanged() {
        // A single-line fragment too wide to fit on a line → leave untouched.
        let source = "Documented:\n. description: \"text {. blobbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb .} more\"\nId: \"x\"\n";
        assert_eq!(format_source(source, 30), None);
    }

    #[test]
    fn does_not_confuse_ellipsis_or_variadic_templates() {
        // `{...}` in text must not be treated as `{.`
        let source = "Documented:\n. description: \"Consider the sequence of terms {...} where each term is clearly specified.\"\nId: \"x\"\n";
        let formatted = format_source(source, 50).expect("expected wrapping");
        assert!(formatted.contains("{...}"));
    }

    #[test]
    fn reflows_triple_quoted_proof_with_unescaped_quotes() {
        let source = "Theorem:\nthen: x = x\nProof: \"\"\"\nBecause \"x = y\" and \"y = z\", we have \"x = z\".\n\"\"\"\nId: \"x\"\n";
        let formatted = format_stable(source, 100);
        assert_eq!(
            formatted,
            "Theorem:\nthen: x = x\nProof: \"\"\"Because \"x = y\" and \"y = z\", we have \"x = z\".\"\"\"\nId: \"x\"\n"
        );
        // Idempotent
        assert_eq!(format_source(&formatted, 100), None);
    }

    #[test]
    fn wraps_triple_quoted_text_to_margin_with_unescaped_quotes() {
        let source = "Proof: \"\"\"Because \"x = y\" and \"y = z\", we know that \"x = z\" holds by transitivity.\"\"\"\nId: \"x\"\n";
        let formatted = format_source(source, 50).expect("expected wrap");
        let lines: Vec<&str> = formatted.split('\n').collect();
        assert!(lines[0].starts_with("Proof: \"\"\"Because \"x = y\""));
        assert!(lines[1].starts_with("          ")); // indented to col 10 (Proof: """)
        assert!(lines.last().unwrap().is_empty() || lines[lines.len() - 2].ends_with("\"\"\""));
        assert_eq!(format_source(&formatted, 50), None);
    }
}

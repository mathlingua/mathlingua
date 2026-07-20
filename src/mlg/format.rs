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

    // A stale `print_margin` aborts rather than falling back: formatting every
    // file to the default width would rewrap exactly the files the author set a
    // narrower margin for.
    let Some(margin) = load_margin(&root, event_log) else {
        return Err(io::Error::other("stale margin field"));
    };
    let files = collection_source_files(&root, event_log, ORIGIN);
    let mut formatted = 0usize;

    for file in files {
        let Ok(source) = fs::read_to_string(&file) else {
            continue;
        };
        if let Some(updated) = format_source(&source, margin) {
            if let Err(error) = fs::write(&file, updated) {
                event_log.user_error_at_path(
                    Some(ORIGIN),
                    file.clone(),
                    format!("Failed to write formatted source: {error}"),
                );
                continue;
            }
            formatted += 1;
        }
    }

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

/// The `margin` from `mlg.json`, or the default when unset/unreadable.
///
/// `None` means the config still uses the pre-rename `print_margin` key, which
/// is reported rather than ignored: silently formatting to the default width
/// would rewrap every file the author had set a narrower margin for.
fn load_margin(root: &Path, event_log: &mut EventLog) -> Option<usize> {
    let path = root.join(CONFIG_FILE);
    let Ok(contents) = fs::read_to_string(&path) else {
        return Some(DEFAULT_MARGIN);
    };

    if uses_legacy_margin_field(&contents) {
        event_log.user_error_at_path(Some(ORIGIN), path, legacy_margin_field_message());
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
    // The opening `"` is the first quote on the line (labels never contain quotes).
    let quote = open_line.find('"')?;
    let content_col = quote + 1;

    // Find the closing line (the first line from `row` that ends with an unescaped
    // `"`, considering the opening quote itself for a single-line value).
    let close_row = (row..lines.len()).find(|&index| {
        let line = &lines[index];
        let end = if index == row { content_col } else { 0 };
        closes_quoted_text(&line[end.min(line.len())..])
    })?;

    // Assemble the raw content lines (between the quotes).
    let mut content_lines: Vec<&str> = Vec::new();
    for (index, line) in lines[row..=close_row].iter().enumerate() {
        let index = row + index;
        let from = if index == row { content_col } else { 0 };
        let mut slice = &line[from.min(line.len())..];
        if index == close_row {
            slice = slice.strip_suffix('"').unwrap_or(slice);
        }
        content_lines.push(slice);
    }

    let first_prefix = &open_line[..content_col];
    let replacement = reflow_text(&content_lines, first_prefix, content_col, margin)?;
    Some((row, close_row, replacement))
}

/// One unit of a text value for reflow: a whitespace-delimited word (with LaTeX
/// blobs kept whole, so their internal spaces don't split it) or a paragraph break.
enum Piece {
    Word(String),
    Paragraph,
}

/// Reflows text content into lines that fit within `margin`, treating LaTeX blobs
/// (`$…$`, `$$…$$`, `\(…\)`, `\[…\]`) as atomic tokens that are never split or
/// modified. The first line keeps the verbatim `first_prefix` (`…label: "`); every
/// other line is indented to `content_col`; a blank line separates paragraphs; the
/// closing `"` is appended to the final line.
///
/// Returns `None` (leave the value unchanged) when a LaTeX blob spans multiple
/// lines or is too wide to fit on a line — the author has laid such content out by
/// hand and it must not be reflowed.
fn reflow_text(
    content_lines: &[&str],
    first_prefix: &str,
    content_col: usize,
    margin: usize,
) -> Option<Vec<String>> {
    let content = content_lines.join("\n");
    let pieces = tokenize_reflow_pieces(&content);
    let available = margin.saturating_sub(content_col);

    for piece in &pieces {
        if let Piece::Word(word) = piece {
            let has_latex = word_contains_latex(word);
            // A multi-line LaTeX blob, or a blob too wide to sit on a line, means the
            // author has already laid this out — leave the whole value untouched.
            if word.contains('\n') || (has_latex && word.chars().count() > available) {
                return None;
            }
        }
    }

    let indent = " ".repeat(content_col);
    let mut out: Vec<String> = Vec::new();
    let mut current = first_prefix.to_string();
    let mut has_word = false;

    for piece in pieces {
        match piece {
            Piece::Paragraph => {
                out.push(std::mem::take(&mut current));
                out.push(String::new());
                current = indent.clone();
                has_word = false;
            }
            Piece::Word(word) => {
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
        }
    }
    out.push(current);

    if let Some(last) = out.last_mut() {
        last.push('"');
    }
    Some(out)
}

/// Splits text content into words and paragraph breaks, keeping each LaTeX blob
/// whole (its internal whitespace and newlines are not word/paragraph separators).
fn tokenize_reflow_pieces(content: &str) -> Vec<Piece> {
    let chars: Vec<char> = content.chars().collect();
    let count = chars.len();
    let mut pieces = Vec::new();
    let mut index = 0;

    while index < count {
        if chars[index].is_whitespace() {
            let mut newlines = 0;
            while index < count && chars[index].is_whitespace() {
                if chars[index] == '\n' {
                    newlines += 1;
                }
                index += 1;
            }
            if newlines >= 2 {
                pieces.push(Piece::Paragraph);
            }
            continue;
        }

        let mut word = String::new();
        while index < count {
            if let Some(end) = latex_blob_end(&chars, index) {
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
        pieces.push(Piece::Word(word));
    }

    pieces
}

/// If a LaTeX blob opens at `start`, returns the char index just past its close
/// (an unclosed blob runs to the end). Handles `$…$`, `$$…$$`, `\(…\)`, `\[…\]`.
fn latex_blob_end(chars: &[char], start: usize) -> Option<usize> {
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

    None
}

/// Whether a word contains any LaTeX delimiter.
fn word_contains_latex(word: &str) -> bool {
    word.contains('$') || word.contains("\\(") || word.contains("\\[")
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
    let own = section.metadata.row;
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
    use super::{CONFIG_FILE, DEFAULT_MARGIN, EventLog, format_source, load_margin};
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
        let source = "[\\foo]\nDescribes: X\nDocumented:\n. description: \"The primitive object of the theory. Belonging is here.\"\nId: \"x\"\n";
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
        let margin = load_margin(&dir, &mut event_log);

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

        assert_eq!(load_margin(&dir, &mut event_log), Some(DEFAULT_MARGIN));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn leaves_text_with_multiline_latex_unchanged() {
        // The exact example from the feature request: a description containing
        // `$$…$$` and `\[…\]` display-math blocks must be left untouched.
        let source = "[\\foo]\nDescribes: x\nDocumented:\n. called: \"family indexed by $I?$\"\n. written: \"\\{A?_i\\}_{i \\in I?}\"\n. description: \"A family of sets is a function $A$ with domain $I$. When $A$ is\n                a family over $I$ one writes $\\{A_i\\}_{i \\in I}$ and $A_i$ for\n                $A(i)$.\n                $$\n                  \\int f(x) \\: dx\n                $$\n                Some more text\n                \\[\n                  \\int f(x) \\: dx\n                \\]\"\nId: \"a2451abb-cfc3-4655-a641-ff6826592e7d\"\n";
        assert_eq!(format_source(source, 100), None);
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
}

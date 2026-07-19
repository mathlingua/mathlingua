use crate::backend::collection::{collection_source_files, find_collection_root};
use crate::backend::config::{CONFIG_FILE, Config, DEFAULT_PRINT_MARGIN};
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

    let margin = load_print_margin(&root);
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

/// The `print_margin` from `mlg.json`, or the default when unset/unreadable.
fn load_print_margin(root: &Path) -> usize {
    fs::read_to_string(root.join(CONFIG_FILE))
        .ok()
        .and_then(|contents| serde_json::from_str::<Config>(&contents).ok())
        .map_or(DEFAULT_PRINT_MARGIN, |config| config.print_margin())
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

/// Builds the reflow edit for the inline text value that begins on `row`, or `None`
/// if it is block-form (content starts on the next line) or malformed.
fn reflow_inline_text(
    row: usize,
    lines: &[String],
    margin: usize,
) -> Option<(usize, usize, Vec<String>)> {
    let open_line = lines.get(row)?;
    // The opening `"` is the first quote on the line (labels never contain quotes).
    let quote = open_line.find('"')?;
    let content_col = quote + 1;

    // Block form: nothing but whitespace follows the opening `"` on this line.
    if open_line[content_col..].trim().is_empty() {
        return None;
    }

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
    let replacement = reflow_text(&content_lines, first_prefix, content_col, margin);
    Some((row, close_row, replacement))
}

/// Reflows text content into lines that fit within `margin`. The first line keeps
/// the verbatim `first_prefix` (`…label: "`); every other line is indented to
/// `content_col`. Blank lines separate paragraphs, each reflowed independently. The
/// closing `"` is appended to the final line.
fn reflow_text(
    content_lines: &[&str],
    first_prefix: &str,
    content_col: usize,
    margin: usize,
) -> Vec<String> {
    let indent = " ".repeat(content_col);
    let mut out: Vec<String> = Vec::new();

    // Split into paragraphs on whitespace-only content lines.
    let paragraphs: Vec<&[&str]> = content_lines
        .split(|line| line.trim().is_empty())
        .filter(|paragraph| paragraph.iter().any(|line| !line.trim().is_empty()))
        .collect();

    if paragraphs.is_empty() {
        // An all-whitespace text value: keep a single empty-content line.
        return vec![format!("{first_prefix}\"")];
    }

    for (paragraph_index, paragraph) in paragraphs.iter().enumerate() {
        if paragraph_index > 0 {
            out.push(String::new());
        }

        let words: Vec<&str> = paragraph
            .iter()
            .flat_map(|line| line.split_whitespace())
            .collect();

        let mut current = if paragraph_index == 0 {
            first_prefix.to_string()
        } else {
            indent.clone()
        };
        let mut has_word = false;

        for word in words {
            if has_word && current.len() + 1 + word.len() > margin {
                out.push(std::mem::take(&mut current));
                current = indent.clone();
                current.push_str(word);
                has_word = true;
            } else {
                if has_word {
                    current.push(' ');
                }
                current.push_str(word);
                has_word = true;
            }
        }

        out.push(current);
    }

    if let Some(last) = out.last_mut() {
        last.push('"');
    }
    out
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
    use super::format_source;

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
        let source =
            "Text: \"Before beginning with the axioms,\n       it is worthwhile to engage the reader.\"\nId: \"x\"\n";
        // At a wide margin the two author lines repack onto one.
        let formatted = format_source(source, 120).expect("expected a change");
        assert!(formatted.contains(
            "Text: \"Before beginning with the axioms, it is worthwhile to engage the reader.\""
        ));
    }

    #[test]
    fn leaves_block_form_text_untouched() {
        // Opening `"` at end of line, content on following lines → block form.
        let source = "Text: \"\nBefore beginning with the axioms of set theory, it is worthwhile.\n\"\nId: \"x\"\n";
        assert_eq!(format_source(source, 40), None);
    }

    #[test]
    fn preserves_paragraph_breaks() {
        let source =
            "Text: \"First paragraph here.\n\nSecond paragraph here.\"\nId: \"x\"\n";
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
        assert_eq!(formatted, "Title: \"A\"\nId: \"1\"\n\n\nTitle: \"B\"\nId: \"2\"\n");
    }

    #[test]
    fn inserts_missing_blank_lines_between_top_level_items() {
        let source = "Title: \"A\"\nId: \"1\"\nTitle: \"B\"\nId: \"2\"\n";
        let formatted = format_source(source, 120).expect("expected a change");
        assert_eq!(formatted, "Title: \"A\"\nId: \"1\"\n\n\nTitle: \"B\"\nId: \"2\"\n");
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
    fn leaves_already_formatted_source_unchanged() {
        let source = "Title: \"A\"\nId: \"1\"\n\n\nText: \"short enough\"\nId: \"2\"\n";
        assert_eq!(format_source(source, 120), None);
    }
}

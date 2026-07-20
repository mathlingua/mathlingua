//! `mlg report` — file a parser issue against the Mathlingua repository.
//!
//! Hidden command. Given the `Id:` of one or more top-level items, it extracts
//! them and their dependencies exactly as `mlg extract` would, drops that into a
//! Markdown issue template, and opens the user's editor to fill in. On exit the
//! finished issue is shown and the user chooses to report it, edit again, or
//! cancel. Reporting posts through the `gh` CLI when it is available, and
//! otherwise opens a prefilled issue form in the browser.

use crate::events::{EventLog, EventLogListener};
use crate::mlg::extract::{check_error_warning, extract_source};
use crate::mlg::util::no_errors_since;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const ORIGIN: &str = "mlg_report";
const REPOSITORY: &str = "mathlingua/mathlingua";
const ISSUE_FORM_URL: &str = "https://github.com/mathlingua/mathlingua/issues/new";

pub struct ReportResult {
    pub event_log: EventLog,
    pub successful: bool,
}

pub fn report(
    cwd: &Path,
    ids: &[String],
    listener: Option<Box<dyn EventLogListener>>,
) -> ReportResult {
    let mut event_log = EventLog::new();
    if let Some(listener) = listener {
        event_log.add_boxed_listener(listener);
    }

    let starting_event_count = event_log.events().len();
    report_in(cwd, ids, &mut event_log);
    let successful = no_errors_since(&event_log, starting_event_count);

    ReportResult {
        event_log,
        successful,
    }
}

fn report_in(cwd: &Path, ids: &[String], event_log: &mut EventLog) {
    let Some(extracted) = extract_source(cwd, ids, event_log) else {
        return;
    };
    let source = extracted.source;

    // The errors themselves are not quoted into the issue — the issue is about
    // what the parser did, not about the collection's state — so only the count
    // is surfaced, pointing at `mlg check` for the detail.
    if !extracted.check_errors.is_empty() {
        event_log.user_warning(
            Some(ORIGIN),
            check_error_warning(extracted.check_errors.len()),
        );
    }

    if !io::stdin().is_terminal() {
        event_log.user_error(
            Some(ORIGIN),
            "`mlg report` needs an interactive terminal; use `mlg extract` to get the code alone",
        );
        return;
    }

    let scratch = match ScratchFile::create(&issue_template(&source)) {
        Ok(scratch) => scratch,
        Err(error) => {
            event_log.user_error(
                Some(ORIGIN),
                format!("Could not create a temporary file for the issue: {error}"),
            );
            return;
        }
    };

    // Edit, preview, and choose — looping for as long as the user keeps editing.
    loop {
        if let Err(error) = open_editor(scratch.path()) {
            event_log.user_error(Some(ORIGIN), format!("Could not open an editor: {error}"));
            return;
        }

        let text = match scratch.read() {
            Ok(text) => text,
            Err(error) => {
                event_log.user_error(
                    Some(ORIGIN),
                    format!("Could not read the edited issue: {error}"),
                );
                return;
            }
        };

        let (title, body) = split_title_and_body(&text);
        if title.is_empty() {
            event_log.user_error(Some(ORIGIN), "The issue has no title; nothing was reported");
            return;
        }

        println!("\n{}\n", preview(&title, &body));
        match prompt_choice(event_log) {
            Some(Choice::Report) => {
                submit(&title, &body, event_log);
                return;
            }
            Some(Choice::Edit) => continue,
            Some(Choice::Cancel) | None => {
                event_log.user_log(Some(ORIGIN), "Cancelled; nothing was reported");
                return;
            }
        }
    }
}

// ----------------------------[ issue contents ]-------------------------------

/// The Markdown the editor opens on: a title line, prompts for the user to fill
/// in, and the extracted MathLingua already fenced and ready to quote.
fn issue_template(source: &str) -> String {
    let fence = fence_for(source);
    format!(
        "\
# Parser issue: <one-line summary>

## What happened

<Describe what the parser did.>

## What was expected

<Describe what it should have done instead.>

## Reproduction

The MathLingua below is a self-contained collection extracted with `mlg extract`.

{fence}mlg
{source}
{fence}
"
    )
}

/// A Markdown fence long enough to wrap `source` intact.
///
/// Quoted text in a `.mlg` file may itself embed ` ```mlg ` fences, so a fixed
/// three-backtick fence could be closed early by the very code being reported.
/// The fence is one backtick longer than the longest run in `source`, and never
/// shorter than three.
fn fence_for(source: &str) -> String {
    let mut longest = 0usize;
    let mut current = 0usize;
    for character in source.chars() {
        if character == '`' {
            current += 1;
            longest = longest.max(current);
        } else {
            current = 0;
        }
    }

    "`".repeat(longest.max(2) + 1)
}

/// Split the edited Markdown into a GitHub issue title and body.
///
/// The title is the first non-empty line with any leading `#` markers stripped;
/// everything after that line is the body. This keeps the template readable as
/// Markdown while giving the issue a real title.
fn split_title_and_body(text: &str) -> (String, String) {
    let mut lines = text.lines();
    let title = lines
        .by_ref()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.trim_start_matches('#').trim())
        .unwrap_or_default()
        .to_string();
    let body = lines.collect::<Vec<_>>().join("\n").trim().to_string();

    (title, body)
}

/// How the finished issue is shown before the user commits to posting it.
fn preview(title: &str, body: &str) -> String {
    let rule = "-".repeat(60);
    format!("{rule}\n{title}\n{rule}\n\n{body}\n\n{rule}")
}

// ------------------------------[ the prompt ]---------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Choice {
    Report,
    Edit,
    Cancel,
}

/// Map a typed answer to a choice. Blank input is deliberately not a choice:
/// posting publicly should never happen by pressing Enter.
fn parse_choice(input: &str) -> Option<Choice> {
    match input.trim().to_ascii_lowercase().as_str() {
        "r" | "report" => Some(Choice::Report),
        "e" | "edit" => Some(Choice::Edit),
        "c" | "cancel" | "q" | "quit" => Some(Choice::Cancel),
        _ => None,
    }
}

/// Ask until the answer is understood. `None` means end-of-input, which is
/// treated as a cancellation by the caller.
fn prompt_choice(event_log: &mut EventLog) -> Option<Choice> {
    loop {
        print!("Report this issue to {REPOSITORY}? [r]eport / [e]dit / [c]ancel: ");
        if io::stdout().flush().is_err() {
            return None;
        }

        let mut answer = String::new();
        match io::stdin().read_line(&mut answer) {
            Ok(0) | Err(_) => return None,
            Ok(_) => {}
        }

        if let Some(choice) = parse_choice(&answer) {
            return Some(choice);
        }
        event_log.user_warning(Some(ORIGIN), "Please answer r, e, or c");
    }
}

// -------------------------------[ posting ]-----------------------------------

/// Post the issue, preferring `gh` and falling back to a prefilled browser form.
///
/// `gh` carries the whole body regardless of size and needs no browser, so it is
/// tried first. The browser form is the fallback for machines without `gh`; its
/// body travels in the URL, so an oversized issue is posted with a pointer to
/// the code rather than silently truncated.
fn submit(title: &str, body: &str, event_log: &mut EventLog) {
    match submit_with_gh(title, body) {
        Ok(url) => {
            let url = url.trim();
            if url.is_empty() {
                event_log.user_log(Some(ORIGIN), "Reported the issue");
            } else {
                event_log.user_log(Some(ORIGIN), format!("Reported the issue: {url}"));
            }
        }
        Err(GhError::Unavailable) => {
            event_log.user_log(
                Some(ORIGIN),
                "The `gh` CLI is not available; opening the issue form in a browser",
            );
            open_issue_form(title, body, event_log);
        }
        Err(GhError::Failed(message)) => {
            event_log.user_error(
                Some(ORIGIN),
                format!("`gh` could not file the issue: {message}"),
            );
        }
    }
}

enum GhError {
    /// `gh` is not installed, so the caller should fall back to the browser.
    Unavailable,
    /// `gh` ran but refused — usually because it is not authenticated. Falling
    /// back would silently discard whatever it was complaining about, so this
    /// surfaces to the user instead.
    Failed(String),
}

fn submit_with_gh(title: &str, body: &str) -> Result<String, GhError> {
    let output = Command::new("gh")
        .args(["issue", "create", "--repo", REPOSITORY, "--title"])
        .arg(title)
        .arg("--body")
        .arg(body)
        .output()
        .map_err(|_| GhError::Unavailable)?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }

    let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(GhError::Failed(if message.is_empty() {
        "gh exited with an error".to_string()
    } else {
        message
    }))
}

/// The longest issue body worth putting in a URL. Browsers and servers cap URL
/// length well above this, but a body near the cap risks a silent truncation, so
/// anything larger is replaced with a note.
const MAX_URL_BODY: usize = 6000;

/// A prefilled "new issue" URL for `title` and `body`.
fn issue_url(title: &str, body: &str) -> String {
    let body = if body.len() > MAX_URL_BODY {
        "<The extracted MathLingua was too large to prefill. Run `mlg extract` \
         and paste its output here.>"
    } else {
        body
    };

    format!(
        "{ISSUE_FORM_URL}?title={}&body={}",
        percent_encode(title),
        percent_encode(body)
    )
}

/// Percent-encode `text` for use in a URL query value, escaping everything
/// outside the unreserved set so that Markdown punctuation survives intact.
fn percent_encode(text: &str) -> String {
    let mut encoded = String::with_capacity(text.len());
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

fn open_issue_form(title: &str, body: &str, event_log: &mut EventLog) {
    let url = issue_url(title, body);
    let (program, leading) = browser_command();

    let opened = Command::new(program)
        .args(leading)
        .arg(&url)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false);

    if opened {
        event_log.user_log(
            Some(ORIGIN),
            "Opened the issue form in your browser; submit it there to finish reporting",
        );
    } else {
        event_log.user_warning(
            Some(ORIGIN),
            format!("Could not open a browser. Open this URL to file the issue:\n{url}"),
        );
    }
}

fn browser_command() -> (&'static str, &'static [&'static str]) {
    if cfg!(target_os = "macos") {
        ("open", &[])
    } else if cfg!(target_os = "windows") {
        ("cmd", &["/C", "start", ""])
    } else {
        ("xdg-open", &[])
    }
}

// -------------------------------[ editing ]-----------------------------------

/// A temp file holding the issue across editor sessions, removed on drop.
struct ScratchFile {
    path: PathBuf,
}

impl ScratchFile {
    fn create(contents: &str) -> io::Result<Self> {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("mlg-report-{}-{unique}.md", std::process::id()));
        fs::write(&path, contents)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn read(&self) -> io::Result<String> {
        fs::read_to_string(&self.path)
    }
}

impl Drop for ScratchFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Run the user's editor on `path` and wait for it to exit.
fn open_editor(path: &Path) -> io::Result<()> {
    let (program, arguments) = editor_command();

    let status = Command::new(&program).args(&arguments).arg(path).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("{program} exited with {status}")))
    }
}

/// The editor to run, as a program and its leading arguments.
///
/// `$VISUAL` wins over `$EDITOR` (the usual precedence for a full-screen
/// editor), and either may carry arguments, as `EDITOR="code --wait"` does.
fn editor_command() -> (String, Vec<String>) {
    let configured = std::env::var("VISUAL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("EDITOR")
                .ok()
                .filter(|value| !value.trim().is_empty())
        });

    let Some(configured) = configured else {
        return (default_editor().to_string(), Vec::new());
    };

    let mut parts = configured.split_whitespace().map(str::to_string);
    match parts.next() {
        Some(program) => (program, parts.collect()),
        None => (default_editor().to_string(), Vec::new()),
    }
}

fn default_editor() -> &'static str {
    if cfg!(target_os = "windows") {
        "notepad"
    } else {
        "vi"
    }
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_template_quotes_the_extracted_source() {
        let template = issue_template("[\\set]\nDescribes: S");

        assert!(template.contains("```mlg\n[\\set]\nDescribes: S\n```"));
    }

    #[test]
    fn the_template_starts_with_a_title_line() {
        let template = issue_template("x");
        let (title, body) = split_title_and_body(&template);

        assert!(title.starts_with("Parser issue:"));
        assert!(body.contains("## Reproduction"));
    }

    #[test]
    fn a_plain_source_uses_a_three_backtick_fence() {
        assert_eq!(fence_for("[\\set]\nDescribes: S"), "```");
    }

    #[test]
    fn a_source_containing_a_fence_uses_a_longer_one() {
        // The extracted item embeds its own ```mlg fence in quoted text.
        let source = "Describes: S\nDocumented:\n. written: \"```mlg\\nx\\n```\"";

        assert_eq!(fence_for(source), "````");
    }

    #[test]
    fn the_fence_clears_the_longest_backtick_run() {
        assert_eq!(fence_for("a ````` b ``` c"), "``````");
    }

    #[test]
    fn splitting_strips_the_heading_markers_from_the_title() {
        let (title, body) = split_title_and_body("# A bug\n\nsome details\n");

        assert_eq!(title, "A bug");
        assert_eq!(body, "some details");
    }

    #[test]
    fn splitting_skips_leading_blank_lines() {
        let (title, _) = split_title_and_body("\n\n   \n# A bug\n\nbody\n");

        assert_eq!(title, "A bug");
    }

    #[test]
    fn splitting_an_empty_document_yields_no_title() {
        let (title, body) = split_title_and_body("   \n\n");

        assert!(title.is_empty());
        assert!(body.is_empty());
    }

    #[test]
    fn choices_accept_short_and_long_answers() {
        assert_eq!(parse_choice("r"), Some(Choice::Report));
        assert_eq!(parse_choice("  REPORT \n"), Some(Choice::Report));
        assert_eq!(parse_choice("e"), Some(Choice::Edit));
        assert_eq!(parse_choice("cancel"), Some(Choice::Cancel));
        assert_eq!(parse_choice("q"), Some(Choice::Cancel));
    }

    #[test]
    fn a_blank_answer_is_not_a_choice() {
        // Enter must never be enough to post publicly.
        assert_eq!(parse_choice(""), None);
        assert_eq!(parse_choice("   "), None);
        assert_eq!(parse_choice("yes"), None);
    }

    #[test]
    fn the_issue_url_targets_the_mathlingua_repository() {
        let url = issue_url("A bug", "details");

        assert!(url.starts_with(
            "https://github.com/mathlingua/mathlingua/issues/new?title=A%20bug&body=details"
        ));
    }

    #[test]
    fn the_issue_url_escapes_markdown_punctuation() {
        let url = issue_url("t", "```mlg\n[\\set]\n```");

        assert!(!url.contains('`'));
        assert!(!url.contains('\\'));
        assert!(!url.contains('\n'));
        assert!(url.contains("%60%60%60mlg"));
    }

    #[test]
    fn an_oversized_body_is_replaced_rather_than_truncated() {
        let url = issue_url("t", &"x".repeat(MAX_URL_BODY + 1));

        assert!(!url.contains(&"x".repeat(100)));
        assert!(url.contains("mlg%20extract"));
    }

    #[test]
    fn the_preview_shows_the_title_and_body() {
        let preview = preview("A bug", "details");

        assert!(preview.contains("A bug"));
        assert!(preview.contains("details"));
    }
}

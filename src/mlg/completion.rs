//! Autocomplete support for the Mathlingua language server.
//!
//! Three cursor contexts are handled:
//!
//!   * **After / within a top-level group** (e.g. on a blank line under
//!     `Defines:`): suggest the next valid section(s) for that group, in order.
//!   * **On a `. ` argument bullet**: suggest the groups that can start there
//!     (clause groups like `forAll:` / `exists:`, or the item groups valid for
//!     the enclosing section such as `written:` under `Documented:`).
//!   * **While typing a `\`-command** (e.g. `\f`): suggest the command
//!     signatures declared by top-level `[...]` headings, inserted as snippets
//!     whose placeholders are the parameter names (`\function:on{A}:to{B}`).
//!
//! The section orders below mirror the `identify_sections(...)` calls in
//! [`crate::frontend::structural::parser`], which are the language's source of
//! truth for section names, order and optionality. If a group's sections change
//! there, update the corresponding entry here.

use std::collections::HashSet;

/// What a candidate inserts, so the server can pick the matching LSP
/// completion-item kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateKind {
    /// A group / section header (e.g. `when:`).
    Section,
    /// A `\`-command signature (e.g. `\function:on{A}:to{B}`).
    Command,
}

/// A single completion suggestion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionCandidate {
    /// The section / group name without its trailing colon (e.g. `when`), or
    /// the full command signature for command candidates.
    pub label: String,
    /// The text to insert. For command candidates this is LSP snippet syntax
    /// (see [`CompletionCandidate::snippet`]).
    pub insert: String,
    /// Human-readable context shown next to the label.
    pub detail: String,
    /// Whether `insert` uses LSP snippet syntax (tab stops / placeholders).
    pub snippet: bool,
    /// Number of characters immediately before the cursor that the insertion
    /// should replace. `0` inserts at the cursor with no replacement.
    pub replace_chars: usize,
    /// The kind of candidate, used to choose the LSP completion-item kind.
    pub kind: CandidateKind,
}

/// A command signature harvested from a top-level `[...]` heading, e.g.
/// `\function:on{A}:to{B}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    /// The signature text exactly as written between the brackets.
    pub text: String,
    /// The group label on the line below the heading (`Describes`, `Defines`,
    /// ...), shown as completion detail.
    pub kind: Option<String>,
}

/// One section within a group: its name and whether it is required.
type Section = (&'static str, bool);

/// A top-level group: the head section label plus the ordered section list
/// (including the head as the first, required, entry).
struct GroupSpec {
    head: &'static str,
    sections: &'static [Section],
}

// Shared tail present on every definition/theorem-like group.
// (Kept inline in each spec for clarity rather than concatenated at runtime.)

const GROUPS: &[GroupSpec] = &[
    GroupSpec {
        head: "Title",
        sections: &[("Title", true), ("Id", false)],
    },
    GroupSpec {
        head: "SectionTitle",
        sections: &[("SectionTitle", true), ("Id", false)],
    },
    GroupSpec {
        head: "SubsectionTitle",
        sections: &[("SubsectionTitle", true), ("Id", false)],
    },
    GroupSpec {
        head: "Text",
        sections: &[("Text", true), ("Id", false)],
    },
    GroupSpec {
        head: "Writing",
        sections: &[("Writing", true), ("Id", false)],
    },
    GroupSpec {
        head: "Describes",
        sections: &[
            ("Describes", true),
            ("using", false),
            ("when", false),
            ("extends", false),
            ("specifies", false),
            ("satisfies", false),
            ("Requires", false),
            ("Enables", false),
            ("Justified", false),
            ("Documented", false),
            ("Aliases", false),
            ("References", false),
            ("Metadata", false),
            ("Id", false),
        ],
    },
    GroupSpec {
        head: "Defines",
        sections: &[
            ("Defines", true),
            ("using", false),
            ("when", false),
            ("expresses", false),
            ("Requires", false),
            ("Enables", false),
            ("Justified", false),
            ("Documented", false),
            ("Aliases", false),
            ("References", false),
            ("Metadata", false),
            ("Id", false),
        ],
    },
    GroupSpec {
        head: "Refines",
        sections: &[
            ("Refines", true),
            ("using", false),
            ("when", false),
            ("extends", false),
            ("satisfies", false),
            ("Requires", false),
            ("Enables", false),
            ("Justified", false),
            ("Documented", false),
            ("Aliases", false),
            ("References", false),
            ("Metadata", false),
            ("Id", false),
        ],
    },
    GroupSpec {
        head: "States",
        sections: &[
            ("States", true),
            ("using", false),
            ("when", false),
            ("that", true),
            ("Requires", false),
            ("Enables", false),
            ("Justified", false),
            ("Documented", false),
            ("Aliases", false),
            ("References", false),
            ("Metadata", false),
            ("Id", false),
        ],
    },
    GroupSpec {
        head: "Axiom",
        sections: THEOREM_LIKE_AXIOM,
    },
    GroupSpec {
        head: "Theorem",
        sections: THEOREM_LIKE_THEOREM,
    },
    GroupSpec {
        head: "Lemma",
        sections: THEOREM_LIKE_LEMMA,
    },
    GroupSpec {
        head: "Conjecture",
        sections: THEOREM_LIKE_CONJECTURE,
    },
    GroupSpec {
        head: "Corollary",
        sections: &[
            ("Corollary", true),
            ("of", true),
            ("given", false),
            ("where", false),
            ("then", true),
            ("iff", false),
            ("Justified", false),
            ("Documented", false),
            ("Aliases", false),
            ("References", false),
            ("Metadata", false),
            ("Id", false),
        ],
    },
    GroupSpec {
        head: "Disambiguates",
        sections: &[
            ("Disambiguates", true),
            ("when", false),
            ("to", false),
            ("else", false),
            ("Justified", false),
            ("Documented", false),
            ("Aliases", false),
            ("References", false),
            ("Metadata", false),
            ("Id", false),
        ],
    },
    GroupSpec {
        head: "Person",
        sections: &[("Person", true), ("biography", false), ("Id", false)],
    },
    GroupSpec {
        head: "Resource",
        sections: &[("Resource", true), ("Id", false)],
    },
    GroupSpec {
        head: "Specify",
        sections: &[("Specify", true), ("Id", false)],
    },
    GroupSpec {
        head: "Relation",
        sections: &[
            ("Relation", true),
            ("using", false),
            ("between", true),
            ("and", true),
            ("when", false),
            ("means", false),
            ("Justified", false),
            ("Documented", false),
            ("Aliases", false),
            ("References", false),
            ("Metadata", false),
            ("Id", false),
        ],
    },
    GroupSpec {
        head: "Equivalent",
        sections: &[
            ("Equivalent", true),
            ("using", false),
            ("when", false),
            ("to", true),
            ("Justified", false),
            ("Documented", false),
            ("References", false),
            ("Id", false),
        ],
    },
    GroupSpec {
        head: "Topic",
        sections: &[
            ("Topic", true),
            ("within", false),
            ("Related", false),
            ("Documented", false),
            ("Id", false),
        ],
    },
];

// `Axiom`/`Theorem`/`Lemma`/`Conjecture` share one shape (see
// `parse_argument_theorem_like`) differing only in the head label.
const THEOREM_LIKE_AXIOM: &[Section] = &[
    ("Axiom", true),
    ("given", false),
    ("where", false),
    ("then", true),
    ("iff", false),
    ("Justified", false),
    ("Documented", false),
    ("Aliases", false),
    ("References", false),
    ("Metadata", false),
    ("Id", false),
];
const THEOREM_LIKE_THEOREM: &[Section] = &[
    ("Theorem", true),
    ("given", false),
    ("where", false),
    ("then", true),
    ("iff", false),
    ("Justified", false),
    ("Documented", false),
    ("Aliases", false),
    ("References", false),
    ("Metadata", false),
    ("Id", false),
];
const THEOREM_LIKE_LEMMA: &[Section] = &[
    ("Lemma", true),
    ("given", false),
    ("where", false),
    ("then", true),
    ("iff", false),
    ("Justified", false),
    ("Documented", false),
    ("Aliases", false),
    ("References", false),
    ("Metadata", false),
    ("Id", false),
];
const THEOREM_LIKE_CONJECTURE: &[Section] = &[
    ("Conjecture", true),
    ("given", false),
    ("where", false),
    ("then", true),
    ("iff", false),
    ("Justified", false),
    ("Documented", false),
    ("Aliases", false),
    ("References", false),
    ("Metadata", false),
    ("Id", false),
];

/// Clause groups that may start inside a clause-valued section (`when:`,
/// `then:`, `satisfies:`, ...). Mirrors the clause dispatcher in the parser.
const CLAUSE_STARTERS: &[&str] = &[
    "not",
    "allOf",
    "anyOf",
    "oneOf",
    "exists",
    "existsUnique",
    "forAll",
    "if",
    "have",
    "equivalently",
    "piecewise",
    "given",
];

/// Sections whose bullets contain clauses, so a bullet there can start a clause
/// group.
const CLAUSE_SECTIONS: &[&str] = &[
    "when",
    "then",
    "where",
    "have",
    "iff",
    "satisfies",
    "suchThat",
    "that",
    "means",
    "expresses",
];

/// For sections whose bullets contain typed item groups, the group labels that
/// can start there. Mirrors the per-section item dispatchers in the parser.
fn item_starters(section: &str) -> Option<&'static [&'static str]> {
    Some(match section {
        "Documented" => &[
            "written",
            "called",
            "adjective",
            "writing",
            "overview",
            "description",
            "related",
            "discoverer",
        ],
        "Enables" => &["capability", "from", "relation"],
        "Requires" => &["capability", "definition"],
        "Justified" => &["label", "by"],
        "Metadata" => &["id", "version"],
        "Aliases" => &["alias"],
        "Specify" => &["positive", "negative", "zero"],
        _ => return None,
    })
}

/// Section orders for the nested groups that appear on argument bullets:
/// clause groups (`forAll:`, `exists:`, ...) and typed item groups (`from:`,
/// `relation:`, ...). Mirrors their `identify_sections(...)` calls in the
/// parser. Single-section groups are omitted (they have no "next section").
const NESTED_GROUPS: &[(&str, &[Section])] = &[
    // clause groups
    ("exists", &[("exists", true), ("suchThat", false)]),
    (
        "existsUnique",
        &[("existsUnique", true), ("suchThat", false)],
    ),
    (
        "forAll",
        &[("forAll", true), ("where", false), ("then", true)],
    ),
    ("if", &[("if", true), ("then", true)]),
    ("have", &[("have", true), ("iff", true)]),
    (
        "piecewise",
        &[
            ("piecewise", true),
            ("if", true),
            ("then", true),
            ("else", false),
        ],
    ),
    (
        "given",
        &[("given", true), ("where", false), ("then", true)],
    ),
    // item groups
    ("alias", &[("alias", true), ("written", false)]),
    ("capability", &[("capability", true), ("written", false)]),
    (
        "from",
        &[
            ("from", true),
            ("capability", false),
            ("as", false),
            ("written", false),
        ],
    ),
    (
        "relation",
        &[
            ("relation", true),
            ("to", true),
            ("when", false),
            ("means", false),
            ("represents", false),
            ("by", false),
        ],
    ),
    ("called", &[("called", true), ("written", false)]),
    ("writing", &[("writing", true), ("as", true)]),
    (
        "label",
        &[("label", true), ("by", false), ("comment", false)],
    ),
    ("by", &[("by", true), ("comment", false)]),
    ("zero", &[("zero", true), ("is", true)]),
];

fn group_spec(head: &str) -> Option<&'static GroupSpec> {
    GROUPS.iter().find(|g| g.head == head)
}

/// The ordered section list for any group head, top-level or nested.
fn sections_for(head: &str) -> Option<&'static [Section]> {
    if let Some(g) = group_spec(head) {
        return Some(g.sections);
    }
    NESTED_GROUPS
        .iter()
        .find(|(h, _)| *h == head)
        .map(|(_, s)| *s)
}

/// All top-level group head labels (used when starting a fresh item).
fn top_level_heads() -> impl Iterator<Item = &'static str> {
    GROUPS.iter().map(|g| g.head)
}

/// Compute completions for `text` at the zero-based `line` / `character`
/// (character counted in Unicode scalar values, which matches LSP offsets for
/// the ASCII section syntax handled here).
pub fn complete_with_signatures(
    text: &str,
    line: usize,
    character: usize,
    signatures: &[Signature],
) -> Vec<CompletionCandidate> {
    let lines: Vec<&str> = text.split('\n').collect();
    if line >= lines.len() {
        return Vec::new();
    }
    let cur = lines[line];
    let before: String = cur.chars().take(character).collect();

    if let Some(items) = command_completions(&before, signatures) {
        return items;
    }
    if is_bullet_prefix(&before) {
        return bullet_completions(&lines, line, &before);
    }
    section_completions(&lines, line, &before)
}

/// Harvest the command signatures declared by top-level `[...]` headings in
/// `text`. A heading is a line whose trimmed form starts with `[` and ends with
/// `]`; only those whose body starts with `\` (command signatures) are kept.
pub fn collect_signatures(text: &str) -> Vec<Signature> {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut signatures = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.len() < 2 || !trimmed.starts_with('[') || !trimmed.ends_with(']') {
            continue;
        }
        let inner = trimmed[1..trimmed.len() - 1].trim();
        if !inner.starts_with('\\') {
            continue;
        }
        signatures.push(Signature {
            text: inner.to_string(),
            kind: heading_kind(&lines, index),
        });
    }
    signatures
}

/// The group label following a heading — the section name on the next line
/// (e.g. `Describes` for `Describes: R`), or `None` if it is blank/absent.
fn heading_kind(lines: &[&str], heading: usize) -> Option<String> {
    let next = lines.get(heading + 1)?;
    if next.trim().is_empty() {
        return None;
    }
    section_header(next).map(|(_, label)| label.to_string())
}

/// If the text just before the cursor is a `\`-command being typed, return the
/// matching command completions; `None` when the cursor is not in a command
/// context (so section / bullet completion can take over).
fn command_completions(before: &str, signatures: &[Signature]) -> Option<Vec<CompletionCandidate>> {
    let typed = command_prefix(before)?;
    let replace_chars = typed.chars().count();

    let mut seen = HashSet::new();
    let mut candidates = Vec::new();
    for signature in signatures {
        if !signature.text.starts_with(typed) || !seen.insert(signature.text.as_str()) {
            continue;
        }
        candidates.push(CompletionCandidate {
            label: signature.text.clone(),
            insert: signature_to_snippet(&signature.text),
            detail: signature
                .kind
                .clone()
                .unwrap_or_else(|| "command".to_string()),
            snippet: true,
            replace_chars,
            kind: CandidateKind::Command,
        });
    }
    Some(candidates)
}

/// The `\`-command fragment ending at the cursor, including the leading `\`, or
/// `None` if the run of characters before the cursor is not a command being
/// typed (no `\`, or interrupted by whitespace / an argument delimiter).
fn command_prefix(before: &str) -> Option<&str> {
    for (index, ch) in before.char_indices().rev() {
        if ch == '\\' {
            return Some(&before[index..]);
        }
        if !is_command_char(ch) {
            return None;
        }
    }
    None
}

/// Characters that may appear in a command signature after the leading `\`:
/// name parts, `:` separators, `.` chains, `?` optional markers, and the
/// parentheses of a refined prefix such as `\(injective)::...`.
fn is_command_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, ':' | '.' | '_' | '?' | '(' | ')')
}

/// Turn a signature into an LSP snippet: each top-level `{...}` / `(...)`
/// argument group's contents become a numbered placeholder, so
/// `\function:on{A}:to{B}` yields `\function:on{${1:A}}:to{${2:B}}`.
fn signature_to_snippet(signature: &str) -> String {
    let mut out = String::new();
    let mut placeholder = 1u32;
    let mut chars = signature.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        let close = match ch {
            '{' => Some('}'),
            '(' => Some(')'),
            _ => None,
        };
        if let (Some(close_ch), Some(end)) = (close, matching_delim(signature, index)) {
            let interior = &signature[index + ch.len_utf8()..end];
            out.push(ch);
            if interior.is_empty() {
                out.push_str(&format!("${{{placeholder}}}"));
            } else {
                out.push_str(&format!("${{{placeholder}:{}}}", escape_snippet(interior)));
            }
            out.push(close_ch);
            placeholder += 1;
            while chars.peek().is_some_and(|(next, _)| *next <= end) {
                chars.next();
            }
            continue;
        }
        push_snippet_literal(&mut out, ch);
    }
    out
}

/// Byte index of the delimiter matching the `{`/`(` at byte index `open`,
/// accounting for nesting of the same delimiter; `None` if unbalanced.
fn matching_delim(text: &str, open: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let opener = bytes[open];
    let closer = match opener {
        b'{' => b'}',
        b'(' => b')',
        _ => return None,
    };
    let mut depth = 0usize;
    for (index, &byte) in bytes.iter().enumerate().skip(open) {
        if byte == opener {
            depth += 1;
        } else if byte == closer {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

/// Append `ch` to a snippet as a literal, escaping the snippet metacharacters
/// `\`, `$` and `}` so they are not interpreted as tab stops / terminators.
fn push_snippet_literal(out: &mut String, ch: char) {
    if matches!(ch, '\\' | '$' | '}') {
        out.push('\\');
    }
    out.push(ch);
}

/// Escape the snippet metacharacters in placeholder text.
fn escape_snippet(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        push_snippet_literal(&mut out, ch);
    }
    out
}

/// True when everything before the cursor is (indented) `.` optionally
/// followed by an identifier prefix — i.e. an argument bullet being started.
fn is_bullet_prefix(before: &str) -> bool {
    let t = before.trim_start();
    let Some(rest) = t.strip_prefix('.') else {
        return false;
    };
    let rest = rest.trim_start();
    !rest.contains(char::is_whitespace)
        && rest.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn indent_of(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ').count()
}

/// Whether a line is an argument bullet (`. ` marker, possibly indented).
fn is_bullet(line: &str) -> bool {
    let t = line.trim_start();
    t == "." || t.starts_with(". ")
}

/// Parse a leading `name:` token, returning `name`.
fn parse_labeled(s: &str) -> Option<&str> {
    let end = s.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))?;
    if end == 0 || s.as_bytes().get(end) != Some(&b':') {
        return None;
    }
    Some(&s[..end])
}

/// A non-bullet section header like `when:` — returns (indent, label).
fn section_header(line: &str) -> Option<(usize, &str)> {
    if is_bullet(line) {
        return None;
    }
    parse_labeled(line.trim_start()).map(|label| (indent_of(line), label))
}

/// The head of an argument bullet like `. forAll: x` — returns (bullet indent,
/// label). The bullet's sibling sections live at `indent + 2`.
fn bullet_head(line: &str) -> Option<(usize, &str)> {
    if !is_bullet(line) {
        return None;
    }
    let rest = line.trim_start().strip_prefix(". ")?.trim_start();
    parse_labeled(rest).map(|label| (indent_of(line), label))
}

fn candidate(label: &str, detail: String) -> CompletionCandidate {
    CompletionCandidate {
        label: label.to_string(),
        insert: format!("{label}:"),
        detail,
        snippet: false,
        replace_chars: 0,
        kind: CandidateKind::Section,
    }
}

/// Suggest the groups that can start on an argument bullet, based on the
/// enclosing section.
fn bullet_completions(lines: &[&str], line: usize, before: &str) -> Vec<CompletionCandidate> {
    let bullet_indent = indent_of(before);
    let prefix = before.trim_start().trim_start_matches('.').trim_start();

    // Find the section that owns this bullet: the nearest non-bullet header
    // above at the same-or-shallower indent (stopping at an item boundary).
    let mut owner: Option<&str> = None;
    for i in (0..line).rev() {
        let l = lines[i];
        if l.trim().is_empty() {
            break;
        }
        if let Some((indent, label)) = section_header(l) {
            if indent <= bullet_indent {
                owner = Some(label);
                break;
            }
        }
    }

    let (starters, kind): (Vec<&str>, &str) = match owner {
        Some(section) if item_starters(section).is_some() => {
            (item_starters(section).unwrap().to_vec(), section)
        }
        Some(section) if CLAUSE_SECTIONS.contains(&section) => (CLAUSE_STARTERS.to_vec(), "clause"),
        // Unknown / non-clause section: clause groups are the common default.
        _ => (CLAUSE_STARTERS.to_vec(), "clause"),
    };

    starters
        .into_iter()
        .filter(|s| s.starts_with(prefix))
        .map(|s| {
            let detail = if kind == "clause" {
                "clause group".to_string()
            } else {
                format!("{kind} group")
            };
            candidate(s, detail)
        })
        .collect()
}

/// Suggest the next section(s) for the group enclosing the cursor, at any
/// indentation level.
fn section_completions(lines: &[&str], line: usize, before: &str) -> Vec<CompletionCandidate> {
    let indent = indent_of(before);
    let prefix = before.trim_start();
    // Only a bare indent or a partial section name is a section context.
    if !prefix
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Vec::new();
    }

    let (head, present_above, present_all) = if indent == 0 {
        gather_top_level(lines, line)
    } else {
        gather_nested(lines, line, indent)
    };

    // No recognised group. At column 0 this is the start of a new item, so
    // offer the top-level group heads; nested contexts stay silent.
    let Some(head) = head.filter(|h| sections_for(h).is_some()) else {
        if indent == 0 {
            return top_level_heads()
                .filter(|h| h.starts_with(prefix))
                .map(|h| candidate(h, "group".to_string()))
                .collect();
        }
        return Vec::new();
    };
    let sections = sections_for(head).unwrap();

    // Anchor on the last present section above the cursor.
    let last_idx = present_above
        .iter()
        .filter_map(|p| sections.iter().position(|(n, _)| n == p))
        .max();

    sections
        .iter()
        .enumerate()
        .filter(|(idx, (name, _))| match last_idx {
            Some(last) => *idx > last && !present_all.contains(name),
            None => !present_all.contains(name),
        })
        .filter(|(_, (name, _))| name.starts_with(prefix))
        .map(|(_, (name, required))| {
            let req = if *required { "required" } else { "optional" };
            candidate(name, format!("{req} section of {head}"))
        })
        .collect()
}

/// Gather the enclosing top-level (column 0) group: its head and the section
/// labels present above / anywhere in the item (bounded by blank lines).
fn gather_top_level<'a>(
    lines: &[&'a str],
    line: usize,
) -> (Option<&'a str>, Vec<&'a str>, Vec<&'a str>) {
    let mut start = line;
    while start > 0 && !lines[start - 1].trim().is_empty() {
        start -= 1;
    }
    let mut present_all = Vec::new();
    let mut present_above = Vec::new();
    let mut head = None;
    for (i, l) in lines.iter().enumerate().skip(start) {
        if l.trim().is_empty() {
            if i >= line {
                break;
            }
            continue;
        }
        if let Some((0, label)) = section_header(l) {
            if head.is_none() {
                head = Some(label);
            }
            present_all.push(label);
            if i < line {
                present_above.push(label);
            }
        }
    }
    (head, present_above, present_all)
}

/// Gather the enclosing nested group for a cursor at `indent` (>= 2): its head
/// (on a `. ` bullet at `indent - 2`) and the sibling sections at `indent`.
fn gather_nested<'a>(
    lines: &[&'a str],
    line: usize,
    indent: usize,
) -> (Option<&'a str>, Vec<&'a str>, Vec<&'a str>) {
    let parent = indent.checked_sub(2);
    let mut present_above = Vec::new();
    let mut head = None;
    for i in (0..line).rev() {
        let l = lines[i];
        if l.trim().is_empty() {
            break;
        }
        let ind = indent_of(l);
        if ind > indent {
            continue; // deeper nested content
        }
        if ind == indent {
            if is_bullet(l) {
                continue; // content inside the current nested section
            }
            if let Some((_, label)) = section_header(l) {
                present_above.push(label);
            }
            continue;
        }
        if Some(ind) == parent {
            if let Some((_, label)) = bullet_head(l) {
                head = Some(label);
                present_above.push(label); // the head is the first section
            }
            break;
        }
        break; // shallower line: group boundary
    }

    // Sections below the cursor (for de-duplication) until the group ends.
    let mut present_all = present_above.clone();
    for l in lines.iter().skip(line + 1) {
        if l.trim().is_empty() {
            break;
        }
        let ind = indent_of(l);
        if ind > indent {
            continue;
        }
        if ind == indent {
            if is_bullet(l) {
                continue;
            }
            if let Some((_, label)) = section_header(l) {
                present_all.push(label);
            }
            continue;
        }
        break;
    }
    (head, present_above, present_all)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Convenience wrapper that sources command signatures from `text` itself,
    /// as the single-document case does.
    fn complete(text: &str, line: usize, character: usize) -> Vec<CompletionCandidate> {
        complete_with_signatures(text, line, character, &collect_signatures(text))
    }

    fn labels(cands: &[CompletionCandidate]) -> Vec<String> {
        cands.iter().map(|c| c.label.clone()).collect()
    }

    #[test]
    fn next_section_after_defines_head() {
        let text = "[\\f]\nDefines: f(x)\n\nId: \"x\"";
        // cursor on the blank line 2 (0-based), column 0
        let got = labels(&complete(text, 2, 0));
        assert_eq!(got.first().map(String::as_str), Some("using"));
        assert!(got.contains(&"when".to_string()));
        assert!(got.contains(&"Documented".to_string()));
        // the head itself is not re-offered
        assert!(!got.contains(&"Defines".to_string()));
    }

    #[test]
    fn next_section_skips_already_present() {
        let text = "Defines: f\nwhen: x\n\n";
        let got = labels(&complete(text, 2, 0));
        assert!(!got.contains(&"using".to_string())); // using is before when
        assert!(!got.contains(&"when".to_string())); // already present
        assert!(got.contains(&"expresses".to_string()));
        assert!(got.contains(&"Id".to_string()));
    }

    #[test]
    fn theorem_requires_then() {
        let text = "Theorem:\ngiven: x\n\n";
        let got = labels(&complete(text, 2, 0));
        assert!(got.contains(&"then".to_string()));
        assert!(got.contains(&"where".to_string()));
    }

    #[test]
    fn bullet_in_clause_section_suggests_clause_groups() {
        let text = "Theorem:\nthen:\n. ";
        let got = labels(&complete(text, 2, 2));
        assert!(got.contains(&"forAll".to_string()));
        assert!(got.contains(&"exists".to_string()));
        assert!(got.contains(&"if".to_string()));
        assert!(got.contains(&"have".to_string()));
        assert!(!got.contains(&"iff".to_string()));
    }

    #[test]
    fn bullet_in_documented_suggests_item_groups() {
        let text = "Defines: f\nDocumented:\n. ";
        let got = labels(&complete(text, 2, 2));
        assert!(got.contains(&"written".to_string()));
        assert!(got.contains(&"called".to_string()));
        assert!(!got.contains(&"forAll".to_string()));
    }

    #[test]
    fn bullet_prefix_filters() {
        let text = "Theorem:\nthen:\n. fo";
        let got = labels(&complete(text, 2, 4));
        assert_eq!(got, vec!["forAll".to_string()]);
    }

    #[test]
    fn nested_bullet_uses_inner_section() {
        // `. existsUnique` under an indented `then:` should still see clauses.
        let text = "Theorem:\nthen:\n. forAll: x\n  then:\n  . ";
        let got = labels(&complete(text, 4, 4));
        assert!(got.contains(&"exists".to_string()));
        assert!(got.contains(&"forAll".to_string()));
    }

    #[test]
    fn next_section_inside_forall_clause() {
        // Cursor indented under `. forAll:` should offer forAll's remaining
        // sections (`where`, `then`).
        let text = "Theorem:\nthen:\n. forAll: x\n  ";
        let got = labels(&complete(text, 3, 2));
        assert!(got.contains(&"where".to_string()));
        assert!(got.contains(&"then".to_string()));
        assert!(!got.contains(&"forAll".to_string())); // head not re-offered
    }

    #[test]
    fn next_section_inside_forall_skips_present() {
        let text = "Theorem:\nthen:\n. forAll: x\n  where: y\n  ";
        let got = labels(&complete(text, 4, 2));
        assert_eq!(got, vec!["then".to_string()]);
    }

    #[test]
    fn next_section_inside_have_clause() {
        let text = "Theorem:\nthen:\n. have:\n  . x = x\n  ";
        let got = labels(&complete(text, 4, 2));
        assert_eq!(got, vec!["iff".to_string()]);
    }

    #[test]
    fn next_section_inside_deep_nested_group() {
        // existsUnique nested two levels deep -> offer `suchThat`.
        let text = "Theorem:\nthen:\n. forAll: x\n  then:\n  . existsUnique: y\n    ";
        let got = labels(&complete(text, 5, 4));
        assert_eq!(got, vec!["suchThat".to_string()]);
    }

    #[test]
    fn next_section_inside_documented_item_group() {
        // A `. from:` item group under Enables offers its `capability`/`as`/...
        let text = "Defines: f\nEnables:\n. from: $x\n  ";
        let got = labels(&complete(text, 3, 2));
        assert!(got.contains(&"capability".to_string()));
        assert!(got.contains(&"as".to_string()));
        assert!(got.contains(&"written".to_string()));
    }

    #[test]
    fn nested_prefix_filter() {
        let text = "Theorem:\nthen:\n. forAll: x\n  th";
        let got = labels(&complete(text, 3, 4));
        assert_eq!(got, vec!["then".to_string()]);
    }

    #[test]
    fn empty_item_offers_group_heads() {
        let text = "";
        let got = labels(&complete(text, 0, 0));
        assert!(got.contains(&"Defines".to_string()));
        assert!(got.contains(&"Theorem".to_string()));
        assert!(got.contains(&"Text".to_string()));
    }

    #[test]
    fn group_head_prefix_filter() {
        let text = "Def";
        let got = labels(&complete(text, 0, 3));
        assert_eq!(got, vec!["Defines".to_string()]);
    }

    #[test]
    fn command_completion_offers_signature_with_placeholders() {
        // A heading declares `\function:on{A}:to{B}`; typing `\f` offers it as a
        // snippet whose placeholders are the parameter names A and B.
        let text = "[\\function:on{A}:to{B}]\nDescribes: f\n\n\\f";
        let got = complete(text, 3, 2);
        assert_eq!(got.len(), 1);
        let candidate = &got[0];
        assert_eq!(candidate.label, "\\function:on{A}:to{B}");
        // The leading `\` is escaped to `\\` for the snippet engine; `{A}`/`{B}`
        // become numbered placeholders.
        assert_eq!(candidate.insert, "\\\\function:on{${1:A}}:to{${2:B}}");
        assert!(candidate.snippet);
        assert_eq!(candidate.replace_chars, 2);
        assert_eq!(candidate.kind, CandidateKind::Command);
        assert_eq!(candidate.detail, "Describes");
    }

    #[test]
    fn command_completion_prefix_filters() {
        let text = "[\\relation:on{X}]\nDescribes: R\n\
                    [\\function:on{A}:to{B}]\nDescribes: f\n\n\\rel";
        let got = labels(&complete(text, 5, 4));
        assert_eq!(got, vec!["\\relation:on{X}".to_string()]);
    }

    #[test]
    fn command_completion_backslash_offers_all() {
        let text = "[\\domain{R}]\nDefines: D\n[\\range{R}]\nDefines: N\n\n\\";
        let mut got = labels(&complete(text, 5, 1));
        got.sort();
        assert_eq!(
            got,
            vec!["\\domain{R}".to_string(), "\\range{R}".to_string()]
        );
    }

    #[test]
    fn command_completion_dedupes_repeated_signatures() {
        let text = "[\\set]\nDescribes: X\n[\\set]\nStates:\n\n\\se";
        assert_eq!(complete(text, 5, 3).len(), 1);
    }

    #[test]
    fn command_context_yields_no_section_completions() {
        // In a `\`-context with no matching signature nothing is offered, rather
        // than falling through to section suggestions.
        let text = "Defines: f\n\\zz";
        assert!(complete(text, 1, 3).is_empty());
    }

    #[test]
    fn signature_snippet_conversion() {
        assert_eq!(signature_to_snippet("\\domain{R}"), "\\\\domain{${1:R}}");
        assert_eq!(signature_to_snippet("\\emptyset"), "\\\\emptyset");
        assert_eq!(
            signature_to_snippet("\\restriction:of{f}:to{C}"),
            "\\\\restriction:of{${1:f}}:to{${2:C}}"
        );
    }

    #[test]
    fn collect_signatures_reads_headings() {
        let text = "[\\domain{R}]\nDefines: D is \\set\n\nText: hi";
        let signatures = collect_signatures(text);
        assert_eq!(signatures.len(), 1);
        assert_eq!(signatures[0].text, "\\domain{R}");
        assert_eq!(signatures[0].kind.as_deref(), Some("Defines"));
    }
}

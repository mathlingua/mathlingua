use super::*;

pub(super) fn render_called_template(
    template: &str,
    substitutions: &HashMap<String, String>,
) -> String {
    let chars = template.chars().collect::<Vec<_>>();
    let mut result = String::new();
    let mut segment = String::new();
    let mut in_math = false;
    let mut index = 0;

    while index < chars.len() {
        if chars[index] == '$' {
            flush_called_segment(&mut result, &mut segment, in_math, substitutions);
            in_math = !in_math;
            index += 1;
            continue;
        }

        if let Some(conditional) = parse_template_conditional(&chars, index) {
            flush_called_segment(&mut result, &mut segment, in_math, substitutions);
            if let Some(branch) = selected_conditional_branch(&conditional, substitutions) {
                if in_math {
                    result.push_str(&substitute_math_template(branch, substitutions));
                } else {
                    result.push_str(&render_called_template(branch, substitutions));
                }
            }
            index = conditional.end;
            continue;
        }

        segment.push(chars[index]);
        index += 1;
    }

    flush_called_segment(&mut result, &mut segment, in_math, substitutions);

    result
}

pub(super) fn render_called_display_template(template: &str) -> String {
    let chars = template.chars().collect::<Vec<_>>();
    let mut result = String::new();
    let mut segment = String::new();
    let mut in_math = false;
    let mut index = 0;

    while index < chars.len() {
        if chars[index] == '$' {
            flush_called_display_segment(&mut result, &mut segment, in_math);
            in_math = !in_math;
            index += 1;
            continue;
        }

        if let Some(conditional) = parse_template_conditional(&chars, index) {
            flush_called_display_segment(&mut result, &mut segment, in_math);
            if let Some(branch) = selected_conditional_branch(&conditional, &HashMap::new()) {
                if in_math {
                    result.push_str(&render_written_display_template(branch));
                } else {
                    result.push_str(&render_called_display_template(branch));
                }
            }
            index = conditional.end;
            continue;
        }

        segment.push(chars[index]);
        index += 1;
    }

    flush_called_display_segment(&mut result, &mut segment, in_math);

    result
}

pub(super) fn join_called_latex_parts(parts: Vec<String>) -> String {
    parts
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\\textrm{ }")
}

fn flush_called_segment(
    result: &mut String,
    segment: &mut String,
    in_math: bool,
    substitutions: &HashMap<String, String>,
) {
    if segment.is_empty() {
        return;
    }

    if in_math {
        result.push_str(&substitute_math_template(segment, substitutions));
    } else {
        result.push_str(&substitute_called_text_segment(segment, substitutions));
    }
    segment.clear();
}

fn flush_called_display_segment(result: &mut String, segment: &mut String, in_math: bool) {
    if segment.is_empty() {
        return;
    }

    if in_math {
        result.push_str(&render_written_display_template(segment));
    } else {
        result.push_str(&substitute_called_display_text_segment(segment));
    }
    segment.clear();
}

fn substitute_called_text_segment(
    segment: &str,
    substitutions: &HashMap<String, String>,
) -> String {
    let mut result = String::new();
    let mut text = String::new();
    let chars = segment.chars().collect::<Vec<_>>();
    let mut index = 0;

    while index < chars.len() {
        match scan_placeholder(&chars, index) {
            Some(PlaceholderScan::Placeholder(placeholder)) => {
                if let Some(value) = substitutions.get(&placeholder.name) {
                    flush_called_text(&mut result, &mut text);
                    result.push_str(&apply_paren_modifier(value, placeholder.modifier));
                } else {
                    text.push_str(&placeholder.name);
                }
                index = placeholder.end;
            }
            Some(PlaceholderScan::LiteralName { end }) => {
                text.extend(chars[index..end].iter());
                index = end;
            }
            None => {
                text.push(chars[index]);
                index += 1;
            }
        }
    }

    flush_called_text(&mut result, &mut text);

    result
}

fn substitute_called_display_text_segment(segment: &str) -> String {
    let mut result = String::new();
    let mut text = String::new();
    let chars = segment.chars().collect::<Vec<_>>();
    let mut index = 0;

    while index < chars.len() {
        match scan_placeholder(&chars, index) {
            // With no value to substitute there is nothing to parenthesize, so a
            // modifier shows the same bare name that `X?` does.
            Some(PlaceholderScan::Placeholder(placeholder)) => {
                flush_called_text(&mut result, &mut text);
                result.push_str(&render_template_placeholder_name(&placeholder.name));
                index = placeholder.end;
            }
            Some(PlaceholderScan::LiteralName { end }) => {
                text.extend(chars[index..end].iter());
                index = end;
            }
            None => {
                text.push(chars[index]);
                index += 1;
            }
        }
    }

    flush_called_text(&mut result, &mut text);

    result
}

fn flush_called_text(result: &mut String, text: &mut String) {
    if text.is_empty() {
        return;
    }

    result.push_str(&format!("\\textrm{{{}}}", escape_latex_text(text)));
    text.clear();
}

pub(super) fn substitute_math_template(
    template: &str,
    substitutions: &HashMap<String, String>,
) -> String {
    let mut result = String::new();
    let chars = template.chars().collect::<Vec<_>>();
    let mut index = 0;

    while index < chars.len() {
        if let Some(conditional) = parse_template_conditional(&chars, index) {
            if let Some(branch) = selected_conditional_branch(&conditional, substitutions) {
                result.push_str(&substitute_math_template(branch, substitutions));
            }
            index = conditional.end;
            continue;
        }

        match scan_placeholder(&chars, index) {
            Some(PlaceholderScan::Placeholder(placeholder)) => {
                match substitutions.get(&placeholder.name) {
                    Some(value) => {
                        result.push_str(&apply_paren_modifier(value, placeholder.modifier))
                    }
                    None => result.push_str(&placeholder.name),
                }
                index = placeholder.end;
            }
            Some(PlaceholderScan::LiteralName { end }) => {
                result.extend(chars[index..end].iter());
                index = end;
            }
            None => {
                result.push(chars[index]);
                index += 1;
            }
        }
    }

    result
}

pub(super) fn render_written_display_template(template: &str) -> String {
    let mut result = String::new();
    let chars = template.chars().collect::<Vec<_>>();
    let mut index = 0;

    while index < chars.len() {
        if let Some(conditional) = parse_template_conditional(&chars, index) {
            if let Some(branch) = selected_conditional_branch(&conditional, &HashMap::new()) {
                result.push_str(&render_written_display_template(branch));
            }
            index = conditional.end;
            continue;
        }

        match scan_placeholder(&chars, index) {
            // As in the called display template, a modifier has no value to act on
            // here and so renders the same bare name that `X?` does.
            Some(PlaceholderScan::Placeholder(placeholder)) => {
                result.push_str(&render_template_placeholder_name(&placeholder.name));
                index = placeholder.end;
            }
            Some(PlaceholderScan::LiteralName { end }) => {
                result.extend(chars[index..end].iter());
                index = end;
            }
            None => {
                result.push(chars[index]);
                index += 1;
            }
        }
    }

    result
}

fn render_template_placeholder_name(name: &str) -> String {
    let registry = RenderRegistry::default();
    let trimmed = name.trim_end_matches('_');
    if trimmed.is_empty() {
        escape_math_identifier(name, &registry)
    } else {
        escape_math_identifier(trimmed, &registry)
    }
}

#[derive(Clone, Debug)]
struct TemplateConditional {
    variables: Vec<String>,
    when_present: String,
    when_absent: Option<String>,
    end: usize,
}

fn parse_template_conditional(chars: &[char], start: usize) -> Option<TemplateConditional> {
    if chars.get(start) != Some(&'@') || chars.get(start + 1) != Some(&'[') {
        return None;
    }

    let variables_end = (start + 2..chars.len()).find(|index| chars[*index] == ']')?;
    let variables = parse_conditional_variables(&chars[start + 2..variables_end])?;
    let present_open = variables_end + 1;
    let (when_present, mut end) = parse_template_braced_body(chars, present_open)?;
    let when_absent = if chars.get(end) == Some(&':') && chars.get(end + 1) == Some(&'{') {
        let (body, after_body) = parse_template_braced_body(chars, end + 1)?;
        end = after_body;
        Some(body)
    } else {
        None
    };

    Some(TemplateConditional {
        variables,
        when_present,
        when_absent,
        end,
    })
}

fn parse_conditional_variables(chars: &[char]) -> Option<Vec<String>> {
    let text = chars.iter().collect::<String>();
    let variables = text
        .split(',')
        .map(str::trim)
        .map(str::to_string)
        .collect::<Vec<_>>();

    if variables.is_empty()
        || variables
            .iter()
            .any(|variable| !is_conditional_variable(variable))
    {
        return None;
    }

    Some(variables)
}

fn is_conditional_variable(variable: &str) -> bool {
    let mut chars = variable.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    is_placeholder_start(first) && chars.all(is_placeholder_continue)
}

fn parse_template_braced_body(chars: &[char], open: usize) -> Option<(String, usize)> {
    if chars.get(open) != Some(&'{') {
        return None;
    }

    let mut depth = 1usize;
    let mut index = open + 1;
    while index < chars.len() {
        match chars[index] {
            '\\' => {
                index += 1;
                if index < chars.len() {
                    index += 1;
                }
            }
            '{' => {
                depth += 1;
                index += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((chars[open + 1..index].iter().collect(), index + 1));
                }
                index += 1;
            }
            _ => index += 1,
        }
    }

    None
}

fn selected_conditional_branch<'a>(
    conditional: &'a TemplateConditional,
    substitutions: &HashMap<String, String>,
) -> Option<&'a str> {
    if conditional
        .variables
        .iter()
        .all(|variable| substitutions.contains_key(variable))
    {
        Some(&conditional.when_present)
    } else {
        conditional.when_absent.as_deref()
    }
}

pub(super) fn template_contains_placeholder(template: &str, name: &str) -> bool {
    [
        format!("{name}?"),
        format!("{name}+?"),
        format!("{name}-?"),
    ]
    .iter()
    .any(|needle| template.contains(needle.as_str()))
}

/// The parenthesis handling a placeholder asks for around its substituted value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ParenModifier {
    /// `X?` — the value is substituted exactly as it was rendered.
    Keep,
    /// `X+?` — the value is wrapped in exactly one pair of parentheses, unless it
    /// is a single atom that needs none.
    Ensure,
    /// `X-?` — every pair of parentheses wrapping the value is removed.
    Strip,
}

/// A `NAME?`, `NAME+?`, or `NAME-?` placeholder found in a template.
pub(super) struct TemplatePlaceholder {
    /// The substitution key, which never includes the `+`/`-` modifier.
    pub(super) name: String,
    pub(super) modifier: ParenModifier,
    /// Index just past the closing `?`.
    pub(super) end: usize,
}

/// What a name-like run of characters starting at `start` turned out to be.
pub(super) enum PlaceholderScan {
    /// A complete placeholder.
    Placeholder(TemplatePlaceholder),
    /// A name not followed by `?`, so it is literal text ending at `end`.
    ///
    /// A `+` or `-` after the name is left for the caller to re-read as ordinary
    /// text, so `A-B` and `A - B` keep rendering as they always have.
    LiteralName { end: usize },
}

/// Reads the placeholder, if any, that starts at `start`.
///
/// Returns `None` when `start` does not begin a name, leaving the caller to treat
/// the character as ordinary text.
pub(super) fn scan_placeholder(chars: &[char], start: usize) -> Option<PlaceholderScan> {
    if !is_placeholder_start(*chars.get(start)?) {
        return None;
    }

    let mut index = start + 1;
    while index < chars.len() && is_placeholder_continue(chars[index]) {
        index += 1;
    }
    let name_end = index;

    let modifier = match chars.get(index) {
        Some('+') => ParenModifier::Ensure,
        Some('-') => ParenModifier::Strip,
        _ => ParenModifier::Keep,
    };
    if modifier != ParenModifier::Keep {
        index += 1;
    }

    if chars.get(index) != Some(&'?') {
        return Some(PlaceholderScan::LiteralName { end: name_end });
    }

    Some(PlaceholderScan::Placeholder(TemplatePlaceholder {
        name: chars[start..name_end].iter().collect(),
        modifier,
        end: index + 1,
    }))
}

/// The LaTeX parentheses this renderer emits and recognizes for grouping.
const LEFT_PAREN: &str = "\\left(";
const RIGHT_PAREN: &str = "\\right)";

/// Applies a placeholder's parenthesis modifier to an already-rendered value.
pub(super) fn apply_paren_modifier(value: &str, modifier: ParenModifier) -> String {
    match modifier {
        ParenModifier::Keep => value.to_string(),
        ParenModifier::Strip => strip_wrapping_parens(value).to_string(),
        ParenModifier::Ensure => {
            // Stripping first is what keeps `(1 + 2)` from becoming `((1 + 2))`:
            // the value is reduced to its bare form and then wrapped exactly once.
            let stripped = strip_wrapping_parens(value);
            if is_atomic_latex(stripped) {
                stripped.to_string()
            } else {
                format!("{LEFT_PAREN}{stripped}{RIGHT_PAREN}")
            }
        }
    }
}

/// Removes every pair of parentheses that wraps the whole of `text`.
fn strip_wrapping_parens(text: &str) -> &str {
    let mut current = text.trim();
    while let Some(inner) = strip_one_wrapping_paren(current) {
        current = inner;
    }
    current
}

/// Removes one pair of parentheses when a single pair encloses all of `text`.
///
/// Returns `None` when `text` does not open with a parenthesis, when the opening
/// parenthesis closes before the end (as in `(a) + (b)`, where the leading `(` is
/// not a wrapper around the whole expression), or when the pair is mismatched
/// (`\left(` closed by a bare `)`), since removing half of a pair would produce
/// broken LaTeX.
fn strip_one_wrapping_paren(text: &str) -> Option<&str> {
    let text = text.trim();
    let (open_len, opened_with_left) = if text.starts_with(LEFT_PAREN) {
        (LEFT_PAREN.len(), true)
    } else if text.starts_with('(') {
        (1, false)
    } else {
        return None;
    };

    let mut depth = 1usize;
    let mut index = open_len;

    while index < text.len() {
        let rest = &text[index..];

        if rest.starts_with(LEFT_PAREN) {
            depth += 1;
            index += LEFT_PAREN.len();
            continue;
        }
        if let Some(after) = rest.strip_prefix(RIGHT_PAREN) {
            depth -= 1;
            if depth == 0 {
                return (after.is_empty() && opened_with_left)
                    .then(|| text[open_len..index].trim());
            }
            index += RIGHT_PAREN.len();
            continue;
        }
        if rest.starts_with('(') {
            depth += 1;
            index += 1;
            continue;
        }
        if let Some(after) = rest.strip_prefix(')') {
            depth -= 1;
            if depth == 0 {
                return (after.is_empty() && !opened_with_left)
                    .then(|| text[open_len..index].trim());
            }
            index += 1;
            continue;
        }

        index += escaped_unit_len(rest);
    }

    None
}

/// Whether `text` reads as a single atom that needs no parentheses around it.
///
/// Rendered compound expressions always separate their operands with spaces
/// (`1 + 2`) or commas (`X, Y`), so a value with neither outside of a bracket —
/// `a`, `x_1`, `\emptyset`, `\mathsf{Field}_{V}`, `f(x)` — is treated as atomic.
fn is_atomic_latex(text: &str) -> bool {
    let mut depth = 0usize;
    let mut index = 0usize;

    while index < text.len() {
        let rest = &text[index..];

        if rest.starts_with(LEFT_PAREN) {
            depth += 1;
            index += LEFT_PAREN.len();
            continue;
        }
        if rest.starts_with(RIGHT_PAREN) {
            depth = depth.saturating_sub(1);
            index += RIGHT_PAREN.len();
            continue;
        }

        match rest.chars().next().expect("rest is not empty") {
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => depth = depth.saturating_sub(1),
            ch if depth == 0 && (ch == ',' || ch.is_whitespace()) => return false,
            _ => {}
        }

        index += escaped_unit_len(rest);
    }

    true
}

/// The byte length of the next unit of LaTeX at the start of `rest`.
///
/// A backslash escape such as `\{` counts as a single unit so that its brace is
/// not mistaken for a grouping delimiter.
fn escaped_unit_len(rest: &str) -> usize {
    let mut chars = rest.chars();
    let first = chars.next().expect("rest is not empty");
    if first != '\\' {
        return first.len_utf8();
    }

    first.len_utf8() + chars.next().map_or(0, char::len_utf8)
}

pub(super) fn is_placeholder_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

pub(super) fn is_placeholder_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '.'
}

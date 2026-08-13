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
                    result.push_str(&render_template_placeholder(
                        value,
                        &placeholder,
                        substitutions,
                    ));
                } else {
                    text.push_str(&placeholder.name);
                    text.push_str(&placeholder_notation_source(&placeholder));
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
                result.push_str(&placeholder_notation_source(&placeholder));
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
                    Some(value) => result.push_str(&render_template_placeholder(
                        value,
                        &placeholder,
                        substitutions,
                    )),
                    None => {
                        result.push_str(&placeholder.name);
                        result.push_str(&placeholder_notation_source(&placeholder));
                    }
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
                result.push_str(&placeholder_notation_source(&placeholder));
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

/// Whether a `called:`/`written:` template still renders to content for the given
/// substitutions.
///
/// A template that is, at the top level, a single `@[vars]{…}` conditional with no
/// `:{…}` fallback is treated as **absent** when its variables are unbound — the
/// block "does not exist", so rendering falls back (`written:` → `called:`). This
/// only fires when the conditional spans the whole template: a template with any
/// other content (a prefix, a fallback branch, a second block) always renders.
pub(super) fn template_is_present(template: &str, substitutions: &HashMap<String, String>) -> bool {
    let chars = template.trim().chars().collect::<Vec<_>>();
    let Some(conditional) = parse_template_conditional(&chars, 0) else {
        return true;
    };
    let spans_whole_template = conditional.end == chars.len();
    let variables_unbound = !conditional
        .variables
        .iter()
        .all(|variable| substitutions.contains_key(variable));
    !(spans_whole_template && conditional.when_absent.is_none() && variables_unbound)
}

pub(super) fn template_contains_placeholder(template: &str, name: &str) -> bool {
    [format!("{name}?"), format!("{name}+?"), format!("{name}-?")]
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
    /// Optional variadic prefix/postfix/infix notation following the `?`.
    variadic_notation: Option<VariadicNotation>,
    matrix_notation: Option<MatrixNotation>,
    /// Index just past the closing `?` or its variadic `{...}` suffix.
    pub(super) end: usize,
}

#[derive(Clone, Debug)]
enum VariadicNotation {
    Postfix(String),
    Prefix(String),
    Infix(String),
}

impl VariadicNotation {
    fn source(&self) -> String {
        match self {
            Self::Postfix(text) => format!("{{...{text}}}"),
            Self::Prefix(text) => format!("{{{text}...}}"),
            Self::Infix(text) => format!("{{...{text}...}}"),
        }
    }
}

#[derive(Clone, Debug)]
struct MatrixNotation {
    row_prefix: String,
    row_suffix: String,
    elements: VariadicNotation,
    source: String,
}

impl MatrixNotation {
    fn source(&self) -> String {
        self.source.clone()
    }
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

    let mut end = index + 1;
    let matrix_notation = parse_matrix_notation(chars, end).map(|(notation, after)| {
        end = after;
        notation
    });
    let variadic_notation = if matrix_notation.is_none() {
        parse_variadic_notation(chars, end).map(|(notation, after)| {
            end = after;
            notation
        })
    } else {
        None
    };

    Some(PlaceholderScan::Placeholder(TemplatePlaceholder {
        name: chars[start..name_end].iter().collect(),
        modifier,
        variadic_notation,
        matrix_notation,
        end,
    }))
}

fn parse_matrix_notation(chars: &[char], open: usize) -> Option<(MatrixNotation, usize)> {
    let (body, end) = parse_template_braced_body(chars, open)?;
    let body_chars = body.chars().collect::<Vec<_>>();
    let inner_open = body_chars.iter().position(|ch| *ch == '{')?;
    let (inner_body, inner_end) = parse_template_braced_body(&body_chars, inner_open)?;
    if body_chars[inner_end..].contains(&'{') {
        return None;
    }
    let inner_source = format!("{{{inner_body}}}").chars().collect::<Vec<_>>();
    let (elements, _) = parse_variadic_notation(&inner_source, 0)?;
    let before = body_chars[..inner_open].iter().collect::<String>();
    let after = body_chars[inner_end..].iter().collect::<String>();
    let row_prefix = if before.is_empty() {
        String::new()
    } else {
        before.strip_suffix("...")?.to_owned()
    };
    let row_suffix = if after.is_empty() {
        String::new()
    } else {
        after.strip_prefix("...").unwrap_or(&after).to_owned()
    };
    if before.is_empty() && after.is_empty() {
        return None;
    }
    Some((
        MatrixNotation {
            row_prefix,
            row_suffix,
            elements,
            source: chars[open..end].iter().collect(),
        },
        end,
    ))
}

fn parse_variadic_notation(chars: &[char], open: usize) -> Option<(VariadicNotation, usize)> {
    let (body, end) = parse_template_braced_body(chars, open)?;
    if body.starts_with("...") && body.ends_with("...") && body.len() >= 6 {
        return Some((
            VariadicNotation::Infix(body[3..body.len() - 3].to_owned()),
            end,
        ));
    }
    if let Some(postfix) = body.strip_prefix("...") {
        return Some((VariadicNotation::Postfix(postfix.to_owned()), end));
    }
    if let Some(prefix) = body.strip_suffix("...") {
        return Some((VariadicNotation::Prefix(prefix.to_owned()), end));
    }
    None
}

fn render_template_placeholder(
    value: &str,
    placeholder: &TemplatePlaceholder,
    substitutions: &HashMap<String, String>,
) -> String {
    let variadic_values = variadic_substitution_values(substitutions, &placeholder.name);
    let matrix_values = matrix_substitution_values(substitutions, &placeholder.name);
    let rendered = match (
        &placeholder.matrix_notation,
        &matrix_values,
        &placeholder.variadic_notation,
        &variadic_values,
    ) {
        (Some(notation), Some(rows), _, _) => render_matrix_notation(rows, notation),
        (Some(notation), None, _, _) => format!("{value}{}", notation.source()),
        (_, _, Some(notation), Some(values)) => render_variadic_notation(values, notation),
        (_, _, Some(notation), None) => format!("{value}{}", notation.source()),
        _ => value.to_owned(),
    };
    if placeholder.modifier == ParenModifier::Ensure
        && variadic_values.is_some_and(|values| values.len() > 1)
    {
        return format!(
            "{LEFT_PAREN}{}{RIGHT_PAREN}",
            strip_wrapping_parens(&rendered)
        );
    }
    apply_paren_modifier(&rendered, placeholder.modifier)
}

fn placeholder_notation_source(placeholder: &TemplatePlaceholder) -> String {
    placeholder
        .matrix_notation
        .as_ref()
        .map(MatrixNotation::source)
        .or_else(|| {
            placeholder
                .variadic_notation
                .as_ref()
                .map(VariadicNotation::source)
        })
        .unwrap_or_default()
}

fn render_matrix_notation(rows: &[Vec<&str>], notation: &MatrixNotation) -> String {
    rows.iter()
        .map(|row| {
            format!(
                "{}{}{}",
                notation.row_prefix,
                render_variadic_notation(row, &notation.elements),
                notation.row_suffix
            )
        })
        .collect()
}

fn render_variadic_notation(values: &[&str], notation: &VariadicNotation) -> String {
    match notation {
        VariadicNotation::Postfix(postfix) => values
            .iter()
            .map(|value| format!("{value}{postfix}"))
            .collect::<String>(),
        VariadicNotation::Prefix(prefix) => values
            .iter()
            .map(|value| format!("{prefix}{value}"))
            .collect::<String>(),
        VariadicNotation::Infix(infix) => values.join(infix),
    }
}

const VARIADIC_COUNT_PREFIX: &str = "\0mlg:variadic:count:";
const VARIADIC_ELEMENT_PREFIX: &str = "\0mlg:variadic:element:";
const VARIADIC_2D_ROWS_PREFIX: &str = "\0mlg:variadic2d:rows:";
const VARIADIC_2D_COLUMNS_PREFIX: &str = "\0mlg:variadic2d:columns:";
const VARIADIC_2D_ROW_LENGTH_PREFIX: &str = "\0mlg:variadic2d:row-length:";
const VARIADIC_2D_ELEMENT_PREFIX: &str = "\0mlg:variadic2d:element:";

pub(super) fn insert_variadic_substitution(
    substitutions: &mut HashMap<String, String>,
    name: &str,
    values: &[String],
) {
    substitutions.insert(
        format!("{VARIADIC_COUNT_PREFIX}{name}"),
        values.len().to_string(),
    );
    for (index, value) in values.iter().enumerate() {
        substitutions.insert(
            format!("{VARIADIC_ELEMENT_PREFIX}{name}:{index}"),
            value.clone(),
        );
    }
}

pub(super) fn insert_variadic_2d_substitution(
    substitutions: &mut HashMap<String, String>,
    name: &str,
    values: &[String],
    row_lengths: &[usize],
) {
    substitutions.insert(
        format!("{VARIADIC_2D_ROWS_PREFIX}{name}"),
        row_lengths.len().to_string(),
    );
    let columns = row_lengths.first().copied().unwrap_or(0);
    substitutions.insert(
        format!("{VARIADIC_2D_COLUMNS_PREFIX}{name}"),
        columns.to_string(),
    );
    for (row, length) in row_lengths.iter().enumerate() {
        substitutions.insert(
            format!("{VARIADIC_2D_ROW_LENGTH_PREFIX}{name}:{row}"),
            length.to_string(),
        );
    }
    for (index, value) in values.iter().enumerate() {
        substitutions.insert(
            format!("{VARIADIC_2D_ELEMENT_PREFIX}{name}:{index}"),
            value.clone(),
        );
    }
}

fn matrix_substitution_values<'a>(
    substitutions: &'a HashMap<String, String>,
    name: &str,
) -> Option<Vec<Vec<&'a str>>> {
    let rows = substitutions
        .get(&format!("{VARIADIC_2D_ROWS_PREFIX}{name}"))?
        .parse::<usize>()
        .ok()?;
    let default_columns = substitutions
        .get(&format!("{VARIADIC_2D_COLUMNS_PREFIX}{name}"))?
        .parse::<usize>()
        .ok()?;
    let mut offset = 0usize;
    (0..rows)
        .map(|row| {
            let columns = substitutions
                .get(&format!("{VARIADIC_2D_ROW_LENGTH_PREFIX}{name}:{row}"))
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(default_columns);
            let start = offset;
            offset += columns;
            (0..columns)
                .map(|column| {
                    substitutions
                        .get(&format!(
                            "{VARIADIC_2D_ELEMENT_PREFIX}{name}:{}",
                            start + column
                        ))
                        .map(String::as_str)
                })
                .collect()
        })
        .collect()
}

fn variadic_substitution_values<'a>(
    substitutions: &'a HashMap<String, String>,
    name: &str,
) -> Option<Vec<&'a str>> {
    let count = substitutions
        .get(&format!("{VARIADIC_COUNT_PREFIX}{name}"))?
        .parse::<usize>()
        .ok()?;
    (0..count)
        .map(|index| {
            substitutions
                .get(&format!("{VARIADIC_ELEMENT_PREFIX}{name}:{index}"))
                .map(String::as_str)
        })
        .collect()
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
    // Reference annotations do not contribute any visible syntax. Inspect their
    // rendered body so a linked compound expression such as
    // `\htmlData{mlg-ref=...}{P \text{ and } Q}` is not mistaken for one atom
    // merely because all of its spaces are inside the wrapper's braces.
    let mut text = text.trim();
    while let Some(body) = transparent_html_data_body(text) {
        text = body.trim();
    }

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
            // A top-level superscript is not atomic for the purpose of applying
            // another suffix. In particular, `x.inv` rendered as `x^{-1}` must
            // become `(x^{-1})^{-1}` when it is itself the owner of `.inv`, not
            // the invalid/ambiguous `x^{-1}^{-1}`.
            ch if depth == 0 && (ch == ',' || ch == '^' || ch.is_whitespace()) => return false,
            _ => {}
        }

        index += escaped_unit_len(rest);
    }

    true
}

/// Returns the visible body of a `\htmlData{...}{...}` wrapper when the wrapper
/// spans all of `text`.
fn transparent_html_data_body(text: &str) -> Option<&str> {
    const HTML_DATA: &str = "\\htmlData";

    let text = text.trim();
    let metadata_start = text.strip_prefix(HTML_DATA).map(|_| HTML_DATA.len())?;
    let (_, body_start) = braced_latex_argument(text, metadata_start)?;
    let (body, end) = braced_latex_argument(text, body_start)?;
    text[end..].trim().is_empty().then_some(body)
}

/// Parses the balanced braced argument beginning at `start`, returning its body
/// and the byte offset immediately after its closing brace.
fn braced_latex_argument(text: &str, start: usize) -> Option<(&str, usize)> {
    if !text.get(start..)?.starts_with('{') {
        return None;
    }

    let mut depth = 1usize;
    let mut index = start + 1;
    while index < text.len() {
        let rest = &text[index..];
        match rest.chars().next().expect("rest is not empty") {
            '\\' => index += escaped_unit_len(rest),
            '{' => {
                depth += 1;
                index += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((&text[start + 1..index], index + 1));
                }
                index += 1;
            }
            ch => index += ch.len_utf8(),
        }
    }

    None
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

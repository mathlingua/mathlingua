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
        if is_placeholder_start(chars[index]) {
            let start = index;
            index += 1;
            while index < chars.len() && is_placeholder_continue(chars[index]) {
                index += 1;
            }
            if index < chars.len() && chars[index] == '?' {
                let name = chars[start..index].iter().collect::<String>();
                if let Some(value) = substitutions.get(&name) {
                    flush_called_text(&mut result, &mut text);
                    result.push_str(value);
                    index += 1;
                    continue;
                }
                text.push_str(&name);
                index += 1;
                continue;
            }
            text.extend(chars[start..index].iter());
        } else {
            text.push(chars[index]);
            index += 1;
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
        if is_placeholder_start(chars[index]) {
            let start = index;
            index += 1;
            while index < chars.len() && is_placeholder_continue(chars[index]) {
                index += 1;
            }
            if index < chars.len() && chars[index] == '?' {
                let name = chars[start..index].iter().collect::<String>();
                flush_called_text(&mut result, &mut text);
                result.push_str(&render_template_placeholder_name(&name));
                index += 1;
                continue;
            }
            text.extend(chars[start..index].iter());
        } else {
            text.push(chars[index]);
            index += 1;
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

        if is_placeholder_start(chars[index]) {
            let start = index;
            index += 1;
            while index < chars.len() && is_placeholder_continue(chars[index]) {
                index += 1;
            }
            if index < chars.len() && chars[index] == '?' {
                let name = chars[start..index].iter().collect::<String>();
                if let Some(value) = substitutions.get(&name) {
                    result.push_str(value);
                    index += 1;
                    continue;
                }
                result.push_str(&name);
                index += 1;
                continue;
            }
            result.extend(chars[start..index].iter());
        } else {
            result.push(chars[index]);
            index += 1;
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

        if is_placeholder_start(chars[index]) {
            let start = index;
            index += 1;
            while index < chars.len() && is_placeholder_continue(chars[index]) {
                index += 1;
            }
            if index < chars.len() && chars[index] == '?' {
                let name = chars[start..index].iter().collect::<String>();
                result.push_str(&render_template_placeholder_name(&name));
                index += 1;
                continue;
            }
            result.extend(chars[start..index].iter());
        } else {
            result.push(chars[index]);
            index += 1;
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
    let needle = format!("{name}?");
    template.contains(&needle)
}

pub(super) fn is_placeholder_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

pub(super) fn is_placeholder_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '.'
}

use super::*;

pub(super) fn render_called_template(
    template: &str,
    substitutions: &HashMap<String, String>,
) -> String {
    let mut result = String::new();
    let mut in_math = false;

    for segment in template.split('$') {
        if in_math {
            result.push_str(&substitute_math_template(segment, substitutions));
        } else if !segment.is_empty() {
            result.push_str(&format!("\\textrm{{{}}}", escape_latex_text(segment)));
        }
        in_math = !in_math;
    }

    result
}

pub(super) fn substitute_math_template(
    template: &str,
    substitutions: &HashMap<String, String>,
) -> String {
    let mut result = String::new();
    let chars = template.chars().collect::<Vec<_>>();
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
                    result.push_str(value);
                    index += 1;
                    continue;
                }
            }
            result.extend(chars[start..index].iter());
        } else {
            result.push(chars[index]);
            index += 1;
        }
    }

    result
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

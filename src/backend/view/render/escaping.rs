use super::RenderRegistry;

pub(super) fn escape_math_identifier(value: &str, registry: &RenderRegistry) -> String {
    if value.contains('_') {
        return render_underscore_identifier(value, registry);
    }

    render_identifier_part(value, registry)
}

fn render_underscore_identifier(value: &str, registry: &RenderRegistry) -> String {
    let mut parts = value.split('_');
    let Some(base_part) = parts.next() else {
        return String::new();
    };

    let base = render_identifier_part(base_part, registry);
    let subscripts = parts
        .filter(|part| !part.is_empty())
        .map(|part| render_identifier_part(part, registry))
        .collect::<Vec<_>>();

    if subscripts.is_empty() {
        base
    } else {
        format_subscript(base, &subscripts.join(","))
    }
}

fn render_identifier_part(value: &str, registry: &RenderRegistry) -> String {
    match trailing_digit_subscript(value) {
        Some((base, digits)) => format_subscript(render_identifier_base(base, registry), digits),
        None => render_identifier_base(value, registry),
    }
}

fn render_identifier_base(value: &str, registry: &RenderRegistry) -> String {
    registry
        .writing
        .get(value)
        .cloned()
        .unwrap_or_else(|| escape_latex_math(value))
}

fn format_subscript(base: String, subscript: &str) -> String {
    if subscript.chars().count() == 1 && subscript.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        format!("{base}_{subscript}")
    } else {
        format!("{base}_{{{subscript}}}")
    }
}

fn trailing_digit_subscript(value: &str) -> Option<(&str, &str)> {
    let split = value
        .char_indices()
        .find_map(|(index, ch)| ch.is_ascii_digit().then_some(index))?;
    let (base, digits) = value.split_at(split);

    if base.is_empty()
        || digits.is_empty()
        || !base.chars().all(|ch| ch.is_ascii_alphabetic())
        || !digits.chars().all(|ch| ch.is_ascii_digit())
    {
        return None;
    }

    Some((base, digits))
}

pub(super) fn escape_latex_math(value: &str) -> String {
    value
        .replace('\\', "\\backslash ")
        .replace('_', "\\_")
        .replace('{', "\\{")
        .replace('}', "\\}")
}

pub(super) fn escape_latex_command_name(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .collect()
}

pub(super) fn escape_latex_text(value: &str) -> String {
    value
        .replace('\\', "\\textbackslash{}")
        .replace('{', "\\{")
        .replace('}', "\\}")
}

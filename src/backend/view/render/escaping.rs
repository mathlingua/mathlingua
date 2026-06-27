pub(super) fn escape_math_identifier(value: &str) -> String {
    match trailing_digit_subscript(value) {
        Some((base, digits)) if digits.len() == 1 => {
            format!("{}_{digits}", escape_latex_math(base))
        }
        Some((base, digits)) => format!("{}_{{{digits}}}", escape_latex_math(base)),
        None => escape_latex_math(value),
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

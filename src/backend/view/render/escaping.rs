/// Escapes a value that will be rendered as a math identifier.
fn escape_math_identifier(value: &str) -> String {
    escape_latex_math(value)
}

/// Escapes characters that are unsafe in the limited math-mode output we emit.
fn escape_latex_math(value: &str) -> String {
    value
        .replace('\\', "\\backslash ")
        .replace('_', "\\_")
        .replace('{', "\\{")
        .replace('}', "\\}")
}

/// Keeps only alphabetic characters for a generated LaTeX command name.
///
/// This is used by temporary quoted-operator rendering, where `"in"` becomes
/// `\in` and punctuation is intentionally discarded.
fn escape_latex_command_name(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_ascii_alphabetic())
        .collect()
}

/// Escapes characters that are unsafe inside rendered text-mode LaTeX.
fn escape_latex_text(value: &str) -> String {
    value
        .replace('\\', "\\textbackslash{}")
        .replace('{', "\\{")
        .replace('}', "\\}")
}


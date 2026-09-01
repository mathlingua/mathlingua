use super::{RenderRegistry, render_formulation_latex};
use std::collections::HashMap;

#[derive(Default)]
struct Scope {
    name: String,
    variables: HashMap<String, String>,
}

/// Converts MathLingua fragments embedded in prose to Markdown math while
/// maintaining scopes declared by `<<name>>` and `<</name>>`.
pub(in crate::backend::view) fn render_scoped_text_markdown(
    text: &str,
    registry: &RenderRegistry,
) -> String {
    ScopedTextRenderer {
        registry,
        scopes: vec![Scope::default()],
    }
    .render(text)
}

struct ScopedTextRenderer<'a> {
    registry: &'a RenderRegistry,
    // The root scope lasts for one text value. Named scopes are pushed and
    // popped as their markers are encountered from left to right.
    scopes: Vec<Scope>,
}

impl ScopedTextRenderer<'_> {
    fn render(mut self, text: &str) -> String {
        let mut output = String::with_capacity(text.len());
        let mut index = 0;
        while index < text.len() {
            let rest = &text[index..];
            if let Some((name, consumed)) = scope_marker(rest, false) {
                self.scopes.push(Scope {
                    name: name.to_string(),
                    variables: HashMap::new(),
                });
                index += consumed;
                continue;
            }
            if let Some((name, consumed)) = scope_marker(rest, true) {
                if self.scopes.len() > 1
                    && self.scopes.last().is_some_and(|scope| scope.name == name)
                {
                    self.scopes.pop();
                }
                index += consumed;
                continue;
            }
            if let Some((source, consumed)) = math_fragment(rest, true) {
                let (modifier, modifier_len) = relation_modifier(&rest[consumed..]);
                output.push_str("$$\n");
                output.push_str(&self.render_fragment(source, modifier));
                output.push_str("\n$$");
                index += consumed + modifier_len;
                continue;
            }
            if let Some((source, consumed)) = math_fragment(rest, false) {
                let (modifier, modifier_len) = relation_modifier(&rest[consumed..]);
                output.push('$');
                output.push_str(&self.render_fragment(source, modifier));
                output.push('$');
                index += consumed + modifier_len;
                continue;
            }
            let character = rest.chars().next().expect("non-empty text remainder");
            output.push(character);
            index += character.len_utf8();
        }
        output
    }

    fn render_fragment(&mut self, source: &str, modifier: Option<RelationModifier<'_>>) -> String {
        self.record_introductions(source);
        let mut latex = render_formulation_latex(source.trim(), self.registry)
            .unwrap_or_else(|| source.trim().to_string());
        if let Some(modifier) = modifier {
            let replacement = format!("\\textrm{{ {} }}", escape_latex_text(modifier.wording));
            let relation = match modifier.relation {
                "is" => "\\textrm{ is }".to_string(),
                operator => format!("\\{}", latex_command_name(operator)),
            };
            latex = latex.replacen(&relation, &replacement, 1);
        }
        latex
    }

    fn record_introductions(&mut self, source: &str) {
        let Some((subjects, relation, target)) = declaration_parts(source) else {
            return;
        };
        let Some(scope) = self.scopes.last_mut() else {
            return;
        };
        let inferred = format!("{relation} {target}");
        for subject in subjects.split(',').map(str::trim) {
            if is_plain_name(subject) {
                scope
                    .variables
                    .insert(subject.trim_end_matches('_').to_string(), inferred.clone());
            }
        }
    }
}

#[derive(Clone, Copy)]
struct RelationModifier<'a> {
    relation: &'a str,
    wording: &'a str,
}

fn scope_marker(input: &str, closing: bool) -> Option<(&str, usize)> {
    let prefix = if closing { "<</" } else { "<<" };
    let tail = input.strip_prefix(prefix)?;
    let end = tail.find(">>")?;
    let name = &tail[..end];
    if name.is_empty()
        || !name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
    {
        return None;
    }
    Some((name, prefix.len() + end + 2))
}

fn math_fragment(input: &str, display: bool) -> Option<(&str, usize)> {
    let (open, close) = if display {
        ("{{.", ".}}")
    } else {
        ("{.", ".}")
    };
    let tail = input.strip_prefix(open)?;
    // `{...`/`{{...` belong to variadic writing templates, not prose math.
    if tail.starts_with('.') {
        return None;
    }
    let end = tail.find(close)?;
    Some((&tail[..end], open.len() + end + close.len()))
}

fn relation_modifier(input: &str) -> (Option<RelationModifier<'_>>, usize) {
    let Some(tail) = input.strip_prefix('[') else {
        return (None, 0);
    };
    let Some(end) = tail.find(']') else {
        return (None, 0);
    };
    let body = tail[..end].trim();
    let Some((relation, wording)) = body.split_once('=') else {
        return (None, 0);
    };
    let relation = relation.trim();
    let relation = relation
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(relation);
    let wording = wording.trim();
    let Some(wording) = wording
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    else {
        return (None, 0);
    };
    if relation.is_empty() {
        return (None, 0);
    }
    (Some(RelationModifier { relation, wording }), end + 2)
}

fn declaration_parts(source: &str) -> Option<(&str, &str, &str)> {
    if let Some(index) = source.find(" is ") {
        return Some((&source[..index], "is", &source[index + 4..]));
    }
    let first_quote = source.find('"')?;
    let second_quote = source[first_quote + 1..].find('"')? + first_quote + 1;
    let relation = &source[first_quote + 1..second_quote];
    let subjects = source[..first_quote].trim_end();
    let target = source[second_quote + 1..]
        .trim_start_matches('?')
        .trim_start();
    (!subjects.is_empty() && !relation.is_empty() && !target.is_empty())
        .then_some((subjects, relation, target))
}

fn is_plain_name(text: &str) -> bool {
    let text = text.trim_end_matches('_');
    !text.is_empty()
        && text
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '\''))
}

fn escape_latex_text(text: &str) -> String {
    text.replace('\\', "\\textbackslash{}")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('%', "\\%")
        .replace('&', "\\&")
        .replace('#', "\\#")
        .replace('_', "\\_")
}

fn latex_command_name(text: &str) -> String {
    text.chars()
        .filter(|character| character.is_ascii_alphabetic())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_nested_scopes_and_math_fragments() {
        let text = "Suppose <<exists>>there exists {.x is \\real.}\n<<forall>>for all {.n is \\natural.}<</forall>><</exists>>";
        let rendered = render_scoped_text_markdown(text, &RenderRegistry::default());
        assert!(!rendered.contains("<<"));
        assert!(rendered.contains("$x \\textrm{ is } \\backslashreal$"));
        assert!(rendered.contains("$n \\textrm{ is } \\backslashnatural$"));
    }

    #[test]
    fn renders_display_fragments_and_relation_wording() {
        assert_eq!(
            render_scoped_text_markdown(
                "Let {.x is \\natural.}[is=\"be a\"]",
                &RenderRegistry::default(),
            ),
            "Let $x \\textrm{ be a } \\backslashnatural$"
        );
        let rendered = render_scoped_text_markdown("{{. x^2 = 0 .}}", &RenderRegistry::default());
        assert!(rendered.starts_with("$$\n"));
        assert!(rendered.ends_with("\n$$"));
        assert_eq!(
            render_scoped_text_markdown(
                "Choose {.x \"in\" X.}[\"in\"=\"from\"]",
                &RenderRegistry::default(),
            ),
            "Choose $x \\textrm{ from } X$"
        );
    }

    #[test]
    fn leaves_variadic_template_ellipsis_unchanged() {
        let text = r#"\left [ x?{{...\:...}...\\} \right ]"#;
        assert_eq!(
            render_scoped_text_markdown(text, &RenderRegistry::default()),
            text
        );
    }
}

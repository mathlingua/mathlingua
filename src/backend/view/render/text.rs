use super::{RenderRegistry, render_formulation_latex};
use std::collections::HashMap;

#[derive(Clone, Default)]
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
    render_scoped_text_markdown_with_labels(text, registry, &HashMap::new())
}

pub(in crate::backend::view) fn render_scoped_text_markdown_with_labels(
    text: &str,
    registry: &RenderRegistry,
    label_links: &HashMap<String, String>,
) -> String {
    ScopedTextRenderer {
        registry,
        label_links,
        scopes: vec![Scope::default()],
    }
    .render(text)
}

struct ScopedTextRenderer<'a> {
    registry: &'a RenderRegistry,
    label_links: &'a HashMap<String, String>,
    // The root scope lasts for one text value. Named scopes are pushed and
    // popped as their markers are encountered from left to right.
    scopes: Vec<Scope>,
}

impl ScopedTextRenderer<'_> {
    fn render_subtext(&self, text: &str) -> String {
        ScopedTextRenderer {
            registry: self.registry,
            label_links: self.label_links,
            scopes: self.scopes.clone(),
        }
        .render(text)
    }

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
            if let Some((inner, labels_str, consumed)) = prose_source_fragment(rest) {
                let rendered_inner = self.render_subtext(inner);
                output.push('*');
                output.push_str(&rendered_inner);
                output.push('*');
                output.push(' ');
                output.push('[');
                let formatted_labels = labels_str
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|label| {
                        if let Some(href) = self.label_links.get(label) {
                            format!("[{label}]({href})")
                        } else {
                            label.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                output.push_str(&formatted_labels);
                output.push(']');
                index += consumed;
                continue;
            }
            if let Some((source, consumed)) = theorem_reference_fragment(rest) {
                output.push('$');
                output.push_str(&self.render_theorem_reference(source));
                output.push('$');
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

    fn render_theorem_reference(&mut self, source: &str) -> String {
        let trimmed = source.trim();
        render_formulation_latex(trimmed, self.registry)
            .unwrap_or_else(|| trimmed.to_string())
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

fn prose_source_fragment(input: &str) -> Option<(&str, &str, usize)> {
    let tail = input.strip_prefix("''")?;
    let end_quote = tail.find("''")?;
    let inner = &tail[..end_quote];
    let after = &tail[end_quote + 2..];
    let trimmed = after.trim_start();
    let ref_tail = trimmed.strip_prefix("(:")?;
    let ref_end = ref_tail.find(":)")?;
    let ref_body = &ref_tail[..ref_end];
    let consumed = (input.len() - after.len()) + (after.len() - trimmed.len()) + 2 + ref_end + 2;
    Some((inner, ref_body, consumed))
}

fn theorem_reference_fragment(input: &str) -> Option<(&str, usize)> {
    let (open, close) = ("{:", ":}");
    let tail = input.strip_prefix(open)?;
    let end = tail.find(close)?;
    Some((&tail[..end], open.len() + end + close.len()))
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

    #[test]
    fn renders_theorem_references() {
        assert_eq!(
            render_scoped_text_markdown(
                "By {: \\some.thm :}, it holds.",
                &RenderRegistry::default(),
            ),
            "By $\\backslashsome.thm$, it holds."
        );
    }

    #[test]
    fn renders_prose_sources_with_and_without_links() {
        let text = r#"Proof: "This is ''some text with a source''(:2:)""#;
        assert_eq!(
            render_scoped_text_markdown(text, &RenderRegistry::default()),
            r#"Proof: "This is *some text with a source* [2]""#
        );

        let mut links = HashMap::new();
        links.insert("2".to_string(), "https://example.com/book.pdf".to_string());
        assert_eq!(
            render_scoped_text_markdown_with_labels(text, &RenderRegistry::default(), &links),
            r#"Proof: "This is *some text with a source* [[2](https://example.com/book.pdf)]""#
        );

        let multi = r#"See ''first quote''(:l1, l2:) and ''second quote'' (:3:)."#;
        assert_eq!(
            render_scoped_text_markdown(multi, &RenderRegistry::default()),
            r#"See *first quote* [l1, l2] and *second quote* [3]."#
        );
    }
}

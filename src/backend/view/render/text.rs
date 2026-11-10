use super::{RenderRegistry, render_formulation_latex};
use std::collections::HashMap;

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
    let rendered = ScopedTextRenderer {
        registry,
        label_links,
        scopes: vec![Scope::default()],
    }
    .render(text);
    decode_html_entities(&rendered)
}

pub(crate) fn markdown_code_span_len(input: &str) -> Option<usize> {
    if !input.starts_with('`') {
        return None;
    }
    let tick_count = input.bytes().take_while(|&b| b == b'`').count();
    let ticks = &input[..tick_count];
    let mut search_from = tick_count;
    while let Some(pos) = input[search_from..].find(ticks) {
        let abs_pos = search_from + pos;
        let after = abs_pos + tick_count;
        if input.as_bytes().get(after) != Some(&b'`') {
            return Some(after);
        }
        let extra = input[after..].bytes().take_while(|&b| b == b'`').count();
        search_from = after + extra;
    }
    None
}

pub(crate) fn theorem_reference_fragment(input: &str) -> Option<(&str, usize)> {
    let (open, close) = ("{:", ":}");
    let tail = input.strip_prefix(open)?;
    let end = tail.find(close)?;
    Some((&tail[..end], open.len() + end + close.len()))
}

pub(crate) fn math_fragment(input: &str, display: bool) -> Option<(&str, usize)> {
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

// =============================================================================

#[derive(Clone, Default)]
struct Scope {
    name: String,
    variables: HashMap<String, String>,
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
            if let Some(code_len) = markdown_code_span_len(rest) {
                output.push_str(&rest[..code_len]);
                index += code_len;
                continue;
            }
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
                output.push_str(&self.format_label_list(labels_str));
                output.push(']');
                index += consumed;
                continue;
            }
            if let Some((labels_str, consumed)) = label_reference_fragment(rest) {
                output.push('[');
                output.push_str(&self.format_label_list(labels_str));
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
        render_formulation_latex(trimmed, self.registry).unwrap_or_else(|| trimmed.to_string())
    }

    fn format_label_list(&self, labels_str: &str) -> String {
        labels_str
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
            .join(", ")
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
    let ref_tail = trimmed.strip_prefix("[:")?;
    let ref_end = ref_tail.find(":]")?;
    let ref_body = &ref_tail[..ref_end];
    if ref_body.contains(',') {
        return None;
    }
    let consumed = (input.len() - after.len()) + (after.len() - trimmed.len()) + 2 + ref_end + 2;
    Some((inner, ref_body.trim(), consumed))
}

fn label_reference_fragment(input: &str) -> Option<(&str, usize)> {
    let tail = input.strip_prefix("(:")?;
    let end = tail.find(":)")?;
    let body = &tail[..end];
    if body.trim().is_empty() {
        return None;
    }
    Some((body, 2 + end + 2))
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

pub(in crate::backend::view) fn decode_html_entities(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut index = 0;

    while index < text.len() {
        let rest = &text[index..];

        // Skip display math blocks: $$\n ... \n$$
        if rest.starts_with("$$\n") {
            if let Some(end) = rest[3..].find("\n$$") {
                let block_len = 3 + end + 3;
                output.push_str(&rest[..block_len]);
                index += block_len;
                continue;
            }
        }

        // Skip inline math spans: $ ... $
        if rest.starts_with('$')
            && !rest.starts_with("$$")
            && (index == 0 || text.as_bytes().get(index - 1) != Some(&b'\\'))
        {
            if let Some(close_idx) = find_unescaped_char(&rest[1..], '$') {
                let span_len = 1 + close_idx + 1;
                output.push_str(&rest[..span_len]);
                index += span_len;
                continue;
            }
        }

        // Check for HTML entity starting with '&'
        if rest.starts_with('&') {
            if let Some((entity_str, entity_len)) = parse_html_entity(rest) {
                output.push_str(&entity_str);
                index += entity_len;
                continue;
            }
        }

        let ch = rest.chars().next().expect("non-empty remainder");
        output.push(ch);
        index += ch.len_utf8();
    }

    output
}

fn find_unescaped_char(text: &str, target: char) -> Option<usize> {
    let mut escaped = false;
    for (i, c) in text.char_indices() {
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == target {
            return Some(i);
        }
    }
    None
}

fn parse_html_entity(input: &str) -> Option<(String, usize)> {
    if !input.starts_with('&') {
        return None;
    }
    let rest = &input[1..];
    let semi_pos = rest.find(';')?;
    if semi_pos == 0 || semi_pos > 31 {
        return None;
    }
    let body = &rest[..semi_pos];
    if body.chars().any(|c| c.is_whitespace() || c == '&' || c == '<' || c == '>') {
        return None;
    }

    let decoded = decode_entity_body(body)?;
    Some((decoded, 1 + semi_pos + 1))
}

fn decode_entity_body(body: &str) -> Option<String> {
    if let Some(dec_body) = body.strip_prefix('#') {
        if let Some(hex_body) = dec_body.strip_prefix('x').or_else(|| dec_body.strip_prefix('X')) {
            if !hex_body.is_empty() && hex_body.chars().all(|c| c.is_ascii_hexdigit()) {
                let code = u32::from_str_radix(hex_body, 16).ok()?;
                return char::from_u32(code).map(|c| c.to_string());
            }
            return None;
        }
        if !dec_body.is_empty() && dec_body.chars().all(|c| c.is_ascii_digit()) {
            let code = dec_body.parse::<u32>().ok()?;
            return char::from_u32(code).map(|c| c.to_string());
        }
        return None;
    }

    // Decimal entity without '#' (e.g. `&126;`)
    if !body.is_empty() && body.chars().all(|c| c.is_ascii_digit()) {
        let code = body.parse::<u32>().ok()?;
        return char::from_u32(code).map(|c| c.to_string());
    }

    // Hex entity without '#' (e.g. `&x7e;` or `&X7E;`)
    if let Some(hex_body) = body.strip_prefix('x').or_else(|| body.strip_prefix('X')) {
        if !hex_body.is_empty() && hex_body.chars().all(|c| c.is_ascii_hexdigit()) {
            let code = u32::from_str_radix(hex_body, 16).ok()?;
            return char::from_u32(code).map(|c| c.to_string());
        }
        return None;
    }

    // Named entities
    let decoded = match body {
        "lt" => "<",
        "gt" => ">",
        "amp" => "&",
        "quot" => "\"",
        "apos" => "'",
        "tilde" => "~",
        "nbsp" => "\u{00a0}",
        "cent" => "¢",
        "pound" => "£",
        "yen" => "¥",
        "euro" => "€",
        "copy" => "©",
        "reg" => "®",
        "sect" => "§",
        "deg" => "°",
        "plusmn" => "±",
        "times" => "×",
        "divide" => "÷",
        "micro" => "µ",
        "para" => "¶",
        "middot" => "·",
        "ndash" => "–",
        "mdash" => "—",
        "lsquo" => "‘",
        "rsquo" => "’",
        "sbquo" => "‚",
        "ldquo" => "“",
        "rdquo" => "”",
        "bdquo" => "„",
        "dagger" => "†",
        "Dagger" => "‡",
        "bull" => "•",
        "hellip" => "…",
        "permil" => "‰",
        "prime" => "′",
        "Prime" => "″",
        "larr" => "←",
        "uarr" => "↑",
        "rarr" => "→",
        "darr" => "↓",
        "harr" => "↔",
        "forall" => "∀",
        "part" => "∂",
        "exist" => "∃",
        "empty" => "∅",
        "nabla" => "∇",
        "isin" => "∈",
        "notin" => "∉",
        "ni" => "∋",
        "prod" => "∏",
        "sum" => "∑",
        "minus" => "−",
        "radic" => "√",
        "prop" => "∝",
        "infin" => "∞",
        "ang" => "∠",
        "and" => "∧",
        "or" => "∨",
        "cap" => "∩",
        "cup" => "∪",
        "int" => "∫",
        "there4" => "∴",
        "sim" => "∼",
        "cong" => "≅",
        "asymp" => "≈",
        "ne" => "≠",
        "equiv" => "≡",
        "le" => "≤",
        "ge" => "≥",
        "sub" => "⊂",
        "sup" => "⊃",
        "nsub" => "⊄",
        "sube" => "⊆",
        "supe" => "⊇",
        "oplus" => "⊕",
        "otimes" => "⊗",
        "perp" => "⊥",
        "sdot" => "⋅",
        _ => return None,
    };
    Some(decoded.to_string())
}

// =============================================================================

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
        let text = r#"Proof: "This is ''some text with a source''[:2:]""#;
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

        let text_with_refs = r#"See ''first quote''[:1:] and ''second quote'' [:3:] and (:2:)."#;
        assert_eq!(
            render_scoped_text_markdown_with_labels(text_with_refs, &RenderRegistry::default(), &links),
            r#"See *first quote* [1] and *second quote* [3] and [[2](https://example.com/book.pdf)]."#
        );
    }

    #[test]
    fn decodes_html_entities_and_escapes_in_prose() {
        assert_eq!(
            decode_html_entities("Tilde: &126; and &#126; and &tilde;"),
            "Tilde: ~ and ~ and ~"
        );
        assert_eq!(
            decode_html_entities("Braces: &#123;. and .&#125;"),
            "Braces: {. and .}"
        );
        assert_eq!(
            decode_html_entities("Quotes and colon: &#39;&#39; and [&#58;label&#58;]"),
            "Quotes and colon: '' and [:label:]"
        );
        assert_eq!(
            decode_html_entities("Angles: &lt;&lt;name&gt;&gt; and &lt;&lt;/name&gt;&gt;"),
            "Angles: <<name>> and <</name>>"
        );
        assert_eq!(
            decode_html_entities("Hex: &#x7e; and &x7E;"),
            "Hex: ~ and ~"
        );
        assert_eq!(
            decode_html_entities("Preserves AT&T and &unknown; and unclosed &126"),
            "Preserves AT&T and &unknown; and unclosed &126"
        );
        assert_eq!(
            decode_html_entities("Preserves math: $a &lt; b$ and $$\n\\begin{matrix} 1 & 2 \\end{matrix}\n$$"),
            "Preserves math: $a &lt; b$ and $$\n\\begin{matrix} 1 & 2 \\end{matrix}\n$$"
        );
    }

    #[test]
    fn renders_escaped_delimiters_without_activating_fragments() {
        let text = "Inline MathLingua opens with &#123;. and closes with .&#125;. For example, let {. x is \\real .}. Use &126;.";
        let rendered = render_scoped_text_markdown(text, &RenderRegistry::default());
        assert_eq!(
            rendered,
            "Inline MathLingua opens with {. and closes with .}. For example, let $x \\textrm{ is } \\backslashreal$. Use ~."
        );
    }
}

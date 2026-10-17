use super::model::{ArgumentView, GroupView, SectionView};

/// Formats a `GroupView` into a clean Markdown live preview for display
/// in Zed's hover popup or preview panels.
pub fn render_group_markdown(group: &GroupView) -> String {
    let mut out = String::new();

    // 1. Title / Header
    let kind = &group.kind;
    match (&group.heading, &group.heading_latex) {
        (Some(h), Some(hl)) if h.starts_with('\\') => {
            out.push_str(&format!("### `[\\{}]`\n\n", h));
            out.push_str(&format!("$${}$$\n\n", hl));
        }
        (Some(h), _) if h.starts_with('\\') => {
            out.push_str(&format!("### `[\\{}]`\n\n", h));
        }
        (Some(h), _) => {
            out.push_str(&format!("### {}: {}\n\n", kind, h));
        }
        (None, Some(hl)) => {
            out.push_str(&format!("### {}\n\n$${}$$\n\n", kind, hl));
        }
        (None, None) => {
            out.push_str(&format!("### {}\n\n", kind));
        }
    }

    if !group.id.is_empty() {
        out.push_str(&format!("*Id:* `{}`\n\n", group.id));
    }

    // 2. Parameter destructurings
    for destructuring in &group.parameter_destructurings {
        out.push_str(&format!("$${}$$\n\n", destructuring));
    }

    // 3. Page or body text
    if let Some(page) = &group.page {
        out.push_str(&page.text);
        out.push_str("\n\n");
    } else if let Some(body) = &group.body_text {
        out.push_str(body);
        out.push_str("\n\n");
    }

    // 4. Sections
    for section in &group.sections {
        render_section_markdown(section, 0, &mut out);
    }

    // 5. Proof text
    if let Some(proof) = &group.proof_text {
        out.push_str("**Proof:**\n\n");
        out.push_str(proof);
        out.push_str("\n\n");
    }

    // 6. Action link
    out.push_str("---\n");
    out.push_str("🌐 [Open Live Page Preview (http://localhost:3000)](http://localhost:3000)\n");

    out
}

fn render_section_markdown(section: &SectionView, indent: usize, out: &mut String) {
    let pad = "  ".repeat(indent);
    let label = &section.label;
    match (&section.inline_latex, &section.inline_argument) {
        (Some(latex), _) => {
            out.push_str(&format!("{}**{}:** ${}$\n", pad, label, latex));
        }
        (None, Some(arg)) => {
            out.push_str(&format!("{}**{}:** `{}`\n", pad, label, arg));
        }
        (None, None) => {
            out.push_str(&format!("{}**{}:**\n", pad, label));
        }
    }

    for arg in &section.arguments {
        render_argument_markdown(arg, indent, out);
    }

    if indent == 0 {
        out.push('\n');
    }
}

fn render_argument_markdown(arg: &ArgumentView, indent: usize, out: &mut String) {
    let pad = "  ".repeat(indent);
    match arg {
        ArgumentView::Formulation {
            text,
            latex,
            label,
            reference_label,
            ..
        } => {
            let math = match latex {
                Some(l) => format!("${}$", l),
                None => format!("`{}`", text),
            };
            let label_prefix = match label {
                Some(lbl) => format!("[{}] ", lbl),
                None => String::new(),
            };
            let ref_suffix = match reference_label {
                Some(r) => format!(" *(ref: {})*", r),
                None => String::new(),
            };
            out.push_str(&format!("{}- {}{}{}\n", pad, label_prefix, math, ref_suffix));
        }
        ArgumentView::Text { text, latex, label } => {
            let label_prefix = match label {
                Some(lbl) => format!("[{}] ", lbl),
                None => String::new(),
            };
            let body = match latex {
                Some(l) => format!("${}$", l),
                None => text.clone(),
            };
            out.push_str(&format!("{}- {}{}\n", pad, label_prefix, body));
        }
        ArgumentView::Reference { text, href, label, .. } => {
            let label_prefix = match label {
                Some(lbl) => format!("[{}] ", lbl),
                None => String::new(),
            };
            let body = match href {
                Some(url) => format!("📖 [{}]({})", text, url),
                None => format!("📖 {}", text),
            };
            out.push_str(&format!("{}- {}{}\n", pad, label_prefix, body));
        }
        ArgumentView::Group { heading, sections } => {
            if let Some(h) = heading {
                out.push_str(&format!("{}- **[{}]**\n", pad, h));
            } else {
                out.push_str(&format!("{}-\n", pad));
            }
            for sec in sections {
                render_section_markdown(sec, indent + 1, out);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::view::model::{ArgumentView, GroupView, SectionView};

    #[test]
    fn renders_theorem_preview() {
        let group = GroupView {
            id: "fermat".to_string(),
            kind: "Theorem".to_string(),
            definition_keys: Vec::new(),
            heading: Some("Fermat's Last Theorem".to_string()),
            heading_latex: None,
            parameter_destructurings: Vec::new(),
            body_text: None,
            proof_text: Some("Left as an exercise to the reader.".to_string()),
            proof_source: None,
            page: None,
            source: String::new(),
            sections: vec![
                SectionView {
                    label: "when".to_string(),
                    inline_argument: None,
                    inline_latex: None,
                    inline_type_info: Vec::new(),
                    arguments: vec![ArgumentView::Formulation {
                        text: "n > 2".to_string(),
                        latex: Some("n > 2".to_string()),
                        type_info: Vec::new(),
                        label: None,
                        reference_label: None,
                    }],
                },
                SectionView {
                    label: "then".to_string(),
                    inline_argument: None,
                    inline_latex: None,
                    inline_type_info: Vec::new(),
                    arguments: vec![ArgumentView::Formulation {
                        text: "a^n + b^n != c^n".to_string(),
                        latex: Some("a^{n} + b^{n} \\neq c^{n}".to_string()),
                        type_info: Vec::new(),
                        label: Some("1".to_string()),
                        reference_label: None,
                    }],
                },
            ],
        };

        let md = render_group_markdown(&group);
        assert!(md.contains("### Theorem: Fermat's Last Theorem"));
        assert!(md.contains("*Id:* `fermat`"));
        assert!(md.contains("**when:**"));
        assert!(md.contains("- $n > 2$"));
        assert!(md.contains("**then:**"));
        assert!(md.contains("- [1] $a^{n} + b^{n} \\neq c^{n}$"));
        assert!(md.contains("**Proof:**"));
        assert!(md.contains("Left as an exercise to the reader."));
        assert!(md.contains("Open Live Page Preview"));
    }
}

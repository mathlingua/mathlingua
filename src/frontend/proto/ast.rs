use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Metadata {
    pub row: usize,
    pub indent: usize,
    pub has_dot: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub text: String,
    pub metadata: Metadata,
}

impl Line {
    pub fn is_blank(&self) -> bool {
        self.text.is_empty()
    }

    pub fn is_comment(&self) -> bool {
        self.text.starts_with("--")
    }

    pub fn is_divider(&self) -> bool {
        self.text.len() >= 3 && self.text.chars().all(|ch| ch == '-')
    }

    pub fn is_blank_or_comment(&self) -> bool {
        self.is_blank() || self.is_comment() || self.is_divider()
    }

    pub fn is_text(&self) -> bool {
        self.text.starts_with('"')
    }

    pub fn is_header(&self) -> bool {
        self.text.starts_with('[')
            && self.text.ends_with(']')
            && extract_line_label(&self.text).is_none()
    }
}

/// Extracts a leading `[label]: ` prefix from a line, returning the label text
/// and the remaining content after the colon.
///
/// A labeled line has the form `[label]: xxx` where `label` is non-empty, contains
/// no whitespace, and `]: ` is immediately followed by non-empty content `xxx`.
pub fn extract_line_label(text: &str) -> Option<(&str, &str)> {
    let stripped = text.strip_prefix('[')?;
    let bracket_index = stripped.find(']')?;
    let label = &stripped[..bracket_index];
    if label.is_empty() || label.contains(char::is_whitespace) || label.contains('[') {
        return None;
    }
    let after_bracket = &stripped[bracket_index + 1..];
    let after_colon = after_bracket.strip_prefix(':')?;
    let trimmed = after_colon.trim_start();
    if trimmed.len() == after_colon.len() {
        // There was no whitespace after the colon (e.g., `[x]:=` or `[x]:y`).
        return None;
    }
    if trimmed.is_empty() {
        return None;
    }
    Some((label, trimmed))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Formulation {
    pub text: String,
    pub label: Option<String>,
    pub metadata: Metadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextLiteral {
    pub text: String,
    pub label: Option<String>,
    pub metadata: Metadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Argument {
    Formulation(Formulation),
    Text(TextLiteral),
    Group(Group),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    pub label: String,
    pub inline_argument: Option<String>,
    pub arguments: Vec<Argument>,
    pub metadata: Metadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub heading: Option<String>,
    pub sections: Vec<Section>,
    pub metadata: Metadata,
}

// =======================[ Display implementations ]===========================

fn write_prefix(f: &mut fmt::Formatter<'_>, metadata: &Metadata) -> fmt::Result {
    let indent_width = if metadata.has_dot {
        metadata.indent.saturating_sub(2)
    } else {
        metadata.indent
    };

    write!(f, "{}", " ".repeat(indent_width))?;
    if metadata.has_dot {
        write!(f, ". ")?;
    }

    Ok(())
}

impl fmt::Display for Line {
    /// Formats a proto line in source-like form.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_prefix(f, &self.metadata)?;
        write!(f, "{}", self.text)
    }
}

impl fmt::Display for Formulation {
    /// Formats a formulation argument in source-like form.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_prefix(f, &self.metadata)?;
        if let Some(label) = &self.label {
            write!(f, "[{label}]: ")?;
        }
        write!(f, "{}", self.text)
    }
}

impl fmt::Display for TextLiteral {
    /// Formats a text argument in source-like form.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_prefix(f, &self.metadata)?;
        if let Some(label) = &self.label {
            write!(f, "[{label}]: ")?;
        }
        write!(f, "{}", self.text)
    }
}

impl fmt::Display for Argument {
    /// Formats any proto argument in source-like form.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Argument::Formulation(formulation) => write!(f, "{formulation}"),
            Argument::Text(text) => write!(f, "{text}"),
            Argument::Group(group) => write!(f, "{group}"),
        }
    }
}

impl fmt::Display for Section {
    /// Formats a proto section in source-like form.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_prefix(f, &self.metadata)?;
        match &self.inline_argument {
            Some(argument) => write!(f, "{}: {}", self.label, argument)?,
            None => write!(f, "{}:", self.label)?,
        }

        for argument in &self.arguments {
            write!(f, "\n{argument}")?;
        }

        Ok(())
    }
}

impl fmt::Display for Group {
    /// Formats a proto group in source-like form.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut wrote_anything = false;

        if let Some(heading) = &self.heading {
            write_prefix(f, &self.metadata)?;
            write!(f, "[{heading}]")?;
            wrote_anything = true;
        }

        for section in &self.sections {
            if wrote_anything {
                writeln!(f)?;
            }
            write!(f, "{section}")?;
            wrote_anything = true;
        }

        Ok(())
    }
}

// ===============================[ tests ]=====================================

#[cfg(test)]
mod tests {
    use super::{Argument, Formulation, Group, Line, Metadata, Section, TextLiteral};

    fn metadata(indent: usize, has_dot: bool) -> Metadata {
        Metadata {
            row: 0,
            indent,
            has_dot,
        }
    }

    #[test]
    fn classifies_blank_comments_text_and_headers() {
        let blank = Line {
            text: String::new(),
            metadata: metadata(0, false),
        };
        assert!(blank.is_blank());
        assert!(blank.is_blank_or_comment());
        assert!(!blank.is_comment());
        assert!(!blank.is_text());
        assert!(!blank.is_header());

        let comment = Line {
            text: "-- note".to_string(),
            metadata: metadata(0, false),
        };
        assert!(comment.is_comment());
        assert!(comment.is_blank_or_comment());
        assert!(!comment.is_text());
        assert!(!comment.is_header());

        let text = Line {
            text: "\"hello\"".to_string(),
            metadata: metadata(0, false),
        };
        assert!(text.is_text());
        assert!(!text.is_blank_or_comment());
        assert!(!text.is_header());

        let header = Line {
            text: "[group]".to_string(),
            metadata: metadata(0, false),
        };
        assert!(header.is_header());
        assert!(!header.is_blank_or_comment());
        assert!(!header.is_text());
    }

    #[test]
    fn display_saturates_dot_indentation_when_indent_is_smaller_than_prefix_width() {
        let line = Line {
            text: "x".to_string(),
            metadata: metadata(1, true),
        };

        assert_eq!(line.to_string(), ". x");
    }

    #[test]
    fn displays_sections_and_groups_with_nested_arguments() {
        let group = Group {
            heading: Some("heading".to_string()),
            sections: vec![
                Section {
                    label: "Defines".to_string(),
                    inline_argument: Some("f(x_)".to_string()),
                    arguments: vec![],
                    metadata: metadata(0, false),
                },
                Section {
                    label: "when".to_string(),
                    inline_argument: None,
                    arguments: vec![
                        Argument::Formulation(Formulation {
                            text: "x in A".to_string(),
                            label: None,
                            metadata: metadata(2, true),
                        }),
                        Argument::Text(TextLiteral {
                            text: "\"note\"".to_string(),
                            label: None,
                            metadata: metadata(2, true),
                        }),
                        Argument::Group(Group {
                            heading: None,
                            sections: vec![Section {
                                label: "exists".to_string(),
                                inline_argument: Some("z".to_string()),
                                arguments: vec![],
                                metadata: metadata(2, true),
                            }],
                            metadata: metadata(2, true),
                        }),
                    ],
                    metadata: metadata(0, false),
                },
            ],
            metadata: metadata(0, false),
        };

        assert_eq!(
            group.to_string(),
            "[heading]\nDefines: f(x_)\nwhen:\n. x in A\n. \"note\"\n. exists: z"
        );
    }

    #[test]
    fn extracts_line_labels_correctly() {
        use super::extract_line_label;

        assert_eq!(extract_line_label("[label]: x > 0"), Some(("label", "x > 0")));
        assert_eq!(
            extract_line_label("[somelabel]:   x > 0"),
            Some(("somelabel", "x > 0"))
        );
        assert_eq!(
            extract_line_label("[abc]: \"hello\""),
            Some(("abc", "\"hello\""))
        );
        assert_eq!(
            extract_line_label("[step_1]: [a, b]"),
            Some(("step_1", "[a, b]"))
        );

        // Not labeled lines
        assert_eq!(extract_line_label("[heading]"), None);
        assert_eq!(extract_line_label("[a, b]"), None);
        assert_eq!(extract_line_label("[x]:= 1"), None);
        assert_eq!(extract_line_label("[x]:"), None);
        assert_eq!(extract_line_label("[x]: "), None);
        assert_eq!(extract_line_label("[]: x"), None);
        assert_eq!(extract_line_label("[a b]: x"), None);
        assert_eq!(extract_line_label("forAll: [label]: x"), None);
    }

    #[test]
    fn displays_labeled_formulation_and_text() {
        let formulation = Formulation {
            text: "x > 0".to_string(),
            label: Some("somelabel".to_string()),
            metadata: metadata(2, true),
        };
        assert_eq!(formulation.to_string(), ". [somelabel]: x > 0");

        let text = TextLiteral {
            text: "\"note\"".to_string(),
            label: Some("abc".to_string()),
            metadata: metadata(2, true),
        };
        assert_eq!(text.to_string(), ". [abc]: \"note\"");
    }
}

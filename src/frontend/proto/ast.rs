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

    pub fn is_blank_or_comment(&self) -> bool {
        self.is_blank() || self.is_comment()
    }

    pub fn is_text(&self) -> bool {
        self.text.starts_with('"') && self.text.ends_with('"')
    }

    pub fn is_header(&self) -> bool {
        self.text.starts_with('[') && self.text.ends_with(']')
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Formulation {
    pub text: String,
    pub metadata: Metadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextLiteral {
    pub text: String,
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
        write!(f, "{}", self.text)
    }
}

impl fmt::Display for TextLiteral {
    /// Formats a text argument in source-like form.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_prefix(f, &self.metadata)?;
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
                            metadata: metadata(2, true),
                        }),
                        Argument::Text(TextLiteral {
                            text: "\"note\"".to_string(),
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
}

use std::fmt;

/// Source metadata attached to a proto line, section, argument, or group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Metadata {
    /// Zero-based row in the original source.
    pub row: usize,
    /// Indentation width after accounting for dot-prefix syntax.
    pub indent: usize,
    /// Whether the line was introduced with `. `.
    pub has_dot: bool,
}

/// One normalized source line produced by the proto lexer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    /// Line text after leading indentation and optional dot prefix are removed.
    pub text: String,
    /// Source metadata for the line.
    pub metadata: Metadata,
}

impl Line {
    /// Returns true when the line has no text after normalization.
    pub fn is_blank(&self) -> bool {
        self.text.is_empty()
    }

    /// Returns true when the line is a comment line.
    pub fn is_comment(&self) -> bool {
        self.text.starts_with("--")
    }

    /// Returns true when the line is blank or a comment.
    pub fn is_blank_or_comment(&self) -> bool {
        self.is_blank() || self.is_comment()
    }

    /// Returns true when the line is a quoted text literal.
    pub fn is_text(&self) -> bool {
        self.text.starts_with('"') && self.text.ends_with('"')
    }

    /// Returns true when the line is a bracketed group heading.
    pub fn is_header(&self) -> bool {
        self.text.starts_with('[') && self.text.ends_with(']')
    }
}

/// Proto argument that should later be parsed as formulation text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Formulation {
    /// Raw formulation text.
    pub text: String,
    /// Source metadata for the formulation line.
    pub metadata: Metadata,
}

/// Proto argument that is explicitly plain text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextLiteral {
    /// Raw text literal including its source-level quotes.
    pub text: String,
    /// Source metadata for the text line.
    pub metadata: Metadata,
}

/// Argument nested under a proto section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Argument {
    /// Formulation argument.
    Formulation(Formulation),
    /// Quoted text argument.
    Text(TextLiteral),
    /// Nested group argument.
    Group(Group),
}

/// Proto section consisting of a label, optional inline argument, and arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    /// Section label before the colon.
    pub label: String,
    /// Optional inline argument after the colon.
    pub inline_argument: Option<String>,
    /// Nested arguments in this section.
    pub arguments: Vec<Argument>,
    /// Source metadata for the section line.
    pub metadata: Metadata,
}

/// Proto group consisting of an optional heading and one or more sections.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    /// Optional heading text without surrounding brackets.
    pub heading: Option<String>,
    /// Sections belonging to the group.
    pub sections: Vec<Section>,
    /// Source metadata for the heading or first section.
    pub metadata: Metadata,
}

/// Writes indentation and dot-prefix text for display round-tripping.
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

// =============================================================================

#[cfg(test)]
mod tests;

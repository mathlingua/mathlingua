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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_prefix(f, &self.metadata)?;
        write!(f, "{}", self.text)
    }
}

impl fmt::Display for Formulation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_prefix(f, &self.metadata)?;
        write!(f, "{}", self.text)
    }
}

impl fmt::Display for TextLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_prefix(f, &self.metadata)?;
        write!(f, "{}", self.text)
    }
}

impl fmt::Display for Argument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Argument::Formulation(formulation) => write!(f, "{formulation}"),
            Argument::Text(text) => write!(f, "{text}"),
            Argument::Group(group) => write!(f, "{group}"),
        }
    }
}

impl fmt::Display for Section {
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

#[cfg(test)]
mod tests;

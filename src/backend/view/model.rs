use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CollectionView {
    pub title: String,
    pub directories: Vec<DirectoryView>,
    pub files: Vec<FileView>,
    /// Rendered items of the collection's optional root `content/_preface_.mlg`,
    /// shown on the cover page beneath the collection title. Empty when absent.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preface: Vec<GroupView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DirectoryView {
    pub path: String,
    pub title: Option<String>,
    /// Rendered items of the directory's optional `_preface_.mlg`, shown on the
    /// section page beneath the title. Empty when the directory has no preface.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preface: Vec<GroupView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FileView {
    pub path: String,
    pub title: Option<String>,
    pub items: Vec<GroupView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GroupView {
    pub id: String,
    pub kind: String,
    pub definition_keys: Vec<String>,
    pub heading: Option<String>,
    pub heading_latex: Option<String>,
    /// The `name ::= …` destructuring of each destructured header parameter (e.g.
    /// `H ::= (X', *', e')`), rendered as LaTeX and shown as lines beneath the
    /// title so the title itself can use the plain parameter names. Empty for a
    /// group with no destructured parameters.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameter_destructurings: Vec<String>,
    pub body_text: Option<String>,
    /// Markdown-with-LaTeX proof prose rendered after, rather than inside, the
    /// theorem card. Absent for items without a `Proof:` section.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof_text: Option<String>,
    pub page: Option<PageView>,
    pub source: String,
    pub sections: Vec<SectionView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PageView {
    pub kind: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SectionView {
    pub label: String,
    pub inline_argument: Option<String>,
    pub inline_latex: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inline_type_info: Vec<TypeEntryView>,
    pub arguments: Vec<ArgumentView>,
}

/// One expression or subexpression and the types resolved for it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TypeEntryView {
    pub depth: usize,
    pub text: String,
    pub types: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ArgumentView {
    Formulation {
        text: String,
        latex: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        type_info: Vec<TypeEntryView>,
        /// The `[:label:]` of a labeled specification (e.g. `1` for `(.….)[:1:]`),
        /// rendered as a right-justified tag beside the formulation. `None` for an
        /// ordinary formulation.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
    Text {
        text: String,
        latex: Option<String>,
    },
    Reference {
        /// Original `$resource[:page{n}]` source, without optional quotes.
        source: String,
        /// Resolved resource title with its authors, or the source when unresolved.
        text: String,
        /// Resource URL, adjusted to the requested physical PDF page when possible.
        href: Option<String>,
    },
    Group {
        heading: Option<String>,
        sections: Vec<SectionView>,
    },
}

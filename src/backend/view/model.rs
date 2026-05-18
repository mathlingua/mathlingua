use serde::Serialize;

/// Serialized payload consumed by the web viewer for an entire collection.
///
/// The backend intentionally emits a presentation-oriented structure here rather
/// than exposing frontend AST internals directly.  That keeps the web code small
/// and lets rendering-specific values, such as LaTeX strings, be computed once in
/// Rust before the JSON is handed to Next.js.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CollectionView {
    /// Human-readable collection title shown by the viewer.
    pub title: String,
    /// Renderable files in the collection, in the order chosen by the caller.
    pub files: Vec<FileView>,
}

/// Serialized view model for one MathLingua source file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FileView {
    /// File path relative to the collection root when possible.
    pub path: String,
    /// Top-level groups rendered from the file.
    pub items: Vec<GroupView>,
}

/// Serialized view model for one top-level MathLingua group.
///
/// `heading_latex` is present only when the backend has enough documented
/// rendering information to produce a friendly title for the group card.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GroupView {
    /// Structural kind of the group, such as `Describes`, `Refines`, or `Theorem`.
    pub kind: String,
    /// Raw bracket heading text, if the source group had one.
    pub heading: Option<String>,
    /// Rendered LaTeX title for the group card, if available.
    pub heading_latex: Option<String>,
    /// Rendered sections belonging to the group.
    pub sections: Vec<SectionView>,
}

/// Serialized view model for one section inside a group.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SectionView {
    /// Section label as written in the source, without the trailing colon.
    pub label: String,
    /// Raw inline argument following the section label, if one was present.
    pub inline_argument: Option<String>,
    /// Rendered LaTeX for the inline argument, if it parsed successfully.
    pub inline_latex: Option<String>,
    /// Block arguments nested under this section.
    pub arguments: Vec<ArgumentView>,
}

/// Serialized representation of an argument nested under a section.
///
/// The enum is tagged for JSON so the web viewer can decide whether to render a
/// formulation, plain text, or a nested group without guessing from shape.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ArgumentView {
    /// A formulation argument, optionally paired with rendered LaTeX.
    Formulation {
        /// Raw formulation text from the source file.
        text: String,
        /// Rendered LaTeX when parsing/rendering succeeded.
        latex: Option<String>,
    },
    /// A plain text argument.
    Text {
        /// Text content with source quoting already removed by the frontend parser.
        text: String,
    },
    /// A nested group argument.
    Group {
        /// Raw nested group heading, if present.
        heading: Option<String>,
        /// Rendered sections inside the nested group.
        sections: Vec<SectionView>,
    },
}

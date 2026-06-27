use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CollectionView {
    pub title: String,
    pub files: Vec<FileView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FileView {
    pub path: String,
    pub items: Vec<GroupView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GroupView {
    pub kind: String,
    pub heading: Option<String>,
    pub heading_latex: Option<String>,
    pub source: String,
    pub sections: Vec<SectionView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SectionView {
    pub label: String,
    pub inline_argument: Option<String>,
    pub inline_latex: Option<String>,
    pub arguments: Vec<ArgumentView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ArgumentView {
    Formulation {
        text: String,
        latex: Option<String>,
    },
    Text {
        text: String,
    },
    Group {
        heading: Option<String>,
        sections: Vec<SectionView>,
    },
}

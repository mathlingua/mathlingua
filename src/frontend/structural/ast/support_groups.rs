/// Nested alias group inside an `Aliases:` section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AliasGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required alias body.
    pub alias: AliasSection,
    /// Optional rendered written form.
    pub written: Option<WrittenSection>,
}

/// Nested symbol group inside a `Provides:` section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required symbol alias body.
    pub symbol: SymbolSection,
    /// Optional rendered written form.
    pub written: Option<WrittenSection>,
}

/// Nested connection group inside a `Provides:` section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Optional connection prose.
    pub connection: ConnectionSection,
    /// Required target text for the connection.
    pub to: ToSection,
    /// Optional `using:` specifications.
    pub using: Option<UsingSection>,
    /// Required meaning text.
    pub means: MeansSection,
    /// Optional signification text.
    pub signifies: Option<SignifiesSection>,
    /// Optional viewability text.
    pub viewable: Option<ViewableSection>,
    /// Optional through text.
    pub through: Option<ThroughSection>,
}

/// Documentation group containing math-mode written rendering text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WrittenGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required written text entries.
    pub written: WrittenSection,
}

/// Documentation group containing plain-text called rendering text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalledGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required called text entries.
    pub called: CalledSection,
    /// Optional math-mode written text bundled with the called form.
    pub written: Option<WrittenSection>,
}

/// Documentation group defining a writing alias and its rendered text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WritingGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required writing alias.
    pub writing: WritingSection,
    /// Required rendered alias text.
    pub as_: AsSection,
}

/// Documentation group containing overview prose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverviewGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required overview text.
    pub overview: OverviewSection,
}

/// Documentation group containing related-entry prose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelatedGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required related text entries.
    pub related: RelatedSection,
}

/// Documentation group containing discoverer prose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscovererGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Optional discoverer text entries.
    pub discoverer: DiscovererSection,
}

/// Justification label group with authoring note text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabelGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Optional label text entries.
    pub label: LabelSection,
    /// Optional by text entries.
    pub by: BySection,
    /// Required comment text.
    pub comment: CommentSection,
}

/// Justification by group with authoring note text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ByGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Optional by text entries.
    pub by: BySection,
    /// Required comment text.
    pub comment: CommentSection,
}

/// Metadata group containing a stable identifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IdGroup {
    /// Required identifier text.
    pub id: IdSection,
}

/// Metadata group containing version text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionGroup {
    /// Required version text.
    pub version: VersionSection,
}

/// Top-level specification group containing numeric-domain items.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecifyGroup {
    /// Required specification items.
    pub specify: SpecifySection,
}


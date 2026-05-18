use super::*;

/// Specification item describing positive integers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositiveIntGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Optional positive text.
    pub positive: PositiveSection,
    /// Optional integer text.
    pub int: IntSection,
    /// Optional descriptive text.
    pub is_: IsSection,
}

/// Specification item describing negative integers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NegativeIntGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Optional negative text.
    pub negative: NegativeSection,
    /// Optional integer text.
    pub int: IntSection,
    /// Optional descriptive text.
    pub is_: IsSection,
}

/// Specification item describing zero.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZeroGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Optional zero text.
    pub zero: ZeroSection,
    /// Optional descriptive text.
    pub is_: IsSection,
}

/// Specification item describing positive decimals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositiveDecimalGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Optional positive text.
    pub positive: PositiveSection,
    /// Optional decimal text.
    pub decimal: DecimalSection,
    /// Optional descriptive text.
    pub is_: IsSection,
}

/// Specification item describing negative decimals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NegativeDecimalGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Optional negative text.
    pub negative: NegativeSection,
    /// Optional decimal text.
    pub decimal: DecimalSection,
    /// Optional descriptive text.
    pub is_: IsSection,
}

/// Top-level person metadata group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonGroup {
    /// Required author heading.
    pub heading: AuthorHeader,
    /// Optional person prose.
    pub person: PersonSection,
    /// Required name text entries.
    pub name: NameSection,
    /// Required biography text.
    pub biography: BiographySection,
}

/// Top-level resource metadata group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceGroup {
    /// Required resource heading.
    pub heading: ResourceHeader,
    /// Required resource item entries.
    pub resource: ResourceSection,
}

/// Resource item containing a title.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceTitleGroup {
    /// Required title text.
    pub title: ResourceTitleSection,
}

/// Resource item containing one or more authors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceAuthorGroup {
    /// Required author text entries.
    pub author: ResourceAuthorSection,
}

/// Resource item containing an offset such as page, chapter, or location.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceOffsetGroup {
    /// Required offset text.
    pub offset: ResourceOffsetSection,
}

/// Resource item containing a URL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceUrlGroup {
    /// Required URL text.
    pub url: ResourceUrlSection,
}

/// Resource item containing a homepage URL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceHomepageGroup {
    /// Required homepage text.
    pub homepage: ResourceHomepageSection,
}

/// Resource item containing a resource type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceTypeGroup {
    /// Required type text.
    pub type_: ResourceTypeSection,
}

/// Resource item containing an edition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceEditionGroup {
    /// Required edition text.
    pub edition: ResourceEditionSection,
}

/// Resource item containing an editor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceEditorGroup {
    /// Required editor text.
    pub editor: ResourceEditorSection,
}

/// Resource item containing an institution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceInstitutionGroup {
    /// Required institution text.
    pub institution: ResourceInstitutionSection,
}

/// Resource item containing a journal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceJournalGroup {
    /// Required journal text.
    pub journal: ResourceJournalSection,
}

/// Resource item containing a publisher.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourcePublisherGroup {
    /// Required publisher text.
    pub publisher: ResourcePublisherSection,
}

/// Resource item containing a volume.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceVolumeGroup {
    /// Required volume text.
    pub volume: ResourceVolumeSection,
}

/// Resource item containing a month.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceMonthGroup {
    /// Required month text.
    pub month: ResourceMonthSection,
}

/// Resource item containing a year.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceYearGroup {
    /// Required year text.
    pub year: ResourceYearSection,
}

/// Resource item containing a description.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceDescriptionGroup {
    /// Required description text.
    pub description: ResourceDescriptionSection,
}

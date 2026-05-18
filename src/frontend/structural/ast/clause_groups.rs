/// Clause group representing logical negation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required negated clause.
    pub not: NotSection,
}

/// Clause group requiring all child clauses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllOfGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required nonempty child clause list.
    pub all_of: AllOfSection,
}

/// Clause group requiring any child clause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnyOfGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required nonempty child clause list.
    pub any_of: AnyOfSection,
}

/// Clause group requiring exactly one child clause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OneOfGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required nonempty child clause list.
    pub one_of: OneOfSection,
}

/// Existential quantifier clause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExistsGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required bound variable/specification.
    pub exists: ExistsSection,
    /// Required predicate clauses.
    pub such_that: SuchThatSection,
}

/// Unique existential quantifier clause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExistsUniqueGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required uniquely bound variable/specification.
    pub exists_unique: ExistsUniqueSection,
    /// Required predicate clauses.
    pub such_that: SuchThatSection,
}

/// Universal quantifier clause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForAllGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required bound variable/specification.
    pub for_all: ForAllSection,
    /// Optional guard clauses.
    pub where_: Option<WhereSection>,
    /// Required consequence clauses.
    pub then: ThenSection,
}

/// Implication clause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required antecedent clauses.
    pub if_: IfSection,
    /// Required consequent clauses.
    pub then: ThenSection,
}

/// Biconditional clause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IffGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required left/right condition clauses.
    pub iff: IffSection,
    /// Required equivalence clauses.
    pub then: ThenSection,
}

/// Piecewise conditional clause.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PiecewiseGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Optional piecewise prose.
    pub piecewise: PiecewiseSection,
    /// Required condition clauses.
    pub if_: IfSection,
    /// Required value/result clauses.
    pub then: ThenSection,
    /// Optional fallback clauses.
    pub else_: Option<ElseSection>,
}

/// Nested given/then clause group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GivenGroup {
    /// Optional local label heading.
    pub heading: Option<LabelHeader>,
    /// Required given statement.
    pub given: GivenClauseSection,
    /// Optional local context clauses.
    pub where_: Option<WhereSection>,
    /// Required consequence clauses.
    pub then: ThenSection,
}


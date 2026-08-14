//! Per-line type information collected as a by-product of the type check.
//!
//! The checker walks a document once, proving facts about every expression it
//! visits.  Those facts are exactly what an editor wants to show for the line
//! the cursor is on, but the walk normally discards them.  A [`TypeRecorder`]
//! installed on the [`SignatureRegistry`] keeps them.
//!
//! Formulation ASTs carry no source positions — the formulation parser splits
//! substrings and gives every node a span relative to its own slice — so a
//! recorded expression cannot be mapped back to a row by its span.  Instead the
//! recorder is seeded from the proto parse, which *does* know the row of every
//! formulation: each row's text is parsed on its own and the resulting AST is
//! matched against the walk's expressions by structural equality.  Both sides
//! parse the same text with the same parser, so the match is exact.

use super::*;

use std::collections::BTreeMap;
use std::ops::Range;

use crate::frontend::formulation::parse_ordinary_declaration_statement;

/// One expression, sub-expression, or statement resolved on a source line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeEntry {
    /// Nesting depth beneath the line's outermost formulation, so a renderer can
    /// indent sub-expressions under the expression that contains them.
    pub depth: usize,
    /// The (sub)expression itself. The outermost entry echoes the source text
    /// verbatim; nested entries are rendered from the AST.
    pub text: String,
    /// The resolved types, already deduplicated and sorted. Empty when the
    /// checker knows nothing about the expression.
    pub types: Vec<String>,
}

/// Zero-based source row to the entries resolved on it. Rows carrying no
/// formulation the checker reached are absent rather than present and empty.
pub type DocumentTypeInfo = BTreeMap<usize, Vec<TypeEntry>>;

/// The AST a line's formulation parses to. A line is registered under every
/// form it parses as, because which one the structural parser chose depends on
/// the section the formulation appears in.
enum TargetForm {
    Expression(Box<Expression>),
    Declaration(Box<DeclarationStatement>),
}

/// A line the recorder is looking for.
struct Target {
    row: usize,
    /// The formulation exactly as written, used as the outermost entry's label
    /// so the popup echoes the author's own spelling rather than a normalized
    /// rendering of it.
    text: String,
    forms: Vec<TargetForm>,
    matched: bool,
}

/// A line the walk has reached, whose entries are yet to be filed.
pub(super) struct Claim {
    row: usize,
    /// The line's source text, used as the outermost entry's label.
    pub(super) text: String,
}

/// Collects the resolved type of every expression on the lines it was seeded
/// with, while the type checker walks a document.
///
/// The checker reaches a line's formulation through several paths — a `satisfies:`
/// clause is checked, a `when:` clause is assumed — and only some of those
/// recurse into every sub-expression. So the recorder deals only in whole lines:
/// a path that reaches a line claims it, then records the line's entire
/// expression tree in one go against the context in force there.
pub(super) struct TypeRecorder {
    targets: Vec<Target>,
    /// Rows of the top-level item currently being walked. Only targets inside it
    /// are eligible, so a formulation spelled the same way in two items cannot
    /// claim the other item's line.
    item: Range<usize>,
    info: DocumentTypeInfo,
}

impl TypeRecorder {
    /// Seeds a recorder with every formulation in `source`, located by row.
    pub(super) fn new(source: &str) -> Self {
        Self {
            targets: collect_targets(source),
            item: 0..usize::MAX,
            info: DocumentTypeInfo::new(),
        }
    }

    /// Restricts matching to the rows of the top-level item about to be walked.
    pub(super) fn begin_item(&mut self, rows: Range<usize>) {
        self.item = rows;
    }

    pub(super) fn claim_expression(&mut self, expression: &Expression) -> Option<Claim> {
        self.claim(|form| match form {
            TargetForm::Expression(target) => target.as_ref() == expression,
            TargetForm::Declaration(_) => false,
        })
    }

    pub(super) fn claim_declaration(&mut self, statement: &DeclarationStatement) -> Option<Claim> {
        self.claim(|form| match form {
            TargetForm::Declaration(target) => target.as_ref() == statement,
            TargetForm::Expression(_) => false,
        })
    }

    /// Claims the not-yet-recorded line in the current item whose formulation
    /// `matches`, if there is one.
    fn claim(&mut self, matches: impl Fn(&TargetForm) -> bool) -> Option<Claim> {
        let item = self.item.clone();
        let target = self.targets.iter_mut().find(|target| {
            !target.matched && item.contains(&target.row) && target.forms.iter().any(&matches)
        })?;
        target.matched = true;
        Some(Claim {
            row: target.row,
            text: target.text.clone(),
        })
    }

    pub(super) fn record(&mut self, claim: Claim, entries: Vec<TypeEntry>) {
        if entries.is_empty() {
            return;
        }
        self.info.insert(claim.row, entries);
    }

    pub(super) fn finish(self) -> DocumentTypeInfo {
        self.info
    }
}

/// Every formulation in `source`, parsed on its own and tagged with its row.
///
/// Parse failures are dropped silently: a formulation the checker cannot read is
/// already being reported as a diagnostic, and a line with no target simply has
/// no type information.
fn collect_targets(source: &str) -> Vec<Target> {
    let mut discarded = EventLog::new();
    let groups = ProtoParser::new(source, &mut discarded).parse();

    let mut targets = Vec::new();
    for group in &groups {
        collect_group_targets(group, &mut targets);
    }
    targets.sort_by_key(|target| target.row);
    targets
}

fn collect_group_targets(group: &ProtoGroup, targets: &mut Vec<Target>) {
    for section in &group.sections {
        if let Some(inline) = section.inline_argument.as_deref() {
            push_target(section.metadata.row, inline, targets);
        }
        for argument in &section.arguments {
            match argument {
                ProtoArgument::Formulation(formulation) => {
                    push_target(formulation.metadata.row, &formulation.text, targets);
                }
                ProtoArgument::Group(nested) => collect_group_targets(nested, targets),
                ProtoArgument::Text(_) => {}
            }
        }
    }
}

fn push_target(row: usize, text: &str, targets: &mut Vec<Target>) {
    let text = text.trim();
    if text.is_empty() {
        return;
    }

    let mut forms = Vec::new();
    if let Ok(statement) = parse_ordinary_declaration_statement(text) {
        forms.push(TargetForm::Declaration(Box::new(statement)));
    }
    if let Ok(expression) = parse_expression(text) {
        forms.push(TargetForm::Expression(Box::new(expression)));
    }
    if forms.is_empty() {
        return;
    }

    targets.push(Target {
        row,
        text: text.to_owned(),
        forms,
        matched: false,
    });
}

/// Renders a fact as a predicate about its (implied) subject — `is \real`
/// rather than `x is \real` — because the entry it annotates already names the
/// expression the fact is about.
pub(super) fn format_fact_predicate(fact: &TypeFact) -> String {
    match fact {
        TypeFact::Is { ty, .. } | TypeFact::RefinedIs { ty, .. } => format!("is {ty}"),
        TypeFact::Spec {
            operator, target, ..
        } => format!("\"{operator}\" {target}"),
        TypeFact::InfixSpec {
            signature,
            args,
            target,
            ..
        } => {
            let rendered_args = if args.is_empty() {
                String::new()
            } else {
                format!("{{{}}}", args.join(", "))
            };
            format!("{signature}{rendered_args} {target}")
        }
        TypeFact::MemberOf { collection, .. } => format!("in {collection}"),
        TypeFact::FunctionType { inputs, output, .. } => format!(
            "is ({}) => ({})",
            inputs
                .iter()
                .map(format_spec_predicate)
                .collect::<Vec<_>>()
                .join(", "),
            format_spec_predicate(output)
        ),
    }
}

fn format_spec_predicate(spec: &FunctionTypeFactSpec) -> String {
    match spec {
        FunctionTypeFactSpec::Is { ty, .. } => format!("_ is {ty}"),
        FunctionTypeFactSpec::Spec { operator, target } => format!("_ \"{operator}\" {target}"),
    }
}

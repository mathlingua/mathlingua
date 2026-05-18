use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::events::{EventLocation, EventLog, EventPosition, EventSpan, Level};
use crate::frontend::formulation::ast::{
    Chain, ChainPart, CommandExpression, CommandExpressionTailPart, CommandHeader,
    CommandHeaderNode, CommandHeaderTailPart, Expression, ExpressionKind, FormOrDeclaration,
    FormOrDeclarationKind, InfixCommand, InfixCommandHeader, IsOrRefinedStatementSpec, IsOrSpec,
    IsStatement, IsSubject, IsSubjectForm, IsSubjectKind, PlaceholderForm, PlaceholderFormKind,
    RefinedCommandExpression, RefinedCommandHeader, RefinedExpressionPart, RefinedTail,
    SpecSubject, SpecSubjectKind, TupleExpressionElement, TupleFormElement, TypeExpression,
};
use crate::frontend::structural::ast::*;

/// Event origin used for all diagnostics produced by the semantic checker.
const ORIGIN: &str = "semantic_check";

/// A source file after the frontend has parsed it into a structural document.
///
/// Backend passes operate on this type instead of repeatedly reading or
/// reparsing files.  It keeps the original source text for diagnostic location
/// lookup, the filesystem path for reporting, and the parsed structural AST for
/// semantic traversal.
#[derive(Clone, Debug)]
pub struct ParsedSourceFile {
    /// Path of the file on disk, used in diagnostics and duplicate reports.
    pub path: PathBuf,
    /// Original file contents, used to recover line and column information.
    pub source: String,
    /// Structural representation produced by the frontend parser.
    pub document: Document,
}

/// Canonical command signature plus the argument groups required by that form.
///
/// The signature intentionally strips concrete `{...}` and `(...)` contents, so
/// `\function{A, B}` and `\function{X}` share `\function`.  The accompanying
/// `arg_groups` records how many arguments each group must contain, while
/// `fallback_shapes` records secondary shapes that a composed refined command
/// may legally refer to.
#[derive(Clone, Debug, PartialEq, Eq)]
struct SignatureShape {
    /// Canonical signature text, such as `\function:on:to`.
    signature: String,
    /// Ordered argument groups that must be supplied for this signature.
    arg_groups: Vec<ArgGroupShape>,
    /// Alternate shapes to validate when a combined refined command is absent.
    fallback_shapes: Vec<SignatureShape>,
}

/// Delimiter used by one command argument group.
///
/// Curly groups are required whenever present in a definition.  Parenthesized
/// groups are optional for use sites because they represent invocation of an
/// already-described callable object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArgDelimiter {
    /// A required `{...}` argument group.
    Curly,
    /// An optional-at-use-site `(...)` argument group.
    Paren,
}

/// Shape of one argument group in a command definition or reference.
///
/// Only the delimiter and arity are tracked here.  The semantic checker does not
/// yet type-check the individual argument expressions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ArgGroupShape {
    /// Whether the group was written with `{}` or `()`.
    delimiter: ArgDelimiter,
    /// Number of comma-separated arguments inside the group.
    count: usize,
}

/// Registered definition metadata for one unique command signature.
///
/// This is the source of truth used to detect duplicate signatures and to check
/// references for undefined commands or argument-shape mismatches.
#[derive(Clone, Debug)]
struct DefinitionEntry {
    /// Structural kind that introduced the signature.
    kind: DefinitionKind,
    /// Canonical signature and expected argument shape.
    shape: SignatureShape,
    /// Source path where the definition was found.
    path: PathBuf,
    /// Best-effort source location of the command in the original file.
    position: Option<SourcePosition>,
}

/// Top-level structural groups that can introduce a command signature.
///
/// The uniqueness rule is global across these kinds: the same signature cannot
/// be reused merely because it appears in a different group type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DefinitionKind {
    /// A `Describes` group.
    Describes,
    /// A `Defines` group.
    Defines,
    /// A `Refines` group.
    Refines,
    /// A `States` group.
    States,
    /// A named `Axiom`.
    Axiom,
    /// A named `Theorem`.
    Theorem,
    /// A named `Corollary`.
    Corollary,
    /// A named `Lemma`.
    Lemma,
    /// A named `Conjecture`.
    Conjecture,
}

impl DefinitionKind {
    /// Returns the user-facing label used in diagnostics for this definition kind.
    fn label(self) -> &'static str {
        match self {
            Self::Describes => "Describes",
            Self::Defines => "Defines",
            Self::Refines => "Refines",
            Self::States => "States",
            Self::Axiom => "Axiom",
            Self::Theorem => "Theorem",
            Self::Corollary => "Corollary",
            Self::Lemma => "Lemma",
            Self::Conjecture => "Conjecture",
        }
    }
}

/// Index of all definitions visible to a semantic checking run.
///
/// The key is the canonical command signature.  Because overloading is not
/// currently supported, one entry per signature is sufficient and duplicates can
/// be reported immediately during collection.
#[derive(Default)]
struct SignatureRegistry {
    /// Map from canonical command signature to the definition that owns it.
    definitions: HashMap<String, DefinitionEntry>,
}

/// Runs semantic checks across a set of parsed MathLingua source files.
///
/// This is intentionally a two-pass check.  The first pass collects all command
/// definitions so cross-file references can be resolved independent of file
/// order.  The second pass walks every expression/specification and validates
/// command existence plus argument shape.
pub fn check_documents(files: &[ParsedSourceFile], event_log: &mut EventLog) {
    let mut registry = SignatureRegistry::default();
    for file in files {
        collect_document_definitions(file, &mut registry, event_log);
    }

    for file in files {
        validate_document_references(file, &registry, event_log);
    }
}

/// Collects every signature-defining top-level item from a single document.
///
/// During collection this also performs checks that are naturally tied to the
/// definition itself: duplicate signatures and required documented rendering
/// metadata for `Defines`, `Describes`, and `Refines`.
fn collect_document_definitions(
    file: &ParsedSourceFile,
    registry: &mut SignatureRegistry,
    event_log: &mut EventLog,
) {
    let mut locator = SourceLocator::new(&file.source);
    for item in &file.document.items {
        let Some(definition) = definition_item(item) else {
            continue;
        };
        let kind = definition.kind;
        let shape = shape_for_header(definition.heading);
        let position = locator.locate_heading(&shape);
        check_documented_rendering(file, kind, definition.documented, position, event_log);
        if let Some(previous) = registry.definitions.get(&shape.signature) {
            emit_error(
                event_log,
                &file.path,
                position,
                format!(
                    "Duplicate command signature `{}` in {}; previously defined as {} in {}",
                    shape.signature,
                    kind.label(),
                    previous.kind.label(),
                    display_definition_location(previous)
                ),
            );
            continue;
        }

        registry.definitions.insert(
            shape.signature.clone(),
            DefinitionEntry {
                kind,
                shape,
                path: file.path.clone(),
                position,
            },
        );
    }
}

/// Borrowed view of the definition-relevant pieces of a top-level item.
///
/// Different structural group types store their heading and documented section
/// under different field names.  This adapter lets the collection pass treat
/// them uniformly.
struct DefinitionItem<'a> {
    /// Kind of top-level group that owns this definition.
    kind: DefinitionKind,
    /// Parsed command header that defines the signature.
    heading: &'a CommandHeader,
    /// Optional documented section for rendering metadata checks.
    documented: Option<&'a DocumentedSection>,
}

/// Extracts definition metadata from top-level items that introduce signatures.
///
/// Anonymous theorem-like groups do not define a command signature, so they are
/// ignored here unless they have an explicit heading.
fn definition_item(item: &TopLevelItem) -> Option<DefinitionItem<'_>> {
    match item {
        TopLevelItem::Describes(group) => Some(DefinitionItem {
            kind: DefinitionKind::Describes,
            heading: &group.heading,
            documented: group.documented.as_ref(),
        }),
        TopLevelItem::Defines(group) => Some(DefinitionItem {
            kind: DefinitionKind::Defines,
            heading: &group.heading,
            documented: group.documented.as_ref(),
        }),
        TopLevelItem::Refines(group) => Some(DefinitionItem {
            kind: DefinitionKind::Refines,
            heading: &group.heading,
            documented: group.documented.as_ref(),
        }),
        TopLevelItem::States(group) => Some(DefinitionItem {
            kind: DefinitionKind::States,
            heading: &group.heading,
            documented: group.documented.as_ref(),
        }),
        TopLevelItem::Axiom(group) => group.heading.as_ref().map(|heading| DefinitionItem {
            kind: DefinitionKind::Axiom,
            heading,
            documented: group.documented.as_ref(),
        }),
        TopLevelItem::Theorem(group) => group.heading.as_ref().map(|heading| DefinitionItem {
            kind: DefinitionKind::Theorem,
            heading,
            documented: group.documented.as_ref(),
        }),
        TopLevelItem::Corollary(group) => group.heading.as_ref().map(|heading| DefinitionItem {
            kind: DefinitionKind::Corollary,
            heading,
            documented: group.documented.as_ref(),
        }),
        TopLevelItem::Lemma(group) => group.heading.as_ref().map(|heading| DefinitionItem {
            kind: DefinitionKind::Lemma,
            heading,
            documented: group.documented.as_ref(),
        }),
        TopLevelItem::Conjecture(group) => group.heading.as_ref().map(|heading| DefinitionItem {
            kind: DefinitionKind::Conjecture,
            heading,
            documented: group.documented.as_ref(),
        }),
        _ => None,
    }
}

/// Verifies that renderable definition groups provide a documented `called:` item.
///
/// Only `Defines`, `Describes`, and `Refines` currently participate in command
/// rendering.  Theorem-like groups may have headings for reference purposes but
/// do not need `Documented:` rendering metadata.
fn check_documented_rendering(
    file: &ParsedSourceFile,
    kind: DefinitionKind,
    documented: Option<&DocumentedSection>,
    position: Option<SourcePosition>,
    event_log: &mut EventLog,
) {
    if !matches!(
        kind,
        DefinitionKind::Describes | DefinitionKind::Defines | DefinitionKind::Refines
    ) {
        return;
    }

    let has_called = documented.is_some_and(|section| {
        section
            .arguments
            .iter()
            .any(|item| matches!(item, DocumentedItem::Called(_)))
    });

    if !has_called {
        emit_error(
            event_log,
            &file.path,
            position,
            format!(
                "{} entries must include a `called:` item in `Documented:`",
                kind.label()
            ),
        );
    }
}

/// Walks a document and validates every command-like reference against the registry.
///
/// The traversal produces signature shapes for commands wherever they can appear:
/// formulas, clauses, type expressions, aliases, and theorem-like statements.
/// Source locations are recovered separately from the original text so errors can
/// point at the actual reference token.
fn validate_document_references(
    file: &ParsedSourceFile,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let mut locator = SourceLocator::new(&file.source);
    for item in &file.document.items {
        walk_top_level_item(item, &mut |shape| {
            validate_reference_shape(
                file.path.as_path(),
                locator.locate_reference(shape),
                shape,
                registry,
                event_log,
            );
        });
    }
}

/// Validates a single reference shape for definition existence and argument arity.
///
/// Refined commands can have fallback shapes.  When a composed refined command is
/// not defined directly, those fallbacks allow the checker to validate the base
/// command and individual refinement pieces instead of reporting a premature
/// undefined-signature error.
fn validate_reference_shape(
    path: &Path,
    position: Option<SourcePosition>,
    shape: &SignatureShape,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let Some(definition) = registry.definitions.get(&shape.signature) else {
        if !shape.fallback_shapes.is_empty() {
            for fallback in &shape.fallback_shapes {
                validate_reference_shape(path, position, fallback, registry, event_log);
            }
            return;
        }
        emit_error(
            event_log,
            path,
            position,
            format!("Undefined command signature `{}`", shape.signature),
        );
        return;
    };

    if !argument_groups_match(&definition.shape.arg_groups, &shape.arg_groups) {
        emit_error(
            event_log,
            path,
            position,
            format!(
                "Command signature `{}` expects argument shape `{}` but found `{}`",
                shape.signature,
                format_arg_groups(&definition.shape.arg_groups),
                format_arg_groups(&shape.arg_groups)
            ),
        );
    }
}

/// Compares expected and actual argument groups, honoring optional invocation groups.
///
/// Exact equality is accepted.  A use site may also omit trailing parenthesized
/// groups, because definitions such as `\some.function{A}(x, y)` can be referred
/// to either as the function object `\some.function{A}` or as the invocation
/// `\some.function{A}(x, y)`.
fn argument_groups_match(expected: &[ArgGroupShape], actual: &[ArgGroupShape]) -> bool {
    if expected == actual {
        return true;
    }

    let Some(remaining_expected) = expected.strip_prefix(actual) else {
        return false;
    };

    !remaining_expected.is_empty()
        && remaining_expected
            .iter()
            .all(|group| group.delimiter == ArgDelimiter::Paren)
}

/// Builds the canonical signature shape for any definition command header.
fn shape_for_header(header: &CommandHeader) -> SignatureShape {
    match header {
        CommandHeader::Command(command) => shape_for_command_header_node(command),
        CommandHeader::Infix(command) => shape_for_infix_command_header(command),
        CommandHeader::Refined(command) => shape_for_refined_command_header(command),
    }
}

/// Builds a signature shape for a normal prefix command header.
///
/// The resulting signature preserves the command chain and tail labels while
/// replacing each argument group with only its delimiter and argument count.
fn shape_for_command_header_node(command: &CommandHeaderNode) -> SignatureShape {
    let mut signature = format!("\\{}", format_chain(&command.chain));
    let mut arg_groups = Vec::new();
    add_heading_curly_groups(&mut arg_groups, &command.head_args);
    add_header_tail(&mut signature, &mut arg_groups, &command.tail);
    for args in &command.paren_args {
        arg_groups.push(ArgGroupShape {
            delimiter: ArgDelimiter::Paren,
            count: args.forms.len(),
        });
    }
    SignatureShape {
        signature,
        arg_groups,
        fallback_shapes: Vec::new(),
    }
}

/// Builds a signature shape for an infix command header.
///
/// Infix commands are wrapped in `\:...:/` so they cannot collide with ordinary
/// prefix command signatures.
fn shape_for_infix_command_header(command: &InfixCommandHeader) -> SignatureShape {
    let mut signature = format!("\\:{}", format_chain(&command.chain));
    let mut arg_groups = Vec::new();
    add_heading_curly_groups(&mut arg_groups, &command.head_args);
    add_header_tail(&mut signature, &mut arg_groups, &command.tail);
    signature.push_str(":/");
    SignatureShape {
        signature,
        arg_groups,
        fallback_shapes: Vec::new(),
    }
}

/// Builds a signature shape for a refined command definition header.
///
/// Refined headers combine optional prefixes, one or more refinement parts, and
/// the refined tail into a single signature such as
/// `\(continuous)::function:on:to`.
fn shape_for_refined_command_header(command: &RefinedCommandHeader) -> SignatureShape {
    let mut signature = "\\".to_string();
    if let Some(prefix) = &command.prefix_chain {
        signature.push_str(&format_chain(prefix));
        signature.push_str("::");
    }
    let mut arg_groups = Vec::new();
    for (index, part) in command.parts.iter().enumerate() {
        if index > 0 {
            signature.push_str("::");
        }
        signature.push_str(&format_chain(&part.chain));
        add_header_tail(&mut signature, &mut arg_groups, &part.tail);
    }
    signature.push_str("::");
    signature.push_str(&format_refined_tail(&command.refined_tail));
    add_heading_curly_groups(&mut arg_groups, &command.head_args);
    add_header_tail(&mut signature, &mut arg_groups, &command.tail);
    for args in &command.paren_args {
        arg_groups.push(ArgGroupShape {
            delimiter: ArgDelimiter::Paren,
            count: args.forms.len(),
        });
    }
    SignatureShape {
        signature,
        arg_groups,
        fallback_shapes: Vec::new(),
    }
}

/// Builds a signature shape for a normal command expression use.
///
/// This mirrors `shape_for_command_header_node` but counts expression arguments
/// instead of declaration/form arguments.
fn shape_for_command_expression(command: &CommandExpression) -> SignatureShape {
    let mut signature = format!("\\{}", format_chain(&command.chain));
    let mut arg_groups = Vec::new();
    add_expression_curly_groups(&mut arg_groups, &command.head_args);
    add_expression_tail(&mut signature, &mut arg_groups, &command.tail);
    for args in &command.paren_args {
        arg_groups.push(ArgGroupShape {
            delimiter: ArgDelimiter::Paren,
            count: args.expressions.len(),
        });
    }
    SignatureShape {
        signature,
        arg_groups,
        fallback_shapes: Vec::new(),
    }
}

/// Builds a signature shape for an infix command expression use.
fn shape_for_infix_command(command: &InfixCommand) -> SignatureShape {
    let mut signature = format!("\\:{}", format_chain(&command.chain));
    let mut arg_groups = Vec::new();
    add_expression_curly_groups(&mut arg_groups, &command.head_args);
    add_expression_tail(&mut signature, &mut arg_groups, &command.tail);
    signature.push_str(":/");
    SignatureShape {
        signature,
        arg_groups,
        fallback_shapes: Vec::new(),
    }
}

/// Builds a signature shape for a refined command expression use.
///
/// The primary shape represents the full composed command.  Additional fallback
/// shapes are attached so a combined use can be accepted when the base command
/// and each refinement piece are defined separately.
fn shape_for_refined_command_expression(command: &RefinedCommandExpression) -> SignatureShape {
    let mut signature = "\\".to_string();
    if let Some(prefix) = &command.prefix_chain {
        signature.push_str(&format_chain(prefix));
        signature.push_str("::");
    }
    let mut arg_groups = Vec::new();
    for (index, part) in command.parts.iter().enumerate() {
        if index > 0 {
            signature.push_str("::");
        }
        signature.push_str(&format_chain(&part.chain));
        add_expression_tail(&mut signature, &mut arg_groups, &part.tail);
    }
    signature.push_str("::");
    signature.push_str(&format_refined_tail(&command.refined_tail));
    add_expression_curly_groups(&mut arg_groups, &command.head_args);
    add_expression_tail(&mut signature, &mut arg_groups, &command.tail);
    for args in &command.paren_args {
        arg_groups.push(ArgGroupShape {
            delimiter: ArgDelimiter::Paren,
            count: args.expressions.len(),
        });
    }
    let mut shape = SignatureShape {
        signature,
        arg_groups,
        fallback_shapes: Vec::new(),
    };
    shape.fallback_shapes = fallback_shapes_for_refined_command_expression(command);
    shape
}

/// Produces fallback validation shapes for a refined command expression.
///
/// The first fallback is the refined base command; subsequent fallbacks represent
/// individual refinement pieces applied to that same base.
fn fallback_shapes_for_refined_command_expression(
    command: &RefinedCommandExpression,
) -> Vec<SignatureShape> {
    let mut shapes = vec![shape_for_refined_command_base(command)];
    shapes.extend(
        command
            .parts
            .iter()
            .map(|part| shape_for_refined_command_part(command, part)),
    );
    shapes
}

/// Builds the fallback shape for the base command of a refined expression.
fn shape_for_refined_command_base(command: &RefinedCommandExpression) -> SignatureShape {
    let mut signature = format!("\\{}", format_refined_tail(&command.refined_tail));
    let mut arg_groups = Vec::new();
    add_expression_curly_groups(&mut arg_groups, &command.head_args);
    add_expression_tail(&mut signature, &mut arg_groups, &command.tail);
    add_expression_paren_groups(&mut arg_groups, &command.paren_args);
    SignatureShape {
        signature,
        arg_groups,
        fallback_shapes: Vec::new(),
    }
}

/// Builds the fallback shape for one refinement part applied to a base command.
///
/// For a use such as `\(continuous)::function:on{A}:to{B}`, this constructs the
/// piece signature `\(continuous)::function:on:to` and attaches the argument
/// shape needed to validate that specific refinement definition.
fn shape_for_refined_command_part(
    command: &RefinedCommandExpression,
    part: &RefinedExpressionPart,
) -> SignatureShape {
    let mut signature = "\\".to_string();
    if let Some(prefix) = &command.prefix_chain {
        signature.push_str(&format_chain(prefix));
        signature.push_str("::");
    }
    signature.push_str(&format_chain(&part.chain));
    let mut arg_groups = Vec::new();
    add_expression_tail(&mut signature, &mut arg_groups, &part.tail);
    signature.push_str("::");
    signature.push_str(&format_refined_tail(&command.refined_tail));
    add_expression_curly_groups(&mut arg_groups, &command.head_args);
    add_expression_tail(&mut signature, &mut arg_groups, &command.tail);
    add_expression_paren_groups(&mut arg_groups, &command.paren_args);
    SignatureShape {
        signature,
        arg_groups,
        fallback_shapes: Vec::new(),
    }
}

/// Appends curly argument-group shapes from a command header.
///
/// Header groups contain forms/declarations, so their arity is counted from the
/// `forms` collection rather than expression values.
fn add_heading_curly_groups(
    arg_groups: &mut Vec<ArgGroupShape>,
    groups: &[crate::frontend::formulation::ast::CurlyHeadingArgs],
) {
    for args in groups {
        arg_groups.push(ArgGroupShape {
            delimiter: ArgDelimiter::Curly,
            count: args.forms.len(),
        });
    }
}

/// Appends curly argument-group shapes from a command expression.
fn add_expression_curly_groups(
    arg_groups: &mut Vec<ArgGroupShape>,
    groups: &[crate::frontend::formulation::ast::CurlyExpressionArgs],
) {
    for args in groups {
        arg_groups.push(ArgGroupShape {
            delimiter: ArgDelimiter::Curly,
            count: args.expressions.len(),
        });
    }
}

/// Appends parenthesized argument-group shapes from a command expression.
fn add_expression_paren_groups(
    arg_groups: &mut Vec<ArgGroupShape>,
    groups: &[crate::frontend::formulation::ast::ParenExpressionArgs],
) {
    for args in groups {
        arg_groups.push(ArgGroupShape {
            delimiter: ArgDelimiter::Paren,
            count: args.expressions.len(),
        });
    }
}

/// Extends a header signature with tail labels and their argument shapes.
///
/// Tail labels contribute to the canonical signature as `:label` segments, while
/// their argument lists contribute only arity metadata.
fn add_header_tail(
    signature: &mut String,
    arg_groups: &mut Vec<ArgGroupShape>,
    tail: &[CommandHeaderTailPart],
) {
    for part in tail {
        signature.push(':');
        signature.push_str(&format_chain(&part.chain));
        add_heading_curly_groups(arg_groups, &part.args);
    }
}

/// Extends an expression signature with tail labels and their argument shapes.
fn add_expression_tail(
    signature: &mut String,
    arg_groups: &mut Vec<ArgGroupShape>,
    tail: &[CommandExpressionTailPart],
) {
    for part in tail {
        signature.push(':');
        signature.push_str(&format_chain(&part.chain));
        add_expression_curly_groups(arg_groups, &part.args);
    }
}

/// Converts a parsed command/name chain into its canonical dotted text form.
fn format_chain(chain: &Chain) -> String {
    chain
        .parts
        .iter()
        .map(format_chain_part)
        .collect::<Vec<_>>()
        .join(".")
}

/// Converts one chain part into the text used in canonical signatures.
fn format_chain_part(part: &ChainPart) -> String {
    match part {
        ChainPart::Name(name) => name.clone(),
        ChainPart::Alias(name) => format!("${name}"),
        ChainPart::Operator(operator) => operator.clone(),
    }
}

/// Converts the base portion of a refined command into canonical signature text.
fn format_refined_tail(tail: &RefinedTail) -> String {
    match tail {
        RefinedTail::Chain(chain) => format_chain(chain),
        RefinedTail::Name { name, .. } => name.clone(),
    }
}

/// Formats an argument-shape list for user-facing diagnostics.
///
/// Examples are `none`, `{2}`, `{1}(2)`, or `{1}:...` depending on the groups
/// present.  Only delimiter kind and count are displayed because semantic type
/// information is not available at this layer yet.
fn format_arg_groups(groups: &[ArgGroupShape]) -> String {
    if groups.is_empty() {
        return "none".to_string();
    }

    groups
        .iter()
        .map(|group| match group.delimiter {
            ArgDelimiter::Curly => format!("{{{}}}", group.count),
            ArgDelimiter::Paren => format!("({})", group.count),
        })
        .collect::<Vec<_>>()
        .join("")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Zero-based source position used internally before converting to event spans.
struct SourcePosition {
    /// Zero-based row in the source file.
    row: usize,
    /// Zero-based Unicode scalar column in the row.
    column: usize,
}

/// Incremental source locator for matching parsed signatures back to raw text.
///
/// The frontend AST currently does not carry exact spans for every backend
/// diagnostic, so this locator scans the original source in order.  Separate
/// cursors are maintained for definitions and references to avoid repeatedly
/// reporting the first matching occurrence.
struct SourceLocator<'a> {
    /// Complete original source text for the file being checked.
    source: &'a str,
    /// Next byte offset to scan from when locating definition headings.
    heading_cursor: usize,
    /// Next byte offset to scan from when locating command references.
    reference_cursor: usize,
}

impl<'a> SourceLocator<'a> {
    /// Creates a locator for one source file.
    fn new(source: &'a str) -> Self {
        Self {
            source,
            heading_cursor: 0,
            reference_cursor: 0,
        }
    }

    /// Finds the next heading occurrence matching a signature shape.
    fn locate_heading(&mut self, shape: &SignatureShape) -> Option<SourcePosition> {
        let offset = find_signature_occurrence(
            self.source,
            shape,
            self.heading_cursor,
            OccurrenceKind::Heading,
        )?;
        self.heading_cursor = offset.saturating_add(1);
        Some(position_at_offset(self.source, offset))
    }

    /// Finds the next non-heading occurrence matching a signature shape.
    fn locate_reference(&mut self, shape: &SignatureShape) -> Option<SourcePosition> {
        let offset = find_signature_occurrence(
            self.source,
            shape,
            self.reference_cursor,
            OccurrenceKind::Reference,
        )?;
        self.reference_cursor = offset.saturating_add(1);
        Some(position_at_offset(self.source, offset))
    }
}

#[derive(Clone, Copy)]
/// Kind of source occurrence the locator should search for.
enum OccurrenceKind {
    /// Match command signatures inside bracketed headings.
    Heading,
    /// Match command signatures anywhere outside bracketed headings.
    Reference,
}

/// Finds the next byte offset where a signature shape appears in raw source text.
///
/// This scanner is deliberately conservative: it starts only at backslash tokens,
/// separates headings from references, and delegates argument skipping so
/// signatures such as `\function:on:to` can match text like
/// `\function:on{A}:to{B}`.
fn find_signature_occurrence(
    source: &str,
    shape: &SignatureShape,
    start: usize,
    kind: OccurrenceKind,
) -> Option<usize> {
    for (relative, _) in source.get(start..)?.match_indices('\\') {
        let offset = start + relative;
        let is_heading = is_heading_line(source, offset);
        match kind {
            OccurrenceKind::Heading if !is_heading => continue,
            OccurrenceKind::Reference if is_heading => continue,
            _ => {}
        }
        if matches_signature_at(source, offset, &shape.signature) {
            return Some(offset);
        }
    }
    None
}

/// Returns true when a byte offset belongs to a bracketed command heading line.
fn is_heading_line(source: &str, offset: usize) -> bool {
    let line_start = source[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = source[offset..]
        .find('\n')
        .map(|i| offset + i)
        .unwrap_or(source.len());
    let line = source[line_start..line_end].trim();
    line.starts_with('[') && line.ends_with(']')
}

/// Checks whether a canonical signature matches source text at a byte offset.
///
/// Normal command signatures skip over concrete argument groups between tail
/// labels.  Refined and infix signatures are currently matched as direct text
/// because their canonical text contains enough punctuation to avoid prefix
/// ambiguity.
fn matches_signature_at(source: &str, offset: usize, signature: &str) -> bool {
    if signature.starts_with("\\:") || signature.contains("::") {
        return source
            .get(offset..)
            .is_some_and(|tail| tail.starts_with(signature));
    }

    let parts: Vec<&str> = signature.split(':').collect();
    let Some(first) = parts.first() else {
        return false;
    };
    let Some(mut remaining) = source.get(offset..) else {
        return false;
    };
    if !remaining.starts_with(first) {
        return false;
    }
    remaining = &remaining[first.len()..];
    remaining = skip_argument_groups(remaining);

    for part in parts.iter().skip(1) {
        let Some(after_colon) = remaining.strip_prefix(':') else {
            return false;
        };
        if !after_colon.starts_with(part) {
            return false;
        }
        remaining = &after_colon[part.len()..];
        remaining = skip_argument_groups(remaining);
    }

    !remaining
        .chars()
        .next()
        .is_some_and(|ch| ch == ':' || ch == '.' || ch == '_' || ch.is_ascii_alphanumeric())
}

/// Skips any immediately adjacent balanced `{...}` or `(...)` groups.
fn skip_argument_groups(mut input: &str) -> &str {
    loop {
        let Some(open) = input.chars().next() else {
            return input;
        };
        let close = match open {
            '{' => '}',
            '(' => ')',
            _ => return input,
        };
        let Some(end) = find_balanced_group_end(input, open, close) else {
            return input;
        };
        input = &input[end..];
    }
}

/// Returns the byte index just after a balanced delimiter group.
///
/// Nested groups of the same delimiter are handled so a command argument can
/// contain structured expressions without breaking signature matching.
fn find_balanced_group_end(input: &str, open: char, close: char) -> Option<usize> {
    let mut depth = 0usize;
    for (index, ch) in input.char_indices() {
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index + ch.len_utf8());
            }
        }
    }
    None
}

/// Converts a byte offset into a zero-based row and column position.
fn position_at_offset(source: &str, offset: usize) -> SourcePosition {
    let mut row = 0usize;
    let mut line_start = 0usize;
    for (index, ch) in source.char_indices() {
        if index >= offset {
            break;
        }
        if ch == '\n' {
            row += 1;
            line_start = index + ch.len_utf8();
        }
    }

    SourcePosition {
        row,
        column: source[line_start..offset].chars().count(),
    }
}

/// Formats a definition location for duplicate-signature diagnostics.
fn display_definition_location(entry: &DefinitionEntry) -> String {
    match entry.position {
        Some(position) => format!(
            "{}:{}:{}",
            entry.path.display(),
            position.row + 1,
            position.column + 1
        ),
        None => entry.path.display().to_string(),
    }
}

/// Emits a user-facing semantic error at an optional source position.
///
/// When a position is unavailable, the diagnostic still points at the owning
/// file so command-line output remains actionable.
fn emit_error(
    event_log: &mut EventLog,
    path: &Path,
    position: Option<SourcePosition>,
    message: impl Into<String>,
) {
    let location = position
        .map(|position| {
            EventLocation::file(
                path.to_path_buf(),
                Some(EventSpan::point(EventPosition::at_row_and_column(
                    position.row,
                    position.column,
                ))),
            )
        })
        .unwrap_or_else(|| EventLocation::file_path(path.to_path_buf()));
    event_log.user_event(Some(ORIGIN), Level::Error, Some(location), message);
}

/// Traverses every command reference that can appear inside a top-level item.
///
/// The visitor receives only signature shapes; it does not need to know the
/// structural context that produced each reference.  This keeps registry
/// validation separate from AST traversal.
fn walk_top_level_item(item: &TopLevelItem, visit: &mut impl FnMut(&SignatureShape)) {
    match item {
        TopLevelItem::Describes(group) => {
            walk_form_or_declaration(&group.describes.argument, visit);
            walk_optional_is_or_specs(&group.using, visit);
            walk_optional_clauses(&group.when, visit);
            if let Some(section) = &group.extends {
                walk_is_or_via_item(&section.argument, visit);
            }
            if let Some(section) = &group.specifies {
                for item in &section.arguments {
                    walk_is_or_via_item(item, visit);
                }
            }
            walk_optional_clauses(&group.satisfies, visit);
            walk_optional_provides(&group.provides, visit);
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::Defines(group) => {
            walk_is_or_spec(&group.defines.argument, visit);
            walk_optional_is_or_specs(&group.using, visit);
            walk_optional_clauses(&group.when, visit);
            if let Some(section) = &group.expresses {
                walk_clause(&section.argument, visit);
            }
            walk_optional_provides(&group.provides, visit);
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::Refines(group) => {
            walk_is_or_refined_spec(&group.refines.argument, visit);
            walk_optional_is_or_specs(&group.using, visit);
            walk_optional_clauses(&group.when, visit);
            if let Some(section) = &group.specifies {
                walk_is_or_refined_spec(&section.argument, visit);
            }
            walk_optional_clauses(&group.satisfies, visit);
            walk_optional_provides(&group.provides, visit);
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::States(group) => {
            walk_optional_is_or_specs(&group.using, visit);
            walk_optional_clauses(&group.when, visit);
            for clause in &group.that.arguments {
                walk_clause(clause, visit);
            }
            walk_optional_provides(&group.provides, visit);
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::Axiom(group) => {
            walk_theorem_like(
                group.given.as_ref(),
                group.where_.as_ref(),
                &group.then,
                group.iff.as_ref(),
                visit,
            );
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::Theorem(group) => {
            walk_theorem_like(
                group.given.as_ref(),
                group.where_.as_ref(),
                &group.then,
                group.iff.as_ref(),
                visit,
            );
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::Corollary(group) => {
            walk_theorem_like(
                group.given.as_ref(),
                group.where_.as_ref(),
                &group.then,
                group.iff.as_ref(),
                visit,
            );
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::Lemma(group) => {
            walk_theorem_like(
                group.given.as_ref(),
                group.where_.as_ref(),
                &group.then,
                group.iff.as_ref(),
                visit,
            );
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::Conjecture(group) => {
            walk_theorem_like(
                group.given.as_ref(),
                group.where_.as_ref(),
                &group.then,
                group.iff.as_ref(),
                visit,
            );
            walk_optional_aliases(&group.aliases, visit);
        }
        TopLevelItem::Specify(group) => {
            for item in &group.specify.arguments {
                walk_specify_item(item, visit);
            }
        }
        TopLevelItem::Title(_)
        | TopLevelItem::Section(_)
        | TopLevelItem::Subsection(_)
        | TopLevelItem::Subsubsection(_)
        | TopLevelItem::Person(_)
        | TopLevelItem::Resource(_) => {}
    }
}

/// Traverses the common sections shared by theorem-like groups.
///
/// `Axiom`, `Theorem`, `Corollary`, `Lemma`, and `Conjecture` all expose the
/// same logical proof/result sections, so this helper centralizes reference
/// discovery for those group kinds.
fn walk_theorem_like(
    given: Option<&GivenSection>,
    where_: Option<&WhereSection>,
    then: &ThenSection,
    iff: Option<&IffSection>,
    visit: &mut impl FnMut(&SignatureShape),
) {
    if let Some(section) = given {
        for spec in &section.arguments {
            walk_is_or_refined_spec(spec, visit);
        }
    }
    if let Some(section) = where_ {
        for clause in &section.arguments {
            walk_clause(clause, visit);
        }
    }
    for clause in &then.arguments {
        walk_clause(clause, visit);
    }
    if let Some(section) = iff {
        for clause in &section.arguments {
            walk_clause(clause, visit);
        }
    }
}

/// Traverses an optional `using:` section containing `is` or `spec` entries.
fn walk_optional_is_or_specs(
    section: &Option<UsingSection>,
    visit: &mut impl FnMut(&SignatureShape),
) {
    if let Some(section) = section {
        for spec in &section.arguments {
            walk_is_or_spec(spec, visit);
        }
    }
}

/// Traverses any optional section that stores a list of logical clauses.
fn walk_optional_clauses<T>(section: &Option<T>, visit: &mut impl FnMut(&SignatureShape))
where
    T: ClauseSection,
{
    if let Some(section) = section {
        for clause in section.clauses() {
            walk_clause(clause, visit);
        }
    }
}

/// Adapter trait for structural sections whose payload is a slice of clauses.
trait ClauseSection {
    /// Returns the clauses contained by the section.
    fn clauses(&self) -> &[Clause];
}

impl ClauseSection for WhenSection {
    /// Returns the clauses in a `when:` section.
    fn clauses(&self) -> &[Clause] {
        &self.arguments
    }
}

impl ClauseSection for SatisfiesSection {
    /// Returns the clauses in a `satisfies:` section.
    fn clauses(&self) -> &[Clause] {
        &self.arguments
    }
}

/// Traverses references that appear inside an optional `provides:` section.
///
/// Symbols can contain aliases that reference forms or commands, and connection
/// groups can introduce `using:` requirements that must also be validated.
fn walk_optional_provides(
    section: &Option<ProvidesSection>,
    visit: &mut impl FnMut(&SignatureShape),
) {
    if let Some(section) = section {
        for item in &section.arguments {
            match item {
                ProvidesItem::Symbol(group) => walk_alias_kind(&group.symbol.argument, visit),
                ProvidesItem::Connection(group) => {
                    if let Some(using) = &group.using {
                        for spec in &using.arguments {
                            walk_is_or_spec(spec, visit);
                        }
                    }
                }
            }
        }
    }
}

/// Traverses references that appear inside an optional `aliases:` section.
fn walk_optional_aliases(
    section: &Option<AliasesSection>,
    visit: &mut impl FnMut(&SignatureShape),
) {
    if let Some(section) = section {
        for item in &section.arguments {
            match item {
                AliasItem::Alias(group) => walk_alias_kind(&group.alias.argument, visit),
            }
        }
    }
}

/// Traverses the definition side of an alias declaration.
///
/// Expression aliases may introduce forms or command headers that reference
/// existing signatures.  Spec-operator aliases are currently pure declarations
/// for this checker and do not contain command references.
fn walk_alias_kind(kind: &AliasKind, visit: &mut impl FnMut(&SignatureShape)) {
    match kind {
        AliasKind::Expression(alias) => match &alias.lhs {
            crate::frontend::formulation::ast::ExpressionAliasLhs::Form(form) => {
                walk_form_or_declaration(form, visit);
            }
            crate::frontend::formulation::ast::ExpressionAliasLhs::Command(command) => {
                let shape = shape_for_command_header_node(command);
                visit(&shape);
            }
            crate::frontend::formulation::ast::ExpressionAliasLhs::InfixCommand(command) => {
                let shape = shape_for_infix_command_header(command);
                visit(&shape);
            }
        },
        AliasKind::SpecOperator(_) => {}
    }
}

/// Traverses one `Specify` item for command references.
fn walk_specify_item(item: &SpecifyItem, visit: &mut impl FnMut(&SignatureShape)) {
    match item {
        SpecifyItem::PositiveInt(group) => walk_open_text_clauses(&group.is_, visit),
        SpecifyItem::NegativeInt(group) => walk_open_text_clauses(&group.is_, visit),
        SpecifyItem::Zero(group) => walk_open_text_clauses(&group.is_, visit),
        SpecifyItem::PositiveDecimal(group) => walk_open_text_clauses(&group.is_, visit),
        SpecifyItem::NegativeDecimal(group) => walk_open_text_clauses(&group.is_, visit),
    }
}

/// Placeholder traversal for `is:` sections that currently contain open text.
///
/// Open text is not parsed into formulation AST nodes yet, so there are no
/// semantic command references to validate here.
fn walk_open_text_clauses(_section: &IsSection, _visit: &mut impl FnMut(&SignatureShape)) {}

/// Traverses a value that may be either an `is via` statement or an ordinary spec.
fn walk_is_or_via_item(item: &IsOrViaItem, visit: &mut impl FnMut(&SignatureShape)) {
    match item {
        IsOrViaItem::IsVia(statement) => {
            walk_is_statement(&statement.is_statement, visit);
            walk_tuple_form(&statement.tuple_form, visit);
        }
        IsOrViaItem::IsOrSpec(spec) => walk_is_or_spec(spec, visit),
    }
}

/// Traverses an `is` statement or subject specification.
fn walk_is_or_spec(spec: &IsOrSpec, visit: &mut impl FnMut(&SignatureShape)) {
    match spec {
        IsOrSpec::Is(statement) => walk_is_statement(statement, visit),
        IsOrSpec::Spec(statement) => walk_spec_subject(&statement.subject, visit),
    }
}

/// Traverses an `is` statement whose type may be refined, or a subject spec.
fn walk_is_or_refined_spec(
    spec: &IsOrRefinedStatementSpec,
    visit: &mut impl FnMut(&SignatureShape),
) {
    match spec {
        IsOrRefinedStatementSpec::Is(statement) => walk_is_statement(statement, visit),
        IsOrRefinedStatementSpec::Spec(statement) => walk_spec_subject(&statement.subject, visit),
    }
}

/// Traverses the subject and type of an `is` statement.
fn walk_is_statement(statement: &IsStatement, visit: &mut impl FnMut(&SignatureShape)) {
    walk_is_subject(&statement.subject, visit);
    walk_type_expression(&statement.ty, visit);
}

/// Traverses the subject portion of an `is` statement.
fn walk_is_subject(subject: &IsSubject, visit: &mut impl FnMut(&SignatureShape)) {
    match &subject.kind {
        IsSubjectKind::Forms(forms) => {
            for form in forms {
                match form {
                    IsSubjectForm::Form(form) => walk_form_or_declaration(form, visit),
                    IsSubjectForm::PlaceholderForm(form) => walk_placeholder_form(form, visit),
                }
            }
        }
        IsSubjectKind::Operator(_) => {}
    }
}

/// Traverses the subject portion of a specification statement.
fn walk_spec_subject(subject: &SpecSubject, visit: &mut impl FnMut(&SignatureShape)) {
    match &subject.kind {
        SpecSubjectKind::Form(form) => walk_form_or_declaration(form, visit),
        SpecSubjectKind::Operator(_) => {}
    }
}

/// Traverses a type expression and the arguments nested inside it.
///
/// The type command itself is visited first, followed by any command references
/// appearing inside the type's argument expressions.
fn walk_type_expression(ty: &TypeExpression, visit: &mut impl FnMut(&SignatureShape)) {
    match ty {
        TypeExpression::Command(command) => {
            let shape = shape_for_command_expression(command);
            visit(&shape);
            walk_command_expression_arguments(command, visit);
        }
        TypeExpression::RefinedCommand(command) => {
            let shape = shape_for_refined_command_expression(command);
            visit(&shape);
            walk_refined_command_expression_arguments(command, visit);
        }
    }
}

/// Traverses a logical clause tree for command references.
///
/// Clause traversal is recursive because quantifiers, conditionals, and grouped
/// logical constructs can contain nested clauses in multiple sections.
fn walk_clause(clause: &Clause, visit: &mut impl FnMut(&SignatureShape)) {
    match clause {
        Clause::Not(group) => walk_clause(&group.not.argument, visit),
        Clause::AllOf(group) => {
            for clause in &group.all_of.arguments {
                walk_clause(clause, visit);
            }
        }
        Clause::AnyOf(group) => {
            for clause in &group.any_of.arguments {
                walk_clause(clause, visit);
            }
        }
        Clause::OneOf(group) => {
            for clause in &group.one_of.arguments {
                walk_clause(clause, visit);
            }
        }
        Clause::Exists(group) => {
            walk_is_or_spec(&group.exists.argument, visit);
            for clause in &group.such_that.arguments {
                walk_clause(clause, visit);
            }
        }
        Clause::ExistsUnique(group) => {
            walk_is_or_spec(&group.exists_unique.argument, visit);
            for clause in &group.such_that.arguments {
                walk_clause(clause, visit);
            }
        }
        Clause::ForAll(group) => {
            walk_is_or_spec(&group.for_all.argument, visit);
            if let Some(section) = &group.where_ {
                for clause in &section.arguments {
                    walk_clause(clause, visit);
                }
            }
            for clause in &group.then.arguments {
                walk_clause(clause, visit);
            }
        }
        Clause::If(group) => {
            for clause in &group.if_.arguments {
                walk_clause(clause, visit);
            }
            for clause in &group.then.arguments {
                walk_clause(clause, visit);
            }
        }
        Clause::Iff(group) => {
            for clause in &group.iff.arguments {
                walk_clause(clause, visit);
            }
            for clause in &group.then.arguments {
                walk_clause(clause, visit);
            }
        }
        Clause::Piecewise(group) => {
            for clause in &group.if_.arguments {
                walk_clause(clause, visit);
            }
            for clause in &group.then.arguments {
                walk_clause(clause, visit);
            }
            if let Some(section) = &group.else_ {
                for clause in &section.arguments {
                    walk_clause(clause, visit);
                }
            }
        }
        Clause::Given(group) => {
            walk_is_or_refined_spec(&group.given.argument, visit);
            if let Some(section) = &group.where_ {
                for clause in &section.arguments {
                    walk_clause(clause, visit);
                }
            }
            for clause in &group.then.arguments {
                walk_clause(clause, visit);
            }
        }
        Clause::IsOrSpec(spec) => walk_is_or_spec(spec, visit),
        Clause::Expression(expression) => walk_expression(expression, visit),
    }
}

/// Traverses an expression tree for command references.
///
/// Command-like expressions are visited as signature shapes, and their argument
/// expressions are traversed recursively so nested references are also checked.
fn walk_expression(expression: &Expression, visit: &mut impl FnMut(&SignatureShape)) {
    match &expression.kind {
        ExpressionKind::Name(_) => {}
        ExpressionKind::FunctionCall { arguments, .. } => {
            for argument in arguments {
                walk_expression(argument, visit);
            }
        }
        ExpressionKind::FunctionNamedCall { elements, .. } => {
            for element in elements {
                walk_expression(&element.expression, visit);
            }
        }
        ExpressionKind::Tuple(elements) => {
            for element in elements {
                if let TupleExpressionElement::Expression(expression) = element {
                    walk_expression(expression, visit);
                }
            }
        }
        ExpressionKind::Set(set) => {
            walk_placeholder_form(&set.target, visit);
            if let Some(predicate) = &set.predicate {
                walk_expression(predicate, visit);
            }
            walk_expression(&set.spec.subject, visit);
        }
        ExpressionKind::Grouped { expression, .. } | ExpressionKind::Labeled { expression, .. } => {
            walk_expression(expression, visit);
        }
        ExpressionKind::SubsetCall(_) => {}
        ExpressionKind::Command(command) => {
            let shape = shape_for_command_expression(command);
            visit(&shape);
            walk_command_expression_arguments(command, visit);
        }
        ExpressionKind::InfixCommand {
            left,
            command,
            right,
        } => {
            walk_expression(left, visit);
            let shape = shape_for_infix_command(command);
            visit(&shape);
            walk_infix_command_arguments(command, visit);
            walk_expression(right, visit);
        }
        ExpressionKind::Prefix { expression, .. } => walk_expression(expression, visit),
        ExpressionKind::Binary { left, right, .. } => {
            walk_expression(left, visit);
            walk_expression(right, visit);
        }
        ExpressionKind::SpecStatement(statement) => walk_expression(&statement.subject, visit),
        ExpressionKind::IsPredicate { subject, command }
        | ExpressionKind::IsNotPredicate { subject, command } => {
            walk_expression(subject, visit);
            let shape = shape_for_command_expression(command);
            visit(&shape);
            walk_command_expression_arguments(command, visit);
        }
        ExpressionKind::IsType { subject, ty } => {
            walk_expression(subject, visit);
            walk_type_expression(ty, visit);
        }
    }
}

/// Traverses all expression arguments supplied to a normal command expression.
fn walk_command_expression_arguments(
    command: &CommandExpression,
    visit: &mut impl FnMut(&SignatureShape),
) {
    for args in &command.head_args {
        for expression in &args.expressions {
            walk_expression(expression, visit);
        }
    }
    for tail in &command.tail {
        for args in &tail.args {
            for expression in &args.expressions {
                walk_expression(expression, visit);
            }
        }
    }
    for args in &command.paren_args {
        for expression in &args.expressions {
            walk_expression(expression, visit);
        }
    }
}

/// Traverses all expression arguments supplied to an infix command expression.
fn walk_infix_command_arguments(command: &InfixCommand, visit: &mut impl FnMut(&SignatureShape)) {
    for args in &command.head_args {
        for expression in &args.expressions {
            walk_expression(expression, visit);
        }
    }
    for tail in &command.tail {
        for args in &tail.args {
            for expression in &args.expressions {
                walk_expression(expression, visit);
            }
        }
    }
}

/// Traverses all expression arguments supplied to a refined command expression.
///
/// This includes arguments attached to refinement parts, base command arguments,
/// tail arguments, and optional parenthesized invocation arguments.
fn walk_refined_command_expression_arguments(
    command: &RefinedCommandExpression,
    visit: &mut impl FnMut(&SignatureShape),
) {
    for part in &command.parts {
        for tail in &part.tail {
            for args in &tail.args {
                for expression in &args.expressions {
                    walk_expression(expression, visit);
                }
            }
        }
    }
    for args in &command.head_args {
        for expression in &args.expressions {
            walk_expression(expression, visit);
        }
    }
    for tail in &command.tail {
        for args in &tail.args {
            for expression in &args.expressions {
                walk_expression(expression, visit);
            }
        }
    }
    for args in &command.paren_args {
        for expression in &args.expressions {
            walk_expression(expression, visit);
        }
    }
}

/// Traverses command references that can occur inside a form or declaration.
fn walk_form_or_declaration(form: &FormOrDeclaration, visit: &mut impl FnMut(&SignatureShape)) {
    match &form.kind {
        FormOrDeclarationKind::Name(_) => {}
        FormOrDeclarationKind::FunctionDeclaration { .. } => {}
        FormOrDeclarationKind::TupleDeclaration { form, .. } => walk_tuple_form(form, visit),
        FormOrDeclarationKind::SetDeclaration { form, .. } => {
            walk_placeholder_form(&form.placeholder_form, visit);
        }
        FormOrDeclarationKind::InfixOperator { .. }
        | FormOrDeclarationKind::PrefixOperator { .. }
        | FormOrDeclarationKind::PostfixOperator { .. } => {}
    }
}

/// Traverses nested forms inside a tuple form declaration.
fn walk_tuple_form(
    form: &crate::frontend::formulation::ast::TupleForm,
    visit: &mut impl FnMut(&SignatureShape),
) {
    for element in &form.elements {
        if let TupleFormElement::Form(form) = element {
            walk_form_or_declaration(form, visit);
        }
    }
}

/// Traverses a placeholder form.
///
/// Placeholder forms currently contain only local placeholder names, so this is a
/// structural no-op kept for symmetry with other walk helpers and future growth.
fn walk_placeholder_form(form: &PlaceholderForm, _visit: &mut impl FnMut(&SignatureShape)) {
    match &form.kind {
        PlaceholderFormKind::Placeholder(_) => {}
        PlaceholderFormKind::Function { .. } => {}
    }
}

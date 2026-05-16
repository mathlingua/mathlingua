use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::events::{EventLocation, EventLog, EventPosition, EventSpan, Level};
use crate::frontend::formulation::ast::{
    Chain, ChainPart, CommandExpression, CommandExpressionTailPart, CommandHeader,
    CommandHeaderNode, CommandHeaderTailPart, Expression, ExpressionKind, FormOrDeclaration,
    FormOrDeclarationKind, InfixCommand, InfixCommandHeader, IsOrRefinedStatementSpec, IsOrSpec,
    IsStatement, IsSubject, IsSubjectForm, IsSubjectKind, PlaceholderForm, PlaceholderFormKind,
    RefinedCommandExpression, RefinedCommandHeader, RefinedTail, SpecSubject, SpecSubjectKind,
    TupleExpressionElement, TupleFormElement, TypeExpression,
};
use crate::frontend::structural::ast::*;

const ORIGIN: &str = "semantic_check";

#[derive(Clone, Debug)]
pub struct ParsedSourceFile {
    pub path: PathBuf,
    pub source: String,
    pub document: Document,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SignatureShape {
    signature: String,
    arg_groups: Vec<ArgGroupShape>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArgDelimiter {
    Curly,
    Paren,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ArgGroupShape {
    delimiter: ArgDelimiter,
    count: usize,
}

#[derive(Clone, Debug)]
struct DefinitionEntry {
    kind: DefinitionKind,
    shape: SignatureShape,
    path: PathBuf,
    position: Option<SourcePosition>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DefinitionKind {
    Describes,
    Defines,
    Refines,
    States,
    Axiom,
    Theorem,
    Corollary,
    Lemma,
    Conjecture,
}

impl DefinitionKind {
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

#[derive(Default)]
struct SignatureRegistry {
    definitions: HashMap<String, DefinitionEntry>,
}

pub fn check_documents(files: &[ParsedSourceFile], event_log: &mut EventLog) {
    let mut registry = SignatureRegistry::default();
    for file in files {
        collect_document_definitions(file, &mut registry, event_log);
    }

    for file in files {
        validate_document_references(file, &registry, event_log);
    }
}

fn collect_document_definitions(
    file: &ParsedSourceFile,
    registry: &mut SignatureRegistry,
    event_log: &mut EventLog,
) {
    let mut locator = SourceLocator::new(&file.source);
    for item in &file.document.items {
        let Some((kind, heading)) = definition_heading(item) else {
            continue;
        };
        let shape = shape_for_header(heading);
        let position = locator.locate_heading(&shape);
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

fn definition_heading(item: &TopLevelItem) -> Option<(DefinitionKind, &CommandHeader)> {
    match item {
        TopLevelItem::Describes(group) => Some((DefinitionKind::Describes, &group.heading)),
        TopLevelItem::Defines(group) => Some((DefinitionKind::Defines, &group.heading)),
        TopLevelItem::Refines(group) => Some((DefinitionKind::Refines, &group.heading)),
        TopLevelItem::States(group) => Some((DefinitionKind::States, &group.heading)),
        TopLevelItem::Axiom(group) => group.heading.as_ref().map(|h| (DefinitionKind::Axiom, h)),
        TopLevelItem::Theorem(group) => {
            group.heading.as_ref().map(|h| (DefinitionKind::Theorem, h))
        }
        TopLevelItem::Corollary(group) => group
            .heading
            .as_ref()
            .map(|h| (DefinitionKind::Corollary, h)),
        TopLevelItem::Lemma(group) => group.heading.as_ref().map(|h| (DefinitionKind::Lemma, h)),
        TopLevelItem::Conjecture(group) => group
            .heading
            .as_ref()
            .map(|h| (DefinitionKind::Conjecture, h)),
        _ => None,
    }
}

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

fn validate_reference_shape(
    path: &Path,
    position: Option<SourcePosition>,
    shape: &SignatureShape,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let Some(definition) = registry.definitions.get(&shape.signature) else {
        emit_error(
            event_log,
            path,
            position,
            format!("Undefined command signature `{}`", shape.signature),
        );
        return;
    };

    if definition.shape.arg_groups != shape.arg_groups {
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

fn shape_for_header(header: &CommandHeader) -> SignatureShape {
    match header {
        CommandHeader::Command(command) => shape_for_command_header_node(command),
        CommandHeader::Infix(command) => shape_for_infix_command_header(command),
        CommandHeader::Refined(command) => shape_for_refined_command_header(command),
    }
}

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
    }
}

fn shape_for_infix_command_header(command: &InfixCommandHeader) -> SignatureShape {
    let mut signature = format!("\\:{}", format_chain(&command.chain));
    let mut arg_groups = Vec::new();
    add_heading_curly_groups(&mut arg_groups, &command.head_args);
    add_header_tail(&mut signature, &mut arg_groups, &command.tail);
    signature.push_str(":/");
    SignatureShape {
        signature,
        arg_groups,
    }
}

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
    }
}

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
    }
}

fn shape_for_infix_command(command: &InfixCommand) -> SignatureShape {
    let mut signature = format!("\\:{}", format_chain(&command.chain));
    let mut arg_groups = Vec::new();
    add_expression_curly_groups(&mut arg_groups, &command.head_args);
    add_expression_tail(&mut signature, &mut arg_groups, &command.tail);
    signature.push_str(":/");
    SignatureShape {
        signature,
        arg_groups,
    }
}

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
    SignatureShape {
        signature,
        arg_groups,
    }
}

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

fn format_chain(chain: &Chain) -> String {
    chain
        .parts
        .iter()
        .map(format_chain_part)
        .collect::<Vec<_>>()
        .join(".")
}

fn format_chain_part(part: &ChainPart) -> String {
    match part {
        ChainPart::Name(name) => name.clone(),
        ChainPart::Alias(name) => format!("${name}"),
        ChainPart::Operator(operator) => operator.clone(),
    }
}

fn format_refined_tail(tail: &RefinedTail) -> String {
    match tail {
        RefinedTail::Chain(chain) => format_chain(chain),
        RefinedTail::Name { name, .. } => name.clone(),
    }
}

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
struct SourcePosition {
    row: usize,
    column: usize,
}

struct SourceLocator<'a> {
    source: &'a str,
    heading_cursor: usize,
    reference_cursor: usize,
}

impl<'a> SourceLocator<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            heading_cursor: 0,
            reference_cursor: 0,
        }
    }

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
enum OccurrenceKind {
    Heading,
    Reference,
}

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

fn is_heading_line(source: &str, offset: usize) -> bool {
    let line_start = source[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = source[offset..]
        .find('\n')
        .map(|i| offset + i)
        .unwrap_or(source.len());
    let line = source[line_start..line_end].trim();
    line.starts_with('[') && line.ends_with(']')
}

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

fn walk_theorem_like(
    given: Option<&GivenSection>,
    where_: Option<&WhereSection>,
    then: &ThenSection,
    iff: Option<&IffSection>,
    visit: &mut impl FnMut(&SignatureShape),
) {
    if let Some(section) = given {
        for spec in &section.arguments {
            walk_is_or_spec(spec, visit);
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

trait ClauseSection {
    fn clauses(&self) -> &[Clause];
}

impl ClauseSection for WhenSection {
    fn clauses(&self) -> &[Clause] {
        &self.arguments
    }
}

impl ClauseSection for SatisfiesSection {
    fn clauses(&self) -> &[Clause] {
        &self.arguments
    }
}

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

fn walk_specify_item(item: &SpecifyItem, visit: &mut impl FnMut(&SignatureShape)) {
    match item {
        SpecifyItem::PositiveInt(group) => walk_open_text_clauses(&group.is_, visit),
        SpecifyItem::NegativeInt(group) => walk_open_text_clauses(&group.is_, visit),
        SpecifyItem::Zero(group) => walk_open_text_clauses(&group.is_, visit),
        SpecifyItem::PositiveDecimal(group) => walk_open_text_clauses(&group.is_, visit),
        SpecifyItem::NegativeDecimal(group) => walk_open_text_clauses(&group.is_, visit),
    }
}

fn walk_open_text_clauses(_section: &IsSection, _visit: &mut impl FnMut(&SignatureShape)) {}

fn walk_is_or_via_item(item: &IsOrViaItem, visit: &mut impl FnMut(&SignatureShape)) {
    match item {
        IsOrViaItem::IsVia(statement) => {
            walk_is_statement(&statement.is_statement, visit);
            walk_tuple_form(&statement.tuple_form, visit);
        }
        IsOrViaItem::IsOrSpec(spec) => walk_is_or_spec(spec, visit),
    }
}

fn walk_is_or_spec(spec: &IsOrSpec, visit: &mut impl FnMut(&SignatureShape)) {
    match spec {
        IsOrSpec::Is(statement) => walk_is_statement(statement, visit),
        IsOrSpec::Spec(statement) => walk_spec_subject(&statement.subject, visit),
    }
}

fn walk_is_or_refined_spec(
    spec: &IsOrRefinedStatementSpec,
    visit: &mut impl FnMut(&SignatureShape),
) {
    match spec {
        IsOrRefinedStatementSpec::Is(statement) => walk_is_statement(statement, visit),
        IsOrRefinedStatementSpec::Spec(statement) => walk_spec_subject(&statement.subject, visit),
    }
}

fn walk_is_statement(statement: &IsStatement, visit: &mut impl FnMut(&SignatureShape)) {
    walk_is_subject(&statement.subject, visit);
    walk_type_expression(&statement.ty, visit);
}

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

fn walk_spec_subject(subject: &SpecSubject, visit: &mut impl FnMut(&SignatureShape)) {
    match &subject.kind {
        SpecSubjectKind::Form(form) => walk_form_or_declaration(form, visit),
        SpecSubjectKind::Operator(_) => {}
    }
}

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
            walk_is_or_spec(&group.given.argument, visit);
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

fn walk_placeholder_form(form: &PlaceholderForm, _visit: &mut impl FnMut(&SignatureShape)) {
    match &form.kind {
        PlaceholderFormKind::Placeholder(_) => {}
        PlaceholderFormKind::Function { .. } => {}
    }
}

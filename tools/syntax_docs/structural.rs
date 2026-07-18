//! Extracts the structural language's syntax from its implementation sources.
//!
//! Two files are read and joined:
//!
//! * `src/frontend/structural/ast.rs` declares *what a group holds*. The three
//!   section macros (`argument_section!`, `arguments_section!`,
//!   `zero_or_more_arguments_section!`) fix each section's arity and element type,
//!   and each `*Group` struct's fields fix which sections exist and whether they are
//!   optional (`Option<...>`).
//! * `src/frontend/structural/parser.rs` declares *what a group accepts*. Every
//!   group's parse function calls `identify_sections(head, .., &["Head", "given?", ..])`,
//!   which fixes the head label, the exact section order, and per-section optionality
//!   (a `?` suffix), plus a heading helper that fixes the heading requirement.
//!
//! The join is by section label, so a label the parser accepts with no backing AST
//! field (a head section taking no argument, or `Id?`) renders without a type, and an
//! AST field no label maps to is an error rather than a silent omission.

use std::collections::HashMap;

use syn::visit::Visit;
use syn::{Expr, File, Item, Lit, Local, Pat, Type};

use super::model::{
    Arity, EnumDef, EnumVariant, ExpectedLabel, GroupField, GroupStruct, HeadingKind, ParsedGroup,
    ResolvedGroup, ResolvedSection, SectionDef, UnderivableGroup,
};
use super::rust_util::{last_segment, strip_wrappers, type_text};

/// Everything read out of the structural sources.
pub struct Structural {
    pub groups: Vec<ResolvedGroup>,
    pub sections: Vec<SectionDef>,
    pub enums: Vec<EnumDef>,
    /// Groups whose parse function does not follow the `identify_sections` pattern, so
    /// their shape cannot be derived. They are reported rather than guessed at.
    pub underivable: Vec<UnderivableGroup>,
}

/// A label entry in a parser's expected-section list. It is either a literal or an
/// identifier that must be resolved through the enclosing function's parameters.
#[derive(Clone, Debug)]
enum LabelSource {
    Literal(String),
    /// An identifier, e.g. `section_name`, resolved via `let` bindings to a parameter.
    Ident(String),
}

/// The `identify_sections` call found inside a parse function.
#[derive(Clone, Debug)]
struct IdentifyCall {
    head: LabelSource,
    labels: Vec<LabelSource>,
}

/// One argument at a call site: either a string literal or a bare identifier.
#[derive(Clone, Debug)]
enum ArgValue {
    Literal(String),
    Ident(String),
}

/// What a single function in the parser declares.
#[derive(Clone, Debug, Default)]
struct FnFacts {
    params: Vec<String>,
    /// `let a = b;` bindings, used to trace an alias back to a parameter.
    lets: HashMap<String, String>,
    identify: Option<IdentifyCall>,
    heading: Option<HeadingKind>,
    /// Calls to other functions, with their literal/identifier arguments.
    calls: Vec<(String, Vec<ArgValue>)>,
    /// The `*Group` type returned as `Option<XGroup>`, when any.
    returns_group: Option<String>,
    /// Every `*Group` the body constructs, in source order. A parse function need not
    /// return its group directly: `parse_from_group` returns an `EnablesItem` wrapping
    /// one of two groups, so the return type alone would miss both.
    built_groups: Vec<String>,
    /// Group field name -> the section label its initializer reads.
    field_labels: HashMap<String, String>,
}

/// Reads and joins the structural AST and parser sources.
pub fn extract(ast_source: &str, parser_source: &str) -> Result<Structural, String> {
    let ast: File = syn::parse_file(ast_source).map_err(|e| format!("structural ast.rs: {e}"))?;
    let parser: File =
        syn::parse_file(parser_source).map_err(|e| format!("structural parser.rs: {e}"))?;

    let sections = section_defs(&ast);
    let structs = group_structs(&ast);
    let enums = enum_defs(&ast);
    let (parsed, underivable) = parsed_groups(&parser)?;

    let section_by_type: HashMap<&str, &SectionDef> = sections
        .iter()
        .map(|s| (s.type_name.as_str(), s))
        .collect();
    let struct_by_name: HashMap<&str, &GroupStruct> =
        structs.iter().map(|s| (s.name.as_str(), s)).collect();

    let mut groups = Vec::new();
    for group in &parsed {
        let mut structures = Vec::new();
        for name in &group.group_types {
            let Some(structure) = struct_by_name.get(name.as_str()) else {
                return Err(format!(
                    "parser builds `{name}` but no such struct exists in structural/ast.rs"
                ));
            };
            structures.push(*structure);
        }
        groups.push(resolve(group, &structures, &section_by_type)?);
    }

    // Completeness guard. Every `*Group` in the AST must be either documented or listed
    // as underivable; anything else would be dropped from the reference without a trace.
    let covered: Vec<&str> = groups
        .iter()
        .flat_map(|group| group.group_types.iter().map(String::as_str))
        .chain(
            underivable
                .iter()
                .flat_map(|group| group.types.iter().map(String::as_str)),
        )
        .collect();
    let missing: Vec<&str> = structs
        .iter()
        .map(|s| s.name.as_str())
        .filter(|name| !covered.contains(name))
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "these groups exist in structural/ast.rs but no parse function was matched to \
             them, so they would be missing from the reference: {}. Either they are dead \
             code, or the generator needs to learn how they are parsed.",
            missing.join(", ")
        ));
    }

    groups.sort_by(|a, b| a.head_label.cmp(&b.head_label));
    Ok(Structural {
        groups,
        sections,
        enums,
        underivable,
    })
}

/// Joins one group's parser-declared labels to its AST-declared field types.
///
/// A label finds its field two ways, in order:
///
/// 1. The parse function's own construction — `then: ThenSection { .. section(&sections,
///    "have") .. }` says the `have:` section lives in `then`. This is read from the code
///    and so survives any naming that does not match.
/// 2. Failing that, the field whose name matches the label (`where_` for `where:`,
///    `all_of` for `allOf:`). This covers groups built from a shared helper's tuple,
///    where the construction carries no labels.
fn resolve(
    group: &ParsedGroup,
    structures: &[&GroupStruct],
    section_by_type: &HashMap<&str, &SectionDef>,
) -> Result<ResolvedGroup, String> {
    let group_label = structures
        .iter()
        .map(|s| s.name.as_str())
        .collect::<Vec<_>>()
        .join("/");

    // When one syntax builds several shapes, a section belongs to whichever shape
    // declares it, so the fields are considered together.
    let mut named: Vec<&GroupField> = Vec::new();
    for structure in structures {
        for field in &structure.fields {
            if field.name == "heading" || named.iter().any(|seen| seen.name == field.name) {
                continue;
            }
            named.push(field);
        }
    }

    // label -> field, as declared by the parse function's construction.
    let mut by_label: HashMap<&str, &GroupField> = HashMap::new();
    for field in &named {
        if let Some(label) = group.field_labels.get(&field.name) {
            by_label.insert(label.as_str(), field);
        }
    }
    // Normalized field name -> field, for the fallback.
    let by_name: HashMap<String, &GroupField> = named
        .iter()
        .map(|field| (field.name.trim_end_matches('_').to_owned(), *field))
        .collect();

    let mut sections = Vec::new();
    let mut used: Vec<String> = Vec::new();
    for label in &group.labels {
        let key = camel_to_snake(&label.label);
        let found = by_label
            .get(label.label.as_str())
            .copied()
            .or_else(|| by_name.get(&key).copied());
        match found {
            Some(field) => {
                let Some(section) = section_by_type.get(field.type_name.as_str()) else {
                    return Err(format!(
                        "{}: field `{}` has type `{}`, which is not declared by a section macro",
                        group_label, field.name, field.type_name
                    ));
                };
                // A section declared `Option<..>` in the AST must be optional in the
                // parser's expected list too; disagreement means the two sources have drifted.
                //
                // Only checkable when the syntax builds a single shape. Where it builds
                // several, an optional section is what selects between them — `capability?`
                // is optional syntax but a required field of `FromCapabilityGroup` — so the
                // two legitimately disagree.
                if structures.len() == 1 && field.optional != label.optional {
                    return Err(format!(
                        "{}: section `{}` is {} in the parser but {} in the AST",
                        group_label,
                        label.label,
                        if label.optional {
                            "optional"
                        } else {
                            "required"
                        },
                        if field.optional {
                            "Option<..>"
                        } else {
                            "required"
                        },
                    ));
                }
                used.push(field.name.clone());
                sections.push(ResolvedSection {
                    label: label.label.clone(),
                    optional: label.optional,
                    element: Some(section.element.clone()),
                    arity: Some(section.arity),
                });
            }
            // A label with no backing field carries no value: either a head section
            // that takes no argument (`Theorem:`) or a section handled outside the
            // structural group (`Id:`).
            None => sections.push(ResolvedSection {
                label: label.label.clone(),
                optional: label.optional,
                element: None,
                arity: None,
            }),
        }
    }

    // Every section the AST stores must be reachable from the parser's expected list.
    // A field left over means the AST holds something no label can fill, which would
    // otherwise be silently omitted from the documentation.
    for field in &named {
        if !used.contains(&field.name) {
            return Err(format!(
                "{}: field `{}` has no matching label in the parser's expected sections",
                group_label, field.name
            ));
        }
    }

    Ok(ResolvedGroup {
        group_types: group.group_types.clone(),
        head_label: group.head_label.clone(),
        heading: group.heading,
        sections,
    })
}

/// `allOf` -> `all_of`, `Describes` -> `describes`, `existsUnique` -> `exists_unique`.
fn camel_to_snake(input: &str) -> String {
    let mut out = String::new();
    for (index, ch) in input.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index != 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

// ===============================[ ast.rs ]=====================================

/// Reads the three section macros, each of which fixes one section's arity.
fn section_defs(file: &File) -> Vec<SectionDef> {
    let mut out = Vec::new();
    for item in &file.items {
        let Item::Macro(item) = item else { continue };
        let Some(name) = last_segment(&item.mac.path) else {
            continue;
        };
        let arity = match name.as_str() {
            "argument_section" => Arity::One,
            "arguments_section" => Arity::OneOrMore,
            "zero_or_more_arguments_section" => Arity::ZeroOrMore,
            _ => continue,
        };
        if let Ok(args) = syn::parse2::<SectionMacroArgs>(item.mac.tokens.clone()) {
            out.push(SectionDef {
                type_name: args.name,
                arity,
                element: args.element,
            });
        }
    }
    out.sort_by(|a, b| a.type_name.cmp(&b.type_name));
    out
}

/// `argument_section!(DescribesSection, DescribesTarget)`
struct SectionMacroArgs {
    name: String,
    element: String,
}

impl syn::parse::Parse for SectionMacroArgs {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let element: Type = input.parse()?;
        Ok(Self {
            name: name.to_string(),
            element: type_text(&strip_wrappers(&element).1),
        })
    }
}

fn group_structs(file: &File) -> Vec<GroupStruct> {
    let mut out = Vec::new();
    for item in &file.items {
        let Item::Struct(item) = item else { continue };
        let name = item.ident.to_string();
        if !name.ends_with("Group") {
            continue;
        }
        let syn::Fields::Named(named) = &item.fields else {
            continue;
        };
        let fields = named
            .named
            .iter()
            .filter_map(|field| {
                let ident = field.ident.as_ref()?.to_string();
                let (optional, inner) = strip_wrappers(&field.ty);
                Some(GroupField {
                    name: ident,
                    optional,
                    type_name: type_text(&inner),
                })
            })
            .collect();
        out.push(GroupStruct { name, fields });
    }
    out
}

fn enum_defs(file: &File) -> Vec<EnumDef> {
    let mut out = Vec::new();
    for item in &file.items {
        let Item::Enum(item) = item else { continue };
        let variants = item
            .variants
            .iter()
            .map(|variant| {
                let payload = match &variant.fields {
                    syn::Fields::Unnamed(unnamed) => unnamed
                        .unnamed
                        .first()
                        .map(|field| type_text(&strip_wrappers(&field.ty).1)),
                    _ => None,
                };
                EnumVariant {
                    name: variant.ident.to_string(),
                    payload,
                }
            })
            .collect();
        out.push(EnumDef {
            name: item.ident.to_string(),
            variants,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

// ===============================[ parser.rs ]=====================================

/// Reads every group's parse function and resolves its declared sections, following
/// one level of indirection through a shared helper (e.g. `parse_argument_theorem_like`)
/// when the parse function only delegates.
///
/// Returns the groups it could derive plus the names of those it could not. A group is
/// only derived when its shape is unambiguous; anything else is reported, because a
/// guessed shape would be worse than an acknowledged gap.
fn parsed_groups(file: &File) -> Result<(Vec<ParsedGroup>, Vec<UnderivableGroup>), String> {
    let mut facts: HashMap<String, FnFacts> = HashMap::new();
    collect_fn_facts(&file.items, &mut facts);

    let mut out = Vec::new();
    let mut underivable = Vec::new();
    for (name, fact) in &facts {
        // A function describes a group when it returns one, or when it declares sections
        // and constructs one. The second case covers functions that wrap their group in
        // an enum instead of returning it.
        let group_types = match &fact.returns_group {
            Some(group) => vec![group.clone()],
            None if fact.identify.is_some() && !fact.built_groups.is_empty() => {
                fact.built_groups.clone()
            }
            None => continue,
        };

        let subst = HashMap::new();
        match resolve_fn(name, &subst, &facts, 0) {
            Some((head, labels, heading)) => out.push(ParsedGroup {
                group_types,
                head_label: head,
                heading,
                labels,
                field_labels: fact.field_labels.clone(),
            }),
            None => underivable.push(UnderivableGroup {
                types: group_types,
                parse_fn: name.clone(),
            }),
        }
    }
    out.sort_by(|a, b| a.group_types.cmp(&b.group_types));
    underivable.sort_by(|a, b| a.types.cmp(&b.types));
    Ok((out, underivable))
}

/// Resolves a function's head label, section labels, and heading, substituting the
/// caller's literal arguments for the callee's parameters.
fn resolve_fn(
    name: &str,
    subst: &HashMap<String, String>,
    facts: &HashMap<String, FnFacts>,
    depth: usize,
) -> Option<(String, Vec<ExpectedLabel>, HeadingKind)> {
    if depth > 4 {
        return None;
    }
    let fact = facts.get(name)?;

    if let Some(identify) = &fact.identify {
        let head = resolve_label(&identify.head, fact, subst)?;
        let mut labels = Vec::new();
        for source in &identify.labels {
            let raw = resolve_label(source, fact, subst)?;
            let optional = raw.ends_with('?');
            labels.push(ExpectedLabel {
                label: raw.trim_end_matches('?').to_owned(),
                optional,
            });
        }
        return Some((head, labels, fact.heading.unwrap_or(HeadingKind::None)));
    }

    // Otherwise the function may be a thin wrapper around a shared group builder, as
    // `parse_theorem` is around `parse_argument_theorem_like`. Only follow the
    // delegation when it is unambiguous: exactly one callee declares sections of its
    // own, and it is handed a literal head name. Any looser rule risks attributing some
    // other function's sections to this group.
    let candidates: Vec<&(String, Vec<ArgValue>)> = fact
        .calls
        .iter()
        .filter(|(callee, _)| facts.get(callee).is_some_and(|f| f.identify.is_some()))
        .collect();
    if candidates.len() != 1 {
        return None;
    }
    let (callee, args) = candidates[0];
    if !args.iter().any(|arg| matches!(arg, ArgValue::Literal(_))) {
        return None;
    }
    let callee_fact = facts.get(callee)?;

    let mut next = HashMap::new();
    for (index, arg) in args.iter().enumerate() {
        let Some(param) = callee_fact.params.get(index) else {
            continue;
        };
        match arg {
            ArgValue::Literal(text) => {
                next.insert(param.clone(), text.clone());
            }
            ArgValue::Ident(ident) => {
                if let Some(value) = subst.get(ident) {
                    next.insert(param.clone(), value.clone());
                }
            }
        }
    }

    let resolved = resolve_fn(callee, &next, facts, depth + 1)?;
    // A delegating wrapper may still choose its own heading.
    let heading = fact.heading.unwrap_or(resolved.2);
    Some((resolved.0, resolved.1, heading))
}

/// Resolves one label entry to text, tracing identifiers through `let` aliases to a
/// parameter and then to the caller's substitution.
fn resolve_label(
    source: &LabelSource,
    fact: &FnFacts,
    subst: &HashMap<String, String>,
) -> Option<String> {
    match source {
        LabelSource::Literal(text) => Some(text.clone()),
        LabelSource::Ident(ident) => {
            let mut current = ident.clone();
            // Follow `let a = b;` chains until reaching a name the caller bound.
            for _ in 0..8 {
                if let Some(value) = subst.get(&current) {
                    return Some(value.clone());
                }
                match fact.lets.get(&current) {
                    Some(next) => current = next.clone(),
                    None => break,
                }
            }
            subst.get(&current).cloned()
        }
    }
}

fn collect_fn_facts(items: &[Item], out: &mut HashMap<String, FnFacts>) {
    for item in items {
        match item {
            Item::Fn(item) => {
                out.insert(item.sig.ident.to_string(), fn_facts(item));
            }
            Item::Mod(module) => {
                // Skip `#[cfg(test)]` modules: test helpers are not part of the language.
                if module.attrs.iter().any(is_cfg_test) {
                    continue;
                }
                if let Some((_, items)) = &module.content {
                    collect_fn_facts(items, out);
                }
            }
            _ => {}
        }
    }
}

/// True for `#[cfg(test)]`.
fn is_cfg_test(attr: &syn::Attribute) -> bool {
    if !attr.path().is_ident("cfg") {
        return false;
    }
    let mut found = false;
    let _ = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("test") {
            found = true;
        }
        Ok(())
    });
    found
}

fn fn_facts(item: &syn::ItemFn) -> FnFacts {
    let mut facts = FnFacts {
        params: item
            .sig
            .inputs
            .iter()
            .map(|arg| match arg {
                syn::FnArg::Typed(typed) => match &*typed.pat {
                    Pat::Ident(ident) => ident.ident.to_string(),
                    _ => String::new(),
                },
                syn::FnArg::Receiver(_) => "self".to_owned(),
            })
            .collect(),
        returns_group: returns_group(&item.sig.output),
        ..FnFacts::default()
    };

    let mut visitor = BodyVisitor { facts: &mut facts };
    visitor.visit_block(&item.block);
    facts
}

/// Finds the section label an initializer reads, via `section(&sections, "L")` or
/// `sections.get("L")`.
fn label_of_expr(expr: &Expr) -> Option<String> {
    struct Finder {
        label: Option<String>,
    }

    fn string_arg(expr: Option<&Expr>) -> Option<String> {
        match expr? {
            Expr::Lit(lit) => match &lit.lit {
                Lit::Str(text) => Some(text.value()),
                _ => None,
            },
            _ => None,
        }
    }

    impl<'ast> Visit<'ast> for Finder {
        fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
            if self.label.is_none()
                && let Expr::Path(path) = &*node.func
                && last_segment(&path.path).as_deref() == Some("section")
                && let Some(label) = string_arg(node.args.get(1))
            {
                self.label = Some(label);
            }
            syn::visit::visit_expr_call(self, node);
        }

        fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
            if self.label.is_none()
                && node.method == "get"
                && let Some(label) = string_arg(node.args.first())
            {
                self.label = Some(label);
            }
            syn::visit::visit_expr_method_call(self, node);
        }
    }

    let mut finder = Finder { label: None };
    finder.visit_expr(expr);
    finder.label
}

/// `-> Option<TheoremGroup>` yields `TheoremGroup`.
fn returns_group(output: &syn::ReturnType) -> Option<String> {
    let syn::ReturnType::Type(_, ty) = output else {
        return None;
    };
    let (was_option, inner) = strip_wrappers(ty);
    if !was_option {
        return None;
    }
    let name = type_text(&inner);
    name.ends_with("Group").then_some(name)
}

struct BodyVisitor<'a> {
    facts: &'a mut FnFacts,
}

impl<'ast> Visit<'ast> for BodyVisitor<'_> {
    fn visit_local(&mut self, node: &'ast Local) {
        if let Pat::Ident(ident) = &node.pat
            && let Some(init) = &node.init
            && let Expr::Path(path) = &*init.expr
            && let Some(source) = path.path.get_ident()
        {
            self.facts
                .lets
                .insert(ident.ident.to_string(), source.to_string());
        }
        syn::visit::visit_local(self, node);
    }

    fn visit_expr_struct(&mut self, node: &'ast syn::ExprStruct) {
        if let Some(name) = last_segment(&node.path)
            && name.ends_with("Group")
        {
            if !self.facts.built_groups.contains(&name) {
                self.facts.built_groups.push(name);
            }
            // Record how the construction fills each field, so a field that stores a
            // differently-named section is still matched to the right label.
            for field in &node.fields {
                let syn::Member::Named(field_name) = &field.member else {
                    continue;
                };
                if let Some(label) = label_of_expr(&field.expr) {
                    self.facts
                        .field_labels
                        .insert(field_name.to_string(), label);
                }
            }
        }
        syn::visit::visit_expr_struct(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let Expr::Path(path) = &*node.func
            && let Some(name) = last_segment(&path.path)
        {
            match name.as_str() {
                "identify_sections" => {
                    if let Some(call) = identify_call(node) {
                        self.facts.identify = Some(call);
                    }
                }
                "parse_required_command_heading" => {
                    self.facts.heading = Some(HeadingKind::CommandRequired);
                }
                "parse_optional_command_heading" => {
                    self.facts.heading = Some(HeadingKind::CommandOptional);
                }
                "parse_optional_label_heading" => {
                    self.facts.heading = Some(HeadingKind::LabelOptional);
                }
                "parse_required_author_heading" => {
                    self.facts.heading = Some(HeadingKind::AuthorRequired);
                }
                "parse_required_resource_heading" => {
                    self.facts.heading = Some(HeadingKind::ResourceRequired);
                }
                "parse_required_topic_heading" => {
                    self.facts.heading = Some(HeadingKind::TopicRequired);
                }
                other => {
                    if other.starts_with("parse_") {
                        self.facts
                            .calls
                            .push((other.to_owned(), node.args.iter().map(arg_value).collect()));
                    }
                }
            }
        }
        syn::visit::visit_expr_call(self, node);
    }
}

fn arg_value(expr: &Expr) -> ArgValue {
    match expr {
        Expr::Lit(lit) => match &lit.lit {
            Lit::Str(text) => ArgValue::Literal(text.value()),
            _ => ArgValue::Ident(String::new()),
        },
        Expr::Path(path) => ArgValue::Ident(
            path.path
                .get_ident()
                .map(|i| i.to_string())
                .unwrap_or_default(),
        ),
        Expr::Reference(reference) => arg_value(&reference.expr),
        _ => ArgValue::Ident(String::new()),
    }
}

/// Reads `identify_sections(head, &group.sections, .., &["A", "b?", ..])`.
///
/// The call must cover the group's whole section list. `parse_disambiguates` calls it
/// as `identify_sections("Disambiguates", &group.sections[index..], ..)` to check only
/// the sections trailing its hand-parsed branches; taking those labels as the group's
/// full shape would silently document it wrong, so a sliced call is rejected and the
/// group is reported as underivable instead.
fn identify_call(node: &syn::ExprCall) -> Option<IdentifyCall> {
    if !covers_whole_group(node.args.get(1)?) {
        return None;
    }
    let head = label_source(node.args.first()?)?;
    let expected = node.args.get(3)?;
    let Expr::Reference(reference) = expected else {
        return None;
    };
    let Expr::Array(array) = &*reference.expr else {
        return None;
    };
    let labels = array.elems.iter().filter_map(label_source).collect();
    Some(IdentifyCall { head, labels })
}

/// True for `&group.sections`; false for a slice such as `&group.sections[index..]`.
fn covers_whole_group(expr: &Expr) -> bool {
    let inner = match expr {
        Expr::Reference(reference) => &*reference.expr,
        other => other,
    };
    matches!(inner, Expr::Field(_))
}

fn label_source(expr: &Expr) -> Option<LabelSource> {
    match expr {
        Expr::Lit(lit) => match &lit.lit {
            Lit::Str(text) => Some(LabelSource::Literal(text.value())),
            _ => None,
        },
        Expr::Path(path) => path
            .path
            .get_ident()
            .map(|ident| LabelSource::Ident(ident.to_string())),
        Expr::Reference(reference) => label_source(&reference.expr),
        _ => None,
    }
}

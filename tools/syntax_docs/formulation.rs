//! Extracts the formulation AST's node shapes from `src/frontend/formulation/ast.rs`.
//!
//! This layer records *what the AST can represent*. The concrete surface syntax that
//! builds these nodes lives in `grammar.lalrpop` and is read by [`super::lalrpop`];
//! the renderer cross-references the two so a node with no production, or a
//! production with no node, is visible.

use syn::{File, Item};

use super::model::{EntryPoint, EnumDef, EnumVariant, StructDef, StructField};
use super::rust_util::{strip_wrappers, type_full, type_text};

pub struct Formulation {
    pub structs: Vec<StructDef>,
    pub enums: Vec<EnumDef>,
    /// The public `parse_*` entry points, which are how section text becomes a node.
    pub entries: Vec<EntryPoint>,
}

pub fn extract(ast_source: &str, parser_source: &str) -> Result<Formulation, String> {
    let file: File = syn::parse_file(ast_source).map_err(|e| format!("formulation ast.rs: {e}"))?;
    let entries = entry_points(parser_source)?;

    let mut structs = Vec::new();
    let mut enums = Vec::new();

    for item in &file.items {
        match item {
            Item::Struct(item) if is_public(&item.vis) => {
                let fields = match &item.fields {
                    syn::Fields::Named(named) => named
                        .named
                        .iter()
                        .filter_map(|field| {
                            Some(StructField {
                                name: field.ident.as_ref()?.to_string(),
                                type_text: type_full(&field.ty),
                            })
                        })
                        .collect(),
                    syn::Fields::Unnamed(unnamed) => unnamed
                        .unnamed
                        .iter()
                        .enumerate()
                        .map(|(index, field)| StructField {
                            name: index.to_string(),
                            type_text: type_full(&field.ty),
                        })
                        .collect(),
                    syn::Fields::Unit => Vec::new(),
                };
                structs.push(StructDef {
                    name: item.ident.to_string(),
                    fields,
                });
            }
            Item::Enum(item) if is_public(&item.vis) => {
                let variants = item
                    .variants
                    .iter()
                    .map(|variant| EnumVariant {
                        name: variant.ident.to_string(),
                        payload: match &variant.fields {
                            syn::Fields::Unnamed(unnamed) => unnamed
                                .unnamed
                                .first()
                                .map(|field| type_text(&strip_wrappers(&field.ty).1)),
                            // A struct-like variant carries its own fields; the
                            // renderer lists the variant name alone.
                            _ => None,
                        },
                    })
                    .collect();
                enums.push(EnumDef {
                    name: item.ident.to_string(),
                    variants,
                });
            }
            _ => {}
        }
    }

    structs.sort_by(|a, b| a.name.cmp(&b.name));
    enums.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Formulation {
        structs,
        enums,
        entries,
    })
}

/// Reads the public `pub fn parse_x(input: &str) -> Result<T, ParseError>` functions
/// from `formulation/parser.rs`. These are the formulation layer's surface API.
fn entry_points(source: &str) -> Result<Vec<EntryPoint>, String> {
    let file: File = syn::parse_file(source).map_err(|e| format!("formulation parser.rs: {e}"))?;
    let mut out = Vec::new();
    for item in &file.items {
        let Item::Fn(item) = item else { continue };
        if !is_public(&item.vis) || !item.sig.ident.to_string().starts_with("parse_") {
            continue;
        }
        let syn::ReturnType::Type(_, ty) = &item.sig.output else {
            continue;
        };
        // The entry points all return `Result<T, ParseError>`; T is the produced node.
        let Some(result_type) = result_ok_type(ty) else {
            continue;
        };
        out.push(EntryPoint {
            name: item.sig.ident.to_string(),
            result_type,
        });
    }
    if out.is_empty() {
        return Err(
            "formulation parser.rs: found no `pub fn parse_*(..) -> Result<..>` entry points"
                .to_owned(),
        );
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// For `Result<T, E>`, yields `T`'s bare name.
fn result_ok_type(ty: &syn::Type) -> Option<String> {
    let syn::Type::Path(path) = ty else { return None };
    let segment = path.path.segments.last()?;
    if segment.ident != "Result" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    let syn::GenericArgument::Type(ok) = args.args.first()? else {
        return None;
    };
    Some(type_text(&strip_wrappers(ok).1))
}

fn is_public(vis: &syn::Visibility) -> bool {
    matches!(vis, syn::Visibility::Public(_))
}

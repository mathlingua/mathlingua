//! Small helpers for reading `syn` types back into readable text.

use quote::ToTokens;
use syn::{GenericArgument, Path, PathArguments, Type};

/// The final identifier of a path, e.g. `ast::Expression` -> `Expression`.
pub fn last_segment(path: &Path) -> Option<String> {
    path.segments.last().map(|s| s.ident.to_string())
}

/// Strips `Option<..>` and `Box<..>` wrappers, reporting whether an `Option` was
/// present. `Option<Box<Clause>>` yields `(true, Clause)`.
pub fn strip_wrappers(ty: &Type) -> (bool, Type) {
    let mut current = ty.clone();
    let mut optional = false;
    for _ in 0..8 {
        let Some((name, inner)) = single_generic(&current) else {
            break;
        };
        match name.as_str() {
            "Option" => {
                optional = true;
                current = inner;
            }
            "Box" => current = inner,
            _ => break,
        }
    }
    (optional, current)
}

/// For `Wrapper<Inner>`, yields `("Wrapper", Inner)`.
fn single_generic(ty: &Type) -> Option<(String, Type)> {
    let Type::Path(path) = ty else { return None };
    let segment = path.path.segments.last()?;
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    if args.args.len() != 1 {
        return None;
    }
    let GenericArgument::Type(inner) = args.args.first()? else {
        return None;
    };
    Some((segment.ident.to_string(), inner.clone()))
}

/// The bare name of a type: its last path segment without generic arguments.
/// Intended for types already reduced by [`strip_wrappers`].
pub fn type_text(ty: &Type) -> String {
    match ty {
        Type::Path(path) => path
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_else(|| normalize(ty)),
        _ => normalize(ty),
    }
}

/// The full type rendered as compact source-like text, e.g. `Option<Box<Expression>>`.
pub fn type_full(ty: &Type) -> String {
    normalize(ty)
}

/// Renders tokens and removes the spacing `proc-macro2` inserts around punctuation,
/// keeping the spaces that separate words (`dyn Trait`, `impl Trait`).
fn normalize(ty: &Type) -> String {
    let text = ty.to_token_stream().to_string();
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    for (index, &ch) in chars.iter().enumerate() {
        if ch != ' ' {
            out.push(ch);
            continue;
        }
        // Drop a space that sits next to punctuation; keep one between two words.
        let before = chars[..index].iter().rev().find(|c| **c != ' ').copied();
        let after = chars[index + 1..].iter().find(|c| **c != ' ').copied();
        let word_boundary = matches!(before, Some(c) if c.is_alphanumeric() || c == '_')
            && matches!(after, Some(c) if c.is_alphanumeric() || c == '_');
        if word_boundary {
            out.push(' ');
        }
    }
    out
}

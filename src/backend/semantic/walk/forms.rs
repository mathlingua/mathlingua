use super::*;

/// Traverses command references that can occur inside a form or declaration.
pub(in crate::backend::semantic) fn walk_form_or_declaration(
    form: &FormOrDeclaration,
    visit: &mut impl FnMut(&SignatureShape),
) {
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
pub(in crate::backend::semantic) fn walk_tuple_form(
    form: &TupleForm,
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
pub(in crate::backend::semantic) fn walk_placeholder_form(
    form: &PlaceholderForm,
    _visit: &mut impl FnMut(&SignatureShape),
) {
    match &form.kind {
        PlaceholderFormKind::Placeholder(_) => {}
        PlaceholderFormKind::Function { .. } => {}
    }
}

use super::*;

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

pub(in crate::backend::semantic) fn walk_placeholder_form(
    form: &PlaceholderForm,
    _visit: &mut impl FnMut(&SignatureShape),
) {
    match &form.kind {
        PlaceholderFormKind::Placeholder(_) => {}
        PlaceholderFormKind::Function { .. } => {}
    }
}

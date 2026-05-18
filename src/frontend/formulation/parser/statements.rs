use super::*;

/// Parses a subject/type `is` statement.
///
/// The `allow_refined` flag controls whether the right-hand side may use the
/// refined command expression grammar.  This keeps ordinary definition checks
/// stricter while theorem assumptions can reference refined definitions.
pub(super) fn parse_is_statement(
    input: &str,
    allow_refined: bool,
) -> Result<IsStatement, ParseError> {
    let index = find_top_level_substring(input, " is ")
        .ok_or_else(|| ParseError::custom("expected top-level ` is `"))?;
    let subject = parse_is_subject(&input[..index])?;
    let ty = parse_type_expression(&input[index + 4..], allow_refined)?;

    Ok(IsStatement {
        span: span_all(input),
        subject,
        ty,
    })
}

/// Parses a subject/operator/name specification statement.
///
/// Specifications use a quoted operator between the subject and target name,
/// for example `x "in" A`.  The quoted operator is discovered at top level so
/// nested formulation constructs remain intact.
pub(super) fn parse_subject_spec_statement(
    input: &str,
) -> Result<SubjectSpecStatement, ParseError> {
    let (subject_text, operator, name_text) =
        split_subject_operator_name(input).ok_or_else(|| {
            ParseError::custom("expected specification of the form `<subject> \"op\" Name`")
        })?;
    let subject = parse_spec_subject(subject_text)?;
    let name = parse_name_token(name_text)?;

    Ok(SubjectSpecStatement {
        span: span_all(input),
        subject,
        operator: operator.to_owned(),
        name,
    })
}

/// Parses a placeholder/operator/name specification used by spec aliases.
///
/// This mirrors [`parse_subject_spec_statement`] but requires the subject side
/// to be placeholder syntax such as `x_` or `f_(x_)`.
pub(super) fn parse_placeholder_spec_statement(
    input: &str,
) -> Result<PlaceholderSpecStatement, ParseError> {
    let (placeholder_text, operator, name_text) =
        split_subject_operator_name(input).ok_or_else(|| {
            ParseError::custom("expected placeholder specification of the form `x_ \"op\" X`")
        })?;
    let placeholder_form = parse_placeholder_form(placeholder_text)?;
    let name = parse_name_token(name_text)?;

    Ok(PlaceholderSpecStatement {
        span: span_all(input),
        placeholder_form,
        operator: operator.to_owned(),
        name,
    })
}

/// Parses the type side of an `is` statement.
///
/// Type expressions are intentionally restricted to command expressions because
/// MathLingua type references are command-backed definitions.  When enabled,
/// refined command expressions are accepted before falling back to ordinary
/// command expressions.
pub(super) fn parse_type_expression(
    input: &str,
    allow_refined: bool,
) -> Result<TypeExpression, ParseError> {
    let input = input.trim();
    if allow_refined && contains_top_level(input, "::") {
        return parse_refined_command_expression(input).map(TypeExpression::RefinedCommand);
    }

    let expression = parse_expression(input)?;
    match expression.kind {
        ExpressionKind::Command(command) => Ok(TypeExpression::Command(command)),
        other => Err(ParseError::custom(format!(
            "expected command expression for type, found {other:?}"
        ))),
    }
}

/// Parses the subject on the left side of an `is` statement.
///
/// A comma at top level indicates a list of forms/placeholders.  Otherwise the
/// parser first attempts a single form-like subject and then falls back to an
/// operator subject for statements about operators themselves.
pub(super) fn parse_is_subject(input: &str) -> Result<IsSubject, ParseError> {
    let input = input.trim();

    if find_first_top_level_char(input, ',').is_some() {
        return parse_is_subject_form_list(input).map(|forms| IsSubject {
            span: span_all(input),
            kind: IsSubjectKind::Forms(forms),
        });
    }

    if let Ok(form) = parse_is_subject_form(input) {
        return Ok(IsSubject {
            span: span_all(input),
            kind: IsSubjectKind::Forms(vec![form]),
        });
    }

    let operator = parse_operator(input)?;
    Ok(IsSubject {
        span: span_all(input),
        kind: IsSubjectKind::Operator(operator),
    })
}

/// Parses the subject side of a quoted-operator specification.
///
/// Specification subjects accept ordinary forms when possible, with operator
/// subjects as the fallback so definitions can describe named operators.
pub(super) fn parse_spec_subject(input: &str) -> Result<SpecSubject, ParseError> {
    let input = input.trim();
    if let Ok(form) = parse_form_or_declaration(input) {
        return Ok(SpecSubject {
            span: span_all(input),
            kind: SpecSubjectKind::Form(form),
        });
    }

    let operator = parse_operator(input)?;
    Ok(SpecSubject {
        span: span_all(input),
        kind: SpecSubjectKind::Operator(operator),
    })
}

/// Parses an operator token from raw text.
///
/// Operators are kept as text rather than grammar tokens here because the
/// higher-level specification syntax extracts them from quoted strings.
pub(super) fn parse_operator(input: &str) -> Result<Operator, ParseError> {
    let input = input.trim();
    if !is_operator_text(input) {
        return Err(ParseError::custom(format!(
            "expected operator, found `{input}`"
        )));
    }

    Ok(Operator::new(span_all(input), input))
}

/// Parses the tuple form after an `is ... via` separator.
///
/// The generated form parser can recognize many form variants; this helper
/// narrows the accepted result to an unnamed tuple declaration because `via`
/// introduces tuple-shaped witness data.
pub(super) fn parse_tuple_form(input: &str) -> Result<TupleForm, ParseError> {
    let input = input.trim();
    match parse_form_or_declaration(input)?.kind {
        FormOrDeclarationKind::TupleDeclaration { name: None, form } => Ok(form),
        other => Err(ParseError::custom(format!(
            "expected tuple form, found {other:?}"
        ))),
    }
}

/// Parses one subject form allowed on the left of an `is` statement.
///
/// Full forms are preferred, then placeholder forms are accepted for cases such
/// as theorem schemata that bind placeholder-shaped subjects.
pub(super) fn parse_is_subject_form(input: &str) -> Result<IsSubjectForm, ParseError> {
    if let Ok(form) = parse_form_or_declaration(input) {
        return Ok(IsSubjectForm::Form(form));
    }

    parse_placeholder_form(input).map(IsSubjectForm::PlaceholderForm)
}

/// Parses a comma-separated top-level list of `is` subject forms.
///
/// Empty lists are rejected after splitting so callers can emit a precise
/// subject error instead of accepting an empty subject set.
pub(super) fn parse_is_subject_form_list(input: &str) -> Result<Vec<IsSubjectForm>, ParseError> {
    let forms = split_top_level(input, ',')?
        .into_iter()
        .map(parse_is_subject_form)
        .collect::<Result<Vec<_>, _>>()?;

    if forms.is_empty() {
        return Err(ParseError::custom("expected at least one form"));
    }

    Ok(forms)
}

/// Parses placeholder form syntax such as `x_` or `f_(x_)`.
///
/// Function placeholder arguments must themselves be simple placeholders.  The
/// balanced-prefix scanner prevents commas inside nested syntax from confusing
/// the argument list splitter.
pub(super) fn parse_placeholder_form(input: &str) -> Result<PlaceholderForm, ParseError> {
    let input = input.trim();
    if let Some(open_index) = find_first_top_level_char(input, '(') {
        let (inside, rest) = consume_balanced_prefix(&input[open_index..], '(', ')')?;
        if !rest.trim().is_empty() {
            return Err(ParseError::custom(
                "unexpected trailing text after placeholder form",
            ));
        }

        let placeholder = parse_placeholder(&input[..open_index])?;
        let arguments = parse_placeholder_list(inside)?;
        return Ok(PlaceholderForm::new(
            span_all(input),
            PlaceholderFormKind::Function {
                placeholder,
                arguments,
            },
        ));
    }

    parse_placeholder(input).map(|placeholder| {
        PlaceholderForm::new(
            span_all(input),
            PlaceholderFormKind::Placeholder(placeholder),
        )
    })
}

/// Parses a single placeholder name ending in exactly one underscore.
///
/// Double-underscore markers are reserved for form declarations like `f(x__)`,
/// so placeholder syntax explicitly rejects names ending in `__`.
pub(super) fn parse_placeholder(input: &str) -> Result<Placeholder, ParseError> {
    let input = input.trim();
    if !input.ends_with('_') || input.ends_with("__") {
        return Err(ParseError::custom(format!(
            "expected placeholder ending with `_`, found `{input}`"
        )));
    }

    let name = &input[..input.len() - 1];
    if !is_name_text(name) {
        return Err(ParseError::custom(format!(
            "invalid placeholder name `{name}`"
        )));
    }

    Ok(Placeholder::new(span_all(input), name))
}

/// Parses a nonempty comma-separated list of placeholders.
///
/// The split is top-level aware so future placeholder syntax with grouping does
/// not accidentally create extra list entries.
pub(super) fn parse_placeholder_list(input: &str) -> Result<Vec<Placeholder>, ParseError> {
    let mut placeholders = Vec::new();
    for item in split_top_level(input, ',')? {
        placeholders.push(parse_placeholder(item)?);
    }

    if placeholders.is_empty() {
        return Err(ParseError::custom("expected at least one placeholder"));
    }

    Ok(placeholders)
}

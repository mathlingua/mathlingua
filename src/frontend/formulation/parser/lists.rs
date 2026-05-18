/// Parses a nonempty comma-separated list of forms/declarations.
///
/// Lists are split only on top-level commas so tuple, set, and command argument
/// syntax can contain nested commas safely.
fn parse_form_list(input: &str) -> Result<Vec<FormOrDeclaration>, ParseError> {
    let forms = split_top_level(input, ',')?
        .into_iter()
        .map(parse_form_or_declaration)
        .collect::<Result<Vec<_>, _>>()?;
    if forms.is_empty() {
        return Err(ParseError::custom("expected at least one form"));
    }
    Ok(forms)
}

/// Parses a nonempty comma-separated list of expressions.
///
/// This is the expression-valued counterpart to [`parse_form_list`] and is used
/// for concrete command references and invocation arguments.
fn parse_expression_list(input: &str) -> Result<Vec<Expression>, ParseError> {
    let expressions = split_top_level(input, ',')?
        .into_iter()
        .map(parse_expression)
        .collect::<Result<Vec<_>, _>>()?;
    if expressions.is_empty() {
        return Err(ParseError::custom("expected at least one expression"));
    }
    Ok(expressions)
}

/// Parses a dot-separated command/name chain.
///
/// Chains are used by commands, refined command parts, and namespaced labels.
/// Each part may be a regular name, an operator token, or an alias reference.
fn parse_chain(input: &str) -> Result<Chain, ParseError> {
    let input = input.trim();
    let parts = split_top_level(input, '.')?
        .into_iter()
        .map(parse_chain_part)
        .collect::<Result<Vec<_>, _>>()?;
    if parts.is_empty() {
        return Err(ParseError::custom("chains require at least one part"));
    }

    Ok(Chain {
        span: span_all(input),
        parts,
    })
}

/// Parses one chain segment.
///
/// A leading `$` marks an alias reference, operator-only text becomes an
/// operator segment, and all other segments must satisfy name-token rules.
fn parse_chain_part(input: &str) -> Result<ChainPart, ParseError> {
    let input = input.trim();
    if let Some(alias) = input.strip_prefix('$') {
        return Ok(ChainPart::Alias(parse_name_token(alias)?));
    }

    if is_operator_text(input) {
        return Ok(ChainPart::Operator(input.to_owned()));
    }

    Ok(ChainPart::Name(parse_name_token(input)?))
}

/// Parses dot-separated plain name parts.
///
/// This is used by label, author, and resource headings where aliases and
/// operator chain segments are not valid.
fn parse_dotted_parts(input: &str) -> Result<Vec<String>, ParseError> {
    let parts = split_top_level(input, '.')?
        .into_iter()
        .map(parse_name_token)
        .collect::<Result<Vec<_>, _>>()?;
    if parts.is_empty() {
        return Err(ParseError::custom("expected at least one name part"));
    }
    Ok(parts)
}

/// Validates and returns one name token.
///
/// Name tokens may be backtick-wrapped operator text, which allows operator
/// names to participate in syntax positions that otherwise require identifiers.
fn parse_name_token(input: &str) -> Result<String, ParseError> {
    let input = input.trim();
    if !is_name_text(input) {
        return Err(ParseError::custom(format!("invalid name `{input}`")));
    }

    Ok(input.to_owned())
}


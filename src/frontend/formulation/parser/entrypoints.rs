use super::*;

/// Parses a standalone expression.
///
/// This is the entry point used for theorem clauses and other places where the
/// source must be mathematical expression syntax rather than a declaration or
/// specification statement.
pub fn parse_expression(input: &str) -> Result<Expression, ParseError> {
    grammar::InputExpressionParser::new()
        .parse(Lexer::new(input))
        .map_err(ParseError::from)
}

/// Parses a local expression binding of the form `<left> := <right>`.
pub fn parse_expression_binding(input: &str) -> Result<ExpressionBinding, ParseError> {
    let input = input.trim();
    let index = find_top_level_substring(input, ":=")
        .ok_or_else(|| ParseError::custom("expected top-level `:=`"))?;
    let left = parse_expression(input[..index].trim())?;
    let right = parse_expression(input[index + 2..].trim())?;

    Ok(ExpressionBinding {
        span: span_all(input),
        left,
        right,
    })
}

/// Parses a form or declaration used in definition-like `Describes` sections.
///
/// The generated grammar handles the detailed expression/form precedence here;
/// this wrapper only adapts lexer and error types for callers.
pub fn parse_form_or_declaration(input: &str) -> Result<FormOrDeclaration, ParseError> {
    grammar::InputFormOrDeclarationParser::new()
        .parse(Lexer::new(input))
        .map_err(ParseError::from)
}

/// Parses either an `is` statement or a subject/operator/name specification.
///
/// The split is performed only at a top-level ` is ` so nested expressions such
/// as tuples, command arguments, or quoted operators do not accidentally change
/// which grammar branch is selected.
pub fn parse_is_or_spec(input: &str) -> Result<IsOrSpec, ParseError> {
    let input = input.trim();
    if contains_top_level(input, " is ") {
        let statement = parse_is_statement(input, false)?;
        return Ok(IsOrSpec::Is(statement));
    }

    parse_subject_spec_statement(input).map(IsOrSpec::Spec)
}

/// Parses an `is` statement that may use refined command syntax, or a spec.
///
/// This variant is used in theorem-style contexts where `\(a, b)::command`
/// references are legal.  Definition contexts still use [`parse_is_or_spec`]
/// when refined type expressions should not be accepted.
pub fn parse_is_or_refined_statement_spec(
    input: &str,
) -> Result<IsOrRefinedStatementSpec, ParseError> {
    let input = input.trim();
    if contains_top_level(input, " is ") {
        let statement = parse_is_statement(input, true)?;
        return Ok(IsOrRefinedStatementSpec::Is(statement));
    }

    parse_subject_spec_statement(input).map(IsOrRefinedStatementSpec::Spec)
}

/// Parses an `is ... via ...` statement.
///
/// The `via` separator is located at top level so tuples and command arguments
/// can contain the same text without being split as statement syntax.
pub fn parse_is_via_statement(input: &str) -> Result<IsViaStatement, ParseError> {
    let input = input.trim();
    let index = find_top_level_substring(input, " via ")
        .ok_or_else(|| ParseError::custom("expected top-level ` via `"))?;
    let statement = parse_is_statement(&input[..index], false)?;
    let tuple_form = parse_tuple_form(&input[index + 5..])?;

    Ok(IsViaStatement {
        span: span_all(input),
        is_statement: statement,
        tuple_form,
    })
}

/// Parses any command header form used in bracketed group headings.
///
/// This dispatches among ordinary commands, infix command headers, and refined
/// command headers.  All variants must begin with `\`, which keeps heading
/// parsing distinct from label, author, and resource headings.
pub fn parse_command_header(input: &str) -> Result<CommandHeader, ParseError> {
    let input = input.trim();
    if !input.starts_with('\\') {
        return Err(ParseError::custom("command header must start with `\\`"));
    }

    if input.starts_with("\\:") {
        return parse_infix_command_header(input).map(CommandHeader::Infix);
    }

    if contains_top_level(input, "::") {
        return parse_refined_command_header(input).map(CommandHeader::Refined);
    }

    parse_simple_command_header(input).map(CommandHeader::Command)
}

/// Parses a writing alias of the form `<form> :~> <body>`.
///
/// The body is kept as raw text because writing aliases currently describe
/// rendering templates rather than formulation AST nodes.
pub fn parse_writing_alias(input: &str) -> Result<WritingAlias, ParseError> {
    let input = input.trim();
    let index = find_top_level_substring(input, ":~>")
        .ok_or_else(|| ParseError::custom("expected top-level `:~>`"))?;
    let form = parse_form_or_declaration(input[..index].trim())?;
    let body = input[index + 3..].trim();
    if body.is_empty() {
        return Err(ParseError::custom("writing alias body cannot be empty"));
    }

    Ok(WritingAlias {
        span: span_all(input),
        form,
        body: body.to_owned(),
    })
}

/// Parses an expression alias of the form `<lhs> :=> <expression>`.
///
/// The left-hand side may be a form declaration, ordinary command header, or
/// infix command header.  Refined command headers are rejected here because
/// aliases define base expression syntax rather than refinement composition.
pub fn parse_expression_alias(input: &str) -> Result<ExpressionAlias, ParseError> {
    let input = input.trim();
    let index = find_top_level_substring(input, ":=>")
        .ok_or_else(|| ParseError::custom("expected top-level `:=>`"))?;
    let lhs_text = input[..index].trim();
    let expression = parse_expression(input[index + 3..].trim())?;

    let lhs = if lhs_text.starts_with("\\:") {
        ExpressionAliasLhs::InfixCommand(parse_infix_command_header(lhs_text)?)
    } else if lhs_text.starts_with('\\') {
        match parse_command_header(lhs_text)? {
            CommandHeader::Command(header) => ExpressionAliasLhs::Command(header),
            CommandHeader::Infix(header) => ExpressionAliasLhs::InfixCommand(header),
            CommandHeader::Refined(_) => {
                return Err(ParseError::custom(
                    "refined command headers are not valid expression alias lhs values",
                ));
            }
        }
    } else {
        ExpressionAliasLhs::Form(parse_form_or_declaration(lhs_text)?)
    };

    Ok(ExpressionAlias {
        span: span_all(input),
        lhs,
        expression,
    })
}

/// Parses a specification-operator alias of the form `<placeholder spec> :-> <target>`.
///
/// These aliases bridge operator-like placeholder declarations to either an
/// `is` statement or a specification target, so both sides need bespoke parsing
/// around the generated formulation grammar.
pub fn parse_spec_operator_alias(input: &str) -> Result<SpecOperatorAlias, ParseError> {
    let input = input.trim();
    let index = find_top_level_substring(input, ":->")
        .ok_or_else(|| ParseError::custom("expected top-level `:->`"))?;
    let placeholder_spec = parse_placeholder_spec_statement(input[..index].trim())?;
    let target = parse_spec_operator_alias_target(input[index + 3..].trim())?;

    Ok(SpecOperatorAlias {
        span: span_all(input),
        placeholder_spec,
        target,
    })
}

/// Parses the target side of a specification-operator alias.
///
/// Besides ordinary `is`/spec targets, this position can name a built-in
/// keyword by sigil, for example `\\abstract`.
fn parse_spec_operator_alias_target(input: &str) -> Result<SpecOperatorAliasTarget, ParseError> {
    let input = input.trim();

    if let Some(builtin) = input.strip_prefix("\\\\") {
        return parse_chain(builtin).map(SpecOperatorAliasTarget::Builtin);
    }

    parse_is_or_spec(input)
        .map(Box::new)
        .map(SpecOperatorAliasTarget::IsOrSpec)
}

/// Parses a dot-separated label heading.
///
/// Labels intentionally accept only dotted name parts, keeping them lightweight
/// identifiers for notes and nested structural groups.
pub fn parse_label_header(input: &str) -> Result<LabelHeader, ParseError> {
    let input = input.trim();
    let parts = parse_dotted_parts(input)?;
    Ok(LabelHeader {
        span: span_all(input),
        parts,
    })
}

/// Parses an author heading beginning with `@`.
///
/// The returned header stores the dotted name path without the sigil so the
/// structural layer can use the same path representation as labels/resources.
pub fn parse_author_header(input: &str) -> Result<AuthorHeader, ParseError> {
    let input = input.trim();
    let rest = input
        .strip_prefix('@')
        .ok_or_else(|| ParseError::custom("author headers must start with `@`"))?;
    let parts = parse_dotted_parts(rest)?;
    Ok(AuthorHeader {
        span: span_all(input),
        parts,
    })
}

/// Parses a resource heading beginning with `$`.
///
/// Resource headings share dotted path validation with labels but use a
/// different sigil to keep bibliography-style entries visually separate.
pub fn parse_resource_header(input: &str) -> Result<ResourceHeader, ParseError> {
    let input = input.trim();
    let rest = input
        .strip_prefix('$')
        .ok_or_else(|| ParseError::custom("resource headers must start with `$`"))?;
    let parts = parse_dotted_parts(rest)?;
    Ok(ResourceHeader {
        span: span_all(input),
        parts,
    })
}

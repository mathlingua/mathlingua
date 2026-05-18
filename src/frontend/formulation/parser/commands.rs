use super::*;

/// Parses a non-infix, non-refined command header.
///
/// Command headers consist of a chain, optional head argument groups, optional
/// colon tail parts, and optional parenthesized argument groups.  The contents
/// of header argument groups are forms/declarations because headings describe
/// definition shapes rather than concrete expression references.
pub(super) fn parse_simple_command_header(input: &str) -> Result<CommandHeaderNode, ParseError> {
    let mut rest = input
        .trim()
        .strip_prefix('\\')
        .ok_or_else(|| ParseError::custom("command headers must start with `\\`"))?;
    let (chain_text, remaining) = split_prefix_by_delimiters(rest, &['{', '(', ':']);
    let chain = parse_chain(chain_text)?;
    rest = remaining;

    let (head_args, remaining) = parse_curly_heading_args(rest)?;
    rest = remaining;
    let (tail, remaining) = parse_command_header_tail(rest)?;
    rest = remaining;
    let (paren_args, remaining) = parse_paren_heading_args(rest)?;
    if !remaining.trim().is_empty() {
        return Err(ParseError::custom(
            "unexpected trailing text after command header",
        ));
    }

    Ok(CommandHeaderNode {
        span: span_all(input),
        chain,
        head_args,
        tail,
        paren_args,
    })
}

/// Parses an infix command header delimited by `\:` and `:/`.
///
/// Infix headers share most command-tail syntax with ordinary command headers
/// but intentionally omit parenthesized invocation arguments at the end.
pub(super) fn parse_infix_command_header(input: &str) -> Result<InfixCommandHeader, ParseError> {
    let input = input.trim();
    let mut rest = input
        .strip_prefix("\\:")
        .ok_or_else(|| ParseError::custom("infix command headers must start with `\\:`"))?;
    rest = rest
        .strip_suffix(":/")
        .ok_or_else(|| ParseError::custom("infix command headers must end with `:/`"))?;

    let (chain_text, remaining) = split_prefix_by_delimiters(rest, &['{', ':']);
    let chain = parse_chain(chain_text)?;
    rest = remaining;

    let (head_args, remaining) = parse_curly_heading_args(rest)?;
    rest = remaining;
    let (tail, remaining) = parse_command_header_tail(rest)?;
    if !remaining.trim().is_empty() {
        return Err(ParseError::custom(
            "unexpected trailing text after infix command header",
        ));
    }

    Ok(InfixCommandHeader {
        span: span_all(input),
        chain,
        head_args,
        tail,
    })
}

/// Parses a refined command header such as `\(continuous)::function:on{A}:to{B}`.
///
/// The left side lists refinement parts, optionally prefixed by a dotted chain.
/// The right side is parsed as the refined target tail plus the same argument
/// and colon-tail shape used by ordinary command headers.
pub(super) fn parse_refined_command_header(
    input: &str,
) -> Result<RefinedCommandHeader, ParseError> {
    let input = input.trim();
    let after_slash = input
        .strip_prefix('\\')
        .ok_or_else(|| ParseError::custom("refined command headers must start with `\\`"))?;
    let separator = find_top_level_substring(after_slash, "::")
        .ok_or_else(|| ParseError::custom("refined command headers require top-level `::`"))?;
    let left = after_slash[..separator].trim();
    let mut right = after_slash[separator + 2..].trim();

    let open_index = find_first_top_level_char(left, '(')
        .ok_or_else(|| ParseError::custom("expected refined header part list"))?;
    let prefix = left[..open_index].trim();
    let (inside, suffix) = consume_balanced_prefix(&left[open_index..], '(', ')')?;
    if !suffix.trim().is_empty() {
        return Err(ParseError::custom("unexpected trailing text before `::`"));
    }

    let prefix_chain = if prefix.is_empty() {
        None
    } else {
        let chain_text = prefix
            .strip_suffix('.')
            .ok_or_else(|| ParseError::custom("refined header prefix chain must end with `.`"))?;
        Some(parse_chain(chain_text)?)
    };

    let parts = split_top_level(inside, ',')?
        .into_iter()
        .map(parse_refined_header_part)
        .collect::<Result<Vec<_>, _>>()?;
    if parts.is_empty() {
        return Err(ParseError::custom(
            "refined command headers require at least one part",
        ));
    }

    let (refined_tail, remaining) = parse_refined_tail(right)?;
    right = remaining;
    let (head_args, remaining) = parse_curly_heading_args(right)?;
    right = remaining;
    let (tail, remaining) = parse_command_header_tail(right)?;
    right = remaining;
    let (paren_args, remaining) = parse_paren_heading_args(right)?;
    if !remaining.trim().is_empty() {
        return Err(ParseError::custom(
            "unexpected trailing text after refined command header",
        ));
    }

    Ok(RefinedCommandHeader {
        span: span_all(input),
        prefix_chain,
        parts,
        refined_tail,
        head_args,
        tail,
        paren_args,
    })
}

/// Parses one part inside a refined command header part list.
///
/// Each part is a command chain plus optional colon-tail header arguments.  The
/// caller is responsible for splitting the comma-separated part list safely.
pub(super) fn parse_refined_header_part(input: &str) -> Result<RefinedHeaderPart, ParseError> {
    let input = input.trim();
    let (chain_text, rest) = split_prefix_by_delimiters(input, &[':']);
    let chain = parse_chain(chain_text)?;
    let (tail, remaining) = parse_command_header_tail(rest)?;
    if !remaining.trim().is_empty() {
        return Err(ParseError::custom(
            "unexpected trailing text after refined header part",
        ));
    }

    Ok(RefinedHeaderPart {
        span: span_all(input),
        chain,
        tail,
    })
}

/// Parses a refined command expression used at reference sites.
///
/// This mirrors refined command header parsing, but argument groups contain
/// expressions instead of forms because use sites supply concrete values.
pub(super) fn parse_refined_command_expression(
    input: &str,
) -> Result<RefinedCommandExpression, ParseError> {
    let input = input.trim();
    let after_slash = input
        .strip_prefix('\\')
        .ok_or_else(|| ParseError::custom("refined command expressions must start with `\\`"))?;
    let separator = find_top_level_substring(after_slash, "::")
        .ok_or_else(|| ParseError::custom("refined command expressions require top-level `::`"))?;
    let left = after_slash[..separator].trim();
    let mut right = after_slash[separator + 2..].trim();

    let open_index = find_first_top_level_char(left, '(')
        .ok_or_else(|| ParseError::custom("expected refined expression part list"))?;
    let prefix = left[..open_index].trim();
    let (inside, suffix) = consume_balanced_prefix(&left[open_index..], '(', ')')?;
    if !suffix.trim().is_empty() {
        return Err(ParseError::custom("unexpected trailing text before `::`"));
    }

    let prefix_chain = if prefix.is_empty() {
        None
    } else {
        let chain_text = prefix.strip_suffix('.').ok_or_else(|| {
            ParseError::custom("refined expression prefix chain must end with `.`")
        })?;
        Some(parse_chain(chain_text)?)
    };

    let parts = split_top_level(inside, ',')?
        .into_iter()
        .map(parse_refined_expression_part)
        .collect::<Result<Vec<_>, _>>()?;
    if parts.is_empty() {
        return Err(ParseError::custom(
            "refined command expressions require at least one part",
        ));
    }

    let (refined_tail, remaining) = parse_refined_tail(right)?;
    right = remaining;
    let (head_args, remaining) = parse_curly_expression_args(right)?;
    right = remaining;
    let (tail, remaining) = parse_command_expression_tail(right)?;
    right = remaining;
    let (paren_args, remaining) = parse_paren_expression_args(right)?;
    if !remaining.trim().is_empty() {
        return Err(ParseError::custom(
            "unexpected trailing text after refined command expression",
        ));
    }

    Ok(RefinedCommandExpression {
        span: span_all(input),
        prefix_chain,
        parts,
        refined_tail,
        head_args,
        tail,
        paren_args,
    })
}

/// Parses one part inside a refined command expression part list.
///
/// Expression parts use expression-valued command tails, which lets references
/// provide concrete command arguments while keeping the same chain syntax as
/// refined headers.
pub(super) fn parse_refined_expression_part(
    input: &str,
) -> Result<RefinedExpressionPart, ParseError> {
    let input = input.trim();
    let (chain_text, rest) = split_prefix_by_delimiters(input, &[':']);
    let chain = parse_chain(chain_text)?;
    let (tail, remaining) = parse_command_expression_tail(rest)?;
    if !remaining.trim().is_empty() {
        return Err(ParseError::custom(
            "unexpected trailing text after refined expression part",
        ));
    }

    Ok(RefinedExpressionPart {
        span: span_all(input),
        chain,
        tail,
    })
}

/// Parses the target tail after a refined command's `::` separator.
///
/// The tail can be either a bracketed name, written as `[[name]]`, or a command
/// chain.  The function returns both the parsed tail and the unconsumed suffix
/// so callers can continue parsing argument groups and colon tails.
pub(super) fn parse_refined_tail(input: &str) -> Result<(RefinedTail, &str), ParseError> {
    let input = input.trim_start();
    if let Some(rest) = input.strip_prefix("[[") {
        let end = rest
            .find("]]")
            .ok_or_else(|| ParseError::custom("unterminated refined tail name"))?;
        let name = parse_name_token(&rest[..end])?;
        let consumed = input.len() - rest[end + 2..].len();
        return Ok((
            RefinedTail::Name {
                span: Span::new(0, consumed),
                name,
            },
            &rest[end + 2..],
        ));
    }

    let (chain_text, rest) = split_prefix_by_delimiters(input, &['{', '(', ':']);
    Ok((RefinedTail::Chain(parse_chain(chain_text)?), rest))
}

/// Parses consecutive `{...}` argument groups in a command header.
///
/// Header argument groups contain forms/declarations because command headings
/// define arity and placeholder names, not concrete use-site expressions.
pub(super) fn parse_curly_heading_args(
    mut input: &str,
) -> Result<(Vec<CurlyHeadingArgs>, &str), ParseError> {
    let mut args = Vec::new();
    while input.trim_start().starts_with('{') {
        input = input.trim_start();
        let (inside, rest) = consume_balanced_prefix(input, '{', '}')?;
        args.push(CurlyHeadingArgs {
            span: span_all(&input[..input.len() - rest.len()]),
            forms: parse_form_list(inside)?,
        });
        input = rest;
    }

    Ok((args, input))
}

/// Parses consecutive `(...)` argument groups in a command header.
///
/// Parenthesized header groups describe invocation arguments for callable
/// definitions.  They are optional at use sites but still part of the definition
/// shape when present in a heading.
pub(super) fn parse_paren_heading_args(
    mut input: &str,
) -> Result<(Vec<ParenHeadingArgs>, &str), ParseError> {
    let mut args = Vec::new();
    while input.trim_start().starts_with('(') {
        input = input.trim_start();
        let (inside, rest) = consume_balanced_prefix(input, '(', ')')?;
        args.push(ParenHeadingArgs {
            span: span_all(&input[..input.len() - rest.len()]),
            forms: parse_form_list(inside)?,
        });
        input = rest;
    }

    Ok((args, input))
}

/// Parses colon-prefixed tail parts for command headers.
///
/// A tail part must include at least one `{...}` group, which prevents bare
/// punctuation like `:to` from being accepted without the argument shape that
/// gives the part semantic content.
pub(super) fn parse_command_header_tail(
    mut input: &str,
) -> Result<(Vec<CommandHeaderTailPart>, &str), ParseError> {
    let mut parts = Vec::new();
    loop {
        input = input.trim_start();
        if !input.starts_with(':') || input.starts_with(":/") || input.starts_with("::") {
            break;
        }

        let (chain_text, rest) = split_prefix_by_delimiters(&input[1..], &['{']);
        let chain = parse_chain(chain_text)?;
        let (args, remaining) = parse_curly_heading_args(rest)?;
        if args.is_empty() {
            return Err(ParseError::custom(
                "header tail parts require at least one `{...}` argument list",
            ));
        }

        parts.push(CommandHeaderTailPart {
            span: span_all(&input[..input.len() - remaining.len()]),
            chain,
            args,
        });
        input = remaining;
    }

    Ok((parts, input))
}

/// Parses consecutive `{...}` argument groups in a command expression.
///
/// Expression argument groups carry concrete expression ASTs and therefore use
/// the expression list parser rather than the form/declaration list parser used
/// for command headers.
pub(super) fn parse_curly_expression_args(
    mut input: &str,
) -> Result<(Vec<CurlyExpressionArgs>, &str), ParseError> {
    let mut args = Vec::new();
    while input.trim_start().starts_with('{') {
        input = input.trim_start();
        let (inside, rest) = consume_balanced_prefix(input, '{', '}')?;
        args.push(CurlyExpressionArgs {
            span: span_all(&input[..input.len() - rest.len()]),
            expressions: parse_expression_list(inside)?,
        });
        input = rest;
    }

    Ok((args, input))
}

/// Parses consecutive `(...)` argument groups in a command expression.
///
/// These groups represent invocation at a use site.  Semantic checking later
/// verifies whether the referenced definition allows the supplied parenthesized
/// arity.
pub(super) fn parse_paren_expression_args(
    mut input: &str,
) -> Result<(Vec<ParenExpressionArgs>, &str), ParseError> {
    let mut args = Vec::new();
    while input.trim_start().starts_with('(') {
        input = input.trim_start();
        let (inside, rest) = consume_balanced_prefix(input, '(', ')')?;
        args.push(ParenExpressionArgs {
            span: span_all(&input[..input.len() - rest.len()]),
            expressions: parse_expression_list(inside)?,
        });
        input = rest;
    }

    Ok((args, input))
}

/// Parses colon-prefixed tail parts for command expressions.
///
/// Each tail part must provide at least one `{...}` expression argument group so
/// reference signatures retain the same required shape as their definitions.
pub(super) fn parse_command_expression_tail(
    mut input: &str,
) -> Result<(Vec<CommandExpressionTailPart>, &str), ParseError> {
    let mut parts = Vec::new();
    loop {
        input = input.trim_start();
        if !input.starts_with(':') || input.starts_with(":/") || input.starts_with("::") {
            break;
        }

        let (chain_text, rest) = split_prefix_by_delimiters(&input[1..], &['{']);
        let chain = parse_chain(chain_text)?;
        let (args, remaining) = parse_curly_expression_args(rest)?;
        if args.is_empty() {
            return Err(ParseError::custom(
                "command tail parts require at least one `{...}` argument list",
            ));
        }

        parts.push(CommandExpressionTailPart {
            span: span_all(&input[..input.len() - remaining.len()]),
            chain,
            args,
        });
        input = remaining;
    }

    Ok((parts, input))
}

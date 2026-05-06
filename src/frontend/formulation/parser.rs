use std::fmt;

use lalrpop_util::ParseError as LalrpopParseError;

use super::ast::{
    AuthorHeader, Chain, ChainPart, CommandExpressionTailPart, CommandHeader, CommandHeaderNode,
    CommandHeaderTailPart, CurlyExpressionArgs, CurlyHeadingArgs, Expression, ExpressionAlias,
    ExpressionAliasLhs, ExpressionKind, FormOrDeclaration, FormOrDeclarationKind,
    InfixCommandHeader, IsOrRefinedStatementSpec, IsOrSpec, IsStatement, IsViaStatement,
    LabelHeader, Operator, ParenExpressionArgs, ParenHeadingArgs, Placeholder, PlaceholderForm,
    PlaceholderFormKind, PlaceholderSpecStatement, RefinedCommandExpression, RefinedCommandHeader,
    RefinedExpressionPart, RefinedHeaderPart, RefinedTail, ResourceHeader, SpecOperatorAlias,
    SpecSubject, SpecSubjectKind, SubjectSpecStatement, TupleForm, TypeExpression, WritingAlias,
};
use super::grammar;
use super::lexer::Lexer;
use super::span::Span;
use super::token::{LexicalError, Token};

type LalrpopError = LalrpopParseError<usize, Token, LexicalError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    Grammar(LalrpopError),
    Custom(String),
}

impl ParseError {
    fn custom(message: impl Into<String>) -> Self {
        Self::Custom(message.into())
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Grammar(error) => write!(f, "{error:?}"),
            Self::Custom(message) => write!(f, "{message}"),
        }
    }
}

impl From<LalrpopError> for ParseError {
    fn from(value: LalrpopError) -> Self {
        Self::Grammar(value)
    }
}

pub fn parse_expression(input: &str) -> Result<Expression, ParseError> {
    grammar::InputExpressionParser::new()
        .parse(Lexer::new(input))
        .map_err(ParseError::from)
}

pub fn parse_form_or_declaration(input: &str) -> Result<FormOrDeclaration, ParseError> {
    grammar::InputFormOrDeclarationParser::new()
        .parse(Lexer::new(input))
        .map_err(ParseError::from)
}

pub fn parse_is_or_spec(input: &str) -> Result<IsOrSpec, ParseError> {
    let input = input.trim();
    if contains_top_level(input, " is ") {
        let statement = parse_is_statement(input, false)?;
        return Ok(IsOrSpec::Is(statement));
    }

    parse_subject_spec_statement(input).map(IsOrSpec::Spec)
}

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

pub fn parse_spec_operator_alias(input: &str) -> Result<SpecOperatorAlias, ParseError> {
    let input = input.trim();
    let index = find_top_level_substring(input, ":->")
        .ok_or_else(|| ParseError::custom("expected top-level `:->`"))?;
    let placeholder_spec = parse_placeholder_spec_statement(input[..index].trim())?;
    let target = parse_is_or_spec(input[index + 3..].trim())?;

    Ok(SpecOperatorAlias {
        span: span_all(input),
        placeholder_spec,
        target,
    })
}

pub fn parse_label_header(input: &str) -> Result<LabelHeader, ParseError> {
    let input = input.trim();
    let parts = parse_dotted_parts(input)?;
    Ok(LabelHeader {
        span: span_all(input),
        parts,
    })
}

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

fn parse_is_statement(input: &str, allow_refined: bool) -> Result<IsStatement, ParseError> {
    let index = find_top_level_substring(input, " is ")
        .ok_or_else(|| ParseError::custom("expected top-level ` is `"))?;
    let subject = parse_spec_subject(&input[..index])?;
    let ty = parse_type_expression(&input[index + 4..], allow_refined)?;

    Ok(IsStatement {
        span: span_all(input),
        subject,
        ty,
    })
}

fn parse_subject_spec_statement(input: &str) -> Result<SubjectSpecStatement, ParseError> {
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

fn parse_placeholder_spec_statement(input: &str) -> Result<PlaceholderSpecStatement, ParseError> {
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

fn parse_type_expression(input: &str, allow_refined: bool) -> Result<TypeExpression, ParseError> {
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

fn parse_spec_subject(input: &str) -> Result<SpecSubject, ParseError> {
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

fn parse_operator(input: &str) -> Result<Operator, ParseError> {
    let input = input.trim();
    if !is_operator_text(input) {
        return Err(ParseError::custom(format!(
            "expected operator, found `{input}`"
        )));
    }

    Ok(Operator::new(span_all(input), input))
}

fn parse_tuple_form(input: &str) -> Result<TupleForm, ParseError> {
    let input = input.trim();
    match parse_form_or_declaration(input)?.kind {
        FormOrDeclarationKind::TupleDeclaration { name: None, form } => Ok(form),
        other => Err(ParseError::custom(format!(
            "expected tuple form, found {other:?}"
        ))),
    }
}

fn parse_placeholder_form(input: &str) -> Result<PlaceholderForm, ParseError> {
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

fn parse_placeholder(input: &str) -> Result<Placeholder, ParseError> {
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

fn parse_placeholder_list(input: &str) -> Result<Vec<Placeholder>, ParseError> {
    let mut placeholders = Vec::new();
    for item in split_top_level(input, ',')? {
        placeholders.push(parse_placeholder(item)?);
    }

    if placeholders.is_empty() {
        return Err(ParseError::custom("expected at least one placeholder"));
    }

    Ok(placeholders)
}

fn parse_simple_command_header(input: &str) -> Result<CommandHeaderNode, ParseError> {
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

fn parse_infix_command_header(input: &str) -> Result<InfixCommandHeader, ParseError> {
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

fn parse_refined_command_header(input: &str) -> Result<RefinedCommandHeader, ParseError> {
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

fn parse_refined_header_part(input: &str) -> Result<RefinedHeaderPart, ParseError> {
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

fn parse_refined_command_expression(input: &str) -> Result<RefinedCommandExpression, ParseError> {
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

fn parse_refined_expression_part(input: &str) -> Result<RefinedExpressionPart, ParseError> {
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

fn parse_refined_tail(input: &str) -> Result<(RefinedTail, &str), ParseError> {
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

fn parse_curly_heading_args(mut input: &str) -> Result<(Vec<CurlyHeadingArgs>, &str), ParseError> {
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

fn parse_paren_heading_args(mut input: &str) -> Result<(Vec<ParenHeadingArgs>, &str), ParseError> {
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

fn parse_command_header_tail(
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

fn parse_curly_expression_args(
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

fn parse_paren_expression_args(
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

fn parse_command_expression_tail(
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

fn parse_name_token(input: &str) -> Result<String, ParseError> {
    let input = input.trim();
    if !is_name_text(input) {
        return Err(ParseError::custom(format!("invalid name `{input}`")));
    }

    Ok(input.to_owned())
}

fn split_subject_operator_name(input: &str) -> Option<(&str, &str, &str)> {
    let input = input.trim();
    let mut state = ScanState::default();
    let mut start = None;

    for (index, ch) in input.char_indices() {
        if state.in_quote {
            if ch == '"' {
                let open = start?;
                let subject = input[..open].trim();
                let operator = input[open + 1..index].trim();
                let name = input[index + 1..].trim();
                return Some((subject, operator, name));
            }
            continue;
        }

        if state.in_backtick {
            if ch == '`' {
                state.in_backtick = false;
            }
            continue;
        }

        match ch {
            '"' if state.is_top_level() => {
                start = Some(index);
                state.in_quote = true;
            }
            '`' => state.in_backtick = true,
            '(' => state.paren_depth += 1,
            ')' => state.paren_depth = state.paren_depth.saturating_sub(1),
            '{' => state.brace_depth += 1,
            '}' => state.brace_depth = state.brace_depth.saturating_sub(1),
            '[' => state.bracket_depth += 1,
            ']' => state.bracket_depth = state.bracket_depth.saturating_sub(1),
            _ => {}
        }
    }

    None
}

fn split_prefix_by_delimiters<'a>(input: &'a str, delimiters: &[char]) -> (&'a str, &'a str) {
    if let Some(index) = find_first_top_level_delimiter(input, delimiters) {
        (&input[..index], &input[index..])
    } else {
        (input, "")
    }
}

fn find_first_top_level_delimiter(input: &str, delimiters: &[char]) -> Option<usize> {
    let mut state = ScanState::default();
    for (index, ch) in input.char_indices() {
        if state.is_top_level() && delimiters.contains(&ch) {
            return Some(index);
        }
        state.advance(ch);
    }

    None
}

fn find_first_top_level_char(input: &str, target: char) -> Option<usize> {
    let mut state = ScanState::default();
    for (index, ch) in input.char_indices() {
        if state.is_top_level() && ch == target {
            return Some(index);
        }
        state.advance(ch);
    }

    None
}

fn contains_top_level(input: &str, needle: &str) -> bool {
    find_top_level_substring(input, needle).is_some()
}

fn find_top_level_substring(input: &str, needle: &str) -> Option<usize> {
    let mut state = ScanState::default();
    for (index, ch) in input.char_indices() {
        if state.is_top_level() && input[index..].starts_with(needle) {
            return Some(index);
        }
        state.advance(ch);
    }

    None
}

fn split_top_level(input: &str, delimiter: char) -> Result<Vec<&str>, ParseError> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut state = ScanState::default();

    for (index, ch) in input.char_indices() {
        if state.is_top_level() && ch == delimiter {
            let part = input[start..index].trim();
            if part.is_empty() {
                return Err(ParseError::custom("empty item in comma-separated list"));
            }
            parts.push(part);
            start = index + ch.len_utf8();
        }
        state.advance(ch);
    }

    let tail = input[start..].trim();
    if !tail.is_empty() {
        parts.push(tail);
    }

    Ok(parts)
}

fn consume_balanced_prefix<'a>(
    input: &'a str,
    open: char,
    close: char,
) -> Result<(&'a str, &'a str), ParseError> {
    let input = input.trim_start();
    if !input.starts_with(open) {
        return Err(ParseError::custom(format!("expected `{open}`")));
    }

    let mut depth = 0usize;
    let mut in_quote = false;
    let mut in_backtick = false;

    for (index, ch) in input.char_indices() {
        if index == 0 {
            depth = 1;
            continue;
        }

        if in_quote {
            if ch == '"' {
                in_quote = false;
            }
            continue;
        }

        if in_backtick {
            if ch == '`' {
                in_backtick = false;
            }
            continue;
        }

        match ch {
            '"' => in_quote = true,
            '`' => in_backtick = true,
            c if c == open => depth += 1,
            c if c == close => {
                depth -= 1;
                if depth == 0 {
                    return Ok((&input[1..index], &input[index + close.len_utf8()..]));
                }
            }
            _ => {}
        }
    }

    Err(ParseError::custom(format!(
        "unterminated `{open}` ... `{close}` block"
    )))
}

fn is_name_text(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }

    if input.starts_with('`') && input.ends_with('`') && input.len() >= 2 {
        return true;
    }

    let mut chars = input.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    let Some(last) = input.chars().last() else {
        return false;
    };

    if !first.is_ascii_alphanumeric() || !last.is_ascii_alphanumeric() {
        return false;
    }

    input
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn is_operator_text(input: &str) -> bool {
    if input.is_empty() {
        return false;
    }

    input.chars().all(|ch| "-~!#%^&*\\+=|<>/".contains(ch))
}

fn span_all(input: &str) -> Span {
    Span::new(0, input.trim().len())
}

#[derive(Clone, Copy, Debug, Default)]
struct ScanState {
    paren_depth: usize,
    brace_depth: usize,
    bracket_depth: usize,
    in_quote: bool,
    in_backtick: bool,
}

impl ScanState {
    fn is_top_level(&self) -> bool {
        !self.in_quote
            && !self.in_backtick
            && self.paren_depth == 0
            && self.brace_depth == 0
            && self.bracket_depth == 0
    }

    fn advance(&mut self, ch: char) {
        if self.in_quote {
            if ch == '"' {
                self.in_quote = false;
            }
            return;
        }

        if self.in_backtick {
            if ch == '`' {
                self.in_backtick = false;
            }
            return;
        }

        match ch {
            '"' => self.in_quote = true,
            '`' => self.in_backtick = true,
            '(' => self.paren_depth += 1,
            ')' => self.paren_depth = self.paren_depth.saturating_sub(1),
            '{' => self.brace_depth += 1,
            '}' => self.brace_depth = self.brace_depth.saturating_sub(1),
            '[' => self.bracket_depth += 1,
            ']' => self.bracket_depth = self.bracket_depth.saturating_sub(1),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{
        parse_author_header, parse_command_header, parse_expression, parse_expression_alias,
        parse_form_or_declaration, parse_is_or_refined_statement_spec, parse_is_or_spec,
        parse_is_via_statement, parse_label_header, parse_resource_header,
        parse_spec_operator_alias, parse_writing_alias,
    };
    use crate::frontend::formulation::ast::{
        BinaryOperator, ChainPart, CommandHeader, CommandHeaderNode, Expression,
        ExpressionAliasLhs, ExpressionKind, FormOrDeclarationKind,
        FunctionNamedExpressionElementLhs, IsOrRefinedStatementSpec, IsOrSpec, NamedOperatorKind,
        PlaceholderFormKind, RefinedTail, SpecSubjectKind, SubsetCall, TypeExpression,
    };

    fn split_golden_entries(text: &str) -> Vec<String> {
        text.replace("\r\n", "\n")
            .trim_matches('\n')
            .split("\n\n\n")
            .filter(|entry| !entry.trim().is_empty())
            .map(str::to_owned)
            .collect()
    }

    fn read_golden_entries(path: &str) -> Vec<String> {
        let text = fs::read_to_string(path).expect("expected formulation golden file");
        split_golden_entries(&text)
    }

    fn assert_simple_command_header(
        input: &str,
        expected_chain_parts: &[&str],
        expected_head_args: usize,
        expected_tail: usize,
        expected_paren_args: usize,
    ) {
        let header = parse_command_header(input).expect("expected command header");

        match header {
            CommandHeader::Command(CommandHeaderNode {
                chain,
                head_args,
                tail,
                paren_args,
                ..
            }) => {
                assert_eq!(chain.parts.len(), expected_chain_parts.len());
                for (part, expected) in chain.parts.iter().zip(expected_chain_parts) {
                    assert!(matches!(part, ChainPart::Name(name) if name == expected));
                }
                assert_eq!(head_args.len(), expected_head_args);
                assert_eq!(tail.len(), expected_tail);
                assert_eq!(paren_args.len(), expected_paren_args);
            }
            other => panic!("expected simple command header, got {other:?}"),
        }
    }

    fn assert_command_expression(
        input: &str,
        expected_chain_parts: &[&str],
        expected_head_args: usize,
        expected_tail: usize,
        expected_paren_args: usize,
    ) {
        let expression = parse_expression(input).expect("expected command expression");

        match expression.kind {
            ExpressionKind::Command(command) => {
                assert_eq!(command.chain.parts.len(), expected_chain_parts.len());
                for (part, expected) in command.chain.parts.iter().zip(expected_chain_parts) {
                    assert!(matches!(part, ChainPart::Name(name) if name == expected));
                }
                assert_eq!(command.head_args.len(), expected_head_args);
                assert_eq!(command.tail.len(), expected_tail);
                assert_eq!(command.paren_args.len(), expected_paren_args);
            }
            other => panic!("expected command expression, got {other:?}"),
        }
    }

    #[test]
    fn parses_operator_precedence() {
        let expression = parse_expression("x + y * z").expect("expected expression to parse");

        match expression.kind {
            ExpressionKind::Binary {
                operator: BinaryOperator::Add(_),
                left,
                right,
            } => {
                assert!(matches!(left.kind, ExpressionKind::Name(ref name) if name == "x"));
                assert!(matches!(
                    right.kind,
                    ExpressionKind::Binary {
                        operator: BinaryOperator::Multiply(_),
                        ..
                    }
                ));
            }
            other => panic!("expected additive binary expression, got {other:?}"),
        }
    }

    #[test]
    fn parses_labeled_grouped_expressions() {
        let expression =
            parse_expression("(x + 1)[:some.label:]").expect("expected labeled expression");

        match expression.kind {
            ExpressionKind::Labeled { label, .. } => {
                assert_eq!(label.parts, vec!["some".to_string(), "label".to_string()]);
            }
            other => panic!("expected labeled expression, got {other:?}"),
        }
    }

    #[test]
    fn parses_named_operators() {
        let expression = parse_expression("x {.plus.} y").expect("expected named operator");

        match expression.kind {
            ExpressionKind::Binary {
                operator: BinaryOperator::Named(operator),
                ..
            } => {
                assert_eq!(operator.name, "plus");
                assert_eq!(operator.kind, NamedOperatorKind::Plain);
            }
            other => panic!("expected named operator expression, got {other:?}"),
        }
    }

    #[test]
    fn parses_command_expressions() {
        let expression =
            parse_expression(r#"\function:on{A}:to{B}(x)"#).expect("expected command expression");

        match expression.kind {
            ExpressionKind::Command(command) => {
                assert_eq!(command.chain.parts.len(), 1);
                assert_eq!(command.head_args.len(), 0);
                assert_eq!(command.tail.len(), 2);
                assert_eq!(command.paren_args.len(), 1);
            }
            other => panic!("expected command expression, got {other:?}"),
        }
    }

    #[test]
    fn parses_function_form_declarations() {
        let form = parse_form_or_declaration("g := f(x_, y_)").expect("expected form declaration");

        match form.kind {
            FormOrDeclarationKind::FunctionDeclaration { name, form } => {
                assert_eq!(name.as_deref(), Some("g"));
                assert_eq!(form.name, "f");
                assert_eq!(form.placeholders.len(), 2);
            }
            other => panic!("expected function declaration, got {other:?}"),
        }
    }

    #[test]
    fn parses_operator_forms() {
        let form = parse_form_or_declaration("x_ {.plus.} y_").expect("expected infix operator");

        match form.kind {
            FormOrDeclarationKind::InfixOperator { operator, .. } => {
                assert_eq!(operator.text, "plus");
            }
            other => panic!("expected infix operator form, got {other:?}"),
        }
    }

    #[test]
    fn parses_is_or_spec_statements() {
        let item = parse_is_or_spec(r#"f(x_) is \function:on{A}:to{B}"#)
            .expect("expected is-or-spec statement");

        match item {
            IsOrSpec::Is(statement) => {
                assert!(matches!(statement.ty, TypeExpression::Command(_)));
            }
            other => panic!("expected is statement, got {other:?}"),
        }
    }

    #[test]
    fn parses_is_via_statements() {
        let item = parse_is_via_statement(r#"x is \type{A} via (x, y)"#)
            .expect("expected is-via statement");

        assert_eq!(item.tuple_form.elements.len(), 2);
    }

    #[test]
    fn parses_refined_is_statements() {
        let item = parse_is_or_refined_statement_spec(r#"x is \(f)::[[g]]"#)
            .expect("expected refined is statement");

        match item {
            IsOrRefinedStatementSpec::Is(statement) => {
                assert!(matches!(statement.ty, TypeExpression::RefinedCommand(_)));
            }
            other => panic!("expected refined is statement, got {other:?}"),
        }
    }

    #[test]
    fn parses_command_headers() {
        let header = parse_command_header(r#"\function:on{A}:to{B}(f(x_))"#)
            .expect("expected command header");

        match header {
            CommandHeader::Command(CommandHeaderNode {
                tail, paren_args, ..
            }) => {
                assert_eq!(tail.len(), 2);
                assert_eq!(paren_args.len(), 1);
            }
            other => panic!("expected command header, got {other:?}"),
        }
    }

    #[test]
    fn parses_commands_without_colon_arguments() {
        assert_simple_command_header(r#"\set"#, &["set"], 0, 0, 0);
        assert_simple_command_header(r#"\closed.set"#, &["closed", "set"], 0, 0, 0);
        assert_simple_command_header(
            r#"\axiom.of.existence.of.empty.set"#,
            &["axiom", "of", "existence", "of", "empty", "set"],
            0,
            0,
            0,
        );
    }

    #[test]
    fn parses_command_variants_with_optional_head_tail_and_paren_arguments() {
        assert_simple_command_header(r#"\Z{n}"#, &["Z"], 1, 0, 0);
        assert_simple_command_header(r#"\function:on{A}:to{B}"#, &["function"], 0, 2, 0);
        assert_simple_command_header(r#"\group:over{A}"#, &["group"], 0, 1, 0);
        assert_simple_command_header(r#"\sin(x)"#, &["sin"], 0, 0, 1);
        assert_simple_command_header(
            r#"\generalized.zeta{n}(x)"#,
            &["generalized", "zeta"],
            1,
            0,
            1,
        );
        assert_simple_command_header(
            r#"\some.function:on{A}(x,y)"#,
            &["some", "function"],
            0,
            1,
            1,
        );
    }

    #[test]
    fn parses_refined_command_headers() {
        let header =
            parse_command_header(r#"\(f)::[[g]]"#).expect("expected refined command header");

        match header {
            CommandHeader::Refined(header) => {
                assert_eq!(header.parts.len(), 1);
                assert!(matches!(header.refined_tail, RefinedTail::Name { .. }));
            }
            other => panic!("expected refined command header, got {other:?}"),
        }
    }

    #[test]
    fn parses_expression_aliases() {
        let alias = parse_expression_alias(r#"\function:on{A}:to{B}(f(x_)) :=> x"#)
            .expect("expected expression alias");

        match alias.lhs {
            ExpressionAliasLhs::Command(_) => {}
            other => panic!("expected command header lhs, got {other:?}"),
        }
    }

    #[test]
    fn parses_spec_operator_aliases() {
        let alias = parse_spec_operator_alias(r#"x_ "in" X :-> x is \type{A}"#)
            .expect("expected spec operator alias");

        assert_eq!(alias.placeholder_spec.operator, "in");
    }

    #[test]
    fn parses_header_labels() {
        let label = parse_label_header("some.label").expect("expected label header");
        let author = parse_author_header("@euclid").expect("expected author header");
        let resource = parse_resource_header("$book.ref").expect("expected resource header");

        assert_eq!(label.parts, vec!["some".to_string(), "label".to_string()]);
        assert_eq!(author.parts, vec!["euclid".to_string()]);
        assert_eq!(resource.parts, vec!["book".to_string(), "ref".to_string()]);
    }

    #[test]
    fn parses_unary_and_dot_grouped_expressions() {
        let expression = parse_expression("-(.x + y.)").expect("expected unary grouped expression");

        match expression.kind {
            ExpressionKind::Prefix { expression, .. } => match expression.kind {
                ExpressionKind::Grouped {
                    expression,
                    dot_delimited,
                } => {
                    assert!(dot_delimited);
                    assert!(matches!(
                        expression.kind,
                        ExpressionKind::Binary {
                            operator: BinaryOperator::Add(_),
                            ..
                        }
                    ));
                }
                other => panic!("expected grouped expression, got {other:?}"),
            },
            other => panic!("expected prefix expression, got {other:?}"),
        }
    }

    #[test]
    fn parses_right_associative_power_expressions() {
        let expression = parse_expression("x ^ y ^ z").expect("expected power expression");

        match expression.kind {
            ExpressionKind::Binary {
                operator: BinaryOperator::Power(_),
                left,
                right,
            } => {
                assert!(matches!(left.kind, ExpressionKind::Name(ref name) if name == "x"));
                assert!(matches!(
                    right.kind,
                    ExpressionKind::Binary {
                        operator: BinaryOperator::Power(_),
                        ..
                    }
                ));
            }
            other => panic!("expected power expression, got {other:?}"),
        }
    }

    #[test]
    fn parses_function_calls_and_tuple_expressions_with_operator_elements() {
        let function_call =
            parse_expression("f(x, y + z)").expect("expected function call expression");
        let tuple = parse_expression("(x, +, y)").expect("expected tuple expression");

        match function_call.kind {
            ExpressionKind::FunctionCall { name, arguments } => {
                assert_eq!(name, "f");
                assert_eq!(arguments.len(), 2);
                assert!(matches!(
                    arguments[1].kind,
                    ExpressionKind::Binary {
                        operator: BinaryOperator::Add(_),
                        ..
                    }
                ));
            }
            other => panic!("expected function call, got {other:?}"),
        }

        match tuple.kind {
            ExpressionKind::Tuple(elements) => {
                assert_eq!(elements.len(), 3);
                assert!(matches!(
                    elements[1],
                    crate::frontend::formulation::ast::TupleExpressionElement::Operator(_)
                ));
            }
            other => panic!("expected tuple expression, got {other:?}"),
        }
    }

    #[test]
    fn parses_set_expressions_with_predicates_and_placeholder_function_targets() {
        let expression = parse_expression(r#"{x "in" X : f_(a_, b_) | y is \type{A}}"#)
            .expect("expected set expression");

        match expression.kind {
            ExpressionKind::Set(set) => {
                assert_eq!(set.spec.operator, "in");
                assert_eq!(set.spec.name, "X");
                match set.target.kind {
                    PlaceholderFormKind::Function {
                        placeholder,
                        arguments,
                    } => {
                        assert_eq!(placeholder.name, "f");
                        assert_eq!(arguments.len(), 2);
                    }
                    other => panic!("expected placeholder function target, got {other:?}"),
                }
                assert!(matches!(
                    set.predicate.as_deref(),
                    Some(Expression {
                        kind: ExpressionKind::IsType { .. },
                        ..
                    })
                ));
            }
            other => panic!("expected set expression, got {other:?}"),
        }
    }

    #[test]
    fn parses_direct_subset_calls_and_named_function_calls() {
        let subset = parse_expression("Domain[element]").expect("expected subset expression");
        let named_call =
            parse_expression("f[|value := x, Pair[left, right] := y, nested[outer[inner]] := z|]")
                .expect("expected named function call");

        match subset.kind {
            ExpressionKind::SubsetCall(SubsetCall::One { target, first, .. }) => {
                assert_eq!(target, "Domain");
                assert_eq!(first, "element");
            }
            other => panic!("expected one-argument subset call, got {other:?}"),
        }

        match named_call.kind {
            ExpressionKind::FunctionNamedCall { name, elements } => {
                assert_eq!(name, "f");
                assert_eq!(elements.len(), 3);
                assert!(matches!(
                    elements[0].lhs,
                    FunctionNamedExpressionElementLhs::Name(ref value) if value == "value"
                ));
                assert!(matches!(
                    elements[1].lhs,
                    FunctionNamedExpressionElementLhs::SubsetCall(SubsetCall::Two {
                        ref target,
                        ref first,
                        ref second,
                        ..
                    }) if target == "Pair" && first == "left" && second == "right"
                ));
                assert!(matches!(
                    elements[2].lhs,
                    FunctionNamedExpressionElementLhs::SubsetCall(SubsetCall::Nested {
                        ref target,
                        ref outer,
                        ref inner_target,
                        ..
                    }) if target == "nested" && outer == "outer" && inner_target == "inner"
                ));
            }
            other => panic!("expected function named call, got {other:?}"),
        }
    }

    #[test]
    fn parses_spec_and_predicate_expression_variants() {
        let spec = parse_expression(r#"x "in" X"#).expect("expected spec expression");
        let predicate = parse_expression(r#"x is? \even"#).expect("expected predicate expression");
        let negative = parse_expression(r#"x is_not? \odd"#).expect("expected negative predicate");

        assert!(matches!(
            spec.kind,
            ExpressionKind::SpecStatement(ref statement)
                if statement.operator == "in" && statement.name == "X"
        ));
        assert!(matches!(predicate.kind, ExpressionKind::IsPredicate { .. }));
        assert!(matches!(
            negative.kind,
            ExpressionKind::IsNotPredicate { .. }
        ));
    }

    #[test]
    fn parses_infix_command_expressions_with_alias_and_operator_chain_parts() {
        let expression = parse_expression(r#"x \:map.$alias.=={A}:to{B}:/ y"#)
            .expect("expected infix command expression");

        match expression.kind {
            ExpressionKind::InfixCommand {
                left,
                command,
                right,
            } => {
                assert!(matches!(left.kind, ExpressionKind::Name(ref name) if name == "x"));
                assert!(matches!(right.kind, ExpressionKind::Name(ref name) if name == "y"));
                assert_eq!(command.chain.parts.len(), 3);
                assert!(matches!(
                    command.chain.parts[0],
                    ChainPart::Name(ref name) if name == "map"
                ));
                assert!(matches!(
                    command.chain.parts[1],
                    ChainPart::Alias(ref name) if name == "alias"
                ));
                assert!(matches!(
                    command.chain.parts[2],
                    ChainPart::Operator(ref operator) if operator == "=="
                ));
                assert_eq!(command.head_args.len(), 1);
                assert_eq!(command.tail.len(), 1);
                assert_eq!(command.tail[0].args.len(), 1);
            }
            other => panic!("expected infix command expression, got {other:?}"),
        }
    }

    #[test]
    fn parses_command_expressions_with_multiple_head_tail_and_paren_args() {
        let expression = parse_expression(r#"\logic.$alias.=={x}{y}:from{A}{B}:to{C}(x, y)(z)"#)
            .expect("expected command expression");

        match expression.kind {
            ExpressionKind::Command(command) => {
                assert_eq!(command.chain.parts.len(), 3);
                assert_eq!(command.head_args.len(), 2);
                assert_eq!(command.tail.len(), 2);
                assert_eq!(command.tail[0].args.len(), 2);
                assert_eq!(command.tail[1].args.len(), 1);
                assert_eq!(command.paren_args.len(), 2);
            }
            other => panic!("expected command expression, got {other:?}"),
        }
    }

    #[test]
    fn parses_command_expressions_for_simple_and_dotted_command_names() {
        assert_command_expression(r#"\set"#, &["set"], 0, 0, 0);
        assert_command_expression(r#"\closed.set"#, &["closed", "set"], 0, 0, 0);
        assert_command_expression(
            r#"\axiom.of.existence.of.empty.set"#,
            &["axiom", "of", "existence", "of", "empty", "set"],
            0,
            0,
            0,
        );
        assert_command_expression(r#"\Z{n}"#, &["Z"], 1, 0, 0);
        assert_command_expression(r#"\function:on{A}:to{B}"#, &["function"], 0, 2, 0);
        assert_command_expression(r#"\group:over{A}"#, &["group"], 0, 1, 0);
        assert_command_expression(r#"\sin(x)"#, &["sin"], 0, 0, 1);
        assert_command_expression(
            r#"\generalized.zeta{n}(x)"#,
            &["generalized", "zeta"],
            1,
            0,
            1,
        );
        assert_command_expression(
            r#"\some.function:on{A}(x,y)"#,
            &["some", "function"],
            0,
            1,
            1,
        );
    }

    #[test]
    fn parses_name_function_tuple_and_set_form_declarations() {
        let name_form = parse_form_or_declaration("Value").expect("expected name form");
        let magnetic_function =
            parse_form_or_declaration("f(x__)").expect("expected magnetic function form");
        let tuple_declaration =
            parse_form_or_declaration("pair := (x, +, y)").expect("expected tuple declaration");
        let set_declaration =
            parse_form_or_declaration("Set := {f_(x_, y_)}").expect("expected set declaration");

        assert!(matches!(
            name_form.kind,
            FormOrDeclarationKind::Name(ref name) if name == "Value"
        ));

        match magnetic_function.kind {
            FormOrDeclarationKind::FunctionDeclaration { name, form } => {
                assert!(name.is_none());
                assert_eq!(form.name, "f");
                assert!(form.magnetic_placeholder.is_some());
                assert!(form.placeholders.is_empty());
            }
            other => panic!("expected function declaration, got {other:?}"),
        }

        match tuple_declaration.kind {
            FormOrDeclarationKind::TupleDeclaration { name, form } => {
                assert_eq!(name.as_deref(), Some("pair"));
                assert_eq!(form.elements.len(), 3);
                assert!(matches!(
                    form.elements[1],
                    crate::frontend::formulation::ast::TupleFormElement::Operator(_)
                ));
            }
            other => panic!("expected tuple declaration, got {other:?}"),
        }

        match set_declaration.kind {
            FormOrDeclarationKind::SetDeclaration { name, form } => {
                assert_eq!(name.as_deref(), Some("Set"));
                match form.placeholder_form.kind {
                    PlaceholderFormKind::Function { arguments, .. } => {
                        assert_eq!(arguments.len(), 2);
                    }
                    other => panic!("expected placeholder function form, got {other:?}"),
                }
            }
            other => panic!("expected set declaration, got {other:?}"),
        }
    }

    #[test]
    fn parses_prefix_postfix_and_infix_operator_forms() {
        let prefix = parse_form_or_declaration("-x_").expect("expected prefix operator form");
        let postfix = parse_form_or_declaration("x_ !").expect("expected postfix operator form");
        let infix =
            parse_form_or_declaration("x_ {.plus.} y_").expect("expected infix operator form");

        assert!(matches!(
            prefix.kind,
            FormOrDeclarationKind::PrefixOperator { ref operator, .. } if operator.text == "-"
        ));
        assert!(matches!(
            postfix.kind,
            FormOrDeclarationKind::PostfixOperator { ref operator, .. } if operator.text == "!"
        ));
        assert!(matches!(
            infix.kind,
            FormOrDeclarationKind::InfixOperator { ref operator, .. } if operator.text == "plus"
        ));
    }

    #[test]
    fn parses_left_and_right_named_operator_expressions() {
        let left = parse_expression("x :{.before.} y").expect("expected left-colon operator");
        let right = parse_expression("x {.after.}: y").expect("expected right-colon operator");

        assert!(matches!(
            left.kind,
            ExpressionKind::Binary {
                operator: BinaryOperator::Named(ref operator),
                ..
            } if operator.kind == NamedOperatorKind::LeftColon && operator.name == "before"
        ));
        assert!(matches!(
            right.kind,
            ExpressionKind::Binary {
                operator: BinaryOperator::Named(ref operator),
                ..
            } if operator.kind == NamedOperatorKind::RightColon && operator.name == "after"
        ));
    }

    #[test]
    fn parses_operator_subject_specs() {
        let item = parse_is_or_spec(r#"+ "on" Nat"#).expect("expected operator spec statement");

        match item {
            IsOrSpec::Spec(statement) => {
                assert_eq!(statement.operator, "on");
                assert_eq!(statement.name, "Nat");
                assert!(matches!(
                    statement.subject.kind,
                    SpecSubjectKind::Operator(ref operator) if operator.text == "+"
                ));
            }
            other => panic!("expected spec statement, got {other:?}"),
        }
    }

    #[test]
    fn parses_refined_command_headers_with_prefix_and_chain_tails() {
        let header = parse_command_header(
            r#"\logic.$prefix.(f:on{A}, g:to{B})::target.$tail{C}:via{D}(E, F)"#,
        )
        .expect("expected refined command header");

        match header {
            CommandHeader::Refined(header) => {
                let prefix_chain = header
                    .prefix_chain
                    .expect("expected refined header prefix chain");
                assert_eq!(prefix_chain.parts.len(), 2);
                assert_eq!(header.parts.len(), 2);
                match header.refined_tail {
                    RefinedTail::Chain(chain) => {
                        assert_eq!(chain.parts.len(), 2);
                        assert!(matches!(
                            chain.parts[1],
                            ChainPart::Alias(ref name) if name == "tail"
                        ));
                    }
                    other => panic!("expected chain refined tail, got {other:?}"),
                }
                assert_eq!(header.head_args.len(), 1);
                assert_eq!(header.tail.len(), 1);
                assert_eq!(header.paren_args.len(), 1);
            }
            other => panic!("expected refined command header, got {other:?}"),
        }
    }

    #[test]
    fn parses_refined_command_expressions_with_prefix_and_chain_tails() {
        let item = parse_is_or_refined_statement_spec(
            r#"x is \logic.$prefix.(f:on{x}, g:to{y})::target.$tail{z}:via{w}(u, v)"#,
        )
        .expect("expected refined command expression");

        match item {
            IsOrRefinedStatementSpec::Is(statement) => match statement.ty {
                TypeExpression::RefinedCommand(command) => {
                    let prefix_chain = command
                        .prefix_chain
                        .expect("expected refined expression prefix chain");
                    assert_eq!(prefix_chain.parts.len(), 2);
                    assert_eq!(command.parts.len(), 2);
                    match command.refined_tail {
                        RefinedTail::Chain(chain) => {
                            assert_eq!(chain.parts.len(), 2);
                        }
                        other => panic!("expected chain refined tail, got {other:?}"),
                    }
                    assert_eq!(command.head_args.len(), 1);
                    assert_eq!(command.tail.len(), 1);
                    assert_eq!(command.paren_args.len(), 1);
                }
                other => panic!("expected refined command type, got {other:?}"),
            },
            other => panic!("expected refined is statement, got {other:?}"),
        }
    }

    #[test]
    fn parses_writing_aliases() {
        let alias =
            parse_writing_alias(r#"f(x_) :~> x + y"#).expect("expected writing alias to parse");

        match alias.form.kind {
            FormOrDeclarationKind::FunctionDeclaration { name, form } => {
                assert!(name.is_none());
                assert_eq!(form.name, "f");
            }
            other => panic!("expected function declaration lhs, got {other:?}"),
        }
        assert_eq!(alias.body, "x + y");
    }

    #[test]
    fn parses_expression_aliases_for_form_and_infix_command_lhs_values() {
        let form_alias = parse_expression_alias(r#"f(x_) :=> x + y"#).expect("expected form alias");
        let infix_alias = parse_expression_alias(r#"\:apply:on{A}:/ :=> x"#)
            .expect("expected infix command alias");

        assert!(matches!(form_alias.lhs, ExpressionAliasLhs::Form(_)));
        assert!(matches!(
            form_alias.expression.kind,
            ExpressionKind::Binary {
                operator: BinaryOperator::Add(_),
                ..
            }
        ));
        assert!(matches!(
            infix_alias.lhs,
            ExpressionAliasLhs::InfixCommand(_)
        ));
    }

    #[test]
    fn rejects_refined_command_headers_as_expression_alias_lhs_values() {
        let error = parse_expression_alias(r#"\(f)::[[g]] :=> x"#)
            .expect_err("expected refined command alias lhs to be rejected");

        assert_eq!(
            error.to_string(),
            "refined command headers are not valid expression alias lhs values"
        );
    }

    #[test]
    fn parses_spec_operator_aliases_with_spec_targets() {
        let alias = parse_spec_operator_alias(r#"x_ "in" X :-> + "on" Nat"#)
            .expect("expected spec operator alias");

        assert_eq!(alias.placeholder_spec.operator, "in");
        assert!(matches!(alias.target, IsOrSpec::Spec(_)));
    }

    #[test]
    fn parses_header_variants_with_backtick_segments() {
        let label =
            parse_label_header(r#"logic.`set.ops`.ref"#).expect("expected dotted label header");
        let author =
            parse_author_header(r#"@`isaac.newton`"#).expect("expected backtick author header");
        let resource =
            parse_resource_header(r#"$book.`chapter.1`"#).expect("expected resource header");

        assert_eq!(
            label.parts,
            vec![
                "logic".to_string(),
                "`set.ops`".to_string(),
                "ref".to_string(),
            ]
        );
        assert_eq!(author.parts, vec!["`isaac.newton`".to_string()]);
        assert_eq!(
            resource.parts,
            vec!["book".to_string(), "`chapter.1`".to_string()]
        );
    }

    #[test]
    fn rejects_missing_writing_alias_bodies() {
        let error =
            parse_writing_alias(r#"f(x_) :~>   "#).expect_err("expected empty body to fail");

        assert_eq!(error.to_string(), "writing alias body cannot be empty");
    }

    #[test]
    fn rejects_missing_via_clauses_in_is_via_statements() {
        let error = parse_is_via_statement(r#"x is \type{A}"#)
            .expect_err("expected missing via clause to fail");

        assert_eq!(error.to_string(), "expected top-level ` via `");
    }

    #[test]
    fn rejects_command_headers_without_leading_backslashes() {
        let error = parse_command_header("function:on{A}")
            .expect_err("expected header without slash to fail");

        assert_eq!(error.to_string(), "command header must start with `\\`");
    }

    #[test]
    fn rejects_command_header_tail_parts_without_curly_argument_lists() {
        let error = parse_command_header(r#"\function:on"#)
            .expect_err("expected tail without arguments to fail");

        assert_eq!(
            error.to_string(),
            "header tail parts require at least one `{...}` argument list"
        );
    }

    #[test]
    fn parses_formulation_golden_file() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/goldens/formulation.golden.txt"
        );
        let entries = read_golden_entries(path);

        assert!(!entries.is_empty(), "expected formulation golden entries");

        for (index, entry) in entries.iter().enumerate() {
            let expression = parse_expression(entry).unwrap_or_else(|error| {
                panic!(
                    "failed to parse formulation golden case {}: {error}\n\n{}",
                    index + 1,
                    entry
                )
            });

            assert_eq!(
                expression.span.start,
                0,
                "formulation golden case {} did not cover the full input start",
                index + 1
            );
            assert_eq!(
                expression.span.end,
                entry.len(),
                "formulation golden case {} did not cover the full input end",
                index + 1
            );
            assert_eq!(
                &entry[expression.span.start..expression.span.end],
                entry,
                "formulation golden case {} did not round-trip",
                index + 1
            );
        }
    }
}

use super::*;

pub(super) fn shape_for_header(header: &CommandHeader) -> SignatureShape {
    match header {
        CommandHeader::Command(command) => shape_for_command_header_node(command),
        CommandHeader::Infix(command) => shape_for_infix_command_header(command),
        CommandHeader::InfixSpec(spec) => shape_for_infix_spec_header(spec),
        CommandHeader::Refined(command) => shape_for_refined_command_header(command),
    }
}

pub(super) fn shapes_for_header(header: &CommandHeader) -> Vec<HeaderShape> {
    match header {
        CommandHeader::Command(command) => shapes_for_command_header_node(command),
        CommandHeader::Infix(command) => shapes_for_infix_command_header(command),
        CommandHeader::InfixSpec(spec) => shapes_for_infix_spec_header(spec),
        CommandHeader::Refined(command) => shapes_for_refined_command_header(command),
    }
}

pub(super) fn shape_for_command_header_node(command: &CommandHeaderNode) -> SignatureShape {
    let mut signature = format!("\\{}", format_chain(&command.chain));
    let mut arg_groups = Vec::new();
    add_heading_curly_groups(&mut arg_groups, &command.head_args);
    add_header_tail(&mut signature, &mut arg_groups, &command.tail);
    for args in &command.paren_args {
        arg_groups.push(ArgGroupShape {
            delimiter: ArgDelimiter::Paren,
            count: args.forms.len(),
        });
    }
    SignatureShape {
        signature,
        arg_groups,
        fallback_shapes: Vec::new(),
    }
}

pub(super) fn shapes_for_command_header_node(command: &CommandHeaderNode) -> Vec<HeaderShape> {
    let base_signature = format!("\\{}", format_chain(&command.chain));
    let mut base_type_key = base_signature.clone();
    append_heading_curly_key_groups(&mut base_type_key, &command.head_args);
    let mut base_arg_groups = Vec::new();
    add_heading_curly_groups(&mut base_arg_groups, &command.head_args);
    let base_parameters = heading_group_parameters(&command.head_args);
    let paren_arg_groups = paren_heading_group_shapes(&command.paren_args);
    let paren_parameters = paren_heading_group_parameters(&command.paren_args);
    let paren_type_key_suffix = paren_heading_group_key_suffix(&command.paren_args);

    header_tail_variants(&command.tail)
        .into_iter()
        .map(|variant| {
            let mut arg_groups = base_arg_groups.clone();
            arg_groups.extend(variant.arg_groups);
            arg_groups.extend(paren_arg_groups.clone());

            let mut parameters = base_parameters.clone();
            parameters.extend(variant.parameters);
            parameters.extend(paren_parameters.clone());

            HeaderShape {
                shape: SignatureShape {
                    signature: format!("{base_signature}{}", variant.signature_suffix),
                    arg_groups,
                    fallback_shapes: Vec::new(),
                },
                parameters,
                hidden_parameters: variant.hidden_parameters.clone(),
                type_key: format!(
                    "{base_type_key}{}{}",
                    variant.type_key_suffix, paren_type_key_suffix
                ),
            }
        })
        .collect()
}

pub(super) fn shape_for_infix_command_header(command: &InfixCommandHeader) -> SignatureShape {
    let mut signature = format!("\\.{}", format_chain(&command.chain));
    let mut arg_groups = Vec::new();
    add_heading_curly_groups(&mut arg_groups, &command.head_args);
    add_header_tail(&mut signature, &mut arg_groups, &command.tail);
    signature.push_str("./");
    SignatureShape {
        signature,
        arg_groups,
        fallback_shapes: Vec::new(),
    }
}

pub(super) fn shapes_for_infix_command_header(command: &InfixCommandHeader) -> Vec<HeaderShape> {
    let base_signature = format!("\\.{}", format_chain(&command.chain));
    let mut base_type_key = base_signature.clone();
    append_heading_curly_key_groups(&mut base_type_key, &command.head_args);
    let mut base_arg_groups = Vec::new();
    add_heading_curly_groups(&mut base_arg_groups, &command.head_args);
    let mut base_parameters = infix_operand_parameters(command.left.as_ref());
    base_parameters.extend(heading_group_parameters(&command.head_args));
    let right_parameters = infix_operand_parameters(command.right.as_ref());

    header_tail_variants(&command.tail)
        .into_iter()
        .map(|variant| {
            let mut arg_groups = base_arg_groups.clone();
            arg_groups.extend(variant.arg_groups);

            let mut parameters = base_parameters.clone();
            parameters.extend(variant.parameters);
            parameters.extend(right_parameters.clone());

            HeaderShape {
                shape: SignatureShape {
                    signature: format!("{base_signature}{}./", variant.signature_suffix),
                    arg_groups,
                    fallback_shapes: Vec::new(),
                },
                parameters,
                hidden_parameters: variant.hidden_parameters.clone(),
                type_key: format!("{base_type_key}{}./", variant.type_key_suffix),
            }
        })
        .collect()
}

pub(super) fn shape_for_infix_spec_header(spec: &InfixSpecHeader) -> SignatureShape {
    let mut signature = format!("\\:{}", format_chain(&spec.chain));
    let mut arg_groups = Vec::new();
    add_heading_curly_groups(&mut arg_groups, &spec.head_args);
    add_header_tail(&mut signature, &mut arg_groups, &spec.tail);
    signature.push_str(":/");
    SignatureShape {
        signature,
        arg_groups,
        fallback_shapes: Vec::new(),
    }
}

pub(super) fn shapes_for_infix_spec_header(spec: &InfixSpecHeader) -> Vec<HeaderShape> {
    let base_signature = format!("\\:{}", format_chain(&spec.chain));
    let mut base_type_key = base_signature.clone();
    append_heading_curly_key_groups(&mut base_type_key, &spec.head_args);
    let mut base_arg_groups = Vec::new();
    add_heading_curly_groups(&mut base_arg_groups, &spec.head_args);
    let mut base_parameters = vec![key_for_form_or_declaration(&spec.left)];
    base_parameters.extend(heading_group_parameters(&spec.head_args));
    let right_parameters = vec![key_for_form_or_declaration(&spec.right)];

    header_tail_variants(&spec.tail)
        .into_iter()
        .map(|variant| {
            let mut arg_groups = base_arg_groups.clone();
            arg_groups.extend(variant.arg_groups);

            let mut parameters = base_parameters.clone();
            parameters.extend(variant.parameters);
            parameters.extend(right_parameters.clone());

            HeaderShape {
                shape: SignatureShape {
                    signature: format!("{base_signature}{}:/", variant.signature_suffix),
                    arg_groups,
                    fallback_shapes: Vec::new(),
                },
                parameters,
                hidden_parameters: variant.hidden_parameters.clone(),
                type_key: format!("{base_type_key}{}:/", variant.type_key_suffix),
            }
        })
        .collect()
}

pub(super) fn shape_for_refined_command_header(command: &RefinedCommandHeader) -> SignatureShape {
    let mut signature = "\\".to_string();
    if let Some(prefix) = &command.prefix_chain {
        signature.push_str(&format_chain(prefix));
        signature.push_str("::");
    }
    let mut arg_groups = Vec::new();
    for (index, part) in command.parts.iter().enumerate() {
        if index > 0 {
            signature.push_str("::");
        }
        signature.push_str(&format_chain(&part.chain));
        add_header_tail(&mut signature, &mut arg_groups, &part.tail);
    }
    signature.push_str("::");
    signature.push_str(&format_refined_tail(&command.refined_tail));
    add_heading_curly_groups(&mut arg_groups, &command.head_args);
    add_header_tail(&mut signature, &mut arg_groups, &command.tail);
    for args in &command.paren_args {
        arg_groups.push(ArgGroupShape {
            delimiter: ArgDelimiter::Paren,
            count: args.forms.len(),
        });
    }
    SignatureShape {
        signature,
        arg_groups,
        fallback_shapes: Vec::new(),
    }
}

pub(super) fn shapes_for_refined_command_header(
    command: &RefinedCommandHeader,
) -> Vec<HeaderShape> {
    let mut variants = vec![HeaderVariant::default()];
    let prefix = if let Some(prefix) = &command.prefix_chain {
        format!("\\{}::", format_chain(prefix))
    } else {
        "\\".to_string()
    };

    for (index, part) in command.parts.iter().enumerate() {
        let part_prefix = if index == 0 {
            format!("{prefix}{}", format_chain(&part.chain))
        } else {
            format!("::{}", format_chain(&part.chain))
        };
        let part_tail_variants = header_tail_variants(&part.tail);
        variants = combine_header_variants(
            variants,
            &part_tail_variants,
            |variant| format!("{part_prefix}{}", variant.signature_suffix),
            |variant| format!("{part_prefix}{}", variant.type_key_suffix),
        );
    }

    let refined_tail = format!("::{}", format_refined_tail(&command.refined_tail));
    let head_arg_groups = heading_group_shapes(&command.head_args);
    let head_parameters = heading_group_parameters(&command.head_args);
    let head_type_key_suffix = heading_group_key_suffix(&command.head_args);
    let tail_variants = header_tail_variants(&command.tail);
    let paren_arg_groups = paren_heading_group_shapes(&command.paren_args);
    let paren_parameters = paren_heading_group_parameters(&command.paren_args);
    let paren_type_key_suffix = paren_heading_group_key_suffix(&command.paren_args);

    variants
        .into_iter()
        .flat_map(|prefix_variant| {
            let head_arg_groups = head_arg_groups.clone();
            let head_parameters = head_parameters.clone();
            let head_type_key_suffix = head_type_key_suffix.clone();
            let paren_arg_groups = paren_arg_groups.clone();
            let paren_parameters = paren_parameters.clone();
            let paren_type_key_suffix = paren_type_key_suffix.clone();
            let refined_tail = refined_tail.clone();
            tail_variants.iter().map(move |tail_variant| {
                let mut signature = prefix_variant.signature_suffix.clone();
                signature.push_str(&refined_tail);
                signature.push_str(&tail_variant.signature_suffix);

                let mut arg_groups = prefix_variant.arg_groups.clone();
                arg_groups.extend(head_arg_groups.clone());
                arg_groups.extend(tail_variant.arg_groups.clone());
                arg_groups.extend(paren_arg_groups.clone());

                let mut parameters = prefix_variant.parameters.clone();
                parameters.extend(head_parameters.clone());
                parameters.extend(tail_variant.parameters.clone());
                parameters.extend(paren_parameters.clone());

                HeaderShape {
                    shape: SignatureShape {
                        signature,
                        arg_groups,
                        fallback_shapes: Vec::new(),
                    },
                    parameters,
                    hidden_parameters: {
                        let mut hidden_parameters = prefix_variant.hidden_parameters.clone();
                        hidden_parameters.extend(tail_variant.hidden_parameters.clone());
                        hidden_parameters
                    },
                    type_key: format!(
                        "{}{}{}{}{}",
                        prefix_variant.type_key_suffix,
                        refined_tail,
                        head_type_key_suffix,
                        tail_variant.type_key_suffix,
                        paren_type_key_suffix
                    ),
                }
            })
        })
        .collect()
}

pub(super) fn shape_for_command_expression(command: &CommandExpression) -> SignatureShape {
    let mut signature = format!("\\{}", format_chain(&command.chain));
    let mut arg_groups = Vec::new();
    add_expression_curly_groups(&mut arg_groups, &command.head_args);
    add_expression_tail(&mut signature, &mut arg_groups, &command.tail);
    for args in &command.paren_args {
        arg_groups.push(ArgGroupShape {
            delimiter: ArgDelimiter::Paren,
            count: args.expressions.len(),
        });
    }
    SignatureShape {
        signature,
        arg_groups,
        fallback_shapes: Vec::new(),
    }
}

pub(super) fn shape_for_infix_command(command: &InfixCommand) -> SignatureShape {
    let mut signature = format!("\\.{}", format_chain(&command.chain));
    let mut arg_groups = Vec::new();
    add_expression_curly_groups(&mut arg_groups, &command.head_args);
    add_expression_tail(&mut signature, &mut arg_groups, &command.tail);
    signature.push_str("./");
    SignatureShape {
        signature,
        arg_groups,
        fallback_shapes: Vec::new(),
    }
}

pub(super) fn shape_for_infix_spec(spec: &InfixSpec) -> SignatureShape {
    let mut signature = format!("\\:{}", format_chain(&spec.chain));
    let mut arg_groups = Vec::new();
    add_expression_curly_groups(&mut arg_groups, &spec.head_args);
    add_expression_tail(&mut signature, &mut arg_groups, &spec.tail);
    signature.push_str(":/");
    SignatureShape {
        signature,
        arg_groups,
        fallback_shapes: Vec::new(),
    }
}

pub(super) fn shape_for_refined_command_expression(
    command: &RefinedCommandExpression,
) -> SignatureShape {
    let mut signature = "\\".to_string();
    if let Some(prefix) = &command.prefix_chain {
        signature.push_str(&format_chain(prefix));
        signature.push_str("::");
    }
    let mut arg_groups = Vec::new();
    for (index, part) in command.parts.iter().enumerate() {
        if index > 0 {
            signature.push_str("::");
        }
        signature.push_str(&format_chain(&part.chain));
        add_expression_tail(&mut signature, &mut arg_groups, &part.tail);
    }
    signature.push_str("::");
    signature.push_str(&format_refined_tail(&command.refined_tail));
    add_expression_curly_groups(&mut arg_groups, &command.head_args);
    add_expression_tail(&mut signature, &mut arg_groups, &command.tail);
    for args in &command.paren_args {
        arg_groups.push(ArgGroupShape {
            delimiter: ArgDelimiter::Paren,
            count: args.expressions.len(),
        });
    }
    let mut shape = SignatureShape {
        signature,
        arg_groups,
        fallback_shapes: Vec::new(),
    };
    shape.fallback_shapes = fallback_shapes_for_refined_command_expression(command);
    shape
}

pub(super) fn fallback_shapes_for_refined_command_expression(
    command: &RefinedCommandExpression,
) -> Vec<SignatureShape> {
    let mut shapes = vec![shape_for_refined_command_base(command)];
    if command.parts.len() > 1 {
        shapes.extend(
            command
                .parts
                .iter()
                .map(|part| shape_for_refined_command_part(command, part)),
        );
    }
    shapes
}

pub(super) fn shape_for_refined_command_base(command: &RefinedCommandExpression) -> SignatureShape {
    let mut signature = format!("\\{}", format_refined_tail(&command.refined_tail));
    let mut arg_groups = Vec::new();
    add_expression_curly_groups(&mut arg_groups, &command.head_args);
    add_expression_tail(&mut signature, &mut arg_groups, &command.tail);
    add_expression_paren_groups(&mut arg_groups, &command.paren_args);
    SignatureShape {
        signature,
        arg_groups,
        fallback_shapes: Vec::new(),
    }
}

pub(super) fn shape_for_refined_command_part(
    command: &RefinedCommandExpression,
    part: &RefinedExpressionPart,
) -> SignatureShape {
    let mut signature = "\\".to_string();
    if let Some(prefix) = &command.prefix_chain {
        signature.push_str(&format_chain(prefix));
        signature.push_str("::");
    }
    signature.push_str(&format_chain(&part.chain));
    let mut arg_groups = Vec::new();
    add_expression_tail(&mut signature, &mut arg_groups, &part.tail);
    signature.push_str("::");
    signature.push_str(&format_refined_tail(&command.refined_tail));
    add_expression_curly_groups(&mut arg_groups, &command.head_args);
    add_expression_tail(&mut signature, &mut arg_groups, &command.tail);
    add_expression_paren_groups(&mut arg_groups, &command.paren_args);
    SignatureShape {
        signature,
        arg_groups,
        fallback_shapes: Vec::new(),
    }
}

pub(super) fn add_heading_curly_groups(
    arg_groups: &mut Vec<ArgGroupShape>,
    groups: &[CurlyHeadingArgs],
) {
    for args in groups {
        arg_groups.push(ArgGroupShape {
            delimiter: ArgDelimiter::Curly,
            count: args.forms.len(),
        });
    }
}

pub(super) fn add_expression_curly_groups(
    arg_groups: &mut Vec<ArgGroupShape>,
    groups: &[CurlyExpressionArgs],
) {
    for args in groups {
        arg_groups.push(ArgGroupShape {
            delimiter: ArgDelimiter::Curly,
            count: args.expressions.len(),
        });
    }
}

pub(super) fn add_expression_paren_groups(
    arg_groups: &mut Vec<ArgGroupShape>,
    groups: &[ParenExpressionArgs],
) {
    for args in groups {
        arg_groups.push(ArgGroupShape {
            delimiter: ArgDelimiter::Paren,
            count: args.expressions.len(),
        });
    }
}

#[derive(Clone, Debug, Default)]
struct HeaderVariant {
    signature_suffix: String,
    type_key_suffix: String,
    arg_groups: Vec<ArgGroupShape>,
    parameters: Vec<String>,
    hidden_parameters: Vec<String>,
}

fn header_tail_variants(tail: &[CommandHeaderTailPart]) -> Vec<HeaderVariant> {
    let mut variants = vec![HeaderVariant::default()];

    for part in tail {
        let included = variants
            .iter()
            .map(|variant| {
                let mut next = variant.clone();
                append_header_tail_part_variant(&mut next, part);
                next
            })
            .collect::<Vec<_>>();

        if part.optional {
            let hidden_parameters = heading_group_parameters(&part.args);
            for variant in &mut variants {
                variant.hidden_parameters.extend(hidden_parameters.clone());
            }
            variants.extend(included);
        } else {
            variants = included;
        }
    }

    deduplicate_header_variants(variants)
}

fn append_header_tail_part_variant(variant: &mut HeaderVariant, part: &CommandHeaderTailPart) {
    variant.signature_suffix.push(':');
    variant
        .signature_suffix
        .push_str(&format_chain(&part.chain));
    variant.type_key_suffix.push(':');
    variant.type_key_suffix.push_str(&format_chain(&part.chain));
    append_heading_curly_key_groups(&mut variant.type_key_suffix, &part.args);
    variant.arg_groups.extend(heading_group_shapes(&part.args));
    variant
        .parameters
        .extend(heading_group_parameters(&part.args));
}

fn combine_header_variants(
    prefixes: Vec<HeaderVariant>,
    suffixes: &[HeaderVariant],
    render_signature_suffix: impl Fn(&HeaderVariant) -> String,
    render_type_key_suffix: impl Fn(&HeaderVariant) -> String,
) -> Vec<HeaderVariant> {
    let mut combined = Vec::new();
    for prefix in prefixes {
        for suffix in suffixes {
            let mut next = prefix.clone();
            next.signature_suffix
                .push_str(&render_signature_suffix(suffix));
            next.type_key_suffix
                .push_str(&render_type_key_suffix(suffix));
            next.arg_groups.extend(suffix.arg_groups.clone());
            next.parameters.extend(suffix.parameters.clone());
            next.hidden_parameters
                .extend(suffix.hidden_parameters.clone());
            combined.push(next);
        }
    }
    deduplicate_header_variants(combined)
}

fn deduplicate_header_variants(variants: Vec<HeaderVariant>) -> Vec<HeaderVariant> {
    let mut deduped = Vec::new();
    for variant in variants {
        if deduped.iter().any(|existing: &HeaderVariant| {
            existing.signature_suffix == variant.signature_suffix
                && existing.type_key_suffix == variant.type_key_suffix
                && existing.arg_groups == variant.arg_groups
                && existing.hidden_parameters == variant.hidden_parameters
        }) {
            continue;
        }
        deduped.push(variant);
    }
    deduped
}

fn heading_group_shapes(groups: &[CurlyHeadingArgs]) -> Vec<ArgGroupShape> {
    groups
        .iter()
        .map(|args| ArgGroupShape {
            delimiter: ArgDelimiter::Curly,
            count: args.forms.len(),
        })
        .collect()
}

fn paren_heading_group_shapes(groups: &[ParenHeadingArgs]) -> Vec<ArgGroupShape> {
    groups
        .iter()
        .map(|args| ArgGroupShape {
            delimiter: ArgDelimiter::Paren,
            count: args.forms.len(),
        })
        .collect()
}

fn heading_group_parameters(groups: &[CurlyHeadingArgs]) -> Vec<String> {
    groups
        .iter()
        .flat_map(|args| args.forms.iter())
        .filter_map(primary_form_name)
        .collect()
}

fn paren_heading_group_parameters(groups: &[ParenHeadingArgs]) -> Vec<String> {
    groups
        .iter()
        .flat_map(|args| args.forms.iter())
        .filter_map(primary_form_name)
        .collect()
}

fn infix_operand_parameters(operand: Option<&FormOrDeclaration>) -> Vec<String> {
    operand.and_then(primary_form_name).into_iter().collect()
}

fn heading_group_key_suffix(groups: &[CurlyHeadingArgs]) -> String {
    let mut key = String::new();
    append_heading_curly_key_groups(&mut key, groups);
    key
}

fn paren_heading_group_key_suffix(groups: &[ParenHeadingArgs]) -> String {
    let mut key = String::new();
    append_heading_paren_key_groups(&mut key, groups);
    key
}

fn append_heading_curly_key_groups(key: &mut String, groups: &[CurlyHeadingArgs]) {
    for args in groups {
        key.push('{');
        key.push_str(
            &args
                .forms
                .iter()
                .map(key_for_form_or_declaration)
                .collect::<Vec<_>>()
                .join(","),
        );
        key.push('}');
    }
}

fn append_heading_paren_key_groups(key: &mut String, groups: &[ParenHeadingArgs]) {
    for args in groups {
        key.push('(');
        key.push_str(
            &args
                .forms
                .iter()
                .map(key_for_form_or_declaration)
                .collect::<Vec<_>>()
                .join(","),
        );
        key.push(')');
    }
}

fn primary_form_name(form: &FormOrDeclaration) -> Option<String> {
    match &form.kind {
        FormOrDeclarationKind::Name(name) => Some(name.clone()),
        FormOrDeclarationKind::FunctionDeclaration { name, form } => {
            Some(name.as_ref().unwrap_or(&form.name).clone())
        }
        FormOrDeclarationKind::TupleDeclaration { name, .. }
        | FormOrDeclarationKind::SetDeclaration { name, .. } => name.clone(),
        FormOrDeclarationKind::InfixOperator { .. }
        | FormOrDeclarationKind::PrefixOperator { .. }
        | FormOrDeclarationKind::PostfixOperator { .. } => None,
    }
}

fn key_for_form_or_declaration(form: &FormOrDeclaration) -> String {
    match &form.kind {
        FormOrDeclarationKind::Name(name) => name.clone(),
        FormOrDeclarationKind::FunctionDeclaration { name, form } => {
            let name = name.as_ref().unwrap_or(&form.name);
            let args = form
                .magnetic_placeholder
                .iter()
                .map(|placeholder| placeholder.name.clone())
                .chain(
                    form.placeholders
                        .iter()
                        .map(|placeholder| placeholder.name.clone()),
                )
                .collect::<Vec<_>>()
                .join(",");
            if args.is_empty() {
                name.clone()
            } else {
                format!("{name}({args})")
            }
        }
        FormOrDeclarationKind::TupleDeclaration { name, form } => {
            let tuple = format!(
                "({})",
                form.elements
                    .iter()
                    .map(|element| match element {
                        TupleFormElement::Form(form) => key_for_form_or_declaration(form),
                        TupleFormElement::Operator(operator) => operator.text.clone(),
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            );
            name.as_ref()
                .map(|name| format!("{name}:={tuple}"))
                .unwrap_or(tuple)
        }
        FormOrDeclarationKind::SetDeclaration { name, form } => {
            let set = format!("{{{}}}", key_for_placeholder_form(&form.placeholder_form));
            name.as_ref()
                .map(|name| format!("{name}:={set}"))
                .unwrap_or(set)
        }
        FormOrDeclarationKind::InfixOperator {
            left,
            operator,
            right,
        } => format!("{}{}{}", left.name, operator.text, right.name),
        FormOrDeclarationKind::PrefixOperator {
            operator,
            placeholder,
        } => format!("{}{}", operator.text, placeholder.name),
        FormOrDeclarationKind::PostfixOperator {
            placeholder,
            operator,
        } => format!("{}{}", placeholder.name, operator.text),
    }
}

fn key_for_placeholder_form(form: &PlaceholderForm) -> String {
    match &form.kind {
        PlaceholderFormKind::Placeholder(placeholder) => placeholder.name.clone(),
        PlaceholderFormKind::Function {
            placeholder,
            arguments,
        } => format!(
            "{}({})",
            placeholder.name,
            arguments
                .iter()
                .map(|argument| argument.name.clone())
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

pub(super) fn add_header_tail(
    signature: &mut String,
    arg_groups: &mut Vec<ArgGroupShape>,
    tail: &[CommandHeaderTailPart],
) {
    for part in tail {
        signature.push(':');
        signature.push_str(&format_chain(&part.chain));
        add_heading_curly_groups(arg_groups, &part.args);
    }
}

pub(super) fn add_expression_tail(
    signature: &mut String,
    arg_groups: &mut Vec<ArgGroupShape>,
    tail: &[CommandExpressionTailPart],
) {
    for part in tail {
        signature.push(':');
        signature.push_str(&format_chain(&part.chain));
        add_expression_curly_groups(arg_groups, &part.args);
    }
}

pub(super) fn format_chain(chain: &Chain) -> String {
    chain
        .parts
        .iter()
        .map(format_chain_part)
        .collect::<Vec<_>>()
        .join(".")
}

pub(super) fn format_chain_part(part: &ChainPart) -> String {
    match part {
        ChainPart::Name(name) => name.clone(),
        ChainPart::Alias(name) => format!("${name}"),
        ChainPart::Operator(operator) => operator.clone(),
    }
}

pub(super) fn format_refined_tail(tail: &RefinedTail) -> String {
    match tail {
        RefinedTail::Chain(chain) => format_chain(chain),
        RefinedTail::Name { name, .. } => name.clone(),
    }
}

pub(super) fn format_arg_groups(groups: &[ArgGroupShape]) -> String {
    if groups.is_empty() {
        return "none".to_string();
    }

    groups
        .iter()
        .map(|group| match group.delimiter {
            ArgDelimiter::Curly => format!("{{{}}}", group.count),
            ArgDelimiter::Paren => format!("({})", group.count),
        })
        .collect::<Vec<_>>()
        .join("")
}

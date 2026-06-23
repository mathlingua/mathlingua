use super::*;

pub(super) fn shape_for_header(header: &CommandHeader) -> SignatureShape {
    match header {
        CommandHeader::Command(command) => shape_for_command_header_node(command),
        CommandHeader::Infix(command) => shape_for_infix_command_header(command),
        CommandHeader::Refined(command) => shape_for_refined_command_header(command),
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

pub(super) fn shape_for_infix_command_header(command: &InfixCommandHeader) -> SignatureShape {
    let mut signature = format!("\\:{}", format_chain(&command.chain));
    let mut arg_groups = Vec::new();
    add_heading_curly_groups(&mut arg_groups, &command.head_args);
    add_header_tail(&mut signature, &mut arg_groups, &command.tail);
    signature.push_str(":/");
    SignatureShape {
        signature,
        arg_groups,
        fallback_shapes: Vec::new(),
    }
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
    let mut signature = format!("\\:{}", format_chain(&command.chain));
    let mut arg_groups = Vec::new();
    add_expression_curly_groups(&mut arg_groups, &command.head_args);
    add_expression_tail(&mut signature, &mut arg_groups, &command.tail);
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
    shapes.extend(
        command
            .parts
            .iter()
            .map(|part| shape_for_refined_command_part(command, part)),
    );
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

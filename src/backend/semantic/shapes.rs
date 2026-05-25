use super::*;

/// Builds the canonical signature shape for any definition command header.
pub(super) fn shape_for_header(header: &CommandHeader) -> SignatureShape {
    match header {
        CommandHeader::Command(command) => shape_for_command_header_node(command),
        CommandHeader::Infix(command) => shape_for_infix_command_header(command),
        CommandHeader::Refined(command) => shape_for_refined_command_header(command),
    }
}

/// Builds a signature shape for a normal prefix command header.
///
/// The resulting signature preserves the command chain and tail labels while
/// replacing each argument group with only its delimiter and argument count.
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

/// Builds a signature shape for an infix command header.
///
/// Infix commands are wrapped in `\:...:/` so they cannot collide with ordinary
/// prefix command signatures.
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

/// Builds a signature shape for a refined command definition header.
///
/// Refined headers combine optional prefixes, one or more refinement parts, and
/// the refined tail into a single signature such as
/// `\(continuous)::function:on:to`.
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

/// Builds a signature shape for a normal command expression use.
///
/// This mirrors `shape_for_command_header_node` but counts expression arguments
/// instead of declaration/form arguments.
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

/// Builds a signature shape for an infix command expression use.
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

/// Builds a signature shape for a refined command expression use.
///
/// The primary shape represents the full composed command.  Additional fallback
/// shapes are attached so a combined use can be accepted when the base command
/// and each refinement piece are defined separately.
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

/// Produces fallback validation shapes for a refined command expression.
///
/// The first fallback is the refined base command; subsequent fallbacks represent
/// individual refinement pieces applied to that same base.
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

/// Builds the fallback shape for the base command of a refined expression.
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

/// Builds the fallback shape for one refinement part applied to a base command.
///
/// For a use such as `\(continuous)::function:on{A}:to{B}`, this constructs the
/// piece signature `\(continuous)::function:on:to` and attaches the argument
/// shape needed to validate that specific refinement definition.
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

/// Appends curly argument-group shapes from a command header.
///
/// Header groups contain forms/declarations, so their arity is counted from the
/// `forms` collection rather than expression values.
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

/// Appends curly argument-group shapes from a command expression.
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

/// Appends parenthesized argument-group shapes from a command expression.
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

/// Extends a header signature with tail labels and their argument shapes.
///
/// Tail labels contribute to the canonical signature as `:label` segments, while
/// their argument lists contribute only arity metadata.
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

/// Extends an expression signature with tail labels and their argument shapes.
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

/// Converts a parsed command/name chain into its canonical dotted text form.
pub(super) fn format_chain(chain: &Chain) -> String {
    chain
        .parts
        .iter()
        .map(format_chain_part)
        .collect::<Vec<_>>()
        .join(".")
}

/// Converts one chain part into the text used in canonical signatures.
pub(super) fn format_chain_part(part: &ChainPart) -> String {
    match part {
        ChainPart::Name(name) => name.clone(),
        ChainPart::Alias(name) => format!("${name}"),
        ChainPart::Operator(operator) => operator.clone(),
    }
}

/// Converts the base portion of a refined command into canonical signature text.
pub(super) fn format_refined_tail(tail: &RefinedTail) -> String {
    match tail {
        RefinedTail::Chain(chain) => format_chain(chain),
        RefinedTail::Name { name, .. } => name.clone(),
    }
}

/// Formats an argument-shape list for user-facing diagnostics.
///
/// Examples are `none`, `{2}`, `{1}(2)`, or `{1}:...` depending on the groups
/// present.  Only delimiter kind and count are displayed because semantic type
/// information is not available at this layer yet.
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

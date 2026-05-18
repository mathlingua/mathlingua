/// Builds the canonical signature for any command header.
fn command_header_signature(header: &crate::frontend::formulation::ast::CommandHeader) -> String {
    match header {
        crate::frontend::formulation::ast::CommandHeader::Command(command) => {
            let mut signature = format!("\\{}", format_chain(&command.chain));
            add_header_tail_signature(&mut signature, &command.tail);
            signature
        }
        crate::frontend::formulation::ast::CommandHeader::Infix(command) => {
            format!("\\:{}:/", format_chain(&command.chain))
        }
        crate::frontend::formulation::ast::CommandHeader::Refined(command) => {
            refined_command_header_signature(command)
        }
    }
}

/// Extracts ordered substitution parameter names from a command header.
fn command_header_parameters(
    header: &crate::frontend::formulation::ast::CommandHeader,
) -> Vec<String> {
    command_header_forms(header)
        .into_iter()
        .filter_map(primary_form_name)
        .collect()
}

/// Builds the canonical signature for a normal command expression.
fn command_expression_signature(command: &CommandExpression) -> String {
    let mut signature = format!("\\{}", format_chain(&command.chain));
    add_expression_tail_signature(&mut signature, &command.tail);
    signature
}

/// Builds the canonical signature for a refined command header.
fn refined_command_header_signature(command: &RefinedCommandHeader) -> String {
    let mut signature = "\\".to_string();
    if let Some(prefix) = &command.prefix_chain {
        signature.push_str(&format_chain(prefix));
        signature.push_str("::");
    }
    for (index, part) in command.parts.iter().enumerate() {
        if index > 0 {
            signature.push_str("::");
        }
        signature.push_str(&format_chain(&part.chain));
        add_header_tail_signature(&mut signature, &part.tail);
    }
    signature.push_str("::");
    signature.push_str(&refined_tail_signature(&command.refined_tail));
    add_header_tail_signature(&mut signature, &command.tail);
    signature
}

/// Builds the canonical signature for the base command of a refined expression.
fn refined_command_base_signature(command: &RefinedCommandExpression) -> String {
    let mut signature = format!("\\{}", refined_tail_signature(&command.refined_tail));
    add_expression_tail_signature(&mut signature, &command.tail);
    signature
}

/// Builds the canonical signature for one refinement part of a refined expression.
fn refined_command_part_signature(
    command: &RefinedCommandExpression,
    part: &RefinedExpressionPart,
) -> String {
    let mut signature = "\\".to_string();
    if let Some(prefix) = &command.prefix_chain {
        signature.push_str(&format_chain(prefix));
        signature.push_str("::");
    }
    signature.push_str(&format_chain(&part.chain));
    add_expression_tail_signature(&mut signature, &part.tail);
    signature.push_str("::");
    signature.push_str(&refined_tail_signature(&command.refined_tail));
    add_expression_tail_signature(&mut signature, &command.tail);
    signature
}

/// Appends header tail labels to a canonical signature string.
fn add_header_tail_signature(
    signature: &mut String,
    tail: &[crate::frontend::formulation::ast::CommandHeaderTailPart],
) {
    for part in tail {
        signature.push(':');
        signature.push_str(&format_chain(&part.chain));
    }
}

/// Appends expression tail labels to a canonical signature string.
fn add_expression_tail_signature(signature: &mut String, tail: &[CommandExpressionTailPart]) {
    for part in tail {
        signature.push(':');
        signature.push_str(&format_chain(&part.chain));
    }
}

/// Converts a refined-command tail into signature text.
fn refined_tail_signature(tail: &RefinedTail) -> String {
    match tail {
        RefinedTail::Chain(chain) => format_chain(chain),
        RefinedTail::Name { name, .. } => name.clone(),
    }
}

/// Converts a parsed chain into dotted signature text.
fn format_chain(chain: &Chain) -> String {
    chain
        .parts
        .iter()
        .map(|part| match part {
            ChainPart::Name(name) => name.clone(),
            ChainPart::Alias(name) => format!("${name}"),
            ChainPart::Operator(operator) => operator.clone(),
        })
        .collect::<Vec<_>>()
        .join(".")
}


use super::*;

pub(super) fn command_header_signature(header: &CommandHeader) -> String {
    match header {
        CommandHeader::Command(command) => {
            let mut signature = format!("\\{}", format_chain(&command.chain));
            add_header_tail_signature(&mut signature, &command.tail);
            signature
        }
        CommandHeader::Infix(command) => {
            format!("\\:{}:/", format_chain(&command.chain))
        }
        CommandHeader::Refined(command) => refined_command_header_signature(command),
    }
}

pub(super) fn command_header_parameters(header: &CommandHeader) -> Vec<String> {
    command_header_forms(header)
        .into_iter()
        .filter_map(primary_form_name)
        .collect()
}

pub(super) fn command_expression_signature(command: &CommandExpression) -> String {
    let mut signature = format!("\\{}", format_chain(&command.chain));
    add_expression_tail_signature(&mut signature, &command.tail);
    signature
}

pub(super) fn refined_command_header_signature(command: &RefinedCommandHeader) -> String {
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

pub(super) fn refined_command_base_signature(command: &RefinedCommandExpression) -> String {
    let mut signature = format!("\\{}", refined_tail_signature(&command.refined_tail));
    add_expression_tail_signature(&mut signature, &command.tail);
    signature
}

pub(super) fn refined_command_part_signature(
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

pub(super) fn add_header_tail_signature(signature: &mut String, tail: &[CommandHeaderTailPart]) {
    for part in tail {
        signature.push(':');
        signature.push_str(&format_chain(&part.chain));
    }
}

pub(super) fn add_expression_tail_signature(
    signature: &mut String,
    tail: &[CommandExpressionTailPart],
) {
    for part in tail {
        signature.push(':');
        signature.push_str(&format_chain(&part.chain));
    }
}

pub(super) fn refined_tail_signature(tail: &RefinedTail) -> String {
    match tail {
        RefinedTail::Chain(chain) => format_chain(chain),
        RefinedTail::Name { name, .. } => name.clone(),
    }
}

pub(super) fn format_chain(chain: &Chain) -> String {
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

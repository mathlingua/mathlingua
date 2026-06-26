use super::*;

pub(super) fn command_header_signature(header: &CommandHeader) -> String {
    match header {
        CommandHeader::Command(command) => {
            let mut signature = format!("\\{}", format_chain(&command.chain));
            add_header_tail_signature(&mut signature, &command.tail);
            signature
        }
        CommandHeader::Infix(command) => {
            format!("\\.{}./", format_chain(&command.chain))
        }
        CommandHeader::InfixSpec(spec) => {
            let mut signature = format!("\\:{}", format_chain(&spec.chain));
            add_header_tail_signature(&mut signature, &spec.tail);
            signature.push_str(":/");
            signature
        }
        CommandHeader::Refined(command) => refined_command_header_signature(command),
    }
}

#[derive(Clone, Debug)]
pub(super) struct CommandHeaderSignature {
    pub(super) signature: String,
    pub(super) parameters: Vec<String>,
}

pub(super) fn command_header_signatures(header: &CommandHeader) -> Vec<CommandHeaderSignature> {
    match header {
        CommandHeader::Command(command) => simple_command_header_signatures(command),
        CommandHeader::Infix(command) => infix_command_header_signatures(command),
        CommandHeader::InfixSpec(spec) => infix_spec_header_signatures(spec),
        CommandHeader::Refined(command) => refined_command_header_signatures(command),
    }
}

pub(super) fn simple_command_header_signatures(
    command: &CommandHeaderNode,
) -> Vec<CommandHeaderSignature> {
    let base_signature = format!("\\{}", format_chain(&command.chain));
    let base_parameters = heading_group_parameters(&command.head_args);
    let paren_parameters = paren_heading_group_parameters(&command.paren_args);

    header_tail_variants(&command.tail)
        .into_iter()
        .map(|variant| {
            let mut parameters = base_parameters.clone();
            parameters.extend(variant.parameters);
            parameters.extend(paren_parameters.clone());
            CommandHeaderSignature {
                signature: format!("{base_signature}{}", variant.signature_suffix),
                parameters,
            }
        })
        .collect()
}

pub(super) fn infix_command_header_signatures(
    command: &InfixCommandHeader,
) -> Vec<CommandHeaderSignature> {
    let base_signature = format!("\\.{}", format_chain(&command.chain));
    let mut base_parameters = infix_operand_parameters(command.left.as_ref());
    base_parameters.extend(heading_group_parameters(&command.head_args));
    let right_parameters = infix_operand_parameters(command.right.as_ref());

    header_tail_variants(&command.tail)
        .into_iter()
        .map(|variant| {
            let mut parameters = base_parameters.clone();
            parameters.extend(variant.parameters);
            parameters.extend(right_parameters.clone());
            CommandHeaderSignature {
                signature: format!("{base_signature}{}./", variant.signature_suffix),
                parameters,
            }
        })
        .collect()
}

pub(super) fn infix_spec_header_signatures(spec: &InfixSpecHeader) -> Vec<CommandHeaderSignature> {
    let base_signature = format!("\\:{}", format_chain(&spec.chain));
    let mut base_parameters = infix_operand_parameters(Some(&spec.left));
    base_parameters.extend(heading_group_parameters(&spec.head_args));
    let right_parameters = infix_operand_parameters(Some(&spec.right));

    header_tail_variants(&spec.tail)
        .into_iter()
        .map(|variant| {
            let mut parameters = base_parameters.clone();
            parameters.extend(variant.parameters);
            parameters.extend(right_parameters.clone());
            CommandHeaderSignature {
                signature: format!("{base_signature}{}:/", variant.signature_suffix),
                parameters,
            }
        })
        .collect()
}

pub(super) fn refined_command_header_signatures(
    command: &RefinedCommandHeader,
) -> Vec<CommandHeaderSignature> {
    let mut variants = vec![HeaderSignatureVariant::default()];
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
        variants = combine_header_signature_variants(variants, &part_tail_variants, |variant| {
            format!("{part_prefix}{}", variant.signature_suffix)
        });
    }

    let refined_tail = format!("::{}", refined_tail_signature(&command.refined_tail));
    let head_parameters = heading_group_parameters(&command.head_args);
    let tail_variants = header_tail_variants(&command.tail);
    let paren_parameters = paren_heading_group_parameters(&command.paren_args);

    variants
        .into_iter()
        .flat_map(|prefix_variant| {
            let head_parameters = head_parameters.clone();
            let paren_parameters = paren_parameters.clone();
            let refined_tail = refined_tail.clone();
            tail_variants.iter().map(move |tail_variant| {
                let mut signature = prefix_variant.signature_suffix.clone();
                signature.push_str(&refined_tail);
                signature.push_str(&tail_variant.signature_suffix);

                let mut parameters = prefix_variant.parameters.clone();
                parameters.extend(head_parameters.clone());
                parameters.extend(tail_variant.parameters.clone());
                parameters.extend(paren_parameters.clone());

                CommandHeaderSignature {
                    signature,
                    parameters,
                }
            })
        })
        .collect()
}

pub(super) fn command_expression_signature(command: &CommandExpression) -> String {
    let mut signature = format!("\\{}", format_chain(&command.chain));
    add_expression_tail_signature(&mut signature, &command.tail);
    signature
}

pub(super) fn infix_command_signature(command: &InfixCommand) -> String {
    let mut signature = format!("\\.{}", format_chain(&command.chain));
    add_expression_tail_signature(&mut signature, &command.tail);
    signature.push_str("./");
    signature
}

pub(super) fn infix_spec_signature(spec: &InfixSpec) -> String {
    let mut signature = format!("\\:{}", format_chain(&spec.chain));
    add_expression_tail_signature(&mut signature, &spec.tail);
    signature.push_str(":/");
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

#[derive(Clone, Debug, Default)]
struct HeaderSignatureVariant {
    signature_suffix: String,
    parameters: Vec<String>,
}

fn header_tail_variants(tail: &[CommandHeaderTailPart]) -> Vec<HeaderSignatureVariant> {
    let mut variants = vec![HeaderSignatureVariant::default()];

    for part in tail {
        let included = variants
            .iter()
            .map(|variant| {
                let mut next = variant.clone();
                next.signature_suffix.push(':');
                next.signature_suffix.push_str(&format_chain(&part.chain));
                next.parameters.extend(
                    part.args
                        .iter()
                        .flat_map(|args| args.forms.iter())
                        .filter_map(primary_form_name),
                );
                next
            })
            .collect::<Vec<_>>();

        if part.optional {
            variants.extend(included);
        } else {
            variants = included;
        }
    }

    deduplicate_header_signature_variants(variants)
}

fn combine_header_signature_variants(
    prefixes: Vec<HeaderSignatureVariant>,
    suffixes: &[HeaderSignatureVariant],
    render_suffix: impl Fn(&HeaderSignatureVariant) -> String,
) -> Vec<HeaderSignatureVariant> {
    let mut combined = Vec::new();
    for prefix in prefixes {
        for suffix in suffixes {
            let mut next = prefix.clone();
            next.signature_suffix.push_str(&render_suffix(suffix));
            next.parameters.extend(suffix.parameters.clone());
            combined.push(next);
        }
    }
    deduplicate_header_signature_variants(combined)
}

fn deduplicate_header_signature_variants(
    variants: Vec<HeaderSignatureVariant>,
) -> Vec<HeaderSignatureVariant> {
    let mut deduped = Vec::new();
    for variant in variants {
        if deduped.iter().any(|existing: &HeaderSignatureVariant| {
            existing.signature_suffix == variant.signature_suffix
        }) {
            continue;
        }
        deduped.push(variant);
    }
    deduped
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

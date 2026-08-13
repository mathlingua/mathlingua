use super::*;

pub(super) fn shape_for_header(header: &CommandHeader) -> SignatureShape {
    match header {
        CommandHeader::Command(command) => shape_for_command_header_node(command),
        CommandHeader::Infix(command) => shape_for_infix_command_header(command),
        CommandHeader::InfixSpec(spec) => shape_for_infix_spec_header(spec),
        CommandHeader::Refined(command) => shape_for_refined_command_header(command),
    }
}

/// Builds the overload key carried by mapping-parameter headers.
///
/// Ordinary headers return `Ok(None)` and continue to use their historical
/// command-only signature. Specialized headers are currently restricted to
/// ordinary commands; their syntax rules intentionally make the associated
/// mapping group and selector group unique.
pub(super) fn placeholder_signature_for_header(
    header: &CommandHeader,
) -> Result<Option<(String, PlaceholderSignaturePattern)>, String> {
    let CommandHeader::Command(command) = header else {
        if header_contains_mapping_parameters(header) {
            return Err(
                "mapping-parameter placeholders are only supported in ordinary command headers"
                    .to_owned(),
            );
        }
        return Ok(None);
    };
    placeholder_signature_for_command_header(command)
}

fn header_contains_mapping_parameters(header: &CommandHeader) -> bool {
    let groups = match header {
        CommandHeader::Command(command) => command_groups(command),
        CommandHeader::Infix(command) => command
            .head_args
            .iter()
            .chain(command.tail.iter().flat_map(|part| part.args.iter()))
            .collect(),
        CommandHeader::InfixSpec(command) => command
            .head_args
            .iter()
            .chain(command.tail.iter().flat_map(|part| part.args.iter()))
            .collect(),
        CommandHeader::Refined(command) => command
            .parts
            .iter()
            .flat_map(|part| part.tail.iter())
            .flat_map(|part| part.args.iter())
            .chain(command.head_args.iter())
            .chain(command.tail.iter().flat_map(|part| part.args.iter()))
            .collect(),
    };
    groups.iter().any(|group| {
        group
            .forms
            .iter()
            .any(|form| matches!(form.kind, FormOrDeclarationKind::MappingParameter { .. }))
    })
}

fn command_groups(command: &CommandHeaderNode) -> Vec<&CurlyHeadingArgs> {
    command
        .head_args
        .iter()
        .chain(command.tail.iter().flat_map(|part| part.args.iter()))
        .collect()
}

fn placeholder_signature_for_command_header(
    command: &CommandHeaderNode,
) -> Result<Option<(String, PlaceholderSignaturePattern)>, String> {
    let groups = command_groups(command);
    let selector_groups = groups
        .iter()
        .enumerate()
        .filter(|(_, group)| {
            group
                .forms
                .iter()
                .any(|form| matches!(form.kind, FormOrDeclarationKind::MappingParameter { .. }))
        })
        .collect::<Vec<_>>();
    if selector_groups.is_empty() {
        return Ok(None);
    }
    if selector_groups.len() != 1 {
        return Err(
            "mapping-parameter placeholders may occur in exactly one curly argument group"
                .to_owned(),
        );
    }
    let (selector_group_index, selector_group) = selector_groups[0];
    if !selector_group
        .forms
        .iter()
        .all(|form| matches!(form.kind, FormOrDeclarationKind::MappingParameter { .. }))
    {
        return Err(
            "a curly argument group containing mapping-parameter placeholders may contain only placeholders"
                .to_owned(),
        );
    }

    let mut owners = selector_group
        .forms
        .iter()
        .filter_map(|form| match &form.kind {
            FormOrDeclarationKind::MappingParameter { owner, .. } => Some(owner.as_str()),
            _ => None,
        });
    let owner = owners.next().expect("selector group is nonempty");
    if owners.any(|candidate| candidate != owner) {
        return Err(
            "all mapping-parameter placeholders in one group must refer to the same mapping"
                .to_owned(),
        );
    }

    let mapping_groups = groups
        .iter()
        .enumerate()
        .filter_map(|(index, group)| {
            let matches = group.forms.iter().filter_map(|form| match &form.kind {
                FormOrDeclarationKind::FunctionDeclaration { name, form }
                    if name.as_deref().unwrap_or(&form.name) == owner =>
                {
                    Some(form)
                }
                _ => None,
            });
            let matches = matches.collect::<Vec<_>>();
            (!matches.is_empty()).then_some((index, matches))
        })
        .collect::<Vec<_>>();
    if mapping_groups.len() != 1 || mapping_groups[0].1.len() != 1 {
        return Err(format!(
            "mapping-parameter placeholders for `{owner}` require exactly one other curly argument group containing `{owner}(...)`"
        ));
    }
    let (mapping_group_index, mappings) = &mapping_groups[0];
    if *mapping_group_index == selector_group_index {
        return Err(format!(
            "the `{owner}(...)` mapping and its parameter placeholders must be in separate curly argument groups"
        ));
    }
    let mapping = mappings[0];
    let mapping_arity =
        if mapping.variadic_parameter.is_some() || mapping.magnetic_placeholder.is_some() {
            MappingArity::Variadic
        } else {
            if mapping.placeholders.is_empty() {
                return Err(format!(
                    "the associated mapping `{owner}` must explicitly list its parameters"
                ));
            }
            MappingArity::Exact(mapping.placeholders.len())
        };

    let mut selector_patterns = Vec::new();
    for form in &selector_group.forms {
        let FormOrDeclarationKind::MappingParameter { selector, .. } = &form.kind else {
            unreachable!()
        };
        let pattern = match selector {
            MappingParameterSelector::Exact { name, .. } => {
                if let Some(parameter) = &mapping.variadic_parameter {
                    if name != &parameter.name {
                        return Err(format!(
                            "`{owner}.{name}_` is not a parameter of variadic mapping `{owner}`"
                        ));
                    }
                    MappingSelectorPattern::Variadic
                } else {
                    let Some(index) = mapping
                        .placeholders
                        .iter()
                        .position(|parameter| parameter.name == *name)
                    else {
                        return Err(format!("`{owner}.{name}_` is not a parameter of `{owner}`"));
                    };
                    MappingSelectorPattern::Exact(index + 1)
                }
            }
            MappingParameterSelector::Arbitrary { name, .. } => {
                if mapping
                    .placeholders
                    .iter()
                    .any(|parameter| parameter.name == *name)
                    || mapping
                        .variadic_parameter
                        .as_ref()
                        .is_some_and(|parameter| parameter.name == *name)
                {
                    return Err(format!(
                        "`{owner}.{name}?_` uses the name of an existing mapping parameter; omit `?` for an exact parameter or choose a fresh name"
                    ));
                }
                MappingSelectorPattern::Arbitrary
            }
            MappingParameterSelector::Variadic {
                name, outer_index, ..
            } => {
                let Some(parameter) = &mapping.variadic_parameter else {
                    return Err(format!(
                        "variadic selector `{owner}.{name}_[...]` requires a ranged variadic mapping parameter"
                    ));
                };
                if name != &parameter.name || outer_index != &parameter.index {
                    return Err(format!(
                        "variadic selector `{owner}.{name}_[{outer_index}_[...]]` does not match mapping parameter `{}_[{}_:= {}...{}]`",
                        parameter.name, parameter.index, parameter.start, parameter.length
                    ));
                }
                MappingSelectorPattern::Variadic
            }
        };
        selector_patterns.push(pattern);
    }

    let mapping_text = match mapping_arity {
        MappingArity::Exact(count) => format!("_({count})"),
        MappingArity::Variadic => "_(*)".to_owned(),
    };
    let selector_text = selector_patterns
        .iter()
        .map(|selector| match selector {
            MappingSelectorPattern::Exact(index) => format!("#{index}"),
            MappingSelectorPattern::Arbitrary => "#?".to_owned(),
            MappingSelectorPattern::Variadic => "#*".to_owned(),
        })
        .collect::<Vec<_>>()
        .join(", ");

    let mut group_index = 0;
    let mut signature = format!("\\{}", format_chain(&command.chain));
    for _ in &command.head_args {
        if group_index == *mapping_group_index {
            signature.push_str(&format!("{{{mapping_text}}}"));
        } else if group_index == selector_group_index {
            signature.push_str(&format!("{{{selector_text}}}"));
        }
        group_index += 1;
    }
    for part in &command.tail {
        signature.push(':');
        signature.push_str(&format_chain(&part.chain));
        for _ in &part.args {
            if group_index == *mapping_group_index {
                signature.push_str(&format!("{{{mapping_text}}}"));
            } else if group_index == selector_group_index {
                signature.push_str(&format!("{{{selector_text}}}"));
            }
            group_index += 1;
        }
    }

    let mut general_signature = format!("\\{}", format_chain(&command.chain));
    add_header_tail(&mut general_signature, &mut Vec::new(), &command.tail);
    Ok(Some((
        signature,
        PlaceholderSignaturePattern {
            general_signature,
            mapping_arity,
            selectors: selector_patterns,
        },
    )))
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
            count: ArgCount::Exact(args.forms.len()),
        });
    }
    let mut shape = SignatureShape {
        signature,
        arg_groups,
        fallback_shapes: Vec::new(),
    };
    if let Ok(Some((specialized, pattern))) = placeholder_signature_for_command_header(command) {
        shape.signature = specialized;
        shape.fallback_shapes.push(SignatureShape {
            signature: pattern.general_signature,
            arg_groups: shape.arg_groups.clone(),
            fallback_shapes: Vec::new(),
        });
    }
    shape
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

    let mut shapes = header_tail_variants(&command.tail)
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
        .collect::<Vec<_>>();
    if shapes.len() == 1
        && let Ok(Some((signature, pattern))) = placeholder_signature_for_command_header(command)
    {
        let arg_groups = shapes[0].shape.arg_groups.clone();
        shapes[0].shape.signature = signature;
        shapes[0].shape.fallback_shapes.push(SignatureShape {
            signature: pattern.general_signature,
            arg_groups,
            fallback_shapes: Vec::new(),
        });
    }
    shapes
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
    if let Some(command) = refined_command_header_for_infix_spec(spec) {
        return wrap_refined_infix_spec_shape(shape_for_refined_command_header(&command));
    }

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
    if let Some(command) = refined_command_header_for_infix_spec(spec) {
        let left = key_for_form_or_declaration(&spec.left);
        let right = key_for_form_or_declaration(&spec.right);
        return shapes_for_refined_command_header(&command)
            .into_iter()
            .map(|mut header| {
                header.shape = wrap_refined_infix_spec_shape(header.shape);
                header.type_key = wrap_refined_infix_spec_key(&header.type_key);
                header.parameters.insert(0, left.clone());
                header.parameters.push(right.clone());
                header
            })
            .collect();
    }

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

fn refined_command_header_for_infix_spec(spec: &InfixSpecHeader) -> Option<RefinedCommandHeader> {
    let refinement = spec.refinement.as_ref()?;
    Some(RefinedCommandHeader {
        span: spec.span,
        prefix_chain: refinement.prefix_chain.clone(),
        parts: refinement.parts.clone(),
        refined_tail: RefinedTail::Chain(spec.chain.clone()),
        head_args: spec.head_args.clone(),
        tail: spec.tail.clone(),
        paren_args: Vec::new(),
    })
}

fn wrap_refined_infix_spec_shape(mut shape: SignatureShape) -> SignatureShape {
    shape.signature = wrap_refined_infix_spec_key(&shape.signature);
    shape.fallback_shapes = shape
        .fallback_shapes
        .into_iter()
        .map(wrap_refined_infix_spec_shape)
        .collect();
    shape
}

fn wrap_refined_infix_spec_key(key: &str) -> String {
    format!("\\:{}:/", key.strip_prefix('\\').unwrap_or(key))
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
            count: ArgCount::Exact(args.forms.len()),
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
            count: ArgCount::Exact(args.expressions.len()),
        });
    }
    let mut shape = SignatureShape {
        signature,
        arg_groups,
        fallback_shapes: Vec::new(),
    };
    if let Some(invocation) = placeholder_invocation_for_command_expression(command) {
        shape.signature = invocation.signature;
        shape.fallback_shapes.push(SignatureShape {
            signature: invocation.general_signature,
            arg_groups: shape.arg_groups.clone(),
            fallback_shapes: Vec::new(),
        });
    }
    shape
}

pub(super) fn placeholder_invocation_for_command_expression(
    command: &CommandExpression,
) -> Option<PlaceholderInvocation> {
    let groups = command
        .head_args
        .iter()
        .chain(command.tail.iter().flat_map(|part| part.args.iter()))
        .collect::<Vec<_>>();
    let mapping_matches = groups
        .iter()
        .enumerate()
        .filter_map(|(index, group)| {
            let [expression] = group.expressions.as_slice() else {
                return None;
            };
            let ExpressionKind::Mapping { lhs, .. } = &expression.kind else {
                return None;
            };
            mapping_parameter_names(lhs).map(|names| (index, names))
        })
        .collect::<Vec<_>>();
    let [(mapping_group, mapping_parameters)] = mapping_matches.as_slice() else {
        return None;
    };
    if mapping_parameters.is_empty() {
        return None;
    }

    let selector_matches = groups
        .iter()
        .enumerate()
        .filter_map(|(index, group)| {
            if index == *mapping_group || group.expressions.is_empty() {
                return None;
            }
            group
                .expressions
                .iter()
                .map(expression_parameter_name)
                .collect::<Option<Vec<_>>>()
                .filter(|names| {
                    names
                        .iter()
                        .all(|name| mapping_parameters.iter().any(|parameter| parameter == name))
                })
                .map(|names| (index, names))
        })
        .collect::<Vec<_>>();
    let [(selector_group, selected)] = selector_matches.as_slice() else {
        return None;
    };
    let selected_positions = selected
        .iter()
        .map(|name| {
            mapping_parameters
                .iter()
                .position(|parameter| parameter == name)
                .map(|index| index + 1)
        })
        .collect::<Option<Vec<_>>>()?;

    let mut general_signature = format!("\\{}", format_chain(&command.chain));
    add_expression_tail(&mut general_signature, &mut Vec::new(), &command.tail);
    let mapping_text = format!("_({})", mapping_parameters.len());
    let selector_text = selected_positions
        .iter()
        .map(|position| format!("#{position}"))
        .collect::<Vec<_>>()
        .join(", ");
    let mut group_index = 0;
    let mut signature = format!("\\{}", format_chain(&command.chain));
    for _ in &command.head_args {
        if group_index == *mapping_group {
            signature.push_str(&format!("{{{mapping_text}}}"));
        } else if group_index == *selector_group {
            signature.push_str(&format!("{{{selector_text}}}"));
        }
        group_index += 1;
    }
    for part in &command.tail {
        signature.push(':');
        signature.push_str(&format_chain(&part.chain));
        for _ in &part.args {
            if group_index == *mapping_group {
                signature.push_str(&format!("{{{mapping_text}}}"));
            } else if group_index == *selector_group {
                signature.push_str(&format!("{{{selector_text}}}"));
            }
            group_index += 1;
        }
    }
    Some(PlaceholderInvocation {
        signature,
        general_signature,
        mapping_arity: mapping_parameters.len(),
        selected_positions,
    })
}

fn mapping_parameter_names(expression: &Expression) -> Option<Vec<String>> {
    let mut names = Vec::new();
    if collect_mapping_parameter_names(expression, &mut names) {
        Some(names)
    } else {
        None
    }
}

fn collect_mapping_parameter_names(expression: &Expression, names: &mut Vec<String>) -> bool {
    match &expression.kind {
        ExpressionKind::Name(name) | ExpressionKind::InferredName(name) => {
            names.push(name.clone());
            true
        }
        ExpressionKind::Tuple(elements) => elements.iter().all(|element| match element {
            TupleExpressionElement::Expression(expression) => {
                collect_mapping_parameter_names(expression, names)
            }
            TupleExpressionElement::Operator(_) => false,
        }),
        ExpressionKind::Grouped { expression, .. }
        | ExpressionKind::Labeled { expression, .. }
        | ExpressionKind::IsType {
            subject: expression,
            ..
        }
        | ExpressionKind::IsBuiltinPredicate {
            subject: expression,
            ..
        }
        | ExpressionKind::IsPredicate {
            subject: expression,
            ..
        } => collect_mapping_parameter_names(expression, names),
        ExpressionKind::SpecStatement(statement) | ExpressionKind::SpecPredicate(statement) => {
            collect_mapping_parameter_names(&statement.subject, names)
        }
        ExpressionKind::SpecStatementExpr { subject, .. } => {
            collect_mapping_parameter_names(subject, names)
        }
        _ => false,
    }
}

fn expression_parameter_name(expression: &Expression) -> Option<String> {
    match &expression.kind {
        ExpressionKind::Name(name) | ExpressionKind::InferredName(name) => Some(name.clone()),
        ExpressionKind::Grouped { expression, .. } => expression_parameter_name(expression),
        _ => None,
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
    if let Some(command) = refined_command_expression_for_infix_spec(spec) {
        return wrap_refined_infix_spec_shape(shape_for_refined_command_expression(&command));
    }

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

fn refined_command_expression_for_infix_spec(spec: &InfixSpec) -> Option<RefinedCommandExpression> {
    let refinement = spec.refinement.as_ref()?;
    Some(RefinedCommandExpression {
        span: spec.span,
        prefix_chain: refinement.prefix_chain.clone(),
        parts: refinement.parts.clone(),
        refined_tail: RefinedTail::Chain(spec.chain.clone()),
        head_args: spec.head_args.clone(),
        tail: spec.tail.clone(),
        paren_args: Vec::new(),
    })
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
            count: ArgCount::Exact(args.expressions.len()),
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
        arg_groups.push(heading_arg_group_shape(args));
    }
}

pub(super) fn add_expression_curly_groups(
    arg_groups: &mut Vec<ArgGroupShape>,
    groups: &[CurlyExpressionArgs],
) {
    for args in groups {
        arg_groups.push(ArgGroupShape {
            delimiter: ArgDelimiter::Curly,
            count: args
                .rows
                .clone()
                .map_or(ArgCount::Exact(args.expressions.len()), |row_lengths| {
                    ArgCount::Exact2D { row_lengths }
                }),
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
            count: ArgCount::Exact(args.expressions.len()),
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
    groups.iter().map(heading_arg_group_shape).collect()
}

fn heading_arg_group_shape(args: &CurlyHeadingArgs) -> ArgGroupShape {
    ArgGroupShape {
        delimiter: ArgDelimiter::Curly,
        count: match &args.variadic {
            Some(variadic) if variadic.dimensions.is_some() => {
                let dimensions = variadic.dimensions.as_ref().expect("checked above");
                ArgCount::Variadic2D {
                    row_length: dimensions.row_length.clone(),
                    column_length: dimensions.column_length.clone(),
                }
            }
            Some(variadic) => ArgCount::Variadic {
                length: variadic.length.clone(),
            },
            None => match args.forms.as_slice() {
                [
                    FormOrDeclaration {
                        kind:
                            FormOrDeclarationKind::MappingParameter {
                                selector: MappingParameterSelector::Variadic { length, .. },
                                ..
                            },
                        ..
                    },
                ] => ArgCount::Variadic {
                    length: Some(length.clone()),
                },
                _ => ArgCount::Exact(args.forms.len()),
            },
        },
    }
}

fn paren_heading_group_shapes(groups: &[ParenHeadingArgs]) -> Vec<ArgGroupShape> {
    groups
        .iter()
        .map(|args| ArgGroupShape {
            delimiter: ArgDelimiter::Paren,
            count: ArgCount::Exact(args.forms.len()),
        })
        .collect()
}

fn heading_group_parameters(groups: &[CurlyHeadingArgs]) -> Vec<String> {
    groups
        .iter()
        .flat_map(|args| {
            args.variadic
                .iter()
                .map(|variadic| variadic.name.clone())
                .chain(args.forms.iter().filter_map(primary_form_name))
        })
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
        if let Some(variadic) = &args.variadic {
            key.push_str(&variadic.name);
            if let Some(dimensions) = &variadic.dimensions {
                key.push_str(&format!(
                    "[({},{}):=({},{})...",
                    dimensions.row_index,
                    dimensions.column_index,
                    dimensions.row_start,
                    dimensions.column_start
                ));
                if let (Some(rows), Some(columns)) =
                    (&dimensions.row_length, &dimensions.column_length)
                {
                    key.push_str(&format!("({rows},{columns})"));
                }
                key.push(']');
            } else {
                key.push_str("...");
                if let Some(length) = &variadic.length {
                    key.push_str(length);
                }
            }
        } else {
            key.push_str(
                &args
                    .forms
                    .iter()
                    .map(key_for_form_or_declaration)
                    .collect::<Vec<_>>()
                    .join(","),
            );
        }
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
        FormOrDeclarationKind::MappingParameter { selector, .. } => {
            Some(selector.name().to_owned())
        }
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
        FormOrDeclarationKind::MappingParameter { owner, selector } => {
            format!("{owner}.{}", selector.name())
        }
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
                .chain(
                    form.variadic_parameter
                        .iter()
                        .map(|parameter| parameter.name.clone()),
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
            let set = format!("{{{}}}", key_for_set_target(&form.target));
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

fn key_for_set_target(target: &SetTarget) -> String {
    match &target.kind {
        SetTargetKind::Name(name) => name.clone(),
        SetTargetKind::PlaceholderForm(form) => key_for_placeholder_form(form),
        SetTargetKind::Expression { .. } => {
            unreachable!("expression targets only occur in collection literals")
        }
        SetTargetKind::Alias { name, target } => format!("{name}:={}", key_for_set_target(target)),
        SetTargetKind::Introduction { name, target } => {
            format!("{name}::={}", key_for_set_target(target))
        }
        SetTargetKind::Function { name, arguments } => format!(
            "{}({})",
            name,
            arguments
                .iter()
                .map(key_for_set_target)
                .collect::<Vec<_>>()
                .join(",")
        ),
        SetTargetKind::Tuple(elements) => format!(
            "({})",
            elements
                .iter()
                .map(|element| match element {
                    SetTargetElement::Target(target) => key_for_set_target(target),
                    SetTargetElement::Operator(operator) => operator.text.clone(),
                })
                .collect::<Vec<_>>()
                .join(",")
        ),
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
            ArgDelimiter::Curly => format!("{{{}}}", format_arg_count(&group.count)),
            ArgDelimiter::Paren => format!("({})", format_arg_count(&group.count)),
        })
        .collect::<Vec<_>>()
        .join("")
}

fn format_arg_count(count: &ArgCount) -> String {
    match count {
        ArgCount::Exact(count) => count.to_string(),
        ArgCount::Exact2D { row_lengths } => row_lengths
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(";"),
        ArgCount::Variadic {
            length: Some(length),
        } => format!("1+:{length}"),
        ArgCount::Variadic { length: None } => "1+".to_string(),
        ArgCount::Variadic2D {
            row_length,
            column_length,
        } => format!(
            "2D:{}x{}",
            row_length.as_deref().unwrap_or("1+"),
            column_length.as_deref().unwrap_or("1+")
        ),
    }
}

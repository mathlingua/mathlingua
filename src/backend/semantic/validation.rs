use super::*;

pub(super) fn validate_document_references(
    file: &ParsedSourceFile,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let mut locator = SourceLocator::new(&file.source);
    for item in &file.document.items {
        walk_top_level_item(item, &mut |shape| {
            validate_reference_shape(
                file.path.as_path(),
                locator.locate_reference(shape),
                shape,
                registry,
                event_log,
            );
        });
    }
}

pub(super) fn validate_reference_shape(
    path: &Path,
    position: Option<SourcePosition>,
    shape: &SignatureShape,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let resolved = match resolve_definition_signature(shape, registry) {
        Ok(resolved) => resolved,
        Err(message) => {
            emit_error(event_log, path, position, message);
            return;
        }
    };
    let Some(signature) = resolved else {
        emit_error(
            event_log,
            path,
            position,
            format!("Undefined command signature `{}`", shape.signature),
        );
        return;
    };
    let definition = registry
        .definitions
        .get(signature)
        .expect("resolved definition signature exists");

    if !argument_groups_match(&definition.shape.arg_groups, &shape.arg_groups) {
        emit_error(
            event_log,
            path,
            position,
            format!(
                "Command signature `{}` expects argument shape `{}` but found `{}`",
                shape.signature,
                format_arg_groups(&definition.shape.arg_groups),
                format_arg_groups(&shape.arg_groups)
            ),
        );
    }
}

pub(super) fn resolve_definition_signature<'a>(
    shape: &SignatureShape,
    registry: &'a SignatureRegistry,
) -> Result<Option<&'a str>, String> {
    if let Some((signature, _)) = registry.definitions.get_key_value(&shape.signature) {
        return Ok(Some(signature.as_str()));
    }
    let Some(invocation) = parse_placeholder_invocation_signature(shape) else {
        return Ok(shape
            .fallback_shapes
            .iter()
            .find_map(|fallback| registry.definitions.get_key_value(&fallback.signature))
            .map(|(signature, _)| signature.as_str()));
    };
    let Some(candidates) = registry
        .placeholder_definitions
        .get(&invocation.general_signature)
    else {
        return Ok(registry
            .definitions
            .get_key_value(&invocation.general_signature)
            .map(|(signature, _)| signature.as_str()));
    };

    let mut matches = candidates
        .iter()
        .filter_map(|signature| {
            let entry = registry.definitions.get(signature)?;
            let pattern = entry.placeholder_pattern.as_ref()?;
            placeholder_pattern_rank(pattern, &invocation).map(|rank| (rank, signature.as_str()))
        })
        .collect::<Vec<_>>();
    matches.sort_by(|left, right| right.0.cmp(&left.0));
    let Some((best_rank, best_signature)) = matches.first().copied() else {
        return Ok(None);
    };
    if matches.iter().skip(1).any(|(rank, _)| *rank == best_rank) {
        return Err(format!(
            "Invocation `{}` could not be resolved unambiguously among mapping-parameter signatures for `{}`",
            shape.signature, invocation.general_signature
        ));
    }
    Ok(Some(best_signature))
}

fn placeholder_pattern_rank(
    pattern: &PlaceholderSignaturePattern,
    invocation: &PlaceholderInvocation,
) -> Option<(u8, u8)> {
    let arity_rank = match pattern.mapping_arity {
        MappingArity::Exact(count) if count == invocation.mapping_arity => 2,
        MappingArity::Exact(_) => return None,
        MappingArity::Variadic => 1,
    };
    let selector_rank = if pattern.selectors.len() == 1
        && pattern.selectors[0] == MappingSelectorPattern::Variadic
    {
        1
    } else {
        if pattern.selectors.len() != invocation.selected_positions.len() {
            return None;
        }
        let mut rank = 3;
        for (expected, actual) in pattern.selectors.iter().zip(&invocation.selected_positions) {
            match expected {
                MappingSelectorPattern::Exact(position) if position == actual => {}
                MappingSelectorPattern::Exact(_) => return None,
                MappingSelectorPattern::Arbitrary => rank = rank.min(2),
                MappingSelectorPattern::Variadic => return None,
            }
        }
        rank
    };
    Some((arity_rank, selector_rank))
}

fn parse_placeholder_invocation_signature(shape: &SignatureShape) -> Option<PlaceholderInvocation> {
    let mapping_start = shape.signature.find("{_(")? + 3;
    let mapping_end = shape.signature[mapping_start..].find(")}")? + mapping_start;
    let mapping_arity = shape.signature[mapping_start..mapping_end]
        .parse::<usize>()
        .ok()?;
    let selected_positions = shape
        .signature
        .match_indices('#')
        .map(|(index, _)| {
            let digits = shape.signature[index + 1..]
                .chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>();
            digits.parse::<usize>().ok()
        })
        .collect::<Option<Vec<_>>>()?;
    let general_signature = shape.fallback_shapes.first()?.signature.clone();
    Some(PlaceholderInvocation {
        signature: shape.signature.clone(),
        general_signature,
        mapping_arity,
        selected_positions,
    })
}

pub(super) fn argument_groups_match(expected: &[ArgGroupShape], actual: &[ArgGroupShape]) -> bool {
    if actual.len() > expected.len() {
        return false;
    }

    let mut lengths = HashMap::<String, usize>::new();
    for (expected, actual) in expected.iter().zip(actual) {
        if expected.delimiter != actual.delimiter {
            return false;
        }
        let actual_count = match actual.count {
            ArgCount::Exact(count) => count,
            ArgCount::Variadic { .. } => {
                if expected.count != actual.count {
                    return false;
                }
                continue;
            }
        };
        match &expected.count {
            ArgCount::Exact(expected_count) if *expected_count != actual_count => return false,
            ArgCount::Exact(_) => {}
            ArgCount::Variadic { .. } if actual_count == 0 => return false,
            ArgCount::Variadic {
                length: Some(length),
            } => match lengths.insert(length.clone(), actual_count) {
                Some(previous) if previous != actual_count => return false,
                _ => {}
            },
            ArgCount::Variadic { length: None } => {}
        }
    }

    expected[actual.len()..]
        .iter()
        .all(|group| group.delimiter == ArgDelimiter::Paren)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry(headers: &[&str]) -> SignatureRegistry {
        let mut registry = SignatureRegistry::default();
        for source in headers {
            let header = crate::frontend::formulation::parse_command_header(source)
                .expect("valid placeholder header");
            let (signature, pattern) = placeholder_signature_for_header(&header)
                .expect("valid placeholder rules")
                .expect("specialized placeholder signature");
            let shape = shape_for_header(&header);
            assert_eq!(shape.signature, signature);
            registry
                .placeholder_definitions
                .entry(pattern.general_signature.clone())
                .or_default()
                .push(signature.clone());
            registry.definitions.insert(
                signature,
                DefinitionEntry {
                    kind: DefinitionKind::Theorem,
                    shape,
                    path: PathBuf::from("test.mlg"),
                    position: None,
                    placeholder_pattern: Some(pattern),
                },
            );
        }
        registry
    }

    fn expression_shape(source: &str) -> SignatureShape {
        let expression = crate::frontend::formulation::parse_expression(source)
            .expect("valid placeholder invocation");
        let ExpressionKind::Command(command) = expression.kind else {
            panic!("expected command expression");
        };
        shape_for_command_expression(&command)
    }

    #[test]
    fn computes_mapping_parameter_signatures() {
        let cases = [
            (r"\integral{f(x_, y_)}:d{f.x_}", r"\integral{_(2)}:d{#1}"),
            (r"\integral{f(x_, y_)}:d{f.y_}", r"\integral{_(2)}:d{#2}"),
            (
                r"\integral{f(x_, y_)}:d{f.u?_, f.v?_}",
                r"\integral{_(2)}:d{#?, #?}",
            ),
            (
                r"\integral{f(x_[i_:=1...n])}:d{f.x1?_, f.x2?_}",
                r"\integral{_(*)}:d{#?, #?}",
            ),
            (
                r"\integral{f(x_[i_:=1...n])}:d{f.x_[i_[j_:=1...m]]}",
                r"\integral{_(*)}:d{#*}",
            ),
        ];
        for (source, expected) in cases {
            let header = crate::frontend::formulation::parse_command_header(source)
                .unwrap_or_else(|error| panic!("valid header `{source}`: {error}"));
            let (signature, pattern) = placeholder_signature_for_header(&header)
                .expect("valid placeholder rules")
                .expect("placeholder signature");
            assert_eq!(signature, expected);
            assert_eq!(pattern.general_signature, r"\integral:d");
        }
    }

    #[test]
    fn resolves_exact_arbitrary_and_variadic_mapping_parameter_overloads() {
        let registry = registry(&[
            r"\integral{f(x_, y_)}:d{f.x_}",
            r"\integral{f(x_, y_)}:d{f.y_}",
            r"\integral{f(x_[i_:=1...n])}:d{f.x1?_, f.x2?_}",
            r"\integral{f(x_[i_:=1...n])}:d{f.x_[i_[j_:=1...m]]}",
        ]);
        let first = expression_shape(r"\integral[x_, y_ is \real]{x_^2+y_^2}:d{x_}");
        let second = expression_shape(r"\integral[x_, y_ is \real]{x_^2+y_^2}:d{y_}");
        let pair = expression_shape(r"\integral[x_, y_, z_ is \real]{x_^2+y_^2+z_^2}:d{x_, y_}");
        let all = expression_shape(r"\integral[x_, y_, z_ is \real]{x_^2+y_^2+z_^2}:d{x_, y_, z_}");
        assert_eq!(
            resolve_definition_signature(&first, &registry).unwrap(),
            Some(r"\integral{_(2)}:d{#1}")
        );
        assert_eq!(
            resolve_definition_signature(&second, &registry).unwrap(),
            Some(r"\integral{_(2)}:d{#2}")
        );
        assert_eq!(
            resolve_definition_signature(&pair, &registry).unwrap(),
            Some(r"\integral{_(*)}:d{#?, #?}")
        );
        assert_eq!(
            resolve_definition_signature(&all, &registry).unwrap(),
            Some(r"\integral{_(*)}:d{#*}")
        );
    }

    #[test]
    fn rejects_invalid_mapping_parameter_header_layouts() {
        let cases = [
            (
                r"\baz{f(x_, y_)}:foo{f.x_}:bar{f.y_}",
                "exactly one curly argument group",
            ),
            (
                r"\baz{f(x_, y_)}:foo{f.x_, y}",
                "may contain only placeholders",
            ),
            (
                r"\baz{f}:foo{f.x_}",
                "require exactly one other curly argument group",
            ),
            (r"\baz{f(x_, y_)}:foo{f.z_}", "is not a parameter"),
            (
                r"\baz{f(x_, y_)}:foo{f.x?_}",
                "uses the name of an existing mapping parameter",
            ),
        ];
        for (source, expected) in cases {
            let header = crate::frontend::formulation::parse_command_header(source)
                .unwrap_or_else(|error| panic!("valid syntax `{source}`: {error}"));
            let error = placeholder_signature_for_header(&header)
                .expect_err("invalid mapping-parameter header");
            assert!(
                error.contains(expected),
                "expected `{error}` to contain `{expected}`"
            );
        }
    }
}

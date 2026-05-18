use super::*;

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
    let function_call = parse_expression("f(x, y + z)").expect("expected function call expression");
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
    let named_prefix =
        parse_form_or_declaration("neg| x_").expect("expected named prefix operator form");
    let named_postfix =
        parse_form_or_declaration("x_ |prime").expect("expected named postfix operator form");
    let infix = parse_form_or_declaration("x_ |plus| y_").expect("expected infix operator form");

    assert!(matches!(
        prefix.kind,
        FormOrDeclarationKind::PrefixOperator { ref operator, .. } if operator.text == "-"
    ));
    assert!(matches!(
        postfix.kind,
        FormOrDeclarationKind::PostfixOperator { ref operator, .. } if operator.text == "!"
    ));
    assert!(matches!(
        named_prefix.kind,
        FormOrDeclarationKind::PrefixOperator { ref operator, .. } if operator.text == "neg"
    ));
    assert!(matches!(
        named_postfix.kind,
        FormOrDeclarationKind::PostfixOperator { ref operator, .. } if operator.text == "prime"
    ));
    assert!(matches!(
        infix.kind,
        FormOrDeclarationKind::InfixOperator { ref operator, .. } if operator.text == "plus"
    ));
}

#[test]
fn parses_left_right_and_both_named_operator_expressions() {
    let left = parse_expression("x :|before| y").expect("expected left-colon operator");
    let right = parse_expression("x |after|: y").expect("expected right-colon operator");
    let both = parse_expression("x :|around|: y").expect("expected both-colon operator");

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
    assert!(matches!(
        both.kind,
        ExpressionKind::Binary {
            operator: BinaryOperator::Named(ref operator),
            ..
        } if operator.kind == NamedOperatorKind::BothColon && operator.name == "around"
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
fn parses_operator_is_statements() {
    let item = parse_is_or_spec(r#"+ is \operator"#).expect("expected operator is statement");

    match item {
        IsOrSpec::Is(statement) => {
            assert!(matches!(
                statement.subject.kind,
                IsSubjectKind::Operator(ref operator) if operator.text == "+"
            ));
        }
        other => panic!("expected is statement, got {other:?}"),
    }
}

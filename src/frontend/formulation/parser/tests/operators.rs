use super::*;

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
fn parses_special_operator_expressions() {
    let expression = parse_expression("x > y + z").expect("expected special operator");

    match expression.kind {
        ExpressionKind::Binary {
            operator: BinaryOperator::Special(operator),
            left,
            right,
        } => {
            assert_eq!(operator.text, ">");
            assert!(matches!(left.kind, ExpressionKind::Name(ref name) if name == "x"));
            assert!(matches!(
                right.kind,
                ExpressionKind::Binary {
                    operator: BinaryOperator::Add(_),
                    ..
                }
            ));
        }
        other => panic!("expected special operator expression, got {other:?}"),
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
    let expression = parse_expression("x |plus| y").expect("expected named operator");

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
    let plain = parse_form_or_declaration("x_ |plus| y_").expect("expected infix operator");
    let left =
        parse_form_or_declaration("x_ :|before| y_").expect("expected left-colon infix operator");
    let right =
        parse_form_or_declaration("x_ |after|: y_").expect("expected right-colon infix operator");
    let both =
        parse_form_or_declaration("x_ :|around|: y_").expect("expected both-colon infix operator");

    assert!(matches!(
        plain.kind,
        FormOrDeclarationKind::InfixOperator { ref operator, .. } if operator.text == "plus"
    ));
    assert!(matches!(
        left.kind,
        FormOrDeclarationKind::InfixOperator { ref operator, .. } if operator.text == "before"
    ));
    assert!(matches!(
        right.kind,
        FormOrDeclarationKind::InfixOperator { ref operator, .. } if operator.text == "after"
    ));
    assert!(matches!(
        both.kind,
        FormOrDeclarationKind::InfixOperator { ref operator, .. } if operator.text == "around"
    ));
}

#[test]
fn parses_is_or_spec_statements() {
    let item = parse_is_or_spec(r#"f(x_), y_ is \function:on{A}:to{B}"#)
        .expect("expected is-or-spec statement");

    match item {
        IsOrSpec::Is(statement) => {
            assert!(matches!(statement.ty, TypeExpression::Command(_)));
            match statement.subject.kind {
                IsSubjectKind::Forms(forms) => {
                    assert_eq!(forms.len(), 2);
                    assert!(matches!(
                        forms[0],
                        IsSubjectForm::Form(FormOrDeclaration {
                            kind: FormOrDeclarationKind::FunctionDeclaration { ref name, .. },
                            ..
                        }) if name.is_none()
                    ));
                    assert!(matches!(forms[1], IsSubjectForm::PlaceholderForm(_)));
                }
                other => panic!("expected forms subject, got {other:?}"),
            }
        }
        other => panic!("expected is statement, got {other:?}"),
    }
}

#[test]
fn parses_is_via_statements() {
    let item = parse_is_via_statement(r#"x_, y_ is \type{A} via (x, y)"#)
        .expect("expected is-via statement");

    assert!(matches!(
        item.is_statement.subject.kind,
        IsSubjectKind::Forms(ref forms) if forms.len() == 2
    ));
    assert_eq!(item.tuple_form.elements.len(), 2);
}

#[test]
fn parses_refined_is_statements() {
    let item = parse_is_or_refined_statement_spec(r#"x_, y_ is \(f)::[[g]]"#)
        .expect("expected refined is statement");

    match item {
        IsOrRefinedStatementSpec::Is(statement) => {
            assert!(matches!(statement.ty, TypeExpression::RefinedCommand(_)));
            assert!(matches!(
                statement.subject.kind,
                IsSubjectKind::Forms(ref forms) if forms.len() == 2
            ));
        }
        other => panic!("expected refined is statement, got {other:?}"),
    }
}

use super::*;

#[test]
fn parses_refined_command_headers_with_prefix_and_chain_tails() {
    let header =
        parse_command_header(r#"\logic.$prefix.(f:on{A}, g:to{B})::target.$tail{C}:via{D}(E, F)"#)
            .expect("expected refined command header");

    match header {
        CommandHeader::Refined(header) => {
            let prefix_chain = header
                .prefix_chain
                .expect("expected refined header prefix chain");
            assert_eq!(prefix_chain.parts.len(), 2);
            assert_eq!(header.parts.len(), 2);
            match header.refined_tail {
                RefinedTail::Chain(chain) => {
                    assert_eq!(chain.parts.len(), 2);
                    assert!(matches!(
                        chain.parts[1],
                        ChainPart::Alias(ref name) if name == "tail"
                    ));
                }
                other => panic!("expected chain refined tail, got {other:?}"),
            }
            assert_eq!(header.head_args.len(), 1);
            assert_eq!(header.tail.len(), 1);
            assert_eq!(header.paren_args.len(), 1);
        }
        other => panic!("expected refined command header, got {other:?}"),
    }
}

#[test]
fn parses_refined_command_expressions_with_prefix_and_chain_tails() {
    let item = parse_is_or_refined_statement_spec(
        r#"x is \logic.$prefix.(f:on{x}, g:to{y})::target.$tail{z}:via{w}(u, v)"#,
    )
    .expect("expected refined command expression");

    match item {
        IsOrRefinedStatementSpec::Is(statement) => match statement.ty {
            TypeExpression::RefinedCommand(command) => {
                let prefix_chain = command
                    .prefix_chain
                    .expect("expected refined expression prefix chain");
                assert_eq!(prefix_chain.parts.len(), 2);
                assert_eq!(command.parts.len(), 2);
                match command.refined_tail {
                    RefinedTail::Chain(chain) => {
                        assert_eq!(chain.parts.len(), 2);
                    }
                    other => panic!("expected chain refined tail, got {other:?}"),
                }
                assert_eq!(command.head_args.len(), 1);
                assert_eq!(command.tail.len(), 1);
                assert_eq!(command.paren_args.len(), 1);
            }
            other => panic!("expected refined command type, got {other:?}"),
        },
        other => panic!("expected refined is statement, got {other:?}"),
    }
}

#[test]
fn parses_writing_aliases() {
    let alias = parse_writing_alias(r#"f(x_) :~> x + y"#).expect("expected writing alias to parse");

    match alias.form.kind {
        FormOrDeclarationKind::FunctionDeclaration { name, form } => {
            assert!(name.is_none());
            assert_eq!(form.name, "f");
        }
        other => panic!("expected function declaration lhs, got {other:?}"),
    }
    assert_eq!(alias.body, "x + y");
}

#[test]
fn parses_expression_aliases_for_form_and_infix_command_lhs_values() {
    let form_alias = parse_expression_alias(r#"f(x_) :=> x + y"#).expect("expected form alias");
    let infix_alias =
        parse_expression_alias(r#"\:apply:on{A}:/ :=> x"#).expect("expected infix command alias");

    assert!(matches!(form_alias.lhs, ExpressionAliasLhs::Form(_)));
    assert!(matches!(
        form_alias.expression.kind,
        ExpressionKind::Binary {
            operator: BinaryOperator::Add(_),
            ..
        }
    ));
    assert!(matches!(
        infix_alias.lhs,
        ExpressionAliasLhs::InfixCommand(_)
    ));
}

#[test]
fn rejects_refined_command_headers_as_expression_alias_lhs_values() {
    let error = parse_expression_alias(r#"\(f)::[[g]] :=> x"#)
        .expect_err("expected refined command alias lhs to be rejected");

    assert_eq!(
        error.to_string(),
        "refined command headers are not valid expression alias lhs values"
    );
}

#[test]
fn parses_spec_operator_aliases_with_spec_targets() {
    let alias = parse_spec_operator_alias(r#"x_ "in" X :-> + "on" Nat"#)
        .expect("expected spec operator alias");

    assert_eq!(alias.placeholder_spec.operator, "in");
    assert!(matches!(alias.target, IsOrSpec::Spec(_)));
}

#[test]
fn parses_header_variants_with_stropped_operator_segments() {
    let label = parse_label_header(r#"logic.`*`.ref"#).expect("expected dotted label header");
    let author = parse_author_header(r#"@`*+`"#).expect("expected stropped author header");
    let resource = parse_resource_header(r#"$book.`*+`"#).expect("expected resource header");

    assert_eq!(
        label.parts,
        vec!["logic".to_string(), "`*`".to_string(), "ref".to_string(),]
    );
    assert_eq!(author.parts, vec!["`*+`".to_string()]);
    assert_eq!(resource.parts, vec!["book".to_string(), "`*+`".to_string()]);
}

#[test]
fn rejects_non_operator_text_in_stropped_names() {
    let expression = parse_expression(r#"`some.thing`"#)
        .expect_err("expected invalid stropped expression name to fail");
    let label = parse_label_header(r#"logic.`set.ops`.ref"#)
        .expect_err("expected invalid stropped label part to fail");
    let author = parse_author_header(r#"@`isaac.newton`"#)
        .expect_err("expected invalid stropped author part to fail");
    let resource = parse_resource_header(r#"$book.`chapter.1`"#)
        .expect_err("expected invalid stropped resource part to fail");

    assert!(
        expression.to_string().contains("InvalidToken"),
        "expected invalid token error, got {}",
        expression
    );
    assert_eq!(label.to_string(), "invalid name ``set.ops``");
    assert_eq!(author.to_string(), "invalid name ``isaac.newton``");
    assert_eq!(resource.to_string(), "invalid name ``chapter.1``");
}

#[test]
fn rejects_missing_writing_alias_bodies() {
    let error = parse_writing_alias(r#"f(x_) :~>   "#).expect_err("expected empty body to fail");

    assert_eq!(error.to_string(), "writing alias body cannot be empty");
}

#[test]
fn rejects_missing_via_clauses_in_is_via_statements() {
    let error = parse_is_via_statement(r#"x is \type{A}"#)
        .expect_err("expected missing via clause to fail");

    assert_eq!(error.to_string(), "expected top-level ` via `");
}

#[test]
fn rejects_command_headers_without_leading_backslashes() {
    let error =
        parse_command_header("function:on{A}").expect_err("expected header without slash to fail");

    assert_eq!(error.to_string(), "command header must start with `\\`");
}

#[test]
fn rejects_command_header_tail_parts_without_curly_argument_lists() {
    let error = parse_command_header(r#"\function:on"#)
        .expect_err("expected tail without arguments to fail");

    assert_eq!(
        error.to_string(),
        "header tail parts require at least one `{...}` argument list"
    );
}

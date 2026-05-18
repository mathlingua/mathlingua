#[test]
fn parses_command_headers() {
    let header =
        parse_command_header(r#"\function:on{A}:to{B}(f(x_))"#).expect("expected command header");

    match header {
        CommandHeader::Command(CommandHeaderNode {
            tail, paren_args, ..
        }) => {
            assert_eq!(tail.len(), 2);
            assert_eq!(paren_args.len(), 1);
        }
        other => panic!("expected command header, got {other:?}"),
    }
}

#[test]
fn parses_commands_without_colon_arguments() {
    assert_simple_command_header(r#"\set"#, &["set"], 0, 0, 0);
    assert_simple_command_header(r#"\closed.set"#, &["closed", "set"], 0, 0, 0);
    assert_simple_command_header(
        r#"\axiom.of.existence.of.empty.set"#,
        &["axiom", "of", "existence", "of", "empty", "set"],
        0,
        0,
        0,
    );
}

#[test]
fn parses_command_variants_with_optional_head_tail_and_paren_arguments() {
    assert_simple_command_header(r#"\Z{n}"#, &["Z"], 1, 0, 0);
    assert_simple_command_header(r#"\function:on{A}:to{B}"#, &["function"], 0, 2, 0);
    assert_simple_command_header(r#"\group:over{A}"#, &["group"], 0, 1, 0);
    assert_simple_command_header(r#"\sin(x)"#, &["sin"], 0, 0, 1);
    assert_simple_command_header(
        r#"\generalized.zeta{n}(x)"#,
        &["generalized", "zeta"],
        1,
        0,
        1,
    );
    assert_simple_command_header(
        r#"\some.function:on{A}(x,y)"#,
        &["some", "function"],
        0,
        1,
        1,
    );
}

#[test]
fn parses_refined_command_headers() {
    let header = parse_command_header(r#"\(f)::[[g]]"#).expect("expected refined command header");

    match header {
        CommandHeader::Refined(header) => {
            assert_eq!(header.parts.len(), 1);
            assert!(matches!(header.refined_tail, RefinedTail::Name { .. }));
        }
        other => panic!("expected refined command header, got {other:?}"),
    }
}

#[test]
fn parses_expression_aliases() {
    let alias = parse_expression_alias(r#"\function:on{A}:to{B}(f(x_)) :=> x"#)
        .expect("expected expression alias");

    match alias.lhs {
        ExpressionAliasLhs::Command(_) => {}
        other => panic!("expected command header lhs, got {other:?}"),
    }
}

#[test]
fn parses_spec_operator_aliases() {
    let alias = parse_spec_operator_alias(r#"x_ "in" X :-> x is \type{A}"#)
        .expect("expected spec operator alias");

    assert_eq!(alias.placeholder_spec.operator, "in");
}

#[test]
fn parses_header_labels() {
    let label = parse_label_header("some.label").expect("expected label header");
    let author = parse_author_header("@euclid").expect("expected author header");
    let resource = parse_resource_header("$book.ref").expect("expected resource header");

    assert_eq!(label.parts, vec!["some".to_string(), "label".to_string()]);
    assert_eq!(author.parts, vec!["euclid".to_string()]);
    assert_eq!(resource.parts, vec!["book".to_string(), "ref".to_string()]);
}


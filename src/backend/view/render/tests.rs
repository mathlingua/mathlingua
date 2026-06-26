use super::{build_render_registry, render_formulation_latex, render_group_heading_latex};
use crate::events::EventLog;
use crate::frontend::{ParsedSourceFile, parse_document};
use std::path::PathBuf;

fn registry_for(source: &str) -> super::RenderRegistry {
    let mut event_log = EventLog::new();
    let document = parse_document(source, &mut event_log);
    assert!(event_log.events().is_empty());
    build_render_registry(&[ParsedSourceFile {
        path: PathBuf::from("test.mlg"),
        source: source.to_string(),
        document,
    }])
}

// ===============================[ tests ]=====================================

#[test]
fn renders_written_templates_with_subject_and_argument_substitutions() {
    let registry = registry_for(
        r#"[\function:on{A}:to{B}]
Describes: f(x__)
Documented:
. called: "function on $A?$ to $B?$"
  written:
  . "f? \: : \: A? \rightarrow B?"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"g is \function:on{X}:to{Y}"#, &registry),
        Some(r#"g \: : \: X \rightarrow Y"#.to_string())
    );
}

#[test]
fn renders_top_level_written_templates_without_repeating_subject() {
    let registry = registry_for(
        r#"[\function:on{A}:to{B}]
Describes: f(x__)
Documented:
. called: "function on $A?$ to $B?$"
. written: "f? : A? \rightarrow B?"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"f is \function:on{A}:to{B}"#, &registry),
        Some(r#"f : A \rightarrow B"#.to_string())
    );
}

#[test]
fn renders_is_predicates_with_called_when_written_contains_subject() {
    let registry = registry_for(
        r#"[\function:on{A}:to{B}]
Describes: f(x__)
Documented:
. called: "function on $A?$ to $B?$"
. written: "f? : A? \rightarrow B?"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"f is? \function:on{A}:to{B}"#, &registry),
        Some(r#"f \textrm{ is } \textrm{function on }A\textrm{ to }B"#.to_string())
    );
}

#[test]
fn renders_called_templates_as_text_when_written_is_missing() {
    let registry = registry_for(
        r#"[\group]
Describes: G ::= (X, *, e)
Documented:
. called: "group"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"G is \group"#, &registry),
        Some(r#"G \textrm{ is } \textrm{group}"#.to_string())
    );
}

#[test]
fn renders_comma_separated_is_or_spec_subjects_with_called_commands() {
    let registry = registry_for(
        r#"[\set]
Describes: X
Documented:
. called: "set"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"A, B is \set"#, &registry),
        Some(r#"A, B \textrm{ is } \textrm{set}"#.to_string())
    );
}

#[test]
fn renders_builtin_type_expressions_as_plain_text() {
    let registry = registry_for("");

    assert_eq!(
        render_formulation_latex(r#"X is \\statement"#, &registry),
        Some(r#"X \textrm{ is } \textrm{statement}"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"X, Y is \\expression"#, &registry),
        Some(r#"X, Y \textrm{ is } \textrm{expression}"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"S is \\specification"#, &registry),
        Some(r#"S \textrm{ is } \textrm{specification}"#.to_string())
    );
}

#[test]
fn renders_quoted_operators_as_temporary_latex_commands() {
    let registry = registry_for("");

    assert_eq!(
        render_formulation_latex(r#"x "in" X"#, &registry),
        Some(r#"x \in X"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"a "to" B"#, &registry),
        Some(r#"a \to B"#.to_string())
    );
}

#[test]
fn renders_called_templates_with_math_substitutions() {
    let registry = registry_for(
        r#"[\field:over{V}]
Describes: F
Documented:
. called: "field over $V?$"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"G is \field:over{X}"#, &registry),
        Some(r#"G \textrm{ is } \textrm{field over }X"#.to_string())
    );
}

#[test]
fn renders_refined_command_types_from_called_templates() {
    let registry = registry_for(
        r#"[\function:on{A}:to{B}]
Describes: f(x__)
Documented:
. called: "function on $A?$ to $B?$"
  written:
  . "f? \: : \: A? \rightarrow B?"

[\(bounded)::function:on{A}:to{B}]
Refines: f(x__) is \function:on{A}:to{B}
Documented:
. called: "bounded"
  written:
  . "\operatorname{Bounded}"

[\(continuous)::function:on{A}:to{B}]
Refines: f(x__) is \function:on{A}:to{B}
Documented:
. called: "continuous"
  written:
  . "\operatorname{Continuous}"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"g is \function:on{X}:to{Y}"#, &registry),
        Some(r#"g \: : \: X \rightarrow Y"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(
            r#"g is \(bounded, continuous)::function:on{X}:to{Y}"#,
            &registry
        ),
        Some(
            r#"g \textrm{ is } \textrm{bounded, continuous function on }X\textrm{ to }Y"#
                .to_string()
        )
    );
}

#[test]
fn renders_definition_group_headings_from_called_text() {
    let registry = registry_for(
        r#"[\function:on{A}:to{B}]
Describes: f(x__)
Documented:
. called: "function on $A?$ to $B?$"
"#,
    );

    assert_eq!(
        render_group_heading_latex(
            "Describes",
            Some(r#"\function:on{A}:to{B}"#),
            None,
            &registry
        ),
        Some(r#"\textrm{function on }A\textrm{ to }B"#.to_string())
    );
}

#[test]
fn renders_definition_group_heading_called_text_without_capitalizing() {
    let registry = registry_for(
        r#"[\set]
Describes: X
Documented:
. called: "set"
"#,
    );

    assert_eq!(
        render_group_heading_latex("Describes", Some(r#"\set"#), None, &registry),
        Some(r#"\textrm{set}"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"X is \set"#, &registry),
        Some(r#"X \textrm{ is } \textrm{set}"#.to_string())
    );
}

#[test]
fn renders_refines_group_headings_from_refinement_and_refined_called_text() {
    let registry = registry_for(
        r#"[\function:on{A}:to{B}]
Describes: f(x__)
Documented:
. called: "function on $A?$ to $B?$"

[\(continuous)::function:on{A}:to{B}]
Refines: f is \function:on{A}:to{B}
Documented:
. called: "continuous"
"#,
    );

    assert_eq!(
        render_group_heading_latex(
            "Refines",
            Some(r#"\(continuous)::function:on{A}:to{B}"#),
            Some(r#"f is \function:on{A}:to{B}"#),
            &registry
        ),
        Some(r#"\textrm{continuous function on }A\textrm{ to }B"#.to_string())
    );
}

#[test]
fn renders_function_forms_with_placeholder_suffixes_hidden() {
    let registry = registry_for("");

    assert_eq!(
        render_formulation_latex("f(x_)", &registry),
        Some(r#"f(x)"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("f(x__)", &registry),
        Some(r#"f(x)"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("f(x)", &registry),
        Some(r#"f(x)"#.to_string())
    );
}

#[test]
fn renders_tuple_declarations_with_operator_symbols() {
    let registry = registry_for("");

    assert_eq!(
        render_formulation_latex("G ::= (X, *, e)", &registry),
        Some(r#"G ::= \left(X, \ast, e\right)"#.to_string())
    );
}

#[test]
fn renders_dot_delimited_grouped_expressions_without_parentheses() {
    let registry = registry_for("");

    assert_eq!(
        render_formulation_latex("(x + y)", &registry),
        Some(r#"\left(x + y\right)"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("(.x + y.)", &registry),
        Some(r#"x + y"#.to_string())
    );
}

#[test]
fn renders_conditional_written_templates_for_optional_infix_tail() {
    let registry = registry_for(
        r#"[A \.intersect:?within{U}./ B]
Describes: I
Documented:
. called: "intersection"
. written: "A? \cap@[U]{_{U?}} B?"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"X \.intersect./ Y"#, &registry),
        Some(r#"X \cap Y"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"X \.intersect:within{Z}./ Y"#, &registry),
        Some(r#"X \cap_{Z} Y"#.to_string())
    );
}

#[test]
fn renders_written_templates_from_states_command_headings() {
    let registry = registry_for(
        r#"[A \.and./ B]
States:
that:
. allOf:
  . A
  . B
Documented:
. written: "A? \text{ and } B?"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"(.c "in"? A.) \.and./ (.c "in"? B.)"#, &registry),
        Some(r#"c \in A \text{ and } c \in B"#.to_string())
    );
}

#[test]
fn renders_theorem_like_command_headings_from_label_when_documentation_is_missing() {
    let registry = registry_for(
        r#"[\axiom.of.unordered.pair]
Axiom:
then: X is \set
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"\axiom.of.unordered.pair"#, &registry),
        Some(r#"\textrm{Axiom of Unordered Pair}"#.to_string())
    );
}

#[test]
fn renders_conditional_templates_with_fallbacks_multiple_vars_and_nesting() {
    let registry = registry_for(
        r#"[\decorate:?with{U}]
Describes: D
Documented:
. called: "decorated"
. written: "d@[U]{_{U?}}:{_X}"

[\both{x}:?and{y}]
Describes: B
Documented:
. called: "both"
. written: "@[x, y]{x? + y?}:{missing}"

[\nest{x}:?with{y}]
Describes: N
Documented:
. called: "nested"
. written: "@[x]{x? + @[y]{y?}:{*}}:{0}"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"\decorate"#, &registry),
        Some(r#"d_X"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"\decorate:with{Z}"#, &registry),
        Some(r#"d_{Z}"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"\both{A}"#, &registry),
        Some(r#"missing"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"\both{A}:and{B}"#, &registry),
        Some(r#"A + B"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"\nest{A}"#, &registry),
        Some(r#"A + *"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"\nest{A}:with{B}"#, &registry),
        Some(r#"A + B"#.to_string())
    );
}

#[test]
fn renders_conditionals_in_called_templates() {
    let registry = registry_for(
        r#"[\ambient:?within{U}]
Describes: A
Documented:
. called: "ambient@[U]{ within $U?$}:{ without ambient}"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"\ambient"#, &registry),
        Some(r#"\textrm{ambient}\textrm{ without ambient}"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"\ambient:within{Z}"#, &registry),
        Some(r#"\textrm{ambient}\textrm{ within }Z"#.to_string())
    );
}

#[test]
fn renders_set_builder_specs() {
    let registry = registry_for("");

    assert_eq!(
        render_formulation_latex("{x : y | z}", &registry),
        Some(r#"\left\{ x \: : \: y \: | \: z \right\}"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"{f_(a_, b_) : x_ "in" X | z}"#, &registry),
        Some(r#"\left\{ f(a, b) \: : \: x \in X \: | \: z \right\}"#.to_string())
    );
}

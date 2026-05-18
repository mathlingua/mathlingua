use super::{build_render_registry, render_formulation_latex, render_group_heading_latex};
use crate::backend::semantic::ParsedSourceFile;
use crate::events::EventLog;
use crate::frontend::structural::parse_document;
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
Describes: G := (X, *, e)
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
        Some(r#"f\left(x\right)"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("f(x__)", &registry),
        Some(r#"f\left(x\right)"#.to_string())
    );
}

#[test]
fn renders_tuple_declarations_with_operator_symbols() {
    let registry = registry_for("");

    assert_eq!(
        render_formulation_latex("G := (X, *, e)", &registry),
        Some(r#"G := \left(X, \ast, e\right)"#.to_string())
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
        render_formulation_latex(r#"{x "in" X : f_(a_, b_) | z}"#, &registry),
        Some(r#"\left\{ f\left(a, b\right) \: : \: x \in X \: | \: z \right\}"#.to_string())
    );
}

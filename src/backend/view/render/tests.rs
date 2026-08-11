use super::{
    build_render_registry, render_documented_text_latex, render_formulation_latex,
    render_group_heading_latex, render_group_parameter_destructurings,
    render_refines_section_latex,
};
use crate::events::EventLog;
use crate::frontend::{
    ParsedSourceFile, SourceFileViewMetadata, parse_document, top_level_item_ids,
};
use std::path::PathBuf;

fn registry_for(source: &str) -> super::RenderRegistry {
    let mut event_log = EventLog::new();
    let document = parse_document(source, &mut event_log);
    assert!(event_log.events().is_empty());
    build_render_registry(&[ParsedSourceFile {
        path: PathBuf::from("test.mlg"),
        source: source.to_string(),
        document,
        item_ids: top_level_item_ids(source),
        view_metadata: SourceFileViewMetadata::default(),
    }])
}

// ===============================[ tests ]=====================================

#[test]
fn renders_command_expressions_from_written_templates() {
    let registry = registry_for(
        r#"[\empty.set]
Declares: X is \set
Documented:
. written: "\emptyset"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"\empty.set"#, &registry),
        Some(r#"\emptyset"#.to_string())
    );
}

#[test]
fn renders_variadic_command_arguments_in_written_templates() {
    let registry = registry_for(
        r#"[\sequence:of{x...n}]
Defines: S
Documented:
. written: "x? \; (n?)"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"\sequence:of{a, b, c}"#, &registry),
        Some(r#"a, b, c \; (3)"#.to_string())
    );
}

#[test]
fn renders_variadic_map_and_reduce_builtins() {
    let registry = registry_for("");

    assert_eq!(
        render_formulation_latex(r#"\\map{x[1...i_...n]}:to{x[i_] + 1}"#, &registry),
        Some(r#"x_{1} + 1, x_{2} + 1, \ldots, x_{n} + 1"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(
            r#"\\map{x[1...i_...n], y[1...i_...n]}:to{x[i_] + y[i_] + 1}"#,
            &registry
        ),
        Some(r#"x_{1} + y_{1} + 1, x_{2} + y_{2} + 1, \ldots, x_{n} + y_{n} + 1"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"\\leftReduce{`+`}:on{x[1...n]}"#, &registry),
        Some(r#"x_{1} + \ldots + x_{n}"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"\\rightReduce{`+`}:on{x[1...i_...n]}"#, &registry),
        Some(r#"x_{1} + \ldots + x_{i} + \ldots + x_{n}"#.to_string())
    );
}

#[test]
fn renders_variadic_parameters_and_slices_symbolically() {
    let registry = registry_for(
        r#"[\plain{x...}]
States:
that: x... = x...
Documented:
. written: "x?"

[\one{x[i_ := 1...n]}]
States:
that: x[1...i_...n] = x[1...i_...n]
Documented:
. written: "x?"

[\zero{x[i_ := 0...n]}]
States:
that: x[0...i_...n] = x[0...i_...n]
Documented:
. written: "x?"
"#,
    );

    assert_eq!(
        render_group_heading_latex("States", Some(r#"\plain{x...}"#), None, &registry),
        Some(r#"x_{1}, \ldots, x_{.}"#.to_string())
    );
    assert_eq!(
        render_group_heading_latex("States", Some(r#"\one{x[i_ := 1...n]}"#), None, &registry),
        Some(r#"x_{1}, \ldots, x_{i}, \ldots, x_{n}"#.to_string())
    );
    assert_eq!(
        render_group_heading_latex("States", Some(r#"\zero{x[i_ := 0...n]}"#), None, &registry),
        Some(r#"x_{0}, \ldots, x_{i}, \ldots, x_{n}"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("x...", &registry),
        Some(r#"x_{1}, \ldots, x_{.}"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("x[1...i_...n]", &registry),
        Some(r#"x_{1}, \ldots, x_{i}, \ldots, x_{n}"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("x[0...i_...n]", &registry),
        Some(r#"x_{0}, \ldots, x_{i}, \ldots, x_{n}"#.to_string())
    );
}

#[test]
fn renders_variadic_broadcasts_symbolically() {
    let registry = registry_for("");

    assert_eq!(
        render_formulation_latex("x... = 0", &registry),
        Some(r#"x_{1} = 0, \; \ldots, \; x_{.} = 0"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("x[1...n] = 0", &registry),
        Some(r#"x_{1} = 0, \; \ldots, \; x_{n} = 0"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("x[1...i_...n] != 0", &registry),
        Some(r#"x_{1} \ne 0, \; \ldots, \; x_{i} \ne 0, \; \ldots, \; x_{n} \ne 0"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("x[0...n] := y", &registry),
        Some(r#"x_{0} := y, \; \ldots, \; x_{n} := y"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("x[1...n] = y[1...n]", &registry),
        Some(r#"x_{1} = y_{1}, \; \ldots, \; x_{n} = y_{n}"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("x[1...i_...n] = y[1...i_...n]", &registry),
        Some(
            r#"x_{1} = y_{1}, \; \ldots, \; x_{i} = y_{i}, \; \ldots, \; x_{n} = y_{n}"#
                .to_string()
        )
    );
    assert_eq!(
        render_formulation_latex(r#"x[1...n] "in" X"#, &registry),
        Some(r#"x_{1} \in X, \; \ldots, \; x_{n} \in X"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"x[1...n] "in"? X"#, &registry),
        Some(r#"x_{1} \in X, \; \ldots, \; x_{n} \in X"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"x[1...n] is \\opaque"#, &registry),
        Some(
            r#"x_{1} \textrm{ is } \textrm{opaque}, \; \ldots, \; x_{n} \textrm{ is } \textrm{opaque}"#
                .to_string()
        )
    );
    assert_eq!(
        render_formulation_latex(r#"x[1...n] is? \\opaque"#, &registry),
        Some(
            r#"x_{1} \textrm{ is } \textrm{opaque}, \; \ldots, \; x_{n} \textrm{ is } \textrm{opaque}"#
                .to_string()
        )
    );
}

#[test]
fn renders_variadic_written_and_called_join_notation() {
    let registry = registry_for(
        r#"[\post{x...}]
States:
that: x... = x...
Documented:
. written: "x?{...A}"

[\pre{x...}]
States:
that: x... = x...
Documented:
. written: "x?{B...}"

[\between{x...}]
States:
that: x... = x...
Documented:
. written: "x?{...\text{ and }...}"

[\called{x...}]
States:
that: x... = x...
Documented:
. called: "all of $x?{...\text{ and }...}$"

[\paren{x...}]
States:
that: x... = x...
Documented:
. written: "x+?{...C...}"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"\post{a, b, c}"#, &registry),
        Some("aAbAcA".to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"\pre{a, b, c}"#, &registry),
        Some("BaBbBc".to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"\between{a, b, c}"#, &registry),
        Some(r#"a\text{ and }b\text{ and }c"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"\called{a, b, c}"#, &registry),
        Some(r#"\textrm{all of }a\text{ and }b\text{ and }c"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"\paren{a, b, c}"#, &registry),
        Some(r#"\left(aCbCc\right)"#.to_string())
    );
}

#[test]
fn renders_command_context_assignments_as_visible_given_values() {
    let registry = registry_for(
        r#"[\axiom.of.extension]
Axiom:
then: P
Documented:
. called: "axiom of extension"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"\axiom.of.extension#given{A := X; B := Y}"#, &registry),
        Some(r#"\textrm{axiom of extension} \textrm{ given } X; Y"#.to_string())
    );
}

#[test]
fn renders_command_context_assignments_as_visible_using_values() {
    let registry = registry_for(
        r#"[\ordered.pair]
Defines: p
using: A, B is \set
Documented:
. called: "ordered pair"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"\ordered.pair#using{A := X; B := Y}"#, &registry),
        Some(r#"\textrm{ordered pair} \textrm{ using } X; Y"#.to_string())
    );
}

#[test]
fn renders_callable_owner_capabilities_from_written_templates() {
    let registry = registry_for(
        r#"[\relation:from{A}:to{B}]
Defines: R
when: A, B is \set
Enables:
. capability: R(a_, b_) :-> (a_, b_) "in" R
  written: "a_? \: R \: b_?"
Documented:
. called: "relation from $A?$ to $B?$"
. written: "R? \subseteq A? \times B?"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"R(a, b)"#, &registry),
        Some(r#"a \: R \: b"#.to_string())
    );
}

#[test]
fn renders_member_capabilities_from_written_templates() {
    let registry = registry_for(
        r#"[\group.element:of{G}]
Defines: x
Enables:
. capability: x.inv :=> \group.inverse:of{x}:in{G}
  written: "x+?^{-1}"
Documented:
. called: "group element"
"#,
    );

    assert_eq!(
        render_formulation_latex("x.inv", &registry),
        Some(r#"x^{-1}"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("(x * y).inv = y.inv * x.inv", &registry),
        Some(r#"\left(x \ast y\right)^{-1} = y^{-1} \ast x^{-1}"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("(x.inv).inv = x", &registry),
        Some(r#"\left(x^{-1}\right)^{-1} = x"#.to_string())
    );
}

#[test]
fn renders_documented_text_for_view_details() {
    assert_eq!(
        render_documented_text_latex("called", r#"membership of $x_?$ in $X?$"#),
        Some(r#"\textrm{membership of }x\textrm{ in }X"#.to_string())
    );
    assert_eq!(
        render_documented_text_latex("written", r#"x_? \in X?"#),
        Some(r#"x \in X"#.to_string())
    );
}

#[test]
fn renders_is_statements_from_called_templates() {
    let registry = registry_for(
        r#"[\function:on{A}:to{B}]
Defines: f(x__)
Documented:
. called: "function on $A?$ to $B?$"
. written: "f? : A? \rightarrow B?"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"g is \function:on{X}:to{Y}"#, &registry),
        Some(r#"g : X \rightarrow Y"#.to_string())
    );
}

#[test]
fn renders_is_predicates_with_called_when_written_contains_subject() {
    let registry = registry_for(
        r#"[\function:on{A}:to{B}]
Defines: f(x__)
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
fn renders_called_templates_as_text_when_called_is_present() {
    let registry = registry_for(
        r#"[\group]
Defines: G ::= (X, *, e)
Documented:
. called: "group"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"\group"#, &registry),
        Some(r#"\textrm{group}"#.to_string())
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
Defines: X
Documented:
. called: "set"
. written: "\operatorname{Set}"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"A, B is \set"#, &registry),
        Some(r#"A, B \textrm{ is } \operatorname{Set}"#.to_string())
    );
}

#[test]
fn renders_is_via_statements_from_called_templates() {
    let registry = registry_for(
        r#"[\set]
Defines: X
Documented:
. called: "set"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"G is \set via X"#, &registry),
        Some(r#"G \textrm{ is } \textrm{set} \textrm{ via } X"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"G is \set via (X, Y)"#, &registry),
        Some(r#"G \textrm{ is } \textrm{set} \textrm{ via } \left(X, Y\right)"#.to_string())
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
fn renders_structural_spec_literal_types() {
    let registry = registry_for(
        r#"[\natural]
Defines: n
Documented:
. written: "\mathbb{N}"

[\reals]
Defines: x
Documented:
. written: "\mathbb{R}"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"x is (? is \natural, ? "in" \reals)"#, &registry),
        Some(
            r#"x \textrm{ is } \left(? \textrm{ is } \mathbb{N}, ? \in \mathbb{R}\right)"#
                .to_string()
        )
    );
    assert_eq!(
        render_formulation_latex(r#"S is {? is \natural : ...}"#, &registry),
        Some(
            r#"S \textrm{ is } \left\{? \textrm{ is } \mathbb{N} \: : \: \ldots\right\}"#
                .to_string()
        )
    );
    assert_eq!(
        render_formulation_latex(
            r#"S is {(? is \natural, ? "in" \reals) : ...}"#,
            &registry
        ),
        Some(
            r#"S \textrm{ is } \left\{\left(? \textrm{ is } \mathbb{N}, ? \in \mathbb{R}\right) \: : \: \ldots\right\}"#
                .to_string()
        )
    );
    assert_eq!(
        render_formulation_latex(
            r#"f is (? is \natural) -> (? "in" \reals)"#,
            &registry
        ),
        Some(
            r#"f \textrm{ is } \left(? \textrm{ is } \mathbb{N}\right) \to \left(? \in \mathbb{R}\right)"#
                .to_string()
        )
    );
    assert_eq!(
        render_formulation_latex(
            r#"f is (? is \natural, ? "in" \reals) -> (? is \natural)"#,
            &registry
        ),
        Some(
            r#"f \textrm{ is } \left(? \textrm{ is } \mathbb{N}, ? \in \mathbb{R}\right) \to \left(? \textrm{ is } \mathbb{N}\right)"#
                .to_string()
        )
    );
}

#[test]
fn uses_written_as_called_form_when_called_is_missing() {
    let registry = registry_for(
        r#"[\empty.set]
Declares: X is \set
Documented:
. written: "\emptyset"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"X != \empty.set"#, &registry),
        Some(r#"X \ne \emptyset"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"X is \empty.set"#, &registry),
        Some(r#"X \textrm{ is } \emptyset"#.to_string())
    );
    assert_eq!(
        render_group_heading_latex("Declares", Some(r#"\empty.set"#), None, &registry),
        Some(r#"\emptyset"#.to_string())
    );
}

#[test]
fn renders_called_templates_with_math_substitutions() {
    let registry = registry_for(
        r#"[\field:over{V}]
Defines: F
Documented:
. called: "field over $V?$"
. written: "\mathsf{Field}_{V?}"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"G is \field:over{X}"#, &registry),
        Some(r#"G \textrm{ is } \mathsf{Field}_{X}"#.to_string())
    );
}

#[test]
fn renders_refined_command_types_from_called_templates() {
    let registry = registry_for(
        r#"[\function:on{A}:to{B}]
Defines: f(x__)
Documented:
. called: "function on $A?$ to $B?$"
  written:
  . "f? \: : \: A? \rightarrow B?"

[\(bounded)::function:on{A}:to{B}]
Refines: f(x__)
Documented:
. adjective: "bounded"
. written: "\operatorname{Bounded}"

[\(continuous)::function:on{A}:to{B}]
Refines: f(x__)
Documented:
. adjective: "continuous"
. written: "\operatorname{Continuous}"
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
            r#"g \textrm{ is } \left(\textrm{bounded}\textrm{ and }\textrm{continuous}\right)\textrm{ }\textrm{function on }X\textrm{ to }Y"#
                .to_string()
        )
    );
}

#[test]
fn renders_refined_predicates_and_missing_placeholders_without_question_marks() {
    let registry = registry_for(
        r#"[\function:?on{A}:?to{B}]
Defines: f(x__)
Documented:
. called: "function on $A?$ to $B?$"
. written: "f? \: : \: A? \rightarrow B?"

[\(injective)::function:?on{A}:?to{B}]
Refines: f(x__)
Documented:
. adjective: "injective"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"f is \function"#, &registry),
        Some(r#"f \: : \: A \rightarrow B"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"f is? \(injective)::function"#, &registry),
        Some(
            r#"f \textrm{ is } \textrm{injective}\textrm{ }\textrm{function on }A\textrm{ to }B"#
                .to_string()
        )
    );
}

#[test]
fn renders_definition_group_headings_from_both_documented_forms() {
    let registry = registry_for(
        r#"[\function:on{A}:to{B}]
Defines: f(x__)
Documented:
. called: "function on $A?$ to $B?$"
. written: "f? \: : \: A? \rightarrow B?"
"#,
    );

    assert_eq!(
        render_group_heading_latex(
            "Defines",
            Some(r#"\function:on{A}:to{B}"#),
            None,
            &registry
        ),
        Some(r#"\textrm{function on }A\textrm{ to }B\quad\htmlClass{mlg-title-written}{f \: : \: A \rightarrow B}"#.to_string())
    );
}

#[test]
fn definition_group_heading_order_is_called_then_written_however_they_are_listed() {
    let registry = registry_for(
        r#"[\function:on{A}:to{B}]
Defines: f(x__)
Documented:
. written: "f? \: : \: A? \rightarrow B?"
. called: "function on $A?$ to $B?$"
"#,
    );

    // `written:` is listed first, but a title that shows both forms always reads
    // `<called>: <written>`, exactly as when `called:` is listed first.
    assert_eq!(
        render_group_heading_latex(
            "Defines",
            Some(r#"\function:on{A}:to{B}"#),
            None,
            &registry
        ),
        Some(r#"\textrm{function on }A\textrm{ to }B\quad\htmlClass{mlg-title-written}{f \: : \: A \rightarrow B}"#.to_string())
    );

    // Listing `written:` first still decides which form names the item inline.
    assert_eq!(
        render_formulation_latex(r#"g is \function:on{X}:to{Y}"#, &registry),
        Some(r#"g \: : \: X \rightarrow Y"#.to_string())
    );
}

#[test]
fn destructured_parameters_render_as_base_names_in_titles_with_separate_destructuring_lines() {
    // A definition whose header parameters are destructured (`H ::= (X', *', e')`)
    // keeps the plain names in its card title (`H \leq G`) and surfaces the
    // destructuring as separate lines; an expression use also uses plain names.
    let registry = registry_for(
        r#"[H ::= (X', *', e') \:submagma:/ G ::= (X, *, e)]
Defines: H ::= (X', *', e')
Documented:
. written: "H? \leq G?"
"#,
    );

    let heading = r#"H ::= (X', *', e') \:submagma:/ G ::= (X, *, e)"#;

    assert_eq!(
        render_group_heading_latex("Defines", Some(heading), None, &registry),
        Some(r#"H \leq G"#.to_string())
    );
    assert_eq!(
        render_group_parameter_destructurings("Defines", Some(heading), &registry),
        vec![
            r#"H ::= \left(X', *', e'\right)"#.to_string(),
            r#"G ::= \left(X, \ast, e\right)"#.to_string(),
        ]
    );
    assert_eq!(
        render_formulation_latex(r#"A \:submagma:/ B"#, &registry),
        Some(r#"A \leq B"#.to_string())
    );
}

#[test]
fn renders_plain_called_placeholders_in_group_headings() {
    let registry = registry_for(
        r#"[A \:subset:/ B]
Defines: A
Documented:
. called: "A? subset of B?"
"#,
    );

    assert_eq!(
        render_group_heading_latex("Defines", Some(r#"A \:subset:/ B"#), None, &registry),
        Some(r#"A\textrm{ subset of }B"#.to_string())
    );
}

#[test]
fn renders_definition_group_heading_forms_without_capitalizing() {
    let registry = registry_for(
        r#"[\set]
Defines: X
Documented:
. called: "set"
. written: "\operatorname{Set}"
"#,
    );

    assert_eq!(
        render_group_heading_latex("Defines", Some(r#"\set"#), None, &registry),
        Some(r#"\textrm{set}\quad\htmlClass{mlg-title-written}{\operatorname{Set}}"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"X is \set"#, &registry),
        Some(r#"X \textrm{ is } \operatorname{Set}"#.to_string())
    );
}

#[test]
fn renders_refines_group_headings_from_refinement_and_refined_called_text() {
    let registry = registry_for(
        r#"[\function:on{A}:to{B}]
Defines: f(x__)
Documented:
. called: "function on $A?$ to $B?$"
. written: "f? \: : \: A? \rightarrow B?"

[\(continuous)::function:on{A}:to{B}]
Refines: f
Documented:
. adjective: "continuous"
. written: "\operatorname{Continuous}"
"#,
    );

    assert_eq!(
        render_group_heading_latex(
            "Refines",
            Some(r#"\(continuous)::function:on{A}:to{B}"#),
            Some(r#"f"#),
            &registry
        ),
        Some(r#"\textrm{continuous}\textrm{ }\textrm{function on }A\textrm{ to }B"#.to_string())
    );
}

#[test]
fn refines_display_keeps_explicit_types_and_infix_refinements_unchanged() {
    let registry = registry_for(
        r#"[\group]
Defines: G
Documented:
. called: "group"
"#,
    );

    assert_eq!(
        render_refines_section_latex(
            r#"G ::= (X, *, e) is \group"#,
            r#"\(finite)::group"#,
            &registry,
        ),
        Some(r#"G ::= \left(X, \ast, e\right) \textrm{ is } \textrm{group}"#.to_string())
    );
    assert_eq!(
        render_refines_section_latex("A", r#"A \:(nonempty)::subset:/ B"#, &registry),
        None
    );
}

#[test]
fn refines_display_adds_the_base_type_to_operator_targets() {
    let registry = registry_for(
        r#"[\binary.operation:on{X}]
Defines: x_ * y_
Documented:
. called: "binary operation on $X?$"
"#,
    );

    assert_eq!(
        render_refines_section_latex(
            "x_ * y_",
            r#"\(commutative)::binary.operation:on{X}"#,
            &registry,
        ),
        Some(r#"x \ast y \textrm{ is } \textrm{binary operation on }X"#.to_string())
    );
}

#[test]
fn renders_refined_spec_infix_headings_and_expressions() {
    let registry = registry_for(
        r#"[A \:subset:/ B]
Defines: A
Documented:
. called: "$A?$ subset of $B?$"

[A \:(nonempty)::subset:/ B]
Refines: A
Documented:
. adjective: "nonempty"
. written: "A? \subsetneq B?"
"#,
    );

    assert_eq!(
        render_group_heading_latex(
            "Refines",
            Some(r#"A \:(nonempty)::subset:/ B"#),
            Some("A"),
            &registry,
        ),
        Some(r#"A\textrm{ subset of }B\textrm{ }\left(\textrm{nonempty}\right)"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"X' \:(nonempty)::subset:/ X"#, &registry),
        Some(r#"X' \subsetneq X"#.to_string())
    );
}

#[test]
fn renders_implicit_refined_spec_infix_adjectives_after_the_base() {
    let registry = registry_for(
        r#"[\set]
Defines: X
Documented:
. called: "set"

[\(a)::set]
Refines: X
Documented:
. adjective: "alpha"

[\(b)::set]
Refines: X
Documented:
. adjective: "beta"

[\(c)::set]
Refines: X
Documented:
. adjective: "gamma"

[A \:subset:/ B]
Defines: A
Documented:
. called: "$A?$ subset of $B?$"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"X is \(a, b, c)::set"#, &registry),
        Some(
            r#"X \textrm{ is } \left(\textrm{alpha}\textrm{, }\textrm{beta}\textrm{, and }\textrm{gamma}\right)\textrm{ }\textrm{set}"#
                .to_string()
        )
    );
    assert_eq!(
        render_formulation_latex(r#"X \:(a, b, c)::subset:/ Y"#, &registry),
        Some(
            r#"X\textrm{ subset of }Y\textrm{ }\left(\textrm{alpha}\textrm{, }\textrm{beta}\textrm{, and }\textrm{gamma}\right)"#
                .to_string()
        )
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
fn renders_trailing_digits_in_names_as_subscripts() {
    let registry = registry_for("");

    assert_eq!(
        render_formulation_latex("x1", &registry),
        Some(r#"x_1"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("abc123 + y2", &registry),
        Some(r#"abc_{123} + y_2"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("f1(x2_)", &registry),
        Some(r#"f_1(x_2)"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("{x1 : y2 | z3}", &registry),
        Some(r#"\left\{ x_1 \: : \: y_2 \: | \: z_3 \right\}"#.to_string())
    );
}

#[test]
fn renders_names_with_collection_writing_aliases() {
    let registry = registry_for(
        r#"Writing:
. alpha :~> \alpha
. beta :~> \beta
"#,
    );

    assert_eq!(
        render_formulation_latex("alpha", &registry),
        Some(r#"\alpha"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("alpha_1", &registry),
        Some(r#"\alpha_1"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("alpha_beta", &registry),
        Some(r#"\alpha_{\beta}"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("alpha123 + beta2", &registry),
        Some(r#"\alpha_{123} + \beta_2"#.to_string())
    );
}

#[test]
fn renders_documented_mapping_writing_templates() {
    let registry = registry_for(
        r#"[\real.sequence]
Defines: x(i_)
Documented:
. called: "real sequence"
. writing: x(i)
  as: "x?_{i?}"
. writing: x(i_)
  as: "\left\{x?\right\}_{i_?=1}^{\infty}"
"#,
    );

    assert_eq!(
        render_formulation_latex("x(j)", &registry),
        Some(r#"x_{j}"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"x(i_) is \real.sequence"#, &registry),
        Some(r#"\left\{x\right\}_{i=1}^{\infty} \textrm{ is } \textrm{real sequence}"#.to_string())
    );
}

#[test]
fn renders_tuple_declarations_with_operator_symbols() {
    let registry = registry_for("");

    assert_eq!(
        render_formulation_latex("G ::= (X, *, e)", &registry),
        Some(r#"G ::= \left(X, \ast, e\right)"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("G1 ::= (X1, *_1, e1)", &registry),
        Some(r#"G_1 ::= \left(X_1, \ast_1, e_1\right)"#.to_string())
    );
    assert_eq!(
        render_formulation_latex("a1 *_1 b1", &registry),
        Some(r#"a_1 \ast_1 b_1"#.to_string())
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
fn renders_stropped_operators_without_backticks() {
    let registry = registry_for("");

    // A backtick-stropped operator renders as the operator itself, not `` `*` ``.
    assert_eq!(
        render_formulation_latex("`*`", &registry),
        Some("*".to_string())
    );
    assert_eq!(
        render_formulation_latex("`*'`", &registry),
        Some("*'".to_string())
    );
}

#[test]
fn renders_conditional_written_templates_for_optional_infix_tail() {
    let registry = registry_for(
        r#"[A \.intersect:?within{U}./ B]
Defines: I
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
fn ensure_paren_modifier_wraps_a_nested_documented_statement() {
    let mut registry = registry_for(
        r#"[\not{P}]
States:
when: P is \\statement
that:
. not: P
Documented:
. written: "\neg P+?"

[P \.and./ Q]
States:
when: P, Q is \\statement
that:
. allOf:
  . P
  . Q
Documented:
. written: "P? \text{ and } Q?"
"#,
    );
    registry.link_references = true;

    assert_eq!(
        render_formulation_latex(r#"\not{P}"#, &registry),
        Some(r#"\htmlData{mlg-ref=5c6e6f74}{\neg P}"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"\not{P \.and./ P}"#, &registry),
        Some(
            r#"\htmlData{mlg-ref=5c6e6f74}{\neg \left(\htmlData{mlg-ref=5c2e616e642e2f}{P \text{ and } P}\right)}"#
                .to_string()
        )
    );
}

#[test]
fn renders_theorem_like_command_headings_from_label_when_documentation_is_missing() {
    let registry = registry_for(
        r#"[\axiom.of.unordered.pair]
Axiom:
then: X is \set

[\twin.prime.conjecture]
Conjecture:
then: X is \set
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"\axiom.of.unordered.pair"#, &registry),
        Some(r#"\textrm{Axiom of Unordered Pair}"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"\twin.prime.conjecture"#, &registry),
        Some(r#"\textrm{Twin Prime Conjecture}"#.to_string())
    );
}

#[test]
fn renders_conditional_templates_with_fallbacks_multiple_vars_and_nesting() {
    let registry = registry_for(
        r#"[\decorate:?with{U}]
Defines: D
Documented:
. called: "decorated"
. written: "d@[U]{_{U?}}:{_X}"

[\both{x}:?and{y}]
Defines: B
Documented:
. called: "both"
. written: "@[x, y]{x? + y?}:{missing}"

[\nest{x}:?with{y}]
Defines: N
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
Defines: A
Documented:
. called: "ambient@[U]{ within $U?$}:{ without ambient}"
. written: "\operatorname{Ambient}@[U]{_{U?}}"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"\ambient"#, &registry),
        Some(r#"\operatorname{Ambient}"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"A is \ambient"#, &registry),
        Some(r#"A \textrm{ is } \operatorname{Ambient}"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"A is \ambient:within{Z}"#, &registry),
        Some(r#"A \textrm{ is } \operatorname{Ambient}_{Z}"#.to_string())
    );
}

#[test]
fn written_that_is_a_whole_unbound_conditional_falls_back_to_called() {
    // The `written:` is a single top-level `@[A, B]{…}` with no fallback: with no
    // `on`/`to` arguments its variables are unbound, so it is treated as missing and
    // rendering falls back to `called:`. With the arguments bound it is used.
    let registry = registry_for(
        r#"[\function:?on{A}:?to{B}]
Defines: f(x__)
Documented:
. called: "func@[A]{ on $A?$}@[B]{ to $B?$}"
. written: "@[A, B]{f? : A? \rightarrow B?}"
"#,
    );

    // No `A`/`B`: the whole `written:` is missing, so `called:` is used ("func").
    assert_eq!(
        render_formulation_latex(r#"\function"#, &registry),
        Some(r#"\textrm{func}"#.to_string())
    );

    // With both bound the `written:` renders.
    assert_eq!(
        render_formulation_latex(r#"\function:on{X}:to{Y}"#, &registry),
        Some(r#"f : X \rightarrow Y"#.to_string())
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

#[test]
fn renders_soft_build_marker_as_space() {
    let registry = registry_for(
        r#"[\set]
Defines: X
Documented:
. called: "set"
"#,
    );

    assert_eq!(
        render_formulation_latex(r#"\set@X"#, &registry),
        Some(r#"\textrm{set}\,X"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"\set@{(a_, b_) : a_ "in" A; b_ "in" B}"#, &registry),
        Some(
            r#"\textrm{set}\,\left\{ \left(a, b\right) \: : \: a \in A;\, b \in B \right\}"#
                .to_string()
        )
    );
}

/// Renders `argument` through a `written:` template that is exactly the single
/// placeholder `A?`, `A+?`, or `A-?`, so the result is the substituted value alone.
fn rendered_with_modifier(modifier: &str, argument: &str) -> String {
    let registry = registry_for(&format!(
        "[\\f:of{{A}}]\nDefines: X\nDocumented:\n. written: \"A{modifier}?\"\n"
    ));

    render_formulation_latex(&format!(r#"\f:of{{{argument}}}"#), &registry)
        .expect("expected the formulation to render")
}

fn keep(argument: &str) -> String {
    rendered_with_modifier("", argument)
}

fn ensure(argument: &str) -> String {
    rendered_with_modifier("+", argument)
}

fn strip(argument: &str) -> String {
    rendered_with_modifier("-", argument)
}

#[test]
fn ensure_paren_modifier_wraps_compound_values_exactly_once() {
    // A compound value gains one pair of parentheses.
    assert_eq!(ensure("1+2"), r#"\left(1 + 2\right)"#);
    // An already-parenthesized value keeps exactly one pair, never two.
    assert_eq!(ensure("(1+2)"), r#"\left(1 + 2\right)"#);
    // Redundant layers collapse to exactly one.
    assert_eq!(ensure("(((1+2)))"), r#"\left(1 + 2\right)"#);
}

#[test]
fn ensure_paren_modifier_leaves_single_names_unwrapped() {
    assert_eq!(ensure("a"), "a");
    // A parenthesized name is reduced to the bare name rather than re-wrapped.
    assert_eq!(ensure("(a)"), "a");
}

#[test]
fn strip_paren_modifier_removes_every_wrapping_layer() {
    assert_eq!(strip("1+2"), "1 + 2");
    assert_eq!(strip("(1+2)"), "1 + 2");
    assert_eq!(strip("(((1+2)))"), "1 + 2");
    assert_eq!(strip("a"), "a");
    assert_eq!(strip("(a)"), "a");
}

#[test]
fn keep_paren_modifier_substitutes_the_value_verbatim() {
    assert_eq!(keep("1+2"), "1 + 2");
    assert_eq!(keep("(1+2)"), r#"\left(1 + 2\right)"#);
    assert_eq!(keep("a"), "a");
    assert_eq!(
        keep("(((1+2)))"),
        r#"\left(\left(\left(1 + 2\right)\right)\right)"#
    );
}

#[test]
fn paren_modifiers_only_act_on_parentheses_that_wrap_the_whole_value() {
    // The leading `(` of `(1+2)+(3+4)` closes before the end, so it does not wrap
    // the expression: stripping must not turn this into `1 + 2\right) + \left(3 + 4`.
    let both = r#"\left(1 + 2\right) + \left(3 + 4\right)"#;

    assert_eq!(keep("(1+2)+(3+4)"), both);
    assert_eq!(strip("(1+2)+(3+4)"), both);
    assert_eq!(ensure("(1+2)+(3+4)"), format!(r#"\left({both}\right)"#));
}

#[test]
fn ensure_paren_modifier_treats_a_function_call_as_a_single_atom() {
    assert_eq!(ensure("f(x)"), "f(x)");
    // A comma-separated tuple is compound, so it keeps its parentheses.
    assert_eq!(ensure("(X, Y)"), r#"\left(X, Y\right)"#);
    assert_eq!(strip("(X, Y)"), "X, Y");
}

#[test]
fn paren_modifiers_render_as_bare_names_without_substitutions() {
    // Card titles and other display renderings have no values to parenthesize, so
    // a modifier shows the same name that `X?` does.
    assert_eq!(
        render_documented_text_latex("written", r#"P-? \iff Q+?"#),
        Some(r#"P \iff Q"#.to_string())
    );
    assert_eq!(
        render_documented_text_latex("called", r#"iff of $P-?$ and $Q+?$"#),
        Some(r#"\textrm{iff of }P\textrm{ and }Q"#.to_string())
    );
}

#[test]
fn a_name_followed_by_plus_or_minus_is_still_ordinary_text() {
    let registry = registry_for(
        r#"[\g:of{A}:and{B}]
Defines: X
Documented:
. written: "A?-B? \: A?+B?"
"#,
    );

    // `A?-B?` is a placeholder, a literal `-`, and another placeholder: the
    // modifier only applies when the `+`/`-` sits between the name and the `?`.
    assert_eq!(
        render_formulation_latex(r#"\g:of{1+2}:and{y}"#, &registry),
        Some(r#"1 + 2-y \: 1 + 2+y"#.to_string())
    );
}

#[test]
fn a_composed_card_title_does_not_change_how_the_item_is_named_inline() {
    let registry = registry_for(
        r#"[\empty.set]
Defines: X
Documented:
. called: "empty set"
. written: "\emptyset"
"#,
    );

    // The card title shows both forms.
    assert_eq!(
        render_group_heading_latex("Defines", Some(r#"\empty.set"#), None, &registry),
        Some(r#"\textrm{empty set}\quad\htmlClass{mlg-title-written}{\emptyset}"#.to_string())
    );

    // Inline uses of the item keep naming it with a single form, so a title that
    // shows both must not leak into `is` statements, references, or expressions.
    assert_eq!(
        render_formulation_latex(r#"X is \empty.set"#, &registry),
        Some(r#"X \textrm{ is } \emptyset"#.to_string())
    );
    assert_eq!(
        render_formulation_latex(r#"Y != \empty.set"#, &registry),
        Some(r#"Y \ne \emptyset"#.to_string())
    );
}

#[test]
fn a_written_nested_inside_called_pairs_with_it_in_the_title() {
    let registry = registry_for(
        r#"[\function:on{A}:to{B}]
Defines: f(x__)
Documented:
. called: "function on $A?$ to $B?$"
  written:
  . "f? \: : \: A? \rightarrow B?"
"#,
    );

    assert_eq!(
        render_group_heading_latex(
            "Defines",
            Some(r#"\function:on{A}:to{B}"#),
            None,
            &registry
        ),
        Some(
            r#"\textrm{function on }A\textrm{ to }B\quad\htmlClass{mlg-title-written}{f \: : \: A \rightarrow B}"#
                .to_string()
        )
    );
}

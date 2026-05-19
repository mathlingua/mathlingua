use super::*;

#[test]
fn check_reports_structural_and_formulation_events_for_invalid_files() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("broken-structural.mlg");

    fs::write(&file, "[\\function]\nDefines: x |plus|\n").unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(
        temp_dir.path(),
        &[PathBuf::from("broken-structural.mlg")],
        &mut event_log,
    );

    assert_eq!(result.files_checked, 1);
    assert!(
        event_log
            .events()
            .iter()
            .filter_map(Event::as_message)
            .any(|event| {
                event.location.as_ref().is_some_and(|location| {
                    matches!(
                        location,
                        crate::events::EventLocation::File { path, .. }
                            if *path == file.canonicalize().unwrap()
                    )
                }) && event.message.starts_with("Invalid Defines formulation:")
            })
    );
    assert!(
        user_events(&event_log).last().is_some_and(
            |event| event == &Event::user_log("Found 1 issue.").with_origin("mlg_check")
        )
    );
}

#[test]
fn check_reports_duplicate_command_signatures_across_definition_kinds() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("duplicates.mlg");

    fs::write(
        &file,
        r#"[\function{A, B}]
Defines: A "defines" B
Documented:
. [docs.called]
  called:
  . "function"

[\function{A}]
Theorem:
then:
. A = A
"#,
    )
    .unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(
        temp_dir.path(),
        &[PathBuf::from("duplicates.mlg")],
        &mut event_log,
    );

    assert_eq!(result.files_checked, 1);
    let canonical_file = file.canonicalize().unwrap();
    assert!(has_user_error_at(
        &event_log,
        &canonical_file,
        7,
        1,
        &format!(
            "Duplicate command signature `\\function` in Theorem; previously defined as Defines in {}:1:2",
            canonical_file.display()
        )
    ));
    assert!(event_log.has_errors());
}

#[test]
fn check_reports_references_to_undefined_command_signatures() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("undefined.mlg");

    fs::write(
        &file,
        r#"[\function:on{A}:to{B}]
Defines: A "defines" B
Documented:
. [docs.called]
  called:
  . "function"

Theorem:
then:
. x is \function{A, B}
"#,
    )
    .unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(
        temp_dir.path(),
        &[PathBuf::from("undefined.mlg")],
        &mut event_log,
    );

    assert_eq!(result.files_checked, 1);
    assert!(has_user_error_at(
        &event_log,
        &file.canonicalize().unwrap(),
        9,
        7,
        "Undefined command signature `\\function`"
    ));
    assert!(event_log.has_errors());
}

#[test]
fn check_reports_command_argument_shape_mismatches() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("arity.mlg");

    fs::write(
        &file,
        r#"[\foo{A, B}(x)]
Defines: A "defines" B
Documented:
. [docs.called]
  called:
  . "foo"

Theorem:
then:
. y is \foo{A}(x, z)
"#,
    )
    .unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(
        temp_dir.path(),
        &[PathBuf::from("arity.mlg")],
        &mut event_log,
    );

    assert_eq!(result.files_checked, 1);
    assert!(has_user_error_at(
        &event_log,
        &file.canonicalize().unwrap(),
        9,
        7,
        "Command signature `\\foo` expects argument shape `{2}(1)` but found `{1}(2)`"
    ));
    assert!(event_log.has_errors());
}

#[test]
fn check_accepts_defined_command_references_with_matching_argument_shape() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("valid-reference.mlg");

    fs::write(
        &file,
        r#"[\thing]
Describes: value
Documented:
. called: "thing"

[\foo{A, B}(x)]
Defines: A "defines" B
Documented:
. [docs.called]
  called:
  . "foo"

Theorem:
given:
. y, C, D, z is \thing
then:
. y is \foo{C, D}(z)
"#,
    )
    .unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(
        temp_dir.path(),
        &[PathBuf::from("valid-reference.mlg")],
        &mut event_log,
    );

    assert_eq!(result.files_checked, 1);
    assert_eq!(
        user_events(&event_log),
        [Event::user_log("Checked 1 file").with_origin("mlg_check")]
    );
}

#[test]
fn check_accepts_command_references_that_omit_defined_paren_arguments() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("optional-parens.mlg");

    fs::write(
        &file,
        r#"[\thing]
Describes: value
Documented:
. called: "thing"

[\some.function{A}(x, y)]
Defines: A "defines" B
Documented:
. [docs.called]
  called:
  . "some function"

Theorem:
given:
. f, X, g, a, b is \thing
then:
. f is \some.function{X}
. g is \some.function{X}(a, b)
"#,
    )
    .unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(
        temp_dir.path(),
        &[PathBuf::from("optional-parens.mlg")],
        &mut event_log,
    );

    assert_eq!(result.files_checked, 1);
    assert_eq!(
        user_events(&event_log),
        [Event::user_log("Checked 1 file").with_origin("mlg_check")]
    );
}

#[test]
fn check_accepts_composed_refined_command_references_in_given_sections() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("refined-list.mlg");

    fs::write(
        &file,
        r#"[\set]
Describes: X
Documented:
. called: "set"

[\function:on{A}:to{B}]
Describes: f(x__)
when: A, B is \set
Documented:
. called: "Function on $A?$ to $B?$"

[\(bounded)::function:on{A}:to{B}]
Refines: f(x__) is \function:on{A}:to{B}
Documented:
. called: "bounded"

[\(continuous)::function:on{A}:to{B}]
Refines: f(x__) is \function:on{A}:to{B}
Documented:
. called: "continuous"

Theorem:
given:
. A, B is \set
. f is \(continuous, bounded)::function:on{A}:to{B}
then: f is? \function:on{A}:to{B}
"#,
    )
    .unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(
        temp_dir.path(),
        &[PathBuf::from("refined-list.mlg")],
        &mut event_log,
    );

    assert_eq!(result.files_checked, 1);
    assert_eq!(
        user_events(&event_log),
        [Event::user_log("Checked 1 file").with_origin("mlg_check")]
    );
}

#[test]
fn check_accepts_command_when_requirement_from_given_type_fact() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("type-fact.mlg");

    fs::write(
        &file,
        r#"[\real]
Describes: x
Documented:
. called: "real"

[\foo{s}]
Describes: x
when: s is \real
Documented:
. called: "foo"

Theorem:
given: r is \real
then:
. \foo{r}
"#,
    )
    .unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(
        temp_dir.path(),
        &[PathBuf::from("type-fact.mlg")],
        &mut event_log,
    );

    assert_eq!(result.files_checked, 1);
    assert_eq!(
        user_events(&event_log),
        [Event::user_log("Checked 1 file").with_origin("mlg_check")]
    );
}

#[test]
fn check_reports_command_when_requirement_type_mismatches() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("type-mismatch.mlg");

    fs::write(
        &file,
        r#"[\real]
Describes: x
Documented:
. called: "real"

[\foo{s}]
Describes: x
when: s is \real
Documented:
. called: "foo"

Theorem:
then:
. \foo{r}
"#,
    )
    .unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(
        temp_dir.path(),
        &[PathBuf::from("type-mismatch.mlg")],
        &mut event_log,
    );

    assert_eq!(result.files_checked, 1);
    assert!(user_events(&event_log).iter().any(|event| {
        event.as_message().is_some_and(|message| {
            message.message == "Could not prove requirement `r is \\real` for command `\\foo`"
        })
    }));
    assert!(event_log.has_errors());
}

#[test]
fn check_reports_type_argument_mismatches_in_is_statements_and_predicates() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("function-argument-types.mlg");

    fs::write(
        &file,
        r#"[\set]
Describes: X
Provides:
. symbol: x_ "in" X :-> \\abstract
Documented:
. called: "set"

[\function:on{A}:to{B}]
Describes: f(x__)
when: A, B is \set
satisfies:
. forAll: x "in" A
  then:
  . existsUnique: y "in" B
    suchThat: f(x) = y
Documented:
. called: "function on $A?$ to $B?$"
. written: "f? \: : \: A? \rightarrow B?"

Theorem:
given:
. X, Y is \set
. x "in" X
. g is \function:on{X}:to{Y}
. h is \function:on{g}:to{x}
then:
. g is? \function:on{X}:to{g}
"#,
    )
    .unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(
        temp_dir.path(),
        &[PathBuf::from("function-argument-types.mlg")],
        &mut event_log,
    );

    assert_eq!(result.files_checked, 1);
    let messages = user_events(&event_log)
        .iter()
        .filter_map(Event::as_message)
        .map(|event| event.message.clone())
        .collect::<Vec<_>>();
    assert!(messages.contains(&String::from(
        "Could not prove requirement `g is \\set` for command `\\function:on:to`"
    )));
    assert!(messages.contains(&String::from(
        "Could not prove requirement `x is \\set` for command `\\function:on:to`"
    )));
    let canonical_file = file.canonicalize().unwrap();
    assert!(has_user_error_at(
        &event_log,
        &canonical_file,
        26,
        8,
        "Could not prove requirement `g is \\set` for command `\\function:on:to`"
    ));
    assert!(has_user_error_at(
        &event_log,
        &canonical_file,
        24,
        7,
        "Could not prove requirement `x is \\set` for command `\\function:on:to`"
    ));
    assert!(event_log.has_errors());
}

#[test]
fn check_reports_unrecognized_symbols_in_command_arguments() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("unrecognized-symbol.mlg");

    fs::write(
        &file,
        r#"[\set]
Describes: X
Documented:
. called: "set"

[\function:on{A}:to{B}]
Describes: f(x__)
when: A, B is \set
Documented:
. called: "function"

Theorem:
given:
. X, Y is \set
. f is \function:on{X}:to{Y}
then:
. f is? \function:on{X}:to{Z}
"#,
    )
    .unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(
        temp_dir.path(),
        &[PathBuf::from("unrecognized-symbol.mlg")],
        &mut event_log,
    );

    assert_eq!(result.files_checked, 1);
    assert!(has_user_error_at(
        &event_log,
        &file.canonicalize().unwrap(),
        16,
        27,
        "Unrecognized symbol `Z`"
    ));
    assert!(event_log.has_errors());
}

#[test]
fn check_uses_local_bindings_when_matching_types() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("type-binding.mlg");

    fs::write(
        &file,
        r#"[\real]
Describes: x
Documented:
. called: "real"

[\foo{s}]
Describes: x
when: s is \real
Documented:
. called: "foo"

Theorem:
given: A is \real
where:
. A := B
then:
. \foo{B}
"#,
    )
    .unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(
        temp_dir.path(),
        &[PathBuf::from("type-binding.mlg")],
        &mut event_log,
    );

    assert_eq!(result.files_checked, 1);
    assert_eq!(
        user_events(&event_log),
        [Event::user_log("Checked 1 file").with_origin("mlg_check")]
    );
}

#[test]
fn check_uses_quantifier_bindings_when_matching_types() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("exists-binding.mlg");

    fs::write(
        &file,
        r#"[\real]
Describes: x
Documented:
. called: "real"

[\foo{s}]
Describes: x
when: s is \real
Documented:
. called: "foo"

Theorem:
then:
. exists: A := B
  suchThat:
  . A is \real
  . \foo{B}
"#,
    )
    .unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(
        temp_dir.path(),
        &[PathBuf::from("exists-binding.mlg")],
        &mut event_log,
    );

    assert_eq!(result.files_checked, 1);
    assert_eq!(
        user_events(&event_log),
        [Event::user_log("Checked 1 file").with_origin("mlg_check")]
    );
}

#[test]
fn check_reduces_spec_operator_aliases_to_type_facts() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("spec-reduction.mlg");

    fs::write(
        &file,
        r#"[\real]
Describes: x
Documented:
. called: "real"

[\reals]
Describes: R
Provides:
. symbol: x_ "in" R :-> x is \real
Documented:
. called: "reals"

[\foo{s}]
Describes: x
when: s is \real
Documented:
. called: "foo"

Theorem:
given:
. S is \reals
. r "in" S
then:
. \foo{r}
"#,
    )
    .unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(
        temp_dir.path(),
        &[PathBuf::from("spec-reduction.mlg")],
        &mut event_log,
    );

    assert_eq!(result.files_checked, 1);
    assert_eq!(
        user_events(&event_log),
        [Event::user_log("Checked 1 file").with_origin("mlg_check")]
    );
}

#[test]
fn check_matches_spec_requirements_without_reducing_to_type_facts() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("direct-spec.mlg");

    fs::write(
        &file,
        r#"[\group]
Describes: G
Documented:
. called: "group"

[\foo{G}:with{x}]
Describes: y
when:
. G is \group
. x "in" G
Documented:
. called: "foo"

Theorem:
given:
. H is \group
. y "in" H
. z is \foo{H}:with{y}
then:
. z = z
"#,
    )
    .unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(
        temp_dir.path(),
        &[PathBuf::from("direct-spec.mlg")],
        &mut event_log,
    );

    assert_eq!(result.files_checked, 1);
    assert_eq!(
        user_events(&event_log),
        [Event::user_log("Checked 1 file").with_origin("mlg_check")]
    );
}

#[test]
fn check_reports_command_references_with_wrong_curly_argument_count() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("wrong-curly-count.mlg");

    fs::write(
        &file,
        r#"[\some.function{A}(x, y)]
Defines: A "defines" B
Documented:
. [docs.called]
  called:
  . "some function"

Theorem:
then:
. f is \some.function{A, B}(x, y)
"#,
    )
    .unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(
        temp_dir.path(),
        &[PathBuf::from("wrong-curly-count.mlg")],
        &mut event_log,
    );

    assert_eq!(result.files_checked, 1);
    assert!(has_user_error_at(
        &event_log,
        &file.canonicalize().unwrap(),
        9,
        7,
        "Command signature `\\some.function` expects argument shape `{1}(2)` but found `{2}(2)`"
    ));
    assert!(event_log.has_errors());
}

#[test]
fn check_reports_command_references_with_wrong_paren_argument_count() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("wrong-paren-count.mlg");

    fs::write(
        &file,
        r#"[\some.function{A}(x, y)]
Defines: A "defines" B
Documented:
. [docs.called]
  called:
  . "some function"

Theorem:
then:
. f is \some.function{A}(x, y, z)
"#,
    )
    .unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(
        temp_dir.path(),
        &[PathBuf::from("wrong-paren-count.mlg")],
        &mut event_log,
    );

    assert_eq!(result.files_checked, 1);
    assert!(has_user_error_at(
        &event_log,
        &file.canonicalize().unwrap(),
        9,
        7,
        "Command signature `\\some.function` expects argument shape `{1}(2)` but found `{1}(3)`"
    ));
    assert!(event_log.has_errors());
}

#[test]
fn check_requires_defines_describes_and_refines_to_have_documented_called() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("documented-called.mlg");

    fs::write(
        &file,
        r#"[\missing.called]
Defines: A "defines" B

[\written.only]
Describes: A
Documented:
. [docs.written]
  written:
  . "written only"

[\refines.missing]
Refines: A is \missing.called
"#,
    )
    .unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(
        temp_dir.path(),
        &[PathBuf::from("documented-called.mlg")],
        &mut event_log,
    );
    let canonical_file = file.canonicalize().unwrap();

    assert_eq!(result.files_checked, 1);
    assert!(has_user_error_at(
        &event_log,
        &canonical_file,
        0,
        1,
        "Defines entries must include a `called:` item in `Documented:`"
    ));
    assert!(has_user_error_at(
        &event_log,
        &canonical_file,
        3,
        1,
        "Describes entries must include a `called:` item in `Documented:`"
    ));
    assert!(has_user_error_at(
        &event_log,
        &canonical_file,
        10,
        1,
        "Refines entries must include a `called:` item in `Documented:`"
    ));
    assert!(event_log.has_errors());
}

#[test]
fn check_accepts_documented_called_with_or_without_written() {
    let temp_dir = TestDir::new();
    let file = temp_dir.path().join("documented-valid.mlg");

    fs::write(
        &file,
        r#"[\called.only]
Defines: A "defines" B
Documented:
. [docs.called]
  called:
  . "called only"

[\called.and.written]
Describes: A
Documented:
. [docs.called]
  called:
  . "called and written"
  written:
  . "written"
"#,
    )
    .unwrap();

    let mut event_log = EventLog::new();
    let result = check_in(
        temp_dir.path(),
        &[PathBuf::from("documented-valid.mlg")],
        &mut event_log,
    );

    assert_eq!(result.files_checked, 1);
    assert_eq!(
        user_events(&event_log),
        [Event::user_log("Checked 1 file").with_origin("mlg_check")]
    );
}

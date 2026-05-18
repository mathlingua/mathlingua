use super::*;

#[test]
fn formulation_golden_files_match_parsers() {
    let directory = formulation_golden_directory();
    let parsers = formulation_golden_parsers();
    let files = read_test_files(directory, "txt");

    assert!(!files.is_empty(), "expected formulation golden files");

    let actual_names = files
        .iter()
        .map(|path| file_name(path))
        .collect::<BTreeSet<_>>();
    let expected_names = parsers
        .keys()
        .map(|name| (*name).to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual_names, expected_names,
        "unexpected formulation golden files"
    );
}

#[test]
fn formulation_golden_author_header() {
    run_formulation_golden("author_header.txt");
}

#[test]
fn formulation_golden_command_header() {
    run_formulation_golden("command_header.txt");
}

#[test]
fn formulation_golden_expression() {
    run_formulation_golden("expression.txt");
}

#[test]
fn formulation_golden_expression_alias() {
    run_formulation_golden("expression_alias.txt");
}

#[test]
fn formulation_golden_form_or_declaration() {
    run_formulation_golden("form_or_declaration.txt");
}

#[test]
fn formulation_golden_is_or_refined_statement_spec() {
    run_formulation_golden("is_or_refined_statement_spec.txt");
}

#[test]
fn formulation_golden_is_or_spec() {
    run_formulation_golden("is_or_spec.txt");
}

#[test]
fn formulation_golden_is_via_statement() {
    run_formulation_golden("is_via_statement.txt");
}

#[test]
fn formulation_golden_label_header() {
    run_formulation_golden("label_header.txt");
}

#[test]
fn formulation_golden_resource_header() {
    run_formulation_golden("resource_header.txt");
}

#[test]
fn formulation_golden_spec_operator_alias() {
    run_formulation_golden("spec_operator_alias.txt");
}

#[test]
fn formulation_golden_writing_alias() {
    run_formulation_golden("writing_alias.txt");
}

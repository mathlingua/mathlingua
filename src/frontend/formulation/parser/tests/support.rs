fn split_test_chunks(text: &str) -> Vec<String> {
    text.replace("\r\n", "\n")
        .split("\n\n")
        .filter_map(|entry| {
            let entry = entry.trim();
            (!entry.is_empty()).then(|| entry.to_owned())
        })
        .collect()
}

fn read_test_chunks(path: &Path) -> Vec<String> {
    let text = fs::read_to_string(path).unwrap_or_else(|error| {
        panic!(
            "expected formulation golden file {}: {error}",
            path.display()
        )
    });
    split_test_chunks(&text)
}

fn read_test_files(directory: &Path, extension: &str) -> Vec<PathBuf> {
    let mut files = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("expected directory {}: {error}", directory.display()))
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some(extension))
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .and_then(|value| value.to_str())
        .expect("expected valid utf-8 file name")
        .to_owned()
}

fn parse_expression_entry(input: &str) -> Result<(), String> {
    parse_expression(input)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn parse_form_or_declaration_entry(input: &str) -> Result<(), String> {
    parse_form_or_declaration(input)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn parse_is_or_spec_entry(input: &str) -> Result<(), String> {
    parse_is_or_spec(input)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn parse_is_or_refined_statement_spec_entry(input: &str) -> Result<(), String> {
    parse_is_or_refined_statement_spec(input)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn parse_is_via_statement_entry(input: &str) -> Result<(), String> {
    parse_is_via_statement(input)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn parse_command_header_entry(input: &str) -> Result<(), String> {
    parse_command_header(input)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn parse_writing_alias_entry(input: &str) -> Result<(), String> {
    parse_writing_alias(input)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn parse_expression_alias_entry(input: &str) -> Result<(), String> {
    parse_expression_alias(input)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn parse_spec_operator_alias_entry(input: &str) -> Result<(), String> {
    parse_spec_operator_alias(input)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn parse_label_header_entry(input: &str) -> Result<(), String> {
    parse_label_header(input)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn parse_author_header_entry(input: &str) -> Result<(), String> {
    parse_author_header(input)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn parse_resource_header_entry(input: &str) -> Result<(), String> {
    parse_resource_header(input)
        .map(|_| ())
        .map_err(|error| error.to_string())
}

fn formulation_golden_parsers() -> BTreeMap<&'static str, fn(&str) -> Result<(), String>> {
    BTreeMap::from([
        (
            "author_header.txt",
            parse_author_header_entry as fn(&str) -> Result<(), String>,
        ),
        ("command_header.txt", parse_command_header_entry),
        ("expression.txt", parse_expression_entry),
        ("expression_alias.txt", parse_expression_alias_entry),
        ("form_or_declaration.txt", parse_form_or_declaration_entry),
        (
            "is_or_refined_statement_spec.txt",
            parse_is_or_refined_statement_spec_entry,
        ),
        ("is_or_spec.txt", parse_is_or_spec_entry),
        ("is_via_statement.txt", parse_is_via_statement_entry),
        ("label_header.txt", parse_label_header_entry),
        ("resource_header.txt", parse_resource_header_entry),
        ("spec_operator_alias.txt", parse_spec_operator_alias_entry),
        ("writing_alias.txt", parse_writing_alias_entry),
    ])
}

fn formulation_golden_directory() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/goldens/formulation"))
}

fn run_formulation_golden(filename: &str) {
    let directory = formulation_golden_directory();
    let parsers = formulation_golden_parsers();
    let path = directory.join(filename);
    let parser = parsers
        .get(filename)
        .unwrap_or_else(|| panic!("no parser configured for {}", path.display()));
    let entries = read_test_chunks(&path);

    assert!(!entries.is_empty(), "expected cases in {}", path.display());

    for (index, entry) in entries.iter().enumerate() {
        parser(entry).unwrap_or_else(|error| {
            panic!(
                "failed to parse formulation golden case {} chunk {}: {}\n\n{}",
                filename,
                index + 1,
                error,
                entry
            )
        });
    }
}

fn assert_simple_command_header(
    input: &str,
    expected_chain_parts: &[&str],
    expected_head_args: usize,
    expected_tail: usize,
    expected_paren_args: usize,
) {
    let header = parse_command_header(input).expect("expected command header");

    match header {
        CommandHeader::Command(CommandHeaderNode {
            chain,
            head_args,
            tail,
            paren_args,
            ..
        }) => {
            assert_eq!(chain.parts.len(), expected_chain_parts.len());
            for (part, expected) in chain.parts.iter().zip(expected_chain_parts) {
                assert!(matches!(part, ChainPart::Name(name) if name == expected));
            }
            assert_eq!(head_args.len(), expected_head_args);
            assert_eq!(tail.len(), expected_tail);
            assert_eq!(paren_args.len(), expected_paren_args);
        }
        other => panic!("expected simple command header, got {other:?}"),
    }
}

fn assert_command_expression(
    input: &str,
    expected_chain_parts: &[&str],
    expected_head_args: usize,
    expected_tail: usize,
    expected_paren_args: usize,
) {
    let expression = parse_expression(input).expect("expected command expression");

    match expression.kind {
        ExpressionKind::Command(command) => {
            assert_eq!(command.chain.parts.len(), expected_chain_parts.len());
            for (part, expected) in command.chain.parts.iter().zip(expected_chain_parts) {
                assert!(matches!(part, ChainPart::Name(name) if name == expected));
            }
            assert_eq!(command.head_args.len(), expected_head_args);
            assert_eq!(command.tail.len(), expected_tail);
            assert_eq!(command.paren_args.len(), expected_paren_args);
        }
        other => panic!("expected command expression, got {other:?}"),
    }
}


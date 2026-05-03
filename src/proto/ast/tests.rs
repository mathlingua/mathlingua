use super::{Argument, Formulation, Group, Line, Metadata, Section, TextLiteral};

fn metadata(indent: usize, has_dot: bool) -> Metadata {
    Metadata {
        row: 0,
        indent,
        has_dot,
    }
}

#[test]
fn classifies_blank_comments_text_and_headers() {
    let blank = Line {
        text: String::new(),
        metadata: metadata(0, false),
    };
    assert!(blank.is_blank());
    assert!(blank.is_blank_or_comment());
    assert!(!blank.is_comment());
    assert!(!blank.is_text());
    assert!(!blank.is_header());

    let comment = Line {
        text: "-- note".to_string(),
        metadata: metadata(0, false),
    };
    assert!(comment.is_comment());
    assert!(comment.is_blank_or_comment());
    assert!(!comment.is_text());
    assert!(!comment.is_header());

    let text = Line {
        text: "\"hello\"".to_string(),
        metadata: metadata(0, false),
    };
    assert!(text.is_text());
    assert!(!text.is_blank_or_comment());
    assert!(!text.is_header());

    let header = Line {
        text: "[group]".to_string(),
        metadata: metadata(0, false),
    };
    assert!(header.is_header());
    assert!(!header.is_blank_or_comment());
    assert!(!header.is_text());
}

#[test]
fn display_saturates_dot_indentation_when_indent_is_smaller_than_prefix_width() {
    let line = Line {
        text: "x".to_string(),
        metadata: metadata(1, true),
    };

    assert_eq!(line.to_string(), ". x");
}

#[test]
fn displays_sections_and_groups_with_nested_arguments() {
    let group = Group {
        heading: Some("heading".to_string()),
        sections: vec![
            Section {
                label: "Defines".to_string(),
                inline_argument: Some("f(x_)".to_string()),
                arguments: vec![],
                metadata: metadata(0, false),
            },
            Section {
                label: "when".to_string(),
                inline_argument: None,
                arguments: vec![
                    Argument::Formulation(Formulation {
                        text: "x in A".to_string(),
                        metadata: metadata(2, true),
                    }),
                    Argument::Text(TextLiteral {
                        text: "\"note\"".to_string(),
                        metadata: metadata(2, true),
                    }),
                    Argument::Group(Group {
                        heading: None,
                        sections: vec![Section {
                            label: "exists".to_string(),
                            inline_argument: Some("z".to_string()),
                            arguments: vec![],
                            metadata: metadata(2, true),
                        }],
                        metadata: metadata(2, true),
                    }),
                ],
                metadata: metadata(0, false),
            },
        ],
        metadata: metadata(0, false),
    };

    assert_eq!(
        group.to_string(),
        "[heading]\nDefines: f(x_)\nwhen:\n. x in A\n. \"note\"\n. exists: z"
    );
}

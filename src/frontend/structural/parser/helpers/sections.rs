/// Returns the first section label of a proto group.
///
/// Structural group dispatch is label-first, so groups without sections cannot
/// be recognized.
fn first_section_label(group: &ProtoGroup) -> Option<&str> {
    group.sections.first().map(|section| section.label.as_str())
}

/// Looks up an identified section by label.
///
/// The section map stores borrowed proto sections keyed by their normalized
/// expected label.
fn section<'a>(
    sections: &'a HashMap<String, &'a ProtoSection>,
    label: &str,
) -> Option<&'a ProtoSection> {
    sections.get(label).copied()
}

/// Validates section order and presence for a structural group.
///
/// The `expected` slice is an ordered pattern where labels ending in `?` are
/// optional.  The returned map contains only sections that were present and
/// accepted.  Diagnostics include the full expected pattern to make authoring
/// mistakes easier to repair.
fn identify_sections<'a>(
    name: &str,
    sections: &'a [ProtoSection],
    tracker: &mut EventLog,
    expected: &[&str],
) -> Option<HashMap<String, &'a ProtoSection>> {
    let mut section_queue: VecDeque<&ProtoSection> = sections.iter().collect();
    let mut expected_queue: VecDeque<&str> = expected.iter().copied().collect();
    let mut result = HashMap::new();

    let pattern = expected
        .iter()
        .map(|value| format!("{value}:"))
        .collect::<Vec<_>>()
        .join("\n");

    while let (Some(next_section), Some(maybe_name)) = (
        section_queue.front().copied(),
        expected_queue.front().copied(),
    ) {
        let is_optional = maybe_name.ends_with('?');
        let true_name = maybe_name.trim_end_matches('?');

        if next_section.label == true_name {
            result.insert(true_name.to_owned(), next_section);
            section_queue.pop_front();
            expected_queue.pop_front();
        } else if is_optional {
            expected_queue.pop_front();
        } else {
            tracker.user_error_at_row(
                Some(ORIGIN),
                next_section.metadata.row,
                format!(
                    "For {name} pattern:\n\n{pattern}\n\nExpected `{true_name}` but found `{}`",
                    next_section.label
                ),
            );
            return None;
        }
    }

    if let Some(unexpected) = section_queue.front() {
        tracker.user_error_at_row(
            Some(ORIGIN),
            unexpected.metadata.row,
            format!(
                "For {name} pattern:\n\n{pattern}\n\nUnexpected section `{}`",
                unexpected.label
            ),
        );
        return None;
    }

    if let Some(missing) = expected_queue
        .iter()
        .find(|name| !name.ends_with('?'))
        .copied()
    {
        let row = sections
            .first()
            .map(|section| section.metadata.row)
            .unwrap_or(0);
        tracker.user_error_at_row(
            Some(ORIGIN),
            row,
            format!(
                "For {name} pattern:\n\n{pattern}\n\nExpected section `{}`",
                missing.trim_end_matches('?')
            ),
        );
        return None;
    }

    Some(result)
}

/// One flattened entry from a proto section body.
///
/// Section entries unify inline arguments with body arguments so helper parsers
/// can apply the same validation logic regardless of how the author chose to
/// place the section content.
enum SectionEntry<'a> {
    /// Inline text after the section colon.
    Inline { text: &'a str, row: usize },
    /// A formulation body argument.
    Formulation { text: &'a str, row: usize },
    /// A quoted text body argument.
    Text { text: &'a str, row: usize },
    /// A nested proto group body argument.
    Group { group: &'a ProtoGroup, row: usize },
}

/// Flattens a section's inline and body arguments into parseable entries.
///
/// Inline arguments are yielded first using the section row, followed by body
/// arguments in source order with their own rows.
fn section_entries(section: &ProtoSection) -> Vec<SectionEntry<'_>> {
    let mut entries = Vec::new();
    if let Some(argument) = section.inline_argument.as_deref() {
        entries.push(SectionEntry::Inline {
            text: argument,
            row: section.metadata.row,
        });
    }

    for argument in &section.arguments {
        match argument {
            ProtoArgument::Formulation(ProtoFormulation { text, metadata }) => {
                entries.push(SectionEntry::Formulation {
                    text,
                    row: metadata.row,
                });
            }
            ProtoArgument::Text(ProtoText { text, metadata }) => {
                entries.push(SectionEntry::Text {
                    text,
                    row: metadata.row,
                });
            }
            ProtoArgument::Group(group) => {
                entries.push(SectionEntry::Group {
                    group,
                    row: group.metadata.row,
                });
            }
        }
    }

    entries
}


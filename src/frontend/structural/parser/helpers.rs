use super::*;

// ===============================[ sections ]=====================================

/// Returns the first section label of a proto group.
///
/// Structural group dispatch is label-first, so groups without sections cannot
/// be recognized.
pub(in crate::frontend::structural::parser) fn first_section_label(
    group: &ProtoGroup,
) -> Option<&str> {
    group.sections.first().map(|section| section.label.as_str())
}

/// Looks up an identified section by label.
///
/// The section map stores borrowed proto sections keyed by their normalized
/// expected label.
pub(in crate::frontend::structural::parser) fn section<'a>(
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
pub(in crate::frontend::structural::parser) fn identify_sections<'a>(
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
pub(in crate::frontend::structural::parser) enum SectionEntry<'a> {
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
pub(in crate::frontend::structural::parser) fn section_entries(
    section: &ProtoSection,
) -> Vec<SectionEntry<'_>> {
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

// ===============================[ groups ]=====================================

/// Parses one or more required nested groups from a section.
///
/// The supplied parser determines which nested group kinds are legal in the
/// section, letting this helper centralize cardinality and non-group diagnostics.
pub(in crate::frontend::structural::parser) fn parse_required_groups<T>(
    section: &ProtoSection,
    label: &str,
    tracker: &mut EventLog,
    parser: fn(&ProtoGroup, &mut EventLog) -> Option<T>,
) -> Option<OneOrMore<T>> {
    let starting_issue_count = tracker.issue_count();
    let items = parse_optional_groups(Some(section), label, tracker, parser);
    one_or_more(items, || {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(
                Some(ORIGIN),
                section.metadata.row,
                format!("Expected nested groups in `{label}`"),
            );
        }
    })
}

/// Parses zero or more nested groups from an optional section.
///
/// Non-group entries are reported because this helper is used only for sections
/// whose grammar requires group-shaped items.
pub(in crate::frontend::structural::parser) fn parse_optional_groups<T>(
    section: Option<&ProtoSection>,
    label: &str,
    tracker: &mut EventLog,
    parser: fn(&ProtoGroup, &mut EventLog) -> Option<T>,
) -> ZeroOrMore<T> {
    let Some(section) = section else {
        return ZeroOrMore::default();
    };

    let mut items = Vec::new();
    for entry in section_entries(section) {
        match entry {
            SectionEntry::Group { group, .. } => {
                if let Some(item) = parser(group, tracker) {
                    items.push(item);
                }
            }
            SectionEntry::Inline { row, .. }
            | SectionEntry::Formulation { row, .. }
            | SectionEntry::Text { row, .. } => {
                tracker.user_error_at_row(
                    Some(ORIGIN),
                    row,
                    format!("Expected nested group in section `{label}`"),
                );
            }
        }
    }

    items.into()
}

// ===============================[ headings ]=====================================

/// Parses an alias body into the supported alias variants.
///
/// Expression aliases are attempted first because their left-hand side grammar
/// is broader; if that fails the body is parsed as a specification-operator
/// alias.
pub(in crate::frontend::structural::parser) fn parse_alias_kind(
    input: &str,
) -> Result<AliasKind, FormulationParseError> {
    if let Ok(alias) = parse_expression_alias(input) {
        return Ok(AliasKind::Expression(alias));
    }
    parse_spec_operator_alias(input).map(AliasKind::SpecOperator)
}

/// Parses an item accepted by `extends:` and related sections.
///
/// `is ... via ...` is more specific, so it is attempted before the broader
/// `is`/spec parser.
pub(in crate::frontend::structural::parser) fn parse_is_or_via_item(
    input: &str,
) -> Result<IsOrViaItem, FormulationParseError> {
    if let Ok(item) = parse_is_via_statement(input) {
        return Ok(IsOrViaItem::IsVia(item));
    }
    parse_is_or_spec(input).map(IsOrViaItem::IsOrSpec)
}

/// Parses a quantifier binding or ordinary specification.
pub(in crate::frontend::structural::parser) fn parse_binding_or_spec(
    input: &str,
) -> Result<BindingOrSpec, FormulationParseError> {
    if let Ok(binding) = parse_expression_binding(input) {
        return Ok(BindingOrSpec::Binding(binding));
    }
    parse_is_or_spec(input).map(BindingOrSpec::IsOrSpec)
}

/// Parses a required command heading from a proto group.
///
/// Missing or malformed headings are reported at the group row because headings
/// live on the group header line rather than in a section body.
pub(in crate::frontend::structural::parser) fn parse_required_command_heading(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<crate::frontend::formulation::ast::CommandHeader> {
    let Some(heading) = group.heading.as_deref() else {
        tracker.user_error_at_row(Some(ORIGIN), group.metadata.row, "Expected command heading");
        return None;
    };
    match parse_command_header(heading) {
        Ok(heading) => Some(heading),
        Err(error) => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Invalid command heading: {error}"),
            );
            None
        }
    }
}

/// Parses an optional command heading from a proto group.
///
/// `None` means the group has no heading; an invalid present heading prevents
/// construction of the enclosing structural group.
pub(in crate::frontend::structural::parser) fn parse_optional_command_heading(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<Option<crate::frontend::formulation::ast::CommandHeader>> {
    match group.heading.as_deref() {
        Some(heading) => match parse_command_header(heading) {
            Ok(heading) => Some(Some(heading)),
            Err(error) => {
                tracker.user_error_at_row(
                    Some(ORIGIN),
                    group.metadata.row,
                    format!("Invalid command heading: {error}"),
                );
                None
            }
        },
        None => Some(None),
    }
}

/// Parses an optional label heading from a nested proto group.
///
/// Label headings are used for local documentation/proof notes and are
/// syntactically distinct from command headings.
pub(in crate::frontend::structural::parser) fn parse_optional_label_heading(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<Option<crate::frontend::formulation::ast::LabelHeader>> {
    match group.heading.as_deref() {
        Some(heading) => match parse_label_header(heading) {
            Ok(heading) => Some(Some(heading)),
            Err(error) => {
                tracker.user_error_at_row(
                    Some(ORIGIN),
                    group.metadata.row,
                    format!("Invalid label heading: {error}"),
                );
                None
            }
        },
        None => Some(None),
    }
}

/// Parses a required author heading from a `Person:` group.
pub(in crate::frontend::structural::parser) fn parse_required_author_heading(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<crate::frontend::formulation::ast::AuthorHeader> {
    let Some(heading) = group.heading.as_deref() else {
        tracker.user_error_at_row(Some(ORIGIN), group.metadata.row, "Expected author heading");
        return None;
    };
    match parse_author_header(heading) {
        Ok(heading) => Some(heading),
        Err(error) => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Invalid author heading: {error}"),
            );
            None
        }
    }
}

/// Parses a required resource heading from a `Resource:` group.
pub(in crate::frontend::structural::parser) fn parse_required_resource_heading(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<crate::frontend::formulation::ast::ResourceHeader> {
    let Some(heading) = group.heading.as_deref() else {
        tracker.user_error_at_row(
            Some(ORIGIN),
            group.metadata.row,
            "Expected resource heading",
        );
        return None;
    };
    match parse_resource_header(heading) {
        Ok(heading) => Some(heading),
        Err(error) => {
            tracker.user_error_at_row(
                Some(ORIGIN),
                group.metadata.row,
                format!("Invalid resource heading: {error}"),
            );
            None
        }
    }
}

/// Ensures a group has no bracket heading.
///
/// Outline and metadata groups derive their identity from sections only; a
/// heading on those groups is almost certainly an authoring mistake.
pub(in crate::frontend::structural::parser) fn ensure_no_heading(
    group: &ProtoGroup,
    tracker: &mut EventLog,
) -> Option<()> {
    if let Some(heading) = &group.heading {
        tracker.user_error_at_row(
            Some(ORIGIN),
            group.metadata.row,
            format!("Unexpected heading `{heading}`"),
        );
        None
    } else {
        Some(())
    }
}

// ===============================[ clauses ]=====================================

/// Parses exactly one required clause from a section.
///
/// This is used for section shapes such as `expresses:` and `not:` where the
/// language grammar expects one logical clause.
pub(in crate::frontend::structural::parser) fn parse_required_clause(
    section: &ProtoSection,
    label: &str,
    tracker: &mut EventLog,
) -> Option<Clause> {
    let starting_issue_count = tracker.issue_count();
    let clauses = parse_optional_clauses(Some(section), label, tracker);
    if clauses.is_empty() {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(
                Some(ORIGIN),
                section.metadata.row,
                format!("Expected a clause in `{label}`"),
            );
        }
        None
    } else {
        Some(clauses.into_iter().next().expect("non-empty clauses"))
    }
}

/// Parses one or more required clauses from a section.
pub(in crate::frontend::structural::parser) fn parse_required_clauses(
    section: &ProtoSection,
    label: &str,
    tracker: &mut EventLog,
) -> Option<OneOrMore<Clause>> {
    let starting_issue_count = tracker.issue_count();
    let clauses = parse_optional_clauses(Some(section), label, tracker);
    one_or_more(clauses, || {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(
                Some(ORIGIN),
                section.metadata.row,
                format!("Expected clauses in `{label}`"),
            );
        }
    })
}

/// Parses zero or more clauses from an optional section.
///
/// Inline formulations are parsed first as expressions and then as `is`/spec
/// statements, while nested groups are dispatched through clause-group parsers.
pub(in crate::frontend::structural::parser) fn parse_optional_clauses(
    section: Option<&ProtoSection>,
    label: &str,
    tracker: &mut EventLog,
) -> ZeroOrMore<Clause> {
    let Some(section) = section else {
        return ZeroOrMore::default();
    };

    let mut result = Vec::new();
    for entry in section_entries(section) {
        match entry {
            SectionEntry::Inline { text, row } | SectionEntry::Formulation { text, row } => {
                if let Ok(binding) = parse_expression_binding(text) {
                    result.push(Clause::Binding(binding));
                    continue;
                }

                match parse_expression(text) {
                    Ok(expression) => result.push(Clause::Expression(expression)),
                    Err(expression_error) => match parse_is_or_spec(text) {
                        Ok(spec) => result.push(Clause::IsOrSpec(spec)),
                        Err(_) => tracker.user_error_at_row(
                            Some(ORIGIN),
                            row,
                            format!("Invalid clause expression in `{label}`: {expression_error}"),
                        ),
                    },
                }
            }
            SectionEntry::Group { group, .. } => {
                if let Some(clause) = parse_clause_group(group, tracker) {
                    result.push(clause);
                }
            }
            SectionEntry::Text { row, .. } => {
                tracker.user_error_at_row(
                    Some(ORIGIN),
                    row,
                    format!("Expected clause in section `{label}`"),
                );
            }
        }
    }

    result.into()
}

// ===============================[ formulations ]=====================================

/// Parses exactly one required formulation from a section.
///
/// The parser function is supplied by the caller so this helper can parse
/// expressions, command headings, labels, resources, or other formulation
/// fragments while sharing diagnostics and cardinality checks.
pub(in crate::frontend::structural::parser) fn parse_required_formulation<T>(
    section: &ProtoSection,
    label: &str,
    tracker: &mut EventLog,
    parser: fn(&str) -> Result<T, FormulationParseError>,
) -> Option<T> {
    let starting_issue_count = tracker.issue_count();
    let items = parse_optional_formulations(Some(section), label, tracker, parser);
    if items.is_empty() {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(
                Some(ORIGIN),
                section.metadata.row,
                format!("Expected a {label} formulation"),
            );
        }
        None
    } else {
        Some(items.into_iter().next().expect("non-empty formulations"))
    }
}

/// Parses one or more required formulations from a section.
///
/// If parsing produced no items and no more specific issue was emitted, this
/// helper reports a missing-content diagnostic for the whole section.
pub(in crate::frontend::structural::parser) fn parse_required_formulations<T>(
    section: &ProtoSection,
    label: &str,
    tracker: &mut EventLog,
    parser: fn(&str) -> Result<T, FormulationParseError>,
) -> Option<OneOrMore<T>> {
    let starting_issue_count = tracker.issue_count();
    let items = parse_optional_formulations(Some(section), label, tracker, parser);
    one_or_more(items, || {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(
                Some(ORIGIN),
                section.metadata.row,
                format!("Expected {label} formulations"),
            );
        }
    })
}

/// Parses zero or more formulations from an optional section.
///
/// Inline section arguments and formulation arguments are accepted.  Text and
/// nested groups are diagnosed because callers requested formulation content.
pub(in crate::frontend::structural::parser) fn parse_optional_formulations<T>(
    section: Option<&ProtoSection>,
    label: &str,
    tracker: &mut EventLog,
    parser: fn(&str) -> Result<T, FormulationParseError>,
) -> ZeroOrMore<T> {
    let Some(section) = section else {
        return ZeroOrMore::default();
    };

    let mut result = Vec::new();
    for entry in section_entries(section) {
        match entry {
            SectionEntry::Inline { text, row } | SectionEntry::Formulation { text, row } => {
                match parser(text) {
                    Ok(value) => result.push(value),
                    Err(error) => tracker.user_error_at_row(
                        Some(ORIGIN),
                        row,
                        format!("Invalid {label} formulation: {error}"),
                    ),
                }
            }
            SectionEntry::Text { row, .. } => {
                tracker.user_error_at_row(
                    Some(ORIGIN),
                    row,
                    format!("Expected formulation in section `{label}`"),
                );
            }
            SectionEntry::Group { row, .. } => {
                tracker.user_error_at_row(
                    Some(ORIGIN),
                    row,
                    format!("Expected formulation in section `{label}`"),
                );
            }
        }
    }

    result.into()
}

// ===============================[ text ]=====================================

/// Parses exactly one required quoted open-text entry.
///
/// Open-text sections accept inline quoted arguments and quoted text arguments,
/// with quote stripping handled by the shared text parser.
pub(in crate::frontend::structural::parser) fn parse_required_open_text(
    section: &ProtoSection,
    label: &str,
    tracker: &mut EventLog,
) -> Option<OpenText> {
    let starting_issue_count = tracker.issue_count();
    let texts = parse_optional_open_texts(Some(section), tracker);
    if texts.is_empty() {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(
                Some(ORIGIN),
                section.metadata.row,
                format!("Expected text in `{label}`"),
            );
        }
        None
    } else {
        Some(texts.into_iter().next().expect("non-empty texts"))
    }
}

/// Parses one or more required quoted open-text entries.
pub(in crate::frontend::structural::parser) fn parse_required_open_texts(
    section: &ProtoSection,
    label: &str,
    tracker: &mut EventLog,
) -> Option<OneOrMore<OpenText>> {
    let starting_issue_count = tracker.issue_count();
    let texts = parse_optional_open_texts(Some(section), tracker);
    one_or_more(texts, || {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(
                Some(ORIGIN),
                section.metadata.row,
                format!("Expected text entries in `{label}`"),
            );
        }
    })
}

/// Parses zero or more open-text entries from an optional section.
///
/// Missing sections become an empty wrapper, which lets callers model optional
/// prose without conflating it with malformed text in a present section.
pub(in crate::frontend::structural::parser) fn parse_optional_open_texts(
    section: Option<&ProtoSection>,
    tracker: &mut EventLog,
) -> ZeroOrMore<OpenText> {
    parse_optional_texts(section, tracker, OpenText)
}

/// Parses one or more required `WrittenText` entries.
///
/// The structural parser only validates the quoted text shape; LaTeX mode and
/// substitution semantics are handled later by the view/backend layers.
pub(in crate::frontend::structural::parser) fn parse_required_written_texts(
    section: &ProtoSection,
    tracker: &mut EventLog,
) -> Option<OneOrMore<WrittenText>> {
    let starting_issue_count = tracker.issue_count();
    let texts = parse_optional_texts(Some(section), tracker, WrittenText);
    one_or_more(texts, || {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(Some(ORIGIN), section.metadata.row, "Expected written text");
        }
    })
}

/// Parses one or more required `CalledText` entries.
///
/// Called text is plain-text rendering metadata, but at this stage it is just
/// quote-stripped and wrapped.
pub(in crate::frontend::structural::parser) fn parse_required_called_texts(
    section: &ProtoSection,
    tracker: &mut EventLog,
) -> Option<OneOrMore<CalledText>> {
    let starting_issue_count = tracker.issue_count();
    let texts = parse_optional_texts(Some(section), tracker, CalledText);
    one_or_more(texts, || {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(Some(ORIGIN), section.metadata.row, "Expected called text");
        }
    })
}

/// Parses one or more required `WritingText` entries.
pub(in crate::frontend::structural::parser) fn parse_required_writing_texts(
    section: &ProtoSection,
    tracker: &mut EventLog,
) -> Option<OneOrMore<WritingText>> {
    let starting_issue_count = tracker.issue_count();
    let texts = parse_optional_texts(Some(section), tracker, WritingText);
    one_or_more(texts, || {
        if tracker.issue_count() == starting_issue_count {
            tracker.user_error_at_row(Some(ORIGIN), section.metadata.row, "Expected writing text");
        }
    })
}

/// Parses quoted text entries from an optional section and wraps them.
///
/// Inline and text arguments are accepted; formulations and nested groups are
/// diagnosed because text sections are intentionally non-formula content.
pub(in crate::frontend::structural::parser) fn parse_optional_texts<T>(
    section: Option<&ProtoSection>,
    tracker: &mut EventLog,
    wrap: fn(String) -> T,
) -> ZeroOrMore<T> {
    let Some(section) = section else {
        return ZeroOrMore::default();
    };

    let mut result = Vec::new();
    for entry in section_entries(section) {
        match entry {
            SectionEntry::Inline { text, row } | SectionEntry::Text { text, row } => {
                if let Some(value) = strip_quoted_text(text) {
                    result.push(wrap(value));
                } else {
                    tracker.user_error_at_row(
                        Some(ORIGIN),
                        row,
                        format!("Expected quoted text, found `{text}`"),
                    );
                }
            }
            SectionEntry::Formulation { row, .. } => {
                tracker.user_error_at_row(Some(ORIGIN), row, "Expected text, found formulation");
            }
            SectionEntry::Group { row, .. } => {
                tracker.user_error_at_row(Some(ORIGIN), row, "Expected text, found nested group");
            }
        }
    }

    result.into()
}

/// Converts a repeated wrapper into a nonempty wrapper or emits a caller-supplied error.
///
/// This keeps the "did a more specific error already happen?" logic at the call
/// site while centralizing the `OneOrMore` conversion.
pub(in crate::frontend::structural::parser) fn one_or_more<T>(
    items: ZeroOrMore<T>,
    on_empty: impl FnOnce(),
) -> Option<OneOrMore<T>> {
    match OneOrMore::try_from(items) {
        Ok(items) => Some(items),
        Err(_) => {
            on_empty();
            None
        }
    }
}

/// Strips one layer of double quotes from text.
///
/// Escapes are not interpreted at this layer; the current structural syntax
/// treats quoted text as the raw inner substring between the first and last
/// quote.
pub(in crate::frontend::structural::parser) fn strip_quoted_text(input: &str) -> Option<String> {
    let input = input.trim();
    let inner = input.strip_prefix('"')?.strip_suffix('"')?;
    Some(inner.to_owned())
}

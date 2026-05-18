/// Walks a document and validates every command-like reference against the registry.
///
/// The traversal produces signature shapes for commands wherever they can appear:
/// formulas, clauses, type expressions, aliases, and theorem-like statements.
/// Source locations are recovered separately from the original text so errors can
/// point at the actual reference token.
fn validate_document_references(
    file: &ParsedSourceFile,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let mut locator = SourceLocator::new(&file.source);
    for item in &file.document.items {
        walk_top_level_item(item, &mut |shape| {
            validate_reference_shape(
                file.path.as_path(),
                locator.locate_reference(shape),
                shape,
                registry,
                event_log,
            );
        });
    }
}

/// Validates a single reference shape for definition existence and argument arity.
///
/// Refined commands can have fallback shapes.  When a composed refined command is
/// not defined directly, those fallbacks allow the checker to validate the base
/// command and individual refinement pieces instead of reporting a premature
/// undefined-signature error.
fn validate_reference_shape(
    path: &Path,
    position: Option<SourcePosition>,
    shape: &SignatureShape,
    registry: &SignatureRegistry,
    event_log: &mut EventLog,
) {
    let Some(definition) = registry.definitions.get(&shape.signature) else {
        if !shape.fallback_shapes.is_empty() {
            for fallback in &shape.fallback_shapes {
                validate_reference_shape(path, position, fallback, registry, event_log);
            }
            return;
        }
        emit_error(
            event_log,
            path,
            position,
            format!("Undefined command signature `{}`", shape.signature),
        );
        return;
    };

    if !argument_groups_match(&definition.shape.arg_groups, &shape.arg_groups) {
        emit_error(
            event_log,
            path,
            position,
            format!(
                "Command signature `{}` expects argument shape `{}` but found `{}`",
                shape.signature,
                format_arg_groups(&definition.shape.arg_groups),
                format_arg_groups(&shape.arg_groups)
            ),
        );
    }
}

/// Compares expected and actual argument groups, honoring optional invocation groups.
///
/// Exact equality is accepted.  A use site may also omit trailing parenthesized
/// groups, because definitions such as `\some.function{A}(x, y)` can be referred
/// to either as the function object `\some.function{A}` or as the invocation
/// `\some.function{A}(x, y)`.
fn argument_groups_match(expected: &[ArgGroupShape], actual: &[ArgGroupShape]) -> bool {
    if expected == actual {
        return true;
    }

    let Some(remaining_expected) = expected.strip_prefix(actual) else {
        return false;
    };

    !remaining_expected.is_empty()
        && remaining_expected
            .iter()
            .all(|group| group.delimiter == ArgDelimiter::Paren)
}


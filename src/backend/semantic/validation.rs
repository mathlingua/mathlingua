use super::*;

pub(super) fn validate_document_references(
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

pub(super) fn validate_reference_shape(
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

pub(super) fn argument_groups_match(expected: &[ArgGroupShape], actual: &[ArgGroupShape]) -> bool {
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

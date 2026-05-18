/// Traverses an expression tree for command references.
///
/// Command-like expressions are visited as signature shapes, and their argument
/// expressions are traversed recursively so nested references are also checked.
fn walk_expression(expression: &Expression, visit: &mut impl FnMut(&SignatureShape)) {
    match &expression.kind {
        ExpressionKind::Name(_) => {}
        ExpressionKind::FunctionCall { arguments, .. } => {
            for argument in arguments {
                walk_expression(argument, visit);
            }
        }
        ExpressionKind::FunctionNamedCall { elements, .. } => {
            for element in elements {
                walk_expression(&element.expression, visit);
            }
        }
        ExpressionKind::Tuple(elements) => {
            for element in elements {
                if let TupleExpressionElement::Expression(expression) = element {
                    walk_expression(expression, visit);
                }
            }
        }
        ExpressionKind::Set(set) => {
            walk_placeholder_form(&set.target, visit);
            if let Some(predicate) = &set.predicate {
                walk_expression(predicate, visit);
            }
            walk_expression(&set.spec.subject, visit);
        }
        ExpressionKind::Grouped { expression, .. } | ExpressionKind::Labeled { expression, .. } => {
            walk_expression(expression, visit);
        }
        ExpressionKind::SubsetCall(_) => {}
        ExpressionKind::Command(command) => {
            let shape = shape_for_command_expression(command);
            visit(&shape);
            walk_command_expression_arguments(command, visit);
        }
        ExpressionKind::InfixCommand {
            left,
            command,
            right,
        } => {
            walk_expression(left, visit);
            let shape = shape_for_infix_command(command);
            visit(&shape);
            walk_infix_command_arguments(command, visit);
            walk_expression(right, visit);
        }
        ExpressionKind::Prefix { expression, .. } => walk_expression(expression, visit),
        ExpressionKind::Binary { left, right, .. } => {
            walk_expression(left, visit);
            walk_expression(right, visit);
        }
        ExpressionKind::SpecStatement(statement) => walk_expression(&statement.subject, visit),
        ExpressionKind::IsPredicate { subject, command }
        | ExpressionKind::IsNotPredicate { subject, command } => {
            walk_expression(subject, visit);
            let shape = shape_for_command_expression(command);
            visit(&shape);
            walk_command_expression_arguments(command, visit);
        }
        ExpressionKind::IsType { subject, ty } => {
            walk_expression(subject, visit);
            walk_type_expression(ty, visit);
        }
    }
}

/// Traverses all expression arguments supplied to a normal command expression.
fn walk_command_expression_arguments(
    command: &CommandExpression,
    visit: &mut impl FnMut(&SignatureShape),
) {
    for args in &command.head_args {
        for expression in &args.expressions {
            walk_expression(expression, visit);
        }
    }
    for tail in &command.tail {
        for args in &tail.args {
            for expression in &args.expressions {
                walk_expression(expression, visit);
            }
        }
    }
    for args in &command.paren_args {
        for expression in &args.expressions {
            walk_expression(expression, visit);
        }
    }
}

/// Traverses all expression arguments supplied to an infix command expression.
fn walk_infix_command_arguments(command: &InfixCommand, visit: &mut impl FnMut(&SignatureShape)) {
    for args in &command.head_args {
        for expression in &args.expressions {
            walk_expression(expression, visit);
        }
    }
    for tail in &command.tail {
        for args in &tail.args {
            for expression in &args.expressions {
                walk_expression(expression, visit);
            }
        }
    }
}

/// Traverses all expression arguments supplied to a refined command expression.
///
/// This includes arguments attached to refinement parts, base command arguments,
/// tail arguments, and optional parenthesized invocation arguments.
fn walk_refined_command_expression_arguments(
    command: &RefinedCommandExpression,
    visit: &mut impl FnMut(&SignatureShape),
) {
    for part in &command.parts {
        for tail in &part.tail {
            for args in &tail.args {
                for expression in &args.expressions {
                    walk_expression(expression, visit);
                }
            }
        }
    }
    for args in &command.head_args {
        for expression in &args.expressions {
            walk_expression(expression, visit);
        }
    }
    for tail in &command.tail {
        for args in &tail.args {
            for expression in &args.expressions {
                walk_expression(expression, visit);
            }
        }
    }
    for args in &command.paren_args {
        for expression in &args.expressions {
            walk_expression(expression, visit);
        }
    }
}


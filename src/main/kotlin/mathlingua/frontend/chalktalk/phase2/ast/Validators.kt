/*
 * Copyright 2019 The MathLingua Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package mathlingua.frontend.chalktalk.phase2.ast

import mathlingua.frontend.chalktalk.phase1.ast.Argument
import mathlingua.frontend.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.frontend.chalktalk.phase1.ast.Group
import mathlingua.frontend.chalktalk.phase1.ast.HasLocation
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Token
import mathlingua.frontend.chalktalk.phase1.ast.Section
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Target
import mathlingua.frontend.chalktalk.phase2.ast.clause.validateClause
import mathlingua.frontend.chalktalk.phase2.ast.clause.validateIdStatement
import mathlingua.frontend.support.ParseError

internal fun <T, U : HasLocation> validateByTransform(
    node: Phase1Node,
    errors: MutableList<ParseError>,
    default: T,
    message: String,
    transform: (node: Phase1Node) -> U?,
    builder: (node: U, row: Int, column: Int) -> T
): T {
    val newNode = transform(node)
    return if (newNode != null) {
        builder(newNode, newNode.row, newNode.column)
    } else {
        errors.add(ParseError(message = message, row = node.row, column = node.column))
        default
    }
}

internal fun <T, U : HasLocation> validateByInlineableTransform(
    node: Phase1Node,
    errors: MutableList<ParseError>,
    default: T,
    message: String,
    isInline: Boolean,
    transform: (node: Phase1Node) -> U?,
    builder: (node: U, row: Int, column: Int, isInline: Boolean) -> T
): T {
    val newNode = transform(node)
    return if (newNode != null) {
        builder(newNode, newNode.row, newNode.column, isInline)
    } else {
        errors.add(ParseError(message = message, row = node.row, column = node.column))
        default
    }
}

internal fun <T> validateGroup(
    node: Phase1Node,
    errors: MutableList<ParseError>,
    name: String,
    default: T,
    builder: (group: Group) -> T
) =
    validateByTransform(
        node,
        errors,
        default,
        "Expected group '$name'",
        {
            if (it is Group && it.sections.isNotEmpty() && it.sections[0].name.text == name) {
                it
            } else {
                null
            } as HasLocation
        }) { n, _, _ -> builder(n as Group) }

internal fun <T> validateSection(
    node: Phase1Node,
    errors: MutableList<ParseError>,
    name: String,
    default: T,
    builder: (section: Section) -> T
) = validateSectionImpl(node, errors, name, default, builder)

internal fun <T> validateSection(
    node: Phase1Node, errors: MutableList<ParseError>, default: T, builder: (section: Section) -> T
) = validateSectionImpl(node, errors, null, default, builder)

private fun <T> validateSectionImpl(
    node: Phase1Node,
    errors: MutableList<ParseError>,
    name: String?,
    default: T,
    builder: (section: Section) -> T
) =
    validateByTransform(
        node,
        errors,
        default,
        if (name == null) {
            "Expected a section but found $node"
        } else {
            "Expected a section '$name'"
        },
        {
            if (it is Section && (name == null || it.name.text == name)) {
                it
            } else {
                null
            }
        }) { n, _, _ -> builder(n) }

internal fun <T> validateTargetSection(
    node: Phase1Node,
    errors: MutableList<ParseError>,
    name: String,
    default: T,
    row: Int,
    column: Int,
    builder: (targets: List<Target>, row: Int, column: Int) -> T
) =
    validateSection(node, errors, name, default) { section ->
        if (section.args.isEmpty()) {
            errors.add(
                ParseError(
                    message = "Section '$name' requires at least one argument.",
                    row = section.row,
                    column = section.column))
            default
        } else {
            val targets = mutableListOf<Target>()
            for (arg in section.args) {
                var shouldContinue = false
                val clause = validateClause(arg, errors, arg.isInline)
                if (clause is Target) {
                    targets.add(clause)
                    shouldContinue = true
                }

                if (shouldContinue) {
                    continue
                }

                errors.add(ParseError("Expected an Target", arg.row, arg.column))
            }
            builder(targets.toList(), row, column)
        }
    }

internal fun getOptionalId(node: Phase1Node, errors: MutableList<ParseError>): IdStatement? {
    val group = node.resolve()
    return if (group is Group && group.id != null) {
        val (rawText, _, row, column) = group.id
        // The id token is of type Id and the text is of the form "[...]"
        // Convert it to look like a statement.
        val statementText = "'" + rawText.substring(1, rawText.length - 1) + "'"
        val stmtToken = Phase1Token(statementText, ChalkTalkTokenType.Statement, row, column)
        validateIdStatement(stmtToken, errors)
    } else {
        null
    }
}

internal fun getId(node: Phase1Node, errors: MutableList<ParseError>): IdStatement {
    val id = getOptionalId(node, errors)
    return if (id != null) {
        id
    } else {
        errors.add(ParseError("Expected an Id", node.row, node.column))
        DEFAULT_ID_STATEMENT
    }
}

internal fun <T> validateSingleArg(
    section: Section,
    errors: MutableList<ParseError>,
    default: T,
    type: String,
    builder: (arg: Argument) -> T
) =
    if (section.args.size != 1) {
        errors.add(
            ParseError(
                message = "Expected a single $type argument",
                row = section.row,
                column = section.column))
        default
    } else {
        builder(section.args[0])
    }

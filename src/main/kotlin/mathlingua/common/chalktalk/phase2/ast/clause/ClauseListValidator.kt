/*
 * Copyright 2019 Google LLC
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

package mathlingua.common.chalktalk.phase2.ast.clause

import mathlingua.common.ParseError
import mathlingua.common.Validation
import mathlingua.common.ValidationFailure
import mathlingua.common.ValidationSuccess
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase1.ast.Section
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow

data class ClauseListSection(val name: String, val clauses: List<Clause>)

fun <T> validateClauseList(
    rawNode: Phase1Node,
    expectedName: String,
    canBeEmpty: Boolean,
    builder: (clauses: ClauseListNode, row: Int, column: Int) -> T
): Validation<T> {
    val node = rawNode.resolve()
    val row = getRow(node)
    val column = getColumn(node)
    return when (val validation = validate(node, expectedName, canBeEmpty)) {
        is ValidationSuccess -> ValidationSuccess(builder(ClauseListNode(validation.value.clauses, row, column),
                row, column))
        is ValidationFailure -> ValidationFailure(validation.errors)
    }
}

private fun validate(node: Phase1Node, expectedName: String, canBeEmpty: Boolean): Validation<ClauseListSection> {
    val errors = ArrayList<ParseError>()
    if (node !is Section) {
        errors.add(
            ParseError(
                "Expected a Section",
                getRow(node), getColumn(node)
            )
        )
    }

    val (name, args) = node as Section
    if (name.text != expectedName) {
        errors.add(
            ParseError(
                "Expected a Section with name " +
                    expectedName + " but found " + name.text,
                getRow(node), getColumn(node)
            )
        )
    }

    if (args.isEmpty() && !canBeEmpty) {
        errors.add(
            ParseError(
                "Section '" + name.text + "' requires at least one argument.",
                getRow(node), getColumn(node)
            )
        )
    }

    val clauses = ArrayList<Clause>()
    for (arg in args) {
        when (val validation = validateClause(arg)) {
            is ValidationSuccess -> clauses.add(validation.value)
            is ValidationFailure -> errors.addAll(validation.errors)
        }
    }

    return if (errors.isNotEmpty()) {
        ValidationFailure(errors)
    } else ValidationSuccess(
            ClauseListSection(
                    name.text,
                    clauses
            )
    )
}

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

import mathlingua.common.*
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase1.ast.Section
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow
import mathlingua.common.chalktalk.phase2.ast.Phase2Node

data class ClauseListSection(val name: String, val clauses: List<Clause>)

fun <T : Phase2Node> validateClauseList(
    tracker: MutableLocationTracker,
    rawNode: Phase1Node,
    expectedName: String,
    canBeEmpty: Boolean,
    builder: (clauses: ClauseListNode) -> T
): Validation<T> {
    val node = rawNode.resolve()
    return when (val validation = validate(node, expectedName, canBeEmpty, tracker)) {
        is ValidationSuccess -> validationSuccess(tracker, rawNode, builder(ClauseListNode(validation.value.clauses)))
        is ValidationFailure -> validationFailure(validation.errors)
    }
}

private fun validate(node: Phase1Node, expectedName: String, canBeEmpty: Boolean, tracker: MutableLocationTracker): Validation<ClauseListSection> {
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
        when (val validation = validateClause(arg, tracker)) {
            is ValidationSuccess -> clauses.add(validation.value)
            is ValidationFailure -> errors.addAll(validation.errors)
        }
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(
            ClauseListSection(
                    name.text,
                    clauses
            )
    )
}

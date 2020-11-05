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

package mathlingua.chalktalk.phase2.ast.validator

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase1.ast.Section
import mathlingua.chalktalk.phase1.ast.getColumn
import mathlingua.chalktalk.phase1.ast.getRow
import mathlingua.chalktalk.phase2.ast.clause.Clause
import mathlingua.chalktalk.phase2.ast.clause.ClauseListNode
import mathlingua.chalktalk.phase2.ast.clause.validateClause
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError
import mathlingua.support.Validation
import mathlingua.support.ValidationFailure
import mathlingua.support.ValidationSuccess
import mathlingua.support.validationFailure
import mathlingua.support.validationSuccess

sealed class NumAllowed {
    abstract fun validate(count: Int): String?
}

class ZeroOrMore : NumAllowed() {
    override fun validate(count: Int) =
        if (count < 0) {
            "Expected zero or more arguments but found $count"
        } else {
            null
        }
}

data class Exactly(private val expected: Int) : NumAllowed() {
    override fun validate(count: Int) =
        if (count != expected) {
            "Expected exactly $expected arguments but found $count"
        } else {
            null
        }
}

data class AtLeast(private val expected: Int) : NumAllowed() {
    override fun validate(count: Int) =
        if (count < expected) {
            "Expected at least $expected arguments but found $count"
        } else {
            null
        }
}

data class ClauseListSection(val name: String, val clauses: List<Clause>)

fun <T : Phase2Node> validateClauseList(
    numAllowed: NumAllowed,
    tracker: MutableLocationTracker,
    rawNode: Phase1Node,
    expectedName: String,
    builder: (clauses: ClauseListNode) -> T
): Validation<T> {
    val node = rawNode.resolve()
    return when (val validation = validate(numAllowed, node, expectedName, tracker)
    ) {
        is ValidationSuccess ->
            try {
                validationSuccess(
                    tracker, rawNode, builder(ClauseListNode(validation.value.clauses)))
            } catch (e: Exception) {
                validationFailure(
                    listOf(
                        ParseError(
                            e.message ?: "An unknown error occurred",
                            getRow(node),
                            getColumn(node))))
            }
        is ValidationFailure -> validationFailure(validation.errors)
    }
}

private fun validate(
    numAllowed: NumAllowed, node: Phase1Node, expectedName: String, tracker: MutableLocationTracker
): Validation<ClauseListSection> {
    val errors = ArrayList<ParseError>()
    if (node !is Section) {
        errors.add(ParseError("Expected a Section", getRow(node), getColumn(node)))
    }

    val (name, args) = node as Section
    if (name.text != expectedName) {
        errors.add(
            ParseError(
                "Expected a Section with name " + expectedName + " but found " + name.text,
                getRow(node),
                getColumn(node)))
    }

    val clauses = ArrayList<Clause>()
    for (arg in args) {
        when (val validation = validateClause(arg, tracker)
        ) {
            is ValidationSuccess -> clauses.add(validation.value)
            is ValidationFailure -> errors.addAll(validation.errors)
        }
    }

    val countMessage = numAllowed.validate(clauses.size)
    if (countMessage != null) {
        errors.add(ParseError(countMessage, getRow(node), getColumn(node)))
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(ClauseListSection(name.text, clauses))
}

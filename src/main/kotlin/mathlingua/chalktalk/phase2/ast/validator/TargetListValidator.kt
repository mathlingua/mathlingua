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
import mathlingua.chalktalk.phase2.ast.clause.Target
import mathlingua.chalktalk.phase2.ast.clause.validateClause
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError
import mathlingua.support.Validation
import mathlingua.support.ValidationFailure
import mathlingua.support.ValidationSuccess
import mathlingua.support.validationFailure
import mathlingua.support.validationSuccess

data class TargetListSection(val targets: List<Target>)

fun <T : Phase2Node> validateTargetList(
    tracker: MutableLocationTracker,
    rawNode: Phase1Node,
    expectedName: String,
    builder: (targets: List<Target>) -> T
): Validation<T> {
    val node = rawNode.resolve()
    return when (val validation = validateTargetListSection(tracker, node, expectedName)
    ) {
        is ValidationSuccess -> {
            val targets = validation.value.targets
            return validationSuccess(tracker, rawNode, builder(targets))
        }
        is ValidationFailure -> validationFailure(validation.errors)
    }
}

private fun validateTargetListSection(
    tracker: MutableLocationTracker, node: Phase1Node, expectedName: String
): Validation<TargetListSection> {
    val errors = ArrayList<ParseError>()
    if (node !is Section) {
        errors.add(ParseError("Expected a Section", getRow(node), getColumn(node)))
    }

    val (name1, args) = node as Section
    val name = name1.text
    if (name != expectedName) {
        errors.add(
            ParseError(
                "Expected a Section with name " + expectedName + " but found " + name,
                getRow(node),
                getColumn(node)))
    }

    val targets = ArrayList<Target>()

    if (args.isEmpty()) {
        errors.add(
            ParseError(
                "Section '" + name1.text + "' requires at least one argument.",
                getRow(node),
                getColumn(node)))
    }

    for (arg in args) {
        var shouldContinue = false
        when (val clauseValidation = validateClause(arg, tracker)
        ) {
            is ValidationSuccess -> {
                val clause = clauseValidation.value
                if (clause is Target) {
                    targets.add(clause)
                    shouldContinue = true
                }
            }
            is ValidationFailure -> errors.addAll(clauseValidation.errors)
        }

        if (shouldContinue) {
            continue
        }

        errors.add(ParseError("Expected an Target", getRow(arg), getColumn(arg)))
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(TargetListSection(targets))
}

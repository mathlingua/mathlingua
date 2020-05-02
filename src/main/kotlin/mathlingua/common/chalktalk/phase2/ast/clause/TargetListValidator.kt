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

data class TargetListSection(val targets: List<Target>)

fun <T> validateTargetList(
    rawNode: Phase1Node,
    expectedName: String,
    builder: (targets: List<Target>, row: Int, column: Int) -> T
): Validation<T> {
    val node = rawNode.resolve()
    val row = getRow(node)
    val column = getColumn(node)
    return when (val validation = validateTargetListSection(node, expectedName)) {
        is ValidationSuccess -> {
            val targets = validation.value.targets
            return ValidationSuccess(builder(targets, row, column))
        }
        is ValidationFailure -> ValidationFailure(validation.errors)
    }
}

private fun validateTargetListSection(
    node: Phase1Node,
    expectedName: String
): Validation<TargetListSection> {
    val errors = ArrayList<ParseError>()
    if (node !is Section) {
        errors.add(
            ParseError(
                "Expected a Section",
                getRow(node), getColumn(node)
            )
        )
    }

    val (name1, args) = node as Section
    val name = name1.text
    if (name != expectedName) {
        errors.add(
            ParseError(
                "Expected a Section with name " +
                    expectedName + " but found " + name,
                getRow(node),
                getColumn(node)
            )
        )
    }

    val targets = ArrayList<Target>()

    if (args.isEmpty()) {
        errors.add(
            ParseError(
                "Section '" + name1.text +
                    "' requires at least one argument.",
                getRow(node),
                getColumn(node)
            )
        )
    }

    for (arg in args) {
        var shouldContinue = false
        when (val clauseValidation = validateClause(arg)) {
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

        errors.add(
            ParseError(
                "Expected an Target",
                getRow(arg), getColumn(arg)
            )
        )
    }

    return if (errors.isNotEmpty()) {
        ValidationFailure(errors)
    } else ValidationSuccess(TargetListSection(targets))
}

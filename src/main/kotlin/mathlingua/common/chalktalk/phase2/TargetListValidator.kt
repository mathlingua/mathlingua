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

package mathlingua.common.chalktalk.phase2

import mathlingua.common.ParseError
import mathlingua.common.Validation
import mathlingua.common.chalktalk.phase1.ast.ChalkTalkNode
import mathlingua.common.chalktalk.phase1.ast.Section
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow

data class TargetListSection(val targets: List<Target>)

fun <T> validateTargetList(
    rawNode: ChalkTalkNode,
    expectedName: String,
    builder: (targets: List<Target>) -> T
): Validation<T> {
    val node = rawNode.resolve()

    val validation =
        validate(node, expectedName)
    if (!validation.isSuccessful) {
        return Validation.failure(validation.errors)
    }

    val targets = validation.value!!.targets
    return Validation.success(builder(targets))
}

private fun validate(
    node: ChalkTalkNode,
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
        val clauseValidation = validateClause(arg)
        if (clauseValidation.isSuccessful) {
            val clause = clauseValidation.value
            if (clause is Target) {
                targets.add(clause)
                continue
            }
        } else {
            errors.addAll(clauseValidation.errors)
        }

        errors.add(
            ParseError(
                "Expected an Target",
                getRow(arg), getColumn(arg)
            )
        )
    }

    return if (errors.isNotEmpty()) {
        Validation.failure(errors)
    } else Validation.success(TargetListSection(targets))
}

/*
 * Copyright 2022 The MathLingua Authors
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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.getColumn
import mathlingua.frontend.chalktalk.phase1.ast.getRow
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_SPECIFY_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_ZERO_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.clause.ValidationPair
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.negativeFloat.isNegativeFloatGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.negativeFloat.validateNegativeFloatGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.negativeInt.isNegativeIntGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.negativeInt.validateNegativeIntGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.positiveFloat.isPositiveFloatGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.positiveFloat.validatePositiveFloatGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.positiveInt.isPositiveIntGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.positiveInt.validatePositiveIntGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.zero.isZeroGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.zero.validateZeroGroup
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateSection
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

internal data class SpecifySection(val numberGroups: List<NumberGroup>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        numberGroups.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        for (i in numberGroups.indices) {
            writer.append(numberGroups[i], true, indent)
            if (i != numberGroups.size - 1) {
                writer.writeNewline()
            }
        }
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node {
        return chalkTransformer(
            SpecifySection(
                numberGroups = numberGroups.map { it.transform(chalkTransformer) as NumberGroup }))
    }
}

internal fun validateSpecifySection(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateSection(node, errors, "Specify", DEFAULT_SPECIFY_SECTION) {
            if (it.args.isEmpty()) {
                errors.add(
                    ParseError(
                        message = "Expected at least one specification but none",
                        row = getRow(node),
                        column = getColumn(node)))
                DEFAULT_SPECIFY_SECTION
            } else {
                SpecifySection(
                    numberGroups =
                        it.args.map { arg -> validateNumberGroup(arg.resolve(), errors, tracker) })
            }
        }
    }

private val NUMBER_GROUP_VALIDATORS =
    listOf(
        ValidationPair<NumberGroup>(::isZeroGroup, ::validateZeroGroup),
        ValidationPair(::isNegativeFloatGroup, ::validateNegativeFloatGroup),
        ValidationPair(::isPositiveFloatGroup, ::validatePositiveFloatGroup),
        ValidationPair(::isNegativeIntGroup, ::validateNegativeIntGroup),
        ValidationPair(::isPositiveIntGroup, ::validatePositiveIntGroup))

private fun validateNumberGroup(
    rawNode: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
): NumberGroup {
    val node = rawNode.resolve()

    for (pair in NUMBER_GROUP_VALIDATORS) {
        if (pair.matches(node)) {
            return pair.validate(node, errors, tracker)
        }
    }

    errors.add(
        ParseError(
            message = "Unrecognized specification.  Perhaps there is a typo.",
            row = getRow(rawNode),
            column = getColumn(rawNode)))
    return DEFAULT_ZERO_GROUP
}

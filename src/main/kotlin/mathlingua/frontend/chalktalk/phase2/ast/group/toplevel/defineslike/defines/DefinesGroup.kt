/*
 * Copyright 2020 Google LLC
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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines

import mathlingua.backend.transform.Signature
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.clause.AbstractionNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.HasUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.DefinesStatesOrViews
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewed.ViewedSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.LocationTracker
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.textalk.Command
import mathlingua.frontend.textalk.TexTalkNodeType
import mathlingua.frontend.textalk.TextTexTalkNode

abstract class DefinesGroup(override val metaDataSection: MetaDataSection?) :
    TopLevelGroup(metaDataSection), HasUsingSection, DefinesStatesOrViews {
    abstract val id: IdStatement
    abstract val signature: Signature?
    abstract val definesSection: DefinesSection
    abstract val requiringSection: RequiringSection?
    abstract val whenSection: WhenSection?
    abstract val viewedSection: ViewedSection?
    abstract val writtenSection: WrittenSection
}

fun isDefinesGroup(node: Phase1Node) = firstSectionMatchesName(node, "Defines")

fun validateDefinesGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
): DefinesGroup =
    when {
        isDefinesCollectsGroup(node) -> {
            validateDefinesCollectsGroup(node, errors, tracker)
        }
        isDefinesGeneratedGroup(node) -> {
            validateDefinesGeneratedGroup(node, errors, tracker)
        }
        isDefinesInstantiatedGroup(node) -> {
            validateDefinesInstantiatedGroup(node, errors, tracker)
        }
        isDefinesMapsGroup(node) -> {
            validateDefinesMapsGroup(node, errors, tracker)
        }
        isDefinesEvaluatedGroup(node) -> {
            validateDefinesEvaluatedGroup(node, errors, tracker)
        }
        else -> {
            validateDefinesMeansGroup(node, errors, tracker)
        }
    }

fun checkIfFunctionSignatureMatchDefines(
    defines: DefinesGroup, tracker: LocationTracker
): ParseError? {
    val idArgs =
        when (val validation = defines.id.texTalkRoot
        ) {
            is ValidationSuccess -> {
                val exp = validation.value
                // check if the id is of the form `\f.g.h{a}(x, y, z)`
                if (exp.children.size == 1 &&
                    exp.children[0] is Command &&
                    (exp.children[0] as Command).parts.isNotEmpty() &&
                    (exp.children[0] as Command).parts.last().paren != null &&
                    (exp.children[0] as Command).parts.last().paren!!.type ==
                        TexTalkNodeType.ParenGroup) {
                    // extract 'x, y, z'
                    (exp.children[0] as Command).parts.last().paren!!.parameters.items.mapNotNull {
                        if (it.children.size == 1 && it.children[0] is TextTexTalkNode) {
                            (it.children[0] as TextTexTalkNode).text
                        } else {
                            null
                        }
                    }
                } else {
                    null
                }
            }
            is ValidationFailure -> null
        }
            ?: return null

    val location = tracker.getLocationOf(defines.definesSection) ?: Location(row = -1, column = -1)

    var hasParenArgs = false
    for (target in defines.definesSection.targets) {
        if (target is AbstractionNode) {
            val abs = target.abstraction
            if (!abs.isVarArgs &&
                !abs.isEnclosed &&
                abs.subParams == null &&
                abs.parts.size == 1 &&
                abs.parts[0].tail == null &&
                abs.parts[0].subParams == null &&
                abs.parts[0].params != null) {
                hasParenArgs = true
                val defArgs = abs.parts[0].params!!.map { it.text }
                if (idArgs != defArgs) {
                    return ParseError(
                        message =
                            "Expected function arguments '${idArgs.joinToString(", ").trim()}' but found '${defArgs.joinToString(", ").trim()}'",
                        row = location.row,
                        column = location.column)
                }
            }
        }
    }

    if (!hasParenArgs) {
        return ParseError(
            message =
                "Expected a definition of a function with arguments '${idArgs.joinToString(", ").trim()}'",
            row = location.row,
            column = location.column)
    }

    return null
}

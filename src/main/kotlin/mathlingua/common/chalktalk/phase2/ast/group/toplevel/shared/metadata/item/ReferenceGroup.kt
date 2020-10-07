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

package mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.metadata.item

import mathlingua.common.chalktalk.phase1.ast.*
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow
import mathlingua.common.chalktalk.phase2.ast.common.OnePartNode
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.ReferenceSection
import mathlingua.common.chalktalk.phase2.ast.section.identifySections
import mathlingua.common.support.MutableLocationTracker
import mathlingua.common.support.ParseError
import mathlingua.common.support.Validation
import mathlingua.common.support.ValidationFailure
import mathlingua.common.support.ValidationSuccess
import mathlingua.common.support.validationFailure
import mathlingua.common.support.validationSuccess

data class ReferenceGroup(val referenceSection: ReferenceSection) : OnePartNode<ReferenceSection>(
    referenceSection,
    ::ReferenceGroup
), MetaDataItem

fun isReferenceGroup(node: Phase1Node) = firstSectionMatchesName(node, "reference")

fun validateReferenceGroup(groupNode: Group, tracker: MutableLocationTracker): Validation<MetaDataItem> {
    val errors = ArrayList<ParseError>()
    val group = groupNode.resolve()
    if (group.id != null) {
        errors.add(
                ParseError(
                        "A reference cannot have an Id",
                        getRow(group), getColumn(group)
                )
        )
    }

    val sections = group.sections

    val sectionMap: Map<String, List<Section>>
    try {
        sectionMap = identifySections(
                sections, "reference"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return validationFailure(errors)
    }

    val rawReference = sectionMap["reference"]!![0]
    var referenceSection: ReferenceSection? = null
    when (val validation = validateReferenceSection(rawReference, tracker)) {
        is ValidationSuccess -> referenceSection = validation.value
        is ValidationFailure -> errors.addAll(validation.errors)
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else {
        validationSuccess(
                tracker,
                groupNode,
                ReferenceGroup(
                        referenceSection = referenceSection!!
                )
        )
    }
}

private fun validateReferenceSection(rawNode: Phase1Node, tracker: MutableLocationTracker): Validation<ReferenceSection> {
    val node = rawNode.resolve()
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
    if (name.text != "reference") {
        errors.add(
                ParseError(
                        "Expected a Section with name 'reference' but found " + name.text,
                        getRow(node), getColumn(node)
                )
        )
    }

    if (args.isEmpty()) {
        errors.add(
                ParseError(
                        "Section '" + name.text + "' requires at least one 'source' argument.",
                        getRow(node), getColumn(node)
                )
        )
    }

    val sourceItems = mutableListOf<SourceItemGroup>()
    for (arg in args) {
        if (arg.chalkTalkTarget is Group) {
            when (val validation = validateSourceItemGroup(arg.chalkTalkTarget, tracker)) {
                is ValidationSuccess -> {
                    sourceItems.add(validation.value)
                }
                is ValidationFailure -> errors.addAll(validation.errors)
            }
        } else {
            errors.add(
                    ParseError(
                            message = "Expected a 'source' group but found ${arg.toCode()}",
                            row = getRow(arg),
                            column = getColumn(arg)
                    )
            )
        }
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(
            tracker,
            rawNode,
            ReferenceSection(sourceItems = sourceItems)
    )
}

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

package mathlingua.common.chalktalk.phase2.ast.group.clause.collection

import mathlingua.common.chalktalk.phase1.ast.*
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.Clause
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.group.clause.`for`.ForSection
import mathlingua.common.chalktalk.phase2.ast.group.clause.`for`.validateForSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.WhereSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.validateWhereSection
import mathlingua.common.chalktalk.phase2.ast.section.identifySections
import mathlingua.common.support.MutableLocationTracker
import mathlingua.common.support.ParseError
import mathlingua.common.support.Validation
import mathlingua.common.support.ValidationFailure
import mathlingua.common.support.ValidationSuccess
import mathlingua.common.support.validationFailure
import mathlingua.common.support.validationSuccess

data class CollectionGroup(
    val collectionSection: CollectionSection,
    val ofSection: OfSection,
    val inSection: InSection?,
    val forSection: ForSection,
    val whereSection: WhereSection
) : Clause {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(collectionSection)
        fn(ofSection)
        if (inSection != null) {
            fn(inSection)
        }
        fn(forSection)
        fn(whereSection)
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
        mathlingua.common.chalktalk.phase2.ast.clause.toCode(
            writer,
            isArg,
            indent,
            collectionSection,
            ofSection,
            inSection,
            forSection,
            whereSection
        )

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(
        CollectionGroup(
        collectionSection = collectionSection.transform(chalkTransformer) as CollectionSection,
        ofSection = ofSection.transform(chalkTransformer) as OfSection,
        inSection = inSection?.transform(chalkTransformer) as InSection?,
        forSection = forSection.transform(chalkTransformer) as ForSection,
        whereSection = whereSection.transform(chalkTransformer) as WhereSection
    )
    )
}

fun isCollectionGroup(node: Phase1Node) = firstSectionMatchesName(node, "collection")

fun validateCollectionGroup(rawNode: Phase1Node, tracker: MutableLocationTracker): Validation<CollectionGroup> {
    val node = rawNode.resolve()

    val errors = ArrayList<ParseError>()
    if (node !is Group) {
        errors.add(
            ParseError(
                "Expected a Group",
                getRow(node), getColumn(node)
            )
        )
        return validationFailure(errors)
    }

    val (sections) = node

    val sectionMap: Map<String, List<Section>>
    try {
        sectionMap = identifySections(
            sections,
            "collection", "of", "in?", "for", "where"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return validationFailure(errors)
    }

    val collectionNode = sectionMap["collection"]!!
    val ofNode = sectionMap["of"]!!
    val inNode = sectionMap["in"]
    val forNode = sectionMap["for"]!!
    val whereNode = sectionMap["where"]!!

    var collectionSection: CollectionSection? = null
    when (val validation = validateCollectionSection(collectionNode[0], tracker)) {
        is ValidationSuccess -> collectionSection = validation.value
        is ValidationFailure -> errors.addAll(validation.errors)
    }

    var ofSection: OfSection? = null
    when (val validation = validateOfSection(ofNode[0], tracker)) {
        is ValidationSuccess -> ofSection = validation.value
        is ValidationFailure -> errors.addAll(validation.errors)
    }

    var inSection: InSection? = null
    if (inNode != null) {
        when (val validation = validateInSection(inNode[0], tracker)) {
            is ValidationSuccess -> inSection = validation.value
            is ValidationFailure -> errors.addAll(validation.errors)
        }
    }

    var forSection: ForSection? = null
    when (val validation = validateForSection(forNode[0], tracker)) {
        is ValidationSuccess -> forSection = validation.value
        is ValidationFailure -> errors.addAll(validation.errors)
    }

    var whereSection: WhereSection? = null
    when (val validation = validateWhereSection(whereNode[0], tracker)) {
        is ValidationSuccess -> whereSection = validation.value
        is ValidationFailure -> errors.addAll(validation.errors)
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(
        tracker,
        rawNode,
        CollectionGroup(
            collectionSection!!,
            ofSection!!,
            inSection,
            forSection!!,
            whereSection!!
        )
    )
}

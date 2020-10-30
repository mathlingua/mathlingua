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
import mathlingua.common.chalktalk.phase2.ast.clause.Validator
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.clause.validateGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.`for`.ForSection
import mathlingua.common.chalktalk.phase2.ast.group.clause.`for`.validateForSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.WhereSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.validateWhereSection
import mathlingua.common.support.MutableLocationTracker
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

fun validateCollectionGroup(rawNode: Phase1Node, tracker: MutableLocationTracker) = validateGroup(
    tracker,
    rawNode,
    listOf(
        Validator(
            name = "collection",
            optional = false,
            validate = ::validateCollectionSection
        ),
        Validator(
            name = "of",
            optional = false,
            validate = ::validateOfSection
        ),
        Validator(
            name = "in",
            optional = true,
            validate = ::validateInSection
        ),
        Validator(
            name = "for",
            optional = false,
            validate = ::validateForSection
        ),
        Validator(
            name = "where",
            optional = false,
            validate = ::validateWhereSection
        )
    )
) {
    validationSuccess(tracker, rawNode, CollectionGroup(
    collectionSection = it["collection"] as CollectionSection,
    ofSection = it["of"] as OfSection,
    inSection = it["in"] as InSection?,
    forSection = it["for"] as ForSection,
    whereSection = it["where"] as WhereSection
)) }

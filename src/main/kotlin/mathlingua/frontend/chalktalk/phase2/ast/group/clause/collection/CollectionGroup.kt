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

package mathlingua.frontend.chalktalk.phase2.ast.group.clause.collection

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_COLLECTION_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_COLLECTION_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_FOR_ALL_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_OF_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_WHERE_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.forAll.ForAllSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.forAll.validateForSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.WhereSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.validateWhereSection
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

data class CollectionGroup(
    val collectionSection: CollectionSection,
    val ofSection: OfSection,
    val inSection: InSection?,
    val forAllSection: ForAllSection,
    val whereSection: WhereSection
) : Clause {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(collectionSection)
        fn(ofSection)
        if (inSection != null) {
            fn(inSection)
        }
        fn(forAllSection)
        fn(whereSection)
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
        mathlingua.frontend.chalktalk.phase2.ast.clause.toCode(
            writer,
            isArg,
            indent,
            collectionSection,
            ofSection,
            inSection,
            forAllSection,
            whereSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            CollectionGroup(
                collectionSection =
                    collectionSection.transform(chalkTransformer) as CollectionSection,
                ofSection = ofSection.transform(chalkTransformer) as OfSection,
                inSection = inSection?.transform(chalkTransformer) as InSection?,
                forAllSection = forAllSection.transform(chalkTransformer) as ForAllSection,
                whereSection = whereSection.transform(chalkTransformer) as WhereSection))
}

fun isCollectionGroup(node: Phase1Node) = firstSectionMatchesName(node, "collection")

fun validateCollectionGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "collection", DEFAULT_COLLECTION_GROUP) { group ->
            identifySections(
                group,
                errors,
                DEFAULT_COLLECTION_GROUP,
                listOf("collection", "of", "in?", "forAll", "where")) { sections ->
                CollectionGroup(
                    collectionSection =
                        ensureNonNull(sections["collection"], DEFAULT_COLLECTION_SECTION) {
                            validateCollectionSection(it, errors, tracker)
                        },
                    ofSection =
                        ensureNonNull(sections["of"], DEFAULT_OF_SECTION) {
                            validateOfSection(it, errors, tracker)
                        },
                    inSection =
                        ifNonNull(sections["in"]) { validateInSection(it, errors, tracker) },
                    forAllSection =
                        ensureNonNull(sections["forAll"], DEFAULT_FOR_ALL_SECTION) {
                            validateForSection(it, errors, tracker)
                        },
                    whereSection =
                        ensureNonNull(sections["where"], DEFAULT_WHERE_SECTION) {
                            validateWhereSection(it, errors, tracker)
                        })
            }
        }
    }

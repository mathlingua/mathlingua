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

package mathlingua.chalktalk.phase2.ast.group.clause.collection

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_COLLECTION_GROUP
import mathlingua.chalktalk.phase2.ast.DEFAULT_COLLECTION_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_FOR_ALL_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_OF_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_WHERE_SECTION
import mathlingua.chalktalk.phase2.ast.clause.Clause
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.clause.forAll.ForAllSection
import mathlingua.chalktalk.phase2.ast.group.clause.forAll.neoValidateForSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.WhereSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.neoValidateWhereSection
import mathlingua.chalktalk.phase2.ast.neoTrack
import mathlingua.chalktalk.phase2.ast.neoValidateGroup
import mathlingua.chalktalk.phase2.ast.section.neoEnsureNonNull
import mathlingua.chalktalk.phase2.ast.section.neoIdentifySections
import mathlingua.chalktalk.phase2.ast.section.neoIfNonNull
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError

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
        mathlingua.chalktalk.phase2.ast.clause.toCode(
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

fun neoValidateCollectionGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    neoTrack(node, tracker) {
        neoValidateGroup(node.resolve(), errors, "collection", DEFAULT_COLLECTION_GROUP) { group ->
            neoIdentifySections(
                group,
                errors,
                DEFAULT_COLLECTION_GROUP,
                listOf("collection", "of", "in?", "forAll", "where")) { sections ->
                CollectionGroup(
                    collectionSection =
                        neoEnsureNonNull(sections["collection"], DEFAULT_COLLECTION_SECTION) {
                            neoValidateCollectionSection(it, errors, tracker)
                        },
                    ofSection =
                        neoEnsureNonNull(sections["of"], DEFAULT_OF_SECTION) {
                            neoValidateOfSection(it, errors, tracker)
                        },
                    inSection =
                        neoIfNonNull(sections["in"]) { neoValidateInSection(it, errors, tracker) },
                    forAllSection =
                        neoEnsureNonNull(sections["forAll"], DEFAULT_FOR_ALL_SECTION) {
                            neoValidateForSection(it, errors, tracker)
                        },
                    whereSection =
                        neoEnsureNonNull(sections["where"], DEFAULT_WHERE_SECTION) {
                            neoValidateWhereSection(it, errors, tracker)
                        })
            }
        }
    }

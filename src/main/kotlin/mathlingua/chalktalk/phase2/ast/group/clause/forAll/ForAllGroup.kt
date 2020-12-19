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

package mathlingua.chalktalk.phase2.ast.group.clause.forAll

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.ast.DEFAULT_FOR_ALL_GROUP
import mathlingua.chalktalk.phase2.ast.DEFAULT_FOR_ALL_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_THEN_SECTION
import mathlingua.chalktalk.phase2.ast.clause.Clause
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.common.FourPartNode
import mathlingua.chalktalk.phase2.ast.group.clause.If.ThenSection
import mathlingua.chalktalk.phase2.ast.group.clause.If.neoValidateThenSection
import mathlingua.chalktalk.phase2.ast.group.clause.exists.SuchThatSection
import mathlingua.chalktalk.phase2.ast.group.clause.exists.neoValidateSuchThatSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.WhereSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.neoValidateWhereSection
import mathlingua.chalktalk.phase2.ast.neoTrack
import mathlingua.chalktalk.phase2.ast.neoValidateGroup
import mathlingua.chalktalk.phase2.ast.section.neoEnsureNonNull
import mathlingua.chalktalk.phase2.ast.section.neoIdentifySections
import mathlingua.chalktalk.phase2.ast.section.neoIfNonNull
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError

data class ForAllGroup(
    val forAllSection: ForAllSection,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection?,
    val thenSection: ThenSection
) :
    FourPartNode<ForAllSection, WhereSection?, SuchThatSection?, ThenSection>(
        forAllSection, whereSection, suchThatSection, thenSection, ::ForAllGroup),
    Clause

fun isForGroup(node: Phase1Node) = firstSectionMatchesName(node, "forAll")

fun neoValidateForGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    neoTrack(node, tracker) {
        neoValidateGroup(node.resolve(), errors, "forAll", DEFAULT_FOR_ALL_GROUP) { group ->
            neoIdentifySections(
                group,
                errors,
                DEFAULT_FOR_ALL_GROUP,
                listOf("forAll", "where?", "suchThat?", "then")) { sections ->
                ForAllGroup(
                    forAllSection =
                        neoEnsureNonNull(sections["forAll"], DEFAULT_FOR_ALL_SECTION) {
                            neoValidateForSection(it, errors, tracker)
                        },
                    whereSection =
                        neoIfNonNull(sections["where"]) {
                            neoValidateWhereSection(it, errors, tracker)
                        },
                    suchThatSection =
                        neoIfNonNull(sections["suchThat"]) {
                            neoValidateSuchThatSection(it, errors, tracker)
                        },
                    thenSection =
                        neoEnsureNonNull(sections["then"], DEFAULT_THEN_SECTION) {
                            neoValidateThenSection(it, errors, tracker)
                        })
            }
        }
    }

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

package mathlingua.frontend.chalktalk.phase2.ast.group.clause.forAll

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_FOR_ALL_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_FOR_ALL_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_THEN_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.ThreePartNode
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.ThenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.validateThenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.exists.SuchThatSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.exists.validateSuchThatSection
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

data class ForAllGroup(
    val forAllSection: ForAllSection,
    val suchThatSection: SuchThatSection?,
    val thenSection: ThenSection
) :
    ThreePartNode<ForAllSection, SuchThatSection?, ThenSection>(
        forAllSection, suchThatSection, thenSection, ::ForAllGroup),
    Clause

fun isForGroup(node: Phase1Node) = firstSectionMatchesName(node, "forAll")

fun validateForGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "forAll", DEFAULT_FOR_ALL_GROUP) { group ->
            identifySections(
                group, errors, DEFAULT_FOR_ALL_GROUP, listOf("forAll", "suchThat?", "then")) {
            sections ->
                ForAllGroup(
                    forAllSection =
                        ensureNonNull(sections["forAll"], DEFAULT_FOR_ALL_SECTION) {
                            validateForSection(it, errors, tracker)
                        },
                    suchThatSection =
                        ifNonNull(sections["suchThat"]) {
                            validateSuchThatSection(it, errors, tracker)
                        },
                    thenSection =
                        ensureNonNull(sections["then"], DEFAULT_THEN_SECTION) {
                            validateThenSection(it, errors, tracker)
                        })
            }
        }
    }

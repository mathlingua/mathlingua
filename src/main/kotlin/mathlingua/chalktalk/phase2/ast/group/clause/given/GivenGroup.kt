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

package mathlingua.chalktalk.phase2.ast.group.clause.given

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.ast.DEFAULT_ALL_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_GIVEN_GROUP
import mathlingua.chalktalk.phase2.ast.DEFAULT_GIVEN_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_WHERE_SECTION
import mathlingua.chalktalk.phase2.ast.clause.Clause
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.common.FourPartNode
import mathlingua.chalktalk.phase2.ast.group.clause.exists.SuchThatSection
import mathlingua.chalktalk.phase2.ast.group.clause.exists.validateSuchThatSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.GivenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.validateGivenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.WhereSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.validateWhereSection
import mathlingua.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.chalktalk.phase2.ast.section.identifySections
import mathlingua.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.chalktalk.phase2.ast.track
import mathlingua.chalktalk.phase2.ast.validateGroup
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError

data class GivenGroup(
    val givenSection: GivenSection,
    val whereSection: WhereSection,
    val allSection: AllSection,
    val suchThatSection: SuchThatSection?
) :
    FourPartNode<GivenSection, WhereSection, AllSection, SuchThatSection?>(
        givenSection, whereSection, allSection, suchThatSection, ::GivenGroup),
    Clause

fun isGivenGroup(node: Phase1Node) = firstSectionMatchesName(node, "given")

fun validateGivenGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "given", DEFAULT_GIVEN_GROUP) { group ->
            identifySections(
                group, errors, DEFAULT_GIVEN_GROUP, listOf("given", "where", "all", "suchThat?")) {
            sections ->
                GivenGroup(
                    givenSection =
                        ensureNonNull(sections["given"], DEFAULT_GIVEN_SECTION) {
                            validateGivenSection(it, errors, tracker)
                        },
                    whereSection =
                        ensureNonNull(sections["where"], DEFAULT_WHERE_SECTION) {
                            validateWhereSection(it, errors, tracker)
                        },
                    allSection =
                        ensureNonNull(sections["all"], DEFAULT_ALL_SECTION) {
                            validateAllSection(it, errors, tracker)
                        },
                    suchThatSection =
                        ifNonNull(sections["suchThat"]) {
                            validateSuchThatSection(it, errors, tracker)
                        })
            }
        }
    }

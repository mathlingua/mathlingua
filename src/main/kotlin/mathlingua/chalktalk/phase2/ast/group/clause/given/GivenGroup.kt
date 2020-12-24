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
import mathlingua.chalktalk.phase2.ast.group.clause.exists.neoValidateSuchThatSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.GivenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.neoValidateGivenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.WhereSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.neoValidateWhereSection
import mathlingua.chalktalk.phase2.ast.neoTrack
import mathlingua.chalktalk.phase2.ast.neoValidateGroup
import mathlingua.chalktalk.phase2.ast.section.neoEnsureNonNull
import mathlingua.chalktalk.phase2.ast.section.neoIdentifySections
import mathlingua.chalktalk.phase2.ast.section.neoIfNonNull
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

fun neoValidateGivenGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    neoTrack(node, tracker) {
        neoValidateGroup(node.resolve(), errors, "given", DEFAULT_GIVEN_GROUP) { group ->
            neoIdentifySections(
                group, errors, DEFAULT_GIVEN_GROUP, listOf("given", "where", "all", "suchThat?")) {
            sections ->
                GivenGroup(
                    givenSection =
                        neoEnsureNonNull(sections["given"], DEFAULT_GIVEN_SECTION) {
                            neoValidateGivenSection(it, errors, tracker)
                        },
                    whereSection =
                        neoEnsureNonNull(sections["where"], DEFAULT_WHERE_SECTION) {
                            neoValidateWhereSection(it, errors, tracker)
                        },
                    allSection =
                        neoEnsureNonNull(sections["all"], DEFAULT_ALL_SECTION) {
                            neoValidateAllSection(it, errors, tracker)
                        },
                    suchThatSection =
                        neoIfNonNull(sections["suchThat"]) {
                            neoValidateSuchThatSection(it, errors, tracker)
                        })
            }
        }
    }

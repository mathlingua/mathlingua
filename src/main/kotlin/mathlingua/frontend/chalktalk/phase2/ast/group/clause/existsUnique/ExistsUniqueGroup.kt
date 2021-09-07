/*
 * Copyright 2020 The MathLingua Authors
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

package mathlingua.frontend.chalktalk.phase2.ast.group.clause.existsUnique

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_EXISTS_UNIQUE_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_EXISTS_UNIQUE_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_SUCH_THAT_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.TwoPartNode
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.exists.SuchThatSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.exists.validateSuchThatSection
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

data class ExistsUniqueGroup(
    val existsUniqueSection: ExistsUniqueSection, val suchThatSection: SuchThatSection
) :
    TwoPartNode<ExistsUniqueSection, SuchThatSection>(
        existsUniqueSection, suchThatSection, ::ExistsUniqueGroup),
    Clause

fun isExistsUniqueGroup(node: Phase1Node) = firstSectionMatchesName(node, "existsUnique")

fun validateExistsUniqueGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "existsUnique", DEFAULT_EXISTS_UNIQUE_GROUP) {
        group ->
            identifySections(
                group, errors, DEFAULT_EXISTS_UNIQUE_GROUP, listOf("existsUnique", "suchThat")) {
            sections ->
                ExistsUniqueGroup(
                    existsUniqueSection =
                        ensureNonNull(sections["existsUnique"], DEFAULT_EXISTS_UNIQUE_SECTION) {
                            validateExistsUniqueSection(it, errors, tracker)
                        },
                    suchThatSection =
                        ensureNonNull(sections["suchThat"], DEFAULT_SUCH_THAT_SECTION) {
                            validateSuchThatSection(it, errors, tracker)
                        })
            }
        }
    }

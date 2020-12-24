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

package mathlingua.chalktalk.phase2.ast.group.clause.expands

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.ast.DEFAULT_AS_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_EXPANDS_SECTION
import mathlingua.chalktalk.phase2.ast.clause.Clause
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.common.TwoPartNode
import mathlingua.chalktalk.phase2.ast.neoTrack
import mathlingua.chalktalk.phase2.ast.section.neoEnsureNonNull
import mathlingua.chalktalk.phase2.ast.section.neoIdentifySections
import mathlingua.chalktalk.phase2.ast.validateGroup
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError

data class ExpandsGroup(val expandsSection: ExpandsSection, val asSection: AsSection) :
    TwoPartNode<ExpandsSection, AsSection>(expandsSection, asSection, ::ExpandsGroup), Clause

fun isExpandsGroup(node: Phase1Node) = firstSectionMatchesName(node, "expands")

fun validateExpandsGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    neoTrack(node, tracker) {
        validateGroup(node.resolve(), errors, "expands", DEFAULT_AS_SECTION) { group ->
            neoIdentifySections(group, errors, DEFAULT_AS_SECTION, listOf("expands", "as")) {
            sections ->
                ExpandsGroup(
                    expandsSection =
                        neoEnsureNonNull(sections["expands"], DEFAULT_EXPANDS_SECTION) {
                            validateExpandsSection(it, errors, tracker)
                        },
                    asSection =
                        neoEnsureNonNull(sections["as"], DEFAULT_AS_SECTION) {
                            validateAsSection(it, errors, tracker)
                        })
            }
        }
    }

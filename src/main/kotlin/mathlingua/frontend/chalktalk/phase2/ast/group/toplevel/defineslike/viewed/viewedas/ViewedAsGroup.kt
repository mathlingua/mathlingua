/*
 * Copyright 2021
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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewed.viewedas

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_VIA_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_VIEWED_AS_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_VIEWED_AS_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.clause.secondSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.TwoPartNode
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

data class ViewedAsGroup(val viewedAsSection: ViewedAsSection, val viaSection: ViaSection) :
    TwoPartNode<ViewedAsSection, ViaSection>(viewedAsSection, viaSection, ::ViewedAsGroup), Clause

fun isViewedAsGroup(node: Phase1Node) =
    firstSectionMatchesName(node, "as") && secondSectionMatchesName(node, "via")

fun validateViewedAsGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "as", DEFAULT_VIEWED_AS_GROUP) { group ->
            identifySections(group, errors, DEFAULT_VIEWED_AS_GROUP, listOf("as", "via")) {
            sections ->
                ViewedAsGroup(
                    viewedAsSection =
                        ensureNonNull(sections["as"], DEFAULT_VIEWED_AS_SECTION) {
                            validateViewedAsSection(it, errors, tracker)
                        },
                    viaSection =
                        ensureNonNull(sections["via"], DEFAULT_VIA_SECTION) {
                            validateViaSection(it, errors, tracker)
                        })
            }
        }
    }

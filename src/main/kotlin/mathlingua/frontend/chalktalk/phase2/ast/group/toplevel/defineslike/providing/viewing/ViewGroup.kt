/*
 * Copyright 2021 The MathLingua Authors
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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.viewing

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_VIA_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_VIEW_AS_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_VIEW_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_VIEW_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.FourPartNode
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.BySection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.validateBySection
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

internal data class ViewGroup(
    val viewSection: ViewSection,
    val viewAsSection: ViewAsSection,
    val viaSection: ViaSection,
    val bySection: BySection?
) :
    FourPartNode<ViewSection, ViewAsSection, ViaSection, BySection?>(
        viewSection, viewAsSection, viaSection, bySection, ::ViewGroup),
    Clause

internal fun isViewingAsGroup(node: Phase1Node) = firstSectionMatchesName(node, "view")

internal fun validateViewingAsGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "view", DEFAULT_VIEW_GROUP) { group ->
            identifySections(
                group, errors, DEFAULT_VIEW_GROUP, listOf("view", "as", "via", "by?")) { sections ->
                ViewGroup(
                    viewSection =
                        ensureNonNull(sections["view"], DEFAULT_VIEW_SECTION) {
                            validateViewSection(it, errors, tracker)
                        },
                    viewAsSection =
                        ensureNonNull(sections["as"], DEFAULT_VIEW_AS_SECTION) {
                            validateViewingAsSection(it, errors, tracker)
                        },
                    viaSection =
                        ensureNonNull(sections["via"], DEFAULT_VIA_SECTION) {
                            validateViaSection(it, errors, tracker)
                        },
                    bySection =
                        ifNonNull(sections["by"]) { validateBySection(it, errors, tracker) })
            }
        }
    }

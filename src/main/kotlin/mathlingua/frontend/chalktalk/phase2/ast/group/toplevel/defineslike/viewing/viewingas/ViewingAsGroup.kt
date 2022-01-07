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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.viewingas

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_VIA_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_VIEWED_AS_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_VIEWED_AS_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.clause.secondSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.ThreePartNode
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.BySection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.validateBySection
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

internal data class ViewingAsGroup(
    val viewingAsSection: ViewingAsSection, val viaSection: ViaSection, val bySection: BySection?
) :
    ThreePartNode<ViewingAsSection, ViaSection, BySection?>(
        viewingAsSection, viaSection, bySection, ::ViewingAsGroup),
    Clause

internal fun isViewingAsGroup(node: Phase1Node) =
    firstSectionMatchesName(node, "as") && secondSectionMatchesName(node, "via")

internal fun validateViewingAsGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "as", DEFAULT_VIEWED_AS_GROUP) { group ->
            identifySections(group, errors, DEFAULT_VIEWED_AS_GROUP, listOf("as", "via", "by?")) {
            sections ->
                ViewingAsGroup(
                    viewingAsSection =
                        ensureNonNull(sections["as"], DEFAULT_VIEWED_AS_SECTION) {
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

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

package mathlingua.frontend.chalktalk.phase2.ast.clause.have

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_BY_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_HAVE_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_HAVE_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.TwoPartNode
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.BySection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.validateBySection
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

data class HaveGroup(val haveSection: HaveSection, val bySection: BySection) :
    TwoPartNode<HaveSection, BySection>(haveSection, bySection, ::HaveGroup), Clause

fun isHaveGroup(node: Phase1Node) = firstSectionMatchesName(node, "have")

fun validateHaveGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "have", DEFAULT_HAVE_GROUP) { group ->
            identifySections(group, errors, DEFAULT_HAVE_GROUP, listOf("have", "by")) { sections ->
                HaveGroup(
                    haveSection =
                        ensureNonNull(sections["have"], DEFAULT_HAVE_SECTION) {
                            validateHaveSection(it, errors, tracker)
                        },
                    bySection =
                        ensureNonNull(sections["by"], DEFAULT_BY_SECTION) {
                            validateBySection(it, errors, tracker)
                        })
            }
        }
    }
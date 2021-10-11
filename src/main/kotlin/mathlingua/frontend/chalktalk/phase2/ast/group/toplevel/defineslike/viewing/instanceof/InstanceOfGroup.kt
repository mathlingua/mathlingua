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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.instanceof

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_INSTANCE_OF_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_INSTANCE_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_OF_SECTION
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

data class InstanceOfGroup(
    val instanceSection: InstanceSection, val ofSection: OfSection, val bySection: BySection?
) :
    ThreePartNode<InstanceSection, OfSection, BySection?>(
        instanceSection, ofSection, bySection, ::InstanceOfGroup),
    Clause

fun isInstanceOfGroup(node: Phase1Node) =
    firstSectionMatchesName(node, "instance") && secondSectionMatchesName(node, "of")

fun validateInstanceOfGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "instance", DEFAULT_INSTANCE_OF_GROUP) { group ->
            identifySections(
                group, errors, DEFAULT_INSTANCE_OF_GROUP, listOf("instance", "of", "by?")) {
            sections ->
                InstanceOfGroup(
                    instanceSection =
                        ensureNonNull(sections["instance"], DEFAULT_INSTANCE_SECTION) {
                            validateInstanceSection(it, errors, tracker)
                        },
                    ofSection =
                        ensureNonNull(sections["of"], DEFAULT_OF_SECTION) {
                            validateOfSection(it, errors, tracker)
                        },
                    bySection =
                        ifNonNull(sections["by"]) { validateBySection(it, errors, tracker) })
            }
        }
    }

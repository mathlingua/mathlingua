/*
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *    http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package mathlingua.frontend.chalktalk.phase2.ast.group.clause.inductively

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_INDUCTIVELY_FROM_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_INDUCTIVELY_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_INDUCTIVELY_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.TwoPartNode
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

data class InductivelyGroup(
    val inductivelySection: InductivelySection, val fromSection: InductivelyFromSection
) :
    TwoPartNode<InductivelySection, InductivelyFromSection>(
        inductivelySection, fromSection, ::InductivelyGroup),
    Clause

fun isInductivelyGroup(node: Phase1Node) = firstSectionMatchesName(node, "inductively")

fun validateInductivelyGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "inductively", DEFAULT_INDUCTIVELY_GROUP) { group ->
            identifySections(
                group, errors, DEFAULT_INDUCTIVELY_GROUP, listOf("inductively", "from")) {
            sections ->
                InductivelyGroup(
                    inductivelySection =
                        ensureNonNull(sections["inductively"], DEFAULT_INDUCTIVELY_SECTION) {
                            validateInductivelySection(it, errors, tracker)
                        },
                    fromSection =
                        ensureNonNull(sections["from"], DEFAULT_INDUCTIVELY_FROM_SECTION) {
                            validateInductivelyFromSection(it, errors, tracker)
                        })
            }
        }
    }

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

package mathlingua.chalktalk.phase2.ast.group.clause.inductively

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.ast.DEFAULT_INDUCTIVELY_FROM_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_INDUCTIVELY_GROUP
import mathlingua.chalktalk.phase2.ast.DEFAULT_INDUCTIVELY_SECTION
import mathlingua.chalktalk.phase2.ast.clause.Clause
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.common.TwoPartNode
import mathlingua.chalktalk.phase2.ast.neoTrack
import mathlingua.chalktalk.phase2.ast.section.neoEnsureNonNull
import mathlingua.chalktalk.phase2.ast.section.neoIdentifySections
import mathlingua.chalktalk.phase2.ast.validateGroup
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError

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
    neoTrack(node, tracker) {
        validateGroup(node.resolve(), errors, "inductively", DEFAULT_INDUCTIVELY_GROUP) { group ->
            neoIdentifySections(
                group, errors, DEFAULT_INDUCTIVELY_GROUP, listOf("inductively", "from")) {
            sections ->
                InductivelyGroup(
                    inductivelySection =
                        neoEnsureNonNull(sections["inductively"], DEFAULT_INDUCTIVELY_SECTION) {
                            validateInductivelySection(it, errors, tracker)
                        },
                    fromSection =
                        neoEnsureNonNull(sections["from"], DEFAULT_INDUCTIVELY_FROM_SECTION) {
                            validateInductivelyFromSection(it, errors, tracker)
                        })
            }
        }
    }

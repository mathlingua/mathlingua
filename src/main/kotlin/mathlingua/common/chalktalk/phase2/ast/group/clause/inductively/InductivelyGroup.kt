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

package mathlingua.common.chalktalk.phase2.ast.group.clause.inductively

import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase2.ast.clause.Clause
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.clause.validateDoubleSectionGroup
import mathlingua.common.chalktalk.phase2.ast.common.TwoPartNode
import mathlingua.common.support.MutableLocationTracker

data class InductivelyGroup(
    val inductivelySection: InductivelySection,
    val fromSection: InductivelyFromSection
) : TwoPartNode<InductivelySection, InductivelyFromSection>(
    inductivelySection,
    fromSection,
    ::InductivelyGroup
), Clause

fun isInductivelyGroup(node: Phase1Node) = firstSectionMatchesName(node, "inductively")

fun validateInductivelyGroup(node: Phase1Node, tracker: MutableLocationTracker) = validateDoubleSectionGroup(
    tracker,
    node,
    "inductively",
    ::validateInductivelySection,
    "from",
    ::validateInductivelyFromSection,
    ::InductivelyGroup
)

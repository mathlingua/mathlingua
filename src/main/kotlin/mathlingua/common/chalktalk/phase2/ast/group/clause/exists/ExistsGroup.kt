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

package mathlingua.common.chalktalk.phase2.ast.group.clause.exists

import mathlingua.common.support.MutableLocationTracker
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase2.ast.common.TwoPartNode
import mathlingua.common.chalktalk.phase2.ast.clause.Clause
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.clause.validateDoubleSectionGroup

data class ExistsGroup(
    val existsSection: ExistsSection,
    val suchThatSection: SuchThatSection
) : TwoPartNode<ExistsSection, SuchThatSection>(
    existsSection,
    suchThatSection,
    ::ExistsGroup
), Clause

fun isExistsGroup(node: Phase1Node) = firstSectionMatchesName(node, "exists")

fun validateExistsGroup(node: Phase1Node, tracker: MutableLocationTracker) = validateDoubleSectionGroup(
        tracker,
        node,
        "exists",
        ::validateExistsSection,
        "suchThat",
        ::validateSuchThatSection,
        ::ExistsGroup
)

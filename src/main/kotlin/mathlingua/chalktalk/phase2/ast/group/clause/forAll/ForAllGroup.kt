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

package mathlingua.chalktalk.phase2.ast.group.clause.forAll

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.ast.clause.Clause
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.clause.validateDoubleMidOptionalQuadrupleSectionGroup
import mathlingua.chalktalk.phase2.ast.common.FourPartNode
import mathlingua.chalktalk.phase2.ast.group.clause.If.ThenSection
import mathlingua.chalktalk.phase2.ast.group.clause.If.validateThenSection
import mathlingua.chalktalk.phase2.ast.group.clause.exists.SuchThatSection
import mathlingua.chalktalk.phase2.ast.group.clause.exists.validateSuchThatSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.WhereSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.validateWhereSection
import mathlingua.support.MutableLocationTracker

data class ForAllGroup(
    val forAllSection: ForAllSection,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection?,
    val thenSection: ThenSection
) :
    FourPartNode<ForAllSection, WhereSection?, SuchThatSection?, ThenSection>(
        forAllSection, whereSection, suchThatSection, thenSection, ::ForAllGroup),
    Clause

fun isForGroup(node: Phase1Node) = firstSectionMatchesName(node, "forAll")

fun validateForGroup(rawNode: Phase1Node, tracker: MutableLocationTracker) =
    validateDoubleMidOptionalQuadrupleSectionGroup(
        tracker,
        rawNode,
        "forAll",
        ::validateForSection,
        "where?",
        ::validateWhereSection,
        "suchThat?",
        ::validateSuchThatSection,
        "then",
        ::validateThenSection,
        ::ForAllGroup)

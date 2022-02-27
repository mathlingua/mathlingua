/*
 * Copyright 2020 The MathLingua Authors
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

package mathlingua.frontend.chalktalk.phase2.ast.group.clause.forAll

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_FOR_ALL_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_FOR_ALL_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_THEN_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.FourPartNode
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.ThenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.validateThenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.exists.SuchThatSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.exists.validateSuchThatSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.symbols.WhereSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.symbols.validateWhereSection
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.ParseError

internal data class ForAllGroup(
    val forAllSection: ForAllSection,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection?,
    val thenSection: ThenSection,
    override val row: Int,
    override val column: Int
) :
    FourPartNode<ForAllSection, WhereSection?, SuchThatSection?, ThenSection>(
        forAllSection, whereSection, suchThatSection, thenSection, row, column, ::ForAllGroup),
    Clause

internal fun isForGroup(node: Phase1Node) = firstSectionMatchesName(node, "forAll")

internal fun validateForGroup(node: Phase1Node, errors: MutableList<ParseError>) =
    validateGroup(node.resolve(), errors, "forAll", DEFAULT_FOR_ALL_GROUP) { group ->
        identifySections(
            group,
            errors,
            DEFAULT_FOR_ALL_GROUP,
            listOf("forAll", "where?", "suchThat?", "then")) { sections ->
            ForAllGroup(
                forAllSection =
                    ensureNonNull(sections["forAll"], DEFAULT_FOR_ALL_SECTION) {
                        validateForSection(it, errors)
                    },
                whereSection = ifNonNull(sections["where"]) { validateWhereSection(it, errors) },
                suchThatSection =
                    ifNonNull(sections["suchThat"]) { validateSuchThatSection(it, errors) },
                thenSection =
                    ensureNonNull(sections["then"], DEFAULT_THEN_SECTION) {
                        validateThenSection(it, errors)
                    },
                row = node.row,
                column = node.column)
        }
    }

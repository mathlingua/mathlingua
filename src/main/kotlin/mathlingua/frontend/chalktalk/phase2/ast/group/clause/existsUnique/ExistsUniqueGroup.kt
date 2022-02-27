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

package mathlingua.frontend.chalktalk.phase2.ast.group.clause.existsUnique

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_EXISTS_UNIQUE_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_EXISTS_UNIQUE_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_SUCH_THAT_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.ThreePartNode
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.exists.SuchThatSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.exists.validateSuchThatSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.symbols.WhereSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.symbols.validateWhereSection
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.ParseError

internal data class ExistsUniqueGroup(
    val existsUniqueSection: ExistsUniqueSection,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection?,
    override val row: Int,
    override val column: Int
) :
    ThreePartNode<ExistsUniqueSection, WhereSection?, SuchThatSection?>(
        existsUniqueSection, whereSection, suchThatSection, row, column, ::ExistsUniqueGroup),
    Clause

internal fun isExistsUniqueGroup(node: Phase1Node) = firstSectionMatchesName(node, "existsUnique")

internal fun validateExistsUniqueGroup(node: Phase1Node, errors: MutableList<ParseError>) =
    validateGroup(node.resolve(), errors, "existsUnique", DEFAULT_EXISTS_UNIQUE_GROUP) { group ->
        identifySections(
            group,
            errors,
            DEFAULT_EXISTS_UNIQUE_GROUP,
            listOf("existsUnique", "where?", "suchThat?")) { sections ->
            val result =
                ExistsUniqueGroup(
                    existsUniqueSection =
                        ensureNonNull(sections["existsUnique"], DEFAULT_EXISTS_UNIQUE_SECTION) {
                            validateExistsUniqueSection(it, errors)
                        },
                    whereSection =
                        ifNonNull(sections["where"]) { validateWhereSection(it, errors) },
                    suchThatSection =
                        ensureNonNull(sections["suchThat"], DEFAULT_SUCH_THAT_SECTION) {
                            validateSuchThatSection(it, errors)
                        },
                    row = node.row,
                    column = node.column)

            if (result.whereSection == null && result.suchThatSection == null) {
                errors.add(
                    ParseError(
                        message = "At least one of `where:` or `suchThat:` needs to be specified.",
                        row = result.row,
                        column = result.column))
                DEFAULT_EXISTS_UNIQUE_GROUP
            } else {
                result
            }
        }
    }

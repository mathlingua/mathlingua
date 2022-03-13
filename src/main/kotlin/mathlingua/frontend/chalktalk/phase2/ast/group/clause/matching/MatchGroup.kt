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

package mathlingua.frontend.chalktalk.phase2.ast.group.clause.matching

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_MATCH_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_MATCH_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.OnePartNode
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.ParseError

internal data class MatchGroup(
    val matchSection: MatchSection, override val row: Int, override val column: Int
) : OnePartNode<MatchSection>(matchSection, row, column, ::MatchGroup), Clause

internal fun isMatchGroup(node: Phase1Node) = firstSectionMatchesName(node, "match")

internal fun validateMatchGroup(node: Phase1Node, errors: MutableList<ParseError>) =
    validateGroup(node.resolve(), errors, "match", DEFAULT_MATCH_GROUP) { group ->
        identifySections(group, errors, DEFAULT_MATCH_GROUP, listOf("match")) { sections ->
            MatchGroup(
                matchSection =
                    ensureNonNull(sections["match"], DEFAULT_MATCH_SECTION) {
                        validateMatchingSection(it, errors)
                    },
                row = node.row,
                column = node.column)
        }
    }

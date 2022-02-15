/*
 * Copyright 2022 The MathLingua Authors
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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.symbols

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_SYMBOLS_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_SYMBOLS_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.TwoPartNode
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

internal data class SymbolsGroup(
    val symbolsSection: SymbolsSection, val whereSection: WhereSection?
) :
    TwoPartNode<SymbolsSection, WhereSection?>(symbolsSection, whereSection, ::SymbolsGroup),
    Clause

internal fun isSymbolsGroup(node: Phase1Node) = firstSectionMatchesName(node, "symbols")

internal fun validateSymbolsGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "symbols", DEFAULT_SYMBOLS_GROUP) { group ->
            identifySections(group, errors, DEFAULT_SYMBOLS_GROUP, listOf("symbols", "where?")) {
            sections ->
                SymbolsGroup(
                    symbolsSection =
                        ensureNonNull(sections["symbols"], DEFAULT_SYMBOLS_SECTION) {
                            validateSymbolsSection(it, errors, tracker)
                        },
                    whereSection =
                        ifNonNull(sections["where"]) { validateWhereSection(it, errors, tracker) })
            }
        }
    }

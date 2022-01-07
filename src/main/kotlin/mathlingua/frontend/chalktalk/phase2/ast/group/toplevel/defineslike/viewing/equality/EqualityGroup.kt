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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.equality

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_BETWEEN_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_EQUALITY_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_EQUALITY_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_PROVIDED_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.ThreePartNode
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.ProvidedSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.validateProvidedSection
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

internal data class EqualityGroup(
    val equalitySection: EqualitySection,
    val betweenSection: BetweenSection,
    val providedSection: ProvidedSection
) :
    ThreePartNode<EqualitySection, BetweenSection, ProvidedSection>(
        equalitySection, betweenSection, providedSection, ::EqualityGroup),
    Clause

internal fun isEqualityGroup(node: Phase1Node) = firstSectionMatchesName(node, "equality")

internal fun validateEqualityGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "equality", DEFAULT_EQUALITY_GROUP) { group ->
            identifySections(
                group, errors, DEFAULT_EQUALITY_GROUP, listOf("equality", "between", "provided")) {
            sections ->
                EqualityGroup(
                    equalitySection =
                        ensureNonNull(sections["equality"], DEFAULT_EQUALITY_SECTION) {
                            validateEqualitySection(it, errors, tracker)
                        },
                    betweenSection =
                        ensureNonNull(sections["between"], DEFAULT_BETWEEN_SECTION) {
                            validateBetweenSection(it, errors, tracker)
                        },
                    providedSection =
                        ensureNonNull(sections["provided"], DEFAULT_PROVIDED_SECTION) {
                            validateProvidedSection(it, errors, tracker)
                        })
            }
        }
    }

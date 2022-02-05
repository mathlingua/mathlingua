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

package mathlingua.frontend.chalktalk.phase2.ast.group.clause.generated

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_GENERATED_FROM_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_GENERATED_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_GENERATED_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.ThreePartNode
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.validateWhenSection
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

internal data class GeneratedGroup(
    val generatedSection: GeneratedSection,
    val generatedFromSection: GeneratedFromSection,
    val whenSection: WhenSection?
) :
    ThreePartNode<GeneratedSection, GeneratedFromSection, WhenSection?>(
        generatedSection, generatedFromSection, whenSection, ::GeneratedGroup),
    Clause

internal fun isGeneratedGroup(node: Phase1Node) = firstSectionMatchesName(node, "generated")

internal fun validateGeneratedGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "generated", DEFAULT_GENERATED_GROUP) { group ->
            identifySections(
                group, errors, DEFAULT_GENERATED_GROUP, listOf("generated", "from", "when?")) {
            sections ->
                GeneratedGroup(
                    generatedSection =
                        ensureNonNull(sections["generated"], DEFAULT_GENERATED_SECTION) {
                            validateGeneratedSection(it, errors, tracker)
                        },
                    generatedFromSection =
                        ensureNonNull(sections["from"], DEFAULT_GENERATED_FROM_SECTION) {
                            validateGeneratedFromSection(it, errors, tracker)
                        },
                    whenSection =
                        ifNonNull(sections["when"]) { validateWhenSection(it, errors, tracker) })
            }
        }
    }

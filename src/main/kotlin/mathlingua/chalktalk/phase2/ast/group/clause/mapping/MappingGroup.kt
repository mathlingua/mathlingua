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

package mathlingua.chalktalk.phase2.ast.group.clause.mapping

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.DEFAULT_AS_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_FROM_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_MAPPING_GROUP
import mathlingua.chalktalk.phase2.ast.DEFAULT_MAPPING_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_TO_SECTION
import mathlingua.chalktalk.phase2.ast.clause.Clause
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.clause.expands.AsSection
import mathlingua.chalktalk.phase2.ast.group.clause.expands.neoValidateAsSection
import mathlingua.chalktalk.phase2.ast.neoTrack
import mathlingua.chalktalk.phase2.ast.neoValidateGroup
import mathlingua.chalktalk.phase2.ast.section.neoEnsureNonNull
import mathlingua.chalktalk.phase2.ast.section.neoIdentifySections
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError

data class MappingGroup(
    val mappingSection: MappingSection,
    val fromSection: FromSection,
    val toSection: ToSection,
    val asSection: AsSection
) : Clause {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(mappingSection)
        fn(fromSection)
        fn(toSection)
        fn(asSection)
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
        mathlingua.chalktalk.phase2.ast.clause.toCode(
            writer, isArg, indent, mappingSection, fromSection, toSection, asSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            MappingGroup(
                mappingSection = mappingSection.transform(chalkTransformer) as MappingSection,
                fromSection = fromSection.transform(chalkTransformer) as FromSection,
                toSection = toSection.transform(chalkTransformer) as ToSection,
                asSection = asSection.transform(chalkTransformer) as AsSection))
}

fun isMappingGroup(node: Phase1Node) = firstSectionMatchesName(node, "mapping")

fun neoValidateMappingGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    neoTrack(node, tracker) {
        neoValidateGroup(node.resolve(), errors, "mapping", DEFAULT_MAPPING_GROUP) { group ->
            neoIdentifySections(
                group, errors, DEFAULT_MAPPING_GROUP, listOf("mapping", "from", "to", "as")) {
            sections ->
                MappingGroup(
                    mappingSection =
                        neoEnsureNonNull(sections["mapping"], DEFAULT_MAPPING_SECTION) {
                            neoValidateMappingSection(it, errors, tracker)
                        },
                    fromSection =
                        neoEnsureNonNull(sections["from"], DEFAULT_FROM_SECTION) {
                            neoValidateFromSection(it, errors, tracker)
                        },
                    toSection =
                        neoEnsureNonNull(sections["to"], DEFAULT_TO_SECTION) {
                            neoValidateToSection(it, errors, tracker)
                        },
                    asSection =
                        neoEnsureNonNull(sections["as"], DEFAULT_AS_SECTION) {
                            neoValidateAsSection(it, errors, tracker)
                        })
            }
        }
    }

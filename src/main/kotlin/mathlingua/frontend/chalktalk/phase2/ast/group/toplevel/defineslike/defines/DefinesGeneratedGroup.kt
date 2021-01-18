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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines

import mathlingua.backend.transform.signature
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_DEFINES_GENERATED_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_DEFINES_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_GENERATED_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_ID_STATEMENT
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_WRITTEN_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.sectionsMatchNames
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.getId
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.validateWrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.WhereSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateMetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.validateUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.validateWhenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.validateWhereSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

data class DefinesGeneratedGroup(
    override val signature: String?,
    override val id: IdStatement,
    override val definesSection: DefinesSection,
    val whereSection: WhereSection?,
    val whenSection: WhenSection?,
    val generatedSection: GeneratedSection,
    val usingSection: UsingSection?,
    override val writtenSection: WrittenSection,
    override val metaDataSection: MetaDataSection?
) : DefinesGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(definesSection)
        if (whereSection != null) {
            fn(whereSection)
        }
        if (whenSection != null) {
            fn(whenSection)
        }
        fn(generatedSection)
        if (usingSection != null) {
            fn(usingSection)
        }
        fn(writtenSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        val sections =
            mutableListOf(
                definesSection,
                whereSection,
                whenSection,
                generatedSection,
                writtenSection,
                metaDataSection)
        return topLevelToCode(writer, isArg, indent, id, *sections.toTypedArray())
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            DefinesGeneratedGroup(
                signature = signature,
                id = id.transform(chalkTransformer) as IdStatement,
                definesSection = definesSection.transform(chalkTransformer) as DefinesSection,
                whereSection = whereSection?.transform(chalkTransformer) as WhereSection?,
                whenSection = whenSection?.transform(chalkTransformer) as WhenSection?,
                generatedSection = generatedSection.transform(chalkTransformer) as GeneratedSection,
                usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
                writtenSection = writtenSection.transform(chalkTransformer) as WrittenSection,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?))
}

fun isDefinesGeneratedGroup(node: Phase1Node) = sectionsMatchNames(node, "Defines", "generated")

fun validateDefinesGeneratedGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "Defines", DEFAULT_DEFINES_GENERATED_GROUP) { group ->
            identifySections(
                group,
                errors,
                DEFAULT_DEFINES_GENERATED_GROUP,
                listOf(
                    "Defines", "where?", "when?", "generated", "using?", "written", "Metadata?")) {
            sections ->
                val id = getId(group, errors, DEFAULT_ID_STATEMENT, tracker)
                DefinesGeneratedGroup(
                    signature = id.signature(),
                    id = id,
                    definesSection =
                        ensureNonNull(sections["Defines"], DEFAULT_DEFINES_SECTION) {
                            validateDefinesSection(it, errors, tracker)
                        },
                    whereSection =
                        ifNonNull(sections["where"]) { validateWhereSection(it, errors, tracker) },
                    whenSection =
                        ifNonNull(sections["when"]) { validateWhenSection(it, errors, tracker) },
                    generatedSection =
                        ensureNonNull(sections["generated"], DEFAULT_GENERATED_SECTION) {
                            validateGeneratedSection(it, errors, tracker)
                        },
                    usingSection =
                        ifNonNull(sections["using"]) { validateUsingSection(it, errors, tracker) },
                    writtenSection =
                        ensureNonNull(sections["written"], DEFAULT_WRITTEN_SECTION) {
                            validateWrittenSection(it, errors, tracker)
                        },
                    metaDataSection =
                        ifNonNull(sections["Metadata"]) {
                            validateMetaDataSection(it, errors, tracker)
                        })
            }
        }
    }

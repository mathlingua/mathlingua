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

package mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.states

import mathlingua.common.support.MutableLocationTracker
import mathlingua.common.support.Validation
import mathlingua.common.chalktalk.phase1.ast.Group
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.common.chalktalk.phase2.ast.clause.Validator
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.clause.validateIdMetadataGroup
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.DefinesStatesOrViews
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.validateUsingSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.validateWhenSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.validateWrittenSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.common.transform.signature
import mathlingua.common.support.validationSuccess

data class StatesGroup(
    val signature: String?,
    val id: IdStatement,
    val statesSection: StatesSection,
    val whenSection: WhenSection?,
    val thatSection: ThatSection,
    val usingSection: UsingSection?,
    val writtenSection: WrittenSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection), DefinesStatesOrViews {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(statesSection)
        if (whenSection != null) {
            fn(whenSection)
        }
        fn(thatSection)
        if (usingSection != null) {
            fn(usingSection)
        }
        if (writtenSection != null) {
            fn(writtenSection)
        }
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        val sections = mutableListOf(statesSection, whenSection)
        sections.add(thatSection)
        sections.add(usingSection)
        sections.add(writtenSection)
        sections.add(metaDataSection)
        return topLevelToCode(
                writer,
                isArg,
                indent,
                id,
                *sections.toTypedArray()
        )
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(
        StatesGroup(
            signature = signature,
            id = id.transform(chalkTransformer) as IdStatement,
            statesSection = statesSection.transform(chalkTransformer) as StatesSection,
            whenSection = whenSection?.transform(chalkTransformer) as WhenSection?,
            thatSection = chalkTransformer(thatSection) as ThatSection,
            usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
            writtenSection = writtenSection?.transform(chalkTransformer) as WrittenSection?,
            metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?
    )
    )
}

fun isStatesGroup(node: Phase1Node) = firstSectionMatchesName(node, "States")

fun validateStatesGroup(groupNode: Group, tracker: MutableLocationTracker): Validation<StatesGroup> = validateIdMetadataGroup(
    tracker, groupNode,
    listOf(
        Validator(
            name = "States",
            optional = false,
            ::validateStatesSection
        ),
        Validator(
            name = "when",
            optional = true,
            ::validateWhenSection
        ),
        Validator(
            name = "that",
            optional = false,
            ::validateThatSection
        ),
        Validator(
            name = "using",
            optional = true,
            ::validateUsingSection
        ),
        Validator(
            name = "written",
            optional = true,
            ::validateWrittenSection
        )
    )
) { id, sections, metaDataSection ->
    validationSuccess(
        tracker,
        groupNode,
        StatesGroup(
            id.signature(),
            id,
            sections["States"] as StatesSection,
            sections["when"] as WhenSection?,
            sections["that"] as ThatSection,
            sections["using"] as UsingSection?,
            sections["written"] as WrittenSection?,
            metaDataSection
        )
    )
}

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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_SPECIFY_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_SPECIFY_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.ParseError

internal interface NumberGroup : Phase2Node

internal data class SpecifyGroup(
    val specifySection: SpecifySection, override val row: Int, override val column: Int
) : TopLevelGroup(null) {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(specifySection)
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
        topLevelToCode(this, writer, isArg, indent, null, specifySection, metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            SpecifyGroup(specifySection.transform(chalkTransformer) as SpecifySection, row, column))
}

internal fun isSpecifyGroup(node: Phase1Node) = firstSectionMatchesName(node, "Specify")

internal fun validateSpecifyGroup(node: Phase1Node, errors: MutableList<ParseError>) =
    validateGroup(node.resolve(), errors, "Specify", DEFAULT_SPECIFY_GROUP) { group ->
        identifySections(group, errors, DEFAULT_SPECIFY_GROUP, listOf("Specify")) { sections ->
            SpecifyGroup(
                specifySection =
                    ensureNonNull(sections["Specify"], DEFAULT_SPECIFY_SECTION) {
                        validateSpecifySection(it, errors)
                    },
                row = node.row,
                column = node.column)
        }
    }

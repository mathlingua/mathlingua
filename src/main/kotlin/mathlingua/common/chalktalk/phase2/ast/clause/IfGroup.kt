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

package mathlingua.common.chalktalk.phase2.ast.clause

import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.section.IfSection
import mathlingua.common.chalktalk.phase2.ast.section.ThenSection
import mathlingua.common.chalktalk.phase2.ast.section.validateIfSection
import mathlingua.common.chalktalk.phase2.ast.section.validateThenSection

data class IfGroup(
    val ifSection: IfSection,
    val thenSection: ThenSection,
    override var row: Int,
    override var column: Int
) : Clause {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(ifSection)
        fn(thenSection)
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
            toCode(writer, isArg, indent, ifSection, thenSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(IfGroup(
            ifSection = ifSection.transform(chalkTransformer) as IfSection,
            thenSection = thenSection.transform(chalkTransformer) as ThenSection,
            row = row,
            column = column
    ))
}

fun isIfGroup(node: Phase1Node) = firstSectionMatchesName(node, "if")

fun validateIfGroup(node: Phase1Node) = validateDoubleSectionGroup(
        node,
        "if",
        ::validateIfSection,
        "then",
        ::validateThenSection,
        ::IfGroup
)

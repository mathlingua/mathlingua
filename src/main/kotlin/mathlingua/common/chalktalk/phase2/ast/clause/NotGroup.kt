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
import mathlingua.common.chalktalk.phase2.ast.section.NotSection
import mathlingua.common.chalktalk.phase2.ast.section.validateNotSection

fun isNotGroup(node: Phase1Node) = firstSectionMatchesName(node, "not")

fun validateNotGroup(node: Phase1Node) = validateSingleSectionGroup(
        node, "not", ::NotGroup,
        ::validateNotSection
)

data class NotGroup(
    val notSection: NotSection,
    override var row: Int,
    override var column: Int
) : Clause {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = fn(notSection)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
            notSection.toCode(isArg, indent, writer)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(NotGroup(
            notSection = notSection.transform(chalkTransformer) as NotSection,
            row = row,
            column = column
    ))
}

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

package mathlingua.common.chalktalk.phase2.ast.group.clause.mapping

import mathlingua.common.chalktalk.phase1.ast.*
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.Clause
import mathlingua.common.chalktalk.phase2.ast.clause.Validator
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.clause.validateGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.expands.AsSection
import mathlingua.common.chalktalk.phase2.ast.group.clause.expands.validateAsSection
import mathlingua.common.support.MutableLocationTracker
import mathlingua.common.support.validationSuccess

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
        mathlingua.common.chalktalk.phase2.ast.clause.toCode(
            writer,
            isArg,
            indent,
            mappingSection,
            fromSection,
            toSection,
            asSection
        )

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(
        MappingGroup(
        mappingSection = mappingSection.transform(chalkTransformer) as MappingSection,
        fromSection = fromSection.transform(chalkTransformer) as FromSection,
        toSection = toSection.transform(chalkTransformer) as ToSection,
        asSection = asSection.transform(chalkTransformer) as AsSection
    )
    )
}

fun isMappingGroup(node: Phase1Node) = firstSectionMatchesName(node, "mapping")

fun validateMappingGroup(rawNode: Phase1Node, tracker: MutableLocationTracker) = validateGroup(
    tracker,
    rawNode,
    listOf(
        Validator(
            name = "mapping",
            optional = false,
            validate = ::validateMappingSection
        ),
        Validator(
            name = "from",
            optional = false,
            validate = ::validateFromSection
        ),
        Validator(
            name = "to",
            optional = false,
            validate = ::validateToSection
        ),
        Validator(
            name = "as",
            optional = false,
            validate = ::validateAsSection
        )
    )
) {
    validationSuccess(tracker, rawNode, MappingGroup(
        mappingSection = it["mapping"] as MappingSection,
        fromSection = it["from"] as FromSection,
        toSection = it["to"] as ToSection,
        asSection = it["as"] as AsSection
    ))
}

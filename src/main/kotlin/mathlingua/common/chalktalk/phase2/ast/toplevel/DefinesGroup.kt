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

package mathlingua.common.chalktalk.phase2.ast.toplevel

import mathlingua.common.MutableLocationTracker
import mathlingua.common.ParseError
import mathlingua.common.Validation
import mathlingua.common.ValidationFailure
import mathlingua.common.ValidationSuccess
import mathlingua.common.chalktalk.phase1.ast.Group
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.AbstractionNode
import mathlingua.common.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.section.*
import mathlingua.common.chalktalk.phase2.ast.metadata.section.MetaDataSection
import mathlingua.common.textalk.Command
import mathlingua.common.validationFailure
import mathlingua.common.validationSuccess

data class DefinesGroup(
    val signature: String?,
    val id: IdStatement,
    val definesSection: DefinesSection,
    val assumingSection: AssumingSection?,
    val meansSections: List<MeansSection>,
    val aliasSection: AliasSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(definesSection)
        if (assumingSection != null) {
            fn(assumingSection)
        }
        meansSections.forEach(fn)
        if (aliasSection != null) {
            fn(aliasSection)
        }
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        val sections = mutableListOf(definesSection, assumingSection)
        sections.addAll(meansSections)
        sections.add(metaDataSection)
        return topLevelToCode(
                writer,
                isArg,
                indent,
                id,
                *sections.toTypedArray()
        )
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(DefinesGroup(
            signature = signature,
            id = id.transform(chalkTransformer) as IdStatement,
            definesSection = definesSection.transform(chalkTransformer) as DefinesSection,
            assumingSection = assumingSection?.transform(chalkTransformer) as AssumingSection?,
            meansSections = meansSections.map { chalkTransformer(it) as MeansSection },
            aliasSection = aliasSection?.transform(chalkTransformer) as AliasSection?,
            metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?
    ))
}

fun isDefinesGroup(node: Phase1Node) = firstSectionMatchesName(node, "Defines")

fun validateDefinesGroup(groupNode: Group, tracker: MutableLocationTracker): Validation<DefinesGroup> {
    val validation = validateDefinesLikeGroup(
        tracker,
        groupNode,
        "Defines",
        ::validateDefinesSection,
        "means",
        ::validateMeansSection,
        ::DefinesGroup
    )

    if (validation is ValidationFailure) {
        return validation
    }

    val errors = mutableListOf<ParseError>()
    val defines = (validation as ValidationSuccess).value
    val idParen = if (defines.id.texTalkRoot is ValidationSuccess) {
        val commandParts = (defines.id.texTalkRoot.value.children.find {
            it is Command
        } as Command?)?.parts ?: emptyList()

        val partsWithParens = commandParts.filter { it.paren != null }
        if (partsWithParens.isNotEmpty() && partsWithParens.size != 1) {
            val location = tracker.getLocationOf(defines.id)
            errors.add(
                ParseError(
                    message = "A signature can only contain zero or one set of parens",
                    row = location?.row ?: -1,
                    column = location?.column ?: -1
                )
            )
        }

        if (partsWithParens.isEmpty()) {
            null
        } else {
            partsWithParens[0].paren
        }
    } else {
        null
    }

    if (errors.isNotEmpty()) {
        return validationFailure(errors)
    } else if (idParen == null) {
        return validationSuccess(defines)
    }

    // it is ok for the id to not have a () but the definition to
    // contain one such as
    //
    //   [\function]
    //   Defines: f(x)
    //   means: ...

    // if the targets is empty then `validation` above would have been
    // a ValidationFailure and the code would not have reached this point
    val target = defines.definesSection.targets[0]
    val location = tracker.getLocationOf(target)

    if (target !is AbstractionNode) {
        errors.add(
            ParseError(
                message = "If the signature of a Defines contains a parens then it must define " +
                    "a function type with a matching signature in parens",
                row = location?.row ?: -1,
                column = location?.column ?: -1
            ))
    } else {
        if (target.abstraction.isVarArgs ||
            target.abstraction.isEnclosed ||
            !target.abstraction.subParams.isNullOrEmpty() ||
            target.abstraction.parts.size != 1) {
            errors.add(
                ParseError(
                    message = "If the signature of a Defines contains a parens then " +
                        "the function type it defines cannot describe a sequence like, set like, or variadic form.",
                    row = location?.row ?: -1,
                    column = location?.column ?: -1
                )
            )
        } else {
            val func = target.abstraction.parts[0]
            val idForm = idParen.toCode().replace(" ", "")
            val defForm = func.toCode().replace(func.name.text, "").replace(" ", "")
            if (idForm != defForm) {
                errors.add(
                    ParseError(
                        message = "If the signature of a Defines contains a parens then " +
                            "then the defines must define a function like that has the exact same " +
                            "signature as the parens in the signature of the Defines.",
                        row = location?.row ?: -1,
                        column = location?.column ?: -1
                    )
                )
            }
        }
    }

    return if (errors.isEmpty()) {
        validationSuccess(defines)
    } else {
        validationFailure(errors)
    }
}

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

package mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines

import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError
import mathlingua.support.ValidationSuccess
import mathlingua.chalktalk.phase1.ast.Group
import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase1.ast.getColumn
import mathlingua.chalktalk.phase1.ast.getRow
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.clause.AbstractionNode
import mathlingua.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.chalktalk.phase2.ast.clause.Validator
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.clause.validateIdMetadataGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.DefinesStatesOrViews
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.validateWrittenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.validateUsingSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.validateWhenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.textalk.Command
import mathlingua.transform.signature
import mathlingua.support.validationFailure
import mathlingua.support.validationSuccess

data class DefinesGroup(
    val signature: String?,
    val id: IdStatement,
    val definesSection: DefinesSection,
    val whenSection: WhenSection?,
    val meansSection: MeansSection?,
    val evaluatedSection: EvaluatedSection?,
    val usingSection: UsingSection?,
    val writtenSection: WrittenSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection), DefinesStatesOrViews {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(definesSection)
        if (whenSection != null) {
            fn(whenSection)
        }
        if (meansSection != null) {
            fn(meansSection)
        }
        if (evaluatedSection != null) {
            fn(evaluatedSection)
        }
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
        val sections = mutableListOf(definesSection, whenSection)
        sections.add(meansSection)
        sections.add(evaluatedSection)
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
        DefinesGroup(
            signature = signature,
            id = id.transform(chalkTransformer) as IdStatement,
            definesSection = definesSection.transform(chalkTransformer) as DefinesSection,
            whenSection = whenSection?.transform(chalkTransformer) as WhenSection?,
            meansSection = meansSection?.transform(chalkTransformer) as MeansSection?,
            evaluatedSection = evaluatedSection?.transform(chalkTransformer) as EvaluatedSection?,
            usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
            writtenSection = writtenSection?.transform(chalkTransformer) as WrittenSection?,
            metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?
    )
    )
}

fun isDefinesGroup(node: Phase1Node) = firstSectionMatchesName(node, "Defines")

fun validateDefinesGroup(groupNode: Group, tracker: MutableLocationTracker) = validateIdMetadataGroup(
    tracker,
    groupNode,
    listOf(
        Validator(
            name = "Defines",
            optional = true,
            ::validateDefinesSection
        ),
        Validator(
            name = "when",
            optional = true,
            ::validateWhenSection
        ),
        Validator(
            name = "means",
            optional = true,
            ::validateMeansSection
        ),
        Validator(
            name = "evaluated",
            optional = true,
            ::validateEvaluatedSection
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
    val errors = mutableListOf<ParseError>()
    val idParen = if (id.texTalkRoot is ValidationSuccess) {
        val commandParts = (id.texTalkRoot.value.children.find {
            it is Command
        } as Command?)?.parts ?: emptyList()

        val partsWithParens = commandParts.filter { it.paren != null }
        if (partsWithParens.isNotEmpty() && partsWithParens.size != 1) {
            val location = tracker.getLocationOf(id)
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

    val definesSection = sections["Defines"] as DefinesSection

    if (idParen != null) {
        // it is ok for the id to not have a () but the definition to
        // contain one such as
        //
        //   [\function]
        //   Defines: f(x)
        //   means: ...

        // if the targets is empty then `validation` above would have been
        // a ValidationFailure and the code would not have reached this point
        val target = definesSection.targets[0]
        val location = tracker.getLocationOf(target)

        if (target !is AbstractionNode) {
            errors.add(
                ParseError(
                    message = "If the signature of a Defines contains a parens then it must define " +
                        "a function type with a matching signature in parens",
                    row = location?.row ?: -1,
                    column = location?.column ?: -1
                )
            )
        } else {
            if (target.abstraction.isVarArgs ||
                target.abstraction.isEnclosed ||
                !target.abstraction.subParams.isNullOrEmpty() ||
                target.abstraction.parts.size != 1
            ) {
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
    }

    val meansSection = sections["means"] as MeansSection?
    val evaluatedSection = sections["evaluated"] as EvaluatedSection?
    if (meansSection == null && evaluatedSection == null) {
        errors.add(
            ParseError(
                "A Defines must have either a 'means' or 'computes' section.  " +
                    "Either they were both omitted or they contain errors.",
                getRow(groupNode), getColumn(groupNode)
            )
        )
    }

    if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else {
        validationSuccess(
            tracker,
            groupNode,
            DefinesGroup(
                id.signature(),
                id,
                definesSection,
                sections["when"] as WhenSection?,
                meansSection,
                evaluatedSection,
                sections["using"] as UsingSection?,
                sections["written"] as WrittenSection?,
                metaDataSection
            )
        )
    }
}

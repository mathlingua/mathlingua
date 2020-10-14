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

package mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.defines

import mathlingua.common.support.MutableLocationTracker
import mathlingua.common.support.ParseError
import mathlingua.common.support.Validation
import mathlingua.common.support.ValidationFailure
import mathlingua.common.support.ValidationSuccess
import mathlingua.common.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.common.chalktalk.phase1.ast.Group
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase1.ast.Phase1Token
import mathlingua.common.chalktalk.phase1.ast.Section
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.AbstractionNode
import mathlingua.common.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.clause.validateIdStatement
import mathlingua.common.chalktalk.phase2.ast.section.*
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateMetaDataSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.DefinesRepresentsOrViews
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.validateWrittenSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.validateUsingSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.validateWhenSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.common.textalk.Command
import mathlingua.common.transform.signature
import mathlingua.common.support.validationFailure
import mathlingua.common.support.validationSuccess

data class DefinesGroup(
    val signature: String?,
    val id: IdStatement,
    val definesSection: DefinesSection,
    val whenSection: WhenSection?,
    val meansSection: MeansSection?,
    val computesSection: ComputesSection?,
    val usingSection: UsingSection?,
    val writtenSection: WrittenSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection), DefinesRepresentsOrViews {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(definesSection)
        if (whenSection != null) {
            fn(whenSection)
        }
        if (meansSection != null) {
            fn(meansSection)
        }
        if (computesSection != null) {
            fn(computesSection)
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
        sections.add(computesSection)
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
            computesSection = computesSection?.transform(chalkTransformer) as ComputesSection?,
            usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
            writtenSection = writtenSection?.transform(chalkTransformer) as WrittenSection?,
            metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?
    )
    )
}

fun isDefinesGroup(node: Phase1Node) = firstSectionMatchesName(node, "Defines")

fun validateDefinesGroup(groupNode: Group, tracker: MutableLocationTracker): Validation<DefinesGroup> {
    val validation = validateDefinesGroupImpl(groupNode, tracker)
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
            )
        )
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

private fun validateDefinesGroupImpl(
    groupNode: Group,
    tracker: MutableLocationTracker
): Validation<DefinesGroup> {
    val errors = ArrayList<ParseError>()
    val group = groupNode.resolve()
    var id: IdStatement? = null
    if (group.id != null) {
        val (rawText, _, row, column) = group.id
        // The id token is of type Id and the text is of the form "[...]"
        // Convert it to look like a statement.
        val statementText = "'" + rawText.substring(1, rawText.length - 1) + "'"
        val stmtToken = Phase1Token(
            statementText, ChalkTalkTokenType.Statement,
            row, column
        )
        when (val idValidation = validateIdStatement(stmtToken, tracker)) {
            is ValidationSuccess -> id = idValidation.value
            is ValidationFailure -> errors.addAll(idValidation.errors)
        }
    } else {
        val type = if (group.sections.isNotEmpty()) {
            group.sections.first().name.text
        } else {
            "Defines or Represents"
        }
        errors.add(
            ParseError(
                "A $type must have an Id",
                getRow(group), getColumn(group)
            )
        )
    }

    val sections = group.sections

    val sectionMap: Map<String, List<Section>>
    try {
        sectionMap = identifySections(
            sections,
            "Defines", "when?", "means?", "computes?",
            "using?", "written?", "Metadata?"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return validationFailure(errors)
    }

    val definesLike = sectionMap["Defines"]!!
    val whenNode = sectionMap["when"] ?: emptyList()
    val means = sectionMap["means"]
    val computes = sectionMap["computes"]
    val using = sectionMap["using"] ?: emptyList()
    val written = sectionMap["written"] ?: emptyList()
    val metadata = sectionMap["Metadata"] ?: emptyList()

    var definesLikeSection: DefinesSection? = null
    when (val definesLikeValidation = validateDefinesSection(definesLike[0], tracker)) {
        is ValidationSuccess -> definesLikeSection = definesLikeValidation.value
        is ValidationFailure -> errors.addAll(definesLikeValidation.errors)
    }

    var whenSection: WhenSection? = null
    if (whenNode.isNotEmpty()) {
        when (val assumingValidation = validateWhenSection(whenNode[0], tracker)) {
            is ValidationSuccess -> whenSection = assumingValidation.value
            is ValidationFailure -> errors.addAll(assumingValidation.errors)
        }
    }

    var meansSection: MeansSection? = null
    if (means != null) {
        when (val endValidation = validateMeansSection(means[0], tracker)) {
            is ValidationSuccess -> meansSection = endValidation.value
            is ValidationFailure -> errors.addAll(endValidation.errors)
        }
    }

    var computesSection: ComputesSection? = null
    if (computes != null) {
        when (val computesValidation = validateComputesSection(computes[0], tracker)) {
            is ValidationSuccess -> computesSection = computesValidation.value
            is ValidationFailure -> errors.addAll(computesValidation.errors)
        }
    }

    var usingSection: UsingSection? = null
    if (using.isNotEmpty()) {
        when (val aliasValidation = validateUsingSection(using[0], tracker)) {
            is ValidationSuccess -> usingSection = aliasValidation.value
            is ValidationFailure -> errors.addAll(aliasValidation.errors)
        }
    }

    var writtenSection: WrittenSection? = null
    if (written.isNotEmpty()) {
        when (val writtenValidation = validateWrittenSection(written[0], tracker)) {
            is ValidationSuccess -> writtenSection = writtenValidation.value
            is ValidationFailure -> errors.addAll(writtenValidation.errors)
        }
    }

    var metaDataSection: MetaDataSection? = null
    if (metadata.isNotEmpty()) {
        when (val metaDataValidation = validateMetaDataSection(metadata[0], tracker)) {
            is ValidationSuccess -> metaDataSection = metaDataValidation.value
            is ValidationFailure -> errors.addAll(metaDataValidation.errors)
        }
    }

    if (meansSection == null && computesSection == null) {
        errors.add(
            ParseError(
                "A Defines must have either a 'means' or 'computes' section.  " +
                "Either they were both omitted or they contain errors.",
                getRow(group), getColumn(group)
            )
        )
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(
        tracker,
        groupNode,
        DefinesGroup(
            id?.signature(),
            id!!, definesLikeSection!!,
            whenSection,
            meansSection,
            computesSection,
            usingSection,
            writtenSection,
            metaDataSection
        )
    )
}

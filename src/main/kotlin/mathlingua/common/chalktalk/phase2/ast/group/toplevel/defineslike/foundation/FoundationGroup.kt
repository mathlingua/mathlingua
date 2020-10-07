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

package mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.foundation

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
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.defines.MeansSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.defines.validateMeansSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.validateWhenSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.validateWrittenSection
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.common.textalk.Command
import mathlingua.common.transform.signature
import mathlingua.common.support.validationFailure
import mathlingua.common.support.validationSuccess

data class FoundationGroup(
    val signature: String?,
    val id: IdStatement,
    val foundationSection: FoundationSection,
    val whenSection: WhenSection?,
    val meansSection: MeansSection?,
    val writtenSection: WrittenSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(foundationSection)
        if (whenSection != null) {
            fn(whenSection)
        }
        if (meansSection != null) {
            fn(meansSection)
        }
        if (writtenSection != null) {
            fn(writtenSection)
        }
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) = topLevelToCode(
        writer,
        isArg,
        indent,
        id,
        foundationSection,
        whenSection,
        meansSection,
        writtenSection,
        metaDataSection
    )

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(
        FoundationGroup(
        signature = signature,
        id = id.transform(chalkTransformer) as IdStatement,
        foundationSection = foundationSection.transform(chalkTransformer) as FoundationSection,
        whenSection = whenSection?.transform(chalkTransformer) as WhenSection?,
        meansSection = meansSection?.transform(chalkTransformer) as MeansSection?,
        writtenSection = writtenSection?.transform(chalkTransformer) as WrittenSection?,
        metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?
    )
    )
}

fun isFoundationGroup(node: Phase1Node) = firstSectionMatchesName(node, "Foundation")

fun validateFoundationGroup(groupNode: Group, tracker: MutableLocationTracker): Validation<FoundationGroup> {
    val validation = validateFoundationGroupImpl(groupNode, tracker)
    if (validation is ValidationFailure) {
        return validation
    }

    val errors = mutableListOf<ParseError>()
    val foundationally = (validation as ValidationSuccess).value
    val idParen = if (foundationally.id.texTalkRoot is ValidationSuccess) {
        val commandParts = (foundationally.id.texTalkRoot.value.children.find {
            it is Command
        } as Command?)?.parts ?: emptyList()

        val partsWithParens = commandParts.filter { it.paren != null }
        if (partsWithParens.isNotEmpty() && partsWithParens.size != 1) {
            val location = tracker.getLocationOf(foundationally.id)
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
        return validationSuccess(foundationally)
    }

    // it is ok for the id to not have a () but the definition to
    // contain one such as
    //
    //   [\function]
    //   Foundation: f(x)
    //   means: ...

    // if the targets is empty then `validation` above would have been
    // a ValidationFailure and the code would not have reached this point
    val target = foundationally.foundationSection.targets[0]
    val location = tracker.getLocationOf(target)

    if (target !is AbstractionNode) {
        errors.add(
            ParseError(
                message = "If the signature of a Foundation contains a parens then it must define " +
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
                    message = "If the signature of a Foundation contains a parens then " +
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
                        message = "If the signature of a Foundation contains a parens then " +
                            "then the defines must define a function like that has the exact same " +
                            "signature as the parens in the signature of the Foundation.",
                        row = location?.row ?: -1,
                        column = location?.column ?: -1
                    )
                )
            }
        }
    }

    return if (errors.isEmpty()) {
        validationSuccess(foundationally)
    } else {
        validationFailure(errors)
    }
}

private fun validateFoundationGroupImpl(
    groupNode: Group,
    tracker: MutableLocationTracker
): Validation<FoundationGroup> {
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
            "Defines, Represents, or Foundation"
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
            "Foundation", "when?", "means?", "written?", "Metadata?"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return validationFailure(errors)
    }

    val foundation = sectionMap["Foundation"]!!
    val whenNode = sectionMap["when"] ?: emptyList()
    val means = sectionMap["means"]
    val written = sectionMap["written"] ?: emptyList()
    val metadata = sectionMap["Metadata"] ?: emptyList()

    var foundationSection: FoundationSection? = null
    when (val foundationValidation = validateFoundationSection(foundation[0], tracker)) {
        is ValidationSuccess -> foundationSection = foundationValidation.value
        is ValidationFailure -> errors.addAll(foundationValidation.errors)
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

    if (meansSection == null) {
        errors.add(
            ParseError(
                "A Foundation must have a 'means' section.",
                getRow(group), getColumn(group)
            )
        )
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else validationSuccess(
        tracker,
        groupNode,
        FoundationGroup(
            id?.signature(),
            id!!,
            foundationSection!!,
            whenSection,
            meansSection,
            writtenSection,
            metaDataSection
        )
    )
}

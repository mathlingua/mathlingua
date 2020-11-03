/*
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *    http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.evaluates

import mathlingua.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.chalktalk.phase1.ast.Group
import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase1.ast.Phase1Token
import mathlingua.chalktalk.phase1.ast.getColumn
import mathlingua.chalktalk.phase1.ast.getRow
import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.clause.validateIdStatement
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.clause.WhenToPair
import mathlingua.chalktalk.phase2.ast.group.clause.`if`.ElseSection
import mathlingua.chalktalk.phase2.ast.group.clause.`if`.isElseSection
import mathlingua.chalktalk.phase2.ast.group.clause.`if`.validateElseSection
import mathlingua.chalktalk.phase2.ast.group.clause.mapping.ToSection
import mathlingua.chalktalk.phase2.ast.group.clause.mapping.isToSection
import mathlingua.chalktalk.phase2.ast.group.clause.mapping.validateToSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.DefinesStatesOrViews
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.isWrittenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.validateWrittenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.isUsingSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.isWhenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.isMetadataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateMetaDataSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.validateUsingSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.validateWhenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError
import mathlingua.support.Validation
import mathlingua.support.ValidationFailure
import mathlingua.support.ValidationSuccess
import mathlingua.support.validationFailure
import mathlingua.support.validationSuccess
import mathlingua.transform.signature

data class EvaluatesGroup(
    val signature: String?,
    val id: IdStatement,
    val evaluatesSection: EvaluatesSection,
    val whenTo: List<WhenToPair>,
    val elseSection: ElseSection,
    val usingSection: UsingSection?,
    val writtenSection: WrittenSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection), DefinesStatesOrViews {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(evaluatesSection)
        for (wt in whenTo) {
            fn(wt.whenSection)
            fn(wt.toSection)
        }
        fn(elseSection)
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
        val sections = mutableListOf<Phase2Node?>(evaluatesSection)
        for (wt in whenTo) {
            sections.add(wt.whenSection)
            sections.add(wt.toSection)
        }
        sections.add(elseSection)
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
        EvaluatesGroup(
            signature = signature,
            id = id.transform(chalkTransformer) as IdStatement,
            evaluatesSection = evaluatesSection.transform(chalkTransformer) as EvaluatesSection,
            whenTo = whenTo.map {
                WhenToPair(
                    whenSection = chalkTransformer(it.whenSection) as WhenSection,
                    toSection = chalkTransformer(it.toSection) as ToSection
                )
            },
            elseSection = elseSection.transform(chalkTransformer) as ElseSection,
            usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
            writtenSection = writtenSection?.transform(chalkTransformer) as WrittenSection?,
            metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?
        )
    )
}

fun isEvaluatesGroup(node: Phase1Node) = firstSectionMatchesName(node, "Evaluates")

fun validateEvaluatesGroup(rawNode: Phase1Node, tracker: MutableLocationTracker): Validation<EvaluatesGroup> {
    val node = rawNode.resolve()

    val errors = ArrayList<ParseError>()
    if (node !is Group) {
        errors.add(
            ParseError(
                "Expected a Group",
                getRow(node), getColumn(node)
            )
        )
        return validationFailure(errors)
    }

    var id: IdStatement? = null
    if (node.id != null) {
        val (rawText, _, row, column) = node.id
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
        errors.add(
            ParseError(
                "An Evaluates: must have an Id",
                getRow(node), getColumn(node)
            )
        )
    }

    val row = getRow(node)
    val column = getColumn(node)

    val whenToList = mutableListOf<WhenToPair>()
    var elseSection: ElseSection? = null
    var usingSection: UsingSection? = null
    var metaDataSection: MetaDataSection? = null
    var writtenSection: WrittenSection? = null
    if (node.sections.isEmpty()) {
        errors.add(
            ParseError(
            message = "Expected an Evaluates section",
            row = row,
            column = column
        )
        )
    } else {
        var i = 1
        while (i < node.sections.size) {
            val sec = node.sections[i]
            if (!isWhenSection(sec)) {
                break
            }
            i++

            var whenSection: WhenSection? = null
            when (val validation = validateWhenSection(sec, tracker)) {
                is ValidationSuccess -> whenSection = validation.value
                is ValidationFailure -> errors.addAll(validation.errors)
            }

            if (i >= node.sections.size || !isToSection(node.sections[i])) {
                errors.add(
                    ParseError(
                        message = "A when: section must have an to: section",
                        row = getRow(sec),
                        column = getColumn(sec)
                    )
                )
                break
            }

            var toSection: ToSection? = null
            when (val validation = validateToSection(node.sections[i], tracker)) {
                is ValidationSuccess -> {
                    i++
                    toSection = validation.value
                }
                is ValidationFailure -> errors.addAll(validation.errors)
            }

            if (whenSection != null && toSection != null) {
                whenToList.add(
                    WhenToPair(
                        whenSection,
                        toSection
                    )
                )
            }
        }

        if (i < node.sections.size && isElseSection(node.sections[i])) {
            when (val validation = validateElseSection(node.sections[i], tracker)) {
                is ValidationSuccess -> {
                    elseSection = validation.value
                    i++
                }
                is ValidationFailure -> errors.addAll(validation.errors)
            }
        }

        if (i < node.sections.size && isUsingSection(node.sections[i])) {
            when (val validation = validateUsingSection(node.sections[i], tracker)) {
                is ValidationSuccess -> {
                    usingSection = validation.value
                    i++
                }
                is ValidationFailure -> errors.addAll(validation.errors)
            }
        }

        if (i < node.sections.size && isWrittenSection(node.sections[i])) {
            when (val validation = validateWrittenSection(node.sections[i], tracker)) {
                is ValidationSuccess -> {
                    writtenSection = validation.value
                    i++
                }
                is ValidationFailure -> errors.addAll(validation.errors)
            }
        }

        if (i < node.sections.size && isMetadataSection(node.sections[i])) {
            when (val metaDataValidation = validateMetaDataSection(node.sections[i++], tracker)) {
                is ValidationSuccess -> metaDataSection = metaDataValidation.value
                is ValidationFailure -> errors.addAll(metaDataValidation.errors)
            }
        }

        while (i < node.sections.size) {
            val sec = node.sections[i++]
            errors.add(
                ParseError(
                    message = "Unexpected section ${sec.name.text}",
                    row = getRow(sec),
                    column = getColumn(sec)
                )
            )
        }
    }

    if (elseSection == null) {
        errors.add(
            ParseError(
                message = "Expected an else: section",
                row = row,
                column = column
            )
        )
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else {
        validationSuccess(
            EvaluatesGroup(
                signature = id?.signature(),
                id = id!!,
                evaluatesSection = EvaluatesSection(),
                whenTo = whenToList,
                usingSection = usingSection,
                writtenSection = writtenSection,
                metaDataSection = metaDataSection,
                elseSection = elseSection!!
        ))
    }
}

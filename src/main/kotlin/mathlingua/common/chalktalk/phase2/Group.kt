/*
 * Copyright 2019 Google LLC
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

package mathlingua.common.chalktalk.phase2

import mathlingua.common.ParseError
import mathlingua.common.Validation
import mathlingua.common.ValidationFailure
import mathlingua.common.ValidationSuccess
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase1.ast.Phase1Token
import mathlingua.common.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.common.chalktalk.phase1.ast.Group
import mathlingua.common.chalktalk.phase1.ast.Section
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow
import mathlingua.common.transform.getSignature

sealed class TopLevelGroup(open val metaDataSection: MetaDataSection?) : Phase2Node

data class SourceGroup(
    val id: String,
    val sourceSection: SourceSection,
    override val metaDataSection: MetaDataSection?,
    override var row: Int,
    override var column: Int
) : TopLevelGroup(metaDataSection) {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(sourceSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) = toCode(writer, isArg, indent,
        IdStatement(
                id,
                ValidationFailure(emptyList()),
                row,
                column
        ), sourceSection, metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(SourceGroup(
        id = id,
        sourceSection = sourceSection.transform(chalkTransformer) as SourceSection,
        metaDataSection = metaDataSection?.transform(chalkTransformer) as? MetaDataSection,
        row = row,
        column = column
    ))
}

fun isSourceGroup(node: Phase1Node) = firstSectionMatchesName(node, "Source")

fun validateSourceGroup(groupNode: Group): Validation<SourceGroup> {
    val id = groupNode.id
    if (id == null) {
        return ValidationFailure(listOf(
            ParseError("A Source group must have an id",
                getRow(groupNode), getColumn(groupNode))
        ))
    }

    // id.text is of the form [...]
    // The [ and ] need to be removed.
    val idText = id.text.substring(1, id.text.length - 1)

    val errors = mutableListOf<ParseError>()
    if (!Regex("[a-zA-Z0-9]+").matches(idText)) {
        errors.add(
            ParseError("A source id can only contain numbers and letters",
                getRow(groupNode), getColumn(groupNode)
            )
        )
    }

    val sections = groupNode.sections
    if (sections.isEmpty()) {
        errors.add(
            ParseError("Expected a Source section",
                getRow(groupNode), getColumn(groupNode))
        )
    }

    val sourceSection = sections[0]
    val sourceValidation = validateSourceSection(sourceSection)
    if (sourceValidation is ValidationFailure) {
        errors.addAll(sourceValidation.errors)
    }

    var metaDataSection: MetaDataSection? = null
    if (sections.size >= 2) {
        val metadataValidation = validateMetaDataSection(sections[1])
        metaDataSection = when (metadataValidation) {
            is ValidationFailure -> {
                errors.addAll(metadataValidation.errors)
                null
            }
            is ValidationSuccess -> {
                metadataValidation.value
            }
        }
    }

    if (sections.size > 2) {
        errors.add(
                ParseError("A Source group can only have a Source section and optionally a Metadata section",
                        getRow(groupNode), getColumn(groupNode))
        )
    }

    if (errors.isNotEmpty()) {
        return ValidationFailure(errors)
    }

    return ValidationSuccess(
            SourceGroup(
                    id = idText,
                    sourceSection = (sourceValidation as ValidationSuccess).value,
                    metaDataSection = metaDataSection,
                    row = getRow(groupNode),
                    column = getColumn(groupNode)
            )
    )
}

data class DefinesGroup(
    val signature: String?,
    val id: IdStatement,
    val definesSection: DefinesSection,
    val assumingSection: AssumingSection?,
    val meansSections: List<MeansSection>,
    val aliasSection: AliasSection?,
    override val metaDataSection: MetaDataSection?,
    override var row: Int,
    override var column: Int
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(definesSection)
        if (assumingSection != null) {
            fn(assumingSection)
        }
        meansSections.forEach(fn)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        val sections = mutableListOf(definesSection, assumingSection)
        sections.addAll(meansSections)
        sections.add(metaDataSection)
        return toCode(
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
        metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?,
        row = row,
        column = column
    ))
}

fun isDefinesGroup(node: Phase1Node) = firstSectionMatchesName(node, "Defines")

fun validateDefinesGroup(groupNode: Group) = validateDefinesLikeGroup(
    groupNode,
    "Defines",
    ::validateDefinesSection,
    "means",
    ::validateMeansSection,
    ::DefinesGroup
)

data class RepresentsGroup(
    val signature: String?,
    val id: IdStatement,
    val representsSection: RepresentsSection,
    val assumingSection: AssumingSection?,
    val thatSections: List<ThatSection>,
    val aliasSection: AliasSection?,
    override val metaDataSection: MetaDataSection?,
    override var row: Int,
    override var column: Int
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(representsSection)
        if (assumingSection != null) {
            fn(assumingSection)
        }
        thatSections.forEach(fn)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        val sections = mutableListOf(representsSection, assumingSection)
        sections.addAll(thatSections)
        sections.add(metaDataSection)
        return toCode(
                writer,
                isArg,
                indent,
                id,
                *sections.toTypedArray()
        )
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(RepresentsGroup(
        signature = signature,
        id = id.transform(chalkTransformer) as IdStatement,
        representsSection = representsSection.transform(chalkTransformer) as RepresentsSection,
        assumingSection = assumingSection?.transform(chalkTransformer) as AssumingSection?,
        thatSections = thatSections.map { chalkTransformer(it) as ThatSection },
        aliasSection = aliasSection?.transform(chalkTransformer) as AliasSection?,
        metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?,
        row = row,
        column = column
    ))
}

fun isRepresentsGroup(node: Phase1Node) = firstSectionMatchesName(node, "Represents")

fun validateRepresentsGroup(groupNode: Group) = validateDefinesLikeGroup(
    groupNode,
    "Represents",
    ::validateRepresentsSection,
    "that",
    ::validateThatSection,
    ::RepresentsGroup
)

data class ResultGroup(
    val resultSection: ResultSection,
    val aliasSection: AliasSection?,
    override val metaDataSection: MetaDataSection?,
    override var row: Int,
    override var column: Int
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(resultSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
            toCode(writer, isArg, indent, null, resultSection, metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(ResultGroup(
        resultSection = resultSection.transform(chalkTransformer) as ResultSection,
        metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?,
        aliasSection = aliasSection?.transform(chalkTransformer) as AliasSection?,
        row = row,
        column = column
    ))
}

fun isResultGroup(node: Phase1Node) = firstSectionMatchesName(node, "Result")

fun validateResultGroup(groupNode: Group) = validateResultLikeGroup(
    groupNode,
    "Result",
    ::validateResultSection,
    ::ResultGroup
)

data class AxiomGroup(
    val axiomSection: AxiomSection,
    val aliasSection: AliasSection?,
    override val metaDataSection: MetaDataSection?,
    override var row: Int,
    override var column: Int
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(axiomSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
            toCode(writer, isArg, indent, null, axiomSection, metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(AxiomGroup(
            axiomSection = axiomSection.transform(chalkTransformer) as AxiomSection,
            aliasSection = aliasSection?.transform(chalkTransformer) as AliasSection,
            metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection,
            row = row,
            column = column
        ))
}

fun isAxiomGroup(node: Phase1Node) = firstSectionMatchesName(node, "Axiom")

fun validateAxiomGroup(groupNode: Group) = validateResultLikeGroup(
    groupNode,
    "Axiom",
    ::validateAxiomSection,
    ::AxiomGroup
)

data class ConjectureGroup(
    val conjectureSection: ConjectureSection,
    val aliasSection: AliasSection?,
    override val metaDataSection: MetaDataSection?,
    override var row: Int,
    override var column: Int
) : TopLevelGroup(metaDataSection) {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(conjectureSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
            toCode(writer, isArg, indent, null, conjectureSection, metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(ConjectureGroup(
            conjectureSection = conjectureSection.transform(chalkTransformer) as ConjectureSection,
            aliasSection = aliasSection?.transform(chalkTransformer) as AliasSection,
            metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection,
            row = row,
            column = column
        ))
}

fun isConjectureGroup(node: Phase1Node) = firstSectionMatchesName(node, "Conjecture")

fun validateConjectureGroup(groupNode: Group) = validateResultLikeGroup(
    groupNode,
    "Conjecture",
    ::validateConjectureSection,
    ::ConjectureGroup
)

fun toCode(writer: CodeWriter, isArg: Boolean, indent: Int, id: IdStatement?, vararg sections: Phase2Node?): CodeWriter {
    var useAsArg = isArg
    if (id != null) {
        writer.writeIndent(isArg, indent)
        writer.writeId(id)
        writer.writeNewline()
        useAsArg = false
    }

    val nonNullSections = sections.filterNotNull()
    for (i in nonNullSections.indices) {
        val sect = nonNullSections[i]
        writer.append(sect, useAsArg, indent)
        useAsArg = false
        if (i != nonNullSections.size - 1) {
            writer.writeNewline()
        }
    }

    return writer
}

class ProtoGroup(
    val textSection: TextSection,
    override val metaDataSection: MetaDataSection?,
    override var row: Int,
    override var column: Int
) : TopLevelGroup(metaDataSection) {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(textSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
            toCode(writer, isArg, indent, null, textSection, metaDataSection)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
            chalkTransformer(ProtoGroup(
                    textSection = textSection.transform(chalkTransformer) as TextSection,
                    metaDataSection = metaDataSection?.transform(chalkTransformer) as? MetaDataSection,
                    row = row,
                    column = column
            ))
}

fun validateProtoGroup(
    groupNode: Group,
    name: String
): Validation<ProtoGroup> {
    val errors = ArrayList<ParseError>()
    val group = groupNode.resolve()
    if (group.id != null) {
        errors.add(
                ParseError(
                        "A proto group cannot have an Id",
                        getRow(group), getColumn(group)
                )
        )
    }

    val sections = group.sections

    val sectionMap: Map<String, List<Section>>
    try {
        sectionMap = identifySections(
                sections, name, "Metadata?"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return ValidationFailure(errors)
    }

    val textSect = sectionMap[name]
    val metadata = sectionMap["Metadata"] ?: emptyList()

    if (textSect == null || textSect.size != 1) {
        errors.add(
                ParseError(
                        "Expected a single section with name $name",
                        getRow(group), getColumn(group)
                )
        )
    }

    var textSection: TextSection? = null
    when (val validation = validateTextSection(textSect!![0], name)) {
        is ValidationSuccess -> textSection = validation.value
        is ValidationFailure -> errors.addAll(validation.errors)
    }

    var metaDataSection: MetaDataSection? = null
    if (metadata.isNotEmpty()) {
        when (val metaDataValidation = validateMetaDataSection(metadata[0])) {
            is ValidationSuccess -> metaDataSection = metaDataValidation.value
            is ValidationFailure -> errors.addAll(metaDataValidation.errors)
        }
    }

    return if (errors.isNotEmpty()) {
        ValidationFailure(errors)
    } else {
        ValidationSuccess(ProtoGroup(
                textSection = textSection!!,
                metaDataSection = metaDataSection,
                row = getRow(group),
                column = getColumn(group)
        ))
    }
}

fun <G, S> validateResultLikeGroup(
    groupNode: Group,
    resultLikeName: String,
    validateResultLikeSection: (section: Section) -> Validation<S>,
    buildGroup: (
        sect: S,
        alias: AliasSection?,
        metadata: MetaDataSection?,
        row: Int,
        column: Int
    ) -> G
): Validation<G> {
    val errors = ArrayList<ParseError>()
    val group = groupNode.resolve()
    if (group.id != null) {
        errors.add(
            ParseError(
                "A result, axiom, or conjecture cannot have an Id",
                getRow(group), getColumn(group)
            )
        )
    }

    val sections = group.sections

    val sectionMap: Map<String, List<Section>>
    try {
        sectionMap = identifySections(
            sections, resultLikeName, "Alias?", "Metadata?"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return ValidationFailure(errors)
    }

    val resultLike = sectionMap[resultLikeName]!!
    val alias = sectionMap["Alias"] ?: emptyList()
    val metadata = sectionMap["Metadata"] ?: emptyList()

    var resultLikeSection: S? = null
    when (val resultLikeValidation = validateResultLikeSection(resultLike[0])) {
        is ValidationSuccess -> resultLikeSection = resultLikeValidation.value
        is ValidationFailure -> errors.addAll(resultLikeValidation.errors)
    }

    var metaDataSection: MetaDataSection? = null
    if (metadata.isNotEmpty()) {
        when (val metaDataValidation = validateMetaDataSection(metadata[0])) {
            is ValidationSuccess -> metaDataSection = metaDataValidation.value
            is ValidationFailure -> errors.addAll(metaDataValidation.errors)
        }
    }

    var aliasSection: AliasSection? = null
    if (alias.isNotEmpty()) {
        when (val aliasValidation = validateAliasSection(alias[0])) {
            is ValidationSuccess -> aliasSection = aliasValidation.value
            is ValidationFailure -> errors.addAll(aliasValidation.errors)
        }
    }

    return if (errors.isNotEmpty()) {
        ValidationFailure(errors)
    } else ValidationSuccess(buildGroup(
            resultLikeSection!!,
            aliasSection,
            metaDataSection,
            getRow(group), getColumn(group)
    ))
}

fun <G, S, E> validateDefinesLikeGroup(
    groupNode: Group,
    definesLikeSectionName: String,
    validateDefinesLikeSection: (section: Section) -> Validation<S>,
    endSectionName: String,
    validateEndSection: (section: Section) -> Validation<E>,
    buildGroup: (
        signature: String?,
        id: IdStatement,
        definesLike: S,
        assuming: AssumingSection?,
        end: List<E>,
        alias: AliasSection?,
        metadata: MetaDataSection?,
        row: Int,
        column: Int
    ) -> G
): Validation<G> {
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
        when (val idValidation = validateIdStatement(stmtToken)) {
            is ValidationSuccess -> id = idValidation.value
            is ValidationFailure -> errors.addAll(idValidation.errors)
        }
    } else {
        errors.add(
            ParseError(
                "A definition must have an Id",
                getRow(group), getColumn(group)
            )
        )
    }

    val sections = group.sections

    val sectionMap: Map<String, List<Section>>
    try {
        sectionMap = identifySections(
            sections,
            definesLikeSectionName, "assuming?", endSectionName, "Alias?", "Metadata?"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return ValidationFailure(errors)
    }

    val definesLike = sectionMap[definesLikeSectionName]!!
    val assuming = sectionMap["assuming"] ?: emptyList()
    val ends = sectionMap[endSectionName]!!
    val alias = sectionMap["Alias"] ?: emptyList()
    val metadata = sectionMap["Metadata"] ?: emptyList()

    var definesLikeSection: S? = null
    when (val definesLikeValidation = validateDefinesLikeSection(definesLike[0])) {
        is ValidationSuccess -> definesLikeSection = definesLikeValidation.value
        is ValidationFailure -> errors.addAll(definesLikeValidation.errors)
    }

    var assumingSection: AssumingSection? = null
    if (assuming.isNotEmpty()) {
        when (val assumingValidation = validateAssumingSection(assuming[0])) {
            is ValidationSuccess -> assumingSection = assumingValidation.value
            is ValidationFailure -> errors.addAll(assumingValidation.errors)
        }
    }

    val endSections = mutableListOf<E>()
    for (end in ends) {
        when (val endValidation = validateEndSection(end)) {
            is ValidationSuccess -> endSections.add(endValidation.value)
            is ValidationFailure -> errors.addAll(endValidation.errors)
        }
    }

    var aliasSection: AliasSection? = null
    if (alias.isNotEmpty()) {
        when (val aliasValidation = validateAliasSection(alias[0])) {
            is ValidationSuccess -> aliasSection = aliasValidation.value
            is ValidationFailure -> errors.addAll(aliasValidation.errors)
        }
    }

    var metaDataSection: MetaDataSection? = null
    if (metadata.isNotEmpty()) {
        when (val metaDataValidation = validateMetaDataSection(metadata[0])) {
            is ValidationSuccess -> metaDataSection = metaDataValidation.value
            is ValidationFailure -> errors.addAll(metaDataValidation.errors)
        }
    }

    return if (errors.isNotEmpty()) {
        ValidationFailure(errors)
    } else ValidationSuccess(
            buildGroup(
                getSignature(id!!.toStatement()),
                id, definesLikeSection!!,
                assumingSection, endSections,
                aliasSection, metaDataSection,
                getRow(group), getColumn(group)
            )
        )
}

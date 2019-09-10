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

data class SourceGroup(val id: String, val sourceSection: SourceSection) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(sourceSection)
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        return toCode(isArg, indent,
            Statement(id, ValidationFailure(emptyList())), sourceSection)
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node {
        return chalkTransformer(SourceGroup(
            id = id,
            sourceSection = sourceSection.transform(chalkTransformer) as SourceSection
        ))
    }
}

fun isSourceGroup(node: Phase1Node): Boolean {
    return firstSectionMatchesName(node, "Source")
}

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
    if (sections.size != 1) {
        errors.add(
            ParseError("Expected a singe section but found ${sections.size}",
                getRow(groupNode), getColumn(groupNode))
        )
    }

    val section = sections[0]
    val validation = validateSourceSection(section)
    if (validation is ValidationFailure) {
        errors.addAll(validation.errors)
    }

    if (errors.isNotEmpty()) {
        return ValidationFailure(errors)
    }

    return ValidationSuccess(SourceGroup(idText, (validation as ValidationSuccess).value))
}

data class DefinesGroup(
    val signature: String?,
    val id: Statement,
    val definesSection: DefinesSection,
    val assumingSection: AssumingSection?,
    val meansSection: MeansSection,
    val aliasSection: AliasSection?,
    val metaDataSection: MetaDataSection?
) : Phase2Node {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(definesSection)
        if (assumingSection != null) {
            fn(assumingSection)
        }
        fn(meansSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        return toCode(
            isArg,
            indent,
            id,
            definesSection,
            assumingSection,
            meansSection,
            metaDataSection
        )
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node {
        return chalkTransformer(DefinesGroup(
            signature = signature,
            id = id.transform(chalkTransformer) as Statement,
            definesSection = definesSection.transform(chalkTransformer) as DefinesSection,
            assumingSection = assumingSection?.transform(chalkTransformer) as AssumingSection?,
            meansSection = meansSection.transform(chalkTransformer) as MeansSection,
            aliasSection = aliasSection?.transform(chalkTransformer) as AliasSection?,
            metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?
        ))
    }
}

fun isDefinesGroup(node: Phase1Node): Boolean {
    return firstSectionMatchesName(node, "Defines")
}

fun validateDefinesGroup(groupNode: Group): Validation<DefinesGroup> {
    return validateDefinesLikeGroup(
        groupNode,
        "Defines",
        ::validateDefinesSection,
        "means",
        ::validateMeansSection,
        ::DefinesGroup
    )
}

data class RepresentsGroup(
    val signature: String?,
    val id: Statement,
    val representsSection: RepresentsSection,
    val assumingSection: AssumingSection?,
    val thatSection: ThatSection,
    val aliasSection: AliasSection?,
    val metaDataSection: MetaDataSection?
) : Phase2Node {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(representsSection)
        if (assumingSection != null) {
            fn(assumingSection)
        }
        fn(thatSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        return toCode(
            isArg,
            indent,
            id,
            representsSection,
            assumingSection,
            thatSection,
            metaDataSection
        )
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node {
        return chalkTransformer(RepresentsGroup(
            signature = signature,
            id = id.transform(chalkTransformer) as Statement,
            representsSection = representsSection.transform(chalkTransformer) as RepresentsSection,
            assumingSection = assumingSection?.transform(chalkTransformer) as AssumingSection?,
            thatSection = thatSection.transform(chalkTransformer) as ThatSection,
            aliasSection = aliasSection?.transform(chalkTransformer) as AliasSection?,
            metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?
        ))
    }
}

fun isRepresentsGroup(node: Phase1Node): Boolean {
    return firstSectionMatchesName(node, "Represents")
}

fun validateRepresentsGroup(groupNode: Group): Validation<RepresentsGroup> {
    return validateDefinesLikeGroup(
        groupNode,
        "Represents",
        ::validateRepresentsSection,
        "that",
        ::validateThatSection,
        ::RepresentsGroup
    )
}

data class ResultGroup(
    val resultSection: ResultSection,
    val aliasSection: AliasSection?,
    val metaDataSection: MetaDataSection?
) : Phase2Node {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(resultSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        return toCode(isArg, indent, null, resultSection, metaDataSection)
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node {
        return chalkTransformer(ResultGroup(
            resultSection = resultSection.transform(chalkTransformer) as ResultSection,
            metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?,
            aliasSection = aliasSection?.transform(chalkTransformer) as AliasSection?
        ))
    }
}

fun isResultGroup(node: Phase1Node): Boolean {
    return firstSectionMatchesName(node, "Result")
}

fun validateResultGroup(groupNode: Group): Validation<ResultGroup> {
    return validateResultLikeGroup(
        groupNode,
        "Result",
        ::validateResultSection,
        ::ResultGroup
    )
}

data class AxiomGroup(
    val axiomSection: AxiomSection,
    val aliasSection: AliasSection?,
    val metaDataSection: MetaDataSection?
) : Phase2Node {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(axiomSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        return toCode(isArg, indent, null, axiomSection, metaDataSection)
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node {
        return chalkTransformer(AxiomGroup(
            axiomSection = axiomSection.transform(chalkTransformer) as AxiomSection,
            aliasSection = aliasSection?.transform(chalkTransformer) as AliasSection,
            metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection
        ))
    }
}

fun isAxiomGroup(node: Phase1Node): Boolean {
    return firstSectionMatchesName(node, "Axiom")
}

fun validateAxiomGroup(groupNode: Group): Validation<AxiomGroup> {
    return validateResultLikeGroup(
        groupNode,
        "Axiom",
        ::validateAxiomSection,
        ::AxiomGroup
    )
}

data class ConjectureGroup(
    val conjectureSection: ConjectureSection,
    val aliasSection: AliasSection?,
    val metaDataSection: MetaDataSection?
) : Phase2Node {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(conjectureSection)
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        return toCode(isArg, indent, null, conjectureSection, metaDataSection)
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node {
        return chalkTransformer(ConjectureGroup(
            conjectureSection = conjectureSection.transform(chalkTransformer) as ConjectureSection,
            aliasSection = aliasSection?.transform(chalkTransformer) as AliasSection,
            metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection
        ))
    }
}

fun isConjectureGroup(node: Phase1Node): Boolean {
    return firstSectionMatchesName(node, "Conjecture")
}

fun validateConjectureGroup(groupNode: Group): Validation<ConjectureGroup> {
    return validateResultLikeGroup(
        groupNode,
        "Conjecture",
        ::validateConjectureSection,
        ::ConjectureGroup
    )
}

fun toCode(isArg: Boolean, indent: Int, id: Statement?, vararg sections: Phase2Node?): String {
    val builder = StringBuilder()
    var useAsArg = isArg
    if (id != null) {
        builder.append(indentedString(isArg, indent, "[${id.text}]\n"))
        useAsArg = false
    }

    for (i in 0 until sections.size) {
        val sect = sections[i]
        if (sect != null) {
            builder.append(sect.toCode(useAsArg, indent))
            useAsArg = false
            if (i != sections.size - 1) {
                builder.append('\n')
            }
        }
    }

    return builder.toString()
}

fun <G, S> validateResultLikeGroup(
    groupNode: Group,
    resultLikeName: String,
    validateResultLikeSection: (section: Section) -> Validation<S>,
    buildGroup: (sect: S, alias: AliasSection?, metadata: MetaDataSection?) -> G
): Validation<G> {
    val errors = ArrayList<ParseError>()
    val group = groupNode.resolve() as Group
    if (group.id != null) {
        errors.add(
            ParseError(
                "A result, axiom, or conjecture cannot have an Id",
                getRow(group), getColumn(group)
            )
        )
    }

    val sections = group.sections

    val sectionMap: Map<String, Section?>
    try {
        sectionMap = identifySections(
            sections, resultLikeName, "Alias?", "Metadata?"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return ValidationFailure(errors)
    }

    val resultLike = sectionMap[resultLikeName]
    val alias = sectionMap.getOrNull("Alias")
    val metadata = sectionMap.getOrNull("Metadata")

    var resultLikeSection: S? = null
    when (val resultLikeValidation = validateResultLikeSection(resultLike!!)) {
        is ValidationSuccess -> resultLikeSection = resultLikeValidation.value
        is ValidationFailure -> errors.addAll(resultLikeValidation.errors)
    }

    var metaDataSection: MetaDataSection? = null
    if (metadata != null) {
        when (val metaDataValidation = validateMetaDataSection(metadata)) {
            is ValidationSuccess -> metaDataSection = metaDataValidation.value
            is ValidationFailure -> errors.addAll(metaDataValidation.errors)
        }
    }

    var aliasSection: AliasSection? = null
    if (alias != null) {
        when (val aliasValidation = validateAliasSection(alias)) {
            is ValidationSuccess -> aliasSection = aliasValidation.value
            is ValidationFailure -> errors.addAll(aliasValidation.errors)
        }
    }

    return if (!errors.isEmpty()) {
        ValidationFailure(errors)
    } else ValidationSuccess(buildGroup(resultLikeSection!!, aliasSection, metaDataSection))
}

fun <G, S, E> validateDefinesLikeGroup(
    groupNode: Group,
    definesLikeSectionName: String,
    validateDefinesLikeSection: (section: Section) -> Validation<S>,
    endSectionName: String,
    validateEndSection: (section: Section) -> Validation<E>,
    buildGroup: (
        signature: String?,
        id: Statement,
        definesLike: S,
        assuming: AssumingSection?,
        end: E,
        alias: AliasSection?,
        metadata: MetaDataSection?
    ) -> G
): Validation<G> {
    val errors = ArrayList<ParseError>()
    val group = groupNode.resolve() as Group
    var id: Statement? = null
    if (group.id != null) {
        val (rawText, _, row, column) = group.id
        // The id token is of type Id and the text is of the form "[...]"
        // Convert it to look like a statement.
        val statementText = "'" + rawText.substring(1, rawText.length - 1) + "'"
        val stmtToken = Phase1Token(
            statementText, ChalkTalkTokenType.Statement,
            row, column
        )
        when (val idValidation = validateStatement(stmtToken)) {
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

    val sectionMap: Map<String, Section?>
    try {
        sectionMap = identifySections(
            sections,
            definesLikeSectionName, "assuming?", endSectionName, "Alias?", "Metadata?"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return ValidationFailure(errors)
    }

    val definesLike = sectionMap[definesLikeSectionName]
    val assuming = sectionMap.getOrNull("assuming")
    val end = sectionMap[endSectionName]
    val alias = sectionMap.getOrNull("Alias")
    val metadata = sectionMap.getOrNull("Metadata")

    var definesLikeSection: S? = null
    when (val definesLikeValidation = validateDefinesLikeSection(definesLike!!)) {
        is ValidationSuccess -> definesLikeSection = definesLikeValidation.value
        is ValidationFailure -> errors.addAll(definesLikeValidation.errors)
    }

    var assumingSection: AssumingSection? = null
    if (assuming != null) {
        when (val assumingValidation = validateAssumingSection(assuming)) {
            is ValidationSuccess -> assumingSection = assumingValidation.value
            is ValidationFailure -> errors.addAll(assumingValidation.errors)
        }
    }

    var endSection: E? = null
    when (val endValidation = validateEndSection(end!!)) {
        is ValidationSuccess -> endSection = endValidation.value
        is ValidationFailure -> errors.addAll(endValidation.errors)
    }

    var aliasSection: AliasSection? = null
    if (alias != null) {
        when (val aliasValidation = validateAliasSection(alias)) {
            is ValidationSuccess -> aliasSection = aliasValidation.value
            is ValidationFailure -> errors.addAll(aliasValidation.errors)
        }
    }

    var metaDataSection: MetaDataSection? = null
    if (metadata != null) {
        when (val metaDataValidation = validateMetaDataSection(metadata)) {
            is ValidationSuccess -> metaDataSection = metaDataValidation.value
            is ValidationFailure -> errors.addAll(metaDataValidation.errors)
        }
    }

    return if (errors.isNotEmpty()) {
        ValidationFailure(errors)
    } else ValidationSuccess(
            buildGroup(
                getSignature(id!!),
                id, definesLikeSection!!,
                assumingSection, endSection!!,
                aliasSection, metaDataSection
            )
        )
}

private fun <K, V> Map<K, V>.getOrNull(key: K): V? {
    return if (this.containsKey(key)) this.get(key) else null
}

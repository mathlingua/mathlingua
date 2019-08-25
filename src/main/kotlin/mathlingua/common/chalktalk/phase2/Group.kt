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
import mathlingua.common.chalktalk.phase1.ast.*
import mathlingua.common.textalk.*

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

    companion object {

        fun isDefinesGroup(node: ChalkTalkNode): Boolean {
            return firstSectionMatchesName(node, "Defines")
        }

        fun validate(groupNode: Group): Validation<DefinesGroup> {
            return validateDefinesLikeGroup(
                    groupNode,
                    "Defines",
                    DefinesSection.Companion::validate,
                    "means",
                    MeansSection.Companion::validate,
                    ::DefinesGroup
            )
        }
    }
}

data class RefinesGroup(
        val signature: String?,
        val id: Statement,
        val refinesSection: RefinesSection,
        val assumingSection: AssumingSection?,
        val meansSection: MeansSection,
        val aliasSection: AliasSection?,
        val metaDataSection: MetaDataSection?
) : Phase2Node {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(refinesSection)
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
                id,
                refinesSection,
                assumingSection,
                meansSection,
                metaDataSection
        )
    }

    companion object {

        fun isRefinesGroup(node: ChalkTalkNode): Boolean {
            return firstSectionMatchesName(node, "Refines")
        }

        fun validate(groupNode: Group): Validation<RefinesGroup> {
            return validateDefinesLikeGroup(
                    groupNode,
                    "Refines",
                    RefinesSection.Companion::validate,
                    "means",
                    MeansSection.Companion::validate,
                    ::RefinesGroup
            )
        }
    }
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

    companion object {

        fun isRepresentsGroup(node: ChalkTalkNode): Boolean {
            return firstSectionMatchesName(node, "Represents")
        }

        fun validate(groupNode: Group): Validation<RepresentsGroup> {
            return validateDefinesLikeGroup(
                    groupNode,
                    "Represents",
                    RepresentsSection.Companion::validate,
                    "that",
                    ThatSection.Companion::validate,
                    ::RepresentsGroup
            )
        }
    }
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

    companion object {

        fun isResultGroup(node: ChalkTalkNode): Boolean {
            return firstSectionMatchesName(node, "Result")
        }

        fun validate(groupNode: Group): Validation<ResultGroup> {
            return validateResultLikeGroup(
                    groupNode,
                    "Result",
                    ResultSection.Companion::validate,
                    ::ResultGroup
            )
        }
    }
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

    companion object {

        fun isAxiomGroup(node: ChalkTalkNode): Boolean {
            return firstSectionMatchesName(node, "Axiom")
        }

        fun validate(groupNode: Group): Validation<AxiomGroup> {
            return validateResultLikeGroup(
                    groupNode,
                    "Axiom",
                    AxiomSection.Companion::validate,
                    ::AxiomGroup
            )
        }
    }
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

    companion object {

        fun isConjectureGroup(node: ChalkTalkNode): Boolean {
            return firstSectionMatchesName(node, "Conjecture")
        }

        fun validate(groupNode: Group): Validation<ConjectureGroup> {
            return validateResultLikeGroup(
                    groupNode,
                    "Conjecture",
                    ConjectureSection.Companion::validate,
                    ::ConjectureGroup
            )
        }
    }
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
                        AstUtils.getRow(group), AstUtils.getColumn(group)
                )
        )
    }

    val sections = group.sections

    val sectionMap: Map<String, Section?>
    try {
        sectionMap = SectionIdentifier.identifySections(
                sections, resultLikeName, "Alias?", "Metadata?"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return Validation.failure(errors)
    }

    val resultLike = sectionMap[resultLikeName]
    val alias = sectionMap.getOrNull("Alias")
    val metadata = sectionMap.getOrNull("Metadata")

    val resultLikeValidation = validateResultLikeSection(resultLike!!)
    var resultLikeSection: S? = null
    if (resultLikeValidation.isSuccessful) {
        resultLikeSection = resultLikeValidation.value
    } else {
        errors.addAll(resultLikeValidation.errors)
    }

    var metaDataSection: MetaDataSection? = null
    if (metadata != null) {
        val metaDataValidation = MetaDataSection.validate(metadata)
        if (metaDataValidation.isSuccessful) {
            metaDataSection = metaDataValidation.value!!
        } else {
            errors.addAll(metaDataValidation.errors)
        }
    }

    var aliasSection: AliasSection? = null
    if (alias != null) {
        val aliasValidation = AliasSection.validate(alias)
        if (aliasValidation.isSuccessful) {
            aliasSection = aliasValidation.value!!
        } else {
            errors.addAll(aliasValidation.errors)
        }
    }

    return if (!errors.isEmpty()) {
        Validation.failure(errors)
    } else Validation
            .success(buildGroup(resultLikeSection!!, aliasSection, metaDataSection))
}

fun <G, S, E> validateDefinesLikeGroup(
        groupNode: Group,
        definesLikeSectionName: String,
        validateDefinesLikeSection: (section: Section) -> Validation<S>,
        endSectionName: String,
        validateEndSection: (section: Section) -> Validation<E>,
        buildGroup: (
                signature: String?, id: Statement, definesLike: S,
                assuming: AssumingSection?, end: E,
                alias: AliasSection?, metadata: MetaDataSection?
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
        val stmtToken = ChalkTalkToken(
                statementText, ChalkTalkTokenType.Statement,
                row, column
        )
        val idValidation = Statement.validate(stmtToken)
        if (idValidation.isSuccessful) {
            id = idValidation.value
        } else {
            errors.addAll(idValidation.errors)
        }
    } else {
        errors.add(
                ParseError(
                        "A definition must have an Id",
                        AstUtils.getRow(group), AstUtils.getColumn(group)
                )
        )
    }

    val sections = group.sections

    val sectionMap: Map<String, Section?>
    try {
        sectionMap = SectionIdentifier.identifySections(
                sections,
                definesLikeSectionName, "assuming?", endSectionName, "Alias?", "Metadata?"
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return Validation.failure(errors)
    }

    val definesLike = sectionMap[definesLikeSectionName]
    val assuming = sectionMap.getOrNull("assuming")
    val end = sectionMap[endSectionName]
    val alias = sectionMap.getOrNull("Alias")
    val metadata = sectionMap.getOrNull("Metadata")

    val definesLikeValidation = validateDefinesLikeSection(definesLike!!)
    var definesLikeSection: S? = null
    if (definesLikeValidation.isSuccessful) {
        definesLikeSection = definesLikeValidation.value
    } else {
        errors.addAll(definesLikeValidation.errors)
    }

    var assumingSection: AssumingSection? = null
    if (assuming != null) {
        val assumingValidation = AssumingSection.validate(assuming)
        if (assumingValidation.isSuccessful) {
            assumingSection = assumingValidation.value!!
        } else {
            errors.addAll(assumingValidation.errors)
        }
    }

    val endValidation = validateEndSection(end!!)
    var endSection: E? = null
    if (endValidation.isSuccessful) {
        endSection = endValidation.value
    } else {
        errors.addAll(endValidation.errors)
    }

    var aliasSection: AliasSection? = null
    if (alias != null) {
        val aliasValidation = AliasSection.validate(alias)
        if (aliasValidation.isSuccessful) {
            aliasSection = aliasValidation.value!!
        } else {
            errors.addAll(aliasValidation.errors)
        }
    }

    var metaDataSection: MetaDataSection? = null
    if (metadata != null) {
        val metaDataValidation = MetaDataSection.validate(metadata)
        if (metaDataValidation.isSuccessful) {
            metaDataSection = metaDataValidation.value!!
        } else {
            errors.addAll(metaDataValidation.errors)
        }
    }

    return if (!errors.isEmpty()) {
        Validation.failure(errors)
    } else Validation
            .success(
                    buildGroup(
                            getSignature(id!!),
                            id, definesLikeSection!!,
                            assumingSection, endSection!!,
                            aliasSection, metaDataSection
                    )
            )
}

fun <K, V> Map<K, V>.getOrNull(key: K): V? {
    return if (this.containsKey(key)) this.get(key) else null
}

fun getSignature(stmt: Statement): String? {
    val rootValidation = stmt.texTalkRoot
    if (!rootValidation.isSuccessful) {
        return null
    }

    val expressionNode = rootValidation.value!!
    for (child in expressionNode.children) {
        if (child is Command) {
            return getCommandSignature(child).toCode()
        }
    }

    return null
}

fun getCommandSignature(command: Command): Command {
    return Command(
            parts = command.parts.map { getCommandPartForSignature(it) }
    )
}

fun <T> callOrNull(input: T?, fn: (t: T) -> T): T? {
    return if (input == null) null else fn(input)
}

private fun getCommandPartForSignature(node: CommandPart): CommandPart {
    return CommandPart(
            name = node.name,
            square = callOrNull(node.square, ::getGroupNodeForSignature),
            subSup = callOrNull(node.subSup, ::getSubSupForSignature),
            groups = node.groups.map { getGroupNodeForSignature(it) },
            namedGroups = node.namedGroups.map { getNamedGroupNodeForSignature(it) }
    )
}

private fun getSubSupForSignature(node: SubSupNode): SubSupNode {
    return SubSupNode(
            sub = callOrNull(node.sub, ::getGroupNodeForSignature),
            sup = callOrNull(node.sup, ::getGroupNodeForSignature)
    )
}

private fun getGroupNodeForSignature(node: GroupNode): GroupNode {
    return GroupNode(
            type = node.type,
            parameters = getParametersNodeForSignature(node.parameters)
    )
}

private fun getParametersNodeForSignature(node: ParametersNode): ParametersNode {
    return ParametersNode(
            items = node.items.map { ExpressionNode(listOf(TextNode(NodeType.Identifier, "?"))) }
    )
}

private fun getNamedGroupNodeForSignature(node: NamedGroupNode): NamedGroupNode {
    return NamedGroupNode(
            name = node.name,
            group = getGroupNodeForSignature(node.group)
    )
}

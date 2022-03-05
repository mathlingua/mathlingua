/*
 * Copyright 2020 The MathLingua Authors
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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines

import mathlingua.backend.transform.Signature
import mathlingua.backend.transform.signature
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_DEFINES_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_DEFINES_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_WRITTEN_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.AbstractionNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.DefinesStatesOrViews
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.getId
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.HasSignature
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.HasUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.CalledSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.MeansSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.WritingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.ProvidingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.validateViewingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.validateCalledSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.validateExtendingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.validateWritingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.validateWrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.GivenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.validateGivenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateMetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.validateUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.validateWhenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topLevelToCode
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.textalk.Command
import mathlingua.frontend.textalk.TexTalkNodeType
import mathlingua.frontend.textalk.TextTexTalkNode

internal data class DefinesGroup(
    override val signature: Signature?,
    override val id: IdStatement,
    val definesSection: DefinesSection,
    val whereSection: WhereTargetSection?,
    val givenSection: GivenSection?,
    val whenSection: WhenSection?,
    val meansSection: MeansSection?,
    val satisfyingSection: SatisfyingSection?,
    val expressingSection: ExpressingSection?,
    val providingSection: ProvidingSection?,
    override val usingSection: UsingSection?,
    val writingSection: WritingSection?,
    val writtenSection: WrittenSection,
    val calledSection: CalledSection?,
    override val metaDataSection: MetaDataSection?,
    override val row: Int,
    override val column: Int
) : TopLevelGroup(metaDataSection), HasUsingSection, HasSignature, DefinesStatesOrViews {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(definesSection)
        if (whereSection != null) {
            fn(whereSection)
        }
        if (givenSection != null) {
            fn(givenSection)
        }
        if (whenSection != null) {
            fn(whenSection)
        }
        if (meansSection != null) {
            fn(meansSection)
        }
        if (satisfyingSection != null) {
            fn(satisfyingSection)
        }
        if (expressingSection != null) {
            fn(expressingSection)
        }
        if (providingSection != null) {
            fn(providingSection)
        }
        if (usingSection != null) {
            fn(usingSection)
        }
        if (writingSection != null) {
            fn(writingSection)
        }
        fn(writtenSection)
        if (calledSection != null) {
            fn(calledSection)
        }
        if (metaDataSection != null) {
            fn(metaDataSection)
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        val sections =
            mutableListOf(
                definesSection,
                whereSection,
                givenSection,
                whenSection,
                meansSection,
                satisfyingSection,
                expressingSection,
                providingSection,
                usingSection,
                writingSection,
                writtenSection,
                calledSection,
                metaDataSection)
        return topLevelToCode(this, writer, isArg, indent, id, *sections.toTypedArray())
    }

    fun getCalled() =
        calledSection?.forms
            ?: writtenSection.forms.map {
                "$${it.removeSurrounding("\"", "\"").replace("textrm", "textbf")}$"
            }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(
            DefinesGroup(
                signature = signature,
                id = id.transform(chalkTransformer) as IdStatement,
                definesSection = definesSection.transform(chalkTransformer) as DefinesSection,
                whereSection = whereSection?.transform(chalkTransformer) as WhereTargetSection?,
                givenSection = givenSection?.transform(chalkTransformer) as GivenSection?,
                whenSection = whenSection?.transform(chalkTransformer) as WhenSection?,
                meansSection = meansSection?.transform(chalkTransformer) as MeansSection?,
                satisfyingSection =
                    satisfyingSection?.transform(chalkTransformer) as SatisfyingSection?,
                expressingSection =
                    expressingSection?.transform(chalkTransformer) as ExpressingSection?,
                providingSection =
                    providingSection?.transform(chalkTransformer) as ProvidingSection?,
                usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
                writingSection = writingSection?.transform(chalkTransformer) as WritingSection?,
                writtenSection = writtenSection.transform(chalkTransformer) as WrittenSection,
                calledSection = calledSection?.transform(chalkTransformer) as CalledSection?,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?,
                row = row,
                column = column))
}

internal fun isDefinesGroup(node: Phase1Node) = firstSectionMatchesName(node, "Defines")

internal fun validateDefinesGroup(node: Phase1Node, errors: MutableList<ParseError>) =
    validateGroup(node.resolve(), errors, "Defines", DEFAULT_DEFINES_GROUP) { group ->
        identifySections(
            group,
            errors,
            DEFAULT_DEFINES_GROUP,
            listOf(
                "Defines",
                "where?",
                "given?",
                "when?",
                "means?",
                "satisfying?",
                "expressing?",
                "providing?",
                "using?",
                "writing?",
                "written",
                "called?",
                "Metadata?")) { sections ->
            val id = getId(group, errors)
            val def =
                DefinesGroup(
                    signature = id.signature(),
                    id = id,
                    definesSection =
                        ensureNonNull(sections["Defines"], DEFAULT_DEFINES_SECTION) {
                            validateDefinesSection(it, errors)
                        },
                    whereSection =
                        ifNonNull(sections["where"]) { validateWhereTargetSection(it, errors) },
                    givenSection =
                        ifNonNull(sections["given"]) { validateGivenSection(it, errors) },
                    whenSection = ifNonNull(sections["when"]) { validateWhenSection(it, errors) },
                    meansSection =
                        ifNonNull(sections["means"]) { validateExtendingSection(it, errors) },
                    satisfyingSection =
                        ifNonNull(sections["satisfying"]) { validateSatisfiesSection(it, errors) },
                    expressingSection =
                        ifNonNull(sections["expressing"]) { validateExpressesSection(it, errors) },
                    providingSection =
                        ifNonNull(sections["providing"]) { validateViewingSection(it, errors) },
                    usingSection =
                        ifNonNull(sections["using"]) { validateUsingSection(it, errors) },
                    writingSection =
                        ifNonNull(sections["writing"]) { validateWritingSection(it, errors) },
                    writtenSection =
                        ensureNonNull(sections["written"], DEFAULT_WRITTEN_SECTION) {
                            validateWrittenSection(it, errors)
                        },
                    calledSection =
                        ifNonNull(sections["called"]) { validateCalledSection(it, errors) },
                    metaDataSection =
                        ifNonNull(sections["Metadata"]) { validateMetaDataSection(it, errors) },
                    row = node.row,
                    column = node.column)

            val funcArgsError = checkIfFunctionSignatureMatchDefines(def)
            if (funcArgsError != null) {
                errors.add(funcArgsError)
                DEFAULT_DEFINES_GROUP
            } else if (def.satisfyingSection == null &&
                def.expressingSection == null &&
                def.meansSection == null) {
                errors.add(
                    ParseError(
                        message =
                            "If a `Defines:` doesn't have a `satisfying:` or `expressing:` section, " +
                                "then it must have a `means:` section",
                        row = node.row,
                        column = node.column))
                DEFAULT_DEFINES_GROUP
            } else if (def.satisfyingSection != null && def.expressingSection != null) {
                errors.add(
                    ParseError(
                        message =
                            "A `Defines:` cannot have both a `satisfying:` and an `expressing:` section",
                        row = node.row,
                        column = node.column))
                DEFAULT_DEFINES_GROUP
            } else {
                def
            }
        }
    }

internal fun checkIfFunctionSignatureMatchDefines(defines: DefinesGroup): ParseError? {
    val idArgs =
        when (val validation = defines.id.texTalkRoot
        ) {
            is ValidationSuccess -> {
                val exp = validation.value
                // check if the id is of the form `\f.g.h{a}(x, y, z)`
                if (exp.children.size == 1 &&
                    exp.children[0] is Command &&
                    (exp.children[0] as Command).parts.isNotEmpty() &&
                    (exp.children[0] as Command).parts.last().paren != null &&
                    (exp.children[0] as Command).parts.last().paren!!.type ==
                        TexTalkNodeType.ParenGroup) {
                    // extract 'x, y, z'
                    (exp.children[0] as Command).parts.last().paren!!.parameters.items.mapNotNull {
                        if (it.children.size == 1 && it.children[0] is TextTexTalkNode) {
                            (it.children[0] as TextTexTalkNode).text
                        } else {
                            null
                        }
                    }
                } else {
                    null
                }
            }
            is ValidationFailure -> null
        }
            ?: return null

    var hasParenArgs = false
    for (target in defines.definesSection.targets) {
        if (target is AbstractionNode) {
            val abs = target.abstraction
            if (!abs.isVarArgs &&
                !abs.isEnclosed &&
                abs.subParams == null &&
                abs.parts.size == 1 &&
                abs.parts[0].tail == null &&
                abs.parts[0].subParams == null &&
                abs.parts[0].params != null) {
                hasParenArgs = true
                val defArgs = abs.parts[0].params!!.map { it.text }
                if (idArgs != defArgs) {
                    return ParseError(
                        message =
                            "Expected function arguments '${idArgs.joinToString(", ").trim()}' but found '${defArgs.joinToString(", ").trim()}'",
                        row = defines.definesSection.row,
                        column = defines.definesSection.column)
                }
            }
        }
    }

    if (!hasParenArgs) {
        return ParseError(
            message =
                "Expected a definition of a function with arguments '${idArgs.joinToString(", ").trim()}'",
            row = defines.definesSection.row,
            column = defines.definesSection.column)
    }

    return null
}

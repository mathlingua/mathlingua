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
import mathlingua.frontend.chalktalk.phase1.ast.getColumn
import mathlingua.frontend.chalktalk.phase1.ast.getRow
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
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.ExtendingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.validateCalledSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.validateExtendingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.validateWrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.ViewingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.validateViewingSection
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
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.LocationTracker
import mathlingua.frontend.support.MutableLocationTracker
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
    val givenSection: GivenSection?,
    val whenSection: WhenSection?,
    val extendingSection: ExtendingSection?,
    val meansSection: MeansSection?,
    val expressesSection: ExpressesSection?,
    val viewingSection: ViewingSection?,
    override val usingSection: UsingSection?,
    val writtenSection: WrittenSection,
    val calledSection: CalledSection?,
    override val metaDataSection: MetaDataSection?
) : TopLevelGroup(metaDataSection), HasUsingSection, HasSignature, DefinesStatesOrViews {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        fn(id)
        fn(definesSection)
        if (givenSection != null) {
            fn(givenSection)
        }
        if (whenSection != null) {
            fn(whenSection)
        }
        if (extendingSection != null) {
            fn(extendingSection)
        }
        if (meansSection != null) {
            fn(meansSection)
        }
        if (expressesSection != null) {
            fn(expressesSection)
        }
        if (viewingSection != null) {
            fn(viewingSection)
        }
        if (usingSection != null) {
            fn(usingSection)
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
                givenSection,
                whenSection,
                extendingSection,
                meansSection,
                expressesSection,
                viewingSection,
                usingSection,
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
                givenSection = givenSection?.transform(chalkTransformer) as GivenSection?,
                whenSection = whenSection?.transform(chalkTransformer) as WhenSection?,
                extendingSection =
                    extendingSection?.transform(chalkTransformer) as ExtendingSection?,
                meansSection = meansSection?.transform(chalkTransformer) as MeansSection?,
                expressesSection =
                    expressesSection?.transform(chalkTransformer) as ExpressesSection?,
                viewingSection = viewingSection?.transform(chalkTransformer) as ViewingSection?,
                usingSection = usingSection?.transform(chalkTransformer) as UsingSection?,
                writtenSection = writtenSection.transform(chalkTransformer) as WrittenSection,
                calledSection = calledSection?.transform(chalkTransformer) as CalledSection?,
                metaDataSection = metaDataSection?.transform(chalkTransformer) as MetaDataSection?))
}

internal fun isDefinesGroup(node: Phase1Node) = firstSectionMatchesName(node, "Defines")

internal fun validateDefinesGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "Defines", DEFAULT_DEFINES_GROUP) { group ->
            identifySections(
                group,
                errors,
                DEFAULT_DEFINES_GROUP,
                listOf(
                    "Defines",
                    "given?",
                    "when?",
                    "extending?",
                    "means?",
                    "expresses?",
                    "viewing?",
                    "using?",
                    "written",
                    "called?",
                    "Metadata?")) { sections ->
                val id = getId(group, errors, tracker)
                val def =
                    DefinesGroup(
                        signature = id.signature(tracker),
                        id = id,
                        definesSection =
                            ensureNonNull(sections["Defines"], DEFAULT_DEFINES_SECTION) {
                                validateDefinesSection(it, errors, tracker)
                            },
                        givenSection =
                            ifNonNull(sections["given"]) {
                                validateGivenSection(it, errors, tracker)
                            },
                        whenSection =
                            ifNonNull(sections["when"]) {
                                validateWhenSection(it, errors, tracker)
                            },
                        extendingSection =
                            ifNonNull(sections["extending"]) {
                                validateExtendingSection(it, errors, tracker)
                            },
                        meansSection =
                            ifNonNull(sections["means"]) {
                                validateMeansSection(it, errors, tracker)
                            },
                        expressesSection =
                            ifNonNull(sections["expresses"]) {
                                validateExpressesSection(it, errors, tracker)
                            },
                        viewingSection =
                            ifNonNull(sections["viewing"]) {
                                validateViewingSection(it, errors, tracker)
                            },
                        usingSection =
                            ifNonNull(sections["using"]) {
                                validateUsingSection(it, errors, tracker)
                            },
                        writtenSection =
                            ensureNonNull(sections["written"], DEFAULT_WRITTEN_SECTION) {
                                validateWrittenSection(it, errors, tracker)
                            },
                        calledSection =
                            ifNonNull(sections["called"]) {
                                validateCalledSection(it, errors, tracker)
                            },
                        metaDataSection =
                            ifNonNull(sections["Metadata"]) {
                                validateMetaDataSection(it, errors, tracker)
                            })

                val funcArgsError = checkIfFunctionSignatureMatchDefines(def, tracker)
                if (funcArgsError != null) {
                    errors.add(funcArgsError)
                    DEFAULT_DEFINES_GROUP
                } else if (def.meansSection != null && def.expressesSection != null) {
                    errors.add(
                        ParseError(
                            message =
                                "A `Defines:` cannot have both a `means:` and an `expresses:` section",
                            row = getRow(node),
                            column = getColumn(node)))
                    DEFAULT_DEFINES_GROUP
                } else {
                    def
                }
            }
        }
    }

internal fun checkIfFunctionSignatureMatchDefines(
    defines: DefinesGroup, tracker: LocationTracker
): ParseError? {
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

    val location = tracker.getLocationOf(defines.definesSection) ?: Location(row = -1, column = -1)

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
                        row = location.row,
                        column = location.column)
                }
            }
        }
    }

    if (!hasParenArgs) {
        return ParseError(
            message =
                "Expected a definition of a function with arguments '${idArgs.joinToString(", ").trim()}'",
            row = location.row,
            column = location.column)
    }

    return null
}

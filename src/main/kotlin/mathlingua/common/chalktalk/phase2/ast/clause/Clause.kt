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

package mathlingua.common.chalktalk.phase2.ast.clause

import mathlingua.common.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase1.ast.Group
import mathlingua.common.chalktalk.phase1.ast.Phase1Token
import mathlingua.common.chalktalk.phase1.ast.Section
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.group.clause.`for`.isForGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.`for`.validateForGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.`if`.isIfGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.`if`.validateIfGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.collection.isCollectionGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.collection.validateCollectionGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.exists.isExistsGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.exists.validateExistsGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.expands.isExpandsGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.expands.validateExpandsGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.iff.isIffGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.iff.validateIffGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.inductively.isConstantGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.inductively.isConstructorGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.inductively.isInductivelyGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.inductively.validateConstantGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.inductively.validateConstructorGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.inductively.validateInductivelyGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.mapping.isMappingGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.mapping.validateMappingGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.matching.isMatchingGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.matching.validateMatchingGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.not.isNotGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.not.validateNotGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.or.isOrGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.or.validateOrGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.piecewise.isPiecewiseGroup
import mathlingua.common.chalktalk.phase2.ast.group.clause.piecewise.validatePiecewiseGroup
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.defines.isDefinesGroup
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.defines.validateDefinesGroup
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.evaluates.isEvaluatesGroup
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.evaluates.validateEvaluatesGroup
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.states.isStatesGroup
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.states.validateStatesGroup
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.views.isViewsGroup
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.views.validateViewsGroup
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.common.chalktalk.phase2.ast.section.identifySections
import mathlingua.common.support.MutableLocationTracker
import mathlingua.common.support.ParseError
import mathlingua.common.support.Validation
import mathlingua.common.support.ValidationFailure
import mathlingua.common.support.ValidationSuccess
import mathlingua.common.support.validationFailure
import mathlingua.common.support.validationSuccess

private data class ValidationPair<T>(
    val matches: (node: Phase1Node) -> Boolean,
    val validate: (node: Phase1Node, tracker: MutableLocationTracker) -> Validation<T>
)

interface Clause : Phase2Node
interface Target : Clause

private val CLAUSE_VALIDATORS = listOf(
        ValidationPair<Clause>(
                ::isAbstraction,
                ::validateAbstractionNode
        ),
        ValidationPair(
                ::isTuple,
                ::validateTupleNode
        ),
        ValidationPair(
                ::isAssignment,
                ::validateAssignmentNode
        ),
        ValidationPair(
                ::isIdentifier,
                ::validateIdentifier
        ),
        ValidationPair(
                ::isStatement,
                ::validateStatement
        ),
        ValidationPair(
                ::isText,
                ::validateText
        ),
        ValidationPair(
                ::isForGroup,
                ::validateForGroup
        ),
        ValidationPair(
                ::isExistsGroup,
                ::validateExistsGroup
        ),
        ValidationPair(
                ::isNotGroup,
                ::validateNotGroup
        ),
        ValidationPair(
                ::isOrGroup,
                ::validateOrGroup
        ),
        ValidationPair(
                ::isIfGroup,
                ::validateIfGroup
        ),
        ValidationPair(
                ::isIffGroup,
                ::validateIffGroup
        ),
        ValidationPair(
                ::isExpandsGroup,
                ::validateExpandsGroup
        ),
        ValidationPair(
            ::isMappingGroup,
            ::validateMappingGroup
        ),
        ValidationPair(
            ::isCollectionGroup,
            ::validateCollectionGroup
        ),
        ValidationPair(
            ::isMatchingGroup,
            ::validateMatchingGroup
        ),
        ValidationPair(
            ::isConstantGroup,
            ::validateConstantGroup
        ),
        ValidationPair(
            ::isConstructorGroup,
            ::validateConstructorGroup
        ),
        ValidationPair(
            ::isInductivelyGroup,
            ::validateInductivelyGroup
        ),
        ValidationPair(
            ::isPiecewiseGroup,
            ::validatePiecewiseGroup
        ),
        ValidationPair(
            ::isEvaluatesGroup,
            ::validateEvaluatesGroup
        ),
        ValidationPair(
            ::isDefinesGroup
        ) { node, tracker ->
            if (node is Group) {
                validateDefinesGroup(node, tracker)
            } else {
                validationFailure(listOf(
                    ParseError("Expected a group", getRow(node), getColumn(node))
                ))
            }
        },
        ValidationPair(
            ::isStatesGroup
        ) { node, tracker ->
            if (node is Group) {
                validateStatesGroup(node, tracker)
            } else {
                validationFailure(
                    listOf(
                        ParseError("Expected a group", getRow(node), getColumn(node))
                    )
                )
            }
        },
        ValidationPair(
            ::isViewsGroup
        ) { node, tracker ->
            if (node is Group) {
                validateViewsGroup(node, tracker)
            } else {
                validationFailure(
                    listOf(
                        ParseError("Expected a group", getRow(node), getColumn(node))
                    )
                )
            }
        }
)

fun validateClause(rawNode: Phase1Node, tracker: MutableLocationTracker): Validation<Clause> {
    val node = rawNode.resolve()

    for (pair in CLAUSE_VALIDATORS) {
        if (pair.matches(node)) {
            return when (val validation = pair.validate(node, tracker)) {
                is ValidationSuccess -> validationSuccess(tracker, rawNode, validation.value)
                is ValidationFailure -> validationFailure(validation.errors)
            }
        }
    }

    return validationFailure(
        listOf(
            ParseError(
                "Expected a Target",
                getRow(node), getColumn(node)
            )
        )
    )
}

fun firstSectionMatchesName(node: Phase1Node, name: String): Boolean {
    if (node !is Group) {
        return false
    }

    val (sections) = node
    return if (sections.isEmpty()) {
        false
    } else sections[0].name.text == name
}

fun <G : Phase2Node, S : Phase2Node> validateSingleSectionGroup(
    tracker: MutableLocationTracker,
    rawNode: Phase1Node,
    sectionName: String,
    buildGroup: (sect: S) -> G,
    validateSection: (section: Section, tracker: MutableLocationTracker) -> Validation<S>
) = validateOptionalSingleSectionGroup(
    tracker,
    rawNode,
    sectionName,
    { buildGroup(it!!) },
    validateSection
)

fun <G : Phase2Node, S : Phase2Node> validateOptionalSingleSectionGroup(
    tracker: MutableLocationTracker,
    rawNode: Phase1Node,
    sectionName: String,
    buildGroup: (sect: S?) -> G,
    validateSection: (section: Section, tracker: MutableLocationTracker) -> Validation<S>
): Validation<G> = validateGroup(tracker, rawNode,
    listOf(Validator(
        name = sectionName.removeSuffix("?"),
        optional = sectionName.endsWith("?"),
        validate = validateSection
    ))
) {
    val node = it[sectionName.removeSuffix("?")]
    validationSuccess(tracker, rawNode, buildGroup(node as S))
}

fun <G : Phase2Node, S1 : Phase2Node, S2 : Phase2Node> validateDoubleSectionGroup(
    tracker: MutableLocationTracker,
    rawNode: Phase1Node,
    section1Name: String,
    validateSection1: (section: Section, tracker: MutableLocationTracker) -> Validation<S1>,
    section2Name: String,
    validateSection2: (section: Section, tracker: MutableLocationTracker) -> Validation<S2>,
    buildGroup: (sect1: S1, sect2: S2) -> G
) = validateOptionalDoubleSectionGroup(
    tracker,
    rawNode,
    section1Name,
    validateSection1,
    section2Name,
    validateSection2,
    { sect1, sect2 -> buildGroup(sect1!!, sect2!!) }
)

fun <G : Phase2Node, S1 : Phase2Node, S2 : Phase2Node> validateOptionalDoubleSectionGroup(
    tracker: MutableLocationTracker,
    rawNode: Phase1Node,
    section1Name: String,
    validateSection1: (section: Section, tracker: MutableLocationTracker) -> Validation<S1>,
    section2Name: String,
    validateSection2: (section: Section, tracker: MutableLocationTracker) -> Validation<S2>,
    buildGroup: (sect1: S1?, sect2: S2?) -> G
): Validation<G> = validateGroup(tracker, rawNode,
    listOf(
        Validator(
            name = section1Name.removeSuffix("?"),
            optional = section1Name.endsWith("?"),
            validate = validateSection1
        ),
        Validator(
            name = section2Name.removeSuffix("?"),
            optional = section2Name.endsWith("?"),
            validate = validateSection2
        )
    )
) {
    val node1 = it[section1Name.removeSuffix("?")]
    val node2 = it[section2Name.removeSuffix("?")]
    validationSuccess(tracker, rawNode, buildGroup(node1 as S1, node2 as S2))
}

fun <G : Phase2Node, S1 : Phase2Node, S2 : Phase2Node, S3 : Phase2Node> validateMidOptionalTripleSectionGroup(
    tracker: MutableLocationTracker,
    rawNode: Phase1Node,
    section1Name: String,
    validateSection1: (section: Section, tracker: MutableLocationTracker) -> Validation<S1>,
    section2Name: String,
    validateSection2: (section: Section, tracker: MutableLocationTracker) -> Validation<S2>,
    section3Name: String,
    validateSection3: (section: Section, tracker: MutableLocationTracker) -> Validation<S3>,
    buildGroup: (sect1: S1, sect2: S2?, sect3: S3) -> G
) = validateOptionalTripleSectionGroup(
    tracker,
    rawNode,
    section1Name,
    validateSection1,
    section2Name,
    validateSection2,
    section3Name,
    validateSection3,
    { sect1, sect2, sect3 -> buildGroup(sect1!!, sect2, sect3!!) }
)

fun <G : Phase2Node, S1 : Phase2Node, S2 : Phase2Node, S3 : Phase2Node> validateOptionalTripleSectionGroup(
    tracker: MutableLocationTracker,
    rawNode: Phase1Node,
    section1Name: String,
    validateSection1: (section: Section, tracker: MutableLocationTracker) -> Validation<S1>,
    section2Name: String,
    validateSection2: (section: Section, tracker: MutableLocationTracker) -> Validation<S2>,
    section3Name: String,
    validateSection3: (section: Section, tracker: MutableLocationTracker) -> Validation<S3>,
    buildGroup: (sect1: S1?, sect2: S2?, sect3: S3?) -> G
): Validation<G> = validateGroup(tracker, rawNode,
    listOf(
        Validator(
            name = section1Name.removeSuffix("?"),
            optional = section1Name.endsWith("?"),
            validate = validateSection1
        ),
        Validator(
            name = section2Name.removeSuffix("?"),
            optional = section2Name.endsWith("?"),
            validate = validateSection2
        ),
        Validator(
            name = section3Name.removeSuffix("?"),
            optional = section3Name.endsWith("?"),
            validate = validateSection3
        )
    )
) {
    val node1 = it[section1Name.removeSuffix("?")]
    val node2 = it[section2Name.removeSuffix("?")]
    val node3 = it[section3Name.removeSuffix("?")]
    validationSuccess(tracker, rawNode, buildGroup(node1 as S1, node2 as S2?, node3 as S3))
}

fun <Wrapped : Phase2Node, Base> validateWrappedNode(
    tracker: MutableLocationTracker,
    rawNode: Phase1Node,
    expectedType: String,
    checkType: (node: Phase1Node) -> Base?,
    build: (base: Base) -> Wrapped
): Validation<Wrapped> {
    val node = rawNode.resolve()

    val base = checkType(node)
        ?: return validationFailure(
            listOf(
                ParseError(
                    "Cannot convert ${node.toCode()} to a $expectedType",
                    getRow(node), getColumn(node)
                )
            )
        )

    return validationSuccess(tracker, rawNode, build(base))
}

fun toCode(writer: CodeWriter, isArg: Boolean, indent: Int, phase1Node: Phase1Node): CodeWriter {
    writer.writeIndent(isArg, indent)
    writer.writePhase1Node(phase1Node)
    return writer
}

fun toCode(writer: CodeWriter, isArg: Boolean, indent: Int, vararg sections: Phase2Node?): CodeWriter {
    for (i in sections.indices) {
        val sect = sections[i]
        if (sect != null) {
            writer.append(sect, isArg && i == 0, indent)
            if (i != sections.size - 1) {
                writer.writeNewline()
            }
        }
    }
    return writer
}

data class Validator(
    val name: String,
    val optional: Boolean,
    val validate: (section: Section, tracker: MutableLocationTracker) -> Validation<Phase2Node>
)

fun <G : Phase2Node> validateGroup(
    tracker: MutableLocationTracker,
    rawNode: Phase1Node,
    validations: List<Validator>,
    build: (sections: Map<String, Phase2Node?>) -> Validation<G>
): Validation<G> {
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

    val (sections) = node

    val sectionMap: Map<String, Section>
    try {
        sectionMap = identifySections(
            sections, *validations.map {
                if (it.optional) {
                    "${it.name}?"
                } else {
                    it.name
                }
            }.toTypedArray()
        )
    } catch (e: ParseError) {
        errors.add(ParseError(e.message, e.row, e.column))
        return validationFailure(errors)
    }

    return if (errors.isNotEmpty()) {
        validationFailure(errors)
    } else {
        val partMap = mutableMapOf<String, Phase2Node?>()
        for (v in validations) {
            var n: Phase2Node? = null
            val sect = sectionMap[v.name]
            if (sect != null) {
                when (val sectionValidation = v.validate(sect, tracker)) {
                    is ValidationSuccess -> n = sectionValidation.value
                    is ValidationFailure -> errors.addAll(sectionValidation.errors)
                }
            }
            if (n == null && !v.optional) {
                errors.add(
                    ParseError(
                        message = "Missing required section ${v.name}",
                        row = getRow(node),
                        column = getColumn(node)
                    )
                )
            }
            partMap[v.name] = n
        }
        if (errors.isNotEmpty()) {
            validationFailure(errors)
        } else {
            build(partMap)
        }
    }
}

fun <G : Phase2Node> validateIdGroup(
    tracker: MutableLocationTracker,
    rawNode: Phase1Node,
    validations: List<Validator>,
    build: (id: IdStatement, sections: Map<String, Phase2Node?>) -> Validation<G>
): Validation<G> {
    val group = rawNode.resolve()
    val errors = ArrayList<ParseError>()
    var id: IdStatement? = null
    if (group is Group && group.id != null) {
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
        errors.add(
            ParseError(
                "Expected an Id",
                getRow(group), getColumn(group)
            )
        )
    }

    if (errors.isNotEmpty()) {
        return validationFailure(errors)
    }

    return validateGroup(tracker, rawNode, validations) {
        build(id!!, it)
    }
}

fun <G : Phase2Node> validateIdMetadataGroup(
    tracker: MutableLocationTracker,
    rawNode: Phase1Node,
    validations: List<Validator>,
    build: (id: IdStatement, sections: Map<String, Phase2Node?>, metadata: MetaDataSection?) -> Validation<G>
): Validation<G> {
    val metaValidations = mutableListOf<Validator>()
    metaValidations.addAll(validations)
    return validateIdGroup(
        tracker,
        rawNode,
        metaValidations
    ) { id, sections -> build(id, sections, sections["Metadata"] as MetaDataSection?) }
}

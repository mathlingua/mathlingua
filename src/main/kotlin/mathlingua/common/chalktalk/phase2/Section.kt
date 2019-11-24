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
import mathlingua.common.chalktalk.phase1.ast.*

private fun canBeOnOneLine(target: Target) =
        target is Identifier ||
        target is TupleNode ||
        target is AbstractionNode ||
        target is AggregateNode ||
        target is AssignmentNode

private fun appendTargetArgs(writer: CodeWriter, targets: List<Target>, indent: Int) {
    var i = 0
    while (i < targets.size) {
        val lineItems = mutableListOf<Target>()
        while (i < targets.size && canBeOnOneLine(targets[i])) {
            lineItems.add(targets[i++])
        }
        if (lineItems.isEmpty()) {
            writer.writeNewline()
            writer.append(targets[i++], true, indent)
        } else {
            writer.writeSpace()
            for (j in lineItems.indices) {
                writer.append(lineItems[j], false, 0)
                if (j != lineItems.size - 1) {
                    writer.writeComma()
                    writer.writeSpace()
                }
            }
        }
    }
}

data class AssumingSection(
    val clauses: ClauseListNode,
    override var row: Int,
    override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = clauses.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("assuming")
        if (clauses.clauses.isNotEmpty()) {
            writer.writeNewline()
        }
        writer.append(clauses, true, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(AssumingSection(
            clauses = clauses.transform(chalkTransformer) as ClauseListNode,
            row = row,
            column = column
        ))
}

fun validateAssumingSection(node: Phase1Node) = validateClauseList(
    node,
    "assuming",
    false,
    ::AssumingSection
)

data class DefinesSection(
    val targets: List<Target>,
    override var row: Int,
    override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = targets.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("Defines")
        appendTargetArgs(writer, targets, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(DefinesSection(
            targets = targets.map { it.transform(chalkTransformer) as Target },
            row = row,
            column = column
        ))
}

fun validateDefinesSection(node: Phase1Node) = validateTargetList(
    node,
    "Defines",
    ::DefinesSection
)

data class RefinesSection(
    val targets: List<Target>,
    override var row: Int,
    override var column: Int
) :
    Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = targets.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("Refines")
        appendTargetArgs(writer, targets, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(RefinesSection(
            targets = targets.map { it.transform(chalkTransformer) as Target },
            row = row,
            column = column
        ))
}

fun validateRefinesSection(node: Phase1Node) = validateTargetList(
    node,
    "Refines",
    ::RefinesSection
)

data class RepresentsSection(
    override var row: Int,
    override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("Represents")
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(this)
}

fun validateRepresentsSection(node: Phase1Node): Validation<RepresentsSection> {
    val errors = ArrayList<ParseError>()
    if (node !is Section) {
        errors.add(
            ParseError(
                "Expected a RepresentsSection",
                getRow(node), getColumn(node)
            )
        )
    }

    val sect = node as Section
    if (sect.args.isNotEmpty()) {
        errors.add(
            ParseError(
                "A Represents cannot have any arguments",
                getRow(node), getColumn(node)
            )
        )
    }

    if (sect.name.text != "Represents") {
        errors.add(
            ParseError(
                "Expected a section named Represents",
                getRow(node), getColumn(node)
            )
        )
    }

    return if (errors.isNotEmpty()) {
        ValidationFailure(errors)
    } else {
        ValidationSuccess(RepresentsSection(
                row = getRow(node),
                column = getColumn(node)
        ))
    }
}

data class ExistsSection(
    val identifiers: List<Target>,
    override var row: Int,
    override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = identifiers.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("exists")
        appendTargetArgs(writer, identifiers, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(ExistsSection(
            identifiers = identifiers.map { it.transform(chalkTransformer) as Target },
            row = row,
            column = column
        ))
}

fun validateExistsSection(node: Phase1Node) = validateTargetList(
    node,
    "exists",
    ::ExistsSection
)

data class ForSection(
    val targets: List<Target>,
    override var row: Int,
    override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = targets.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("for")
        appendTargetArgs(writer, targets, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(ForSection(
            targets = targets.map { it.transform(chalkTransformer) as Target },
                row = row,
                column = column
        ))
}

fun validateForSection(node: Phase1Node) = validateTargetList(
    node,
    "for",
        ::ForSection
)

data class MeansSection(
    val clauses: ClauseListNode,
    override var row: Int,
    override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = fn(clauses)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("means")
        if (clauses.clauses.isNotEmpty()) {
            writer.writeNewline()
        }
        writer.append(clauses, true, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(MeansSection(
        clauses = clauses.transform(chalkTransformer) as ClauseListNode,
            row = row,
            column = column
    ))
}

fun validateMeansSection(node: Phase1Node) = validateClauseList(
    node,
    "means",
    false,
    ::MeansSection
)

data class ResultSection(
    val clauses: ClauseListNode,
    override var row: Int,
    override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = clauses.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("Result")
        if (clauses.clauses.isNotEmpty()) {
            writer.writeNewline()
        }
        writer.append(clauses, true, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(ResultSection(
            clauses = clauses.transform(chalkTransformer) as ClauseListNode,
            row = row,
            column = column
        ))
}

fun validateResultSection(node: Phase1Node) = validateClauseList(
    node,
    "Result",
    false,
        ::ResultSection
)

data class AxiomSection(
    val clauses: ClauseListNode,
    override var row: Int,
    override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = clauses.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("Axiom")
        if (clauses.clauses.isNotEmpty()) {
            writer.writeNewline()
        }
        writer.append(clauses, true, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(AxiomSection(
            clauses = clauses.transform(chalkTransformer) as ClauseListNode,
            row = row,
            column = column
        ))
}

fun validateAxiomSection(node: Phase1Node) = validateClauseList(
    node,
    "Axiom",
    false,
        ::AxiomSection
)

data class ConjectureSection(
    val clauses: ClauseListNode,
    override var row: Int,
    override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = clauses.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("Conjecture")
        if (clauses.clauses.isNotEmpty()) {
            writer.writeNewline()
        }
        writer.append(clauses, true, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(ConjectureSection(
            clauses = clauses.transform(chalkTransformer) as ClauseListNode,
                row = row,
                column = column
        ))
}

fun validateConjectureSection(node: Phase1Node) = validateClauseList(
    node,
    "Conjecture",
    false,
        ::ConjectureSection
)

data class SuchThatSection(
    val clauses: ClauseListNode,
    override var row: Int,
    override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = clauses.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("suchThat")
        if (clauses.clauses.isNotEmpty()) {
            writer.writeNewline()
        }
        writer.append(clauses, true, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(SuchThatSection(
            clauses = clauses.transform(chalkTransformer) as ClauseListNode,
            row = row,
            column = column
        ))
}

fun validateSuchThatSection(node: Phase1Node) = validateClauseList(
    node,
    "suchThat",
    false,
    ::SuchThatSection
)

data class ThatSection(
    val clauses: ClauseListNode,
    override var row: Int,
    override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = clauses.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("that")
        if (clauses.clauses.isNotEmpty()) {
            writer.writeNewline()
        }
        writer.append(clauses, true, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(ThatSection(
            clauses = clauses.transform(chalkTransformer) as ClauseListNode,
                row = row,
                column = column
        ))
}

fun validateThatSection(node: Phase1Node) = validateClauseList(
    node,
    "that",
    false,
    ::ThatSection
)

data class IfSection(
    val clauses: ClauseListNode,
    override var row: Int,
    override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = clauses.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("if")
        if (clauses.clauses.isNotEmpty()) {
            writer.writeNewline()
        }
        writer.append(clauses, true, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(IfSection(
            clauses = clauses.transform(chalkTransformer) as ClauseListNode,
            row = row,
            column = column
        ))
}

fun validateIfSection(node: Phase1Node) = validateClauseList(
    node,
    "if",
    true,
    ::IfSection
)

data class IffSection(
    val clauses: ClauseListNode,
    override var row: Int,
    override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = clauses.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("iff")
        if (clauses.clauses.isNotEmpty()) {
            writer.writeNewline()
        }
        writer.append(clauses, true, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(IffSection(
        clauses = clauses.transform(chalkTransformer) as ClauseListNode,
        row = row,
        column = column
    ))
}

fun validateIffSection(node: Phase1Node) = validateClauseList(
    node,
    "iff",
    true,
    ::IffSection
)

data class ThenSection(
    val clauses: ClauseListNode,
    override var row: Int,
    override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = clauses.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("then")
        if (clauses.clauses.isNotEmpty()) {
            writer.writeNewline()
        }
        writer.append(clauses, true, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(ThenSection(
        clauses = clauses.transform(chalkTransformer) as ClauseListNode,
        row = row,
        column = column
    ))
}

fun validateThenSection(node: Phase1Node) = validateClauseList(
    node,
    "then",
    false,
    ::ThenSection
)

data class WhereSection(
    val clauses: ClauseListNode,
    override var row: Int,
    override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = clauses.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("where")
        if (clauses.clauses.isNotEmpty()) {
            writer.writeNewline()
        }
        writer.append(clauses, true, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(WhereSection(
            clauses = clauses.transform(chalkTransformer) as ClauseListNode,
            row = row,
            column = column
        ))
}

fun validateWhereSection(node: Phase1Node) = validateClauseList(
    node,
    "where",
    false,
    ::WhereSection
)

data class NotSection(
    val clauses: ClauseListNode,
    override var row: Int,
    override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = clauses.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("not")
        if (clauses.clauses.isNotEmpty()) {
            writer.writeNewline()
        }
        writer.append(clauses, true, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(NotSection(
            clauses = clauses.transform(chalkTransformer) as ClauseListNode,
            row = row,
            column = column
        ))
}

fun validateNotSection(node: Phase1Node) = validateClauseList(
    node,
    "not",
    false,
    ::NotSection
)

data class OrSection(
    val clauses: ClauseListNode,
    override var row: Int,
    override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) = clauses.forEach(fn)

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("or")
        if (clauses.clauses.isNotEmpty()) {
            writer.writeNewline()
        }
        writer.append(clauses, true, indent + 2)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(OrSection(
            clauses = clauses.transform(chalkTransformer) as ClauseListNode,
            row = row,
            column = column
        ))
}

fun validateOrSection(node: Phase1Node) = validateClauseList(
    node,
    "or",
    false,
    ::OrSection
)

data class TextSection(
    val name: String,
    val text: String,
    override var row: Int,
    override var column: Int
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader(name)
        writer.writeNewline()
        writer.writeIndent(true, indent + 2)
        writer.writeDirect(text)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
            chalkTransformer(this)
}

fun validateTextSection(
    rawNode: Phase1Node,
    name: String
): Validation<TextSection> {
    val node = rawNode.resolve()
    val row = getRow(node)
    val column = getColumn(node)

    val errors = ArrayList<ParseError>()
    if (node !is Section) {
        errors.add(
                ParseError(
                        "Expected a Section",
                        getRow(node), getColumn(node)
                )
        )
    }

    val sect = node as Section
    if (sect.name.text != name) {
        errors.add(
                ParseError(
                        "Expected a Section with name " +
                                name + " but found " + sect.name.text,
                        row, column
                )
        )
    }

    if (sect.args.size != 1) {
        errors.add(
                ParseError(
                        "Section '" + sect.name.text + "' requires exactly one text argument.",
                        row, column
                )
        )
        return ValidationFailure(errors)
    }

    val arg = sect.args[0].chalkTalkTarget
    if (arg !is Phase1Token) {
        errors.add(ParseError(
                "Expected a string but found ${arg.toCode()}",
                row, column
        ))
    }

    return if (errors.isNotEmpty()) {
        ValidationFailure(errors)
    } else {
        ValidationSuccess(TextSection(
                name = name,
                text = (arg as Phase1Token).text,
                row = row,
                column = column
        ))
    }
}

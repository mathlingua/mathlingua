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
import mathlingua.common.chalktalk.phase1.ast.Section
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow

private fun canBeOnOneLine(target: Target) =
        target is Identifier ||
        target is TupleNode ||
        target is AbstractionNode ||
        target is AggregateNode ||
        target is AssignmentNode

private fun appendTargetArgs(builder: StringBuilder, targets: List<Target>, indent: Int) {
    var i = 0
    while (i < targets.size) {
        val lineItems = mutableListOf<Target>()
        while (i < targets.size && canBeOnOneLine(targets[i])) {
            lineItems.add(targets[i++])
        }
        if (lineItems.isEmpty()) {
            builder.append('\n')
            builder.append(targets[i++].toCode(true, indent))
        } else {
            builder.append(' ')
            for (j in lineItems.indices) {
                builder.append(lineItems[j].toCode(false, 0))
                if (j != lineItems.size - 1) {
                    builder.append(", ")
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

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "assuming:"))
        if (clauses.clauses.isNotEmpty()) {
            builder.append('\n')
        }
        builder.append(clauses.toCode(true, indent + 2))
        return builder.toString()
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

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "Defines:"))
        appendTargetArgs(builder, targets, indent + 2)
        return builder.toString()
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

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "Refines:"))
        appendTargetArgs(builder, targets, indent + 2)
        return builder.toString()
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

    override fun toCode(isArg: Boolean, indent: Int) = indentedString(isArg, indent, "Represents:")

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

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "exists:"))
        appendTargetArgs(builder, identifiers, indent + 2)
        return builder.toString()
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

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "for:"))
        appendTargetArgs(builder, targets, indent + 2)
        return builder.toString()
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

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "means:"))
        if (clauses.clauses.isNotEmpty()) {
            builder.append('\n')
        }
        builder.append(clauses.toCode(true, indent + 2))
        return builder.toString()
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

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "Result:"))
        if (clauses.clauses.isNotEmpty()) {
            builder.append('\n')
        }
        builder.append(clauses.toCode(true, indent + 2))
        return builder.toString()
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

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "Axiom:"))
        if (clauses.clauses.isNotEmpty()) {
            builder.append('\n')
        }
        builder.append(clauses.toCode(true, indent + 2))
        return builder.toString()
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

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "Conjecture:"))
        if (clauses.clauses.isNotEmpty()) {
            builder.append('\n')
        }
        builder.append(clauses.toCode(true, indent + 2))
        return builder.toString()
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

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "suchThat:"))
        if (clauses.clauses.isNotEmpty()) {
            builder.append('\n')
        }
        builder.append(clauses.toCode(true, indent + 2))
        return builder.toString()
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

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "that:"))
        if (clauses.clauses.isNotEmpty()) {
            builder.append('\n')
        }
        builder.append(clauses.toCode(true, indent + 2))
        return builder.toString()
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

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "if:"))
        if (clauses.clauses.isNotEmpty()) {
            builder.append('\n')
        }
        builder.append(clauses.toCode(true, indent + 2))
        return builder.toString()
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

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "iff:"))
        if (clauses.clauses.isNotEmpty()) {
            builder.append('\n')
        }
        builder.append(clauses.toCode(true, indent + 2))
        return builder.toString()
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

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "then:"))
        if (clauses.clauses.isNotEmpty()) {
            builder.append('\n')
        }
        builder.append(clauses.toCode(true, indent + 2))
        return builder.toString()
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

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "where:"))
        if (clauses.clauses.isNotEmpty()) {
            builder.append('\n')
        }
        builder.append(clauses.toCode(true, indent + 2))
        return builder.toString()
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

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "not:"))
        if (clauses.clauses.isNotEmpty()) {
            builder.append('\n')
        }
        builder.append(clauses.toCode(true, indent + 2))
        return builder.toString()
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

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "or:"))
        if (clauses.clauses.isNotEmpty()) {
            builder.append('\n')
        }
        builder.append(clauses.toCode(true, indent + 2))
        return builder.toString()
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

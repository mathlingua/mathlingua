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

package mathlingua.chalktalk.phase2.ast

import mathlingua.chalktalk.phase1.ast.AstUtils
import mathlingua.chalktalk.phase1.ast.ChalkTalkNode
import mathlingua.chalktalk.phase1.ast.Section
import mathlingua.common.ParseError
import mathlingua.common.Validation

fun appendClauseArgs(builder: StringBuilder, clauses: List<Clause>, indent: Int) {
    for (i in 0 until clauses.size) {
        builder.append(clauses[i].toCode(true, indent))
        if (i != clauses.size - 1) {
            builder.append('\n')
        }
    }
}

fun appendTargetArgs(builder: StringBuilder, targets: List<Target>, indent: Int) {
    for (i in 0 until targets.size) {
        builder.append(targets[i].toCode(true, indent))
        if (i != targets.size - 1) {
            builder.append('\n')
        }
    }
}

data class AssumingSection(val clauses: List<Clause>) : Phase2Node {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        clauses.forEach(fn)
    }

    companion object {
        fun validate(node: ChalkTalkNode): Validation<AssumingSection> {
            return ClauseListValidator.validate(node, "assuming") { AssumingSection(it) }
        }
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "assuming:"))
        builder.append('\n')
        appendClauseArgs(builder, clauses, indent + 2)
        return builder.toString()
    }
}

data class DefinesSection(val targets: List<Target>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        targets.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "Defines:"))
        builder.append('\n')
        appendTargetArgs(builder, targets, indent + 2)
        return builder.toString()
    }

    companion object {
        fun validate(node: ChalkTalkNode): Validation<DefinesSection> {
            return TargetListValidator.validate(node, "Defines") { DefinesSection(it) }
        }
    }
}

data class RefinesSection(val targets: List<Target>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        targets.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "Refines:"))
        builder.append('\n')
        appendTargetArgs(builder, targets, indent + 2)
        return builder.toString()
    }

    companion object {
        fun validate(node: ChalkTalkNode): Validation<RefinesSection> {
            return TargetListValidator.validate(node, "Refines") { RefinesSection(it) }
        }
    }
}

class RepresentsSection : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        return indentedString(isArg, indent, "Represents:")
    }

    companion object {
        fun validate(node: ChalkTalkNode): Validation<RepresentsSection> {
            val errors = ArrayList<ParseError>()
            if (node !is Section) {
                errors.add(
                    ParseError(
                        "Cannot construct a RepresentsSection " +
                            "from a ${node.javaClass.simpleName}", AstUtils.getRow(node), AstUtils.getColumn(node)
                    )
                )
            }

            val sect = node as Section
            if (sect.args.isNotEmpty()) {
                errors.add(
                    ParseError(
                        "A Represents cannot have any arguments",
                        AstUtils.getRow(node), AstUtils.getColumn(node)
                    )
                )
            }

            if (sect.name.text != "Represents") {
                errors.add(
                    ParseError(
                        "Expected a section named Represents",
                        AstUtils.getRow(node), AstUtils.getColumn(node)
                    )
                )
            }

            return if (errors.isNotEmpty()) {
                Validation.failure(errors)
            } else {
                Validation.success(RepresentsSection())
            }
        }
    }
}

data class ExistsSection(val identifiers: List<Target>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        identifiers.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "exists:"))
        builder.append('\n')
        appendTargetArgs(builder, identifiers, indent + 2)
        return builder.toString()
    }

    companion object {
        fun validate(node: ChalkTalkNode): Validation<ExistsSection> {
            return TargetListValidator.validate(node, "exists") { ExistsSection(it) }
        }
    }
}

data class ForSection(val targets: List<Target>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        targets.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "for:"))
        builder.append('\n')
        appendTargetArgs(builder, targets, indent + 2)
        return builder.toString()
    }

    companion object {
        fun validate(node: ChalkTalkNode): Validation<ForSection> {
            return TargetListValidator
                .validate(node, "for") { ForSection(it) }
        }
    }
}

data class MeansSection(val clauses: List<Clause>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        clauses.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "means:"))
        builder.append('\n')
        appendClauseArgs(builder, clauses, indent + 2)
        return builder.toString()
    }

    companion object {
        fun validate(node: ChalkTalkNode): Validation<MeansSection> {
            return ClauseListValidator.validate(node, "means") { MeansSection(it) }
        }
    }
}

data class ResultSection(val clauses: List<Clause>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        clauses.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "Result:"))
        builder.append('\n')
        appendClauseArgs(builder, clauses, indent + 2)
        return builder.toString()
    }

    companion object {
        fun validate(node: ChalkTalkNode): Validation<ResultSection> {
            return ClauseListValidator.validate(node, "Result") { ResultSection(it) }
        }
    }
}

data class AxiomSection(val clauses: List<Clause>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        clauses.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "Axiom:"))
        builder.append('\n')
        appendClauseArgs(builder, clauses, indent + 2)
        return builder.toString()
    }

    companion object {
        fun validate(node: ChalkTalkNode): Validation<AxiomSection> {
            return ClauseListValidator.validate(node, "Axiom") { AxiomSection(it) }
        }
    }
}

data class ConjectureSection(val clauses: List<Clause>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        clauses.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "Conjecture:"))
        builder.append('\n')
        appendClauseArgs(builder, clauses, indent + 2)
        return builder.toString()
    }

    companion object {
        fun validate(node: ChalkTalkNode): Validation<ConjectureSection> {
            return ClauseListValidator.validate(node, "Conjecture") { ConjectureSection(it) }
        }
    }
}

data class SuchThatSection(val clauses: List<Clause>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        clauses.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "suchThat:"))
        builder.append('\n')
        appendClauseArgs(builder, clauses, indent + 2)
        return builder.toString()
    }

    companion object {
        fun validate(node: ChalkTalkNode): Validation<SuchThatSection> {
            return ClauseListValidator.validate(node, "suchThat") { SuchThatSection(it) }
        }
    }
}

data class ThatSection(val clauses: List<Clause>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        clauses.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "that:"))
        builder.append('\n')
        appendClauseArgs(builder, clauses, indent + 2)
        return builder.toString()
    }

    companion object {
        fun validate(node: ChalkTalkNode): Validation<ThatSection> {
            return ClauseListValidator.validate(node, "that") { ThatSection(it) }
        }
    }
}

data class IfSection(val clauses: List<Clause>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        clauses.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "if:"))
        builder.append('\n')
        appendClauseArgs(builder, clauses, indent + 2)
        return builder.toString()
    }

    companion object {
        fun validate(node: ChalkTalkNode): Validation<IfSection> {
            return ClauseListValidator.validate(node, "if") { IfSection(it) }
        }
    }
}

data class IffSection(val clauses: List<Clause>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        clauses.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "iff:"))
        builder.append('\n')
        appendClauseArgs(builder, clauses, indent + 2)
        return builder.toString()
    }

    companion object {
        fun validate(node: ChalkTalkNode): Validation<IffSection> {
            return ClauseListValidator.validate(node, "iff") { IffSection(it) }
        }
    }
}

data class ThenSection(val clauses: List<Clause>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        clauses.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "then:"))
        builder.append('\n')
        appendClauseArgs(builder, clauses, indent + 2)
        return builder.toString()
    }

    companion object {
        fun validate(node: ChalkTalkNode): Validation<ThenSection> {
            return ClauseListValidator.validate(node, "then") { ThenSection(it) }
        }
    }
}

data class WhereSection(val clauses: List<Clause>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        clauses.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "where:"))
        builder.append('\n')
        appendClauseArgs(builder, clauses, indent + 2)
        return builder.toString()
    }

    companion object {
        fun validate(node: ChalkTalkNode): Validation<WhereSection> {
            return ClauseListValidator.validate(node, "where") { WhereSection(it) }
        }
    }
}

data class NotSection(val clauses: List<Clause>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        clauses.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "not:"))
        builder.append('\n')
        appendClauseArgs(builder, clauses, indent + 2)
        return builder.toString()
    }

    companion object {
        fun validate(node: ChalkTalkNode): Validation<NotSection> {
            return ClauseListValidator.validate(node, "not") { NotSection(it) }
        }
    }
}

data class OrSection(val clauses: List<Clause>) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        clauses.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()
        builder.append(indentedString(isArg, indent, "or:"))
        builder.append('\n')
        appendClauseArgs(builder, clauses, indent + 2)
        return builder.toString()
    }

    companion object {
        fun validate(node: ChalkTalkNode): Validation<OrSection> {
            return ClauseListValidator.validate(node, "or") { OrSection(it) }
        }
    }
}

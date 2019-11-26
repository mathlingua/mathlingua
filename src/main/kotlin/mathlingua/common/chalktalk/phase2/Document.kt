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

import mathlingua.common.*
import mathlingua.common.chalktalk.phase1.ast.*
import mathlingua.common.textalk.ExpressionTexTalkNode

interface CodeWriter {
    fun append(node: Phase2Node, hasDot: Boolean, indent: Int)
    fun writeHeader(header: String)
    fun writeNewline(count: Int = 1)
    fun writeSpace()
    fun writeDot()
    fun writeComma()
    fun writeIndent(hasDot: Boolean, indent: Int)
    fun writePhase1Node(phase1Node: Phase1Node)
    fun writeId(id: Statement)
    fun writeStatement(stmtText: String, root: Validation<ExpressionTexTalkNode>)
    fun writeIdentifier(name: String, isVarArgs: Boolean)
    fun writeText(text: String)
    fun writeDirect(text: String)
    fun getCode(): String
}

interface Phase2Node {
    var row: Int
    var column: Int
    fun forEach(fn: (node: Phase2Node) -> Unit)
    fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter = MathLinguaCodeWriter()): CodeWriter
    fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node
}

data class Document(
    val defines: List<DefinesGroup>,
    val represents: List<RepresentsGroup>,
    val results: List<ResultGroup>,
    val axioms: List<AxiomGroup>,
    val conjectures: List<ConjectureGroup>,
    val sources: List<SourceGroup>,
    val protoGroups: List<ProtoGroup>,
    override var row: Int,
    override var column: Int
) : Phase2Node {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        defines.forEach(fn)
        represents.forEach(fn)
        results.forEach(fn)
        axioms.forEach(fn)
        conjectures.forEach(fn)
        sources.forEach(fn)
        protoGroups.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        for (grp in defines) {
            writer.append(grp, false, 0)
            writer.writeNewline(3)
        }

        for (grp in represents) {
            writer.append(grp, false, 0)
            writer.writeNewline(3)
        }

        for (grp in axioms) {
            writer.append(grp, false, 0)
            writer.writeNewline(3)
        }

        for (grp in conjectures) {
            writer.append(grp, false, 0)
            writer.writeNewline(3)
        }

        for (grp in results) {
            writer.append(grp, false, 0)
            writer.writeNewline(3)
        }

        for (grp in protoGroups) {
            writer.append(grp, false, 0)
            writer.writeNewline(3)
        }

        for (src in sources) {
            writer.append(src, false, 0)
            writer.writeNewline(3)
        }

        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node {
        return chalkTransformer(
            Document(
                defines = defines.map { it.transform(chalkTransformer) as DefinesGroup },
                axioms = axioms.map { it.transform(chalkTransformer) as AxiomGroup },
                conjectures = conjectures.map { it.transform(chalkTransformer) as ConjectureGroup },
                represents = represents.map { it.transform(chalkTransformer) as RepresentsGroup },
                results = results.map { it.transform(chalkTransformer) as ResultGroup },
                sources = sources.map { it.transform(chalkTransformer) as SourceGroup },
                protoGroups = protoGroups.map { it.transform(chalkTransformer) as ProtoGroup },
                row = row,
                column = column
            )
        )
    }
}

fun validateDocument(rawNode: Phase1Node): Validation<Document> {
    val node = rawNode.resolve()

    val errors = ArrayList<ParseError>()
    if (node !is Root) {
        errors.add(
            ParseError(
                "Expected a Root",
                getRow(node),
                getColumn(node)
            )
        )
        return ValidationFailure(errors)
    }

    val defines = ArrayList<DefinesGroup>()
    val represents = ArrayList<RepresentsGroup>()
    val results = ArrayList<ResultGroup>()
    val axioms = ArrayList<AxiomGroup>()
    val conjectures = ArrayList<ConjectureGroup>()
    val protoGroups = ArrayList<ProtoGroup>()
    val sources = ArrayList<SourceGroup>()

    val (groups) = node
    for (group in groups) {
        if (isResultGroup(group)) {
            when (val resultValidation = validateResultGroup(group)) {
                is ValidationSuccess -> results.add(resultValidation.value)
                is ValidationFailure -> errors.addAll(resultValidation.errors)
            }
        } else if (isAxiomGroup(group)) {
            when (val axiomValidation = validateAxiomGroup(group)) {
                is ValidationSuccess -> axioms.add(axiomValidation.value)
                is ValidationFailure -> errors.addAll(axiomValidation.errors)
            }
        } else if (isConjectureGroup(group)) {
            when (val conjectureValidation = validateConjectureGroup(group)) {
                is ValidationSuccess -> conjectures.add(conjectureValidation.value)
                is ValidationFailure -> errors.addAll(conjectureValidation.errors)
            }
        } else if (isDefinesGroup(group)) {
            when (val definesValidation = validateDefinesGroup(group)) {
                is ValidationSuccess -> defines.add(definesValidation.value)
                is ValidationFailure -> errors.addAll(definesValidation.errors)
            }
        } else if (isRepresentsGroup(group)) {
            when (val representsValidation = validateRepresentsGroup(group)) {
                is ValidationSuccess -> represents.add(representsValidation.value)
                is ValidationFailure -> errors.addAll(representsValidation.errors)
            }
        } else if (isSourceGroup(group)) {
            when (val sourceValidation = validateSourceGroup(group)) {
                is ValidationSuccess -> sources.add(sourceValidation.value)
                is ValidationFailure -> errors.addAll(sourceValidation.errors)
            }
        } else if (firstSectionMatchesName(group, "ProtoDefines")) {
            when (val validation = validateProtoGroup(group, "ProtoDefines")) {
                is ValidationSuccess -> protoGroups.add(validation.value)
                is ValidationFailure -> errors.addAll(validation.errors)
            }
        } else if (firstSectionMatchesName(group, "ProtoResult")) {
            when (val validation = validateProtoGroup(group, "ProtoResult")) {
                is ValidationSuccess -> protoGroups.add(validation.value)
                is ValidationFailure -> errors.addAll(validation.errors)
            }
        } else if (firstSectionMatchesName(group, "ProtoAxiom")) {
            when (val validation = validateProtoGroup(group, "ProtoAxiom")) {
                is ValidationSuccess -> protoGroups.add(validation.value)
                is ValidationFailure -> errors.addAll(validation.errors)
            }
        } else if (firstSectionMatchesName(group, "ProtoConjecture")) {
            when (val validation = validateProtoGroup(group, "ProtoConjecture")) {
                is ValidationSuccess -> protoGroups.add(validation.value)
                is ValidationFailure -> errors.addAll(validation.errors)
            }
        } else {
            errors.add(
                ParseError(
                    "Expected a top level group but found " + group.toCode(),
                    getRow(group), getColumn(group)
                )
            )
        }
    }

    var row = Int.MAX_VALUE
    var column = Int.MAX_VALUE

    fun updateRowCol(nodes: List<Phase2Node>) {
        for (n in nodes) {
            row = min(row, n.row)
            column = min(column, n.column)
        }
    }

    updateRowCol(defines)
    updateRowCol(represents)
    updateRowCol(results)
    updateRowCol(axioms)
    updateRowCol(conjectures)
    updateRowCol(sources)

    return if (errors.isNotEmpty()) {
        ValidationFailure(errors)
    } else ValidationSuccess(
        Document(
            defines,
            represents,
            results,
            axioms,
            conjectures,
            sources,
            protoGroups,
            row,
            column
        )
    )
}

fun min(x: Int, y: Int) = if (x < y) x else y

class HtmlCodeWriter : CodeWriter {
    private val builder = StringBuilder()

    override fun append(node: Phase2Node, hasDot: Boolean, indent: Int) {
        builder.append(node.toCode(hasDot, indent, HtmlCodeWriter()).getCode())
    }

    override fun writeHeader(header: String) {
        builder.append("<span class='mathlingua-header'>")
        builder.append(header)
        builder.append(':')
        builder.append("</span>")
    }

    override fun writeNewline(count: Int) {
        for (i in 0 until count) {
            builder.append("<br/>")
        }
    }

    override fun writeSpace() {
        builder.append("<span class='mathlingua-whitespace'></span>")
    }

    override fun writeDot() {
        builder.append("<span class='mathlingua-dot'>")
        builder.append('.')
        builder.append("</span>")
    }

    override fun writeComma() {
        builder.append("<span class='mathlingua-comma'>")
        builder.append(',')
        builder.append("</span>")
    }

    override fun writeIndent(hasDot: Boolean, indent: Int) {
        for (i in 0 until indent - 2) {
            writeSpace()
        }
        if (indent - 2 >= 0) {
            if (hasDot) {
                writeDot()
            } else {
                writeSpace()
            }
        }
        if (indent - 1 >= 0) {
            writeSpace()
        }
    }

    override fun writePhase1Node(phase1Node: Phase1Node) {
        builder.append("<span class='mathlingua-argument'>")
        builder.append("\\[${phase1Node.toCode()
                .replace("{", "\\{")
                .replace("}", "\\}")}\\]")
        builder.append("</span>")
    }

    override fun writeId(id: Statement) {
        builder.append("<span class='mathlingua-id'>")
        builder.append('[')
        val stmt = id.toCode(false, 0, MathLinguaCodeWriter()).getCode()
        builder.append(stmt.removeSurrounding("'", "'"))
        builder.append(']')
        builder.append("</span>")
    }

    override fun writeText(text: String) {
        builder.append("<span class='mathlingua-text'>")
        builder.append('"')
        builder.append(text)
        builder.append('"')
        builder.append("</span>")
    }

    override fun writeStatement(stmtText: String, root: Validation<ExpressionTexTalkNode>) {
        builder.append("<span class='mathlingua-statement'>")
        val IS = "is"
        if (stmtText.contains(IS)) {
            val index = stmtText.indexOf(IS)
            builder.append("\\[${stmtText.substring(0, index)}\\]")
            writeSpace()
            writeDirect("is")
            writeSpace()
            writeDirect(stmtText.substring(index + IS.length).trim())
        } else {
            builder.append("\\[$stmtText\\]")
        }
        builder.append("</span>")
    }

    override fun writeIdentifier(name: String, isVarArgs: Boolean) {
        builder.append("<span class='mathlingua-identifier'>\\[")
        builder.append(name)
        if (isVarArgs) {
            builder.append("...")
        }
        builder.append("\\]</span>")
    }

    override fun writeDirect(text: String) {
        builder.append("<span>")
        builder.append(text)
        builder.append("</span>")
    }

    override fun getCode(): String {
        val text = builder.toString()
                .replace(Regex("(\\s*<\\s*br\\s*/\\s*>\\s*)+$"), "")
                .replace(Regex("^(\\s*<\\s*br\\s*/\\s*>\\s*)+$"), "")
        return "<span class='mathlingua'>$text</span>"
    }
}

class MathLinguaCodeWriter : CodeWriter {
    private val builder = StringBuilder()

    override fun append(node: Phase2Node, hasDot: Boolean, indent: Int) {
        builder.append(node.toCode(hasDot, indent, MathLinguaCodeWriter()).getCode())
    }

    override fun writeHeader(header: String) {
        builder.append(header)
        builder.append(':')
    }

    override fun writeNewline(count: Int) {
        for (i in 0 until count) {
            builder.append('\n')
        }
    }

    override fun writeSpace() {
        builder.append(' ')
    }

    override fun writeDot() {
        builder.append('.')
    }

    override fun writeComma() {
        builder.append(',')
    }

    override fun writeIndent(hasDot: Boolean, indent: Int) {
        for (i in 0 until indent - 2) {
            writeSpace()
        }
        if (indent - 2 >= 0) {
            if (hasDot) {
                writeDot()
            } else {
                writeSpace()
            }
        }
        if (indent - 1 >= 0) {
            writeSpace()
        }
    }

    override fun writePhase1Node(phase1Node: Phase1Node) {
        builder.append(phase1Node.toCode())
    }

    override fun writeId(id: Statement) {
        builder.append('[')
        val stmt = id.toCode(false, 0, MathLinguaCodeWriter()).getCode()
        builder.append(stmt.removeSurrounding("'", "'"))
        builder.append(']')
    }

    override fun writeText(text: String) {
        builder.append('"')
        builder.append(text)
        builder.append('"')
    }

    override fun writeStatement(stmtText: String, root: Validation<ExpressionTexTalkNode>) {
        builder.append("'$stmtText'")
    }

    override fun writeIdentifier(name: String, isVarArgs: Boolean) {
        builder.append(name)
        if (isVarArgs) {
            builder.append("...")
        }
    }

    override fun writeDirect(text: String) {
        builder.append(text)
    }

    override fun getCode() = builder.toString()
}

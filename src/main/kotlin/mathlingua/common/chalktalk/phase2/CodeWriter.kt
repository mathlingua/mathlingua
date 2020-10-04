/*
 * Copyright 2020 Google LLC
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

import mathlingua.common.MathLingua
import mathlingua.common.support.Validation
import mathlingua.common.support.ValidationFailure
import mathlingua.common.support.ValidationSuccess
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.common.chalktalk.phase2.ast.toplevel.DefinesGroup
import mathlingua.common.chalktalk.phase2.ast.toplevel.RepresentsGroup
import mathlingua.common.textalk.ExpressionTexTalkNode
import mathlingua.common.textalk.TextTexTalkNode
import mathlingua.common.textalk.newTexTalkLexer
import mathlingua.common.textalk.newTexTalkParser
import mathlingua.common.transform.Expansion
import mathlingua.common.transform.expandAsWritten

interface CodeWriter {
    fun append(node: Phase2Node, hasDot: Boolean, indent: Int)
    fun writeHeader(header: String)
    fun writeNewline(count: Int = 1)
    fun writeSpace()
    fun writeDot()
    fun writeComma()
    fun writeIndent(hasDot: Boolean, indent: Int)
    fun writePhase1Node(phase1Node: Phase1Node)
    fun writeId(id: IdStatement)
    fun writeStatement(stmtText: String, root: Validation<ExpressionTexTalkNode>)
    fun writeIdentifier(name: String, isVarArgs: Boolean)
    fun writeText(text: String)
    fun writeDirect(text: String)
    fun beginTopLevel()
    fun endTopLevel()
    fun newCodeWriter(defines: List<DefinesGroup>, represents: List<RepresentsGroup>): CodeWriter
    fun getCode(): String
}

const val IS = " is "

open class HtmlCodeWriter(
    val defines: List<DefinesGroup>,
    val represents: List<RepresentsGroup>
) : CodeWriter {
    protected val builder = StringBuilder()

    override fun append(node: Phase2Node, hasDot: Boolean, indent: Int) {
        builder.append(node.toCode(hasDot, indent, newCodeWriter(defines, represents)).getCode())
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
        if (shouldExpand()) {
            builder.append("<span class='mathlingua-argument'>")
            val code = prettyPrintIdentifier(phase1Node.toCode())
            builder.append(
                "\\[${code
                    .replace("{", "\\{")
                    .replace("}", "\\}")
                    // replace _\{...\} with _{...}
                    // because that is required to support things
                    // like M_{i, j}
                    .replace(Regex("_\\\\\\{(.*?)\\\\\\}"), "_{$1}")}\\]"
            )
            builder.append("</span>")
        } else {
            builder.append("<span class='mathlingua-argument-no-render'>")
            builder.append(phase1Node.toCode())
            builder.append("</span>")
        }
    }

    override fun writeId(id: IdStatement) {
        builder.append("<span class='mathlingua-id'>")
        builder.append('[')
        val stmt = id.toStatement().toCode(false, 0, MathLinguaCodeWriter(emptyList(), emptyList())).getCode()
        builder.append(stmt.removeSurrounding("'", "'"))
        builder.append(']')
        builder.append("</span>")
    }

    override fun writeText(text: String) {
        if (shouldExpand()) {
            val expansion = expandTextAsWritten(text, false, defines, represents)
            val title = text + if (expansion.errors.isNotEmpty()) {
                "\n\nWarning:\n" + expansion.errors.joinToString("\n\n")
            } else {
                ""
            }.removeSurrounding("\"", "\"")
            builder.append("<span class='mathlingua-text' title=\"$title\">")
            builder.append((expansion.text ?: text).replace("?", ""))
            builder.append("</span>")
        } else {
            builder.append("<span class='mathlingua-text-no-render'>")
            builder.append("\"${text}\"")
            builder.append("</span>")
        }
    }

    override fun writeStatement(stmtText: String, root: Validation<ExpressionTexTalkNode>) {
        if (shouldExpand()) {
            val expansionErrors = mutableListOf<String>()
            if (root is ValidationFailure) {
                expansionErrors.addAll(root.errors.map { it.message })
            }
            val fullExpansion = if (root is ValidationSuccess && (defines.isNotEmpty() || represents.isNotEmpty())) {
                val patternsToWrittenAs = MathLingua.getPatternsToWrittenAs(defines, represents)
                val result = expandAsWritten(root.value.transform {
                    when (it) {
                        is TextTexTalkNode -> it.copy(text = prettyPrintIdentifier(it.text))
                        else -> it
                    }
                }, patternsToWrittenAs)
                expansionErrors.addAll(result.errors)
                result.text
            } else {
                ""
            }

            val title = stmtText.removeSurrounding("'", "'") + if (expansionErrors.isNotEmpty()) {
                "\n\nWarning:\n" + expansionErrors.joinToString("\n\n")
            } else {
                ""
            }.replace("'", "")

            builder.append("<span class='mathlingua-statement' title='$title'>")
            if (stmtText.contains(IS)) {
                val index = stmtText.indexOf(IS)
                builder.append("\\[${stmtText.substring(0, index)}\\]")
                writeSpace()
                writeDirect("<span class='mathlingua-is'>is</span>")
                writeSpace()
                val lhs = stmtText.substring(index + IS.length).trim()
                val lhsParsed = newTexTalkParser().parse(newTexTalkLexer(lhs))
                if (lhsParsed.errors.isEmpty()) {
                    val patternsToWrittenAs = MathLingua.getPatternsToWrittenAs(defines, represents)
                    builder.append("\\[${expandAsWritten(lhsParsed.root.transform {
                        when (it) {
                            is TextTexTalkNode -> it.copy(text = prettyPrintIdentifier(it.text))
                            else -> it
                        }
                    }, patternsToWrittenAs).text ?: lhsParsed.root.toCode()}\\]")
                } else {
                    writeDirect(lhs)
                }
            } else {
                if (root is ValidationSuccess && (defines.isNotEmpty() || represents.isNotEmpty())) {
                    builder.append("\\[$fullExpansion\\]")
                } else {
                    builder.append("\\[$stmtText\\]")
                }
            }
            builder.append("</span>")
        } else {
            builder.append("<span class='mathlingua-statement-no-render'>")
            builder.append("'$stmtText'")
            builder.append("</span>")
        }
    }

    override fun writeIdentifier(name: String, isVarArgs: Boolean) {
        builder.append("<span class='mathlingua-identifier'>\\[")
        builder.append(prettyPrintIdentifier(name))
        if (isVarArgs) {
            builder.append("...")
        }
        builder.append("\\]</span>")
    }

    override fun writeDirect(text: String) {
        builder.append(text)
    }

    override fun beginTopLevel() {
        builder.append("<span class='mathlingua-top-level'>")
    }

    override fun endTopLevel() {
        builder.append("</span>")
        builder.append("<span class='end-mathlingua-top-level'/>")
    }

    override fun newCodeWriter(defines: List<DefinesGroup>, represents: List<RepresentsGroup>) = HtmlCodeWriter(defines, represents)

    override fun getCode(): String {
        val text = builder.toString()
                .replace(Regex("(\\s*<\\s*br\\s*/\\s*>\\s*)+$"), "")
                .replace(Regex("^(\\s*<\\s*br\\s*/\\s*>\\s*)+$"), "")
        return "<span class='mathlingua'>$text</span>"
    }

    private fun shouldExpand() = defines.isNotEmpty() || represents.isNotEmpty()
}

class MathLinguaCodeWriter(
    val defines: List<DefinesGroup>,
    val represents: List<RepresentsGroup>
) : CodeWriter {
    private val builder = StringBuilder()

    override fun append(node: Phase2Node, hasDot: Boolean, indent: Int) {
        builder.append(node.toCode(hasDot, indent, newCodeWriter(defines, represents)).getCode())
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

    override fun writeId(id: IdStatement) {
        builder.append('[')
        val stmt = id.toStatement().toCode(false, 0, newCodeWriter(emptyList(), emptyList())).getCode()
        builder.append(stmt.removeSurrounding("'", "'"))
        builder.append(']')
    }

    override fun writeText(text: String) {
        builder.append('"')
        builder.append(expandTextAsWritten(text, true, defines, represents).text ?: text)
        builder.append('"')
    }

    override fun writeStatement(stmtText: String, root: Validation<ExpressionTexTalkNode>) {
        if (root is ValidationSuccess && (defines.isNotEmpty() || represents.isNotEmpty())) {
            val patternsToWrittenAs = MathLingua.getPatternsToWrittenAs(defines, represents)
            val expansion = expandAsWritten(root.value, patternsToWrittenAs)
            builder.append(if (expansion.text != null) {
                "'${expansion.text}'"
            } else {
                "'$stmtText'"
            })
        } else {
            builder.append("'$stmtText'")
        }
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

    override fun beginTopLevel() {}

    override fun endTopLevel() {}

    override fun newCodeWriter(defines: List<DefinesGroup>, represents: List<RepresentsGroup>) = MathLinguaCodeWriter(defines, represents)

    override fun getCode() = builder.toString()
}

internal fun prettyPrintIdentifier(text: String): String {
    val regex = Regex("([a-zA-Z]+)([0-9]+)")
    val match = regex.find(text)
    return if (match != null) {
        val groups = match.groupValues
        val name = if (isGreekLetter(groups[1])) {
            "\\${groups[1]}"
        } else {
            groups[1]
        }
        val number = groups[2]
        "${name}_$number"
    } else {
        if (isGreekLetter(text)) {
            "\\$text"
        } else {
            text
        }
    }
}

// ------------------------------------------------------------------------------------------------------------------ //

private fun isGreekLetter(letter: String) =
        mutableSetOf(
                "alpha",
                "beta",
                "gamma",
                "delta",
                "epsilon",
                "zeta",
                "eta",
                "theta",
                "iota",
                "kappa",
                "lambda",
                "mu",
                "nu",
                "xi",
                "omicron",
                "pi",
                "rho",
                "sigma",
                "tau",
                "upsilon",
                "phi",
                "chi",
                "psi",
                "omega",
                "Alpha",
                "Beta",
                "Gamma",
                "Delta",
                "Epsilon",
                "Zeta",
                "Eta",
                "Theta",
                "Iota",
                "Kappa",
                "Lambda",
                "Mu",
                "Nu",
                "Xi",
                "Omicron",
                "Pi",
                "Rho",
                "Sigma",
                "Tau",
                "Upsilon",
                "Phi",
                "Chi",
                "Psi",
                "Omega").contains(letter)

const val ESCAPED_SINGLE_QUOTE = "MATHLINGUA_ESCAPED_SINGLE_QUOTE"

private data class TextRange(
    val text: String,
    val isMathlingua: Boolean
)

private fun splitByMathlingua(text: String): List<TextRange> {
    var remaining = text.replace("\\'", ESCAPED_SINGLE_QUOTE)
    val result = mutableListOf<TextRange>()
    while (remaining.isNotEmpty()) {
        val startIndex = remaining.indexOf("'")
        if (startIndex < 0) {
            result.add(
                TextRange(
                    text = remaining.replace(ESCAPED_SINGLE_QUOTE, "\\'"),
                    isMathlingua = false
                )
            )
            break
        }

        result.add(
            TextRange(
                text = remaining.substring(0, startIndex).replace(ESCAPED_SINGLE_QUOTE, "\\'"),
                isMathlingua = false
            )
        )

        // the index right after the starting '
        val newStart = (startIndex + 1).coerceAtMost(remaining.length - 1)
        remaining = remaining.substring(newStart)

        val endIndex = remaining.indexOf("'")
        if (endIndex < 0) {
            result.add(
                TextRange(
                    text = "'$remaining".replace(ESCAPED_SINGLE_QUOTE, "\\'"),
                    isMathlingua = false
                )
            )
            break
        }

        result.add(
            TextRange(
                text = remaining.substring(0, endIndex).replace(ESCAPED_SINGLE_QUOTE, "\\'"),
                isMathlingua = true
            )
        )

        remaining = remaining.substring((endIndex + 1).coerceAtMost(remaining.length - 1))
    }
    return result
}

private fun expandTextAsWritten(text: String, addQuotes: Boolean, defines: List<DefinesGroup>, represents: List<RepresentsGroup>): Expansion {
    if (defines.isEmpty() && represents.isEmpty()) {
        return Expansion(
            text = text,
            errors = emptyList()
        )
    }

    val errors = mutableListOf<String>()
    val patternsToWrittenAs = MathLingua.getPatternsToWrittenAs(defines, represents)
    val parts = splitByMathlingua(text)
    val builder = StringBuilder()
    for (part in parts) {
        if (part.isMathlingua) {
            val parser = newTexTalkParser()
            val lexer = newTexTalkLexer(part.text)
            val result = parser.parse(lexer)
            if (result.errors.isEmpty()) {
                if (addQuotes) {
                    builder.append('\'')
                }
                val expansion = expandAsWritten(result.root, patternsToWrittenAs)
                builder.append(expansion.text)
                errors.addAll(expansion.errors)
                if (addQuotes) {
                    builder.append('\'')
                }
            } else {
                errors.addAll(result.errors.map { it.message })
            }
        } else {
            builder.append(part.text)
        }
    }
    return Expansion(
        text = builder.toString(),
        errors = errors
    )
}

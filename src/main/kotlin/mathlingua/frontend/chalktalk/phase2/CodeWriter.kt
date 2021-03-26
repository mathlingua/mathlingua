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

package mathlingua.frontend.chalktalk.phase2

import kotlin.random.Random
import mathlingua.backend.getPatternsToWrittenAs
import mathlingua.backend.transform.Expansion
import mathlingua.backend.transform.expandAsWritten
import mathlingua.backend.transform.findAllStatementSignatures
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.FoundationGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.mutually.MutuallyGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.frontend.support.Validation
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.support.newLocationTracker
import mathlingua.frontend.textalk.ExpressionTexTalkNode
import mathlingua.frontend.textalk.TextTexTalkNode
import mathlingua.frontend.textalk.newTexTalkLexer
import mathlingua.frontend.textalk.newTexTalkParser

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
    fun writeUrl(url: String, name: String?)
    fun writeDirect(text: String)
    fun writeHorizontalLine()
    fun beginTopLevel(label: String)
    fun endTopLevel(numNewlines: Int)
    fun newCodeWriter(
        defines: List<DefinesGroup>,
        states: List<StatesGroup>,
        foundations: List<FoundationGroup>,
        mutuallyGroups: List<MutuallyGroup>
    ): CodeWriter
    fun getCode(): String
}

const val IS = " is "

const val IN = " in "

open class HtmlCodeWriter(
    val defines: List<DefinesGroup>,
    val states: List<StatesGroup>,
    val foundations: List<FoundationGroup>,
    val mutuallyGroups: List<MutuallyGroup>
) : CodeWriter {
    private var statementIndex = System.nanoTime() + Random.Default.nextLong()
    protected val builder = StringBuilder()

    override fun append(node: Phase2Node, hasDot: Boolean, indent: Int) {
        builder.append(
            node.toCode(hasDot, indent, newCodeWriter(defines, states, foundations, mutuallyGroups))
                .getCode())
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
                    .replace(Regex("_\\\\\\{(.*?)\\\\\\}"), "_{$1}")}\\]")
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
        val stmt =
            id.toStatement()
                .toCode(
                    false,
                    0,
                    MathLinguaCodeWriter(emptyList(), emptyList(), emptyList(), emptyList()))
                .getCode()
        builder.append(stmt.removeSurrounding("'", "'"))
        builder.append(']')
        builder.append("</span>")
    }

    private fun newlinesToHtml(text: String) = text.replace(Regex("[\r\n][\r\n]+"), "<br/></br>")

    override fun writeUrl(url: String, name: String?) {
        val urlNoSpace = url.replace(Regex("[ \\r\\n\\t]+"), "")
        val title =
            if (name != null) {
                name
            } else {
                val rawName =
                    urlNoSpace
                        .replace("https://", "")
                        .replace("http://", "")
                        .replace("ftp://", "")
                        .replace("file://", "")
                if (rawName.length > 30) {
                    val questionIndex = rawName.indexOf("?")
                    val withoutQueryString =
                        if (questionIndex < 0) {
                            rawName
                        } else {
                            rawName.substring(0, questionIndex)
                        }

                    if (withoutQueryString.length > 30) {
                        val parts = withoutQueryString.split("/")
                        if (parts.size == 1) {
                            withoutQueryString
                        } else {
                            "${parts.first()}/.../${parts.last()}"
                        }
                    } else {
                        withoutQueryString
                    }
                } else {
                    rawName
                }
            }
        builder.append(
            "<span class=\"mathlingua-url\"><a target=\"_blank\" href=\"$urlNoSpace\">$title</a></span>")
    }

    override fun writeText(text: String) {
        val textWithBreaks = newlinesToHtml(text)
        if (shouldExpand()) {
            val expansion =
                expandTextAsWritten(
                    textWithBreaks, false, defines, states, foundations, mutuallyGroups)
            val title =
                textWithBreaks +
                    if (expansion.errors.isNotEmpty()) {
                            "\n\nWarning:\n" + expansion.errors.joinToString("\n\n")
                        } else {
                            ""
                        }
                        .removeSurrounding("\"", "\"")
            builder.append("<span class='mathlingua-text' title=\"$title\">")
            builder.append(newlinesToHtml(expansion.text ?: textWithBreaks).replace("?", ""))
            builder.append("</span>")
        } else {
            builder.append("<span class='mathlingua-text-no-render'>")
            builder.append(textWithBreaks)
            builder.append("</span>")
        }
    }

    private fun prettyPrintTexTalk(text: String): String {
        val newText = "{${text}}"
        val lhsParsed = newTexTalkParser().parse(newTexTalkLexer(newText))
        return if (lhsParsed.errors.isEmpty()) {
                val patternsToWrittenAs =
                    getPatternsToWrittenAs(defines, states, foundations, mutuallyGroups)
                expandAsWritten(
                        lhsParsed.root.transform {
                            when (it) {
                                is TextTexTalkNode -> it.copy(text = prettyPrintIdentifier(it.text))
                                else -> it
                            }
                        },
                        patternsToWrittenAs)
                    .text
                    ?: lhsParsed.root.toCode()
            } else {
                newText
            }
            .removeSurrounding("{", "}")
    }

    override fun writeStatement(stmtText: String, root: Validation<ExpressionTexTalkNode>) {
        val dropdownIndex = statementIndex++
        builder.append(
            "<div class='mathlingua-statement-container' onclick=\"mathlinguaToggleDropdown('statement-$dropdownIndex')\">")
        if (shouldExpand()) {
            val expansionErrors = mutableListOf<String>()
            if (root is ValidationFailure) {
                expansionErrors.addAll(root.errors.map { it.message })
            }
            val fullExpansion =
                if (root is ValidationSuccess &&
                    (defines.isNotEmpty() ||
                        states.isNotEmpty() ||
                        foundations.isNotEmpty() ||
                        mutuallyGroups.isNotEmpty())) {
                    val patternsToWrittenAs =
                        getPatternsToWrittenAs(defines, states, foundations, mutuallyGroups)
                    val result =
                        expandAsWritten(
                            root.value.transform {
                                when (it) {
                                    is TextTexTalkNode ->
                                        it.copy(text = prettyPrintIdentifier(it.text))
                                    else -> it
                                }
                            },
                            patternsToWrittenAs)
                    expansionErrors.addAll(result.errors)
                    result.text
                } else {
                    ""
                }

            val title =
                stmtText.removeSurrounding("'", "'") +
                    if (expansionErrors.isNotEmpty()) {
                            "\n\nWarning:\n" + expansionErrors.joinToString("\n\n")
                        } else {
                            ""
                        }
                        .replace("'", "")

            builder.append("<span class='mathlingua-statement' title='$title'>")
            if (stmtText.contains(IS)) {
                val index = stmtText.indexOf(IS)
                val lhs = stmtText.substring(0, index)
                builder.append("\\[${prettyPrintTexTalk(lhs)}\\]")
                writeSpace()
                writeDirect("<span class='mathlingua-is'>is</span>")
                writeSpace()
                val rhs = stmtText.substring(index + IS.length).trim()
                builder.append("\\[${prettyPrintTexTalk(rhs)}\\]")
            } else if (stmtText.contains(IN)) {
                val index = stmtText.indexOf(IN)
                val lhs = stmtText.substring(0, index)
                builder.append("\\[")
                builder.append(prettyPrintTexTalk(lhs))
                writeDirect(" \\in ")
                val rhs = stmtText.substring(index + IN.length).trim()
                builder.append(prettyPrintTexTalk(rhs))
                builder.append("\\]")
            } else {
                if (root is ValidationSuccess &&
                    (defines.isNotEmpty() ||
                        states.isNotEmpty() ||
                        foundations.isNotEmpty() ||
                        mutuallyGroups.isNotEmpty())) {
                    builder.append("\\[$fullExpansion\\]")
                } else {
                    builder.append("\\[$stmtText\\]")
                }
            }
            builder.append("</span>")
        } else {
            builder.append("<span class='mathlingua-statement-no-render'>")
            builder.append(stmtText)
            builder.append("</span>")
        }
        builder.append("</div>")
        if (root is ValidationSuccess) {
            val stmt = Statement(text = stmtText, texTalkRoot = root)
            val signatures = findAllStatementSignatures(stmt, newLocationTracker())
            if (signatures.isNotEmpty()) {
                builder.append(
                    "<div class='mathlingua-dropdown-menu-hidden' id='statement-$dropdownIndex'>")
                for (sig in signatures) {
                    builder.append(
                        "<a class='mathlingua-dropdown-menu-item' onclick=\"mathlinguaViewSignature('${sig.form}', 'statement-$dropdownIndex')\">")
                    builder.append(sig.form)
                    builder.append("</a>")
                }
                builder.append("</div>")
            }
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

    override fun beginTopLevel(label: String) {
        builder.append("<div id='$label'>")
        builder.append("<div class='mathlingua-top-level'>")
    }

    override fun endTopLevel(numNewlines: Int) {
        builder.append("<div class='end-mathlingua-top-level'></div>")
        builder.append("</div>")
        writeNewline(numNewlines)
        builder.append("</div>")
    }

    override fun newCodeWriter(
        defines: List<DefinesGroup>,
        states: List<StatesGroup>,
        foundations: List<FoundationGroup>,
        mutuallyGroups: List<MutuallyGroup>
    ) = HtmlCodeWriter(defines, states, foundations, mutuallyGroups)

    override fun getCode(): String {
        val text =
            builder
                .toString()
                .replace(Regex("(\\s*<\\s*br\\s*/\\s*>\\s*)+$"), "")
                .replace(Regex("^(\\s*<\\s*br\\s*/\\s*>\\s*)+$"), "")
        return "<span class='mathlingua'>$text</span>"
    }

    override fun writeHorizontalLine() {
        builder.append("<hr/>")
    }

    private fun shouldExpand() =
        defines.isNotEmpty() ||
            states.isNotEmpty() ||
            foundations.isNotEmpty() ||
            mutuallyGroups.isNotEmpty()
}

class MathLinguaCodeWriter(
    val defines: List<DefinesGroup>,
    val states: List<StatesGroup>,
    val foundations: List<FoundationGroup>,
    val mutuallyGroups: List<MutuallyGroup>
) : CodeWriter {
    private val builder = StringBuilder()

    override fun append(node: Phase2Node, hasDot: Boolean, indent: Int) {
        builder.append(
            node.toCode(hasDot, indent, newCodeWriter(defines, states, foundations, mutuallyGroups))
                .getCode())
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
        val stmt =
            id.toStatement()
                .toCode(false, 0, newCodeWriter(emptyList(), emptyList(), emptyList(), emptyList()))
                .getCode()
        builder.append(stmt.removeSurrounding("'", "'"))
        builder.append(']')
    }

    override fun writeText(text: String) {
        builder.append('"')
        builder.append(
            expandTextAsWritten(text, true, defines, states, foundations, mutuallyGroups).text
                ?: text)
        builder.append('"')
    }

    override fun writeUrl(url: String, name: String?) {
        builder.append('"')
        builder.append(url)
        builder.append('"')
    }

    override fun writeStatement(stmtText: String, root: Validation<ExpressionTexTalkNode>) {
        if (root is ValidationSuccess &&
            (defines.isNotEmpty() ||
                states.isNotEmpty() ||
                foundations.isNotEmpty() ||
                mutuallyGroups.isNotEmpty())) {
            val patternsToWrittenAs =
                getPatternsToWrittenAs(defines, states, foundations, mutuallyGroups)
            val expansion = expandAsWritten(root.value, patternsToWrittenAs)
            builder.append(
                if (expansion.text != null) {
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

    override fun beginTopLevel(label: String) {}

    override fun endTopLevel(numNewlines: Int) {
        writeNewline(numNewlines)
    }

    override fun writeHorizontalLine() {}

    override fun newCodeWriter(
        defines: List<DefinesGroup>,
        states: List<StatesGroup>,
        foundations: List<FoundationGroup>,
        mutuallyGroups: List<MutuallyGroup>
    ) = MathLinguaCodeWriter(defines, states, foundations, mutuallyGroups)

    override fun getCode() = builder.toString()
}

internal fun prettyPrintIdentifier(text: String): String {
    val regex = Regex("([a-zA-Z]+)([0-9]+)")
    val match = regex.find(text)
    return if (match != null) {
        val groups = match.groupValues
        val name =
            if (isGreekLetter(groups[1])) {
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
            "Omega")
        .contains(letter)

const val ESCAPED_SINGLE_QUOTE = "MATHLINGUA_ESCAPED_SINGLE_QUOTE"

private data class TextRange(val text: String, val isMathlingua: Boolean)

private fun splitByMathlingua(text: String): List<TextRange> {
    var remaining = text.replace("\\'", ESCAPED_SINGLE_QUOTE)
    val result = mutableListOf<TextRange>()
    while (remaining.isNotEmpty()) {
        val startIndex = remaining.indexOf("'")
        if (startIndex < 0) {
            result.add(
                TextRange(
                    text = remaining.replace(ESCAPED_SINGLE_QUOTE, "\\'"), isMathlingua = false))
            break
        }

        result.add(
            TextRange(
                text = remaining.substring(0, startIndex).replace(ESCAPED_SINGLE_QUOTE, "\\'"),
                isMathlingua = false))

        // the index right after the starting '
        val newStart = (startIndex + 1).coerceAtMost(remaining.length - 1)
        remaining = remaining.substring(newStart)

        val endIndex = remaining.indexOf("'")
        if (endIndex < 0) {
            result.add(
                TextRange(
                    text = "'$remaining".replace(ESCAPED_SINGLE_QUOTE, "\\'"),
                    isMathlingua = false))
            break
        }

        result.add(
            TextRange(
                text = remaining.substring(0, endIndex).replace(ESCAPED_SINGLE_QUOTE, "\\'"),
                isMathlingua = true))

        remaining = remaining.substring((endIndex + 1).coerceAtMost(remaining.length - 1))
    }
    return result
}

private fun expandTextAsWritten(
    text: String,
    addQuotes: Boolean,
    defines: List<DefinesGroup>,
    states: List<StatesGroup>,
    foundations: List<FoundationGroup>,
    mutuallyGroups: List<MutuallyGroup>
): Expansion {
    if (defines.isEmpty() && states.isEmpty()) {
        return Expansion(text = text, errors = emptyList())
    }

    val errors = mutableListOf<String>()
    val patternsToWrittenAs = getPatternsToWrittenAs(defines, states, foundations, mutuallyGroups)
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
    return Expansion(text = builder.toString(), errors = errors)
}

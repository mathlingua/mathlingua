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
import mathlingua.frontend.FrontEnd
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.CalledSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.note.NoteGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.ConjectureGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.ConjectureSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.RelatedGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.ResourcesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.SiteGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.SourceItemGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.StringItem
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.StringSectionGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.TopicsGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topic.TopicGroup
import mathlingua.frontend.support.Validation
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.support.newLocationTracker
import mathlingua.frontend.textalk.ExpressionTexTalkNode
import mathlingua.frontend.textalk.TextTexTalkNode
import mathlingua.frontend.textalk.newTexTalkLexer
import mathlingua.frontend.textalk.newTexTalkParser
import mathlingua.getRandomUuid
import org.intellij.markdown.flavours.commonmark.CommonMarkFlavourDescriptor
import org.intellij.markdown.html.HtmlGenerator
import org.intellij.markdown.parser.MarkdownParser

interface CodeWriter {
    fun generateCode(node: Phase2Node): String
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
    fun writeDirectStatement(stmtText: String, root: Validation<ExpressionTexTalkNode>)
    fun writeIdentifier(name: String, isVarArgs: Boolean)
    fun writeText(text: String)
    fun writeUrl(url: String, name: String?)
    fun writeDirect(text: String)
    fun writeBlockComment(text: String)
    fun writeHorizontalLine()
    fun beginTopLevel(label: String)
    fun endTopLevel(numNewlines: Int)
    fun newCodeWriter(
        defines: List<DefinesGroup>, states: List<StatesGroup>, literal: Boolean
    ): CodeWriter
    fun getCode(): String
}

const val IS = " is "

const val IN = " in "

data class BlockCommentSection(val text: String, val isMathlinguaCode: Boolean)

open class HtmlCodeWriter(
    val defines: List<DefinesGroup>,
    val states: List<StatesGroup>,
    val axioms: List<AxiomGroup>,
    val literal: Boolean
) : CodeWriter {
    private var statementIndex = System.nanoTime() + Random.Default.nextLong()
    protected val builder = StringBuilder()

    private fun generateMetaDataSectionCode(meta: MetaDataSection): String {
        val builder = StringBuilder()

        builder.append("<hr/>")
        builder.append("<span class='mathlingua-metadata'>")

        val overview =
            meta.items.firstOrNull {
                it is StringSectionGroup && it.section.name == "overview"
            } as StringSectionGroup?

        if (overview != null && overview.section.values.isNotEmpty()) {
            for (text in overview.section.values) {
                builder.append("<span class='mathlingua-overview'>")
                builder.append(getExpandedMarkdownedText(text))
                builder.append("</span>")
            }
        }

        // TODO: Support tags, authors, and contributors
        // val tags = meta.items.filter { it is StringSectionGroup && it.section.name ==
        // "tag" } as List<StringSectionGroup>
        // val authors = meta.items.filter { it is StringSectionGroup && it.section.name
        // == "author" } as List<StringSectionGroup>
        // val contributors = meta.items.filter { it is StringSectionGroup&&
        // it.section.name == "contributor" } as List<StringSectionGroup>

        val resourceGroups = meta.items.filterIsInstance<ResourcesGroup>()
        if (resourceGroups.isNotEmpty()) {
            builder.append("<span class='mathlingua-resources-header'>Resources</span>")
            for (grp in resourceGroups) {
                for (res in grp.resourcesSection.items) {
                    when (res) {
                        is SiteGroup -> {
                            val urlNoSpace = getUrlWithoutSpaces(res.siteItemSection.url)
                            val title =
                                getUrlTitle(res.siteItemSection.url, res.nameItemSection?.name)
                            builder.append(
                                "<span class=\"mathlingua-url\"><a class=\"mathlingua-link\" target=\"_blank\" href=\"${urlNoSpace.removeSurrounding("\"", "\"")}\">$title</a></span>")
                        }
                        is SourceItemGroup -> {
                            builder.append("<span class='mathlingua-resources-item'>")

                            val textBuilder = StringBuilder()
                            textBuilder.append(
                                res.sourceSection.sourceReference.removeSurrounding("\"", "\""))
                            val page = res.pageSection?.page
                            val offset = res.offsetSection?.offset
                            if (page != null || offset != null) {
                                textBuilder.append(" (")
                                if (page != null) {
                                    textBuilder.append("Page ")
                                    textBuilder.append(page.removeSurrounding("\"", "\""))
                                }
                                if (offset != null) {
                                    if (page != null) {
                                        textBuilder.append(", ")
                                    }
                                    textBuilder.append("Offset ")
                                    textBuilder.append(offset.removeSurrounding("\"", "\""))
                                }
                                textBuilder.append(")")
                            }

                            builder.append(textBuilder.toString())
                            builder.append("</span>")
                        }
                        is StringItem -> {
                            builder.append("<span class='mathlingua-resources-item'>")
                            builder.append(res.text.removeSurrounding("\"", "\""))
                            builder.append("</span>")
                        }
                        else -> {
                            System.err.println(
                                "ERROR: Unrecognized resource type: ${res.javaClass.simpleName}")
                        }
                    }
                }
            }
        }

        val topicsGroups = meta.items.filterIsInstance<TopicsGroup>()
        if (topicsGroups.isNotEmpty()) {
            for (topicGrp in topicsGroups) {
                builder.append("<span class='mathlingua-topics-header'>Topics</span>")
                for (item in topicGrp.topicsSection.items) {
                    builder.append("<span class='mathlingua-topic-item'>${item.text}</span>")
                }
                builder.append("</span>")
            }
        }

        val relatedGroups = meta.items.filterIsInstance<RelatedGroup>()
        if (relatedGroups.isNotEmpty()) {
            for (relatedGrp in relatedGroups) {
                builder.append("<span class='mathlingua-related-header'>Related</span>")
                for (item in relatedGrp.relatedSection.items) {
                    builder.append("<span class='mathlingua-related-item'>${item.text}</span>")
                }
                builder.append("</span>")
            }
        }

        builder.append("</span>")
        return builder.toString()
    }

    private fun capitalizeWords(text: String): String {
        val calledWords = mutableListOf<String>()
        val words = text.split(" ")
        for (i in words.indices) {
            val word = words[i]
            if (i > 0 && (word == "a" || word == "an" || word == "the")) {
                calledWords.add(word)
            } else {
                calledWords.add(word.replaceFirstChar { it.uppercase() })
            }
        }
        return calledWords.joinToString(" ")
    }

    override fun generateCode(node: Phase2Node): String {
        if (literal) {
            return node.toCode(false, 0, this).getCode()
        }

        return when (node) {
            is StatesGroup -> {
                val builder = StringBuilder()
                builder.append("<span class='mathlingua-data'>")
                builder.append("<span class='mathlingua-called'>")
                val called =
                    capitalizeWords(node.calledSection.forms.first().removeSurrounding("\"", "\""))
                builder.append(parseMarkdown(called))
                builder.append("</span>")
                builder.append(
                    node.copy(
                            metaDataSection = null,
                            calledSection =
                                if (node.calledSection.forms.size == 1) {
                                    // use an empty list to specify that the called: section should
                                    // not
                                    // be rendered since the single form that the item is called is
                                    // used
                                    // as the title
                                    CalledSection(forms = emptyList())
                                } else {
                                    node.calledSection
                                })
                        .toCode(false, 0, writer = newCodeWriter(defines, states, literal))
                        .getCode())
                builder.append("</span>")
                if (node.metaDataSection != null) {
                    builder.append(generateMetaDataSectionCode(node.metaDataSection))
                }
                builder.toString()
            }
            is DefinesGroup -> {
                val builder = StringBuilder()
                builder.append("<span class='mathlingua-data'>")
                builder.append("<span class='mathlingua-called'>")
                val called =
                    capitalizeWords(node.calledSection.forms.first().removeSurrounding("\"", "\""))
                builder.append(parseMarkdown(called))
                builder.append("</span>")
                builder.append(
                    node.copy(metaDataSection = null)
                        .let {
                            // if there is only one "called" form then use an empty list of called
                            // forms to signal that the "called:" section shouldn't be rendered
                            // since
                            // the called form is used as the title
                            if (it.calledSection.forms.size == 1) {
                                it.copy(calledSection = CalledSection(forms = emptyList()))
                            } else {
                                it
                            }
                        }
                        .toCode(false, 0, writer = newCodeWriter(defines, states, literal))
                        .getCode())
                builder.append("</span>")
                if (node.metaDataSection != null) {
                    builder.append(generateMetaDataSectionCode(node.metaDataSection!!))
                }
                builder.toString()
            }
            is TheoremGroup -> {
                val builder = StringBuilder()
                builder.append("<span class='mathlingua-data'>")
                val names = node.theoremSection.names
                if (names.isNotEmpty()) {
                    builder.append("<span class='mathlingua-called'>")
                    val name = capitalizeWords(names.first().removeSurrounding("\"", "\""))
                    builder.append(parseMarkdown(name))
                    builder.append("</span>")
                }
                builder.append(
                    node.copy(
                            metaDataSection = null,
                            proofSection = null,
                            theoremSection =
                                if (node.theoremSection.names.size == 1) {
                                    // the single name is already listed as the title
                                    TheoremSection(names = emptyList())
                                } else {
                                    node.theoremSection
                                })
                        .toCode(false, 0, writer = newCodeWriter(defines, states, literal))
                        .getCode())
                builder.append("</span>")
                if (node.proofSection != null) {
                    val id = getRandomUuid()
                    builder.append(
                        "<hr/><div class='mathlingua-proof-header' onclick=\"toggleProof('$id')\">")
                    builder.append(
                        "Proof<span class='mathlingua-proof-icon' id=\"icon-$id\">&#9662;</span></div>")
                    builder.append("<span class='mathlingua-proof-hidden' id=\"proof-$id\">")
                    val writer = newCodeWriter(defines, states, literal)
                    writer.writeText(node.proofSection.text)
                    builder.append(writer.getCode())
                    builder.append("</span>")
                }
                if (node.metaDataSection != null) {
                    builder.append(generateMetaDataSectionCode(node.metaDataSection))
                }
                builder.toString()
            }
            is AxiomGroup -> {
                val builder = StringBuilder()
                builder.append("<span class='mathlingua-data'>")
                val names = node.axiomSection.names
                if (names.isNotEmpty()) {
                    builder.append("<span class='mathlingua-called'>")
                    val name = capitalizeWords(names.first().removeSurrounding("\"", "\""))
                    builder.append(parseMarkdown(name))
                    builder.append("</span>")
                }
                builder.append(
                    node.copy(
                            metaDataSection = null,
                            axiomSection =
                                if (node.axiomSection.names.size == 1) {
                                    // the single name is already listed as the title
                                    AxiomSection(names = emptyList())
                                } else {
                                    node.axiomSection
                                })
                        .toCode(false, 0, writer = newCodeWriter(defines, states, literal))
                        .getCode())
                builder.append("</span>")
                if (node.metaDataSection != null) {
                    builder.append(generateMetaDataSectionCode(node.metaDataSection))
                }
                builder.toString()
            }
            is ConjectureGroup -> {
                val builder = StringBuilder()
                builder.append("<span class='mathlingua-data'>")
                val names = node.conjectureSection.names
                if (names.isNotEmpty()) {
                    builder.append("<span class='mathlingua-called'>")
                    val name = capitalizeWords(names.first().removeSurrounding("\"", "\""))
                    builder.append(parseMarkdown(name))
                    builder.append("</span>")
                }
                builder.append(
                    node.copy(
                            metaDataSection = null,
                            conjectureSection =
                                if (node.conjectureSection.names.size == 1) {
                                    // the single name is already listed as the title
                                    ConjectureSection(names = emptyList())
                                } else {
                                    node.conjectureSection
                                })
                        .toCode(false, 0, writer = newCodeWriter(defines, states, literal))
                        .getCode())
                builder.append("</span>")
                if (node.metaDataSection != null) {
                    builder.append(generateMetaDataSectionCode(node.metaDataSection))
                }
                builder.toString()
            }
            is TopicGroup -> {
                val builder = StringBuilder()
                builder.append("<span class='mathlingua-topic-group'>")
                for (name in node.topicSection.names) {
                    builder.append("<span class='mathlingua-topic-name'>")
                    builder.append(name)
                    builder.append("</span>")
                }
                if (node.id != null) {
                    builder.append("<span class='mathlingua-topic-group-id'>(")
                    builder.append(node.id)
                    builder.append(")</span>")
                }
                builder.append("<span class='mathlingua-topic-content'>")
                builder.append(parseMarkdown(node.contentSection.text))
                builder.append("</span>")
                builder.append("</span>")
                builder.toString()
            }
            is NoteGroup -> {
                val builder = StringBuilder()
                builder.append("<span class='mathlingua-note-group'>")
                builder.append("<span class='mathlingua-note-title'>Note</span>")
                if (node.id != null) {
                    builder.append("<span class='mathlingua-note-group-id'>(")
                    builder.append(node.id)
                    builder.append(")</span>")
                }
                builder.append("<span class='mathlingua-note-content'>")
                builder.append(parseMarkdown(node.contentSection.text))
                builder.append("</span>")
                builder.append("</span>")
                builder.toString()
            }
            else -> {
                node.toCode(false, 0, writer = this).getCode()
            }
        }
    }

    override fun append(node: Phase2Node, hasDot: Boolean, indent: Int) {
        if (literal) {
            builder.append(
                node.toCode(
                        hasDot,
                        indent,
                        HtmlCodeWriter(
                            defines = emptyList(),
                            states = emptyList(),
                            axioms = emptyList(),
                            literal = literal))
                    .getCode())
            return
        }

        builder.append(
            node.toCode(hasDot, indent, newCodeWriter(defines, states, literal)).getCode())
    }

    override fun writeHeader(header: String) {
        val className =
            if (literal) {
                "mathlingua-header-literal"
            } else {
                "mathlingua-header"
            }
        builder.append("<span class='$className'>")
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
                "$$$${code
                    .replace("{", "\\{")
                    .replace("}", "\\}")
                    // replace _\{...\} with _{...}
                    // because that is required to support things
                    // like M_{i, j}
                    .replace(Regex("_\\\\\\{(.*?)\\\\\\}"), "_{$1}")}$$$")
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
                .toCode(false, 0, MathLinguaCodeWriter(emptyList(), emptyList(), emptyList()))
                .getCode()
        builder.append(stmt.removeSurrounding("'", "'"))
        builder.append(']')
        builder.append("</span>")
    }

    private fun getUrlWithoutSpaces(url: String) = url.replace(Regex("[ \\r\\n\\t]+"), "")

    private fun getUrlTitle(url: String, name: String?): String {
        return if (name != null) {
            name
        } else {
            val urlNoSpace = getUrlWithoutSpaces(url)
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
    }

    override fun writeUrl(url: String, name: String?) {
        if (literal) {
            builder.append("<span class=\"literal-mathlingua-text\">\"$url\"</span>")
            return
        }
        val urlNoSpace = getUrlWithoutSpaces(url.removeSurrounding("\"", "\""))
        val title = getUrlTitle(url, name)
        builder.append(
            "<span class=\"mathlingua-url\"><a class=\"mathlingua-link\" target=\"_blank\" href=\"${urlNoSpace.removeSurrounding("\"", "\"")}\">$title</a></span>")
    }

    override fun writeText(text: String) {
        if (literal) {
            val displayTextBuilder = StringBuilder()
            if (!text.startsWith("\"")) {
                displayTextBuilder.append("\"")
            }
            displayTextBuilder.append(text)
            if (!text.endsWith("\"")) {
                displayTextBuilder.append("\"")
            }
            builder.append(
                "<span class=\"literal-mathlingua-text-no-render\"> $displayTextBuilder </span>")
            return
        }

        val innerText = text.removeSurrounding("\"", "\"")
        if (innerText.startsWith("@") && !innerText.contains(' ')) {
            builder.append("<span class='mathlingua-text-no-render'>")
            builder.append(
                "<a class=\"mathlingua-link\" onclick=\"mathlinguaViewSignature('${innerText.removePrefix("@")}')\">$innerText</a>")
            builder.append("</span>")
            return
        }

        if (shouldExpand()) {
            val textWithBreaks = parseMarkdown(innerText)
            val expansion = expandTextAsWritten(textWithBreaks, false, defines, states, axioms)
            val title =
                textWithBreaks +
                    if (expansion.errors.isNotEmpty()) {
                            "\n\nWarning:\n" + expansion.errors.joinToString("\n\n")
                        } else {
                            ""
                        }
                        .removeSurrounding("\"", "\"")

            val expanded = getExpandedMarkdownedText(innerText)
            builder.append("<span class='mathlingua-text' title=\"${title.replace("\"", "")}\">")
            builder.append(expanded)
            builder.append("</span>")
        } else {
            builder.append("<span class='mathlingua-text-no-render'>")
            builder.append(parseMarkdown(innerText))
            builder.append("</span>")
        }
    }

    private fun prettyPrintTexTalk(text: String): String {
        val newText = "{${text}}"
        val lhsParsed = newTexTalkParser().parse(newTexTalkLexer(newText))
        return if (lhsParsed.errors.isEmpty()) {
                val patternsToWrittenAs = getPatternsToWrittenAs(defines, states, axioms)
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

    override fun writeDirectStatement(stmtText: String, root: Validation<ExpressionTexTalkNode>) {
        writeStatement(stmtText, root, direct = true)
    }

    override fun writeStatement(stmtText: String, root: Validation<ExpressionTexTalkNode>) {
        writeStatement(stmtText, root, direct = false)
    }

    fun writeStatement(stmtText: String, root: Validation<ExpressionTexTalkNode>, direct: Boolean) {
        if (literal || direct) {
            val text =
                if (literal) {
                    "'$stmtText'"
                } else {
                    stmtText
                }
            val className =
                if (literal) {
                    "mathlingua-statement-no-render"
                } else {
                    "mathlingua-statement"
                }
            builder.append("<span class=\"$className\">$text</span>")
            return
        }

        val dropdownIndex = statementIndex++
        builder.append(
            "<div class='mathlingua-statement-container' onclick=\"mathlinguaToggleDropdown('statement-$dropdownIndex')\">")
        if (shouldExpand()) {
            val expansionErrors = mutableListOf<String>()
            if (root is ValidationFailure) {
                expansionErrors.addAll(root.errors.map { it.message })
            }
            val fullExpansion =
                if (root is ValidationSuccess && (defines.isNotEmpty() || states.isNotEmpty())) {
                    val patternsToWrittenAs = getPatternsToWrittenAs(defines, states, axioms)
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
                builder.append("$$$${prettyPrintTexTalk(lhs)}$$$")
                writeSpace()
                writeDirect("<span class='mathlingua-is'>is</span>")
                writeSpace()
                val rhs = stmtText.substring(index + IS.length).trim()
                builder.append("$$$${prettyPrintTexTalk(rhs)}$$$")
            } else if (stmtText.contains(IN)) {
                val index = stmtText.indexOf(IN)
                val lhs = stmtText.substring(0, index)
                builder.append("$$$")
                builder.append(prettyPrintTexTalk(lhs))
                writeDirect(" \\in ")
                val rhs = stmtText.substring(index + IN.length).trim()
                builder.append(prettyPrintTexTalk(rhs))
                builder.append("$$$")
            } else {
                if (root is ValidationSuccess && (defines.isNotEmpty() || states.isNotEmpty())) {
                    builder.append("$$$$fullExpansion$$$")
                } else {
                    builder.append("$$$$stmtText$$$")
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
            val signatures =
                findAllStatementSignatures(stmt, ignoreLhsEqual = false, newLocationTracker())
            if (signatures.isNotEmpty()) {
                builder.append(
                    "<div class='mathlingua-dropdown-menu-hidden' id='statement-$dropdownIndex'>")
                for (sig in signatures) {
                    builder.append(
                        "<a class='mathlingua-dropdown-menu-item' onclick=\"mathlinguaViewSignature('${sig.form.replace("\\", "\\\\")}', 'statement-$dropdownIndex')\">")
                    builder.append(sig.form)
                    builder.append("</a>")
                }
                builder.append("</div>")
            }
        }
    }

    override fun writeIdentifier(name: String, isVarArgs: Boolean) {
        builder.append("<span class='mathlingua-identifier'>$$$")
        builder.append(prettyPrintIdentifier(name))
        if (isVarArgs) {
            builder.append("...")
        }
        builder.append("$$$</span>")
    }

    override fun writeDirect(text: String) {
        if (literal) {
            writeText(text)
            return
        }
        builder.append(text)
    }

    private fun processMathCodeBlocks(text: String): List<BlockCommentSection> {
        val result = mutableListOf<BlockCommentSection>()

        var remaining = text.removeSurrounding("::", "::")
        while (true) {
            val index = remaining.indexOf("```math")
            if (index < 0) {
                result.add(BlockCommentSection(text = remaining, isMathlinguaCode = false))
                break
            }
            result.add(
                BlockCommentSection(text = remaining.substring(0, index), isMathlinguaCode = false))
            remaining = remaining.substring(index + 7)
            var endIndex = remaining.indexOf("```")
            if (endIndex < 0) {
                endIndex = remaining.length
            }
            val mlgCode = remaining.substring(0, endIndex)
            when (val validation = FrontEnd.parse(mlgCode)
            ) {
                is ValidationSuccess -> {
                    val mlgHtml =
                        HtmlCodeWriter(
                                defines = listOf(),
                                states = listOf(),
                                axioms = listOf(),
                                literal = true)
                            .generateCode(validation.value)
                    val htmlWithoutBreaks = mlgHtml.replace(Regex("<br/><br/>(<br/>)*"), "")
                    result.add(
                        BlockCommentSection(text = htmlWithoutBreaks, isMathlinguaCode = true))
                }
                else -> {
                    result.add(
                        BlockCommentSection(text = "```$mlgCode```", isMathlinguaCode = false))
                }
            }
            remaining = remaining.substring(endIndex + 3)
        }

        return result.map {
            if (it.isMathlinguaCode) {
                it
            } else {
                BlockCommentSection(
                    text = it.text.split("\n").joinToString("\n") { line -> line.trimStart() },
                    isMathlinguaCode = it.isMathlinguaCode)
            }
        }
    }

    override fun writeBlockComment(text: String) {
        builder.append("<span class='mathlingua-block-comment'>")
        builder.append(getExpandedMarkdownedText(text))
        builder.append("</span>")
    }

    private fun getExpandedMarkdownedText(text: String): String {
        val comment =
            processMathCodeBlocks(text).joinToString(" ") {
                if (it.isMathlinguaCode) {
                    "<div class='mathlingua-top-level-comment-block'>${it.text}</div>"
                } else {
                    parseMarkdown(it.text)
                }
            }
        return expandTextAsWritten(comment, false, defines, states, axioms).text ?: comment
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
        defines: List<DefinesGroup>, states: List<StatesGroup>, literal: Boolean
    ) = HtmlCodeWriter(defines, states, axioms, literal)

    override fun getCode(): String {
        val text =
            builder
                .toString()
                .replace(Regex("(\\s*<\\s*br\\s*/\\s*>\\s*)+$"), "")
                .replace(Regex("^(\\s*<\\s*br\\s*/\\s*>\\s*)+$"), "")
        return "<span class='mathlingua'>$text</span>"
    }

    override fun writeHorizontalLine() {
        if (literal) {
            return
        }
        builder.append("<hr/>")
    }

    private fun shouldExpand() = defines.isNotEmpty() || states.isNotEmpty()
}

class MathLinguaCodeWriter(
    val defines: List<DefinesGroup>, val states: List<StatesGroup>, val axioms: List<AxiomGroup>
) : CodeWriter {
    private val builder = StringBuilder()

    override fun generateCode(node: Phase2Node) = node.toCode(false, 0, writer = this).getCode()

    override fun append(node: Phase2Node, hasDot: Boolean, indent: Int) {
        builder.append(node.toCode(hasDot, indent, newCodeWriter(defines, states, true)).getCode())
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
                .toCode(false, 0, newCodeWriter(emptyList(), emptyList(), true))
                .getCode()
        builder.append(stmt.removeSurrounding("'", "'"))
        builder.append(']')
    }

    override fun writeText(text: String) {
        if (!text.startsWith("\"")) {
            builder.append('"')
        }
        builder.append(expandTextAsWritten(text, false, defines, states, axioms).text ?: text)
        if (!text.endsWith("\"")) {
            builder.append('"')
        }
    }

    override fun writeBlockComment(text: String) {
        builder.append(text)
    }

    override fun writeUrl(url: String, name: String?) {
        if (!url.startsWith("\"")) {
            builder.append('"')
        }
        builder.append(url)
        if (!url.endsWith("\"")) {
            builder.append('"')
        }
    }

    override fun writeDirectStatement(stmtText: String, root: Validation<ExpressionTexTalkNode>) {
        // this code writer only supports writing results literally and so this method does the
        // same thing as writeStatement
        writeStatement(stmtText, root)
    }

    override fun writeStatement(stmtText: String, root: Validation<ExpressionTexTalkNode>) {
        if (root is ValidationSuccess && (defines.isNotEmpty() || states.isNotEmpty())) {
            val patternsToWrittenAs = getPatternsToWrittenAs(defines, states, axioms)
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
        defines: List<DefinesGroup>, states: List<StatesGroup>, literal: Boolean
    ) = MathLinguaCodeWriter(defines, states, axioms)

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
        // the following finds the first index of ' in `remaining` that does not come right
        // after a backslash, and returns -1 if not found.  The code returns -1 if the only
        // occurrences of ' are in the form \' and also works correctly if `remaining` starts
        // with '
        val startIndex = Regex("([^\\\\]|^)'").find(remaining)?.range?.endInclusive ?: -1
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

private fun parseMarkdown(text: String): String {
    val flavor = CommonMarkFlavourDescriptor()
    val tree = MarkdownParser(flavor).buildMarkdownTreeFromString(text)
    return HtmlGenerator(text, tree, flavor).generateHtml()
}

private fun expandTextAsWritten(
    text: String,
    addQuotes: Boolean,
    defines: List<DefinesGroup>,
    states: List<StatesGroup>,
    axioms: List<AxiomGroup>
): Expansion {
    if (defines.isEmpty() && states.isEmpty()) {
        return Expansion(text = text, errors = emptyList())
    }

    val errors = mutableListOf<String>()
    val patternsToWrittenAs = getPatternsToWrittenAs(defines, states, axioms)
    val parts = splitByMathlingua(parseMarkdown(text))
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
                if (addQuotes) {
                    builder.append('\'')
                }
                builder.append(part.text)
                if (addQuotes) {
                    builder.append('\'')
                }
            }
        } else {
            builder.append(part.text.replace("\\'", "'"))
        }
    }
    return Expansion(text = builder.toString(), errors = errors)
}

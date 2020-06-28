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

package mathlingua.common

import mathlingua.common.chalktalk.phase1.newChalkTalkLexer
import mathlingua.common.chalktalk.phase1.newChalkTalkParser
import mathlingua.common.chalktalk.phase2.*
import mathlingua.common.chalktalk.phase2.ast.Document
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.clause.Statement
import mathlingua.common.chalktalk.phase2.ast.metadata.item.StringSectionGroup
import mathlingua.common.chalktalk.phase2.ast.toplevel.DefinesGroup
import mathlingua.common.chalktalk.phase2.ast.toplevel.RepresentsGroup
import mathlingua.common.chalktalk.phase2.ast.toplevel.TopLevelGroup
import mathlingua.common.chalktalk.phase2.ast.validateDocument
import mathlingua.common.textalk.Command
import mathlingua.common.textalk.OperatorTexTalkNode
import mathlingua.common.textalk.TexTalkNode
import mathlingua.common.transform.*

data class Parse(val document: Document, val tracker: LocationTracker)

data class Signature(val form: String, val location: Location)

object MathLingua {
    fun parse(input: String): Validation<Document> =
        when (val validation = parseWithLocations(input)) {
            is ValidationSuccess -> validationSuccess(validation.value.document)
            is ValidationFailure -> validationFailure(validation.errors)
        }

    fun parseWithLocations(input: String): Validation<Parse> {
        val lexer = newChalkTalkLexer(input)

        val allErrors = mutableListOf<ParseError>()
        allErrors.addAll(lexer.errors())

        val parser = newChalkTalkParser()
        val (root, errors) = parser.parse(lexer)
        allErrors.addAll(errors)

        if (root == null || allErrors.isNotEmpty()) {
            return validationFailure(allErrors)
        }

        val tracker = newLocationTracker()
        return when (val documentValidation = validateDocument(root, tracker)) {
            is ValidationSuccess -> validationSuccess(Parse(
                    document = documentValidation.value,
                    tracker = tracker
            ))
            is ValidationFailure -> {
                allErrors.addAll(documentValidation.errors)
                validationFailure(allErrors)
            }
        }
    }

    fun justify(text: String, width: Int) = mathlingua.common.justify(text, width)

    fun prettyPrintIdentifier(text: String) = mathlingua.common.chalktalk.phase2.prettyPrintIdentifier(text)

    fun signatureOf(group: TopLevelGroup) = getSignature(group)

    fun signatureOf(command: Command) = command.signature()

    fun findAllSignatures(node: Phase2Node, locationTracker: LocationTracker) = locateAllSignatures(node, locationTracker).toList()

    fun findAllCommands(node: Phase2Node) = locateAllCommands(node).toList()

    fun findUndefinedSignatures(input: String, supplemental: List<String>): List<Signature> {
        val definedSignatures = mutableSetOf<String>()
        definedSignatures.addAll(getAllDefinedSignatures(input))
        for (sup in supplemental) {
            definedSignatures.addAll(getAllDefinedSignatures(sup))
        }

        return when (val validation = parseWithLocations(input)) {
            is ValidationSuccess -> {
                val result = mutableListOf<Signature>()
                val signatures = findAllSignatures(validation.value.document, validation.value.tracker)
                for (sig in signatures) {
                    if (!definedSignatures.contains(sig.form)) {
                        result.add(sig)
                    }
                }
                result
            }
            is ValidationFailure -> emptyList()
        }
    }

    private fun getAllDefinedSignatures(input: String): List<String> {
        return when (val validation = parse(input)) {
            is ValidationSuccess -> {
                val result = mutableListOf<String>()
                val document = validation.value
                result.addAll(document.defines.mapNotNull { it.signature })
                result.addAll(document.represents.mapNotNull { it.signature })
                result
            }
            is ValidationFailure -> emptyList()
        }
    }

    fun expandAtPosition(
        text: String,
        row: Int,
        column: Int,
        defines: List<DefinesGroup>,
        represents: List<RepresentsGroup>
    ): Validation<Document> = when (val validation = parseWithLocations(text)) {
        is ValidationFailure -> validationFailure(validation.errors)
        is ValidationSuccess -> {
            val doc = validation.value.document
            val tracker = validation.value.tracker
            val target = findNode(tracker, doc, row, column)
            val newDoc = expandAtNode(doc, target, defines, represents) as Document
            validationSuccess(newDoc)
        }
    }

    fun expand(doc: Document) = fullExpandComplete(doc)

    fun getPatternsToWrittenAs(
        defines: List<DefinesGroup>,
        represents: List<RepresentsGroup>
    ): Map<OperatorTexTalkNode, String> {
        val result = mutableMapOf<OperatorTexTalkNode, String>()
        for (rep in represents) {
            val allItems = rep.metaDataSection?.items
            var writtenAs: String? = null
            if (allItems != null) {
                for (item in allItems) {
                    if (item is StringSectionGroup &&
                        item.section.name == "written" &&
                        item.section.values.isNotEmpty()) {
                        writtenAs = item.section.values[0].removeSurrounding("\"", "\"")
                        break
                    }
                }
            }

            if (writtenAs == null) {
                continue
            }

            val validation = rep.id.texTalkRoot
            if (validation is ValidationSuccess) {
                val exp = validation.value
                if (exp.children.size == 1 && exp.children[0] is OperatorTexTalkNode) {
                    result[exp.children[0] as OperatorTexTalkNode] = writtenAs
                }
            }
        }

        for (def in defines) {
            val allItems = def.metaDataSection?.items
            var writtenAs: String? = null
            if (allItems != null) {
                for (item in allItems) {
                    if (item is StringSectionGroup &&
                            item.section.name == "written" &&
                            item.section.values.isNotEmpty()) {
                        writtenAs = item.section.values[0].removeSurrounding("\"", "\"")
                        break
                    }
                }
            }

            if (writtenAs == null) {
                continue
            }

            val validation = def.id.texTalkRoot
            if (validation is ValidationSuccess) {
                val exp = validation.value
                if (exp.children.size == 1 && exp.children[0] is Command) {
                    val cmd = exp.children[0] as Command
                    result[OperatorTexTalkNode(
                        lhs = null,
                        command = cmd,
                        rhs = null
                    )] = writtenAs
                }
            }
        }

        return result
    }

    fun expandWrittenAs(node: TexTalkNode, defines: List<DefinesGroup>, represents: List<RepresentsGroup>) =
        expandAsWritten(node, getPatternsToWrittenAs(defines, represents))

    fun expandWrittenAs(
        phase2Node: Phase2Node,
        patternToExpansion: Map<OperatorTexTalkNode, String>
    ): Phase2Node {
        return phase2Node.transform {
            when (it) {
                is Statement -> when (val validation = it.texTalkRoot) {
                    is ValidationFailure -> it
                    is ValidationSuccess -> {
                        val texTalkNode = validation.value
                        val newText = expandAsWritten(texTalkNode, patternToExpansion)
                        Statement(
                                text = newText,
                                texTalkRoot = validation
                        )
                    }
                }
                else -> it
            }
        }
    }

    fun printExpanded(input: String, supplemental: String, html: Boolean): Validation<String> {
        val totalText = "$input\n\n\n$supplemental"
        val totalTextValidation = parse(totalText)
        val defines = when (totalTextValidation) {
            is ValidationFailure -> emptyList()
            is ValidationSuccess -> totalTextValidation.value.defines
        }
        val represents = when (totalTextValidation) {
            is ValidationFailure -> emptyList()
            is ValidationSuccess -> totalTextValidation.value.represents
        }

        val result = StringBuilder()
        val errors = mutableListOf<ParseError>()
        for (part in input.split("\n\n").filter { it.isNotBlank() }) {
            when (val validation = parse(part)) {
                is ValidationFailure -> {
                    errors.addAll(validation.errors)
                }
                is ValidationSuccess -> {
                    result.append(prettyPrint(validation.value, defines, represents, html))
                }
            }
        }

        return if (errors.isNotEmpty()) {
            validationFailure(errors)
        } else {
            validationSuccess(result.toString())
        }
    }

    fun prettyPrint(
        node: Phase2Node,
        defines: List<DefinesGroup>,
        represents: List<RepresentsGroup>,
        html: Boolean
    ): String {
        val writer = if (html) {
            HtmlCodeWriter(defines = defines, represents = represents)
        } else {
            MathLinguaCodeWriter(defines = defines, represents = represents)
        }
        val code = node.toCode(false, 0, writer = writer).getCode()
        return if (html) {
            getHtml(code.replace("<br/><br/><br/>", "<br/><br/>"))
        } else {
            code
        }
    }
}

private fun getHtml(body: String) = """
<!DOCTYPE html>
<html>
    <head>
        <link rel="stylesheet"
              href="https://cdn.jsdelivr.net/npm/katex@0.11.1/dist/katex.min.css"
              integrity="sha384-zB1R0rpPzHqg7Kpt0Aljp8JPLqbXI3bhnPWROx27a9N0Ll6ZP/+DiW/UqRcLbRjq"
              crossorigin="anonymous">
        <script defer
                src="https://cdn.jsdelivr.net/npm/katex@0.11.1/dist/katex.min.js"
                integrity="sha384-y23I5Q6l+B6vatafAwxRu/0oK/79VlbSz7Q9aiSZUvyWYIYsd+qj+o24G5ZU2zJz"
                crossorigin="anonymous">
        </script>
        <script>
            function buildMathFragment(rawText) {
                var text = rawText;
                if (text[0] === '"') {
                    text = text.substring(1);
                }
                if (text[text.length - 1] === '"') {
                    text = text.substring(0, text.length - 1);
                }
                text = text.replace(/([a-zA-Z0-9])\?/g, '${'$'}1');
                const fragment = document.createDocumentFragment();
                var buffer = '';
                var i = 0;
                while (i < text.length) {
                    if (text[i] === '\\' && text[i+1] === '[') {
                        i += 2; // skip over \ and [
                        fragment.appendChild(document.createTextNode(buffer));
                        buffer = '';

                        const span = document.createElement('span');
                        var math = '';
                        while (i < text.length &&
                            !(text[i] === '\\' && text[i+1] === ']')) {
                            math += text[i++];
                        }
                        if (text[i] === '\\') {
                            i++; // move past the \
                        }
                        if (text[i] === ']') {
                            i++; // move past the ]
                        }
                        try {
                            katex.render(math, span, {
                                throwOnError: true,
                                displayMode: true
                            });
                        } catch {
                            span.appendChild(document.createTextNode(math));
                        }
                        fragment.appendChild(span);
                    } else if (text[i] === '\\' && text[i+1] === '(') {
                        i += 2; // skip over \ and ()
                        fragment.appendChild(document.createTextNode(buffer));
                        buffer = '';

                        const span = document.createElement('span');
                        var math = '';
                        while (i < text.length &&
                            !(text[i] === '\\' && text[i+1] === ')')) {
                            math += text[i++];
                        }
                        if (text[i] === '\\') {
                            i++; // move past the \
                        }
                        if (text[i] === ')') {
                            i++; // move past the )
                        }
                        try {
                            katex.render(math, span, {
                                throwOnError: true,
                                displayMode: true
                            });
                        } catch {
                            span.appendChild(document.createTextNode(math));
                        }
                        fragment.appendChild(span);
                    } else if (text[i] === '${'$'}' && text[i+1] === '${'$'}') {
                        i += 2; // skip over ${'$'} and ${'$'}
                        fragment.appendChild(document.createTextNode(buffer));
                        buffer = '';

                        const span = document.createElement('span');
                        var math = '';
                        while (i < text.length &&
                            !(text[i] === '${'$'}' && text[i+1] === '${'$'}')) {
                            math += text[i++];
                        }
                        if (text[i] === '${'$'}') {
                            i++; // move past the ${'$'}
                        }
                        if (text[i] === '${'$'}') {
                            i++; // move past the ${'$'}
                        }
                        try {
                            katex.render(math, span, {
                                throwOnError: true,
                                displayMode: true
                            });
                        } catch {
                            span.appendChild(document.createTextNode(math));
                        }
                        fragment.appendChild(span);
                    } else if (text[i] === '${'$'}') {
                        i++; // skip over the ${'$'}
                        fragment.appendChild(document.createTextNode(buffer));
                        buffer = '';

                        const span = document.createElement('span');
                        var math = '';
                        while (i < text.length &&
                             text[i] !== '${'$'}') {
                            math += text[i++];
                        }
                        if (text[i] === '${'$'}') {
                            i++; // move past the ${'$'}
                        }
                        try {
                            katex.render(math, span, {
                                throwOnError: true,
                                displayMode: true
                            });
                        } catch {
                            span.appendChild(document.createTextNode(math));
                        }
                        fragment.appendChild(span);
                    } else {
                        buffer += text[i++];
                    }
                }

                if (buffer.length > 0) {
                    fragment.appendChild(document.createTextNode(buffer));
                }

                return fragment;
            }

            function render(node) {
                /*
                 * The layout for nodes corresponding to text in a "written:" section
                 * looks like the following.  Determine if any child nodes of 'node'
                 * that are text nodes are in a "written:" section so that the text
                 * can be trated as if it is in math mode.
                 *
                 * <span class='mathlingua'>
                 *   ...
                 *   <span class='mathlingua-header'>
                 *     written:
                 *   </span>
                 *   ...
                 *   <span class='mathlingua-text'>
                 *     "some text"
                 *   </span>
                 * </span>
                 */
                let isInWritten = false;
                const parent = node.parentNode;
                if (parent && node.className === 'mathlingua-text') {
                    for (let j=0; j<parent.childNodes.length; j++) {
                        const sibling = parent.childNodes[j];
                        if (sibling.childNodes.length > 0 &&
                            sibling.childNodes[0].nodeType === 3 &&
                            sibling.childNodes[0].textContent === 'written:') {
                            isInWritten = true;
                            break;
                        }
                    }
                }

                for (let i = 0; i < node.childNodes.length; i++) {
                    const child = node.childNodes[i];

                    // node is an element node => nodeType === 1
                    // node is an attribute node => nodeType === 2
                    // node is a text node => nodeType === 3
                    // node is a comment node => nodeType === 8
                    if (child.nodeType === 3) {
                        let text = child.textContent;
                        if (text.trim()) {
                            if (isInWritten) {
                                // if the text is in a written: section
                                // turn "some text" to \[some text\]
                                // so the text is in math mode
                                if (text[0] === '"') {
                                    text = text.substring(1);
                                }
                                if (text[text.length - 1] === '"') {
                                    text = text.substring(0, text.length - 1);
                                }
                                text = '\\[' + text + '\\]';
                            }
                            const fragment = buildMathFragment(text);
                            i += fragment.childNodes.length - 1;
                            node.replaceChild(fragment, child);
                        }
                    } else if (child.nodeType === 1) {
                        render(child);
                    }
                }
            }
        </script>
        <style>
            .content {
                margin-top: 1em;
                margin-bottom: 1em;
                font-size: 1em;
            }

            .mathlingua {
                font-family: monospace;
            }

            .mathlingua-header {
                font-weight: bold;
                color: #0055bb;
            }

            .mathlingua-whitespace {
                padding: 0;
                margin: 0;
                margin-left: 1ex;
            }

            .mathlingua-id {
                font-weight: bold;
                color: #5500aa;
            }

            .mathlingua-text {
                color: #007700;
            }

            .katex {
                font-size: 0.75em;
            }

            .katex-display {
                display: contents;
            }

            .katex-display > .katex {
                display: contents;
            }

            .katex-display > .katex > .katex-html {
                display: contents;
            }
        </style>
    </head>
    <body onload="render(document.body)">
        <div class="content">
            $body
        </div>
    </body>
</html>
"""

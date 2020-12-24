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

package mathlingua

import mathlingua.chalktalk.phase1.newChalkTalkLexer
import mathlingua.chalktalk.phase1.newChalkTalkParser
import mathlingua.chalktalk.phase2.HtmlCodeWriter
import mathlingua.chalktalk.phase2.MathLinguaCodeWriter
import mathlingua.chalktalk.phase2.ast.Document
import mathlingua.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.chalktalk.phase2.ast.clause.Statement
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesCollectsGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesEvaluatedGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGeneratedGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesInstantiatedGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesMapsGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesMeansGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.FoundationGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.mutually.MutuallyGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resource.ResourceGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.ConjectureGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremGroup
import mathlingua.chalktalk.phase2.ast.validateDocument
import mathlingua.support.Location
import mathlingua.support.LocationTracker
import mathlingua.support.ParseError
import mathlingua.support.Validation
import mathlingua.support.ValidationFailure
import mathlingua.support.ValidationSuccess
import mathlingua.support.newLocationTracker
import mathlingua.support.validationFailure
import mathlingua.support.validationSuccess
import mathlingua.textalk.Command
import mathlingua.textalk.ExpressionTexTalkNode
import mathlingua.textalk.OperatorTexTalkNode
import mathlingua.textalk.TexTalkNode
import mathlingua.transform.expandAsWritten
import mathlingua.transform.getSignature
import mathlingua.transform.locateAllCommands
import mathlingua.transform.locateAllSignatures
import mathlingua.transform.signature

data class Parse(val document: Document, val tracker: LocationTracker)

data class Signature(val form: String, val location: Location)

data class ContentLocation(val path: String, val location: Location)

object MathLingua {
    fun parse(input: String): Validation<Document> =
        when (val validation = parseWithLocations(input)
        ) {
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
        return when (val documentValidation = validateDocument(root, tracker)
        ) {
            is ValidationSuccess ->
                validationSuccess(Parse(document = documentValidation.value, tracker = tracker))
            is ValidationFailure -> {
                allErrors.addAll(documentValidation.errors)
                validationFailure(allErrors)
            }
        }
    }

    fun justify(text: String, width: Int) = mathlingua.support.justify(text, width)

    fun prettyPrintIdentifier(text: String) =
        mathlingua.chalktalk.phase2.prettyPrintIdentifier(text)

    fun signatureOf(group: TopLevelGroup) = getSignature(group)

    fun signatureOf(command: Command) = command.signature()

    fun findAllSignatures(node: Phase2Node, locationTracker: LocationTracker) =
        locateAllSignatures(node, locationTracker)

    fun findAllCommands(node: Phase2Node) = locateAllCommands(node).toList()

    private fun getContent(group: TopLevelGroup) =
        when (group) {
                is StatesGroup ->
                    group.copy(
                        id =
                            IdStatement(
                                text = "",
                                texTalkRoot =
                                    validationSuccess(
                                        ExpressionTexTalkNode(children = emptyList()))),
                        metaDataSection = null)
                is DefinesCollectsGroup ->
                    group.copy(
                        id =
                            IdStatement(
                                text = "",
                                texTalkRoot =
                                    validationSuccess(
                                        ExpressionTexTalkNode(children = emptyList()))),
                        metaDataSection = null)
                is DefinesEvaluatedGroup ->
                    group.copy(
                        id =
                            IdStatement(
                                text = "",
                                texTalkRoot =
                                    validationSuccess(
                                        ExpressionTexTalkNode(children = emptyList()))),
                        metaDataSection = null)
                is DefinesGeneratedGroup ->
                    group.copy(
                        id =
                            IdStatement(
                                text = "",
                                texTalkRoot =
                                    validationSuccess(
                                        ExpressionTexTalkNode(children = emptyList()))),
                        metaDataSection = null)
                is DefinesInstantiatedGroup ->
                    group.copy(
                        id =
                            IdStatement(
                                text = "",
                                texTalkRoot =
                                    validationSuccess(
                                        ExpressionTexTalkNode(children = emptyList()))),
                        metaDataSection = null)
                is DefinesMapsGroup ->
                    group.copy(
                        id =
                            IdStatement(
                                text = "",
                                texTalkRoot =
                                    validationSuccess(
                                        ExpressionTexTalkNode(children = emptyList()))),
                        metaDataSection = null)
                is DefinesMeansGroup ->
                    group.copy(
                        id =
                            IdStatement(
                                text = "",
                                texTalkRoot =
                                    validationSuccess(
                                        ExpressionTexTalkNode(children = emptyList()))),
                        metaDataSection = null)
                is ResourceGroup -> group.copy(id = "", metaDataSection = null)
                is TheoremGroup -> group.copy(metaDataSection = null)
                is AxiomGroup -> group.copy(metaDataSection = null)
                is ConjectureGroup -> group.copy(metaDataSection = null)
                else -> throw RuntimeException("Unknown group: ${group.toCode(false, 0).getCode()}")
            }
            .toCode(false, 0)
            .getCode()
            .replace("^\\[]\\n".toRegex(), "")

    fun findUndefinedSignatureLocations(
        files: Map<String, String>
    ): Map<String, Set<ContentLocation>> {
        val definedSignatures = mutableSetOf<String>()
        for (content in files.values) {
            definedSignatures.addAll(getAllDefinedSignatures(content).map { it.form })
        }

        val result = mutableMapOf<String, MutableSet<ContentLocation>>()
        for ((path, content) in files.entries.sortedBy { it.key }) {
            val validation = parseWithLocations(content)
            if (validation is ValidationSuccess) {
                val doc = validation.value.document
                val tracker = validation.value.tracker
                for (signature in findAllSignatures(doc, tracker)) {
                    val key = signature.form
                    if (!definedSignatures.contains(key)) {
                        if (!result.containsKey(key)) {
                            result[key] = mutableSetOf()
                        }
                        result[key]!!.add(
                            ContentLocation(path = path, location = signature.location))
                    }
                }
            }
        }
        return result
    }

    fun findContentLocations(files: Map<String, String>): Map<String, Set<ContentLocation>> {
        val result = mutableMapOf<String, MutableSet<ContentLocation>>()
        for ((path, content) in files.entries.sortedBy { it.key }) {
            val validation = parseWithLocations(content)
            if (validation is ValidationSuccess) {
                val doc = validation.value.document
                val tracker = validation.value.tracker
                for (group in doc.groups) {
                    val groupContent = getContent(group)
                    if (!result.containsKey(groupContent)) {
                        result[groupContent] = mutableSetOf()
                    }
                    result[groupContent]!!.add(
                        ContentLocation(
                            path = path,
                            location = tracker.getLocationOf(group)
                                    ?: Location(row = -1, column = -1)))
                }
            }
        }
        return result
    }

    fun findSignatureLocations(files: Map<String, String>): Map<String, Set<ContentLocation>> {
        val result = mutableMapOf<String, MutableSet<ContentLocation>>()
        for ((path, content) in files.entries.sortedBy { it.key }) {
            val validation = parseWithLocations(content)
            if (validation is ValidationSuccess) {
                val doc = validation.value.document
                val tracker = validation.value.tracker
                for (group in doc.groups) {
                    val signature =
                        when (group) {
                            is DefinesGroup -> {
                                group.signature
                            }
                            is StatesGroup -> {
                                group.signature
                            }
                            else -> {
                                null
                            }
                        }

                    if (signature != null) {
                        if (!result.containsKey(signature)) {
                            result[signature] = mutableSetOf()
                        }
                        result[signature]!!.add(
                            ContentLocation(
                                path = path,
                                location = tracker.getLocationOf(group)
                                        ?: Location(row = -1, column = -1)))
                    }
                }
            }
        }
        return result
    }

    fun findDuplicateContent(input: String, supplemental: List<String>): List<Location> {
        val suppContent =
            when (val validation = parse(supplemental.joinToString("\n\n\n"))
            ) {
                is ValidationFailure -> emptySet()
                is ValidationSuccess -> {
                    validation.value.groups.map { getContent(it) }.toSet()
                }
            }

        return when (val validation = parseWithLocations(input)
        ) {
            is ValidationFailure -> emptyList()
            is ValidationSuccess -> {
                val doc = validation.value.document
                val tracker = validation.value.tracker

                val result = mutableListOf<Location>()

                val inputContentSet = mutableSetOf<String>()
                for (group in doc.groups) {
                    val content = getContent(group)
                    val location = tracker.getLocationOf(group)
                    if (location != null &&
                        (suppContent.contains(content) || inputContentSet.contains(content))) {
                        result.add(location)
                    }
                    inputContentSet.add(content)
                }

                result
            }
        }
    }

    fun findDuplicateSignatures(input: String, supplemental: List<String>): List<Signature> {
        val suppSignatures = mutableSetOf<String>()
        for (sup in supplemental) {
            suppSignatures.addAll(getAllDefinedSignatures(sup).map { it.form })
        }

        val result = mutableListOf<Signature>()
        val signatures = getAllDefinedSignatures(input)

        val dupSet = mutableSetOf<String>()
        for (sig in signatures) {
            if (dupSet.contains(sig.form)) {
                result.add(sig)
            }
            dupSet.add(sig.form)
        }

        for (sig in signatures) {
            if (suppSignatures.contains(sig.form)) {
                result.add(sig)
            }
        }

        return result
    }

    fun findUndefinedSignatures(input: String, supplemental: List<String>): List<Signature> {
        val definedSignatures = mutableSetOf<String>()
        definedSignatures.addAll(getAllDefinedSignatures(input).map { it.form })
        for (sup in supplemental) {
            definedSignatures.addAll(getAllDefinedSignatures(sup).map { it.form })
        }

        return when (val validation = parseWithLocations(input)
        ) {
            is ValidationSuccess -> {
                val result = mutableListOf<Signature>()
                val signatures =
                    findAllSignatures(validation.value.document, validation.value.tracker)
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

    private fun getAllDefinedSignatures(input: String): List<Signature> {
        return when (val validation = parseWithLocations(input)
        ) {
            is ValidationSuccess -> {
                val result = mutableListOf<Signature>()
                val document = validation.value.document
                val tracker = validation.value.tracker
                result.addAll(
                    document.defines().mapNotNull {
                        if (it.signature == null) {
                            null
                        } else {
                            Signature(
                                form = it.signature!!,
                                location = tracker.getLocationOf(it)
                                        ?: Location(row = -1, column = -1))
                        }
                    })
                result.addAll(
                    document.states().mapNotNull {
                        if (it.signature == null) {
                            null
                        } else {
                            Signature(
                                form = it.signature,
                                location = tracker.getLocationOf(it)
                                        ?: Location(row = -1, column = -1))
                        }
                    })
                result
            }
            is ValidationFailure -> emptyList()
        }
    }

    /*
    fun expandAtPosition(
        text: String,
        row: Int,
        column: Int,
        defines: List<DefinesGroup>,
        represents: List<StatesGroup>
    ): Validation<Document> =
        when (val validation = parseWithLocations(text)
        ) {
            is ValidationFailure -> validationFailure(validation.errors)
            is ValidationSuccess -> {
                val doc = validation.value.document
                val tracker = validation.value.tracker
                val target = findNode(tracker, doc, row, column)
                val newDoc = expandAtNode(doc, target, defines, represents) as Document
                validationSuccess(newDoc)
            }
        }
     */

    // fun expand(doc: Document) = fullExpandComplete(doc)

    fun getPatternsToWrittenAs(
        defines: List<DefinesGroup>,
        states: List<StatesGroup>,
        foundations: List<FoundationGroup>,
        mutuallyGroups: List<MutuallyGroup>
    ): Map<OperatorTexTalkNode, String> {
        val allDefines = mutableListOf<DefinesGroup>()
        allDefines.addAll(defines)

        val allStates = mutableListOf<StatesGroup>()
        allStates.addAll(states)

        for (f in foundations) {
            val content = f.foundationSection.content
            if (content is DefinesGroup) {
                allDefines.add(content)
            } else if (content is StatesGroup) {
                allStates.add(content)
            }
        }

        for (m in mutuallyGroups) {
            for (item in m.mutuallySection.items) {
                if (item is DefinesGroup) {
                    allDefines.add(item)
                } else if (item is StatesGroup) {
                    allStates.add(item)
                }
            }
        }

        val result = mutableMapOf<OperatorTexTalkNode, String>()
        for (rep in allStates) {
            val writtenAs =
                rep.writtenSection?.forms?.getOrNull(0)?.removeSurrounding("\"", "\"") ?: continue

            val validation = rep.id.texTalkRoot
            if (validation is ValidationSuccess) {
                val exp = validation.value
                if (exp.children.size == 1 && exp.children[0] is OperatorTexTalkNode) {
                    result[exp.children[0] as OperatorTexTalkNode] = writtenAs
                } else if (exp.children.size == 1 && exp.children[0] is Command) {
                    result[
                        OperatorTexTalkNode(
                            lhs = null, command = exp.children[0] as Command, rhs = null)] =
                        writtenAs
                }
            }
        }

        for (def in allDefines) {
            val writtenAs =
                def.writtenSection?.forms?.getOrNull(0)?.removeSurrounding("\"", "\"") ?: continue

            val validation = def.id.texTalkRoot
            if (validation is ValidationSuccess) {
                val exp = validation.value
                if (exp.children.size == 1 && exp.children[0] is OperatorTexTalkNode) {
                    result[exp.children[0] as OperatorTexTalkNode] = writtenAs
                } else if (exp.children.size == 1 && exp.children[0] is Command) {
                    val cmd = exp.children[0] as Command
                    result[OperatorTexTalkNode(lhs = null, command = cmd, rhs = null)] = writtenAs
                }
            }
        }

        return result
    }

    fun expandWrittenAs(
        node: TexTalkNode,
        defines: List<DefinesGroup>,
        represents: List<StatesGroup>,
        foundations: List<FoundationGroup>,
        mutuallyGroups: List<MutuallyGroup>
    ) =
        expandAsWritten(
            node, getPatternsToWrittenAs(defines, represents, foundations, mutuallyGroups))

    fun expandWrittenAs(
        phase2Node: Phase2Node, patternToExpansion: Map<OperatorTexTalkNode, String>
    ): Phase2Node {
        return phase2Node.transform {
            when (it) {
                is Statement ->
                    when (val validation = it.texTalkRoot
                    ) {
                        is ValidationFailure -> it
                        is ValidationSuccess -> {
                            val texTalkNode = validation.value
                            val expansion = expandAsWritten(texTalkNode, patternToExpansion)
                            Statement(
                                text = expansion.text ?: it.toCode(false, 0).getCode(),
                                texTalkRoot = validation)
                        }
                    }
                else -> it
            }
        }
    }

    fun printExpanded(input: String, supplemental: String, html: Boolean): Validation<String> {
        val totalText = "$input\n\n\n$supplemental"
        val totalTextValidation = parse(totalText)
        val defines =
            when (totalTextValidation) {
                is ValidationFailure -> emptyList()
                is ValidationSuccess -> totalTextValidation.value.defines()
            }
        val states =
            when (totalTextValidation) {
                is ValidationFailure -> emptyList()
                is ValidationSuccess -> totalTextValidation.value.states()
            }
        val foundations =
            when (totalTextValidation) {
                is ValidationFailure -> emptyList()
                is ValidationSuccess -> totalTextValidation.value.foundations()
            }
        val mutuallyGroups =
            when (totalTextValidation) {
                is ValidationFailure -> emptyList()
                is ValidationSuccess -> totalTextValidation.value.mutually()
            }

        val result = StringBuilder()
        val errors = mutableListOf<ParseError>()
        for (part in input.split("\n\n").filter { it.isNotBlank() }) {
            when (val validation = parse(part)
            ) {
                is ValidationFailure -> {
                    errors.addAll(validation.errors)
                }
                is ValidationSuccess -> {
                    result.append(
                        prettyPrint(
                            validation.value, defines, states, foundations, mutuallyGroups, html))
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
        states: List<StatesGroup>,
        foundations: List<FoundationGroup>,
        mutuallyGroups: List<MutuallyGroup>,
        html: Boolean
    ): String {
        val writer =
            if (html) {
                HtmlCodeWriter(
                    defines = defines,
                    states = states,
                    foundations = foundations,
                    mutuallyGroups = mutuallyGroups)
            } else {
                MathLinguaCodeWriter(
                    defines = defines,
                    states = states,
                    foundations = foundations,
                    mutuallyGroups = mutuallyGroups)
            }
        val code = node.toCode(false, 0, writer = writer).getCode()
        return if (html) {
            getHtml(code)
        } else {
            code
        }
    }
}

private fun getHtml(body: String) =
    """
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
                text = text.replace(/([a-zA-Z0-9])\?\??/g, '${'$'}1');
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
                if (node.className && node.className.indexOf('no-render') >= 0) {
                    return;
                }

                let isInWritten = false;
                const parent = node.parentNode;
                if (node.className === 'mathlingua') {
                    for (let i=0; i<node.childNodes.length; i++) {
                        const n = node.childNodes[i];
                        if (n && n.className === 'mathlingua-header' &&
                            n.textContent === 'written:') {
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

            .mathlingua-text-no-render {
                color: #007700;
            }

            .mathlingua-statement-no-render {
                color: #007377;
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

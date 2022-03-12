/*
 * Copyright 2021 The MathLingua Authors
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

package mathlingua.cli

import mathlingua.backend.SourceCollection
import mathlingua.backend.SourceFile
import mathlingua.backend.ValueAndSource
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.ValidationFailure

internal fun bold(text: String) = "\u001B[1m$text\u001B[0m".onWindowsReturn(text)

@Suppress("SAME_PARAMETER_VALUE")
internal fun green(text: String) = "\u001B[32m$text\u001B[0m".onWindowsReturn(text)

internal fun red(text: String) = "\u001B[31m$text\u001B[0m".onWindowsReturn(text)

@Suppress("UNUSED")
internal fun yellow(text: String) = "\u001B[33m$text\u001B[0m".onWindowsReturn(text)

internal val COMPLETIONS =
    Completions(
        items =
            listOf(
                CompletionItem(name = "and", parts = listOf("and:")),
                CompletionItem(name = "exists", parts = listOf("exists:", "where?:", "suchThat?:")),
                CompletionItem(
                    name = "existsUnique",
                    parts = listOf("existsUnique:", "where?:", "suchThat?:")),
                CompletionItem(
                    name = "forAll", parts = listOf("forAll:", "where?:", "suchThat?:", "then:")),
                CompletionItem(name = "if", parts = listOf("if:", "then:")),
                CompletionItem(name = "iff", parts = listOf("iff:", "then:")),
                CompletionItem(name = "not", parts = listOf("not:")),
                CompletionItem(name = "or", parts = listOf("or:")),
                CompletionItem(name = "piecewise", parts = listOf("piecewise:")),
                CompletionItem(name = "generated", parts = listOf("generated:", "from:", "when?:")),
                CompletionItem(
                    name = "Defines:",
                    parts =
                        listOf(
                            "Defines:",
                            "where?",
                            "given?:",
                            "when?:",
                            "means?:",
                            "satisfying:",
                            "expressing:",
                            "providing?:",
                            "using?:",
                            "writing?:",
                            "written:",
                            "called?:",
                            "Metadata?:"),
                ),
                CompletionItem(
                    name = "States",
                    parts =
                        listOf(
                            "States:",
                            "given?:",
                            "when?:",
                            "that:",
                            "using?:",
                            "written:",
                            "called?:",
                            "Metadata?:"),
                ),
                CompletionItem(
                    name = "equality", parts = listOf("equality:", "between:", "provided:")),
                CompletionItem(name = "membership", parts = listOf("membership:", "through:")),
                CompletionItem(name = "view", parts = listOf("view:", "as:", "via:", "by?:")),
                CompletionItem(name = "symbols", parts = listOf("symbols:", "where?:")),
                CompletionItem(name = "memberSymbols", parts = listOf("memberSymbols:", "where?:")),
                CompletionItem(
                    name = "Resource",
                    parts =
                        listOf(
                            "Resource:\n. type? = \"\"\n. name? = \"\"\n. author? = \"\"\n. homepage? = \"\"\n. url? = \"\"\n. offset? = \"\"\nMetadata?:"),
                ),
                CompletionItem(
                    name = "Axiom",
                    parts =
                        listOf(
                            "Axiom:",
                            "given?:",
                            "where?:",
                            "suchThat?:",
                            "then:",
                            "iff?:",
                            "using?:",
                            "Metadata?:"),
                ),
                CompletionItem(
                    name = "Conjecture",
                    parts =
                        listOf(
                            "Conjecture:",
                            "given?:",
                            "where?:",
                            "suchThat?:",
                            "then:",
                            "iff?:",
                            "using?:",
                            "Metadata?:")),
                CompletionItem(
                    name = "Theorem",
                    parts =
                        listOf(
                            "Theorem:",
                            "given?:",
                            "where?:",
                            "suchThat?:",
                            "then:",
                            "iff?:",
                            "using?:",
                            "Proof?:",
                            "Metadata?:"),
                ),
                CompletionItem(name = "Topic", parts = listOf("Topic:", "content:", "Metadata?:")),
                CompletionItem(name = "Note", parts = listOf("Note:", "content:", "Metadata?:")),
                CompletionItem(name = "Specify", parts = listOf("Specify:")),
                CompletionItem(name = "zero", parts = listOf("zero:", "is:")),
                CompletionItem(name = "positiveInt", parts = listOf("positiveInt:", "is:")),
                CompletionItem(name = "negativeInt", parts = listOf("negativeInt:", "is:")),
                CompletionItem(name = "positiveFloat", parts = listOf("positiveFloat:", "is:")),
                CompletionItem(name = "negativeFloat", parts = listOf("negativeFloat:", "is:"))))

internal fun getGitHubUrl(): String? {
    val pro =
        try {
            ProcessBuilder("git", "ls-remote", "--get-url").start()
        } catch (e: Exception) {
            return null
        }
    val exit = pro.waitFor()
    return if (exit != 0) {
        null
    } else {
        String(pro.inputStream.readAllBytes()).replace("git@github.com:", "https://github.com/")
    }
}

internal fun buildSignatureIndex(sourceCollection: SourceCollection): SignatureIndex {
    val entries = mutableListOf<SignatureIndexEntry>()
    for (path in sourceCollection.getAllPaths()) {
        val page = sourceCollection.getPage(path) ?: continue
        for (entity in page.fileResult.entities) {
            val signature = entity.signature ?: continue
            entries.add(
                SignatureIndexEntry(
                    id = entity.id,
                    relativePath = entity.relativePath,
                    signature = signature,
                    called = entity.called))
        }
    }
    return SignatureIndex(entries = entries)
}

internal data class RenderedTopLevelElement(
    val renderedFormHtml: String, val rawFormHtml: String, val node: Phase2Node?)

internal fun getCompleteRenderedTopLevelElements(
    f: VirtualFile,
    sourceCollection: SourceCollection,
    noexpand: Boolean,
    errors: MutableList<ValueAndSource<ParseError>>
): List<RenderedTopLevelElement> {
    val result = mutableListOf<RenderedTopLevelElement>()
    val expandedPair =
        sourceCollection.prettyPrint(file = f, html = true, literal = false, doExpand = !noexpand)
    val literalPair =
        sourceCollection.prettyPrint(file = f, html = true, literal = true, doExpand = false)
    errors.addAll(
        expandedPair.second.map {
            ValueAndSource(
                value = it,
                source =
                    SourceFile(file = f, content = "", validation = ValidationFailure(emptyList())))
        })
    for (i in 0 until expandedPair.first.size) {
        result.add(
            RenderedTopLevelElement(
                renderedFormHtml = fixClassNameBug(expandedPair.first[i].first),
                rawFormHtml = fixClassNameBug(literalPair.first[i].first),
                node = expandedPair.first[i].second))
    }
    return result
}

/*
 * Class names generation has a bug where the class description looks like
 *    class=mathlingua - top - level
 * instead of the correct
 *    class="mathlingua-top-level"
 * The following function finds the incorrect class names and converts them
 * to their correct form.
 */
internal fun fixClassNameBug(html: String) =
    html
        .replace(Regex("class[ ]*=[ ]*([ \\-_a-zA-Z0-9]+)")) {
            // for each `class=..`. found replace ` - ` with `-`
            val next = it.groups[0]?.value?.replace(" - ", "-") ?: it.value
            // then replace `class=...` with `class="..."`
            "class=\"${next.replaceFirst(Regex("class[ ]*=[ ]*"), "")}\""
        }
        .replace("<body>", " ")
        .replace("</body>", " ")

// -----------------------------------------------------------------------------

private fun String.onWindowsReturn(text: String): String =
    if (System.getProperty("os.name").lowercase().contains("win")) {
        text
    } else {
        this
    }

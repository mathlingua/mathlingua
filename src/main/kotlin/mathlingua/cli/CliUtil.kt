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
import mathlingua.backend.ValueSourceTracker
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.validationFailure

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
                CompletionItem(name = "and", value = "and:"),
                CompletionItem(name = "exists", value = "exists:\nsuchThat:"),
                CompletionItem(name = "existsUnique", value = "existsUnique:\nsuchThat:"),
                CompletionItem(name = "forAll", value = "forAll:\nsuchThat?:\nthen:"),
                CompletionItem(name = "if", value = "if:\nthen:"),
                CompletionItem(name = "iff", value = "iff:\nthen:"),
                CompletionItem(name = "not", value = "not:"),
                CompletionItem(name = "or", value = "or:"),
                CompletionItem(name = "piecewise", value = "piecewise:"),
                CompletionItem(name = "generated", value = "generated:\nfrom:\nwhen?:"),
                CompletionItem(
                    name = "Defines:",
                    value =
                        "Defines:\ngiven?:\nwhen?:\nmeans?:\nsatisfying:\nexpressing:\nviewing?:\nusing?:\nwritten:\ncalled?:\nMetadata?:",
                ),
                CompletionItem(
                    name = "States",
                    value =
                        "States:\ngiven?:\nwhen?:\nthat:\nusing?:\nwritten:\ncalled?:\nMetadata?:",
                ),
                CompletionItem(name = "equality", value = "equality:\nbetween:\nprovided:"),
                CompletionItem(name = "membership", value = "membership:\nthrough:"),
                CompletionItem(name = "as", value = "as:\nvia:\nby?:"),
                CompletionItem(
                    name = "Resource",
                    value =
                        "Resource:\n. type? = \"\"\n. name? = \"\"\n. author? = \"\"\n. homepage? = \"\"\n. url? = \"\"\n. offset? = \"\"\nMetadata?:",
                ),
                CompletionItem(
                    name = "Axiom",
                    value = "Axiom:\ngiven?:\nwhen?:\nthen:\niff?:\nusing?:\nMetadata?:",
                ),
                CompletionItem(
                    name = "Conjecture",
                    value = "Conjecture:\ngiven?:\nwhen?:\nthen:\niff?:\nusing?:\nMetadata?:"),
                CompletionItem(
                    name = "Theorem",
                    value = "Theorem:\ngiven?:\nwhen?:\nthen:\niff?:\nusing?:\nProof?:\nMetadata?:",
                ),
                CompletionItem(name = "Topic", value = "Topic:\ncontent:\nMetadata?:"),
                CompletionItem(name = "Note", value = "Note:\ncontent:\nMetadata?:")))

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
    errors: MutableList<ValueSourceTracker<ParseError>>
): List<RenderedTopLevelElement> {
    val result = mutableListOf<RenderedTopLevelElement>()
    val expandedPair =
        sourceCollection.prettyPrint(file = f, html = true, literal = false, doExpand = !noexpand)
    val literalPair =
        sourceCollection.prettyPrint(file = f, html = true, literal = true, doExpand = false)
    errors.addAll(
        expandedPair.second.map {
            ValueSourceTracker(
                value = it,
                source =
                    SourceFile(file = f, content = "", validation = validationFailure(emptyList())),
                tracker = null)
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

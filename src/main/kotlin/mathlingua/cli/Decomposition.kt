/*
 * Copyright 2022 The MathLingua Authors
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
import mathlingua.backend.ValueSourceTracker
import mathlingua.backend.findMathLinguaFiles
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resource.ResourceGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.ConjectureGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topic.TopicGroup
import mathlingua.frontend.chalktalk.phase2.getCalledNames
import mathlingua.frontend.support.ParseError
import mathlingua.md5Hash

internal fun decompose(
    fs: VirtualFileSystem, sourceCollection: SourceCollection, mlgFiles: List<VirtualFile>?
): DecompositionResult {
    val resolvedMlgFiles = mlgFiles ?: findMathLinguaFiles(listOf(fs.cwd()))
    val fileResults = mutableListOf<FileResult>()
    val errors = mutableListOf<ValueSourceTracker<ParseError>>()
    for (i in resolvedMlgFiles.indices) {
        val f = resolvedMlgFiles[i]
        val fErrors = mutableListOf<ValueSourceTracker<ParseError>>()
        val elements =
            getCompleteRenderedTopLevelElements(
                f = f, sourceCollection = sourceCollection, noexpand = false, errors = fErrors)
        errors.addAll(fErrors)
        val relativePath = f.relativePath()
        fileResults.add(
            FileResult(
                relativePath = relativePath,
                previousRelativePath = resolvedMlgFiles.getOrNull(i - 1)?.relativePath(),
                nextRelativePath = resolvedMlgFiles.getOrNull(i + 1)?.relativePath(),
                content = sanitizeHtmlForJs(f.readText()),
                entities =
                    elements.map {
                        EntityResult(
                            id = md5Hash(it.node?.toCode(false, 0)?.getCode() ?: ""),
                            relativePath = relativePath,
                            type = it.node?.javaClass?.simpleName ?: "",
                            signature = getSignature(it.node),
                            rawHtml = it.rawFormHtml,
                            renderedHtml = it.renderedFormHtml,
                            words =
                                if (it.node != null) {
                                    getAllWords(it.node).toList()
                                } else {
                                    emptyList()
                                },
                            called =
                                if (it.node is TopLevelGroup) {
                                    it.node.getCalledNames()
                                } else {
                                    emptyList()
                                })
                    },
                errors =
                    fErrors.filter { it.source.file == f }.map {
                        ErrorResult(
                            relativePath = relativePath,
                            message = it.value.message,
                            row = it.value.row,
                            column = it.value.column)
                    }))
    }
    return DecompositionResult(
        gitHubUrl = getGitHubUrl(),
        signatureIndex = buildSignatureIndex(sourceCollection),
        collectionResult =
            CollectionResult(
                fileResults = fileResults,
                errors =
                    errors.map {
                        ErrorResult(
                            relativePath = it.source.file.relativePath(),
                            message = it.value.message,
                            row = it.value.row,
                            column = it.value.column)
                    }))
}

// -----------------------------------------------------------------------------

private fun getSignature(node: Phase2Node?): String? {
    return when (node) {
        null -> {
            null
        }
        is DefinesGroup -> {
            node.signature?.form
        }
        is StatesGroup -> {
            node.signature?.form
        }
        is TheoremGroup -> {
            node.signature?.form
        }
        is AxiomGroup -> {
            node.signature?.form
        }
        is ConjectureGroup -> {
            node.signature?.form
        }
        is TopicGroup -> {
            node.id?.trim()?.removeSurrounding("[", "]")?.trim()
        }
        is ResourceGroup -> {
            node.id.trim().removeSurrounding("[", "]").trim()
        }
        else -> {
            null
        }
    }
}

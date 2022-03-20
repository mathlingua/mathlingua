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

import java.io.File
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import mathlingua.backend.BackEnd
import mathlingua.backend.SourceCollection
import mathlingua.backend.SourceFile
import mathlingua.backend.ValueAndSource
import mathlingua.backend.buildSourceFile
import mathlingua.backend.findMathLinguaFiles
import mathlingua.backend.newSourceCollection
import mathlingua.backend.newSourceCollectionFromCwd
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelBlockComment
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.ValidationFailure
import mathlingua.getRandomUuid

const val MATHLINGUA_VERSION = "0.15.1"

object Mathlingua {
    fun check(fs: VirtualFileSystem, logger: Logger, files: List<VirtualFile>, json: Boolean): Int {
        val sourceCollection =
            newSourceCollection(fs, files.ifEmpty { listOf(fs.getDirectory(listOf("content"))) })
        val errors = BackEnd.check(sourceCollection)
        logger.log(getErrorOutput(errors, sourceCollection.size(), json))
        return if (errors.isEmpty()) {
            0
        } else {
            1
        }
    }

    private fun export(fs: VirtualFileSystem, logger: Logger): List<ValueAndSource<ParseError>> {
        val files = findMathLinguaFiles(listOf(getContentDirectory(fs)))

        val errors = mutableListOf<ValueAndSource<ParseError>>()
        for (target in files) {
            errors.addAll(
                exportFile(
                    fs = fs,
                    logger = logger,
                    target = target,
                    stdout = false,
                    noExpand = false,
                    raw = false))
        }

        return errors
    }

    fun render(fs: VirtualFileSystem, logger: Logger): Int {
        val result = renderAll(fs = fs, logger = logger)
        val files = result.first
        val errors = mutableListOf<ErrorResult>()
        errors.addAll(result.second)
        errors.addAll(
            export(fs = fs, logger = logger).map {
                ErrorResult(
                    relativePath = it.source.file.relativePath(),
                    message = it.value.message,
                    row = it.value.row,
                    column = it.value.column)
            })
        logger.log(
            getErrorOutput(
                errors.map {
                    ValueAndSource(
                        value = ParseError(message = it.message, row = it.row, column = it.column),
                        source = fs.getFileOrDirectory(it.relativePath).buildSourceFile())
                },
                files.size,
                false))
        return if (errors.isEmpty()) {
            0
        } else {
            1
        }
    }

    fun clean(fs: VirtualFileSystem, logger: Logger): Int {
        val docsDir = getDocsDirectory(fs)
        val result =
            if (!docsDir.exists()) {
                logger.log("Nothing to clean")
                0
            } else {
                val deletedDocs = docsDir.delete()
                if (deletedDocs) {
                    logger.log("Cleaned the 'docs' directory")
                    0
                } else {
                    logger.log("${bold(red("ERROR: "))} Failed to clean the 'docs' directory")
                    1
                }
            }
        docsDir.mkdirs()
        return result
    }

    fun version(logger: Logger): Int {
        logger.log("MathLingua $MATHLINGUA_VERSION")
        return 0
    }

    fun serve(fs: VirtualFileSystem, logger: Logger, port: Int, onStart: (() -> Unit)?) =
        startServer(fs, logger, port, onStart)

    fun decompose(fs: VirtualFileSystem, logger: Logger) {
        logger.log(
            Json.encodeToString(
                decompose(
                    fs = fs, sourceCollection = newSourceCollectionFromCwd(fs), mlgFiles = null)))
    }

    fun completionJson(logger: Logger) {
        logger.log(Json.encodeToString(COMPLETIONS))
    }
}

internal fun getDocsDirectory(fs: VirtualFileSystem) = fs.getDirectory(listOf("docs"))

// -----------------------------------------------------------------------------

private fun String.jsonSanitize() =
    this.replace("\\", "\\\\")
        .replace("\b", "\\b")
        .replace("\n", "\\n")
        .replace("\t", "\\t")
        .replace("\r", "\\r")
        .replace("\"", "\\\"")

private fun maybePlural(text: String, count: Int) =
    if (count == 1) {
        text
    } else {
        "${text}s"
    }

private fun getContentDirectory(fs: VirtualFileSystem) = fs.getDirectory(listOf("content"))

private fun getErrorOutput(
    errors: List<ValueAndSource<ParseError>>, numFilesProcessed: Int, json: Boolean
): String {
    val builder = StringBuilder()
    if (json) {
        builder.append("[")
    }
    for (i in errors.indices) {
        val err = errors[i]
        if (json) {
            builder.append("{")
            builder.append("  \"file\": \"${err.source.file.absolutePath().jsonSanitize()}\",")
            builder.append("  \"type\": \"ERROR\",")
            builder.append("  \"message\": \"${err.value.message.jsonSanitize()}\",")
            builder.append("  \"failedLine\": \"\",")
            builder.append("  \"row\": ${err.value.row},")
            builder.append("  \"column\": ${err.value.column}")
            builder.append("}")
            if (i != errors.size - 1) {
                builder.append(",")
            }
        } else {
            builder.append(bold(red("ERROR: ")))
            builder.append(
                bold(
                    "${err.source.file.relativePath()} (Line: ${err.value.row + 1}, Column: ${err.value.column + 1})\n"))
            builder.append(err.value.message.trim())
            builder.append("\n\n")
        }
    }

    if (json) {
        builder.append("]")
    } else {
        builder.append(
            if (errors.isEmpty()) {
                bold(green("SUCCESS\n"))
            } else {
                bold(red("FAILED\n"))
            })
        builder.append("Processed $numFilesProcessed ${maybePlural("file", numFilesProcessed)}\n")
        builder.append("${errors.size} ${maybePlural("error", errors.size)} detected")
    }

    return builder.toString()
}

private fun exportFile(
    fs: VirtualFileSystem,
    logger: Logger,
    target: VirtualFile,
    stdout: Boolean,
    noExpand: Boolean,
    raw: Boolean
): List<ValueAndSource<ParseError>> {
    if (!target.exists()) {
        val message = "ERROR: The file ${target.absolutePath()} does not exist"
        logger.log(message)
        return listOf(
            ValueAndSource(
                value = ParseError(message = message, row = -1, column = -1),
                source =
                    SourceFile(
                        file = target, content = "", validation = ValidationFailure(emptyList()))))
    }

    if (target.isDirectory() || !target.absolutePath().endsWith(".math")) {
        val message = "ERROR: The path ${target.absolutePath()} is not a .math file"
        logger.log(message)
        return listOf(
            ValueAndSource(
                value = ParseError(message = message, row = -1, column = -1),
                source =
                    SourceFile(
                        file = target, content = "", validation = ValidationFailure(emptyList()))))
    }

    val sourceCollection = newSourceCollection(fs, listOf(fs.cwd()))
    val errors = mutableListOf<ValueAndSource<ParseError>>()
    val elements = getUnifiedRenderedTopLevelElements(target, sourceCollection, noExpand, errors)

    val contentBuilder = StringBuilder()
    for (element in elements) {
        if (element.second != null && element.second is TopLevelBlockComment) {
            contentBuilder.append("<div class='mathlingua-block-comment-top-level'>")
            contentBuilder.append(element.first)
            contentBuilder.append("</div>")
        } else {
            contentBuilder.append(element.first)
        }
    }

    val text =
        if (raw) {
            contentBuilder.toString()
        } else {
            buildStandaloneHtml(content = contentBuilder.toString())
        }

    if (stdout) {
        logger.log(text)
    } else {
        // get the path relative to the current working directory with
        // the file extension replaced with ".html"
        val relHtmlPath = target.relativePath().split("/").toMutableList()
        if (relHtmlPath.size > 0) {
            relHtmlPath[relHtmlPath.size - 1] =
                relHtmlPath[relHtmlPath.size - 1].replace(".math", ".html")
        }
        val htmlPath = mutableListOf<String>()
        htmlPath.add("docs")
        htmlPath.addAll(relHtmlPath)
        val outFile = fs.getFile(htmlPath)
        val parentDir =
            fs.getDirectory(htmlPath.filterIndexed { index, _ -> index < htmlPath.size - 1 })
        parentDir.mkdirs()
        outFile.writeText(text)
        logger.log("Wrote ${outFile.relativePath().split("/").joinToString(File.separator)}")
    }

    return errors
}

private fun getUnifiedRenderedTopLevelElements(
    f: VirtualFile,
    sourceCollection: SourceCollection,
    noexpand: Boolean,
    errors: MutableList<ValueAndSource<ParseError>>
): List<Pair<String, Phase2Node?>> {
    val codeElements = mutableListOf<Pair<String, Phase2Node?>>()
    val elements = getCompleteRenderedTopLevelElements(f, sourceCollection, noexpand, errors)
    for (element in elements) {
        val expanded = element.renderedFormHtml
        val node = element.node
        if (node != null && node is TopLevelBlockComment) {
            codeElements.add(Pair(expanded, node))
        } else {
            val literal = element.rawFormHtml
            val id = getRandomUuid()
            val html =
                "<div><button class='mathlingua-flip-icon' onclick=\"flipEntity('$id')\">&#8226;</button><div id='rendered-$id' class='mathlingua-rendered-visible'>${expanded}</div>" +
                    "<div id='literal-$id' class='mathlingua-literal-hidden'>${literal}</div></div>"
            codeElements.add(Pair(html, node))
        }
    }
    return codeElements
}

private fun renderAll(
    fs: VirtualFileSystem, logger: Logger
): Pair<List<String>, List<ErrorResult>> {
    val docDir = ensureDocsDirExists(fs, logger)

    val decomp =
        decompose(fs = fs, sourceCollection = newSourceCollectionFromCwd(fs = fs), mlgFiles = null)
    val data = Json.encodeToString(decomp)

    val errors = mutableListOf<ErrorResult>()

    val docContentDir = File(docDir, "content")
    val contentDir = File("content")
    contentDir.copyRecursively(docContentDir, overwrite = true) { file, ioException ->
        errors.add(
            ErrorResult(
                relativePath = file.toRelativeString(contentDir),
                message =
                    "Failed to copy ${file.toRelativeString(contentDir)} to the docs directory: ${ioException.message}",
                row = 0,
                column = 0))
        OnErrorAction.SKIP
    }

    val dataFile = File(docDir, "data.js")
    dataFile.writeText("window.MATHLINGUA_DATA = $data")
    logger.log("Wrote docs${File.separator}data.js")

    errors.addAll(decomp.collectionResult.errors)
    return Pair(decomp.collectionResult.fileResults.map { it.relativePath }, errors)
}

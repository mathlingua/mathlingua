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

import io.javalin.Javalin
import java.io.File
import java.nio.file.Files
import java.nio.file.Paths
import java.util.jar.JarFile
import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import mathlingua.backend.BackEnd
import mathlingua.backend.EntityResult
import mathlingua.backend.FileResult
import mathlingua.backend.SourceCollection
import mathlingua.backend.SourceFile
import mathlingua.backend.ValueSourceTracker
import mathlingua.backend.buildSourceFile
import mathlingua.backend.findMathLinguaFiles
import mathlingua.backend.fixClassNameBug
import mathlingua.backend.newSourceCollection
import mathlingua.frontend.chalktalk.phase2.ast.clause.Identifier
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Text
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelBlockComment
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resource.ResourceGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.ConjectureGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.StringItem
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.ContentItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.NameItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.OffsetItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.PageItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.SiteItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.StringSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topic.TopicGroup
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.support.validationFailure
import mathlingua.frontend.textalk.TexTalkNode
import mathlingua.frontend.textalk.TextTexTalkNode
import mathlingua.getRandomUuid
import mathlingua.md5Hash

const val MATHLINGUA_VERSION = "0.10.0"

private fun bold(text: String) = "\u001B[1m$text\u001B[0m"

@Suppress("SAME_PARAMETER_VALUE")
private fun green(text: String) = "\u001B[32m$text\u001B[0m"

private fun red(text: String) = "\u001B[31m$text\u001B[0m"

@Suppress("UNUSED")
private fun yellow(text: String) = "\u001B[33m$text\u001B[0m"

object Mathlingua {
    fun check(fs: VirtualFileSystem, logger: Logger, files: List<VirtualFile>, json: Boolean): Int {
        val sourceCollection =
            newSourceCollection(fs, files.ifEmpty { listOf(fs.getDirectory(listOf("content"))) })
        val errors = BackEnd.check(sourceCollection)
        logger.log(getErrorOutput(fs, errors, sourceCollection.size(), json))
        return if (errors.isEmpty()) {
            0
        } else {
            1
        }
    }

    private fun export(
        fs: VirtualFileSystem, logger: Logger
    ): List<ValueSourceTracker<ParseError>> {
        val files = findMathLinguaFiles(listOf(getContentDirectory(fs)))

        val errors = mutableListOf<ValueSourceTracker<ParseError>>()
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
        val cwd = fs.cwd()
        val sep = fs.getFileSeparator()
        errors.addAll(
            export(fs = fs, logger = logger).map {
                ErrorResult(
                    relativePath = it.source.file.relativePathTo(cwd).joinToString(sep),
                    message = it.value.message,
                    row = it.value.row,
                    column = it.value.column)
            })
        logger.log(
            getErrorOutput(
                fs,
                errors.map {
                    ValueSourceTracker(
                        value = ParseError(message = it.message, row = it.row, column = it.column),
                        source = buildSourceFile(fs.getFileOrDirectory(it.relativePath)),
                        tracker = null,
                    )
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

    fun serve(fs: VirtualFileSystem, logger: Logger, port: Int) {
        logger.log("Visit localhost:$port to see your rendered MathLingua code.")
        logger.log("Every time you refresh the page, your MathLingua code will be re-analyzed.")

        var sourceCollection: SourceCollection? = null
        fun getSourceCollection(): SourceCollection {
            if (sourceCollection == null) {
                sourceCollection = buildSourceCollection(fs = fs)
            }
            return sourceCollection!!
        }

        val app = Javalin.create().start(port)
        app.config.addStaticFiles("/assets")
        app.before("/") { ctx ->
            try {
                logger.log("Re-analyzing the MathLingua code.")
                // invalidate the source collection and regenerate it
                sourceCollection = null
                getSourceCollection()
            } catch (err: Exception) {
                err.printStackTrace()
                ctx.status(500)
            }
        }
        app
            .routes {}
            .put("/api/writePage") { ctx ->
                try {
                    val pathAndContent = ctx.bodyAsClass(WritePageRequest::class.java)
                    logger.log("Writing page ${pathAndContent.path}")
                    val file = fs.getFileOrDirectory(pathAndContent.path)
                    file.writeText(pathAndContent.content)
                    val newSource = buildSourceFile(file)
                    getSourceCollection().removeSource(pathAndContent.path)
                    getSourceCollection().addSource(newSource)
                    ctx.status(200)
                } catch (err: Exception) {
                    err.printStackTrace()
                    ctx.status(500)
                }
            }
            .get("/api/readPage") { ctx ->
                try {
                    val path = ctx.queryParam("path", null)
                    logger.log("Reading page $path")
                    if (path == null) {
                        ctx.status(400)
                    } else {
                        val file = fs.getFileOrDirectory(path)
                        val content = file.readText()
                        ctx.json(ReadPageResponse(content = content))
                    }
                } catch (err: Exception) {
                    err.printStackTrace()
                    ctx.status(500)
                }
            }
            .get("/api/fileResult") { ctx ->
                try {
                    val path = ctx.queryParam("path", null)
                    logger.log("Getting file result for $path")
                    if (path == null) {
                        ctx.status(400)
                    } else {
                        val page = getSourceCollection().getPage(path)
                        if (page == null) {
                            ctx.status(404)
                        } else {
                            ctx.json(page.fileResult)
                        }
                    }
                } catch (err: Exception) {
                    err.printStackTrace()
                    ctx.status(500)
                }
            }
            .post("/api/deleteDir") { ctx ->
                try {
                    val data = ctx.bodyAsClass(DeleteDirRequest::class.java)
                    logger.log("Deleting directory ${data.path}")
                    Paths.get(data.path).toFile().deleteRecursively()
                    sourceCollection = null
                    getSourceCollection()
                    ctx.status(200)
                } catch (err: Exception) {
                    err.printStackTrace()
                    ctx.status(500)
                }
            }
            .post("/api/deleteFile") { ctx ->
                try {
                    val data = ctx.bodyAsClass(DeleteFileRequest::class.java)
                    logger.log("Deleting file ${data.path}")
                    Paths.get(data.path).toFile().delete()
                    sourceCollection = null
                    getSourceCollection()
                    ctx.status(200)
                } catch (err: Exception) {
                    err.printStackTrace()
                    ctx.status(500)
                }
            }
            .post("/api/renameDir") { ctx ->
                try {
                    val data = ctx.bodyAsClass(RenameDirRequest::class.java)
                    logger.log("Renaming directory ${data.fromPath} to ${data.toPath}")
                    val from = Paths.get(data.fromPath)
                    val to = Paths.get(data.toPath)
                    Files.move(from, to)
                    sourceCollection = null
                    getSourceCollection()
                    ctx.status(200)
                } catch (err: Exception) {
                    err.printStackTrace()
                    ctx.status(500)
                }
            }
            .post("/api/renameFile") { ctx ->
                try {
                    val data = ctx.bodyAsClass(RenameFileRequest::class.java)
                    logger.log("Renaming file ${data.fromPath} to ${data.toPath}")
                    val from = Paths.get(data.fromPath)
                    val to = Paths.get(data.toPath)
                    Files.move(from, to)
                    sourceCollection = null
                    getSourceCollection()
                    ctx.status(200)
                } catch (err: Exception) {
                    err.printStackTrace()
                    ctx.status(500)
                }
            }
            .post("/api/newDir") { ctx ->
                try {
                    val data = ctx.bodyAsClass(NewDirRequest::class.java)
                    logger.log("Creating new directory ${data.path}")
                    val dir = Paths.get(data.path).toFile()
                    dir.mkdirs()
                    val newFile = File(dir, "Untitled.math")
                    newFile.writeText("::\n::")
                    sourceCollection = null
                    getSourceCollection()
                    ctx.status(200)
                } catch (err: Exception) {
                    err.printStackTrace()
                    ctx.status(500)
                }
            }
            .post("/api/newFile") { ctx ->
                try {
                    val data = ctx.bodyAsClass(NewFileRequest::class.java)
                    logger.log("Creating new file ${data.path}")
                    Paths.get(data.path).toFile().writeText("::\n::")
                    sourceCollection = null
                    getSourceCollection()
                    ctx.status(200)
                } catch (err: Exception) {
                    err.printStackTrace()
                    ctx.status(500)
                }
            }
            .get("/api/check") { ctx ->
                try {
                    logger.log("Checking")
                    ctx.json(
                        CheckResponse(
                            errors =
                                BackEnd.check(getSourceCollection()).map {
                                    CheckError(
                                        path =
                                            it.source
                                                .file
                                                .relativePathTo(fs.cwd())
                                                .joinToString(fs.getFileSeparator()),
                                        message = it.value.message,
                                        row = it.value.row,
                                        column = it.value.column)
                                }))
                } catch (err: Exception) {
                    err.printStackTrace()
                    ctx.status(500)
                }
            }
            .get("/api/allPaths") { ctx ->
                try {
                    logger.log("Getting all paths")
                    ctx.json(AllPathsResponse(paths = getSourceCollection().getAllPaths()))
                } catch (err: Exception) {
                    err.printStackTrace()
                    ctx.status(500)
                }
            }
            .get("/api/withSignature") { ctx ->
                try {
                    val signature = ctx.queryParam("signature", null)
                    logger.log("Getting entity with signature '${signature}'")
                    if (signature == null) {
                        ctx.status(400)
                    } else {
                        val entityResult = getSourceCollection().getWithSignature(signature)
                        if (entityResult == null) {
                            ctx.status(404)
                        } else {
                            ctx.json(entityResult)
                        }
                    }
                } catch (err: Exception) {
                    err.printStackTrace()
                    ctx.status(500)
                }
            }
            .get("/api/search") { ctx ->
                try {
                    val query = ctx.queryParam("query") ?: ""
                    logger.log("Searching with query '$query'")
                    ctx.json(
                        SearchResponse(
                            paths =
                                getSourceCollection().search(query).map {
                                    it.file
                                        .relativePathTo(fs.cwd())
                                        .joinToString(fs.getFileSeparator())
                                }))
                } catch (err: Exception) {
                    err.printStackTrace()
                    ctx.status(500)
                }
            }
            .get("/api/completeWord") { ctx ->
                try {
                    val word = ctx.queryParam("word") ?: ""
                    logger.log("Getting completions for word '$word'")
                    val suffixes = getSourceCollection().findWordSuffixesFor(word)
                    ctx.json(CompleteWordResponse(suffixes = suffixes))
                } catch (err: Exception) {
                    err.printStackTrace()
                    ctx.status(500)
                }
            }
            .get("/api/completeSignature") { ctx ->
                try {
                    val prefix = ctx.queryParam("prefix") ?: ""
                    logger.log("Getting signature completions for prefix '$prefix'")
                    val suffixes = getSourceCollection().findSignaturesSuffixesFor(prefix)
                    ctx.json(CompleteSignatureResponse(suffixes = suffixes))
                } catch (err: Exception) {
                    err.printStackTrace()
                    ctx.status(500)
                }
            }
            .get("/api/*") { ctx -> ctx.status(400) }
    }

    fun decompose(fs: VirtualFileSystem, logger: Logger) {
        logger.log(
            Json.encodeToString(
                decompose(fs = fs, sourceCollection = buildSourceCollection(fs), mlgFiles = null)))
    }
}

@Serializable data class DeleteDirRequest(val path: String)

@Serializable data class DeleteFileRequest(val path: String)

@Serializable data class RenameDirRequest(val fromPath: String, val toPath: String)

@Serializable data class RenameFileRequest(val fromPath: String, val toPath: String)

@Serializable data class NewDirRequest(val path: String)

@Serializable data class NewFileRequest(val path: String)

@Serializable data class ReadPageRequest(val path: String)

@Serializable data class ReadPageResponse(val content: String)

@Serializable data class WritePageRequest(val path: String, val content: String)

@Serializable data class CheckResponse(val errors: List<CheckError>)

@Serializable
data class CheckError(val path: String, val message: String, val row: Int, val column: Int)

@Serializable data class AllPathsResponse(val paths: List<String>)

@Serializable data class HomeResponse(val homeHtml: String)

@Serializable data class SearchResponse(val paths: List<String>)

@Serializable data class CompleteWordResponse(val suffixes: List<String>)

@Serializable data class CompleteSignatureResponse(val suffixes: List<String>)

private fun String.jsonSanitize() =
    this.replace("\\", "\\\\")
        .replace("\b", "\\b")
        .replace("\n", "\\n")
        .replace("\t", "\\t")
        .replace("\r", "\\r")
        .replace("\"", "\\\"")

private fun sanitizeHtmlForJs(html: String) =
    html.replace("\\", "\\\\").replace("\n", "\\n").replace("\"", "\\\"")

private fun maybePlural(text: String, count: Int) =
    if (count == 1) {
        text
    } else {
        "${text}s"
    }

private fun getDocsDirectory(fs: VirtualFileSystem) = fs.getDirectory(listOf("docs"))

private fun getExportedDirectory(fs: VirtualFileSystem) = fs.getDirectory(listOf("exported"))

private fun getContentDirectory(fs: VirtualFileSystem) = fs.getDirectory(listOf("content"))

private fun getErrorOutput(
    fs: VirtualFileSystem,
    errors: List<ValueSourceTracker<ParseError>>,
    numFilesProcessed: Int,
    json: Boolean
): String {
    val builder = StringBuilder()
    if (json) {
        builder.append("[")
    }
    val cwd = fs.cwd()
    for (i in errors.indices) {
        val err = errors[i]
        if (json) {
            builder.append("{")
            builder.append(
                "  \"file\": \"${err.source.file.absolutePath().joinToString(fs.getFileSeparator()).jsonSanitize()}\",")
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
                    "${err.source.file.relativePathTo(cwd).joinToString(fs.getFileSeparator())} (Line: ${err.value.row + 1}, Column: ${err.value.column + 1})\n"))
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
): List<ValueSourceTracker<ParseError>> {
    if (!target.exists()) {
        val message =
            "ERROR: The file ${target.absolutePath().joinToString(fs.getFileSeparator())} does not exist"
        logger.log(message)
        return listOf(
            ValueSourceTracker(
                value = ParseError(message = message, row = -1, column = -1),
                source =
                    SourceFile(
                        file = target, content = "", validation = validationFailure(emptyList())),
                tracker = null))
    }

    if (target.isDirectory() || !target.absolutePath().last().endsWith(".math")) {
        val message =
            "ERROR: The path ${target.absolutePath().joinToString(fs.getFileSeparator())} is not a .math file"
        logger.log(message)
        return listOf(
            ValueSourceTracker(
                value = ParseError(message = message, row = -1, column = -1),
                source =
                    SourceFile(
                        file = target, content = "", validation = validationFailure(emptyList())),
                tracker = null))
    }

    val sourceCollection = newSourceCollection(fs, listOf(fs.cwd()))
    val errors = mutableListOf<ValueSourceTracker<ParseError>>()
    val elements = getUnifiedRenderedTopLevelElements(target, sourceCollection, noExpand, errors)

    val contentBuilder = StringBuilder()
    for (element in elements) {
        if (element.second != null && element.second is TopLevelBlockComment) {
            contentBuilder.append("<div class='mathlingua-block-comment-top-level'>")
            contentBuilder.append(element.first)
            contentBuilder.append("</div>")
        } else {
            contentBuilder.append("<div class='mathlingua-top-level'>")
            contentBuilder.append(element.first)
            contentBuilder.append("</div>")
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
        val relHtmlPath = target.relativePathTo(fs.cwd()).toMutableList()
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
        logger.log("Wrote ${outFile.relativePathTo(fs.cwd()).joinToString(fs.getFileSeparator())}")
    }

    return errors
}

private fun renderAll(
    fs: VirtualFileSystem, logger: Logger
): Pair<List<String>, List<ErrorResult>> {
    val uri =
        Mathlingua.javaClass.getResource("/assets")?.toURI()?.toString()?.trim()
            ?: throw Exception("Failed to load assets directory")
    val index = uri.indexOf('!')
    val uriPrefix =
        if (index < 0) {
            uri
        } else {
            uri.substring(0, index)
        }
    val jarPath = uriPrefix.replace("jar:file:", "")

    val docDir = File(getDocsDirectory(fs).absolutePath().joinToString(fs.getFileSeparator()))
    docDir.mkdirs()

    val cnameFile = File("CNAME")
    if (cnameFile.exists()) {
        val docsCnameFile = File(docDir, "CNAME")
        cnameFile.copyTo(target = docsCnameFile, overwrite = true)
    }

    val jarTimestamp = File(jarPath).lastModified().toString()
    val timestampFile = File(docDir, "timestamp")
    if (!timestampFile.exists() || timestampFile.readText() != jarTimestamp) {
        logger.log("Initial run detected. Saving webapp files to speed up future runs.")
        timestampFile.writeText(jarTimestamp)

        val jar = JarFile(jarPath)
        for (entry in jar.entries()) {
            if (!entry.toString().startsWith("assets/") || entry.toString() == "assets/") {
                continue
            }

            val outFile = File(docDir, entry.name.replace("assets/", ""))
            if (entry.isDirectory) {
                outFile.mkdirs()
            } else {
                outFile.writeBytes(jar.getInputStream(entry).readAllBytes())
            }
        }
        jar.close()

        val indexFile = File(docDir, "index.html")
        val indexText =
            indexFile.readText().replace("<head>", "<head><script src=\"./data.js\"></script>")
        indexFile.writeText(indexText)
        logger.log("Wrote docs/index.html")
    }

    val decomp =
        decompose(fs = fs, sourceCollection = buildSourceCollection(fs = fs), mlgFiles = null)
    val data = Json.encodeToString(decomp)

    val dataFile = File(docDir, "data.js")
    dataFile.writeText("window.MATHLINGUA_DATA = $data")
    logger.log("Wrote docs/data.js")

    return Pair(
        decomp.collectionResult.fileResults.map { it.relativePath }, decomp.collectionResult.errors)
}

class LockedValue<T> {
    private var data: T? = null

    fun setValue(value: T) {
        if (data == null) {
            data = value!!
        }
    }

    fun getValue(): T = data!!

    fun getValueOrDefault(default: T) =
        if (data == null) {
            default
        } else {
            getValue()
        }
}

private data class RenderedTopLevelElement(
    val renderedFormHtml: String, val rawFormHtml: String, val node: Phase2Node?)

private fun getCompleteRenderedTopLevelElements(
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

@Serializable
data class ErrorResult(
    val relativePath: String, val message: String, val row: Int, val column: Int)

@Serializable
data class CollectionResult(val fileResults: List<FileResult>, val errors: List<ErrorResult>)

@Serializable data class DecompositionResult(val collectionResult: CollectionResult)

private fun getSignature(node: Phase2Node?): String? {
    return when (node) {
        null -> {
            null
        }
        is DefinesGroup -> {
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
        else -> {
            null
        }
    }
}

private fun buildSourceCollection(fs: VirtualFileSystem) = newSourceCollection(fs, listOf(fs.cwd()))

private fun decompose(
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
        val relativePath = f.relativePathTo(fs.cwd()).joinToString(fs.getFileSeparator())
        fileResults.add(
            FileResult(
                relativePath = relativePath,
                previousRelativePath =
                    resolvedMlgFiles
                        .getOrNull(i - 1)
                        ?.relativePathTo(fs.cwd())
                        ?.joinToString(fs.getFileSeparator()),
                nextRelativePath =
                    resolvedMlgFiles
                        .getOrNull(i + 1)
                        ?.relativePathTo(fs.cwd())
                        ?.joinToString(fs.getFileSeparator()),
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
        collectionResult =
            CollectionResult(
                fileResults = fileResults,
                errors =
                    errors.map {
                        ErrorResult(
                            relativePath =
                                it.source
                                    .file
                                    .relativePathTo(fs.cwd())
                                    .joinToString(fs.getFileSeparator()),
                            message = it.value.message,
                            row = it.value.row,
                            column = it.value.column)
                    }))
}

private fun getUnifiedRenderedTopLevelElements(
    f: VirtualFile,
    sourceCollection: SourceCollection,
    noexpand: Boolean,
    errors: MutableList<ValueSourceTracker<ParseError>>
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
                "<div><button class='mathlingua-flip-icon' onclick=\"flipEntity('$id')\">" +
                    "&#8226;</button><div id='rendered-$id' class='mathlingua-rendered-visible'>${expanded}</div>" +
                    "<div id='literal-$id' class='mathlingua-literal-hidden'>${literal}</div></div>"
            codeElements.add(Pair(html, node))
        }
    }
    return codeElements
}

fun getAllWords(node: Phase2Node): Set<String> {
    val result = mutableSetOf<String>()
    getAllWordsImpl(node, result)
    return result
}

private fun getAllWordsImpl(node: Phase2Node, words: MutableSet<String>) {
    when (node) {
        is ResourceGroup -> {
            // searching for a reference with or without @ in front
            // should find the associated reference group
            words.add(node.id)
            words.add("@node.id")
        }
        is DefinesGroup -> {
            if (node.signature != null) {
                words.add(node.signature!!.form)
            }
            when (val validation = node.id.texTalkRoot
            ) {
                is ValidationSuccess -> {
                    getAllWordsImpl(validation.value, words)
                }
            }
        }
        is StringItem -> {
            getAllWordsImpl(node.text, words)
        }
        is StringSection -> {
            getAllWordsImpl(node.name, words)
            for (value in node.values) {
                getAllWordsImpl(value, words)
            }
        }
        is ContentItemSection -> {
            getAllWordsImpl(node.content, words)
        }
        is NameItemSection -> {
            getAllWordsImpl(node.name, words)
        }
        is OffsetItemSection -> {
            getAllWordsImpl(node.offset, words)
        }
        is PageItemSection -> {
            getAllWordsImpl(node.page, words)
        }
        is SiteItemSection -> {
            getAllWordsImpl(node.url, words)
        }
        is Statement -> {
            val root = node.texTalkRoot
            if (root is ValidationSuccess) {
                getAllWordsImpl(root.value, words)
            }
        }
        is Identifier -> {
            words.add(node.name.lowercase())
        }
        is Text -> {
            getAllWordsImpl(node.text, words)
        }
        is TopLevelBlockComment -> {
            getAllWordsImpl(node.blockComment.text.removeSurrounding("::", "::"), words)
        }
        is TopicGroup -> {
            getAllWordsImpl(node.contentSection.text, words)
            if (node.id != null) {
                getAllWordsImpl(node.id, words)
            }
            for (name in node.topicSection.names) {
                getAllWordsImpl(name, words)
            }
        }
    }

    node.forEach { getAllWordsImpl(it, words) }
}

private fun getAllWordsImpl(node: TexTalkNode, words: MutableSet<String>) {
    when (node) {
        is TextTexTalkNode -> {
            getAllWordsImpl(node.text, words)
        }
    }

    node.forEach { getAllWordsImpl(it, words) }
}

private fun getAllWordsImpl(text: String, words: MutableSet<String>) {
    val builder = StringBuilder()
    for (c in text) {
        if (c.isLetterOrDigit()) {
            builder.append(c)
        } else {
            builder.append(' ')
        }
    }

    words.addAll(
        builder
            .toString()
            .replace("\r", " ")
            .replace("\n", " ")
            .split(" ")
            .map { it.trim() }
            .filter { it.isNotEmpty() }
            .map { sanitizeHtmlForJs(it.lowercase()) })
}

const val SHARED_HEADER =
    """
    <meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1">
    <meta content="text/html;charset=utf-8" http-equiv="Content-Type">
    <meta content="utf-8" http-equiv="encoding">
    <meta name="description" content="A codex of mathematical knowledge">
    <meta name="keywords" content="math, maths, mathematics, knowledge, database, repository, codex, encyclopedia">
    <link rel="stylesheet"
          href="https://cdn.jsdelivr.net/npm/katex@0.11.1/dist/katex.min.css"
          integrity="sha384-zB1R0rpPzHqg7Kpt0Aljp8JPLqbXI3bhnPWROx27a9N0Ll6ZP/+DiW/UqRcLbRjq"
          crossorigin="anonymous">
    <script defer
            src="https://cdn.jsdelivr.net/npm/katex@0.11.1/dist/katex.min.js"
            integrity="sha384-y23I5Q6l+B6vatafAwxRu/0oK/79VlbSz7Q9aiSZUvyWYIYsd+qj+o24G5ZU2zJz"
            crossorigin="anonymous">
    </script>
    <script defer
            src="https://cdnjs.cloudflare.com/ajax/libs/mark.js/8.11.1/mark.min.js">
    </script>
"""

const val KATEX_RENDERING_JS =
    """
    function buildMathFragment(rawText) {
        let text = rawText;
        if (text[0] === '"') {
            text = text.substring(1);
        }
        if (text[text.length - 1] === '"') {
            text = text.substring(0, text.length - 1);
        }
        text = text.replace(/([a-zA-Z0-9])\?\??/g, '${'$'}1');
        const fragment = document.createDocumentFragment();
        let buffer = '';
        let i = 0;
        while (i < text.length) {
            if (text[i] === '${'$'}' && text[i+1] === '${'$'}' && text[i+2] === '${'$'}') {
                i += 3; // skip over the ${'$'}s
                fragment.appendChild(document.createTextNode(buffer));
                buffer = '';

                const span = document.createElement('span');
                let math = '';
                while (i < text.length &&
                    !(text[i] === '${'$'}' && text[i+1] === '${'$'}' && text[i+2] === '${'$'}')) {
                    math += text[i++];
                }
                if (text[i] === '${'$'}') {
                    i++; // move past the ${'$'}
                }
                if (text[i] === '${'$'}') {
                    i++; // move past the second ${'$'}
                }
                if (text[i] === '${'$'}') {
                    i++; // move past the third ${'$'}
                }
                try {
                    katex.render(math, span, {
                        throwOnError: true,
                        displayMode: false
                    });
                } catch {
                    span.appendChild(document.createTextNode(math));
                }
                fragment.appendChild(span);
            } else if (text[i] === '\\' && text[i+1] === '[') {
                i += 2; // skip over \ and [
                fragment.appendChild(document.createTextNode(buffer));
                buffer = '';

                const span = document.createElement('span');
                span.className = 'display-mode';
                let math = '';
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
                let math = '';
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
                        displayMode: false
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
                span.className = 'display-mode';
                let math = '';
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
                let math = '';
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
                        displayMode: false
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
        if (node.className && node.className.indexOf && node.className.indexOf('no-render') >= 0) {
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
                        // turn "some text" to ${'$'}${'$'}${'$'}some text${'$'}${'$'}${'$'}
                        // so the text is in math mode
                        if (text[0] === '"') {
                            text = text.substring(1);
                        }
                        if (text[text.length - 1] === '"') {
                            text = text.substring(0, text.length - 1);
                        }
                        text = '${'$'}${'$'}${'$'}' + text + '${'$'}${'$'}${'$'}';
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
"""

const val INTERACTIVE_JS_CODE =
    """
    function flipEntity(id) {
        const renEl = document.getElementById('rendered-' + id);
        const litEl = document.getElementById('literal-' + id);
        if (renEl && litEl) {
            if (renEl.className === 'mathlingua-rendered-visible') {
                renEl.className = 'mathlingua-rendered-hidden';
                litEl.className = 'mathlingua-literal-visible';
            } else {
                renEl.className = 'mathlingua-rendered-visible';
                litEl.className = 'mathlingua-literal-hidden';
            }
        }
    }

    function toggleProof(id) {
        const proofEl = document.getElementById('proof-' + id);
        const iconEl = document.getElementById('icon-' + id);
        if (proofEl) {
            if (proofEl.className === 'mathlingua-proof-hidden') {
                proofEl.className = 'mathlingua-proof-shown';
                if (iconEl) {
                    iconEl.innerHTML = '&#9652;';
                }
            } else {
                proofEl.className = 'mathlingua-proof-hidden';
                if (iconEl) {
                    iconEl.innerHTML = '&#9662;';
                }
            }
        }
    }
"""

const val SHARED_CSS =
    """
    .content {
        padding-top: 1.5em;
        padding-bottom: 1em;
        margin-top: 2.5em;
        margin-bottom: 1em;
        margin-left: auto; /* for centering content */
        margin-right: auto; /* for centering content */
        font-size: 1em;
        width: 50em;
        background-color: white;
        border: solid;
        border-width: 1px;
        border-color: rgba(210, 210, 210);
        border-radius: 2px;
        box-shadow: rgba(0, 0, 0, 0.1) 0px 3px 10px,
                    inset 0  0 0 0 rgba(240, 240, 240, 0.5);
    }

    @media screen and (max-width: 500px) {
        .content {
            width: 99%;
            margin-top: 2.5vh;
            margin-bottom: 0;
        }
    }

    body {
        background-color: #dddddd;
        padding-bottom: 1em;
    }

    hr {
        border: 0.5px solid #efefef;
    }

    h1, h2, h3, h4 {
        color: #0055bb;
        text-align: center;
    }

    p {
        text-align: left;
        text-indent: 0;
    }

    .mathlingua-flip-icon {
        position: relative;
        top: 0;
        left: 100%;
        border: none;
        color: #aaaaaa;
        background: #ffffff;
        margin: 0;
        padding: 0;
        font-size: 110%;
    }

    .mathlingua-rendered-visible {
        display: block;
    }

    .mathlingua-rendered-hidden {
        display: none;
    }

    .mathlingua-literal-visible {
        display: block;
    }

    .mathlingua-literal-hidden {
        display: none;
    }

    .mathlingua-home {
        width: 80%;
        display: block;
        margin-left: auto;
        margin-right: auto;
    }

    .mathlingua-dir-item-hidden {
        display: none;
    }

    .mathlingua-dir-item-shown {
        display: block;
    }

    .mathlingua-home-item {
        font-weight: bold;
        display: block;
        margin-top: -1.25ex;
        margin-bottom: -1ex;
    }

    .mathlingua-list-dir-item {
        font-weight: bold;
        display: block;
        margin-top: -0.5ex;
        margin-bottom: -0.5ex;
    }

    .mathlingua-list-file-item {
        display: block;
        margin-top: -0.5ex;
        margin-bottom: -0.5ex;
    }

    .mathlingua-top-level {
        overflow-y: hidden;
        overflow-x: auto;
        background-color: white;
        border: solid;
        border-width: 1px;
        border-radius: 2px;
        padding-top: 0;
        padding-bottom: 1em;
        padding-left: 1.1em;
        padding-right: 1.1em;
        max-width: 75%;
        width: max-content;
        margin-left: auto; /* for centering content */
        margin-right: auto; /* for centering content */
        margin-top: 2.25em;
        margin-bottom: 2.25em;
        border-color: rgba(230, 230, 230);
        border-radius: 2px;
        box-shadow: rgba(0, 0, 0, 0.1) 0px 3px 10px,
                    inset 0  0 0 0 rgba(240, 240, 240, 0.5);
    }

    .mathlingua-block-comment {
        font-family: Georgia, 'Times New Roman', Times, serif;
    }

    .mathlingua-block-comment-top-level {
        font-family: Georgia, 'Times New Roman', Times, serif;
        line-height: 1.3;
        text-align: left;
        text-indent: 0;
        background-color: #ffffff;
        max-width: 80%;
        width: 80%;
        margin-left: auto; /* for centering content */
        margin-right: auto; /* for centering content */
    }

    .end-mathlingua-top-level {
        padding-top: 0.5em;
        margin: 0;
    }

    .mathlingua-topic-group-id {
        display: block;
        padding: 0 0 1em 0;
        font-family: monospace;
        text-align: center;
        color: #5500aa;
    }

    .mathlingua-topic-name {
        display: block;
        padding: 0 0 0.2em 0;
        font-family: Georgia, 'Times New Roman', Times, serif;
        font-weight: bold;
        text-align: center;
        color: #0055bb;
    }

    .mathlingua-topic-content {
        display: block;
        padding: 0 0 1.2em 0;
        margin: 0.2em 0 -1.2em 0;
        font-family: Georgia, 'Times New Roman', Times, serif;
        line-height: 1.3;
        color: #000000;
    }

    .mathlingua-overview {
        display: block;
        padding: 0.5em 0 0.5em 0;
        font-family: Georgia, 'Times New Roman', Times, serif;
        line-height: 1.3;
        color: #000000;
        margin-top: -2ex;
        margin-bottom: -2ex;
    }

    .mathlingua-resources-header {
        display: block;
        color: #0055bb;
        text-shadow: 0px 1px 0px rgba(255,255,255,0), 0px 0.4px 0px rgba(0,0,0,0.2);
        font-family: Georgia, 'Times New Roman', Times, serif;
        padding-top: 0.25em;
    }

    .mathlingua-resources-item {
        display: block;
        font-family: Georgia, 'Times New Roman', Times, serif;
        padding: 0.1em 0 0.1em 0;
    }
    
    .mathlingua-topics-header {
        display: block;
        color: #0055bb;
        text-shadow: 0px 1px 0px rgba(255,255,255,0), 0px 0.4px 0px rgba(0,0,0,0.2);
        font-family: Georgia, 'Times New Roman', Times, serif;
        padding-top: 0.5em;
    }

    .mathlingua-topic-item {
        display: block;
        font-family: Georgia, 'Times New Roman', Times, serif;
        padding: 0.1em 0 0.1em 0;
    }

    .mathlingua-related-header {
        display: block;
        color: #0055bb;
        text-shadow: 0px 1px 0px rgba(255,255,255,0), 0px 0.4px 0px rgba(0,0,0,0.2);
        font-family: Georgia, 'Times New Roman', Times, serif;
        padding-top: 0.5em;
    }

    .mathlingua-related-item {
        display: block;
        font-family: Georgia, 'Times New Roman', Times, serif;
        padding: 0.1em 0 0.1em 0;
    }

    .mathlingua-foundation-header {
        display: block;
        font-family: Georgia, 'Times New Roman', Times, serif;
        text-align: center;
        font-weight: normal;
        padding-bottom: 1ex;
        color: #777777;
    }

    .mathlingua {
        font-family: monospace;
    }

    .mathlingua-proof-icon {
        float:right;
        color: #aaaaaa;
        cursor: default;
    }

    .mathlingua-proof-shown {
        display: block;
        margin-top: -0.6ex;
    }

    .mathlingua-proof-hidden {
        display: none;
        margin-top: -0.6ex;
    }

    .mathlingua-proof-header {
        display: block;
        color: #0055bb;
        text-shadow: 0px 1px 0px rgba(255,255,255,0), 0px 0.4px 0px rgba(0,0,0,0.2);
        font-family: Georgia, 'Times New Roman', Times, serif;
    }

    .mathlingua-header {
        color: #0055bb;
        text-shadow: 0px 1px 0px rgba(255,255,255,0), 0px 0.4px 0px rgba(0,0,0,0.2);
        font-family: Georgia, 'Times New Roman', Times, serif;
    }

    .mathlingua-header-literal {
        color: #0055bb;
        text-shadow: 0px 1px 0px rgba(255,255,255,0), 0px 0.4px 0px rgba(0,0,0,0.2);
        font-family: monospace;
    }

    .mathlingua-whitespace {
        padding: 0;
        margin: 0;
        margin-left: 1ex;
    }

    .mathlingua-id {
        color: #5500aa;
        overflow-x: scroll;
    }

    .mathlingua-text {
        color: #000000;
        display: inline-block;
        font-family: Georgia, 'Times New Roman', Times, serif;
        margin-top: -1.6ex;
        margin-bottom: -1.6ex;
    }

    .mathlingua-text-no-render {
        color: #000000;
        display: inline-block;
        font-family: Georgia, 'Times New Roman', Times, serif;
        margin-top: -1.6ex;
        margin-bottom: -1.6ex;
    }

    .literal-mathlingua-text {
        color: #386930;
        display: inline;
        font-family: monospace;
        line-height: 1.3;
    }

    .literal-mathlingua-text-no-render {
        color: #386930;
        display: inline;
        font-family: monospace;
        line-height: 1.3;
    }

    .mathlingua-url {
        color: #0000aa;
        text-decoration: none;
        display: inline-block;
        font-family: Georgia, 'Times New Roman', Times, serif;
    }

    .mathlingua-link {
        color: #0000aa;
        text-decoration: none;
        display: inline-block;
    }

    .mathlingua-statement-no-render {
        color: #007377;
    }

    .mathlingua-statement-container {
        display: inline;
    }

    .literal-mathlingua-statement {
        color: #007377;
    }

    .mathlingua-dropdown-menu-shown {
        position: absolute;
        display: block;
        z-index: 1;
        background-color: #ffffff;
        box-shadow: rgba(0, 0, 0, 0.5) 0px 3px 10px,
                    inset 0  0 10px 0 rgba(200, 200, 200, 0.25);
        border: solid;
        border-width: 1px;
        border-radius: 0px;
        border-color: rgba(200, 200, 200);
    }

    .mathlingua-dropdown-menu-hidden {
        display: none;
    }

    .mathlingua-dropdown-menu-item {
        display: block;
        margin: 0.75ex;
    }

    .mathlingua-dropdown-menu-item:hover {
        font-style: italic;
    }

    .mathlingua-called {
        font-weight: bold;
        display: table;
        margin-top: -1em;
        margin-left: auto;
        margin-bottom: -1ex;
        margin-right: auto;
    }

    .display-mode {
        margin-left: auto;
        margin-right: auto;
        width: max-content;
        display: block;
        padding-top: 2ex;
        padding-bottom: 3ex;
    }

    .katex {
        font-size: 1em;
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
"""

fun buildStandaloneHtml(content: String) =
    """
<!DOCTYPE html>
<html>
    <head>
        $SHARED_HEADER
        <script>
            $KATEX_RENDERING_JS

            $INTERACTIVE_JS_CODE

            function initPage() {
                const el = document.getElementById('__main_content__');
                if (el) {
                    render(el);
                }
            }
        </script>
        <style>
            $SHARED_CSS
        </style>
    </head>
    <body onload="initPage()">
        <div class="content" id="__main_content__">
            $content
        </div>
    </body>
"""

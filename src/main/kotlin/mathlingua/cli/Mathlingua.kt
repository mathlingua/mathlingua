/*
 * Copyright 2021
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

import java.io.BufferedReader
import java.io.File
import java.io.InputStreamReader
import java.lang.IllegalArgumentException
import java.net.ServerSocket
import java.net.Socket
import java.net.SocketException
import java.util.UUID
import java.util.concurrent.Executors
import mathlingua.backend.BackEnd
import mathlingua.backend.SourceCollection
import mathlingua.backend.SourceFile
import mathlingua.backend.ValueSourceTracker
import mathlingua.backend.isMathLinguaFile
import mathlingua.backend.newSourceCollection
import mathlingua.frontend.FrontEnd
import mathlingua.frontend.chalktalk.phase2.HtmlCodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.clause.Identifier
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Text
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelBlockComment
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.FoundationGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resource.ResourceGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomGroup
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.support.validationFailure
import mathlingua.frontend.textalk.TexTalkNode
import mathlingua.frontend.textalk.TextTexTalkNode

const val TOOL_VERSION = "0.13"

const val MATHLINGUA_VERSION = "0.8"

private fun bold(text: String) = "\u001B[1m$text\u001B[0m"

private fun green(text: String) = "\u001B[32m$text\u001B[0m"

private fun red(text: String) = "\u001B[31m$text\u001B[0m"

private fun yellow(text: String) = "\u001B[33m$text\u001B[0m"

object Mathlingua {
    fun check(fs: VirtualFileSystem, logger: Logger, files: List<VirtualFile>, json: Boolean): Int {
        val sourceCollection =
            newSourceCollection(
                if (files.isEmpty()) {
                    listOf(fs.getDirectory(listOf("content")))
                } else {
                    files
                })
        val errors = BackEnd.check(sourceCollection)
        logger.log(getErrorOutput(fs, errors, sourceCollection.size(), json))
        return if (errors.isEmpty()) {
            0
        } else {
            1
        }
    }

    fun render(
        fs: VirtualFileSystem,
        logger: Logger,
        file: VirtualFile?,
        noexpand: Boolean,
        stdout: Boolean,
        raw: Boolean
    ): Int {
        val errors =
            if (file != null) {
                renderFile(
                    fs = fs,
                    logger = logger,
                    target = file,
                    stdout = stdout,
                    noExpand = noexpand,
                    raw = raw)
            } else {
                if (raw) {
                    val message = "ERROR: A file must be provided if --raw is used."
                    logger.log(message)
                    listOf(
                        ValueSourceTracker(
                            value = ParseError(message = message, row = -1, column = -1),
                            source =
                                SourceFile(
                                    file = fs.getFile(listOf("")),
                                    content = "",
                                    validation = validationFailure(emptyList())),
                            tracker = null))
                } else {
                    renderAll(fs = fs, logger = logger, stdout = stdout, noExpand = noexpand)
                }
            }
        return if (errors.isEmpty()) {
            0
        } else {
            1
        }
    }

    fun clean(fs: VirtualFileSystem, logger: Logger): Int {
        val indexFile = fs.getFile(listOf("docs", "index.html"))
        return if (!indexFile.exists()) {
            logger.log("Nothing to clean")
            0
        } else {
            if (indexFile.delete()) {
                logger.log("Deleted docs${File.separator}index.html")
                0
            } else {
                logger.log(
                    "${bold(red("ERROR: "))} Failed to delete docs${File.separator}index.html")
                1
            }
        }
    }

    fun version(logger: Logger): Int {
        logger.log("MathLingua CLI Version:      $TOOL_VERSION")
        logger.log("MathLingua Language Version: $MATHLINGUA_VERSION")
        return 0
    }

    fun serve(fs: VirtualFileSystem, logger: Logger, port: Int): Int {
        logger.log("Visit localhost:$port to see your rendered MathLingua code.")
        logger.log("Every time you refresh the page, your MathLingua code will be rerendered.")
        val serverSocket = ServerSocket(port)
        val service = Executors.newCachedThreadPool()
        Runtime.getRuntime().addShutdownHook(Thread { service.shutdown() })
        while (true) {
            val client = serverSocket.accept()
            service.submit { handleServeRequest(fs, logger, client) }
        }
    }

    private fun handleServeRequest(fs: VirtualFileSystem, logger: Logger, client: Socket) {
        try {
            val reader = BufferedReader(InputStreamReader(client.getInputStream()))
            // The first line of the request is of the form
            //   <method> <path> <http-version>
            // for example.
            //   GET / HTTP/1.1
            val firstLine = reader.readLine() ?: ""
            val parts = firstLine.split(" ")
            val method =
                if (parts.isEmpty()) {
                    null
                } else {
                    parts[0]
                }
            val urlWithoutParams =
                if (parts.size < 2) {
                    null
                } else {
                    val url = parts[1]
                    val index = url.indexOf("?")
                    if (index < 0) {
                        url
                    } else {
                        url.substring(0, index)
                    }
                }
            if (method != "GET" || urlWithoutParams != "/") {
                val output = client.getOutputStream()
                output.write("HTTP/1.1 404 Not Found\r\n".toByteArray())
                output.write("ContentType: text/html\r\n\r\n".toByteArray())
                output.write("Not Found".toByteArray())
                output.write("\r\n\r\n".toByteArray())
                output.flush()
                output.close()
            } else {
                logger.log("Rendering...")
                val start = System.currentTimeMillis()
                val triple = getRenderAllText(fs = fs, noExpand = false)
                val end = System.currentTimeMillis()
                try {
                    val output = client.getOutputStream()
                    output.write("HTTP/1.1 200 OK\r\n".toByteArray())
                    output.write("ContentType: text/html\r\n\r\n".toByteArray())
                    output.write(triple.first.toByteArray())
                    output.write("\r\n\r\n".toByteArray())
                    output.flush()
                    output.close()
                } finally {
                    logger.log("Completed in ${end - start} ms")
                    logger.log(triple.third)
                    logger.log("")
                }
            }
        } catch (e: SocketException) {
            // It appears to be common for a SocketException with message
            // "Broken pipe (Write failed)" if a request comes in while another
            // is still being processed. Just absorb the exception and try to
            // reconnect.
        }
    }
}

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
                "  \"file\": \"${err.source.file.absolutePath().joinToString(File.separator).jsonSanitize() ?: "None"}\",")
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
                    "${err.source.file.relativePathTo(cwd).joinToString(File.separator) ?: "None"} (Line: ${err.value.row + 1}, Column: ${err.value.column + 1})\n"))
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

private fun renderFile(
    fs: VirtualFileSystem,
    logger: Logger,
    target: VirtualFile,
    stdout: Boolean,
    noExpand: Boolean,
    raw: Boolean
): List<ValueSourceTracker<ParseError>> {
    if (!target.exists()) {
        val message =
            "ERROR: The file ${target.absolutePath().joinToString(File.separator)} does not exist"
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
            "ERROR: The path ${target.absolutePath().joinToString(File.separator)} is not a .math file"
        logger.log(message)
        return listOf(
            ValueSourceTracker(
                value = ParseError(message = message, row = -1, column = -1),
                source =
                    SourceFile(
                        file = target, content = "", validation = validationFailure(emptyList())),
                tracker = null))
    }

    val sourceCollection = newSourceCollection(listOf(fs.cwd()))
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

    if (!stdout) {
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
        logger.log("Wrote ${outFile.relativePathTo(fs.cwd()).joinToString(File.separator)}")
    }

    if (stdout) {
        logger.log(text)
    } else {
        logger.log(getErrorOutput(fs, errors, sourceCollection.size(), false))
    }

    return errors
}

private fun getRenderAllText(
    fs: VirtualFileSystem, noExpand: Boolean
): Triple<String, List<ValueSourceTracker<ParseError>>, String> {
    val docsDir = getDocsDirectory(fs)
    docsDir.mkdirs()

    val contentDir = getContentDirectory(fs)
    contentDir.mkdirs()

    val errors = mutableListOf<ValueSourceTracker<ParseError>>()

    val indexTest = getIndexFileText(fs, noExpand, errors)

    val sourceCollection = newSourceCollection(listOf(fs.cwd()))
    val errorOutput = getErrorOutput(fs, errors, sourceCollection.size(), false)

    return Triple(indexTest, errors, errorOutput)
}

private fun renderAll(
    fs: VirtualFileSystem, logger: Logger, stdout: Boolean, noExpand: Boolean
): List<ValueSourceTracker<ParseError>> {
    val triple = getRenderAllText(fs, noExpand)
    val indexFileText = triple.first
    val errors = triple.second
    val errorText = triple.third

    if (!stdout) {
        val indexFile = fs.getFile(listOf("docs", "index.html"))
        indexFile.writeText(indexFileText)
        logger.log("Wrote ${indexFile.relativePathTo(fs.cwd()).joinToString(File.separator)}")
    }

    if (stdout) {
        logger.log(indexFileText)
    } else {
        logger.log(errorText)
    }

    return errors
}

private fun findMathlinguaFiles(fileOrDir: VirtualFile, result: MutableList<VirtualFile>) {
    if (isMathLinguaFile(fileOrDir)) {
        result.add(fileOrDir)
    }
    if (fileOrDir.isDirectory()) {
        for (f in fileOrDir.listFiles()) {
            findMathlinguaFiles(f, result)
        }
    }
}

class LockedValue<T>() {
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

private fun getIndexFileText(
    fs: VirtualFileSystem, noexpand: Boolean, errors: MutableList<ValueSourceTracker<ParseError>>
): String {
    val cwd = fs.cwd()
    val allFileIds = mutableListOf<String>()
    val fileListBuilder = StringBuilder()
    val counter = Counter(0)
    val contentDir = getContentDirectory(fs)
    val firstFilePath = LockedValue<String>()
    if (contentDir.isDirectory()) {
        val children = getChildren(fs, contentDir)
        val hasDirSiblings = children.any { it.isDirectory() }
        for (child in children) {
            buildFileList(
                fs, child, hasDirSiblings, 0, fileListBuilder, allFileIds, firstFilePath, counter)
        }
    }

    val searchIndex = generateSearchIndex(fs)
    val searchIndexBuilder = StringBuilder()
    searchIndexBuilder.append("const index = new Map();\n")
    val words = searchIndex.keys.toList()
    for (i in words.indices) {
        val word = words[i]
        val pathToIndices = searchIndex[word]
        if (pathToIndices != null) {
            searchIndexBuilder.append("                const map$i = new Map();\n")
            val paths = pathToIndices.keys.toList()
            for (j in paths.indices) {
                val path = paths[j]
                val ids = pathToIndices[path]
                if (ids != null) {
                    searchIndexBuilder.append("                const set${i}_$j = new Set();\n")
                    for (id in ids) {
                        searchIndexBuilder.append("                set${i}_$j.add($id);\n")
                    }
                    searchIndexBuilder.append("                map$i.set('$path', set${i}_$j);\n")
                }
            }
            searchIndexBuilder.append(
                "                index.set('${word.replace("\\", "\\\\").replace("'", "\\\\\\'")}', map$i);\n")
        }
    }
    searchIndexBuilder.append("                return index;")

    val sigToPathCode = generateSignatureToPathJsCode(fs)

    val sourceCollection = newSourceCollection(listOf(cwd))
    val filesToProcess = mutableListOf<VirtualFile>()
    findMathlinguaFiles(cwd, filesToProcess)

    val pathToEntityMap =
        generatePathToEntityList(fs, filesToProcess, sourceCollection, noexpand, errors)
    val pathToEntityBuilder = StringBuilder()
    pathToEntityBuilder.append("                const map = new Map();\n")
    val keys = pathToEntityMap.keys.toList()
    for (path in keys) {
        val key = path.replace("\"", "\\\"")
        val entities = pathToEntityMap[key]!!.map { sanitizeHtmlForJs(it.first) }.map { "\"$it\"" }
        pathToEntityBuilder.append("                map.set(\"$key\", $entities);\n")
    }
    pathToEntityBuilder.append("                return map;\n")

    val homeContentFile = fs.getFile(listOf("docs", "home.md"))
    val showHome = homeContentFile.exists()
    val homeContent =
        if (showHome) {
            val writer =
                HtmlCodeWriter(
                    defines = emptyList(),
                    states = emptyList(),
                    axioms = emptyList(),
                    foundations = emptyList(),
                    literal = true)
            val homeText = homeContentFile.readText()
            writer.writeBlockComment("::$homeText::")
            writer.getCode()
        } else {
            ""
        }
    val homeHtml = sanitizeHtmlForJs("<span class=\"mathlingua-home\">$homeContent</span>")

    var gitHubUrl: String? = null
    try {
        val url =
            String(
                    Runtime.getRuntime()
                        .exec("git config --get remote.origin.url")
                        .inputStream
                        .readBytes())
                .trim()
        if (url.isNotEmpty()) {
            gitHubUrl =
                url.removeSuffix(".git").replaceFirst("git@github.com:", "https://github.com/")
        }
    } catch (e: Exception) {
        gitHubUrl = null
    }

    return buildIndexHtml(
        fileListBuilder.toString(),
        firstFilePath.getValueOrDefault(""),
        homeHtml,
        showHome,
        searchIndexBuilder.toString(),
        allFileIds,
        sigToPathCode,
        pathToEntityBuilder.toString(),
        gitHubUrl)
}

data class Counter(var count: Int) {
    fun next() = count++
}

private fun getChildren(fs: VirtualFileSystem, file: VirtualFile): List<VirtualFile> {
    if (!file.isDirectory()) {
        throw IllegalArgumentException("Cannot get children on non-directory: $file")
    }
    val tocPath = file.relativePathTo(fs.cwd()).toMutableList().plus("toc")
    val tocFile = fs.getFile(tocPath)
    return if (tocFile.exists()) {
        val relPath = file.relativePathTo(fs.cwd())
        tocFile.readText().split("\n").filter { it.trim().isNotEmpty() }.map {
            val path = mutableListOf<String>()
            path.addAll(relPath)
            path.add(it)
            // TODO: Update this to only use fs
            val tmpFile = File(fs.getFile(path).absolutePath().joinToString(File.separator))
            if (tmpFile.isFile) {
                fs.getFile(path)
            } else {
                fs.getDirectory(path)
            }
        }
    } else {
        file.listFiles()
    }
}

private fun buildFileList(
    fs: VirtualFileSystem,
    file: VirtualFile,
    hasDirSiblings: Boolean,
    indent: Int,
    builder: StringBuilder,
    allFileIds: MutableList<String>,
    firstFilePath: LockedValue<String>,
    counter: Counter
) {
    val elementId = counter.next()
    val dirSpanId = "dir-$elementId"
    val childBuilder = StringBuilder()
    if (file.isDirectory()) {
        val children = getChildren(fs, file)
        childBuilder.append("<span id='$dirSpanId' class='mathlingua-dir-item-hidden'>")
        val childrenContainsDir = children.any { it.isDirectory() }
        for (child in children) {
            buildFileList(
                fs,
                child,
                childrenContainsDir,
                indent + 12,
                childBuilder,
                allFileIds,
                firstFilePath,
                counter)
        }
        childBuilder.append("</span>")
    }

    val isMathFile = !file.isDirectory() && file.absolutePath().last().endsWith(".math")
    if ((file.isDirectory() && childBuilder.isNotEmpty()) || isMathFile) {
        val src =
            file.relativePathTo(fs.cwd()).joinToString(File.separator).replace(".math", ".html")
        val cssDesc = "style='padding-left: ${indent}px'"
        val classDesc =
            if (file.isDirectory()) {
                "class='mathlingua-list-dir-item'"
            } else {
                "class='mathlingua-list-file-item'"
            }
        val id = src.removeSuffix(".html")
        allFileIds.add(id)
        val iconId = "icon-$elementId"
        val onclick =
            if (file.isDirectory()) {
                "onclick=\"mathlinguaToggleDirItem('$dirSpanId', '$iconId')\""
            } else {
                "onclick=\"view('$src')\""
            }
        val icon =
            when {
                file.isDirectory() -> {
                    "&#9656;&nbsp;"
                }
                hasDirSiblings -> {
                    "&nbsp;&nbsp;"
                }
                else -> {
                    ""
                }
            }
        if (!file.isDirectory()) {
            firstFilePath.setValue(src)
        }
        builder.append(
            "<span $classDesc><a id='$id' $onclick><span $cssDesc><span id='$iconId'>$icon</span>${
                file.absolutePath().last().removeSuffix(".math").replace('_', ' ')
            }</span></a></span>")
        builder.append(childBuilder.toString())
    }
}

private fun generateSearchIndex(fs: VirtualFileSystem): Map<String, Map<String, Set<Int>>> {
    val result = mutableMapOf<String, MutableMap<String, MutableSet<Int>>>()
    generateSearchIndexImpl(fs, getContentDirectory(fs), result)
    return result
}

private fun generateSearchIndexImpl(
    fs: VirtualFileSystem,
    file: VirtualFile,
    index: MutableMap<String, MutableMap<String, MutableSet<Int>>>
) {
    if (file.isDirectory()) {
        val children = file.listFiles()
        for (child in children) {
            generateSearchIndexImpl(fs, child, index)
        }
    }

    if (!file.isDirectory() && file.absolutePath().last().endsWith(".math")) {
        val path = file.relativePathTo(fs.cwd()).joinToString(File.separator).removeSuffix(".math")
        when (val validation = FrontEnd.parse(file.readText())
        ) {
            is ValidationSuccess -> {
                val doc = validation.value
                val groups = doc.groups
                for (i in groups.indices) {
                    val words = getAllWords(groups[i])
                    for (word in words) {
                        val pathToIndex = index.getOrDefault(word, mutableMapOf())
                        val indices = pathToIndex.getOrDefault(path, mutableSetOf())
                        indices.add(i)
                        pathToIndex[path] = indices
                        index[word] = pathToIndex
                    }
                }
            }
        }
    }
}

private fun generateSignatureToPath(fs: VirtualFileSystem): Map<String, String> {
    val result = mutableMapOf<String, String>()
    generateSignatureToPathImpl(fs, getContentDirectory(fs), result, 0)
    return result
}

private fun generateSignatureToPathImpl(
    fs: VirtualFileSystem, file: VirtualFile, result: MutableMap<String, String>, depth: Int
) {
    if (file.isDirectory()) {
        val children = file.listFiles()
        for (child in children) {
            generateSignatureToPathImpl(fs, child, result, depth + 1)
        }
    }

    if (!file.isDirectory() && file.absolutePath().last().endsWith(".math")) {
        val path = file.relativePathTo(fs.cwd()).joinToString(File.separator).removeSuffix(".math")
        when (val validation = FrontEnd.parse(file.readText())
        ) {
            is ValidationSuccess -> {
                val doc = validation.value
                val groups = doc.groups
                for (i in groups.indices) {
                    val grp = groups[i]
                    val signature =
                        when (grp) {
                            is AxiomGroup -> {
                                grp.id?.text
                            }
                            is ResourceGroup -> {
                                grp.id
                            }
                            is DefinesGroup -> {
                                grp.signature?.form
                            }
                            is StatesGroup -> {
                                grp.signature?.form
                            }
                            is FoundationGroup -> {
                                when (val content = grp.foundationSection.content
                                ) {
                                    is DefinesGroup -> {
                                        content.signature?.form
                                    }
                                    is StatesGroup -> {
                                        content.signature?.form
                                    }
                                    else -> {
                                        null
                                    }
                                }
                            }
                            else -> {
                                null
                            }
                        }
                    if (signature != null) {
                        val pathBuilder = StringBuilder()
                        for (j in 0 until depth) {
                            pathBuilder.append("../")
                        }
                        pathBuilder.append(path)
                        pathBuilder.append(".html?show=")
                        pathBuilder.append(i)
                        result[signature] = pathBuilder.toString()
                    }
                }
            }
        }
    }
}

private fun generateSignatureToPathJsCode(fs: VirtualFileSystem): String {
    val signatureToPath = generateSignatureToPath(fs)
    val signatureToPathBuilder = StringBuilder()
    signatureToPathBuilder.append("const sigToPath = new Map();")
    for (entry in signatureToPath.entries) {
        signatureToPathBuilder.append(
            "sigToPath.set('${sanitizeHtmlForJs(entry.key)}', '${sanitizeHtmlForJs(entry.value)}');")
    }
    signatureToPathBuilder.append("return sigToPath;")
    return signatureToPathBuilder.toString()
}

private fun generatePathToEntityList(
    fs: VirtualFileSystem,
    filesToProcess: List<VirtualFile>,
    sourceCollection: SourceCollection,
    noexpand: Boolean,
    errors: MutableList<ValueSourceTracker<ParseError>>
): Map<String, List<Pair<String, Phase2Node?>>> {
    val result = mutableMapOf<String, List<Pair<String, Phase2Node?>>>()
    for (f in filesToProcess) {
        val path = f.relativePathTo(fs.cwd()).joinToString(File.separator)
        result[path] = getUnifiedRenderedTopLevelElements(f, sourceCollection, noexpand, errors)
    }
    return result
}

private fun generatePathToCompleteEntityList(
    fs: VirtualFileSystem,
    filesToProcess: List<VirtualFile>,
    sourceCollection: SourceCollection,
    noexpand: Boolean,
    errors: MutableList<ValueSourceTracker<ParseError>>
): Map<String, List<RenderedTopLevelElement>> {
    val result = mutableMapOf<String, List<RenderedTopLevelElement>>()
    for (f in filesToProcess) {
        val path = f.relativePathTo(fs.cwd()).joinToString(File.separator)
        result[path] = getCompleteRenderedTopLevelElements(f, sourceCollection, noexpand, errors)
    }
    return result
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
        sourceCollection.prettyPrint(file = f, html = true, literal = true, doExpand = !noexpand)
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
                renderedFormHtml = expandedPair.first[i].first,
                rawFormHtml = literalPair.first[i].first,
                node = expandedPair.first[i].second))
    }
    return result
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
            val id = UUID.randomUUID()
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
        is Statement -> {
            val root = node.texTalkRoot
            if (root is ValidationSuccess) {
                getAllWordsImpl(root.value, words)
            }
        }
        is Identifier -> {
            words.add(node.name.toLowerCase())
        }
        is Text -> {
            words.addAll(
                node.text.split(" ").map { it.trim() }.filter { it.isNotEmpty() }.map {
                    it.toLowerCase()
                })
        }
    }

    node.forEach { getAllWordsImpl(it, words) }
}

private fun getAllWordsImpl(node: TexTalkNode, words: MutableSet<String>) {
    when (node) {
        is TextTexTalkNode -> {
            words.add(node.text.toLowerCase())
        }
    }

    node.forEach { getAllWordsImpl(it, words) }
}

const val SHARED_HEADER =
    """
    <meta name="viewport" content="width=100%, initial-scale=1.0">
    <meta content="text/html;charset=utf-8" http-equiv="Content-Type">
    <meta content="utf-8" http-equiv="encoding">
    <link rel="stylesheet"
          href="https://cdn.jsdelivr.net/npm/katex@0.11.1/dist/katex.min.css"
          integrity="sha384-zB1R0rpPzHqg7Kpt0Aljp8JPLqbXI3bhnPWROx27a9N0Ll6ZP/+DiW/UqRcLbRjq"
          crossorigin="anonymous">
    <script defer
            src="https://cdn.jsdelivr.net/npm/katex@0.11.1/dist/katex.min.js"
            integrity="sha384-y23I5Q6l+B6vatafAwxRu/0oK/79VlbSz7Q9aiSZUvyWYIYsd+qj+o24G5ZU2zJz"
            crossorigin="anonymous">
    </script>
"""

const val KATEX_RENDERING_JS =
    """
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
            if (text[i] === '${'$'}' && text[i+1] === '${'$'}' && text[i+2] === '${'$'}') {
                i += 3; // skip over the ${'$'}s
                fragment.appendChild(document.createTextNode(buffer));
                buffer = '';

                const span = document.createElement('span');
                var math = '';
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
        margin-top: 3%;
        margin-bottom: 1em;
        margin-left: auto; /* for centering content */
        margin-right: auto; /* for centering content */
        font-size: 1em;
        width: 90%;
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

    .github-icon-container {
        position: absolute;
        top: 18%;
        right: 0%;
        margin-right: 1ex;
    }

    @media screen and (max-width: 500px) {
        .github-icon-container {
            position: absolute;
            top: 3%;
            right: 0%;
            margin-right: 1ex;
        }
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
        font-weight: bold;
        padding-bottom: 0.25em;
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

fun buildIndexHtml(
    fileListHtml: String,
    initialPath: String,
    homeHtml: String,
    showHome: Boolean,
    searchIndexInitCode: String,
    allFileIds: List<String>,
    signatureToPathCode: String,
    pathToEntityList: String,
    gitHubUrl: String?
) =
    """
<!DOCTYPE html>
<html>
    <head>
        $SHARED_HEADER
        <script>
            const ALL_FILE_IDS = [${allFileIds.joinToString(",") { "'$it'" }}];
            ALL_FILE_IDS.push('home');
            const INITIAL_SRC = "home.html";
            const SEARCH_INDEX = (function() {
                $searchIndexInitCode
            })();

            const SIGNATURE_TO_PATH = (function() {
                $signatureToPathCode
            })();

            const PATH_TO_ENTITY_LIST = (function() {
                $pathToEntityList
            })();

            const HOME_SRC = "$homeHtml";

            $INTERACTIVE_JS_CODE

            function mathlinguaToggleDirItem(dirSpanId, dirIconId) {
                const span = document.getElementById(dirSpanId);
                const icon = document.getElementById(dirIconId);
                if (span && icon) {
                    if (span.className === 'mathlingua-dir-item-hidden') {
                        span.className = 'mathlingua-dir-item-shown';
                        icon.innerHTML = '&#9662;&nbsp;';
                    } else {
                        span.className = 'mathlingua-dir-item-hidden'
                        icon.innerHTML = '&#9656;&nbsp;';
                    }
                }
            }

            function mathlinguaToggleDropdown(id) {
                const el = document.getElementById(id);
                if (el) {
                    if (el.className === 'mathlingua-dropdown-menu-hidden') {
                        el.className = 'mathlingua-dropdown-menu-shown';
                    } else {
                        el.className = 'mathlingua-dropdown-menu-hidden';
                    }
                }
            }

            function mathlinguaViewSignature(signature, id) {
                mathlinguaToggleDropdown(id);
                const path = SIGNATURE_TO_PATH.get(signature);
                if (path) {
                    const bottom = document.getElementById('__bottom_panel__');
                    if (bottom) {
                        const key = path.replace(/\?.*/g, '')
                                        .replace(/\.html/g, '.math')
                                        .replace(/\.\.\//g, '');
                        const index = path.replace(/.*\?show=/g, '');
                        const entHtml = PATH_TO_ENTITY_LIST.get(key)[index];

                        const ent = document.createElement('div');
                        ent.className = 'mathlingua-top-level';
                        ent.innerHTML = entHtml;

                        const div = document.createElement('div');
                        div.style.margin = '1em';
                        div.style.display = 'inline-block';
                        div.style.backgroundColor = '#ffffff';

                        const closeButton = document.createElement('a');
                        closeButton.text = '✕';
                        closeButton.style.fontSize = '80%';
                        closeButton.style.padding = '1ex';
                        closeButton.style.float = 'right';
                        closeButton.onclick = () => {
                            bottom.removeChild(div);
                            if (bottom.childElementCount === 0) {
                                const content = document.getElementById('__main_content__');
                                if (content) {
                                    content.style.marginBottom = '1em';
                                    bottom.style.display = 'none';
                                }
                            }
                        };

                        div.appendChild(closeButton);
                        div.appendChild(ent);

                        bottom.appendChild(div);
                        bottom.style.display = 'block';
                        render(div);

                        const content = document.getElementById('__main_content__');
                        if (content) {
                            content.style.marginBottom = bottom.clientHeight + 'px';
                        }
                    }
                }
            }

            $KATEX_RENDERING_JS

            function setup() {
                render(document.body);
                const params = new URLSearchParams(window.location.search);
                const showIds = new Set(params.getAll('show'));
                if (showIds.size > 0) {
                    let i = 0;
                    while (true) {
                        const id = '' + (i++);
                        const el = document.getElementById(id);
                        if (!el) {
                            break;
                        }
                        if (showIds.has(id)) {
                            el.style.display = 'block';
                        } else {
                            el.style.display = 'none';
                        }
                    }
                }
            }

            function forMobile() {
                return window?.screen?.width <= 500;
            }

            let open = !forMobile();

            function toggleSidePanel() {
                if (open) {
                    document.getElementById('sidebar').style.width = '0';
                    document.getElementById('main').style.marginLeft = '0';
                    if (forMobile()) {
                        document.getElementById('__bottom_panel__').style.width = '91.5%';
                    } else {
                        document.getElementById('__bottom_panel__').style.width = '96.5%';
                    }
                } else {
                    let margin = forMobile() ? '50%' : '20%';
                    document.getElementById('sidebar').style.width = margin;
                    document.getElementById('main').style.marginLeft = margin;
                    if (forMobile()) {
                        document.getElementById('__bottom_panel__').style.width = '43.5%';
                    } else {
                        document.getElementById('__bottom_panel__').style.width = '76.5%';
                    }
                }
                open = !open;
            }

            function view(path, setHistory = true) {
                const content = document.getElementById('__main_content__');
                if (!path) {
                    path = '${
                        if (showHome) {
                            "home.html"
                        } else {
                            initialPath
                        }
                    }';
                }
                if (setHistory) {
                    history.pushState({}, '', '?path=' + path);
                }
                if (content) {
                    for (const path of ALL_FILE_IDS) {
                        if (!path) {
                            continue;
                        }
                        const el = document.getElementById(path);
                        if (el) {
                            el.style.fontStyle = 'normal';
                        }
                    }
                    const id = path.replace(/\.html.*/, '');
                    if (id) {
                        const selectedEntry = document.getElementById(id);
                        if (selectedEntry) {
                            selectedEntry.style.fontStyle = 'italic';
                        }
                    }

                    if (!path || path === 'home.html') {
                        content.innerHTML = HOME_SRC;
                        render(content);
                        return;
                    }

                    let idSet = null;
                    const parts = path.split(';');
                    if (parts.length === 2) {
                        idSet = new Set(parts[1].split(','));
                    }

                    const newPath = path.replace(/;.*/g, '').replace(/\.html${'$'}/, ".math");
                    const entityList = PATH_TO_ENTITY_LIST.get(newPath);
                    if (entityList) {
                        while (content.firstChild) {
                            content.removeChild(content.firstChild);
                        }
                        for (let i=0; i<entityList.length; i++) {
                            if (!!idSet && idSet.has(id)) {
                                continue;
                            }

                            const el = document.createElement('div');
                            if (entityList[i].indexOf('class=\'mathlingua-block-comment\'') >= 0) {
                                el.className = 'mathlingua-block-comment-top-level';
                            } else {
                                el.className = 'mathlingua-top-level';
                            }
                            el.innerHTML = entityList[i];
                            content.appendChild(el);
                        }
                        render(content);
                    }
                }

                // the following code fixes a bug where text spans don't have enough
                // space under them if they are the last element in a top-level-group
                const topLevels = document.getElementsByClassName('mathlingua-top-level');
                if (topLevels) {
                    for (const topLevel of topLevels) {
                        let rightmost = null;
                        let node = topLevel;
                        while (node && node.childElementCount > 0) {
                            node = node.lastChild;
                            if (node && (node.className === 'mathlingua-text' ||
                                         node.className === 'mathlingua-text-no-render' ||
                                         node.className === 'literal-mathlingua-text')) {
                                rightmost = node;
                                break;
                            }
                        }
                        if (rightmost) {
                            rightmost.style.marginBottom = '0.2em';
                        }
                    }
                }
            }

            function clearSearch() {
                for (const id of ALL_FILE_IDS) {
                    if (!id) {
                        continue;
                    }
                    const el = document.getElementById(id);
                    if (el) {
                        el.style.display = 'block';
                        el.setAttribute('onclick', "view('" + id + ".html" + "')");
                    }
                }
                view('');
                const el = document.getElementById('search-input');
                if (el) {
                    el.value = '';
                }
            }

            function search() {
                const el = document.getElementById('search-input');
                if (el) {
                    if (!el.value || !el.value.trim()) {
                        clearSearch();
                        return;
                    }

                    const terms = el.value.split(' ')
                                    .map(it => it.trim().toLowerCase())
                                    .filter(it => it.length > 0);
                    if (terms.length == 0) {
                        clearSearch();
                        return;
                    }

                    view('');
                    const pathsToIndices = new Map();
                    if (terms.length > 0) {
                        const temp = SEARCH_INDEX.get(terms[0]) || new Map();
                        for (const [path, indices] of temp) {
                            pathsToIndices.set(path, new Set(indices));
                        }
                    }

                    for (let i=1; i<terms.length; i++) {
                        const term = terms[i];
                        const temp = SEARCH_INDEX.get(term) || new Map();
                        for (const path of pathsToIndices.keys()) {
                            if (!temp.has(path)) {
                                pathsToIndices.delete(path);
                            } else {
                                const intersection = new Set();
                                const tempIndices = temp.get(path) || new Set();
                                for (const id of (pathsToIndices.get(path) || new Set())) {
                                    if (tempIndices.has(id)) {
                                        intersection.add(id);
                                    }
                                }
                                if (intersection.size > 0) {
                                    pathsToIndices.set(path, intersection);
                                } else {
                                    pathsToIndices.delete(path);
                                }
                            }
                        }
                    }

                    if (pathsToIndices.size === 0) {
                        alert('No results found');
                        return;
                    }

                    const pathToNewPath = new Map();
                    let firstPath = null;
                    for (const [path, ids] of pathsToIndices) {
                        let newPath = path + '.html;' + Array.from(ids).toString();
                        if (firstPath === null) {
                            firstPath = newPath;
                        }
                        pathToNewPath.set(path, newPath);
                    }

                    for (const id of ALL_FILE_IDS) {
                        if (!id) {
                            continue;
                        }
                        const el = document.getElementById(id);
                        if (el) {
                            if (pathToNewPath.has(id)) {
                                el.style.display = 'block';
                                el.setAttribute('onclick', "view('" + pathToNewPath.get(id) + "')");
                            } else if (id !== 'home') {
                                el.style.display = 'none';
                            }
                        }
                    }

                    for (const id of ALL_FILE_IDS) {
                        if (pathToNewPath.has(id)) {
                            const parts = id.split('/');
                            while (parts.length > 0) {
                                parts.pop();
                                let parent = parts.join('/');
                                if (parent) {
                                    const parentEl = document.getElementById(parent);
                                    if (parentEl) {
                                        parentEl.style.display = 'block';
                                    }
                                }
                            }
                        }
                    }

                    if (firstPath) {
                        view(firstPath);
                    }
                }
            }

            function initPage() {
                window.onpopstate = function(event) {
                    if (document.location && document.location.search) {
                        const search = document.location.search.substring(1);
                        if (search.startsWith('path=')) {
                            const path = search.substring(5);
                            view(path, false);
                        }
                    }
                }

                let path = '';
                if (window.location && window.location.search) {
                    const search = window.location.search.substring(1);
                    if (search.indexOf('path=') === 0) {
                        path = search.substring(5).trim();
                    }
                }

                const el = document.getElementById('search-input');
                if (el) {
                    el.addEventListener("keyup", function(event) {
                        if (event.keyCode === 13) {
                            event.preventDefault();
                            search();
                        }
                    });
                }
                setup();
                view(path);
            }
        </script>
        <style>
            $SHARED_CSS

            .topbar {
                display: block;
                height: 4%;
                background-color: #444444;
                position: fixed;
                z-index: 2;
                top: 0;
                left: 0;
                width: 100%;
                border-width: 1px;
                border-color: #555555;
                border-bottom-style: solid;
                box-shadow: rgba(0, 0, 0, 0.3) 0px 3px 10px,
                            inset 0  0 10px 0 rgba(200, 200, 200, 0.25);
                padding-top: 0.5em;
            }

            @media screen and (max-width: 500px) {
                .topbar {
                    height: 2.5%;
                    }
            }

            .search-area {
                position: absolute;
                left: 50%;
                top: 50%;
                transform: translate(-50%, -50%);
                width: max-content;
            }

            .sidebar {
                height: 95%;
                width: 20%;
                position: fixed;
                z-index: 1;
                top: 4%;
                left: 0;
                background-color: #fefefe;
                border-right: solid;
                border-width: 1px;
                border-color: rgba(215, 215, 215);
                box-shadow: rgba(0, 0, 0, 0.2) 0px 3px 10px,
                            inset 0  0 10px 0 rgba(200, 200, 200, 0);
                overflow-x: scroll;
                padding-top: 1.5em;
                transition: 0.5s;
            }

            @media screen and (max-width: 500px) {
                .sidebar {
                    width: 0%;
                    top: 2.5%;
                }
            }

            .sidebar a {
                text-decoration: none;
                color: #000000;
                display: block;
                transition: 0.3s;
                padding-left: 1.25em;
                padding-right: 0;
                padding-top: 0.5em;
                padding-bottom: 0.5em;
                margin: 0;
            }

            .search {
                border: none;
                border-radius: 0;
                line-height: 1.75em;
                background-color: #eeeeee;
            }

            .button {
                border: solid;
                border-width: 1px;
                padding-left: 0.5em;
                padding-right: 0.5em;
                padding-top: 0.25em;
                padding-bottom: 0.25em;
                background-color: #333333;
                border-color: #303030;
                color: #cccccc;
            }

            .closeButton {
                text-decoration: none;
                color: #cccccc;
                margin-left: 0.75em;
                position: absolute;
                left: 0;
                top: 50%;
                transform: translate(0, -50%);
            }

            #main {
                transition: margin-left .5s;
                padding-left: 0;
                padding-right: 0;
                padding-bottom: 0;
                padding-top: 1em;
                margin-left: 20%;
                margin-right: 0;
                margin-bottom: 0;
            }

            @media screen and (max-width: 500px) {
                #main {
                    margin: 0;
                    padding: 0;
                }
            }

            .bottom-panel {
                display: none;
                background-color: #ffffff;
                overflow-x: scroll;
                overflow-y: scroll;
                white-space: nowrap;
                width: 76.5%;
                max-width: max-content;
                max-height: 50%;
                border-width: 1px;
                border-color: #555555;
                border-bottom-style: solid;
                box-shadow: rgba(0, 0, 0, 0.3) 0px 3px 10px,
                            inset 0  0 10px 0 rgba(200, 200, 200, 0.25);
                z-index: 1;
                bottom: 0;
                position: fixed;
            }

            @media screen and (max-width: 500px) {
                .bottom-panel {
                    width: 91.5%;
                }
            }
        </style>
    </head>
    <body id="main" onload="initPage()">
        <div id="top-bar" class="topbar">
            <a id="closeButton" class="closeButton" onclick="toggleSidePanel()">&#x2630;</a>
            <span class="search-area">
                <input type="text" id="search-input" class="search" aria-label="search">
                <button type="button" class="button" onclick="search()">Search</button>
                <button type="button" class="button" onclick="clearSearch()">Reset</button>
            </span>
            ${
                if (gitHubUrl != null) {
                    """
                        <div class="github-icon-container">
                            <a href="$gitHubUrl">
                                <svg fill="#cccccc" width="3vh" height="3vh" role="img" viewBox="0 0 24 24"
                                     xmlns="http://www.w3.org/2000/svg"><title>GitHub</title><path d="M12
                                     .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577
                                     0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7
                                     3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07
                                     1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93
                                     0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3
                                     1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23
                                     3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805
                                     5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0
                                     .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"/></svg>
                            </a>
                        </div>
                    """
                } else {
                    ""
                }
            }
        </div>

        <div id="sidebar" class="sidebar">
            ${
                if (showHome) {
                    "<a id='home' onclick=\"view('home.html')\"><span class=\"mathlingua-home-item\">Home</span></a>\n" +
                    "<hr>"
                } else {
                    ""
                }
            }
            $fileListHtml
        </div>

        <div class='bottom-panel' id='__bottom_panel__'></div>
        <div class="content" id="__main_content__"></div>
    </body>
</html>
"""

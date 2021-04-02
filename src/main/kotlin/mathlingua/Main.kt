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

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.subcommands
import com.github.ajalt.clikt.output.TermUi
import com.github.ajalt.clikt.parameters.arguments.argument
import com.github.ajalt.clikt.parameters.arguments.multiple
import com.github.ajalt.clikt.parameters.options.default
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.types.choice
import java.io.File
import java.lang.IllegalArgumentException
import java.nio.file.FileSystems
import java.nio.file.Paths
import java.nio.file.StandardWatchEventKinds
import java.nio.file.WatchService
import kotlin.system.exitProcess
import kotlinx.coroutines.runBlocking
import mathlingua.backend.BackEnd
import mathlingua.backend.SourceCollection
import mathlingua.backend.SourceFile
import mathlingua.backend.ValueSourceTracker
import mathlingua.backend.isMathLinguaFile
import mathlingua.backend.newSourceCollectionFromFiles
import mathlingua.frontend.FrontEnd
import mathlingua.frontend.chalktalk.phase2.ast.clause.Identifier
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Text
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.FoundationGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resource.ResourceGroup
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

private fun error(msg: String) = TermUi.echo(message = msg, err = true)

private fun log(msg: String) = TermUi.echo(message = msg, err = false)

private fun String.jsonSanitize() =
    this.replace("\\", "\\\\")
        .replace("\b", "\\b")
        .replace("\n", "\\n")
        .replace("\t", "\\t")
        .replace("\r", "\\r")
        .replace("\"", "\\\"")

private fun getDocsDirectory(): File {
    val cwd = Paths.get(".").toAbsolutePath().normalize().toFile()
    return File(cwd, "docs")
}

private fun getContentDirectory(): File {
    val cwd = Paths.get(".").toAbsolutePath().normalize().toFile()
    return File(cwd, "content")
}

private class Mlg : CliktCommand() {
    override fun run() = Unit
}

private class Check : CliktCommand(help = "Check input files for errors.") {
    private val file: List<String> by argument(
            help = "The *.math files and/or directories to process")
        .multiple(required = false)
    private val json: Boolean by option(help = "Output the results in JSON format.").flag()

    override fun run(): Unit =
        runBlocking {
            val sourceCollection =
                newSourceCollectionFromFiles(
                    if (file.isEmpty()) {
                        listOf(Paths.get(".").toAbsolutePath().normalize().toFile())
                    } else {
                        file.map { File(it) }
                    })
            val errors = BackEnd.check(sourceCollection)
            log(getErrorOutput(errors, sourceCollection.size(), json))
            exitProcess(
                if (errors.isEmpty()) {
                    0
                } else {
                    1
                })
        }
}

private class Version : CliktCommand(help = "Prints the tool and MathLingua language version.") {
    override fun run() {
        log("MathLingua CLI Version:      $TOOL_VERSION")
        log("MathLingua Language Version: $MATHLINGUA_VERSION")
        exitProcess(0)
    }
}

private fun write(cwd: File, content: String, outFile: File, stdout: Boolean) {
    if (stdout) {
        log(content)
    } else {
        val parent = outFile.parentFile
        if (!parent.exists()) {
            parent.mkdirs()
        }
        outFile.writeText(content)
        log("Wrote ${outFile.normalize().relativePath(cwd)}")
    }
}

private fun buildFileList(
    cwd: File, file: File, indent: Int, builder: StringBuilder, allFileIds: MutableList<String>
) {
    val childBuilder = StringBuilder()
    if (file.isDirectory) {
        val children = file.listFiles()
        if (children != null) {
            for (child in children) {
                buildFileList(cwd, child, indent + 12, childBuilder, allFileIds)
            }
        }
    }

    val isMathFile = file.isFile && file.extension == "math"
    if ((file.isDirectory && childBuilder.isNotEmpty()) || isMathFile) {
        val src = file.relativePath(cwd).replace(".math", ".html")
        val cssBuilder = StringBuilder()
        cssBuilder.append(
            "padding-left: ${indent}px;font-family: Georgia, 'Times New Roman', Times, serif;")
        if (file.isDirectory) {
            cssBuilder.append("font-weight: bold;")
        }
        val id = src.removeSuffix(".html")
        allFileIds.add(id)
        val onclick =
            if (file.isDirectory) {
                ""
            } else {
                "onclick=\"view('$src')\""
            }
        builder.append(
            "<a id='$id' $onclick><span style=\"${cssBuilder}\">${file.name.removeSuffix(".math")}</span></a>")
        builder.append(childBuilder.toString())
    }
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

private fun generateSearchIndex(cwd: File, contentDir: File): Map<String, Map<String, Set<Int>>> {
    val result = mutableMapOf<String, MutableMap<String, MutableSet<Int>>>()
    generateSearchIndexImpl(cwd, contentDir, result)
    return result
}

private fun generateSearchIndexImpl(
    cwd: File, file: File, index: MutableMap<String, MutableMap<String, MutableSet<Int>>>
) {
    if (file.isDirectory) {
        val children = file.listFiles()
        if (children != null) {
            for (child in children) {
                generateSearchIndexImpl(cwd, child, index)
            }
        }
    }

    if (file.isFile && file.extension == "math") {
        val path = file.relativePath(cwd).removeSuffix(".math")
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

private fun generateSignatureToPath(cwd: File, contentDir: File): Map<String, String> {
    val result = mutableMapOf<String, String>()
    generateSignatureToPathImpl(cwd, contentDir, result, 0)
    return result
}

private fun generateSignatureToPathImpl(
    cwd: File, file: File, result: MutableMap<String, String>, depth: Int
) {
    if (file.isDirectory) {
        val children = file.listFiles()
        if (children != null) {
            for (child in children) {
                generateSignatureToPathImpl(cwd, child, result, depth + 1)
            }
        }
    }

    if (file.isFile && file.extension == "math") {
        val path = file.relativePath(cwd).removeSuffix(".math")
        when (val validation = FrontEnd.parse(file.readText())
        ) {
            is ValidationSuccess -> {
                val doc = validation.value
                val groups = doc.groups
                for (i in groups.indices) {
                    val grp = groups[i]
                    val signature =
                        when (grp) {
                            is ResourceGroup -> {
                                grp.id
                            }
                            is DefinesGroup -> {
                                grp.signature
                            }
                            is StatesGroup -> {
                                grp.signature
                            }
                            is FoundationGroup -> {
                                when (val content = grp.foundationSection.content
                                ) {
                                    is DefinesGroup -> {
                                        content.signature
                                    }
                                    is StatesGroup -> {
                                        content.signature
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

private fun generateSignatureToPathJsCode(cwd: File, contentDir: File): String {
    val signatureToPath = generateSignatureToPath(cwd, contentDir)
    val signatureToPathBuilder = StringBuilder()
    signatureToPathBuilder.append("const sigToPath = new Map();")
    for (entry in signatureToPath.entries) {
        signatureToPathBuilder.append("sigToPath.set('${entry.key}', '${entry.value}');")
    }
    signatureToPathBuilder.append("return sigToPath;")
    return signatureToPathBuilder.toString()
}

private fun sanitizeHtmlForJs(html: String) =
    html.replace("\\", "\\\\").replace("\n", "\\n").replace("\"", "\\\"")

private fun getIndexFileText(
    files: List<String>,
    cwd: File,
    contentDir: File,
    html: Boolean,
    noexpand: Boolean,
    errors: MutableList<ValueSourceTracker<ParseError>>
): String {
    val allFileIds = mutableListOf<String>()
    val fileListBuilder = StringBuilder()
    if (cwd.isDirectory) {
        val children = cwd.listFiles()
        if (children != null) {
            for (child in children) {
                buildFileList(cwd, child, 0, fileListBuilder, allFileIds)
            }
        }
    }

    val searchIndex = generateSearchIndex(cwd, contentDir)
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
            searchIndexBuilder.append("                index.set('$word', map$i);\n")
        }
    }
    searchIndexBuilder.append("                return index;")

    val sigToPathCode = generateSignatureToPathJsCode(cwd, contentDir)

    val sourceCollection = newSourceCollectionFromFiles(listOf(cwd))
    val filesToProcess = mutableListOf<File>()
    val inputFiles =
        if (files.isEmpty()) {
            listOf(cwd)
        } else {
            files.map { File(it) }
        }
    for (f in inputFiles) {
        if (f.isDirectory) {
            filesToProcess.addAll(f.walk().filter { isMathLinguaFile(it) })
        } else if (isMathLinguaFile(f)) {
            filesToProcess.add(f)
        }
    }

    val pathToEntityMap =
        generatePathToEntityList(cwd, filesToProcess, sourceCollection, html, noexpand, errors)
    val pathToEntityBuilder = StringBuilder()
    pathToEntityBuilder.append("                const map = new Map();\n")
    val keys = pathToEntityMap.keys.toList()
    for (path in keys) {
        val key = path.replace("\"", "\\\"")
        val entities = pathToEntityMap[key]!!.map { sanitizeHtmlForJs(it) }.map { "\"$it\"" }
        pathToEntityBuilder.append("                map.set(\"$key\", $entities);\n")
    }
    pathToEntityBuilder.append("                return map;\n")

    val homeContentFile = File(getDocsDirectory(), "home.html")
    val homeContent =
        if (homeContentFile.exists()) {
            homeContentFile.readText()
        } else {
            "<p>Create a file <code>docs/home.html</code> to describe this repository.</p>"
        }
    val homeHtml =
        sanitizeHtmlForJs(
            "<div style=\"font-size: 80%;font-family: Georgia, 'Times New Roman', Times, serif;\"><div class='mathlingua-top-level'>$homeContent</div></div>")
    return buildIndexHtml(
        fileListBuilder.toString(),
        homeHtml,
        searchIndexBuilder.toString(),
        allFileIds,
        sigToPathCode,
        pathToEntityBuilder.toString())
}

private fun generatePathToEntityList(
    cwd: File,
    filesToProcess: List<File>,
    sourceCollection: SourceCollection,
    html: Boolean,
    noexpand: Boolean,
    errors: MutableList<ValueSourceTracker<ParseError>>
): Map<String, List<String>> {
    val result = mutableMapOf<String, List<String>>()
    for (f in filesToProcess) {
        val path = f.relativePath(cwd)
        val pair = sourceCollection.prettyPrint(file = f, html = html, doExpand = !noexpand)
        errors.addAll(
            pair.second.map {
                ValueSourceTracker(
                    value = it,
                    source =
                        SourceFile(
                            file = f, content = "", validation = validationFailure(emptyList())),
                    tracker = null)
            })
        result[path] = pair.first
    }
    return result
}

private fun render(
    filename: String, format: String, stdout: Boolean, noexpand: Boolean
): List<ValueSourceTracker<ParseError>> {
    val cwd = Paths.get(".").toAbsolutePath().normalize().toFile()
    val sourceCollection = newSourceCollectionFromFiles(listOf(cwd))

    val file = File(filename)
    val html = format == "html"
    val pair = sourceCollection.prettyPrint(file = file, html = html, doExpand = !noexpand)

    val fileContent = file.readText()
    val errors =
        pair.second.map {
            ValueSourceTracker(
                value = it,
                source =
                    SourceFile(
                        file = file,
                        content = fileContent,
                        validation = validationFailure(emptyList())),
                tracker = null)
        }

    val contentBuilder = StringBuilder()
    for (item in pair.first) {
        contentBuilder.append("<div class='mathlingua-top-level'>")
        contentBuilder.append(item)
        contentBuilder.append("</div><br/><br/>")
    }

    val text = buildStandaloneHtml(content = contentBuilder.toString())

    if (html && !stdout) {
        val fileToHtmlExt = File(file.parentFile, file.nameWithoutExtension + ".html")
        val relPath = fileToHtmlExt.relativePath(cwd)
        val outFile = File(getDocsDirectory(), relPath)
        outFile.parentFile.mkdirs()
        outFile.writeText(text)
        log("Wrote ${outFile.relativeTo(cwd)}")
    }

    if (stdout) {
        log(text)
    } else {
        log(getErrorOutput(errors, sourceCollection.size(), false))
    }

    return errors
}

private fun render(
    files: List<String>, format: String, stdout: Boolean, noexpand: Boolean
): List<ValueSourceTracker<ParseError>> {
    val cwd = Paths.get(".").toAbsolutePath().normalize().toFile()
    val sourceCollection = newSourceCollectionFromFiles(listOf(cwd))

    val docsDir = getDocsDirectory()
    docsDir.mkdirs()

    val contentDir = getContentDirectory()
    contentDir.mkdirs()

    val html = format == "html"
    val errors = mutableListOf<ValueSourceTracker<ParseError>>()

    val indexFileText = getIndexFileText(files, cwd, contentDir, html, noexpand, errors)

    if (html && !stdout) {
        val indexFile = File(docsDir, "index.html")
        indexFile.writeText(indexFileText)
        log("Wrote ${indexFile.relativeTo(cwd)}")
    }

    if (stdout) {
        log(indexFileText)
    } else {
        log(getErrorOutput(errors, sourceCollection.size(), false))
    }

    return errors
}

private class Watch :
    CliktCommand("Watches the working directory for changes and renders the code on file changes") {

    private fun isHidden(file: File): Boolean {
        if (file.name.startsWith(".")) {
            return true
        }

        val parent = file.parentFile ?: return false
        return isHidden(parent)
    }

    override fun run(): Unit =
        runBlocking {
            fun registerAll(file: File, watchService: WatchService) {
                if (file.isDirectory) {
                    file.walk().forEach {
                        val path = it.toPath()
                        if (it.isDirectory && !isHidden(it)) {
                            path.register(
                                watchService,
                                StandardWatchEventKinds.ENTRY_CREATE,
                                StandardWatchEventKinds.ENTRY_DELETE,
                                StandardWatchEventKinds.ENTRY_MODIFY)
                        }
                    }
                }
            }

            var watchService = FileSystems.getDefault().newWatchService()
            val cwd = Paths.get(".").toAbsolutePath().normalize().toFile()
            registerAll(cwd, watchService)

            log("Waiting for changes...")
            while (true) {
                var doRender = false
                val watchKey = watchService.take()
                for (event in watchKey.pollEvents()) {
                    val filename = event.context().toString()
                    if (!filename.endsWith(".html") || filename == "docs-home.html") {
                        doRender = true
                    }
                    if (event.kind() == StandardWatchEventKinds.ENTRY_CREATE ||
                        event.kind() == StandardWatchEventKinds.ENTRY_DELETE) {
                        watchService.close()
                        watchService = FileSystems.getDefault().newWatchService()
                        registerAll(cwd, watchService)
                    }
                }

                if (doRender) {
                    log("Change detected...")
                    render(
                        files = listOf(cwd.absolutePath),
                        format = "html",
                        stdout = false,
                        noexpand = false)
                    println()
                    println()
                }

                watchKey.reset()
            }
        }
}

private class Render :
    CliktCommand("Generates either HTML or MathLingua code with definitions expanded.") {
    private val file: List<String> by argument(
            help = "The *.math files and/or directories to process")
        .multiple(required = false)
    private val format by option(help = "Whether to generate HTML or Mathlingua.")
        .choice("html", "mathlingua")
        .default("html")
    private val noexpand: Boolean by option(
            help = "Specifies to not expand the contents of entries.")
        .flag()
    private val stdout: Boolean by option(
            help =
                "If specified, the rendered content will be printed to standard" +
                    "out.  Otherwise, it is written to a file with the same path as the input file except for a '.html' or " +
                    "'.out.math' extension.")
        .flag()

    override fun run(): Unit =
        runBlocking {
            val errors =
                if (file.size == 1) {
                    render(
                        filename = file[0], format = format, stdout = stdout, noexpand = noexpand)
                } else {
                    render(files = file, format = format, stdout = stdout, noexpand = noexpand)
                }
            exitProcess(
                if (errors.isEmpty()) {
                    0
                } else {
                    1
                })
        }
}

private fun getErrorOutput(
    errors: List<ValueSourceTracker<ParseError>>, numFilesProcessed: Int, json: Boolean
): String {
    val builder = StringBuilder()
    if (json) {
        builder.append("[")
    }
    val cwd = Paths.get(".").toAbsolutePath().normalize().toFile()
    for (i in errors.indices) {
        val err = errors[i]
        if (json) {
            builder.append("{")
            builder.append(
                "  \"file\": \"${err.source.file?.normalize()?.absolutePath?.jsonSanitize() ?: "None"}\",")
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
                    "${err.source.file?.relativePath(cwd) ?: "None"} (Line: ${err.value.row + 1}, Column: ${err.value.column + 1})\n"))
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

private fun File.relativePath(dir: File) =
    try {
        this.relativeTo(dir).path
    } catch (e: IllegalArgumentException) {
        this.path
    }

private fun maybePlural(text: String, count: Int) =
    if (count == 1) {
        text
    } else {
        "${text}s"
    }

// this value will be populated in main()
var helpText = ""

class Help : CliktCommand(help = "Show this message and exit") {
    override fun run() {
        log(helpText)
        exitProcess(0)
    }
}

class Clean : CliktCommand(help = "Delete the docs directory") {
    override fun run() {
        val docsDir = getDocsDirectory()
        val indexFile = File(docsDir, "index.html")
        if (!indexFile.exists()) {
            log("Nothing to clean")
            exitProcess(0)
        } else {
            if (indexFile.delete()) {
                log("Deleted $indexFile")
                exitProcess(0)
            } else {
                log("${bold(red("ERROR: "))} Failed to directory $docsDir")
                exitProcess(1)
            }
        }
    }
}

fun main(args: Array<String>) {
    val mlg = Mlg().subcommands(Help(), Check(), Clean(), Render(), Watch(), Version())
    helpText = mlg.getFormattedHelp()
    mlg.main(args)
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
"""

const val SHARED_CSS =
    """
    .content {
        margin-top: 1.5em;
        margin-bottom: 1em;
        font-size: 1em;
        width: 70%;
        margin-left: auto; /* for centering content */
        margin-right: auto; /* for centering content */
    }

    @media screen and (max-width: 400px) {
        .content {
            width: 90%;
        }
    }

    body {
        background-color: #fafafa;
        padding-bottom: 1em;
    }

    hr {
        border: 0.5px solid #efefef;
    }

    h1, h2, h3, h4 {
        color: #0055bb;
    }

    .mathlingua-top-level {
        background-color: white;
        border: solid;
        border-width: 1px;
        border-color: rgba(200, 200, 200);
        border-radius: 2px;
        box-shadow: rgba(0, 0, 0, 0.5) 0px 3px 10px,
                    inset 0  0 10px 0 rgba(200, 200, 200, 0.25);
        padding-top: 1.25em;
        padding-bottom: 1em;
        padding-left: 1.1em;
        padding-right: 1.1em;
        max-width: 90%;
        width: max-content;
        margin-left: auto; /* for centering content */
        margin-right: auto; /* for centering content */
    }

    .end-mathlingua-top-level {
        padding-top: 0.5em;
        margin: 0;
    }

    .mathlingua {
        font-family: monospace;
    }

    .mathlingua-header {
        color: #0055bb;
        text-shadow: 0px 1px 0px rgba(255,255,255,0), 0px 0.4px 0px rgba(0,0,0,0.2);
        font-size: 80%;
        font-family: Georgia, 'Times New Roman', Times, serif;
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
        display: block;
        margin: 0 0 -1em 0;
        padding: 0 0 0 2.5em;
        font-size: 80%;
        font-family: Georgia, 'Times New Roman', Times, serif;
    }

    .mathlingua-text-no-render {
        color: #000000;
        display: block;
        margin: 0 0 -1em 0;
        padding: 0 0 0 2.5em;
        font-size: 80%;
        font-family: Georgia, 'Times New Roman', Times, serif;
    }

    .mathlingua-url {
        color: #0000aa;
        text-decoration: none;
        display: block;
        margin: 0 0 -1em 0;
        padding: 0 0 0 2.5em;
        font-size: 80%;
        font-family: Georgia, 'Times New Roman', Times, serif;
    }

    .mathlingua-link {
        color: #0000aa;
        text-decoration: none;
    }

    .mathlingua-statement-no-render {
        color: #007377;
    }

    .mathlingua-statement-container {
        display: inline;
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

    .katex {
        font-size: 0.9em;
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
    homeHtml: String,
    searchIndexInitCode: String,
    allFileIds: List<String>,
    signatureToPathCode: String,
    pathToEntityList: String
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

            function view(path) {
                const content = document.getElementById('__main_content__');
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
                            el.className = 'mathlingua-top-level';
                            el.innerHTML = entityList[i];
                            content.appendChild(el);
                            content.appendChild(document.createElement('br'));
                            content.appendChild(document.createElement('br'));
                        }
                        render(content);
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
                view(INITIAL_SRC);
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
                view('');
            }
        </script>
        <style>
            $SHARED_CSS

            .topbar {
                display: block;
                height: 1.75em;
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

            .search-area {
                position: absolute;
                left: 50%;
                top: 50%;
                transform: translate(-50%, -50%);
                width: max-content;
            }

            .sidebar {
                height: 100%;
                width: 20%;
                position: fixed;
                z-index: 1;
                top: 1.75em;
                left: 0;
                background-color: #fefefe;
                border-right: solid;
                border-width: 1px;
                border-color: rgba(215, 215, 215);
                box-shadow: rgba(0, 0, 0, 0.2) 0px 3px 10px,
                            inset 0  0 10px 0 rgba(200, 200, 200, 0);
                overflow-x: scroll;
                padding-top: 25px;
                transition: 0.5s;
            }

            @media screen and (max-width: 500px) {
                .sidebar {
                    width: 0%;
                }
            }

            .sidebar a {
                text-decoration: none;
                color: #000000;
                display: block;
                transition: 0.3s;
                font-size: 80%;
                padding-left: 15px;
                padding-right: 0px;
                padding-top: 5px;
                padding-bottom: 5px;
                margin: 0px;
            }

            .search {
                border: solid;
                border-width: 1px;
                border-radius: 3px;
                border-color: #303030;
                line-height: 1.3em;
                background-color: #eeeeee;
            }

            .button {
                border: solid;
                border-width: 1px;
                border-radius: 3px;
                padding-left: 0.5em;
                padding-right: 0.5em;
                padding-top: 0.2em;
                padding-bottom: 0.2em;
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
                padding-right: 0;
                padding-bottom: 0;
                padding-left: 1em;
                padding-top: 1em;
                margin-left: 20%;
                margin-right: 0;
                margin-bottom: 0;
            }

            @media screen and (max-width: 500px) {
                #main {
                    margin-left: 0;
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
                <input type="search" id="search-input" class="search" aria-label="search">
                <button type="button" class="button" onclick="clearSearch()">Clear</button>
                <button type="button" class="button" onclick="search()">Search</button>
            </span>
        </div>

        <div id="sidebar" class="sidebar">
            <a id='home' onclick="view('home.html')" style="padding-top: 0;padding-bottom: 0;margin-top: 0;margin-bottom: 0;"><span style="padding-left: 0px;font-weight: bold;">Home</span></a>
            <hr>
            $fileListHtml
        </div>

        <div class='bottom-panel' id='__bottom_panel__'></div>
        <div class="content" id="__main_content__"></div>
    </body>
</html>
"""

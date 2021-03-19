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
import mathlingua.backend.SourceFile
import mathlingua.backend.ValueSourceTracker
import mathlingua.backend.isMathLinguaFile
import mathlingua.backend.newSourceCollectionFromFiles
import mathlingua.frontend.FrontEnd
import mathlingua.frontend.chalktalk.phase2.ast.clause.Identifier
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Text
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
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
    cwd: File,
    file: File,
    indent: Int,
    builder: StringBuilder,
    allFileIds: MutableList<String>,
    firstSrc: Array<String>
) {
    val childBuilder = StringBuilder()
    if (file.isDirectory) {
        val children = file.listFiles()
        if (children != null) {
            for (child in children) {
                buildFileList(cwd, child, indent + 12, childBuilder, allFileIds, firstSrc)
            }
        }
    }

    val isMathFile = file.isFile && file.extension == "math"
    if ((file.isDirectory && childBuilder.isNotEmpty()) || isMathFile) {
        val src = file.relativePath(cwd).replace(".math", ".html")
        if (isMathFile && firstSrc[0].isEmpty()) {
            firstSrc[0] = src
        }
        val cssBuilder = StringBuilder()
        cssBuilder.append("padding-left: ${indent}px;")
        if (file.isDirectory) {
            cssBuilder.append("font-weight: bold;")
        }
        val id = src.removeSuffix(".html")
        allFileIds.add(id)
        builder.append(
            "<a id='$id' onclick=\"view('$src')\"><span style=\"${cssBuilder}\">${file.name.removeSuffix(".math")}</span></a>")
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

private fun writeIndexFile(cwd: File, docsDir: File, contentDir: File): File {
    val allFileIds = mutableListOf<String>()
    val firstSrc = arrayOf("")
    val builder = StringBuilder()
    if (cwd.isDirectory) {
        val children = cwd.listFiles()
        if (children != null) {
            for (child in children) {
                buildFileList(cwd, child, 0, builder, allFileIds, firstSrc)
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
            searchIndexBuilder.append("const map$i = new Map();\n")
            val paths = pathToIndices.keys.toList()
            for (j in paths.indices) {
                val path = paths[j]
                val ids = pathToIndices[path]
                if (ids != null) {
                    searchIndexBuilder.append("const set${i}_$j = new Set();\n")
                    for (id in ids) {
                        searchIndexBuilder.append("set${i}_$j.add($id);\n")
                    }
                    searchIndexBuilder.append("map$i.set('$path', set${i}_$j);\n")
                }
            }
            searchIndexBuilder.append("index.set('$word', map$i);\n")
        }
    }
    searchIndexBuilder.append("return index;")

    val html =
        getIndexHtml(builder.toString(), searchIndexBuilder.toString(), allFileIds, firstSrc[0])
    val indexFile = File(docsDir, "index.html")
    indexFile.writeText(html)
    return indexFile
}

private fun render(
    files: List<String>, format: String, stdout: Boolean, noexpand: Boolean
): List<ValueSourceTracker<ParseError>> {
    val cwd = Paths.get(".").toAbsolutePath().normalize().toFile()
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

    val docsDir = getDocsDirectory()
    docsDir.mkdirs()

    val contentDir = getContentDirectory()
    contentDir.mkdirs()

    val html = format == "html"
    if (html && !stdout) {
        val indexFile = writeIndexFile(cwd, docsDir, contentDir)
        log("Wrote ${indexFile.relativeTo(cwd)}")
    }

    val errors = mutableListOf<ValueSourceTracker<ParseError>>()
    for (f in filesToProcess) {
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
        val ext =
            if (html) {
                ".html"
            } else {
                ".out.math"
            }
        val docRelFile = File(docsDir, f.relativePath(cwd))
        val docRelParent = docRelFile.parentFile
        val docRelName = docRelFile.nameWithoutExtension + ext
        val outFile = File(docRelParent, docRelName)
        write(cwd = cwd, content = pair.first, outFile = outFile, stdout = stdout)
    }

    if (!stdout) {
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
                    if (!filename.endsWith(".html")) {
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
            val errors = render(files = file, format = format, stdout = stdout, noexpand = noexpand)
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
        if (docsDir.deleteRecursively()) {
            log("Deleted directory $docsDir")
            exitProcess(0)
        } else {
            log("${bold(red("ERROR: "))} Failed to delete directory $docsDir")
            exitProcess(1)
        }
    }
}

fun main(args: Array<String>) {
    val mlg = Mlg().subcommands(Help(), Check(), Clean(), Render(), Watch(), Version())
    helpText = mlg.getFormattedHelp()
    mlg.main(args)
}

fun getIndexHtml(
    fileListHtml: String, searchIndexInitCode: String, allFileIds: List<String>, initialSrc: String
) =
    """
<html>
    <head>
        <meta name="viewport" content="width=100%, initial-scale=1.0">
        <meta content="text/html;charset=utf-8" http-equiv="Content-Type">
        <meta content="utf-8" http-equiv="encoding">
        <style>
            body {
                background-color: #fafafa;
                overflow: hidden;
            }

            .topbar {
                display: block;
                height: 1.75em;
                background-color: #ffffff;
                position: fixed;
                z-index: 2;
                top: 0;
                left: 0;
                width: 100%;
                border-width: 1px;
                border-color: rgba(200, 200, 200);
                box-shadow: rgba(0, 0, 0, 0.5) 0px 3px 10px,
                            inset 0  0 10px 0 rgba(200, 200, 200, 0.25);
                padding-top: 0.5em;
            }

            .search-area {
                margin-left: auto;
                margin-right: auto;
            }

            .sidebar {
                height: 100%;
                width: 20%;
                position: fixed;
                z-index: 1;
                top: 1.75em;
                left: 0;
                background-color: #ffffff;
                border-right: solid;
                border-width: 1px;
                border-color: rgba(200, 200, 200);
                box-shadow: rgba(0, 0, 0, 0.5) 0px 3px 10px,
                            inset 0  0 10px 0 rgba(200, 200, 200, 0.25);
                overflow-x: scroll;
                padding-top: 25px;
                transition: 0.5s;
            }

            @media screen and (max-width: 400px) {
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

            .closeButton {
                text-decoration: none;
                color: black;
                margin-left: 0.75em;
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

            @media screen and (max-width: 400px) {
                #main {
                    margin-left: 0%;
                }
            }

            #__content__frame__ {
                border: none;
                width: 100%;
                height: 100%;
                margin: 0;
                padding: 0;
            }
        </style>
        <script>
            const ALL_FILE_IDS = [${allFileIds.joinToString(",") { "'$it'" }}];
            const INITIAL_SRC = "$initialSrc";
            const SEARCH_INDEX = (function() {
                $searchIndexInitCode
            })();

            function forMobile() {
                return window?.screen?.width <= 400;
            }

            let open = !forMobile();

            function toggleSidePanel() {
                if (open) {
                    document.getElementById('sidebar').style.width = '0';
                    document.getElementById('main').style.marginLeft = '0';
                } else {
                    let margin = forMobile() ? '50%' : '20%';
                    document.getElementById('sidebar').style.width = margin;
                    document.getElementById('main').style.marginLeft = margin;
                }
                open = !open;
            }

                        function view(path) {
                const content = document.getElementById('__content__frame__');
                if (content) {
                    content.src = path;
                    for (const path of ALL_FILE_IDS) {
                        const el = document.getElementById(path);
                        if (el) {
                            el.style.fontStyle = 'normal';
                        }
                    }
                    const id = path.replace(/\.html.*/, '');
                    const selectedEntry = document.getElementById(id);
                    if (selectedEntry) {
                        selectedEntry.style.fontStyle = 'italic';
                    }
                }
            }

            function clearSearch() {
                for (const id of ALL_FILE_IDS) {
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

            function search(terms) {
                const el = document.getElementById('search-input');
                if (el) {
                    const term = el.value;
                    if (!term) {
                        clearSearch();
                        return;
                    }

                    view('');
                    const pathsToIndices = SEARCH_INDEX.get(term);
                    if (!pathsToIndices) {
                        alert('No results found');
                        return;
                    }

                    let firstPath = null;
                    const pathToNewPath = new Map();
                    for (const [path, ids] of pathsToIndices) {
                        let newPath = path + '.html';
                        if (ids.size > 0) {
                            newPath += '?';
                            let isFirst = true;
                            for (const id of ids) {
                                if (!isFirst) {
                                    newPath += '&'
                                }
                                isFirst = false;
                                newPath += "show=" + id;
                            }
                            if (!firstPath) {
                                firstPath = newPath;
                            }
                            pathToNewPath.set(path, newPath);
                        }
                    }

                    for (const id of ALL_FILE_IDS) {
                        const el = document.getElementById(id);
                        if (el) {
                            if (pathToNewPath.has(id)) {
                                el.style.display = 'block';
                                el.setAttribute('onclick', "view('" + pathToNewPath.get(id) + "')");
                            } else {
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
            }
        </script>
    </head>
    <body id="main" onload="initPage()">
        <div id="top-bar" class="topbar">
            <a id="closeButton" class="closeButton" onclick="toggleSidePanel()">&#x2630;</a>
            <span class="search-area">
                <input type="search" id="search-input" aria-label="search">
                <button type="button" onclick="clearSearch()">Clear</button>
                <button type="button" onclick="search()">Search</button>
            </span>
        </div>

        <div id="sidebar" class="sidebar">
            $fileListHtml
        </div>

        <iframe id="__content__frame__" src="$initialSrc"></iframe>
    </body>
</html>
"""

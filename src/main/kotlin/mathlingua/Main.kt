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
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.validationFailure

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
    cwd: File, file: File, indent: Int, builder: StringBuilder, firstSrc: Array<String>
) {
    val childBuilder = StringBuilder()
    if (file.isDirectory) {
        val children = file.listFiles()
        if (children != null) {
            for (child in children) {
                buildFileList(cwd, child, indent + 12, childBuilder, firstSrc)
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
        builder.append(
            "<a onclick=\"view('$src')\"><span style=\"${cssBuilder}\">${file.name.removeSuffix(".math")}</span></a>")
        builder.append(childBuilder.toString())
    }
}

private fun writeIndexFile(cwd: File, docsDir: File): File {
    val firstSrc = arrayOf("")
    val builder = StringBuilder()
    if (cwd.isDirectory) {
        val children = cwd.listFiles()
        if (children != null) {
            for (child in children) {
                buildFileList(cwd, child, 0, builder, firstSrc)
            }
        }
    }
    val html = getIndexHtml(builder.toString(), firstSrc[0])
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

    val html = format == "html"
    if (html && !stdout) {
        val indexFile = writeIndexFile(cwd, docsDir)
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

fun getIndexHtml(fileListHtml: String, initialSrc: String) =
    """
<html>
    <head>
        <style>
            body {
                background-color: #fafafa;
            }

            .sidebar {
                height: 100%;
                width: 15%;
                position: fixed;
                z-index: 1;
                top: 0;
                left: 0;
                background-color: #ffffff;
                border-right: solid;
                border-width: 1px;
                border-color: #e5e5e5;
                box-shadow: rgba(0, 0, 0, 0.2) 0px 0px 10px;
                overflow-x: scroll;
                padding-top: 25px;
                transition: 0.5s;
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
            }

            #main {
                transition: margin-left .5s;
                padding: 20px;
                margin-left: 15%;
            }

            #content {
                border: none;
                width: 100%;
                height: 100%;
            }
        </style>
        <script>
            let open = true;

            function toggleSidePanel() {
                if (open) {
                    document.getElementById('sidebar').style.width = '0';
                    document.getElementById('main').style.marginLeft = '0';
                    document.getElementById('closeButton').textContent = '›';
                } else {
                    document.getElementById('sidebar').style.width = '15%';
                    document.getElementById('main').style.marginLeft = '15%';
                    document.getElementById('closeButton').textContent = '‹';
                }
                open = !open;
            }

            function view(path) {
                const content = document.getElementById('content');
                if (content) {
                    content.src = path;
                }
            }
        </script>
    </head>
    <body id="main">
        <div id="sidebar" class="sidebar">
            $fileListHtml
        </div>

        <a href="javascript:void(0)" id="closeButton" class="closeButton" onclick="toggleSidePanel()">&#x2039;</a>

        <iframe id="content" src="$initialSrc"></iframe>
    </body>
</html>
"""

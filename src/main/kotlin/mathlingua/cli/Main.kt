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

package mathlingua.cli

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.subcommands
import com.github.ajalt.clikt.output.TermUi
import com.github.ajalt.clikt.parameters.arguments.argument
import com.github.ajalt.clikt.parameters.arguments.multiple
import com.github.ajalt.clikt.parameters.arguments.optional
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import java.io.File
import java.nio.file.FileSystems
import java.nio.file.Paths
import java.nio.file.StandardWatchEventKinds
import java.nio.file.WatchService
import kotlin.system.exitProcess
import kotlinx.coroutines.runBlocking

private fun getCwd() = Paths.get(".").toAbsolutePath().normalize().toFile()

private fun cwdParts() = getCwd().absolutePath.split(File.separator)

private fun toVirtualFile(fs: VirtualFileSystem, path: String): VirtualFile {
    val file = File(path).normalize().absoluteFile
    val relPath = file.toRelativeString(getCwd()).split(File.separator)
    return if (file.isDirectory) {
        fs.getDirectory(relPath)
    } else {
        fs.getFile(relPath)
    }
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
            val fs = newDiskFileSystem(cwdParts())
            exitProcess(
                Mathlingua.check(
                    fs = fs,
                    logger = TermUiLogger(termUi = TermUi),
                    files = file.toList().map { toVirtualFile(fs, it) },
                    json = json))
        }
}

private class Version : CliktCommand(help = "Prints the tool and MathLingua language version.") {
    override fun run() {
        exitProcess(Mathlingua.version(logger = TermUiLogger(termUi = TermUi)))
    }
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
            val fs = newDiskFileSystem(cwdParts())
            val logger = TermUiLogger(termUi = TermUi)

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

            // do an initial render to ensure the docs directory is up-to-date
            // even before any changes occur
            Mathlingua.render(
                fs = fs,
                logger = logger,
                file = null,
                noexpand = false,
                stdout = false,
                raw = false)
            logger.log("")

            logger.log("Waiting for changes...")
            logger.log("")

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
                    logger.log("Change detected...")
                    Mathlingua.render(
                        fs = fs,
                        logger = logger,
                        file = null,
                        noexpand = false,
                        stdout = false,
                        raw = false)
                    logger.log("")
                }

                watchKey.reset()
            }
        }
}

private class Render : CliktCommand("Generates HTML code with definitions expanded.") {
    private val file: String? by argument(
            help =
                "If specified, the .math file to render as an HTML document.  Otherwise, all .math files " +
                    "in the 'contents' directory will be rendered into a single dynamic HTML document.")
        .optional()
    private val noexpand: Boolean by option(
            help =
                "Specifies to not expand the contents of entries using the 'written' form of definitions.")
        .flag()
    private val stdout: Boolean by option(
            help =
                "If specified, the rendered content will be printed to standard " +
                    "out.  Otherwise, it is written to a file in the `docs` directory with the same path as the " +
                    "input file except for a '.html' extension if the FILE argument is specified.  Otherwise, the " +
                    "content is written to an `index.html` file in the `docs` directory.")
        .flag()
    private val raw: Boolean by option(
            help =
                "If specified with a single file, the raw HTML will be rendered excluding any " +
                    "script or style tages.  It is an error to specify this flag without specify a specific " +
                    "file to render.")
        .flag()

    override fun run(): Unit =
        runBlocking {
            val fs = newDiskFileSystem(cwdParts())
            exitProcess(
                Mathlingua.render(
                    fs = fs,
                    logger = TermUiLogger(termUi = TermUi),
                    file =
                        if (file == null) {
                            null
                        } else {
                            toVirtualFile(fs, file!!)
                        },
                    noexpand = noexpand,
                    stdout = stdout,
                    raw = raw))
        }
}

// this value will be populated in main()
var helpText = ""

class Help : CliktCommand(help = "Show this message and exit") {
    override fun run() {
        val logger = TermUiLogger(termUi = TermUi)
        logger.log(helpText)
        exitProcess(0)
    }
}

class Clean : CliktCommand(help = "Delete the docs directory") {
    override fun run() {
        val fs = newDiskFileSystem(cwdParts())
        val logger = TermUiLogger(termUi = TermUi)
        exitProcess(Mathlingua.clean(fs, logger))
    }
}

fun main(args: Array<String>) {
    val mlg = Mlg().subcommands(Help(), Check(), Clean(), Render(), Watch(), Version())
    helpText = mlg.getFormattedHelp()
    mlg.main(args)
}

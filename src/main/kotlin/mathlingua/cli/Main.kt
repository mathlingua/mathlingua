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
import com.github.ajalt.clikt.parameters.options.default
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.types.int
import kotlin.system.exitProcess
import kotlinx.coroutines.runBlocking
import mathlingua.newDiskFileSystem

private class Mlg : CliktCommand() {
    override fun run() = Unit
}

private class Check : CliktCommand(help = "Check input files for errors") {
    private val file: List<String> by argument(
            help = "The *.math files and/or directories to process")
        .multiple(required = false)
    private val json: Boolean by option(help = "Output the results in JSON format").flag()

    override fun run(): Unit =
        runBlocking {
            val fs = newDiskFileSystem()
            exitProcess(
                Mathlingua.check(
                    fs = fs,
                    logger = TermUiLogger(termUi = TermUi),
                    files = file.toList().map { fs.getFileOrDirectory(it) },
                    json = json))
        }
}

private class Edit :
    CliktCommand(
        help =
            "Starts a web app on the specified port (defaults to 8080) that " +
                "allows the MathLingua files to be viewed and edited.  Open " +
                "localhost:<port> (i.e. localhost:8080) in your web browser to " +
                "access the app.") {
    private val port: Int by option(help = "The port to listen on").int().default(8080)

    override fun run() {
        val fs = newDiskFileSystem()
        Mathlingua.serve(fs = fs, logger = TermUiLogger(termUi = TermUi), port = port)
    }
}

private class Version : CliktCommand(help = "Prints the tool and MathLingua language version") {
    override fun run() {
        exitProcess(Mathlingua.version(logger = TermUiLogger(termUi = TermUi)))
    }
}

private class Export : CliktCommand(help = "Exports MathLingua files to static HTML files") {
    private val file: String? by argument(
            help =
                "If specified, the .math file to render as an HTML document.  Otherwise, all .math files " +
                    "in the 'contents' directory will be rendered into a single dynamic HTML document.")
        .optional()
    private val noexpand: Boolean by option(
            help =
                "Specifies to not expand the contents of entries using the 'written' form of definitions")
        .flag()
    private val raw: Boolean by option(
            help =
                "If specified with a single file, the raw HTML will be rendered excluding any " +
                    "script or style tages.  It is an error to specify this flag without specify a specific " +
                    "file to render.")
        .flag()
    private val stdout: Boolean by option(
            help =
                "If specified, the HTML contents will be written to standard output instead of" +
                    "to files in the 'exported' directory")
        .flag()

    override fun run() {
        if (file == "whte_rbt.obj") {
            println("Jurassic Park, System Security Interface")
            Thread.sleep(700)
            println("Version 4.0.5, Alpha E")
            Thread.sleep(700)
            println("Ready...")
            Thread.sleep(700)
            print("> ")
            "access security".forEach {
                Thread.sleep(100)
                print(it)
            }
            println()
            Thread.sleep(700)
            println("access: PERMISSION DENIED.")
            print("> ")
            "access security grid".forEach {
                Thread.sleep(100)
                print(it)
            }
            println()
            Thread.sleep(700)
            println("access: PERMISSION DENIED.")
            print("> ")
            "access main security grid".forEach {
                Thread.sleep(100)
                print(it)
            }
            println()
            Thread.sleep(1000)
            print("access: PERMISSION DENIED.")
            Thread.sleep(700)
            println("...and....")
            Thread.sleep(1000)
            while (true) {
                println("YOU DIDN'T SAY THE MAGIC WORD!")
                Thread.sleep(50)
            }
        }

        runBlocking {
            val fs = newDiskFileSystem()
            exitProcess(
                Mathlingua.export(
                    fs = fs,
                    logger = TermUiLogger(termUi = TermUi),
                    file =
                        if (file == null) {
                            null
                        } else {
                            fs.getFileOrDirectory(file!!)
                        },
                    stdout = stdout,
                    noExpand = noexpand,
                    raw = raw))
        }
    }
}

private class Render :
    CliktCommand(help = "Generates a static website for exploring the MathLingua content") {
    override fun run() {
        runBlocking {
            val fs = newDiskFileSystem()
            exitProcess(Mathlingua.render(fs = fs, logger = TermUiLogger(termUi = TermUi)))
        }
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

class Clean : CliktCommand(help = "Deletes generated HTML files") {
    override fun run() {
        val fs = newDiskFileSystem()
        val logger = TermUiLogger(termUi = TermUi)
        exitProcess(Mathlingua.clean(fs, logger))
    }
}

fun main(args: Array<String>) {
    val mlg = Mlg().subcommands(Help(), Check(), Clean(), Render(), Export(), Edit(), Version())
    helpText = mlg.getFormattedHelp()
    mlg.main(args)
}

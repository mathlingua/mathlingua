/*
 * Copyright 2019 The MathLingua Authors
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

private class Check : CliktCommand(help = "Check the MathLingua files for errors") {
    private val file: List<String> by argument(
            help = "The *.math files and/or directories to process")
        .multiple(required = false)
    private val json: Boolean by option(help = "Output the results in JSON format").flag()

    override fun run(): Unit =
        runBlocking {
            if (file.size == 1 && file[0] == "whte_rbt.obj") {
                whteRbtObj()
                exitProcess(0)
            }

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
            "Start a web app on the specified port (defaults to 8080) that " +
                "allows the MathLingua files to be viewed as well as edited.  Open " +
                "localhost:<port> (i.e. localhost:8080) in your web browser to " +
                "access the app.") {
    private val port: Int by option(help = "The port to listen on").int().default(8080)

    override fun run() {
        val logger = TermUiLogger(termUi = TermUi)
        val fs = newDiskFileSystem()
        Mathlingua.serve(fs = fs, logger = logger, port = port) {
            val url = "http://localhost:${port}"
            try {
                val osName = System.getProperty("os.name").lowercase()
                if (osName.contains("win")) {
                    ProcessBuilder("rundll32", "url.dll,FileProtocolHandler", url)
                } else
                    if (osName.contains("mac")) {
                            ProcessBuilder("open", url)
                        } else {
                            ProcessBuilder("xdg-open", url)
                        }
                        .start()
            } catch (e: Exception) {
                logger.error("Failed to automatically open $url")
                if (e.message != null) {
                    logger.error(e.message!!)
                }
            }
        }
    }
}

private class Version : CliktCommand(help = "Print the MathLingua version") {
    override fun run() {
        exitProcess(Mathlingua.version(logger = TermUiLogger(termUi = TermUi)))
    }
}

private class Document :
    CliktCommand(
        help =
            "Generate a read-only web app in the 'docs' directory for exploring the MathLingua content") {
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

class Clean : CliktCommand(help = "Delete generated HTML files") {
    override fun run() {
        val fs = newDiskFileSystem()
        val logger = TermUiLogger(termUi = TermUi)
        exitProcess(Mathlingua.clean(fs, logger))
    }
}

class Versions : CliktCommand(help = "Lists available MathLingua versions") {
    override fun run() {
        val logger = TermUiLogger(termUi = TermUi)
        logger.log(
            "This functionality is not available when the MathLingua jar " +
                "file is run directly.  Use the mlg executable instead.")
        exitProcess(1)
    }
}

class Update : CliktCommand(help = "Updates to the specified version of MathLingua") {
    private val version: String by option(
            help =
                "The MathLingua version to update to " +
                    "or 'latest' to update to the latest version.  See the 'versions' command for " +
                    "available versions.")
        .default("latest")

    override fun run() {
        val logger = TermUiLogger(termUi = TermUi)
        logger.log(
            "This functionality is not available when the MathLingua jar " +
                "file is run directly.  Use the mlg executable instead.")
        exitProcess(1)
    }
}

private fun whteRbtObj() {
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

fun main(args: Array<String>) {
    val mlg =
        Mlg()
            .subcommands(
                Check(), Clean(), Document(), Edit(), Help(), Update(), Version(), Versions())
    helpText = mlg.getFormattedHelp()
    try {
        mlg.main(args)
    } catch (e: Exception) {
        // handle the case when a port is already in use as an error that doesn't
        // cause the stack trace to be displayed
        if (e.message != null && (e.message!!).contains("Port already in use")) {
            println(e.message)
        } else {
            throw e
        }
    }
}

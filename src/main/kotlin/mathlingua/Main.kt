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
import java.nio.file.Paths
import kotlinx.coroutines.runBlocking
import mathlingua.backend.BackEnd
import mathlingua.backend.SourceFile
import mathlingua.backend.ValueSourceTracker
import mathlingua.backend.isMathLinguaFile
import mathlingua.backend.newSourceCollectionFromFiles
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.validationFailure

const val TOOL_VERSION = "0.10"

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
        }
}

private class Version : CliktCommand(help = "Prints the tool and MathLingua language version.") {
    override fun run() {
        log("MathLingua CLI Version:      $TOOL_VERSION")
        log("MathLingua Language Version: $MATHLINGUA_VERSION")
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
            val cwd = Paths.get(".").toAbsolutePath().normalize().toFile()
            val sourceCollection = newSourceCollectionFromFiles(listOf(cwd))
            val filesToProcess = mutableListOf<File>()
            val inputFiles =
                if (file.isEmpty()) {
                    listOf(cwd)
                } else {
                    file.map { File(it) }
                }
            for (f in inputFiles) {
                if (f.isDirectory) {
                    filesToProcess.addAll(f.walk().filter { isMathLinguaFile(it) })
                } else if (isMathLinguaFile(f)) {
                    filesToProcess.add(f)
                }
            }

            val html = format == "html"
            val errors = mutableListOf<ValueSourceTracker<ParseError>>()
            for (f in filesToProcess) {
                val pair = sourceCollection.prettyPrint(file = f, html = html, doExpand = !noexpand)
                errors.addAll(
                    pair.second.map {
                        ValueSourceTracker(
                            value = it,
                            source =
                                SourceFile(
                                    file = f,
                                    content = "",
                                    validation = validationFailure(emptyList())),
                            tracker = null)
                    })
                write(content = pair.first, fileBeingProcessed = f, stdout = stdout, html = html)
            }

            if (!stdout) {
                log(getErrorOutput(errors, sourceCollection.size(), false))
            }
        }

    private fun write(content: String, fileBeingProcessed: File, stdout: Boolean, html: Boolean) {
        if (stdout) {
            log(content)
        } else {
            val ext =
                if (html) {
                    ".html"
                } else {
                    ".out.math"
                }
            val outFile =
                File(fileBeingProcessed.parentFile, fileBeingProcessed.nameWithoutExtension + ext)
            outFile.writeText(content)
            log("Wrote ${outFile.normalize().canonicalPath}")
        }
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
    }
}

fun main(args: Array<String>) {
    val mlg = Mlg().subcommands(Help(), Check(), Render(), Version())
    helpText = mlg.getFormattedHelp()
    mlg.main(args)
}

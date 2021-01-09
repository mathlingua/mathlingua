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
import java.nio.file.Paths
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.runBlocking
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.FoundationGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.mutually.MutuallyGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.support.Location
import mathlingua.support.ParseError
import mathlingua.support.ValidationFailure
import mathlingua.support.ValidationSuccess

const val TOOL_VERSION = "0.8"

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
                SourceCollection(
                    if (file.isEmpty()) {
                        listOf(Paths.get(".").toAbsolutePath().normalize().toFile())
                    } else {
                        file.map { File(it) }
                    })
            val errors = mutableListOf<ValueSourceTracker<ParseError>>()
            errors.addAll(sourceCollection.getParseErrors())
            errors.addAll(
                sourceCollection.getUndefinedSignatures().map {
                    ValueSourceTracker(
                        source = it.source,
                        tracker = it.tracker,
                        value =
                            ParseError(
                                message = "Undefined signature '${it.value.form}'",
                                row = it.value.location.row,
                                column = it.value.location.column))
                })
            errors.addAll(
                sourceCollection.getDuplicateDefinedSignatures().map {
                    ValueSourceTracker(
                        source = it.source,
                        tracker = it.tracker,
                        value =
                            ParseError(
                                message = "Duplicate defined signature '${it.value.form}'",
                                row = it.value.location.row,
                                column = it.value.location.column))
                })
            errors.addAll(sourceCollection.findInvalidTypes())
            log(getErrorOutput(errors, json))
        }
}

private class DuplicateContent :
    CliktCommand(name = "dup-content", help = "Identifies duplicate content.") {
    private val file: List<String> by argument(
            help = "The *.math files and/or directories to process")
        .multiple(required = false)
    private val json: Boolean by option(help = "Output the results in JSON format.").flag()

    override fun run() {
        val sourceCollection =
            SourceCollection(
                if (file.isEmpty()) {
                    listOf(Paths.get(".").toAbsolutePath().normalize().toFile())
                } else {
                    file.map { File(it) }
                })
        val errors = mutableListOf<ValueSourceTracker<ParseError>>()
        for (grp in sourceCollection.getDuplicateContent()) {
            val location = grp.tracker?.getLocationOf(grp.value) ?: Location(row = -1, column = -1)
            errors.add(
                ValueSourceTracker(
                    source = grp.source,
                    tracker = grp.tracker,
                    value =
                        ParseError(
                            message = "Duplicate content detected",
                            row = location.row,
                            column = location.column)))
        }
        log(getErrorOutput(errors, json))
    }
}

private class DuplicateSignatures :
    CliktCommand(name = "dup-sig", help = "Identifies duplicate signature definitions.") {
    private val file: List<String> by argument(
            help = "The *.math files and/or directories to process")
        .multiple(required = false)
    private val json: Boolean by option(help = "Output the results in JSON format.").flag()

    override fun run() {
        val sourceCollection =
            SourceCollection(
                if (file.isEmpty()) {
                    listOf(Paths.get(".").toAbsolutePath().normalize().toFile())
                } else {
                    file.map { File(it) }
                })
        val errors = mutableListOf<ValueSourceTracker<ParseError>>()
        errors.addAll(
            sourceCollection.getDuplicateDefinedSignatures().map {
                ValueSourceTracker(
                    source = it.source,
                    tracker = it.tracker,
                    value =
                        ParseError(
                            message = "Duplicate defined signature '${it.value.form}'",
                            row = it.value.location.row,
                            column = it.value.location.column))
            })
        log(getErrorOutput(errors, json))
    }
}

private class UndefinedSignatures :
    CliktCommand(
        name = "undef-sig",
        help = "Identifies command that have been used but have not been defined.") {
    private val file: List<String> by argument(
            help = "The *.math files and/or directories to process")
        .multiple(required = false)
    private val json: Boolean by option(help = "Output the results in JSON format.").flag()

    override fun run() {
        val sourceCollection =
            SourceCollection(
                if (file.isEmpty()) {
                    listOf(Paths.get(".").toAbsolutePath().normalize().toFile())
                } else {
                    file.map { File(it) }
                })
        val errors = mutableListOf<ValueSourceTracker<ParseError>>()
        errors.addAll(
            sourceCollection.getUndefinedSignatures().map {
                ValueSourceTracker(
                    source = it.source,
                    tracker = it.tracker,
                    value =
                        ParseError(
                            message = "Undefined signature '${it.value.form}'",
                            row = it.value.location.row,
                            column = it.value.location.column))
            })
        log(getErrorOutput(errors, json))
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
    private val file: String by argument(help = "The .math file to process")
    private val filter: String? by option(
        help =
            "If a file path contains this string(s) it will be " +
                "processed for definitions.  Separate multiple filters with commas.")
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

    override fun run() =
        runBlocking {
            val f = File(file)
            if (f.isFile) {
                processFile(f)
            } else {
                f.walk().filter { it.isFile && it.name.endsWith(".math") }.forEach {
                    processFile(it)
                }
            }
        }

    private fun write(content: String, fileBeingProcessed: File) {
        if (stdout) {
            log(content)
        } else {
            val ext =
                if (format == "html") {
                    ".html"
                } else {
                    ".out.math"
                }
            val outFile =
                File(fileBeingProcessed.parentFile, fileBeingProcessed.nameWithoutExtension + ext)
            outFile.writeText(content)
            log("Wrote ${outFile.absolutePath}")
        }
    }

    private suspend fun processFile(fileToProcess: File) {
        when (val validation = MathLingua.parse(fileToProcess.readText())
        ) {
            is ValidationFailure -> {
                when (format) {
                    "html" -> {
                        val builder = StringBuilder()
                        builder.append(
                            "<html><head><style>.content { font-size: 1em; }" +
                                "</style></head><body class='content'><ul>")
                        for (err in validation.errors) {
                            builder.append(
                                "<li><span style='color: #e61919;'>ERROR:</span> " +
                                    "${err.message} (${err.row + 1}, ${err.column + 1})</li>")
                        }
                        builder.append("</ul></body></html>")
                        write(builder.toString(), fileToProcess)
                    }
                    "mathlingua" -> {
                        val builder = StringBuilder()
                        for (err in validation.errors) {
                            builder.append(
                                "ERROR: ${err.message} (${err.row + 1}, ${err.column + 1})\n")
                        }
                        write(builder.toString(), fileToProcess)
                    }
                }
            }
            is ValidationSuccess -> {
                val defines = mutableListOf<DefinesGroup>()
                val states = mutableListOf<StatesGroup>()
                val foundations = mutableListOf<FoundationGroup>()
                val mutuallyGroups = mutableListOf<MutuallyGroup>()

                val cwd = Paths.get(".").toAbsolutePath().normalize().toFile()
                val filterItems =
                    (filter ?: "").split(",").map { it.trim() }.filter { it.isNotEmpty() }

                if (!noexpand) {
                    val allFiles =
                        cwd.walk()
                            .filter { it.isFile && it.name.endsWith(".math") }
                            .filter {
                                if (filterItems.isEmpty()) {
                                    return@filter true
                                }

                                var matchesOne = false
                                for (f in filterItems) {
                                    if (it.absolutePath.contains(f)) {
                                        matchesOne = true
                                        break
                                    }
                                }
                                matchesOne
                            }
                            .toList()

                    awaitAll(
                        *allFiles
                            .map {
                                GlobalScope.async {
                                    val result = MathLingua.parse(it.readText())
                                    if (result is ValidationSuccess) {
                                        defines.addAll(result.value.defines())
                                        states.addAll(result.value.states())
                                        foundations.addAll(result.value.foundations())
                                        mutuallyGroups.addAll(result.value.mutually())
                                    }
                                }
                            }
                            .toTypedArray())
                }

                val content =
                    MathLingua.prettyPrint(
                        node = validation.value,
                        defines = defines,
                        states = states,
                        foundations = foundations,
                        mutuallyGroups = mutuallyGroups,
                        html = format == "html")

                write(content, fileToProcess)
            }
        }
    }
}

private fun getErrorOutput(errors: List<ValueSourceTracker<ParseError>>, json: Boolean): String {
    val builder = StringBuilder()
    if (json) {
        builder.append("[")
    }
    for (i in errors.indices) {
        val err = errors[i]
        if (json) {
            builder.append("{")
            builder.append(
                "  \"file\": \"${err.source.file.normalize().absolutePath.jsonSanitize()}\",")
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
                    "${err.source.file} (Line: ${err.value.row + 1}, Column: ${err.value.column + 1})\n"))
            builder.append(err.value.message.trim())
            if (i != errors.size - 1) {
                builder.append("\n\n")
            } else {
                builder.append("\n")
            }
        }
    }

    if (json) {
        builder.append("]")
    }

    return builder.toString()
}

fun main(args: Array<String>) =
    Mlg()
        .subcommands(
            Check(),
            Render(),
            DuplicateContent(),
            DuplicateSignatures(),
            UndefinedSignatures(),
            Version())
        .main(args)

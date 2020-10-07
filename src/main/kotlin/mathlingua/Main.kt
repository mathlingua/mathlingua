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
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import mathlingua.common.MathLingua
import mathlingua.common.support.ParseError
import mathlingua.common.support.ValidationFailure
import mathlingua.common.support.ValidationSuccess
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.common.chalktalk.phase2.ast.group.toplevel.represents.RepresentsGroup
import java.io.File
import java.nio.file.Paths
import kotlin.system.exitProcess
import kotlinx.coroutines.runBlocking

private const val TOOL_VERSION = "0.7"
private const val MATHLINGUA_VERSION = "0.7"

private fun bold(text: String) = "\u001B[1m$text\u001B[0m"
private fun green(text: String) = "\u001B[32m$text\u001B[0m"
private fun red(text: String) = "\u001B[31m$text\u001B[0m"
private fun yellow(text: String) = "\u001B[33m$text\u001B[0m"

private enum class OutputType {
    Json,
    TestCase,
    UserFocused
}

private fun error(msg: String) = TermUi.echo(message = msg, err = true)
private fun log(msg: String) = TermUi.echo(message = msg, err = false)

private enum class MessageType(val text: String) {
    Error("ERROR"),
    Warning("WARNING")
}

private fun String.jsonSanitize() =
    this.replace("\\", "\\\\")
        .replace("\b", "\\b")
        .replace("\n", "\\n")
        .replace("\t", "\\t")
        .replace("\r", "\\r")
        .replace("\"", "\\\"")

private data class MessageInfo(
    val file: File,
    val messageType: MessageType,
    val message: String,
    val failedLine: String,
    val row: Int,
    val column: Int
) {
    fun toString(type: OutputType): String {
        val builder = StringBuilder()
        when (type) {
            OutputType.Json -> {
                builder.append("{")
                builder.append("  \"file\": \"${file.normalize().absolutePath.jsonSanitize()}\",")
                builder.append("  \"type\": \"${messageType.text.jsonSanitize()}\",")
                builder.append("  \"message\": \"${message.jsonSanitize()}\",")
                builder.append("  \"failedLine\": \"${failedLine.jsonSanitize()}\",")
                builder.append("  \"row\": $row,")
                builder.append("  \"column\": $column")
                builder.append("}")
            }
            OutputType.TestCase -> {
                builder.append("Row: $row\n")
                builder.append("Column: $column\n")
                builder.append("Message:\n")
                builder.append(message)
                builder.append("\nEndMessage:\n")
            }
            else -> {
                builder.append(bold(red("${messageType.text}: ")))
                builder.append(bold("$file\n"))
                builder.append(failedLine.trim())
                builder.append("\n")
                builder.append(message.trim())
                builder.append("\n")
            }
        }
        return builder.toString()
    }
}

private suspend fun runMlg(
    printJson: Boolean,
    failOnWarnings: Boolean,
    generateTestCases: Boolean,
    inputs: List<File>,
    output: String,
    expand: Boolean
): Int {
    val files = mutableListOf<File>()
    for (curFile in inputs) {
        if (curFile.isFile && curFile.name.endsWith(".math")) {
            files.add(curFile)
        } else if (curFile.isDirectory) {
            files.addAll(curFile.walk()
                    .filter { it.isFile && it.name.endsWith(".math") })
        }
    }

    val allSignatures = mutableSetOf<String>()
    val defSignatures = mutableSetOf<String>()

    val allErrorInfo = mutableListOf<MessageInfo>()
    awaitAll(*files.map {
        GlobalScope.async {
            allErrorInfo.addAll(processFile(it, allSignatures, defSignatures))
        }
    }.toTypedArray())

    if (printJson && generateTestCases) {
        error("Cannot specify to output both to json and to generate test cases")
        return 1
    }

    val outputBuilder = StringBuilder()

    when {
        output.toLowerCase() == "html" || output.toLowerCase() == "mathlingua" -> {
            val contents = StringBuilder()
            for (f in files) {
                contents.append(f.readText())
                contents.append("\n\n\n")
            }
            when (val validation = MathLingua.parse(contents.toString())) {
                is ValidationFailure -> {
                    error("Could not output the contents due to the errors:")
                    for (err in validation.errors) {
                        error(
                            "  ${bold(
                                red(
                                    "ERROR(${err.row}, ${err.column}):  "
                                )
                            )}${err.message}"
                        )
                    }
                }
                is ValidationSuccess -> {
                    val doc = validation.value
                    val defines = if (expand) {
                        doc.defines()
                    } else {
                        emptyList()
                    }

                    val represents = if (expand) {
                        doc.represents()
                    } else {
                        emptyList()
                    }

                    outputBuilder.append(MathLingua.prettyPrint(
                            node = doc,
                            defines = defines,
                            represents = represents,
                            html = output.toLowerCase() == "html"))
                }
            }
        }
        printJson -> {
            log("[")
            for (i in 0 until allErrorInfo.size) {
                print(allErrorInfo[i].toString(OutputType.Json))
                if (i != allErrorInfo.size - 1) {
                    log(",")
                }
            }
            log("]")
        }
        generateTestCases -> {
            for (err in allErrorInfo) {
                log(err.toString(OutputType.TestCase))
                log("")
            }
        }
        else -> {
            for (err in allErrorInfo) {
                error(err.toString(OutputType.UserFocused))
            }
        }
    }

    val prefix = if (failOnWarnings) {
        "ERROR:"
    } else {
        "WARNING:"
    }

    // filter out signatures from proto groups
    val notDefinedSignatures = allSignatures.minus(defSignatures).filter { it.trim().startsWith("\\") }
    if (!printJson && notDefinedSignatures.isNotEmpty()) {
        val signatureOrSignatures = if (notDefinedSignatures.size == 1) {
            "signature is"
        } else {
            "signatures are"
        }
        val coloredPrefix = if (failOnWarnings) {
            bold(red(prefix))
        } else {
            bold(yellow(prefix))
        }
        error(
            "$coloredPrefix The following ${notDefinedSignatures.size} " +
                "$signatureOrSignatures used but not defined:"
        )
    }

    if (!printJson) {
        val indent = " ".repeat(prefix.length + 1)
        for (sig in notDefinedSignatures) {
            error("$indent${bold(sig)}")
        }
    }

    val failed = allErrorInfo.isNotEmpty() || (failOnWarnings && notDefinedSignatures.isNotEmpty())
    if (!printJson && !generateTestCases) {
        val fileOrFiles = if (files.size > 1) {
            "files"
        } else {
            "file"
        }
        if (failed) {
            if (failOnWarnings) {
                error(
                    "${bold(
                        red(
                            "FAILURE:"
                        )
                    )} Found ${allErrorInfo.size + notDefinedSignatures.size} errors from ${files.size} $fileOrFiles"
                )
            } else {
                error(
                    "${bold(
                        red(
                            "FAILURE:"
                        )
                    )} Found ${allErrorInfo.size} errors and ${notDefinedSignatures.size} warnings in ${files.size} $fileOrFiles"
                )
            }
        } else {
            error(
                "${bold(
                    green(
                        "SUCCESS:"
                    )
                )} Processed ${files.size} $fileOrFiles with ${notDefinedSignatures.size} warnings"
            )
        }
    }

    if (output.toLowerCase() != "none") {
        log(outputBuilder.toString())
    }

    return if (failed) 1 else 0
}

private fun getErrorInfo(err: ParseError, file: File, inputLines: List<String>): MessageInfo {
    val lineNumber = "Line ${err.row + 1}: "
    val lineBuilder = StringBuilder()
    lineBuilder.append(lineNumber)
    if (err.row >= 0 && err.row < inputLines.size) {
        lineBuilder.append(inputLines[err.row])
        lineBuilder.append('\n')
        for (i in lineNumber.indices) {
            lineBuilder.append(' ')
        }
        for (i in 0 until err.column) {
            lineBuilder.append(' ')
        }
        lineBuilder.append("^\n")
    }

    return MessageInfo(
        file,
        MessageType.Error,
        err.message,
        lineBuilder.toString(),
        err.row, err.column
    )
}

private fun processFile(file: File, allSignatures: MutableSet<String>, defSignatures: MutableSet<String>): List<MessageInfo> {
    val input = String(file.readBytes())
    val inputLines = input.split("\n")

    return when (val validation = MathLingua.parseWithLocations(input)) {
        is ValidationSuccess -> {
            val parse = validation.value
            val document = parse.document
            val tracker = parse.tracker
            allSignatures.addAll(MathLingua.findAllSignatures(document, tracker).map { it.form })

            for (def in document.defines()) {
                if (def.signature != null) {
                    defSignatures.add(def.signature)
                }
            }

            for (rep in document.represents()) {
                if (rep.signature != null) {
                    defSignatures.add(rep.signature)
                }
            }
            emptyList()
        }
        is ValidationFailure -> validation.errors.map {
            getErrorInfo(it, file, inputLines)
        }
    }
}

private class Mlg : CliktCommand() {
    override fun run() = Unit
}

private class Check : CliktCommand(help = "Analyzes input files for errors.") {
    private val file: List<String> by argument(help = "The *.math files to process").multiple(required = false)
    private val failOnWarnings: Boolean by option(help = "Treat warnings as errors").flag()
    private val json: Boolean by option(help = "Output the results in JSON format.").flag()

    override fun run() = exitProcess(
        runBlocking {
            runMlg(
                printJson = json,
                failOnWarnings = failOnWarnings,
                generateTestCases = false,
                inputs = if (file.isEmpty()) {
                    listOf(Paths.get(".").toAbsolutePath().normalize().toFile())
                } else {
                    file.map { File(it) }
                },
                output = "none",
                expand = false
            )
        }
    )
}

private class Render : CliktCommand("Generates either HTML or MathLingua code with definitions expanded.") {
    private val file: String by argument(help = "The .math file to process")
    private val filter: String? by option(help = "If a file path contains this string(s) it will be " +
        "processed for definitions.  Separate multiple filters with commas.")
    private val format by option(help = "Whether to generate HTML or Mathlingua.")
            .choice("html", "mathlingua").default("html")
    private val noexpand: Boolean by option(help = "Specifies to not expand the contents of entries.").flag()
    private val stdout: Boolean by option(help = "If specified, the rendered content will be printed to standard" +
        "out.  Otherwise, it is written to a file with the same path as the input file except for a '.html' or " +
        "'.out.math' extension.").flag()

    override fun run() = runBlocking {
        val f = File(file)
        if (f.isFile) {
            processFile(f)
        } else {
            f.walk()
                .filter { it.isFile && it.name.endsWith(".math") }
                .forEach {
                    processFile(it)
                }
        }
    }

    private fun write(content: String, fileBeingProcessed: File) {
        if (stdout) {
            log(content)
        } else {
            val ext = if (format == "html") {
                ".html"
            } else {
                ".out.math"
            }
            val outFile = File(fileBeingProcessed.parentFile,
                fileBeingProcessed.nameWithoutExtension + ext)
            outFile.writeText(content)
            log("Wrote ${outFile.absolutePath}")
        }
    }

    private suspend fun processFile(fileToProcess: File) {
        when (val validation = MathLingua.parse(fileToProcess.readText())) {
            is ValidationFailure -> {
                when (format) {
                    "html" -> {
                        val builder = StringBuilder()
                        builder.append("<html><head><style>.content { font-size: 1em; }" +
                            "</style></head><body class='content'><ul>")
                        for (err in validation.errors) {
                            builder.append("<li><span style='color: #e61919;'>ERROR:</span> " +
                                "${err.message} (${err.row + 1}, ${err.column + 1})</li>")
                        }
                        builder.append("</ul></body></html>")
                        write(builder.toString(), fileToProcess)
                    }
                    "mathlingua" -> {
                        val builder = StringBuilder()
                        for (err in validation.errors) {
                            builder.append("ERROR: ${err.message} (${err.row + 1}, ${err.column + 1})\n")
                        }
                        write(builder.toString(), fileToProcess)
                    }
                }
            }
            is ValidationSuccess -> {
                val defines = mutableListOf<DefinesGroup>()
                val represents = mutableListOf<RepresentsGroup>()

                val cwd = Paths.get(".").toAbsolutePath().normalize().toFile()
                val filterItems = (filter ?: "").split(",")
                    .map { it.trim() }.filter { it.isNotEmpty() }

                if (!noexpand) {
                    val allFiles = cwd.walk()
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
                        }.toList()

                    awaitAll(*allFiles.map {
                        GlobalScope.async {
                            val result = MathLingua.parse(it.readText())
                            if (result is ValidationSuccess) {
                                defines.addAll(result.value.defines())
                                represents.addAll(result.value.represents())
                            }
                        }
                    }.toTypedArray())
                }

                val content = MathLingua.prettyPrint(
                    node = validation.value,
                    defines = defines,
                    represents = represents,
                    html = format == "html"
                )

                write(content, fileToProcess)
            }
        }
    }
}

private class DuplicateContent : CliktCommand(name = "dup-content", help = "Identifies duplicate content.") {
    private val file: List<String> by argument(help = "The *.math files to process").multiple(required = false)
    private val json: Boolean by option(help = "Output the results in JSON format.").flag()

    override fun run() = runBlocking {
        val cwd = Paths.get(".").toAbsolutePath().normalize().toFile()
        val inputFiles = if (file.isEmpty()) {
            listOf(cwd.absolutePath)
        } else {
            file
        }

        val allFiles = mutableListOf<File>()
        for (curPath in inputFiles) {
            val curFile = File(curPath)
            if (curFile.isFile && curPath.endsWith(".math")) {
                allFiles.add(curFile)
            } else if (curFile.isDirectory) {
                allFiles.addAll(curFile.walk()
                        .filter { it.isFile && it.name.endsWith(".math") })
            }
        }

        val filesMap = mutableMapOf<String, String>()
        awaitAll(*allFiles.map {
            async {
                filesMap[it.path] = it.readText()
            }
        }.toTypedArray())

        val output = StringBuilder()
        if (json) {
            output.append("[")
        }

        var isFirst = true
        var count = 0
        val contentLocations = MathLingua.findContentLocations(filesMap)
        for ((content, locationSet) in contentLocations) {
            if (locationSet.size > 1) {
                count++
                if (json) {
                    for (loc in locationSet) {
                        if (!isFirst) {
                            output.append(",")
                        }
                        output.append("{")
                        output.append("\"path\": \"${loc.path.jsonSanitize()}\",")
                        output.append("\"row\": ${loc.location.row},")
                        output.append("\"column\": ${loc.location.column}")
                        output.append("}")
                        isFirst = false
                    }
                } else {
                    output.append(bold(content))
                    output.append("\n")
                    output.append("-".repeat(content.split("\n").map { it.length }.max() ?: 1))
                    output.append("\n")
                    val cwdPath = cwd.absolutePath
                    for (loc in locationSet) {
                        output.append("- ${loc.path.relativeTo(cwdPath)} (${loc.location.row + 1}, ${loc.location.column + 1})")
                        output.append("\n")
                    }
                    output.append("\n\n")
                }
                isFirst = false
            }
        }

        if (json) {
            output.append("]")
        } else {
            output.append(
                "Found $count duplicate content${if (count > 1) {
                    "s"
                } else {
                    ""
                }}"
            )
        }

        log(output.toString())
    }
}

private class DuplicateSignatures : CliktCommand(
        name = "dup-sig",
        help = "Identifies duplicate signature definitions.") {
    private val file: List<String> by argument(help = "The *.math files to process").multiple(required = false)
    private val json: Boolean by option(help = "Output the results in JSON format.").flag()

    override fun run() = runBlocking {
        val cwd = Paths.get(".").toAbsolutePath().normalize().toFile()
        val inputFiles = if (file.isEmpty()) {
            listOf(cwd.absolutePath)
        } else {
            file
        }

        val allFiles = mutableListOf<File>()
        for (curPath in inputFiles) {
            val curFile = File(curPath)
            if (curFile.isFile && curPath.endsWith(".math")) {
                allFiles.add(curFile)
            } else if (curFile.isDirectory) {
                allFiles.addAll(curFile.walk()
                        .filter { it.isFile && it.name.endsWith(".math") })
            }
        }

        val filesMap = mutableMapOf<String, String>()
        awaitAll(*allFiles.map {
            async {
                filesMap[it.path] = it.readText()
            }
        }.toTypedArray())

        val output = StringBuilder()
        if (json) {
            output.append("[")
        }

        var isFirst = true
        var count = 0
        val sigLocations = MathLingua.findSignatureLocations(filesMap)
        for ((signature, locationSet) in sigLocations) {
            if (!signature.trim().startsWith("\\")) {
                // only process non-proto signatures
                continue
            }

            if (locationSet.size > 1) {
                count++
                if (json) {
                    for (loc in locationSet) {
                        if (!isFirst) {
                            output.append(",")
                        }
                        output.append("{")
                        output.append("\"path\": \"${loc.path.jsonSanitize()}\",")
                        output.append("\"row\": ${loc.location.row},")
                        output.append("\"column\": ${loc.location.column}")
                        output.append("}")
                        isFirst = false
                    }
                } else {
                    output.append(bold(signature))
                    output.append("\n")
                    output.append("-".repeat(signature.split("\n").map { it.length }.max() ?: 1))
                    output.append("\n")
                    val cwdPath = cwd.absolutePath
                    for (loc in locationSet) {
                        output.append("- ${loc.path.relativeTo(cwdPath)} (${loc.location.row + 1}, ${loc.location.column + 1})")
                        output.append("\n")
                    }
                    output.append("\n\n")
                }
                isFirst = false
            }
        }

        if (json) {
            output.append("]")
        } else {
            output.append(
                "Found $count duplicate signature${if (count > 1) {
                    "s"
                } else {
                    ""
                }}"
            )
        }

        log(output.toString())
    }
}

private class UndefinedSignatures : CliktCommand(
        name = "undef-sig",
        help = "Identifies command that have been used but have not been defined.") {
    private val file: List<String> by argument(help = "The *.math files to process").multiple(required = false)
    private val json: Boolean by option(help = "Output the results in JSON format.").flag()

    override fun run() = runBlocking {
        val cwd = Paths.get(".").toAbsolutePath().normalize().toFile()
        val inputFiles = if (file.isEmpty()) {
            listOf(cwd.absolutePath)
        } else {
            file
        }

        val allFiles = mutableListOf<File>()
        for (curPath in inputFiles) {
            val curFile = File(curPath)
            if (curFile.isFile && curPath.endsWith(".math")) {
                allFiles.add(curFile)
            } else if (curFile.isDirectory) {
                allFiles.addAll(curFile.walk()
                        .filter { it.isFile && it.name.endsWith(".math") })
            }
        }

        val filesMap = mutableMapOf<String, String>()
        awaitAll(*allFiles.map {
            async {
                filesMap[it.path] = it.readText()
            }
        }.toTypedArray())

        val output = StringBuilder()
        if (json) {
            output.append("[")
        }

        var isFirst = true
        var count = 0
        val sigLocations = MathLingua.findUndefinedSignatureLocations(filesMap)
        for ((signature, locationSet) in sigLocations) {
            if (!signature.trim().startsWith("\\")) {
                // only process non-proto signatures
                continue
            }

            count++
            if (json) {
                for (loc in locationSet) {
                    if (!isFirst) {
                        output.append(",")
                    }
                    output.append("{")
                    output.append("\"path\": \"${loc.path.jsonSanitize()}\",")
                    output.append("\"row\": ${loc.location.row},")
                    output.append("\"column\": ${loc.location.column}")
                    output.append("}")
                    isFirst = false
                }
            } else {
                output.append(bold(signature))
                output.append("\n")
                output.append("-".repeat(signature.split("\n").map { it.length }.max() ?: 1))
                output.append("\n")
                val cwdPath = cwd.absolutePath
                for (loc in locationSet) {
                    output.append("- ${loc.path.relativeTo(cwdPath)} (${loc.location.row + 1}, ${loc.location.column + 1})")
                    output.append("\n")
                }
                output.append("\n\n")
            }
            isFirst = false
        }

        if (json) {
            output.append("]")
        } else {
            output.append(
                "Found $count reference${if (count > 1) {
                    "s"
                } else {
                    ""
                }} to undefined signatures"
            )
        }

        log(output.toString())
    }
}

private class Version : CliktCommand(help = "Prints the tool and MathLingua language version.") {
    override fun run() {
        log("MathLingua CLI Version:      $TOOL_VERSION")
        log("MathLingua Language Version: $MATHLINGUA_VERSION")
    }
}

private fun String.relativeTo(dir: String): String {
    val separator = File.separator
    val prefix = if (dir.endsWith(separator)) {
        dir
    } else {
        dir + separator
    }

    return if (this.startsWith(prefix)) {
        this.substring(prefix.length)
    } else {
        this
    }
}

fun main(args: Array<String>) = Mlg().subcommands(
        Check(), Render(), DuplicateContent(), DuplicateSignatures(), UndefinedSignatures(), Version()
).main(args)

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

package mathlingua.jvm

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.output.TermUi
import com.github.ajalt.clikt.parameters.arguments.argument
import com.github.ajalt.clikt.parameters.arguments.multiple
import com.github.ajalt.clikt.parameters.options.default
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import com.github.ajalt.clikt.parameters.types.choice
import mathlingua.common.MathLingua
import mathlingua.common.ParseError
import mathlingua.common.ValidationFailure
import mathlingua.common.ValidationSuccess
import java.io.File
import kotlin.system.exitProcess

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

private data class ErrorInfo(
    val file: File,
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
                builder.append("  \"file\": \"${file.normalize().absolutePath}\",")
                builder.append("  \"message\": \"${message.replace("\n", "\\n")}\",")
                builder.append("  \"failedLine\": \"${failedLine.replace("\n", "\\n")}\",")
                builder.append("  \"row\": $row,")
                builder.append("  \"column\": $column")
                builder.append("}")
            }
            OutputType.TestCase -> {
                builder.append("Row: $row")
                builder.append("Column: $column")
                builder.append("Message:")
                builder.append(message)
                builder.append("EndMessage:")
            }
            else -> {
                builder.append(bold("File: $file"))
                builder.append(failedLine.trim())
                builder.append(message.trim())
            }
        }
        return builder.toString()
    }
}

private fun runMlg(
    printJson: Boolean,
    failOnWarnings: Boolean,
    generateTestCases: Boolean,
    inputs: List<File>,
    output: String,
    expand: Boolean
): Int {
    val files = mutableListOf<File>()
    inputs.forEach {
        files.addAll(findFiles(it, ".math"))
    }

    val allSignatures = mutableSetOf<String>()
    val defSignatures = mutableSetOf<String>()

    val allErrorInfo = mutableListOf<ErrorInfo>()
    for (f in files) {
        allErrorInfo.addAll(processFile(f, allSignatures, defSignatures))
    }

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
                        error("  ${bold(red("ERROR(${err.row}, ${err.column}):  "))}${err.message}")
                    }
                }
                is ValidationSuccess -> {
                    val doc = validation.value
                    val defines = if (expand) {
                        doc.defines
                    } else {
                        emptyList()
                    }

                    outputBuilder.append(MathLingua.prettyPrint(
                            node = doc,
                            defines = defines,
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

    val notDefinedSignatures = allSignatures.minus(defSignatures)
    if (notDefinedSignatures.isNotEmpty()) {
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
        error("$coloredPrefix The following ${notDefinedSignatures.size} " +
            "$signatureOrSignatures used but not defined:")
    }

    val indent = " ".repeat(prefix.length + 1)
    for (sig in notDefinedSignatures) {
        error("$indent${bold(sig)}")
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
                error("${bold(red("FAILURE:"))} Found ${allErrorInfo.size + notDefinedSignatures.size} errors from ${files.size} $fileOrFiles")
            } else {
                error("${bold(red("FAILURE:"))} Found ${allErrorInfo.size} errors and ${notDefinedSignatures.size} warnings in ${files.size} $fileOrFiles")
            }
        } else {
            error("${bold(green("SUCCESS:"))} Processed ${files.size} $fileOrFiles with ${notDefinedSignatures.size} warnings")
        }
    }

    if (output.toLowerCase() != "none") {
        log(outputBuilder.toString())
    }

    return if (failed) 1 else 0
}

private fun getErrorInfo(err: ParseError, file: File, inputLines: List<String>): ErrorInfo {
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

    return ErrorInfo(
        file,
        err.message,
        lineBuilder.toString(),
        err.row, err.column
    )
}

private fun processFile(file: File, allSignatures: MutableSet<String>, defSignatures: MutableSet<String>): List<ErrorInfo> {
    val input = String(file.readBytes())
    val inputLines = input.split("\n")

    return when (val validation = MathLingua.parseWithLocations(input)) {
        is ValidationSuccess -> {
            val parse = validation.value
            val document = parse.document
            val tracker = parse.tracker
            allSignatures.addAll(MathLingua.findAllSignatures(document, tracker).map { it.form })

            for (def in document.defines) {
                if (def.signature != null) {
                    defSignatures.add(def.signature)
                }
            }

            for (rep in document.represents) {
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

private fun findFiles(file: File, ext: String): List<File> {
    if (file.isFile) {
        return if (file.absolutePath.endsWith(ext)) {
            listOf(file)
        } else {
            emptyList()
        }
    }

    val result = mutableListOf<File>()
    val children = file.listFiles() ?: arrayOf()
    for (child in children) {
        result.addAll(findFiles(child, ext))
    }

    return result
}

private class Mlg : CliktCommand() {
    private val json: Boolean by option(help = "Print results in JSON").flag()
    private val testCase: Boolean by option(help = "Print results as unit test cases").flag()
    private val warningsAsErrors: Boolean by option(help = "Treat warnings as errors").flag()
    private val file: List<String> by argument(help = "The *.math files to process").multiple(required = true)
    private val output by option(help = "Whether output the specified files as html, mathlingua, or none")
            .choice("html", "mathlingua", "none").default("none")
    private val expand: Boolean by option(help = "Whether to expand the contents of the entries.").flag()

    override fun run() {
        exitProcess(runMlg(json, warningsAsErrors, testCase, file.map { File(it) }, output, expand))
    }
}

fun main(args: Array<String>) = Mlg().main(args)

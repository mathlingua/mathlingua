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

import mathlingua.common.MathLingua
import mathlingua.common.ParseError
import mathlingua.common.ValidationFailure
import mathlingua.common.ValidationSuccess
import java.io.File
import kotlin.system.exitProcess

data class ErrorInfo(
    val file: String,
    val message: String,
    val failedLine: String,
    val row: Int,
    val column: Int
) {
    fun toString(json: Boolean): String {
        val builder = StringBuilder()
        if (json) {
            println("{")
            println("  \"file\": \"$file\",")
            println("  \"message\": \"${message.replace("\n", "\\n")}\",")
            println("  \"failedLine\": \"${failedLine.replace("\n", "\\n")}\",")
            println("  \"row\": $row,")
            println("  \"column\": $column")
            println("}")
        } else {
            println("File: $file")
            println(failedLine)
            println(message)
        }
        return builder.toString()
    }
}

fun main(args: Array<String>) {
    val files = mutableListOf<File>()
    var printJson = false
    for (arg in args) {
        if (arg == "--json") {
            printJson = true
        } else {
            files.addAll(findFiles(File(arg), ".txt"))
        }
    }

    val allSignatures = mutableSetOf<String>()
    val defSignatures = mutableSetOf<String>()

    val allErrorInfo = mutableListOf<ErrorInfo>()
    for (f in files) {
        allErrorInfo.addAll(processFile(f, allSignatures, defSignatures))
    }

    if (printJson) {
        println("[")
        for (i in 0 until allErrorInfo.size) {
            print(allErrorInfo[i].toString(true))
            if (i != allErrorInfo.size - 1) {
                println(",")
            }
        }
        println("]")
    } else {
        for (err in allErrorInfo) {
            println(err.toString(false))
            println()
        }
        if (allErrorInfo.isEmpty()) {
            println("SUCCESS: Processed ${files.size} files")
        } else {
            println("FAILURE: Found ${allErrorInfo.size} errors from ${files.size} files")
        }
    }

    val notDefinedSignatures = allSignatures.minus(defSignatures)
    if (notDefinedSignatures.isNotEmpty()) {
        println("WARNING: The following ${notDefinedSignatures.size} " +
            "signatures are used but not defined:")
    }

    for (sig in notDefinedSignatures) {
        println("  $sig")
    }

    exitProcess(if (allErrorInfo.isEmpty()) 0 else 1)
}

fun getErrorInfo(err: ParseError, file: File, inputLines: List<String>): ErrorInfo {
    val lineNumber = "Line ${err.row + 1}: "
    val lineBuilder = StringBuilder()
    lineBuilder.append(lineNumber)
    lineBuilder.append(inputLines[err.row])
    lineBuilder.append('\n')
    for (i in 0 until lineNumber.length) {
        lineBuilder.append(' ')
    }
    for (i in 0 until err.column) {
        lineBuilder.append(' ')
    }
    lineBuilder.append("^\n")

    return ErrorInfo(
        file.absolutePath,
        err.message,
        lineBuilder.toString(),
        err.row, err.column
    )
}

fun processFile(file: File, allSignatures: MutableSet<String>, defSignatures: MutableSet<String>): List<ErrorInfo> {
    val input = String(file.readBytes())
    val inputLines = input.split("\n")

    return when (val validation = MathLingua().parse(input)) {
        is ValidationSuccess -> {
            val document = validation.value
            val ml = MathLingua()
            allSignatures.addAll(ml.findAllSignatures(document))

            for (def in document.defines) {
                defSignatures.addAll(ml.findAllSignatures(def))
            }

            for (rep in document.represents) {
                defSignatures.addAll(ml.findAllSignatures(rep))
            }
            emptyList()
        }
        is ValidationFailure -> validation.errors.map {
            getErrorInfo(it, file, inputLines)
        }
    }
}

fun findFiles(file: File, ext: String): List<File> {
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

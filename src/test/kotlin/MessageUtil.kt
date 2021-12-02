/*
 * Copyright 2020 The MathLingua Authors
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

import java.io.File
import mathlingua.cli.VirtualFile
import mathlingua.cli.VirtualFileException
import mathlingua.cli.VirtualFileSystem
import mathlingua.frontend.FrontEnd
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.ValidationFailure

data class MessageTestCase(
    val name: String, val input: String, val expectedErrors: List<ParseError>)

private fun getNamePrefix(name: String) = "$name:"

private fun String.expectedName(name: String) {
    val prefix = getNamePrefix(name)
    if (!this.startsWith(prefix)) {
        throw RuntimeException("Expected line $this to start with $prefix")
    }
}

private fun String.expectedNameValue(name: String): Int {
    this.expectedName(name)
    val prefix = getNamePrefix(name)
    return this.substring(prefix.length).trim().toInt()
}

private fun loadExpectedErrors(input: String): List<ParseError> {
    val expectedErrors = mutableListOf<ParseError>()
    val lines = input.split("\n")
    var index = 0
    while (index < lines.size) {
        while (index < lines.size && lines[index].isBlank()) {
            index++
        }

        if (index >= lines.size) {
            break
        }

        val row = lines[index++].expectedNameValue("Row")
        val column = lines[index++].expectedNameValue("Column")
        lines[index++].expectedName("Message")
        val builder = StringBuilder()
        while (index < lines.size && lines[index].trim() != "EndMessage:") {
            if (builder.isNotEmpty()) {
                builder.append('\n')
            }
            builder.append(lines[index++])
        }
        lines[index++].expectedName("EndMessage")
        expectedErrors.add(ParseError(message = builder.toString(), row = row, column = column))
    }
    return expectedErrors
}

private fun containsInputDotMath(dir: VirtualFile): Boolean {
    if (!dir.isDirectory()) {
        return false
    }

    for (child in dir.listFiles()) {
        if (child.absolutePath().lastOrNull() == "input.math") {
            return true
        }
    }
    return false
}

private fun loadMessageTestCaseImpl(
    fs: VirtualFileSystem, file: VirtualFile, result: MutableList<MessageTestCase>
) {
    if (!file.isDirectory()) {
        return
    }

    if (file.absolutePath().last() == ".DS_Store") {
        return
    }

    if (containsInputDotMath(file)) {
        val messageFile = fs.getFile(file.relativePath().split("/").plus("messages.txt"))
        val input = fs.getFile(file.relativePath().split("/").plus("input.math")).readText()
        val expectedErrors =
            if (OVERWRITE_GOLDEN_FILES) {
                val validation = FrontEnd.parse(input)
                val errors =
                    if (validation is ValidationFailure) {
                        validation.errors
                    } else {
                        emptyList()
                    }

                val builder = StringBuilder()
                for ((index, err) in errors.withIndex()) {
                    builder.append("Row: ")
                    builder.append(err.row)
                    builder.append("\nColumn: ")
                    builder.append(err.column)
                    builder.append("\nMessage:\n")
                    builder.append(err.message)
                    builder.append("\nEndMessage:")
                    if (index != errors.size - 1) {
                        builder.append("\n\n\n")
                    }
                }
                messageFile.writeText(builder.toString())

                errors
            } else {
                loadExpectedErrors(messageFile.readText())
            }

        result.add(
            MessageTestCase(
                name = file.absolutePath().last(), input = input, expectedErrors = expectedErrors))
    }

    for (child in file.listFiles()) {
        loadMessageTestCaseImpl(fs, child, result)
    }
}

fun loadMessageTestCases(): List<MessageTestCase> {
    val fs = newDiskFileSystem()
    val root = fs.getDirectory(listOf("src", "test", "resources", "goldens", "messages"))
    if (!root.exists()) {
        throw VirtualFileException(
            "Golden root directory ${root.absolutePath().joinToString(File.separator)} does not exist")
    }

    val result = mutableListOf<MessageTestCase>()
    loadMessageTestCaseImpl(fs, root, result)
    return result
}

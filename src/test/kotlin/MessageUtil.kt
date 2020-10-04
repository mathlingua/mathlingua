/*
 * Copyright 2020 Google LLC
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

import mathlingua.common.support.ParseError
import java.io.File
import java.io.IOException
import java.lang.RuntimeException
import java.lang.StringBuilder
import java.nio.file.Paths

data class MessageTestCase(
    val name: String,
    val input: String,
    val expectedErrors: List<ParseError>
)

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
        expectedErrors.add(
            ParseError(
                message = builder.toString(),
                row = row,
                column = column
            )
        )
    }
    return expectedErrors
}

fun loadMessageTestCases(): List<MessageTestCase> {
    val result = mutableListOf<MessageTestCase>()

    val root = Paths.get("src", "test", "resources", "goldens", "messages").toFile()
    if (!root.exists()) {
        throw IOException("Golden root directory ${root.absolutePath} does not exist")
    }

    val caseDirs = root.listFiles()
    if (caseDirs != null) {
        for (caseDir in caseDirs) {
            if (caseDir.name == ".DS_Store") {
                continue
            }

            result.add(MessageTestCase(
                name = caseDir.name,
                input = File(caseDir, "input.math").readText(),
                expectedErrors = loadExpectedErrors(File(caseDir, "messages.txt").readText())
            ))
        }
    }

    return result
}

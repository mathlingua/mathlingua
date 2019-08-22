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

import java.io.File

data class TestCase(val name: String, val input: String, val expectedOutput: String)

const val TEST_NAME_PREFIX = "-- Test:"
const val INPUT_PREFIX = "-- Input:"
const val OUTPUT_PREFIX = "-- Output:"
const val END_PREFIX = "-- End;"

fun loadTestCases(file: File): List<TestCase> {
    // This function is only used in tests, and thus
    // it is good for it to fail with expections thrown
    // if the input file does not have the correct syntax.

    val result = mutableListOf<TestCase>()
    val lines = file.readLines()

    var i = 0
    while (i < lines.size) {
        // skip blank lines
        while (i < lines.size && lines[i].isBlank()) {
            i++
        }

        val testNameLine = lines[i]
        expectPrefix(TEST_NAME_PREFIX, testNameLine, i + 1)
        val testName = trimPrefix(TEST_NAME_PREFIX, testNameLine)
        i++

        val inputLine = lines[i]
        expectPrefix(INPUT_PREFIX, inputLine, i + 1)
        i++

        val inputBuilder = StringBuilder()
        while (i < lines.size && !hasPrefix(OUTPUT_PREFIX, lines[i])) {
            inputBuilder.append(lines[i++])
            inputBuilder.append("\n")
        }

        val outputLine = lines[i]
        expectPrefix(OUTPUT_PREFIX, outputLine, i + 1)
        i++

        val outputBuilder = StringBuilder()
        while (i < lines.size && !hasPrefix(END_PREFIX, lines[i])) {
            outputBuilder.append(lines[i++])
            outputBuilder.append('\n')
        }

        val endLine = lines[i]
        expectPrefix(END_PREFIX, endLine, i + 1)
        i++

        result.add(
            TestCase(
                name = testName,
                input = inputBuilder.toString(),
                expectedOutput = outputBuilder.toString()
            )
        )
    }

    return result
}

fun expectPrefix(prefix: String, line: String, lineNumber: Int) {
    if (!line.startsWith(prefix)) {
        throw RuntimeException(
            "Expected the line $lineNumber: '$line' to begin with $prefix"
        )
    }
}

fun hasPrefix(prefix: String, line: String): Boolean {
    return line.startsWith(prefix)
}

fun trimPrefix(prefix: String, line: String): String {
    return if (hasPrefix(prefix, line)) {
        line.substring(prefix.length)
    } else {
        line
    }
}

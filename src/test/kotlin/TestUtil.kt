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

data class TestCase(val name: String,
                    val input: String,
                    val phase1Output: String,
                    val phase2Output: String)

const val TEST_NAME_PREFIX = "-- Test:"
const val INPUT_START = "-- Input:"
const val INPUT_END = "-- EndInput:"
const val OUTPUT_START = "-- Output:"
const val OUTPUT_END = "-- EndOutput:"
const val OUTPUT_PHASE1_START = "-- Output(Phase1):"
const val OUTPUT_PHASE1_END = "-- EndOutput(Phase1):"
const val OUTPUT_PHASE2_START = "-- Output(Phase2):"
const val OUTPUT_PHASE2_END = "-- EndOutput(Phase2):"

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

        fun readSection(start: String, end: String): String? {
            if (i >= lines.size || lines[i] != start) {
                return null
            }

            expectPrefix(start, lines[i], i + 1)
            i++ // move past the start line

            val buffer = StringBuilder()
            while (i < lines.size && lines[i] != end) {
                buffer.append(lines[i++])
                buffer.append('\n')
            }
            expectPrefix(end, lines[i], i + 1)
            i++ // move past the end line

            return buffer.toString()
        }

        val testNameLine = lines[i]
        expectPrefix(TEST_NAME_PREFIX, testNameLine, i + 1)
        val testName = trimPrefix(TEST_NAME_PREFIX, testNameLine)
        i++

        val input = readSection(INPUT_START, INPUT_END)
        val output = readSection(OUTPUT_START, OUTPUT_END)
        val phase1Output = readSection(OUTPUT_PHASE1_START, OUTPUT_PHASE1_END)
        val phase2Output = readSection(OUTPUT_PHASE2_START, OUTPUT_PHASE2_END)

        if (input == null) {
            throw Exception("Line ${i+1}: Input not specified")
        }

        if (output == null && (phase1Output == null || phase2Output == null)) {
            throw Exception("Line ${i+1}: Output is not specified and " +
                    "one of Phase1Output or Phase2Output is missing")
        }

        if (output != null && (phase1Output != null || phase2Output != null)) {
            throw Exception("Line ${i+1}: Output is specified but so " +
                    "is one of Phase1Output or Phase2Output")
        }

        result.add(
            TestCase(
                name = testName,
                input = input,
                phase1Output = (phase1Output ?: output)!!,
                phase2Output = (phase2Output ?: output)!!
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

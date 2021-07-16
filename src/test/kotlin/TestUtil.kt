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

import com.tylerthrailkill.helpers.prettyprint.pp
import java.io.File
import java.io.IOException
import java.lang.StringBuilder
import java.nio.file.Paths

// Set to true to overwrite the golden files instead of checking against them.
// This is useful if a change was made that impacts a lot of golden files, and
// you want to regenerate them and use `git diff` to see how your change
// impacted the tests.
internal const val OVERWRITE_GOLDEN_FILES = false

data class TestCase(
    val name: String,
    val input: File,
    val phase1Output: File,
    val phase1Structure: File,
    val phase2Output: File,
    val phase2Structure: File)

enum class GoldenType {
    Chalktalk,
    Textalk
}

fun loadTestCases(type: GoldenType): List<TestCase> {
    val result = mutableListOf<TestCase>()

    val root = Paths.get("src", "test", "resources", "goldens", type.name.toLowerCase()).toFile()
    if (!root.exists()) {
        throw IOException("Golden root directory ${root.absolutePath} does not exist")
    }

    val caseDirs = root.walkTopDown().filter { File(it, "input.math").exists() }
    for (caseDir in caseDirs) {
        result.add(
            TestCase(
                name = caseDir.name,
                input = File(caseDir, "input.math"),
                phase1Output = File(caseDir, "phase1-output.math"),
                phase1Structure = File(caseDir, "phase1-structure.txt"),
                phase2Output = File(caseDir, "phase2-output.math"),
                phase2Structure = File(caseDir, "phase2-structure.txt")))
    }

    return result
}

fun serialize(obj: Any): String {
    val builder = StringBuilder()
    pp(obj, writeTo = builder, indent = 2, wrappedLineWidth = Integer.MAX_VALUE)
    return builder.toString()
}

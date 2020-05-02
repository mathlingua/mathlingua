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

data class TestCase(
    val name: String,
    val input: String,
    val phase1Output: String,
    val phase1Structure: String,
    val phase2Output: String,
    val phase2Structure: String
)

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

    val caseDirs = root.listFiles()
    if (caseDirs != null) {
        for (caseDir in caseDirs) {
            result.add(TestCase(
                    name = caseDir.name,
                    input = File(caseDir, "input.math").readText(),
                    phase1Output = File(caseDir, "phase1-output.math").readText(),
                    phase1Structure = File(caseDir, "phase1-structure.txt").readText(),
                    phase2Output = File(caseDir, "phase2-output.math").readText(),
                    phase2Structure = File(caseDir, "phase2-structure.txt").readText()
            ))
        }
    }

    return result
}

fun serialize(obj: Any): String {
    val builder = StringBuilder()
    pp(obj, writeTo = builder, indent = 2, wrappedLineWidth = Integer.MAX_VALUE)
    return builder.toString()
}

/*
 * Copyright 2019 The MathLingua Authors
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
import mathlingua.cli.VirtualFile
import mathlingua.cli.VirtualFileException
import mathlingua.cli.VirtualFileSystem

// Set to true to overwrite the golden files instead of checking against them.
// This is useful if a change was made that impacts a lot of golden files, and
// you want to regenerate them and use `git diff` to see how your change
// impacted the tests.
internal const val OVERWRITE_GOLDEN_FILES = false

data class TestCase(
    val name: String,
    val input: VirtualFile,
    val phase1Output: VirtualFile,
    val phase1Structure: VirtualFile,
    val phase2Output: VirtualFile,
    val phase2Structure: VirtualFile)

enum class GoldenType {
    Chalktalk,
    Textalk
}

fun loadTestCases(type: GoldenType): List<TestCase> {
    val result = mutableListOf<TestCase>()
    val fs = newDiskFileSystem()
    val root = fs.getDirectory(listOf("src", "test", "resources", "goldens", type.name.lowercase()))
    if (!root.exists()) {
        throw VirtualFileException(
            "Golden root directory ${root.absolutePath().joinToString(fs.getFileSeparator())} does not exist")
    }
    loadTestCasesImpl(fs, root, result)
    return result
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

private fun loadTestCasesImpl(
    fs: VirtualFileSystem, file: VirtualFile, result: MutableList<TestCase>
) {
    if (file.isDirectory() && containsInputDotMath(file)) {
        val cwd = fs.cwd()
        result.add(
            TestCase(
                name = file.absolutePath().last(),
                input = fs.getFile(file.relativePathTo(cwd).plus("input.math")),
                phase1Output = fs.getFile(file.relativePathTo(cwd).plus("phase1-output.math")),
                phase1Structure = fs.getFile(file.relativePathTo(cwd).plus("phase1-structure.txt")),
                phase2Output = fs.getFile(file.relativePathTo(cwd).plus("phase2-output.math")),
                phase2Structure =
                    fs.getFile(file.relativePathTo(cwd).plus("phase2-structure.txt"))))
    }

    if (file.isDirectory()) {
        for (child in file.listFiles()) {
            loadTestCasesImpl(fs, child, result)
        }
    }
}

fun serialize(obj: Any): String {
    val builder = StringBuilder()
    pp(obj, writeTo = builder, indent = 2, wrappedLineWidth = Integer.MAX_VALUE)
    return builder.toString()
}

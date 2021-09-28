/*
 * Copyright 2020 The MathLingua Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *    http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package mathlingua

import assertk.assertThat
import assertk.assertions.isEqualTo
import mathlingua.cli.Mathlingua
import mathlingua.cli.MemoryLogger
import mathlingua.cli.newMemoryFileSystem
import org.junit.jupiter.api.Test

private data class PathAndContent(val path: List<String>, val content: String)

private fun String.removeColorCodes() = this.replace(Regex("\\x1b\\[[0-9]*m"), "")

internal class EndToEndCheckTest {
    private fun runCheckTest(
        files: List<PathAndContent>,
        expectedOutput: String,
        expectedNumErrors: Int,
        expectedExitCode: Int
    ) {
        val fs = newMemoryFileSystem(cwd = listOf(""))
        for (file in files) {
            val vf = fs.getFile(file.path)
            vf.writeText(file.content)
        }

        val logger = MemoryLogger()
        val ret = Mathlingua.check(fs = fs, logger = logger, files = emptyList(), json = false)

        val logText = logger.getLogs().joinToString("\n")
        val numErrors = logText.split("\n").filter { it.contains("ERROR") }.size

        assertThat(logText.removeColorCodes()).isEqualTo(expectedOutput)
        assertThat(numErrors).isEqualTo(expectedNumErrors)
        assertThat(ret).isEqualTo(expectedExitCode)
    }

    @Test
    fun `check reports errors on undefined signatures`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    Theorem:
                    given: x
                    then:
                    . 'x is \something'
                """.trimIndent())),
            expectedOutput =
                """
            ERROR: content/file1.math (Line: 4, Column: 3)
            Undefined signature '\something'

            ERROR: content/file1.math (Line: 4, Column: 3)
            No matching definition found for \something

            FAILED
            Processed 1 file
            2 errors detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 2)
    }

    @Test
    fun `check does not report errors on defined signatures`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    means: X
                    written: "X"
                    called: "X"


                    Theorem:
                    given: x
                    then:
                    . 'x is \something'
                """.trimIndent())),
            expectedOutput =
                """
            SUCCESS
            Processed 1 file
            0 errors detected
        """.trimIndent(),
            expectedExitCode = 0,
            expectedNumErrors = 0)
    }

    @Test
    fun `check reports errors on undefined variables`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    means: X
                    written: "X"
                    called: "X"


                    Theorem:
                    given: x
                    then:
                    . 'y is \something'
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 11, Column: 3)
                Undefined symbol 'y'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check does not reports errors on defined variables`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    means: X
                    written: "X"
                    called: "X"


                    Theorem:
                    given: x
                    then:
                    . 'x is \something'
                """.trimIndent())),
            expectedOutput =
                """
                SUCCESS
                Processed 1 file
                0 errors detected
        """.trimIndent(),
            expectedExitCode = 0,
            expectedNumErrors = 0)
    }

    @Test
    fun `check reports errors on duplicate defined signatures`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    means: X
                    written: "X"
                    called: "X"


                    [\something]
                    Defines: f(x)
                    means: f
                    written: "f"
                    called: "f"


                    Theorem:
                    given: x
                    then:
                    . 'x is \something'
                """.trimIndent())),
            expectedOutput =
                """
            ERROR: content/file1.math (Line: 8, Column: 1)
            Duplicate defined signature '\something'
            
            FAILED
            Processed 1 file
            1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check does not report errors on non-duplicate defined signatures`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    means: X
                    written: "X"
                    called: "X"


                    [\something.else]
                    Defines: f(x)
                    means: f
                    written: "f"
                    called: "f"


                    Theorem:
                    given: x
                    then:
                    . 'x is \something'
                """.trimIndent())),
            expectedOutput =
                """
            SUCCESS
            Processed 1 file
            0 errors detected
        """.trimIndent(),
            expectedExitCode = 0,
            expectedNumErrors = 0)
    }

    @Test
    fun `check reports errors on duplicate defined variables from 'exists'`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    Theorem:
                    given: x
                    then:
                    . exists: x
                      suchThat:
                      . 'x'
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 0, Column: 0)
                Duplicate defined symbol 'x'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check reports errors on duplicate defined variables from 'existsUnique'`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    Theorem:
                    given: x
                    then:
                    . existsUnique: x
                      suchThat:
                      . 'x'
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 0, Column: 0)
                Duplicate defined symbol 'x'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check reports errors on duplicate defined variables from 'forAll'`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    Theorem:
                    given: x
                    then:
                    . forAll: x
                      then:
                      . 'x'
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 0, Column: 0)
                Duplicate defined symbol 'x'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check reports errors on duplicate defined variables in Axiom_given`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    Axiom:
                    given: x, x
                    then:
                    . 'x'
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 0, Column: 0)
                Duplicate defined symbol 'x'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check reports errors on duplicate defined variables in Theorem_given`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    Theorem:
                    given: x, x
                    then:
                    . 'x'
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 0, Column: 0)
                Duplicate defined symbol 'x'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check reports errors on duplicate defined variables in Conjecture_given`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    Conjecture:
                    given: x, x
                    then:
                    . 'x'
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 0, Column: 0)
                Duplicate defined symbol 'x'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check reports errors on duplicate defined variables in exists`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: x
                    means:
                    . exists: y, y
                      suchThat: 'x'
                    written: ""
                    called: ""
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 0, Column: 0)
                Duplicate defined symbol 'y'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check reports errors on duplicate defined variables in existsUnique`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: x
                    means:
                    . existsUnique: y, y
                      suchThat: 'x'
                    written: ""
                    called: ""
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 0, Column: 0)
                Duplicate defined symbol 'y'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check reports errors on duplicate defined variables in forAll`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: x
                    means:
                    . forAll: y, y
                      then: 'x'
                    written: ""
                    called: ""
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 0, Column: 0)
                Duplicate defined symbol 'y'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check reports errors on duplicate defined variables in exists_forAll`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: x
                    means:
                    . exists: y
                      suchThat:
                      . forAll: y
                        then: 'y'
                    written: ""
                    called: ""
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 6, Column: 5)
                Duplicate defined symbol 'y'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check reports errors on duplicate defined variables in existsUnique_forAll`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: x
                    means:
                    . existsUnique: y
                      suchThat:
                      . forAll: y
                        then: 'y'
                    written: ""
                    called: ""
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 6, Column: 5)
                Duplicate defined symbol 'y'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check reports errors on duplicate defined variables in forAll_exists`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: x
                    means:
                    . forAll: y
                      then:
                      . exists: y
                        suchThat: 'y'
                    written: ""
                    called: ""
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 6, Column: 5)
                Duplicate defined symbol 'y'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check reports errors on duplicate defined variables in forAll_existsUnique`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: x
                    means:
                    . forAll: y
                      then:
                      . existsUnique: y
                        suchThat: 'y'
                    written: ""
                    called: ""
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 6, Column: 5)
                Duplicate defined symbol 'y'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check does not report errors on non-duplicate defined variables`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    Theorem:
                    given: x
                    then:
                    . exists: y
                      suchThat:
                      . 'y'
                """.trimIndent())),
            expectedOutput =
                """
                SUCCESS
                Processed 1 file
                0 errors detected
        """.trimIndent(),
            expectedExitCode = 0,
            expectedNumErrors = 0)
    }

    @Test
    fun `check reports errors on undefined signatures usage`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    means: X
                    written: "X"
                    called: "X"


                    Theorem:
                    given: x
                    then:
                    . 'x is \something.else'
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 11, Column: 3)
                Undefined signature '\something.else'

                ERROR: content/file1.math (Line: 11, Column: 3)
                No matching definition found for \something.else

                FAILED
                Processed 1 file
                2 errors detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 2)
    }

    @Test
    fun `check does not report errors on defined signature usage`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    means: X
                    written: "X"
                    called: "X"


                    Theorem:
                    given: x
                    then:
                    . 'x is \something'
                """.trimIndent())),
            expectedOutput =
                """
                SUCCESS
                Processed 1 file
                0 errors detected
        """.trimIndent(),
            expectedExitCode = 0,
            expectedNumErrors = 0)
    }

    @Test
    fun `check reports errors on incorrect signature usage missing named group`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\function:on{A}:to{B}]
                    Defines: f(x)
                    means: "a function on A? to B?"
                    written: "X"
                    called: "X"


                    Theorem:
                    given: f, A
                    then:
                    . 'f is \function:on{A}'
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 11, Column: 3)
                Undefined signature '\function:on'

                ERROR: content/file1.math (Line: 11, Column: 3)
                No matching definition found for \function:on{A}

                FAILED
                Processed 1 file
                2 errors detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 2)
    }

    @Test
    fun `check does not report errors on correct signature usage on multiple named groups`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\function:on{A}:to{B}]
                    Defines: f(x)
                    means: "a function on A? to B?"
                    written: "X"
                    called: "X"


                    Theorem:
                    given: f, A, B
                    then:
                    . 'f is \function:on{A}:to{B}'
                """.trimIndent())),
            expectedOutput =
                """
                SUCCESS
                Processed 1 file
                0 errors detected
        """.trimIndent(),
            expectedExitCode = 0,
            expectedNumErrors = 0)
    }

    @Test
    fun `check reports errors on incorrect signature usage missing all parameters`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something{x}]
                    Defines: X
                    means: 'X'
                    written: "X"
                    called: "X"


                    Theorem:
                    given: x
                    then:
                    . 'x is \something'
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 11, Column: 3)
                Expected exactly 1 curly brace parameter group but found 0

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check does not report errors on correct signature usage with a single parameter`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something{x}]
                    Defines: X
                    means: 'X'
                    written: "X"
                    called: "X"


                    Theorem:
                    given: x
                    then:
                    . 'x is \something{x}'
                """.trimIndent())),
            expectedOutput =
                """
                SUCCESS
                Processed 1 file
                0 errors detected
        """.trimIndent(),
            expectedExitCode = 0,
            expectedNumErrors = 0)
    }

    @Test
    fun `check reports errors on incorrect signature usage missing some parameters`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something{x, y, z}]
                    Defines: X
                    means: 'X'
                    written: "X"
                    called: "X"


                    Theorem:
                    given: x
                    then:
                    . 'x is \something{x}'
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 11, Column: 3)
                Expected exactly 3 arguments but found 1 for '{x}'
                
                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check does not report errors on correct signature usage with multiple parameters`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something{x, y, z}]
                    Defines: X
                    means: 'X'
                    written: "X"
                    called: "X"


                    Theorem:
                    given: a, x, y, z
                    then:
                    . 'a is \something{x, y, z}'
                """.trimIndent())),
            expectedOutput =
                """
                SUCCESS
                Processed 1 file
                0 errors detected
        """.trimIndent(),
            expectedExitCode = 0,
            expectedNumErrors = 0)
    }

    @Test
    fun `check reports errors if a function-like definition with (x) doesn't define a function`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                        [\some.function(x)]
                        Defines: f
                        means: "something"
                        written: "something"
                        called: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 2, Column: 1)
                Expected a definition of a function with arguments 'x'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check does not report errors if a function-like definition with (x) defines a function`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                        [\some.function(x)]
                        Defines: f(x)
                        means: "something"
                        written: "something"
                        called: "something"
                """.trimIndent())),
            expectedOutput =
                """
                SUCCESS
                Processed 1 file
                0 errors detected
        """.trimIndent(),
            expectedExitCode = 0,
            expectedNumErrors = 0)
    }

    @Test
    fun `check reports errors if the variable in a function definition is duplicated`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                        [\some.function(x)]
                        Defines: f(x)
                        means:
                        . forAll: x
                          then: x
                        written: "something"
                        called: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 0, Column: 0)
                Duplicate defined symbol 'x'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check does not report errors when placeholder variables are shared`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    Theorem:
                    given: f(x), g(x)
                    then:
                    . "something"
                """.trimIndent())),
            expectedOutput =
                """
                SUCCESS
                Processed 1 file
                0 errors detected
        """.trimIndent(),
            expectedExitCode = 0,
            expectedNumErrors = 0)
    }

    @Test
    fun `check does not report errors when placeholder variables are used in square brackets`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something[x]{f}]
                    Defines: X
                    means: "something"
                    written: "something"
                    called: "something"

                    Theorem:
                    given: f(x)
                    then:
                    . '\something[t]{f(t)}'
                """.trimIndent())),
            expectedOutput =
                """
                SUCCESS
                Processed 1 file
                0 errors detected
        """.trimIndent(),
            expectedExitCode = 0,
            expectedNumErrors = 0)
    }
}

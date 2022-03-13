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
import mathlingua.cli.newMemoryFileSystem
import mathlingua.cli.newMemoryLogger
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

        val logger = newMemoryLogger()
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
                    satisfying: X
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
                    satisfying: X
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
                    satisfying: X
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
    fun `check does not report errors on variables defined in Defines-where`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"


                    [\something]
                    Defines: G := (A, B)
                    where: A := (X, *, e)
                    means: 'X is \something.else'
                    satisfying: 'e is \something.else'
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
                    satisfying: X
                    written: "X"
                    called: "X"


                    [\something]
                    Defines: f(x)
                    satisfying: f
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
                    satisfying: X
                    written: "X"
                    called: "X"


                    [\something.else]
                    Defines: f(x)
                    satisfying: f
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
                ERROR: content/file1.math (Line: 4, Column: 3)
                Duplicate defined symbol 'x' in `exists:`

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
                ERROR: content/file1.math (Line: 4, Column: 3)
                Duplicate defined symbol 'x' in `existsUnique:`

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
                ERROR: content/file1.math (Line: 4, Column: 3)
                Duplicate defined symbol 'x' in `forAll:`

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
                ERROR: content/file1.math (Line: 2, Column: 1)
                Duplicate defined symbol 'x' in `given:`

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
                ERROR: content/file1.math (Line: 2, Column: 1)
                Duplicate defined symbol 'x' in `given:`

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
                ERROR: content/file1.math (Line: 2, Column: 1)
                Duplicate defined symbol 'x' in `given:`

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
                    satisfying:
                    . exists: y, y
                      suchThat: 'x'
                    written: ""
                    called: ""
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 4, Column: 3)
                Duplicate defined symbol 'y' in `exists:`

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
                    satisfying:
                    . existsUnique: y, y
                      suchThat: 'x'
                    written: ""
                    called: ""
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 4, Column: 3)
                Duplicate defined symbol 'y' in `existsUnique:`

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
                    satisfying:
                    . forAll: y, y
                      then: 'x'
                    written: ""
                    called: ""
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 4, Column: 3)
                Duplicate defined symbol 'y' in `forAll:`

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
                    satisfying:
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
                Duplicate defined symbol 'y' in `forAll:`

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
                    satisfying:
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
                Duplicate defined symbol 'y' in `forAll:`

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
                    satisfying:
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
                Duplicate defined symbol 'y' in `exists:`

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
                    satisfying:
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
                Duplicate defined symbol 'y' in `existsUnique:`

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
                    satisfying: X
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
                    satisfying: X
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
                    satisfying: "a function on A? to B?"
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
                    satisfying: "a function on A? to B?"
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
                    satisfying: 'X'
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
                    satisfying: 'X'
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
                    satisfying: 'X'
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
                    satisfying: 'X'
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
                        satisfying: "something"
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
                        satisfying: "something"
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
                        satisfying:
                        . forAll: x
                          then: x
                        written: "something"
                        called: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 4, Column: 3)
                Duplicate defined symbol 'x' in `forAll:`

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
                    expressing:
                    . 'x'
                    . 'X'
                    . 'f'
                    written: "something"
                    called: "something"

                    Theorem:
                    given: f(x)
                    then:
                    . 'x'
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

    @Test
    fun `check does not report errors when placeholder variables are shared in square brackets`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something[x]{f}]
                    Defines: X
                    expressing:
                    . 'x'
                    . 'X'
                    . 'f'
                    written: "something"
                    called: "something"

                    Theorem:
                    given: f(x)
                    then:
                    . 'x'
                    . '\something[x]{f(x)}'
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
    fun `check does not report errors when placeholder variables are shared in square brackets with using section with a function`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something[x]{f}]
                    Defines: X
                    expressing: "something"
                    written: "something"
                    called: "something"

                    Theorem:
                    given: f(x)
                    then: '\something[x]{f}'
                    using: 'g(x) := x'
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
    fun `check does not report errors when placeholder variables are shared in square brackets with using section with function with new placeholder vars`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something[x]{f}]
                    Defines: X
                    expressing: "something"
                    written: "something"
                    called: "something"

                    Theorem:
                    given: f(x)
                    then: '\something[x]{f}'
                    using: 'g(t) := t'
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
    fun `check does not report errors when placeholder variables are shared in square brackets with using section with identifier`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something[x]{f}]
                    Defines: X
                    expressing: "something"
                    written: "something"
                    called: "something"

                    Theorem:
                    given: f(x)
                    then: '\something[x]{f}'
                    using: 'g := 0'
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
    fun `check does not report errors when placeholder variables are shared in square brackets with using section with signature`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something[x]{f}]
                    Defines: X
                    expressing: "something"
                    written: "something"
                    called: "something"

                    Theorem:
                    given: f(x)
                    then: '\something[x]{f}'
                    using: '\g(x) := x'
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
    fun `check does not report errors when placeholder variables are shared in square brackets with using section with signature operator using new placeholder vars`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something[x]{f}]
                    Defines: X
                    expressing: "something"
                    written: "something"
                    called: "something"

                    Theorem:
                    given: f(x)
                    then: '\something[x]{f}'
                    using: 'u \op/ v := 0'
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
    fun `check does not report errors when placeholder variables are shared in square brackets with using section with signature operator re-using placeholder var`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something[x]{f}]
                    Defines: X
                    expressing: "something"
                    written: "something"
                    called: "something"

                    Theorem:
                    given: f(x)
                    then: '\something[x]{f}'
                    using: 'x \op/ y := 0'
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
    fun `check does not report errors when placeholder variables are shared in square brackets with using section with operator`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something[x]{f}]
                    Defines: X
                    expressing: "something"
                    written: "something"
                    called: "something"

                    Theorem:
                    given: f(x)
                    then: '\something[x]{f}'
                    using: 'u ** v := 0'
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
    fun `check does not report errors when placeholder variables are shared in square brackets with using section with operator re-using placeholder vars`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something[x]{f}]
                    Defines: X
                    expressing: "something"
                    written: "something"
                    called: "something"

                    Theorem:
                    given: f(x)
                    then: '\something[x]{f}'
                    using: 'x ** y := 0'
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
    fun `check does not report errors when placeholder variables are shared in function like mappings`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    satisfying: "something"
                    written: "something"
                    called: "something"


                    [\something.else]
                    Defines: f(x)
                    means: 'f(x) is \something'
                    written: "something else"
                    called: "something else"
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
    fun `check does not report errors when placeholder variables are shared in sequence like mappings`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    satisfying: "something"
                    written: "something"
                    called: "something"


                    [\something.else]
                    Defines: {x_{i}}_{i}
                    means:
                    . '{x_{i}}_{i} is \something'
                    satisfying:
                    . '{x_{i}}_{i}'
                    . 'x_{i}'
                    written: "something else"
                    called: "something else"
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
    fun `check does not report errors for function like mappings used as signature vars`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    satisfying: "something"
                    written: "something"
                    called: "something"


                    [\something.else{f(x)}]
                    Defines: X
                    when: 'f(x) is \something'
                    satisfying:
                    . 'f(x) = 0'
                    written: "something else"
                    called: "something else"


                    [\another.thing{f(x)}]
                    Defines: X
                    satisfying:
                    . 'f(x) = 0'
                    written: "another thing"
                    called: "another thing"
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
    fun `check does not report errors for sequence like mappings used as signature vars`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    satisfying: "something"
                    written: "something"
                    called: "something"


                    [\something.else{{x_{i}}_{i}}]
                    Defines: X
                    when: '{x_{i}}_{i} is \something'
                    satisfying:
                    . '{x_{i}}_{i} = 0'
                    written: "something else"
                    called: "something else"


                    [\another.thing{{x_{i}}_{i}}]
                    Defines: X
                    satisfying:
                    . '{x_{i}}_{i} = 0'
                    written: "another thing"
                    called: "another thing"
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
    fun `check reports errors for function like mappings that use new placeholder vars`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: f(x)
                    satisfying: "something"
                    written: "something"
                    called: "something"


                    [\something.else{f(x)}]
                    Defines: X
                    when:
                    . 'f(t) is \something'
                    . 'f(u) is \something'
                    satisfying:
                    . 'f(v) = 0'
                    written: "something else"
                    called: "something else"


                    [\another.thing{{x_{i}}_{i}}]
                    Defines: X
                    satisfying:
                    . 'x_{t} = 0'
                    written: "another thing"
                    called: "another thing"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 14, Column: 3)
                Undefined symbol 'v'

                ERROR: content/file1.math (Line: 22, Column: 3)
                Undefined symbol 't'

                FAILED
                Processed 1 file
                2 errors detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 2)
    }

    @Test
    fun `check reports errors for sequence like mappings that use new placeholder vars`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: f(x)
                    satisfying: "something"
                    written: "something"
                    called: "something"


                    [\something.else{{x_{i}}_{i}}]
                    Defines: X
                    when: '{x_{j}}_{j} is \something'
                    satisfying:
                    . '{x_{k}}_{k}'
                    . '{x_{t}}_{t}'
                    written: "something else"
                    called: "something else"


                    [\another.thing{{x_{i}}_{i}}]
                    Defines: X
                    when:
                    . '{x_{j}}_{j}'
                    satisfying: "something"
                    written: "another thing"
                    called: "another thing"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 12, Column: 3)
                Undefined symbol 'k'

                ERROR: content/file1.math (Line: 13, Column: 3)
                Undefined symbol 't'

                ERROR: content/file1.math (Line: 21, Column: 3)
                Undefined symbol 'j'

                FAILED
                Processed 1 file
                3 errors detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 3)
    }

    @Test
    fun `check reports errors for sequence like mappings that don't consistently use the same placeholder vars`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    satisfying: "something"
                    written: "something"
                    called: "something"


                    [\something.else{{x_{i}}_{j}}]
                    Defines: X
                    when: '{x_{i}}_{j} is \something'
                    satisfying:
                    . '{x_{i}}_{j} := 0'
                    written: "something else"
                    called: "something else"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 1, Column: 24)
                An item of the form {x_{A}}_{B} must specify the exact same symbols in A and B

                ERROR: content/file1.math (Line: 1, Column: 8)
                An item of the form {x_{A}}_{B} must specify the exact same symbols in A and B

                ERROR: content/file1.math (Line: 1, Column: 8)
                An item of the form {x_{A}}_{B} must specify the exact same symbols in A and B

                ERROR: content/file1.math (Line: 10, Column: 14)
                An item of the form {x_{A}}_{B} must specify the exact same symbols in A and B

                ERROR: content/file1.math (Line: 12, Column: 10)
                An item of the form {x_{A}}_{B} must specify the exact same symbols in A and B

                FAILED
                Processed 1 file
                5 errors detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 5)
    }

    @Test
    fun `check does not report errors when defining X colon equals`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\f]
                    Defines: X := (a, b)
                    satisfying:
                    . 'X = a'
                    . 'X = b'
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
    fun `check does not report errors when referencing variables in {} when using section exists`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    satisfying: "something"
                    written: "something"
                    called: "something"


                    [\some.function{S}]
                    Defines: beta
                    when: 'S is \something'
                    satisfying: "something"
                    using: 'x := x'
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
    fun `check does not report errors when referencing a function in a using section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    Theorem:
                    given: x
                    then: 'f(x)'
                    using: 'f(y) := y'
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
    fun `check does not report errors when referencing a signature in a using section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    Theorem:
                    given: x
                    then: '\f(x)'
                    using: '\f(y) := y'
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
    fun `check does not report undefined symbols for symbols defined in Defines-given`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    satisfying: "something"
                    written: "something"
                    called: "something"


                    [\f{X}]
                    Defines: Y
                    given: S, <
                    when: 'X := (S, <) is \something'
                    satisfying: "something"
                    written: "f"
                    called: "f"
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
    fun `check does not report undefined symbols for symbols defined in States-given`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    satisfying: "something"
                    written: "something"
                    called: "something"


                    [\f{X}]
                    States:
                    given: S, <
                    that: 'X := (S, <) is \something'
                    written: "f"
                    called: "f"
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
    fun `check reports errors of duplicate base types when duplicate base types exist not including means`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    satisfying: "something"
                    written: "something"
                    called: "something"


                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"
                    called: "something else"


                    [\f{X}]
                    States:
                    when:
                    . 'X is \something'
                    . 'X is \something.else'
                    that: "something"
                    written: "f"
                    called: "f"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 18, Column: 3)
                'X' has more than one base type: {\something, \something.else}
                Found type paths:
                {\something}, {\something.else}

                ERROR: content/file1.math (Line: 19, Column: 3)
                'X' has more than one base type: {\something, \something.else}
                Found type paths:
                {\something}, {\something.else}

                FAILED
                Processed 1 file
                2 errors detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 2)
    }

    @Test
    fun `check reports errors of duplicate base types when duplicate base types exist including means`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    satisfying: "something"
                    written: "something"
                    called: "something"


                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"
                    called: "something else"


                    [\f{x}]
                    Defines: X
                    means: 'X is \something'
                    satisfying: 'X is \something.else'
                    written: "f"
                    called: "f"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 17, Column: 8)
                'X' has more than one base type: {\something, \something.else}
                Found type paths:
                {\something}, {\something.else}

                ERROR: content/file1.math (Line: 18, Column: 13)
                'X' has more than one base type: {\something, \something.else}
                Found type paths:
                {\something}, {\something.else}

                FAILED
                Processed 1 file
                2 errors detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 2)
    }

    @Test
    fun `check does not report errors of duplicate base types when providing-view-as causes no duplicate base types`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    satisfying: "something"
                    written: "something"
                    called: "something"


                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"
                    called: "something else"
                    Providing:
                    . view:
                      as: '\something'
                      via: 'X'


                    [\f{X}]
                    States:
                    when:
                    . 'X is \something'
                    . 'X is \something.else'
                    that: "something"
                    written: "f"
                    called: "f"
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
    fun `check does not report errors of duplicate base types when a Defines doesn't define an identifier`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    satisfying: "something"
                    written: "something"
                    called: "something"


                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"
                    called: "something else"


                    [\another]
                    Defines: X
                    satisfying: "another"
                    written: "another"
                    called: "another"


                    [\f]
                    Defines: (S, <)
                    means: 'S is \something'
                    satisfying:
                    . '< is \something.else'
                    . '0 < 1'
                    written: "\textrm{ordered set}"
                    called: "ordered set"
                    Providing:
                    . view:
                      as: '\another'
                      via: 'S'
                    . membership:
                      through: 'S'
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
    fun `check does not report errors of duplicate base types when a Defines uses a viewing-as without a identifier target`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    satisfying: "something"
                    written: "something"
                    called: "something"


                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"
                    called: "something else"


                    [\another]
                    Defines: (X, Y)
                    satisfying: "another"
                    written: "another"
                    called: "another"


                    [\f]
                    Defines: (S, <)
                    means: 'S is \something'
                    satisfying:
                    . '< is \something.else'
                    . '0 < 1'
                    written: "\textrm{ordered set}"
                    called: "ordered set"
                    Providing:
                    . view:
                      as: '\another'
                      via: 'S'
                    . membership:
                      through: 'S'
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
    fun `check does reports errors of base types that don't align with 'means' section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\A]
                    Defines: A
                    satisfying: "a"
                    written: "a"


                    [\A1]
                    Defines: A1
                    means: 'A1 is \A'
                    satisfying: "a1"
                    written: "a1"


                    [\A2]
                    Defines: A2
                    means: 'A2 is \A'
                    satisfying: "a2"
                    written: "a2"


                    [\B]
                    Defines: B
                    satisfying: "b"
                    written: "b"


                    [\f]
                    Defines: f
                    means: 'f is \B'
                    satisfying:
                    . 'f is \A1'
                    . 'f is \A2'
                    written: "f"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 29, Column: 8)
                'f' has more than one base type: {\B, \A}
                Found type paths:
                {\B}, {\A1 -> \A}, {\A2 -> \A}

                ERROR: content/file1.math (Line: 31, Column: 3)
                'f' has more than one base type: {\B, \A}
                Found type paths:
                {\B}, {\A1 -> \A}, {\A2 -> \A}

                ERROR: content/file1.math (Line: 32, Column: 3)
                'f' has more than one base type: {\B, \A}
                Found type paths:
                {\B}, {\A1 -> \A}, {\A2 -> \A}

                FAILED
                Processed 1 file
                3 errors detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 3)
    }

    @Test
    fun `check does not report errors of base types that align with 'means' section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\A]
                    Defines: A
                    satisfying: "a"
                    written: "a"


                    [\A1]
                    Defines: A1
                    means: 'A1 is \A'
                    satisfying: "a1"
                    written: "a1"


                    [\A2]
                    Defines: A2
                    means: 'A2 is \A'
                    satisfying: "a2"
                    written: "a2"


                    [\B]
                    Defines: B
                    satisfying: "b"
                    written: "b"


                    [\f]
                    Defines: f
                    means: 'f is \A'
                    satisfying:
                    . 'f is \A1'
                    . 'f is \A2'
                    written: "f"
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
    fun `check does not report errors if multiple 'is' statements exists in 'satisfying' without a 'means' section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\A]
                    Defines: a
                    satisfying: "something"
                    written: "A"


                    [\B]
                    Defines: b
                    means: 'b is \A'
                    satisfying: "something"
                    written: "B"


                    [\C]
                    Defines: c
                    means: 'c is \A'
                    satisfying: "something"
                    written: "C"


                    [\F]
                    Defines: f
                    satisfying:
                    . 'f is \B'
                    . 'f is \C'
                    written: "F"
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
    fun `check does not report errors if multiple 'is' statements exists in 'expressing' without a 'means' section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\A]
                    Defines: a
                    satisfying: "something"
                    written: "A"


                    [\B]
                    Defines: b
                    means: 'b is \A'
                    satisfying: "something"
                    written: "B"


                    [\C]
                    Defines: c
                    means: 'c is \A'
                    satisfying: "something"
                    written: "C"


                    [\F]
                    Defines: f
                    expressing:
                    . 'f is \B'
                    . 'f is \C'
                    written: "F"
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
    fun `check does not report errors if multiple 'in' statements exists in 'satisfying' without a 'means' section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\A]
                    Defines: {a}
                    expressing: "something"
                    written: "A"


                    [\B]
                    Defines: {b}
                    means: 'b in \A'
                    expressing: "something"
                    written: "B"


                    [\C]
                    Defines: {c}
                    means: 'c in \A'
                    expressing: "something"
                    written: "C"


                    [\F]
                    Defines: f
                    satisfying:
                    . 'f in \B'
                    . 'f in \C'
                    written: "F"
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
    fun `check does not report errors if multiple 'in' statements exists in 'expressing' without a 'means' section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\A]
                    Defines: {a}
                    expressing: "something"
                    written: "A"


                    [\B]
                    Defines: {b}
                    means: 'b in \A'
                    expressing: "something"
                    written: "B"


                    [\C]
                    Defines: {c}
                    means: 'c in \A'
                    expressing: "something"
                    written: "C"


                    [\F]
                    Defines: f
                    expressing:
                    . 'f in \B'
                    . 'f in \C'
                    written: "F"
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
    fun `check does not report errors if a 'means' section exists with an 'is' statement and no 'satisfying' or 'expressing' section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\A]
                    Defines: a
                    satisfying: "something"
                    written: "A"


                    [\F]
                    Defines: f
                    means: 'f is \A'
                    written: "F"
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
    fun `check does not report errors if a 'means' section exists with an 'in' statement and no 'satisfying' or 'expressing' section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\A]
                    Defines: {a}
                    expressing: "something"
                    written: "A"


                    [\F]
                    Defines: f
                    means: 'f in \A'
                    written: "F"
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
    fun `check does not report errors if a 'means' section exists with an 'is' statement and 'satisfying' section without 'is' or 'in'`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\A]
                    Defines: a
                    satisfying: "something"
                    written: "A"


                    [\F]
                    Defines: f
                    means: 'f is \A'
                    satisfying: "something"
                    written: "F"
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
    fun `check does not report errors if a 'means' section exists with an 'is' statement and 'expressing' section without 'is' or 'in'`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\A]
                    Defines: a
                    satisfying: "something"
                    written: "A"


                    [\F]
                    Defines: f
                    means: 'f is \A'
                    expressing: "something"
                    written: "F"
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
    fun `check does not report errors if a 'means' section exists with an 'in' statement and 'satisfying' section without 'is' or 'in'`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\A]
                    Defines: {a}
                    expressing: "something"
                    written: "A"


                    [\F]
                    Defines: f
                    means: 'f in \A'
                    satisfying: "something"
                    written: "F"
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
    fun `check does not report errors if a 'means' section exists with an 'in' statement and 'expressing' section without 'is' or 'in'`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\A]
                    Defines: {a}
                    expressing: "something"
                    written: "A"


                    [\F]
                    Defines: f
                    means: 'f in \A'
                    expressing: "something"
                    written: "F"
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
    fun `check does not report for symbols introduced in given section and used in colon equals`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    given: a, b
                    satisfying: 'X := (a, b)'
                    written: "something"
                    called: "something"


                    [\something.else{X}]
                    States:
                    given: a, b
                    when: 'X := (a, b)'
                    that: "something"
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
    fun `check reports errors for symbols used in when section and not introduced in given section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    when: '(a, b)'
                    satisfying: "something"
                    written: "something"
                    called: "something"


                    [\something.else{X}]
                    States:
                    when: 'X := (a, b)'
                    that: "something"
                    written: "something"
                    called: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 3, Column: 7)
                Undefined symbol 'a'

                ERROR: content/file1.math (Line: 3, Column: 7)
                Undefined symbol 'b'

                ERROR: content/file1.math (Line: 11, Column: 7)
                Undefined symbol 'a' in `:=`

                ERROR: content/file1.math (Line: 11, Column: 7)
                Undefined symbol 'b' in `:=`

                ERROR: content/file1.math (Line: 11, Column: 7)
                Undefined symbol 'a'

                ERROR: content/file1.math (Line: 11, Column: 7)
                Undefined symbol 'b'

                FAILED
                Processed 1 file
                6 errors detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 6)
    }

    @Test
    fun `check does not report errors for signatures introduced as operators in given section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: f(x, y)
                    given: R := (X, *)
                    satisfying: 'f(x, y) := x * y'
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
    fun `check does not report errors for placeholder vars in sequence definitions in when sections`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: A
                    given: X, {x_{i}}_{i}
                    when: 'X := {x_{i}}_{i} is \something'
                    satisfying: "something"
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
    fun `check reports errors for undefined vars used in function like mapping calls`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: f(x)
                    satisfying: 'f(t)'
                    written: "something"
                    called: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 3, Column: 13)
                Undefined symbol 't'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check does not reports errors for placeholder vars used in function like mapping calls`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: f(x)
                    satisfying: 'f(x)'
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
    fun `check reports errors for undefined vars used in sequence like mapping calls`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: {x_{i}}_{i}
                    satisfying: 'x_{j}'
                    written: "something"
                    called: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 3, Column: 13)
                Undefined symbol 'j'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check does not reports errors for placeholder vars used in sequence like mapping calls`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: {x_{i}}_{i}
                    satisfying: 'x_{i}'
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

    /*
     * TODO: When the operator resolution algorithm is implemented, fix
     *       this test to verify it.
    @Test
    fun `check reports errors for using undefined operator signatures`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: x
                    given: u, v, -, a, b
                    when: 'a - b := a ** b'
                    satisfying:
                    . 'u - v'
                    written: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 4, Column: 7)
                Undefined signature '**'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }
     */

    @Test
    fun `check does not report errors for using defined operator signatures`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [a ** b]
                    Defines: x
                    satisfying: "something"
                    written: "a?? +++ b??"


                    [\something]
                    Defines: x
                    given: u, v, -, a, b
                    when: 'a - b := a ** b'
                    satisfying:
                    . 'u - v'
                    written: "something"
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
    fun `check reports errors for using a command on right-hand-side of an 'is' that has an expresses section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    expressing: 'X := 0'
                    written: "something"


                    [\something.else]
                    Defines: X
                    means: 'X is \something'
                    satisfying: 'X is \something'
                    written: "something else"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 9, Column: 8)
                The right-hand-side of an `is` cannot reference a `Defines:` with an `expressing:` section but found '\something'

                ERROR: content/file1.math (Line: 10, Column: 13)
                The right-hand-side of an `is` cannot reference a `Defines:` with an `expressing:` section but found '\something'

                FAILED
                Processed 1 file
                2 errors detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 2)
    }

    @Test
    fun `check does not report errors for using a command on right-hand-side of an 'is' that does not have an expresses section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    satisfying: "something"
                    written: "something"


                    [\something.else]
                    Defines: X
                    means: 'X is \something'
                    written: "something else"
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
    fun `check reports errors for using a command on right-hand-side of an colon equals that does not has an expresses section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    satisfying: "something"
                    written: "something"


                    [\something.else]
                    Defines: X
                    satisfying: 'X := \something'
                    written: "something else"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 9, Column: 13)
                The right-hand-side of an `:=` cannot reference a `Defines:` without an `expressing:` section but found '\something'

                ERROR: content/file1.math (Line: 9, Column: 13)
                Cannot use '\something' in a non-`is`, non-`as`, or non-`in` statement since its definition doesn't have an `expressing:` section

                FAILED
                Processed 1 file
                2 errors detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 2)
    }

    @Test
    fun `check does not report errors for using a command on right-hand-side of an colon equals that has an expresses section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    expressing: 'X := 0'
                    written: "something"


                    [\something.else]
                    Defines: X
                    satisfying: 'X := \something'
                    written: "something else"
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
    fun `check reports errors for id input var used in lhs of is statement in satisfying`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"


                    [\another.thing]
                    Defines: X
                    means: 'X is \something.else'
                    written: "another thing"


                    [\something{A}]
                    Defines: X
                    means: 'X is \another.thing'
                    satisfying: 'A is \something.else'
                    written: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 16, Column: 13)
                A `satisfying:` or `expressing:` section cannot describe a symbol introduced in a [...] or `given:` section but found 'A'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check does not report errors for id input var used in lhs of is statement in when`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"


                    [\something{A}]
                    Defines: X
                    when: 'A is \something.else'
                    satisfying: "something"
                    written: "something"
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
    fun `check reports errors for id input var used in lhs of colon equals statement in satisfying`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    expressing: "something else"
                    written: "something else"


                    [\something{A}]
                    Defines: X
                    satisfying: 'A := \something.else'
                    written: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 9, Column: 13)
                A `satisfying:` or `expressing:` section cannot describe a symbol introduced in a [...] or `given:` section but found 'A'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check does not report errors for id input var used in lhs of colon equals statement in when`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    expressing: "something else"
                    written: "something else"


                    [\something{A}]
                    Defines: X
                    when: 'A := \something.else'
                    satisfying: "something"
                    written: "something"
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
    fun `check reports errors for id input var used in lhs of is statement in expresses`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"


                    [\another.thing]
                    Defines: X
                    means: 'X is \something.else'
                    satisfying: "another thing"
                    written: "another thing"


                    [\something{A}]
                    Defines: X
                    means: 'X is \another.thing'
                    expressing: 'A is \something.else'
                    written: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 17, Column: 13)
                A `satisfying:` or `expressing:` section cannot describe a symbol introduced in a [...] or `given:` section but found 'A'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check reports errors for id input var used in lhs of colon equals statement in expresses`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    expressing: "something else"
                    written: "something else"


                    [\something{A}]
                    Defines: X
                    expressing: 'A := \something.else'
                    written: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 9, Column: 13)
                A `satisfying:` or `expressing:` section cannot describe a symbol introduced in a [...] or `given:` section but found 'A'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check reports errors for given section input var used in lhs of is statement in satisfying`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"


                    [\another.thing]
                    Defines: X
                    means: 'X is \something.else'
                    written: "another thing"


                    [\something]
                    Defines: X
                    given: A
                    means: 'A is \another.thing'
                    satisfying: 'A is \something.else'
                    written: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 17, Column: 13)
                A `satisfying:` or `expressing:` section cannot describe a symbol introduced in a [...] or `given:` section but found 'A'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check does not report errors for given section input var used in lhs of is statement in when`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"


                    [\something]
                    Defines: X
                    given: A
                    when: 'A is \something.else'
                    satisfying: "something"
                    written: "something"
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
    fun `check reports errors for given section input var used in lhs of colon equals statement in satisfying`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    expressing: "something else"
                    written: "something else"


                    [\something]
                    Defines: X
                    given: A
                    satisfying: 'A := \something.else'
                    written: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 10, Column: 13)
                A `satisfying:` or `expressing:` section cannot describe a symbol introduced in a [...] or `given:` section but found 'A'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check does not report errors for given section input var used in lhs of colon equals statement in when`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    expressing: "something else"
                    written: "something else"


                    [\something]
                    Defines: X
                    given: A
                    when: 'A := \something.else'
                    satisfying: "something"
                    written: "something"
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
    fun `check reports errors for given section input var used in lhs of is statement in expresses`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"


                    [\another.thing]
                    Defines: X
                    means: 'X is \something.else'
                    satisfying: "another thing"
                    written: "another thing"


                    [\something]
                    Defines: X
                    given: A
                    means: 'X is \another.thing'
                    expressing: 'A is \something.else'
                    written: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 18, Column: 13)
                A `satisfying:` or `expressing:` section cannot describe a symbol introduced in a [...] or `given:` section but found 'A'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check reports errors for given section input var used in lhs of colon equals statement in expresses`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    expressing: "something else"
                    written: "something else"


                    [\something]
                    Defines: X
                    given: A
                    expressing: 'A := \something.else'
                    written: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 10, Column: 13)
                A `satisfying:` or `expressing:` section cannot describe a symbol introduced in a [...] or `given:` section but found 'A'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check reports errors for Defines section var used in lhs of is statement in when`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"


                    [\something]
                    Defines: X
                    when: 'X is \something.else'
                    satisfying: "something"
                    written: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 9, Column: 7)
                A `when:` section cannot describe a symbol introduced in a `Defines:` section but found 'X'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check does not report errors for Defines section var used in lhs of is statement in satisfying`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"


                    [\another.thing]
                    Defines: X
                    means: 'X is \something.else'
                    written: "another thing"


                    [\something]
                    Defines: X
                    means: 'X is \another.thing'
                    satisfying: 'X is \something.else'
                    written: "something"
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
    fun `check does not report errors for Defines section var used in lhs of is statement in expresses`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"


                    [\another.thing]
                    Defines: X
                    means: 'X is \something.else'
                    satisfying: "another thing"
                    written: "another thing"


                    [\something]
                    Defines: X
                    means: 'X is \another.thing'
                    expressing: 'X is \something.else'
                    written: "something"
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
    fun `check reports errors for Defines section var used in lhs of colon equals statement in when`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    expressing: "something else"
                    written: "something else"


                    [\something]
                    Defines: X
                    when: 'X := \something.else'
                    satisfying: "something"
                    written: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 9, Column: 7)
                A `when:` section cannot describe a symbol introduced in a `Defines:` section but found 'X'

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check does not report errors for Defines section var used in lhs of colon equals statement in satisfying`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    expressing: "something else"
                    written: "something else"


                    [\something]
                    Defines: X
                    satisfying: 'X := \something.else'
                    written: "something"
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
    fun `check does not report errors for Defines section var used in lhs of colon equals statement in expresses`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    expressing: "something else"
                    written: "something else"


                    [\something]
                    Defines: X
                    expressing: 'X := \something.else'
                    written: "something"
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
    fun `check does not report errors for id input vars also declared in Defines used in lhs of is statement in satisfying`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\real.numbers]
                    Defines: R
                    satisfying: "something"
                    written: "something"


                    [x \real.+/ y]
                    Defines: f(x, y)
                    given: R := (X, +, *, 0, 1, <)
                    when: 'R is \real.numbers'
                    satisfying: 'f(x, y) := x + y'
                    written: "x?? + y??"
                    called: "real addition of x? and y?"
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
    fun `check does not report errors for id input vars also declared in Defines used in lhs of is statement in expresses`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\real.numbers]
                    Defines: R
                    satisfying: "something"
                    written: "something"


                    [x \real.+/ y]
                    Defines: f(x, y)
                    given: R := (X, +, *, 0, 1, <)
                    when: 'R is \real.numbers'
                    expressing: 'f(x, y) := x + y'
                    written: "x?? + y??"
                    called: "real addition of x? and y?"
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
    fun `check does not report errors for an edge case defining a function declared in a given section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [a \integers.+/ b]
                    Defines: f(a, b)
                    expressing: "something"
                    written: "a? + b?"


                    [a \integers.-/ b]
                    Defines: f(a, b)
                    expressing: "something"
                    written: "a? - b?"


                    [a \integers.gt/ b]
                    States:
                    that: "something"
                    written: "a? > b?"


                    [a \integers.leq/ b]
                    States:
                    that: "something"
                    written: "a? \leq b?"


                    [\finite.sum[i]_{a}^{b}{f(i)}]
                    Defines: L
                    given: S(n)
                    when:
                    . 'S(a) := f(a)'
                    . forAll: j
                      suchThat:
                      . 'j \integers.gt/ a'
                      . 'j \integers.leq/ b'
                      then:
                      . 'S(j) := f(j) \integers.+/ S(j \integers.-/ 1)'
                    expressing:
                    . 'L := S(b)'
                    written: "\displaystyle \sum_{i? = a?}^{b?} f?"
                    called: "finite sum of f? from a? to b?"
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
    fun `check reports errors for non-expresses signatures used in non-'is' statement`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"


                    [\something]
                    Defines: X
                    satisfying: '\something.else'
                    written: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 9, Column: 13)
                Cannot use '\something.else' in a non-`is`, non-`as`, or non-`in` statement since its definition doesn't have an `expressing:` section

                FAILED
                Processed 1 file
                1 error detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check does not report errors for non-expresses signatures used in 'is' statements`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"


                    [\something]
                    Defines: X
                    means: 'X is \something.else'
                    written: "something"
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
    fun `check reports errors for non-expresses signatures used in 'in' statements`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"


                    [\some.states]
                    States:
                    that: "something"
                    written: "something"


                    [\some.axiom]
                    Axiom:
                    then: "something"


                    [\some.conjecture]
                    Conjecture:
                    then: "something"


                    [\some.theorem]
                    Theorem:
                    then: "something"


                    [\something]
                    Defines: X
                    means:
                    . 'X in \something.else'
                    satisfying:
                    . 'X in \some.states'
                    . 'X in \some.axiom'
                    . 'X in \some.conjecture'
                    . 'X in \some.theorem'
                    written: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 35, Column: 3)
                No matching definition found for \some.conjecture

                ERROR: content/file1.math (Line: 36, Column: 3)
                No matching definition found for \some.theorem

                ERROR: content/file1.math (Line: 31, Column: 3)
                The right-hand-side of an `in` cannot reference a `Defines:` without an `expressing:` section but found '\something.else'

                ERROR: content/file1.math (Line: 33, Column: 3)
                The right-hand-side of an `in` cannot reference a `Defines:` without an `expressing:` section but found '\some.states'

                ERROR: content/file1.math (Line: 34, Column: 3)
                The right-hand-side of an `in` cannot reference a `Defines:` without an `expressing:` section but found '\some.axiom'

                ERROR: content/file1.math (Line: 35, Column: 3)
                The right-hand-side of an `in` cannot reference a `Defines:` without an `expressing:` section but found '\some.conjecture'

                ERROR: content/file1.math (Line: 36, Column: 3)
                The right-hand-side of an `in` cannot reference a `Defines:` without an `expressing:` section but found '\some.theorem'

                FAILED
                Processed 1 file
                7 errors detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 7)
    }

    @Test
    fun `check reports errors for non-expresses signatures used in 'is' statements`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"


                    [\some.states]
                    States:
                    that: "something"
                    written: "something"


                    [\some.axiom]
                    Axiom:
                    then: "something"


                    [\some.conjecture]
                    Conjecture:
                    then: "something"


                    [\some.theorem]
                    Theorem:
                    then: "something"


                    [\something1]
                    Defines: X
                    means:
                    . 'X is \something.else'
                    written: "something"


                    [\something2]
                    Defines: X
                    means:
                    . 'X is \some.states'
                    written: "something"


                    [\something3]
                    Defines: X
                    means:
                    . 'X is \some.axiom'
                    written: "something"


                    [\something4]
                    Defines: X
                    means:
                    . 'X is \some.conjecture'
                    written: "something"


                    [\something5]
                    Defines: X
                    means:
                    . 'X is \some.theorem'
                    written: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 52, Column: 3)
                No matching definition found for \some.conjecture

                ERROR: content/file1.math (Line: 59, Column: 3)
                No matching definition found for \some.theorem

                ERROR: content/file1.math (Line: 38, Column: 3)
                The right-hand-side of an `is` cannot reference a `States:` but found '\some.states'

                ERROR: content/file1.math (Line: 45, Column: 3)
                The right-hand-side of an `is` cannot reference a `Axiom:` but found '\some.axiom'

                ERROR: content/file1.math (Line: 45, Column: 3)
                The right-hand-side of an `is` cannot reference a `Conjecture:` but found '\some.axiom'

                ERROR: content/file1.math (Line: 59, Column: 3)
                The right-hand-side of an `is` cannot reference a `Theorem:` but found '\some.theorem'

                FAILED
                Processed 1 file
                6 errors detected
        """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 6)
    }

    @Test
    fun `check does not reports errors for expresses signatures used in the right-hand-side of an 'in' statement`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"


                    [\f{x}]
                    Defines: y
                    expressing: "something"
                    written: "something"


                    [\another.thing]
                    Defines: X
                    satisfying: "another thing"
                    written: "another thing"


                    [\something]
                    Defines: X
                    means: 'X is \another.thing'
                    satisfying: 'X in \f{\something.else}'
                    written: "something"
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
    fun `check reports errors for non-expresses signatures used in the right-hand-side of an 'in' statement`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"


                    [\f{x}]
                    Defines: y
                    satisfying: "something"
                    written: "something"


                    [\something]
                    Defines: X
                    means: 'X in \f{\something.else}'
                    written: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 15, Column: 8)
                The right-hand-side of an `in` cannot reference a `Defines:` without an `expressing:` section but found '\f'

                FAILED
                Processed 1 file
                1 error detected
            """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check does not report errors for States signatures used in States contexts`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\some.states]
                    States:
                    that: "something"
                    written: "something"


                    [\f{x}]
                    States:
                    that: "something"
                    written: "something"


                    [\something]
                    Defines: X
                    satisfying: '\f{\some.states}'
                    written: "something"
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
    fun `check does not report errors for axioms used Defines in top-level statements`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\some.axiom]
                    Axiom:
                    then: "something"


                    [\something]
                    Defines: X
                    satisfying: '\some.axiom'
                    written: "something"
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
    fun `check reports errors for Defines with both satisfying and expressing sections`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    satisfying: "something else"
                    expressing: "something else"
                    written: "something else"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 1, Column: 1)
                A `Defines:` cannot have both a `satisfying:` and an `expressing:` section

                FAILED
                Processed 1 file
                1 error detected
            """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check does not report errors for Defines with only a satisfying section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"
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
    fun `check does not report errors for Defines with only an expressing section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    expressing: "something else"
                    written: "something else"
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
    fun `check reports errors for Defines without satisfying, expressing, or means sections`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something]
                    Defines: X
                    written: "something else"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 1, Column: 1)
                If a `Defines:` doesn't have a `satisfying:` or `expressing:` section, then it must have a `means:` section

                FAILED
                Processed 1 file
                1 error detected
            """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check does not report errors for Defines with only a means section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\something.else]
                    Defines: X
                    satisfying: "something else"
                    written: "something else"


                    [\something]
                    Defines: X
                    means: 'X is \something.else'
                    written: "something"
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
    fun `check does not report errors for commands introduced inductively in a Defines`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\natural]
                    Defines: n
                    satisfying:
                    . generated:
                      from: 0, succ(x)
                      when: 'x is \natural'
                    written: "\textrm{natural}"


                    [\something]
                    Defines: X
                    means:
                    . 'X := \natural.succ(\natural.0)'
                    written: "something"
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
    fun `check does not report errors for 'satisfying' commands used in 'is' expressions within an argument`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\f]
                    Defines: X
                    satisfying: "something"
                    written: "something"


                    [\g{x}]
                    Defines: y
                    expressing: "something"
                    written: "something"


                    Theorem:
                    given: y
                    then: '\g{y is \f}'
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
    fun `check does not report errors for using axiom signatures correctly`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\some.axiom]
                    Axiom:
                    given: x
                    then: x


                    [\some.other.axiom]
                    Axiom:
                    given: x, y
                    then: "something"


                    [\f]
                    Defines: X
                    satisfying:
                    . '\some.axiom'
                    . '\some.axiom:given{X}'
                    . '\some.other.axiom:given{X, X}'
                    written: "something"
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
    fun `check reports errors for using axiom signatures incorrectly`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\some.axiom]
                    Axiom:
                    then: "something"


                    [\another.axiom]
                    Axiom:
                    given: x, y, z
                    then: "something"


                    [\f]
                    Defines: X
                    satisfying:
                    . '\some.axiom:given{}'
                    . '\some.axiom:given{X}'
                    . '\another.axiom:given{X}'
                    written: "something"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 15, Column: 3)
                No matching definition found for \some.axiom:given{}

                ERROR: content/file1.math (Line: 16, Column: 3)
                No matching definition found for \some.axiom:given{X}

                ERROR: content/file1.math (Line: 17, Column: 3)
                Expected exactly 3 arguments but found 1 for '{X}'

                FAILED
                Processed 1 file
                3 errors detected
            """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 3)
    }

    @Test
    fun `check does not report errors for multiple base types with a tuple with a means section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\A]
                    Defines: (X, +, *, 0, 1)
                    satisfying: "something"
                    written: "A"


                    [\B]
                    Defines: Z := (X, +, *)
                    means: 'Z is \A'
                    written: "B"


                    [\C]
                    Defines: Z := (X, +, *)
                    means: '(X, +, *) is \A'
                    written: "C"


                    [\D]
                    Defines: Z := (X, +, *)
                    means: '(X, +) is \A'
                    written: "D"


                    [\E]
                    Defines: Z := (X, +, *)
                    means: 'X is \A'
                    written: "D"


                    Theorem:
                    given: Z
                    then:
                    . 'Z is \A'
                    . 'Z is \B'
                    . 'Z is \C'
                    . 'Z is \D'
                    . 'Z is \E'
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
    fun `check does not report errors when using 'notin'`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    Theorem:
                    given: x, X
                    then: 'x notin X'
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
    fun `check does not report errors when using '!='`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    Theorem:
                    given: x, X
                    then: 'x != X'
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
    fun `check does not report errors when using form 'xyz dot abc'`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\group]
                    Defines: G := (X, *)
                    satisfying: "something"
                    written: "\textrm{group}"
                    Providing:
                    . symbols: *, X
                      where:
                      . 'a * b := a X.* b'
                      . 'X := G.X'
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
    fun `check does not report errors when using multiple 'as' statements`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\X]
                    Defines: X
                    satisfying: "something"
                    written: "X"


                    Theorem:
                    given: x
                    then: 'x as \X as \X'
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
    fun `check reports errors when a 'means' section has more than one statement`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\A]
                    Defines: A
                    satisfying: "something"
                    written: "A"


                    [\B]
                    Defines: B
                    satisfying: "something"
                    written: "B"


                    [\C]
                    Defines: C
                    means:
                    . 'C is \A'
                    . 'C is \B'
                    written: "X"
                """.trimIndent())),
            expectedOutput =
                """
                ERROR: content/file1.math (Line: 15, Column: 1)
                A `means:` section must contain exactly one statement.

                FAILED
                Processed 1 file
                1 error detected
            """.trimIndent(),
            expectedExitCode = 1,
            expectedNumErrors = 1)
    }

    @Test
    fun `check does not report errors when a 'means' section has exactly one statement`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\A]
                    Defines: A
                    satisfying: "something"
                    written: "A"


                    [\B]
                    Defines: B
                    means: 'B is \A'
                    written: "B"
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
    fun `check does not report errors function args specified in when section`() {
        runCheckTest(
            files =
                listOf(
                    PathAndContent(
                        path = listOf("content", "file1.math"),
                        content =
                            """
                    [\a]
                    Defines: a
                    satisfying: "something"
                    written: "something"


                    [\f(x)]
                    Defines: f(x)
                    when: 'x is \a'
                    expressing: "something"
                    written: "f"


                    [x + y]
                    Defines: f(x, y)
                    when: 'x, y is \a'
                    expressing: "something"
                    written: "x? + y?"
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

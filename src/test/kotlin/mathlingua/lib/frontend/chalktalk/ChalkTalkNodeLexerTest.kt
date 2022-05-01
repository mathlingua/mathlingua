/*
 * Copyright 2022 Dominic Kramer
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

package mathlingua.lib.frontend.chalktalk

import kotlin.test.Test
import mathlingua.lib.frontend.MetaData
import mathlingua.lib.frontend.ast.BeginArgument
import mathlingua.lib.frontend.ast.BeginGroup
import mathlingua.lib.frontend.ast.BeginSection
import mathlingua.lib.frontend.ast.EndArgument
import mathlingua.lib.frontend.ast.EndGroup
import mathlingua.lib.frontend.ast.EndSection
import mathlingua.lib.frontend.ast.Id
import mathlingua.lib.frontend.ast.Name
import mathlingua.lib.frontend.ast.NameAssignment
import mathlingua.lib.frontend.ast.NameParam
import mathlingua.lib.frontend.ast.NodeLexerToken
import mathlingua.lib.frontend.ast.OperatorName
import mathlingua.lib.frontend.ast.RegularFunction
import mathlingua.lib.frontend.ast.Set
import mathlingua.lib.frontend.ast.Statement
import mathlingua.lib.frontend.ast.SubAndRegularParamFunction
import mathlingua.lib.frontend.ast.SubAndRegularParamFunctionSequence
import mathlingua.lib.frontend.ast.SubParamFunction
import mathlingua.lib.frontend.ast.SubParamFunctionSequence
import mathlingua.lib.frontend.ast.Text
import mathlingua.lib.frontend.ast.TextBlock
import mathlingua.lib.frontend.ast.Tuple
import strikt.api.expect
import strikt.assertions.isEmpty
import strikt.assertions.isEqualTo

internal class ChalkTalkNodeLexerTest {
    @Test fun `correctly parses empty input`() = runTest("", emptyList())

    @Test
    fun `correctly parses single text block`() =
        runTest(
            "::some text::",
            listOf(
                TextBlock(
                    text = "some text",
                    metadata = MetaData(row = 0, column = 0, isInline = false))))

    @Test
    fun `correctly parses single section group without an arg`() =
        runTest(
            "someName:",
            listOf(
                BeginGroup(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses a single section group with a name arg`() =
        runTest(
            "someName: xyz",
            listOf(
                BeginGroup(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 0, column = 10, isInline = true)),
                Name(text = "xyz", metadata = MetaData(row = 0, column = 10, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses a single section group with an indented name arg`() =
        runTest(
            """
            someName:
            . xyz
        """.trimIndent(),
            listOf(
                BeginGroup(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 1, column = 2, isInline = false)),
                Name(text = "xyz", metadata = MetaData(row = 1, column = 2, isInline = false)),
                EndArgument,
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses a single section group with mixed name args`() =
        runTest(
            """
            someName: a, b
            . c, d
            . e
        """.trimIndent(),
            listOf(
                BeginGroup(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 0, column = 10, isInline = true)),
                Name(text = "a", metadata = MetaData(row = 0, column = 10, isInline = true)),
                EndArgument,
                BeginArgument(metadata = MetaData(row = 0, column = 13, isInline = true)),
                Name(text = "b", metadata = MetaData(row = 0, column = 13, isInline = true)),
                EndArgument,
                BeginArgument(metadata = MetaData(row = 1, column = 2, isInline = false)),
                Name(text = "c", metadata = MetaData(row = 1, column = 2, isInline = false)),
                EndArgument,
                BeginArgument(metadata = MetaData(row = 1, column = 5, isInline = true)),
                Name(text = "d", metadata = MetaData(row = 1, column = 5, isInline = true)),
                EndArgument,
                BeginArgument(metadata = MetaData(row = 2, column = 2, isInline = false)),
                Name(text = "e", metadata = MetaData(row = 2, column = 2, isInline = false)),
                EndArgument,
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses a single section group with an operator arg`() =
        runTest(
            "someName: *+",
            listOf(
                BeginGroup(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 0, column = 10, isInline = true)),
                OperatorName(
                    text = "*+", metadata = MetaData(row = 0, column = 10, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses a single section group with an regular function arg`() =
        runTest(
            "someName: f(x, y)",
            listOf(
                BeginGroup(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 0, column = 10, isInline = true)),
                RegularFunction(
                    name =
                        Name(
                            text = "f", metadata = MetaData(row = 0, column = 10, isInline = true)),
                    params =
                        listOf(
                            NameParam(
                                Name(
                                    text = "x",
                                    metadata = MetaData(row = 0, column = 12, isInline = true)),
                                isVarArgs = false),
                            NameParam(
                                Name(
                                    text = "y",
                                    metadata = MetaData(row = 0, column = 15, isInline = true)),
                                isVarArgs = false)),
                    metadata = MetaData(row = 0, column = 10, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses a single section group with an sub params function arg`() =
        runTest(
            "someName: f_{x, y}",
            listOf(
                BeginGroup(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 0, column = 10, isInline = true)),
                SubParamFunction(
                    name =
                        Name(
                            text = "f", metadata = MetaData(row = 0, column = 10, isInline = true)),
                    subParams =
                        listOf(
                            NameParam(
                                Name(
                                    text = "x",
                                    metadata = MetaData(row = 0, column = 13, isInline = true)),
                                isVarArgs = false),
                            NameParam(
                                Name(
                                    text = "y",
                                    metadata = MetaData(row = 0, column = 16, isInline = true)),
                                isVarArgs = false)),
                    metadata = MetaData(row = 0, column = 10, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses a single section group with a sub and regular params function arg`() =
        runTest(
            "someName: f_{i, j}(x, y)",
            listOf(
                BeginGroup(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 0, column = 10, isInline = true)),
                SubAndRegularParamFunction(
                    name =
                        Name(
                            text = "f", metadata = MetaData(row = 0, column = 10, isInline = true)),
                    subParams =
                        listOf(
                            NameParam(
                                Name(
                                    text = "i",
                                    metadata = MetaData(row = 0, column = 13, isInline = true)),
                                isVarArgs = false),
                            NameParam(
                                Name(
                                    text = "j",
                                    metadata = MetaData(row = 0, column = 16, isInline = true)),
                                isVarArgs = false)),
                    params =
                        listOf(
                            NameParam(
                                Name(
                                    text = "x",
                                    metadata = MetaData(row = 0, column = 19, isInline = true)),
                                isVarArgs = false),
                            NameParam(
                                Name(
                                    text = "y",
                                    metadata = MetaData(row = 0, column = 22, isInline = true)),
                                isVarArgs = false)),
                    metadata = MetaData(row = 0, column = 10, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses a single section group with a sub params function sequence arg`() =
        runTest(
            "someName: {f_{x, y}}_{x, y}",
            listOf(
                BeginGroup(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 0, column = 10, isInline = true)),
                SubParamFunctionSequence(
                    func =
                        SubParamFunction(
                            name =
                                Name(
                                    text = "f",
                                    metadata = MetaData(row = 0, column = 11, isInline = true)),
                            subParams =
                                listOf(
                                    NameParam(
                                        Name(
                                            text = "x",
                                            metadata =
                                                MetaData(row = 0, column = 14, isInline = true)),
                                        isVarArgs = false),
                                    NameParam(
                                        Name(
                                            text = "y",
                                            metadata =
                                                MetaData(row = 0, column = 17, isInline = true)),
                                        isVarArgs = false)),
                            metadata = MetaData(row = 0, column = 11, isInline = true)),
                    metadata = MetaData(row = 0, column = 10, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses a single section group with a sub and regular params function sequence arg`() =
        runTest(
            "someName: {f_{i, j}(x, y)}_{i, j}",
            listOf(
                BeginGroup(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 0, column = 10, isInline = true)),
                SubAndRegularParamFunctionSequence(
                    func =
                        SubAndRegularParamFunction(
                            name =
                                Name(
                                    text = "f",
                                    metadata = MetaData(row = 0, column = 11, isInline = true)),
                            subParams =
                                listOf(
                                    NameParam(
                                        Name(
                                            text = "i",
                                            metadata =
                                                MetaData(row = 0, column = 14, isInline = true)),
                                        isVarArgs = false),
                                    NameParam(
                                        Name(
                                            text = "j",
                                            metadata =
                                                MetaData(row = 0, column = 17, isInline = true)),
                                        isVarArgs = false)),
                            params =
                                listOf(
                                    NameParam(
                                        Name(
                                            text = "x",
                                            metadata =
                                                MetaData(row = 0, column = 20, isInline = true)),
                                        isVarArgs = false),
                                    NameParam(
                                        Name(
                                            text = "y",
                                            metadata =
                                                MetaData(row = 0, column = 23, isInline = true)),
                                        isVarArgs = false)),
                            metadata = MetaData(row = 0, column = 11, isInline = true)),
                    metadata = MetaData(row = 0, column = 10, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses a single section group with a basic set arg`() =
        runTest(
            "someName: {x, y}",
            listOf(
                BeginGroup(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 0, column = 10, isInline = true)),
                Set(
                    items =
                        listOf(
                            Name(
                                text = "x",
                                metadata = MetaData(row = 0, column = 11, isInline = true)),
                            Name(
                                text = "y",
                                metadata = MetaData(row = 0, column = 14, isInline = true))),
                    metadata = MetaData(row = 0, column = 10, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses a single section group with a colon equals set arg`() =
        runTest(
            "someName: {x := a, y}",
            listOf(
                BeginGroup(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 0, column = 10, isInline = true)),
                Set(
                    items =
                        listOf(
                            NameAssignment(
                                lhs =
                                    Name(
                                        text = "x",
                                        metadata = MetaData(row = 0, column = 11, isInline = true)),
                                rhs =
                                    Name(
                                        text = "a",
                                        metadata = MetaData(row = 0, column = 16, isInline = true)),
                                metadata = MetaData(row = 0, column = 11, isInline = true)),
                            Name(
                                text = "y",
                                metadata = MetaData(row = 0, column = 19, isInline = true))),
                    metadata = MetaData(row = 0, column = 10, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses a single section group with a basic tuple arg`() =
        runTest(
            "someName: (x, y)",
            listOf(
                BeginGroup(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 0, column = 10, isInline = true)),
                Tuple(
                    targets =
                        listOf(
                            Name(
                                text = "x",
                                metadata = MetaData(row = 0, column = 11, isInline = true)),
                            Name(
                                text = "y",
                                metadata = MetaData(row = 0, column = 14, isInline = true))),
                    metadata = MetaData(row = 0, column = 10, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses a single section group with a colon equals tuple arg`() =
        runTest(
            "someName: (x := a, y)",
            listOf(
                BeginGroup(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 0, column = 10, isInline = true)),
                Tuple(
                    targets =
                        listOf(
                            NameAssignment(
                                lhs =
                                    Name(
                                        text = "x",
                                        metadata = MetaData(row = 0, column = 11, isInline = true)),
                                rhs =
                                    Name(
                                        text = "a",
                                        metadata = MetaData(row = 0, column = 16, isInline = true)),
                                metadata = MetaData(row = 0, column = 11, isInline = true)),
                            Name(
                                text = "y",
                                metadata = MetaData(row = 0, column = 19, isInline = true))),
                    metadata = MetaData(row = 0, column = 10, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses a single section group with a simple name assignment arg`() =
        runTest(
            "someName: x := y",
            listOf(
                BeginGroup(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "someName", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 0, column = 10, isInline = true)),
                NameAssignment(
                    lhs =
                        Name(
                            text = "x", metadata = MetaData(row = 0, column = 10, isInline = true)),
                    rhs =
                        Name(
                            text = "y", metadata = MetaData(row = 0, column = 15, isInline = true)),
                    metadata = MetaData(row = 0, column = 10, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses multi section groups`() =
        runTest(
            """
            sectionA:
            sectionB:
            sectionC:
        """.trimIndent(),
            listOf(
                BeginGroup(
                    name = "sectionA", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "sectionA", metadata = MetaData(row = 0, column = 0, isInline = false)),
                EndSection,
                BeginSection(
                    name = "sectionB", metadata = MetaData(row = 1, column = 0, isInline = false)),
                EndSection,
                BeginSection(
                    name = "sectionC", metadata = MetaData(row = 2, column = 0, isInline = false)),
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses groups with groups as args`() =
        runTest(
            """
            A:
            . X:
            B:
        """.trimIndent(),
            listOf(
                BeginGroup(name = "A", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "A", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 1, column = 2, isInline = false)),
                BeginGroup(name = "X", metadata = MetaData(row = 1, column = 2, isInline = false)),
                BeginSection(
                    name = "X", metadata = MetaData(row = 1, column = 2, isInline = false)),
                EndSection,
                EndGroup,
                EndArgument,
                EndSection,
                BeginSection(
                    name = "B", metadata = MetaData(row = 2, column = 0, isInline = false)),
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses groups with nested groups as args`() =
        runTest(
            """
            A:
            . X:
              . Y:
            B:
        """.trimIndent(),
            listOf(
                BeginGroup(name = "A", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "A", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 1, column = 2, isInline = false)),
                BeginGroup(name = "X", metadata = MetaData(row = 1, column = 2, isInline = false)),
                BeginSection(
                    name = "X", metadata = MetaData(row = 1, column = 2, isInline = false)),
                BeginArgument(metadata = MetaData(row = 2, column = 4, isInline = false)),
                BeginGroup(name = "Y", metadata = MetaData(row = 2, column = 4, isInline = false)),
                BeginSection(
                    name = "Y", metadata = MetaData(row = 2, column = 4, isInline = false)),
                EndSection,
                EndGroup,
                EndArgument,
                EndSection,
                EndGroup,
                EndArgument,
                EndSection,
                BeginSection(
                    name = "B", metadata = MetaData(row = 3, column = 0, isInline = false)),
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses groups with deeply nested groups as args`() =
        runTest(
            """
            A:
            . X:
              . Y:
                . Z:
            B:
        """.trimIndent(),
            listOf(
                BeginGroup(name = "A", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "A", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 1, column = 2, isInline = false)),
                BeginGroup(name = "X", metadata = MetaData(row = 1, column = 2, isInline = false)),
                BeginSection(
                    name = "X", metadata = MetaData(row = 1, column = 2, isInline = false)),
                BeginArgument(metadata = MetaData(row = 2, column = 4, isInline = false)),
                BeginGroup(name = "Y", metadata = MetaData(row = 2, column = 4, isInline = false)),
                BeginSection(
                    name = "Y", metadata = MetaData(row = 2, column = 4, isInline = false)),
                BeginArgument(metadata = MetaData(row = 3, column = 6, isInline = false)),
                BeginGroup(name = "Z", metadata = MetaData(row = 3, column = 6, isInline = false)),
                BeginSection(
                    name = "Z", metadata = MetaData(row = 3, column = 6, isInline = false)),
                EndSection,
                EndGroup,
                EndArgument,
                EndSection,
                EndGroup,
                EndArgument,
                EndSection,
                EndGroup,
                EndArgument,
                EndSection,
                BeginSection(
                    name = "B", metadata = MetaData(row = 4, column = 0, isInline = false)),
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses groups with deeply nested groups as args and offset trailing section`() =
        runTest(
            """
            A:
            . X:
              . Y:
                . Z:
              B:
        """.trimIndent(),
            listOf(
                BeginGroup(name = "A", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "A", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 1, column = 2, isInline = false)),
                BeginGroup(name = "X", metadata = MetaData(row = 1, column = 2, isInline = false)),
                BeginSection(
                    name = "X", metadata = MetaData(row = 1, column = 2, isInline = false)),
                BeginArgument(metadata = MetaData(row = 2, column = 4, isInline = false)),
                BeginGroup(name = "Y", metadata = MetaData(row = 2, column = 4, isInline = false)),
                BeginSection(
                    name = "Y", metadata = MetaData(row = 2, column = 4, isInline = false)),
                BeginArgument(metadata = MetaData(row = 3, column = 6, isInline = false)),
                BeginGroup(name = "Z", metadata = MetaData(row = 3, column = 6, isInline = false)),
                BeginSection(
                    name = "Z", metadata = MetaData(row = 3, column = 6, isInline = false)),
                EndSection,
                EndGroup,
                EndArgument,
                EndSection,
                EndGroup,
                EndArgument,
                EndSection,
                BeginSection(
                    name = "B", metadata = MetaData(row = 4, column = 2, isInline = false)),
                EndSection,
                EndGroup,
                EndArgument,
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses groups with deeply nested groups as args and offset trailing section and aligned section`() =
        runTest(
            """
            A:
            . X:
              . Y:
                . Z:
              B:
            C:
        """.trimIndent(),
            listOf(
                BeginGroup(name = "A", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "A", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 1, column = 2, isInline = false)),
                BeginGroup(name = "X", metadata = MetaData(row = 1, column = 2, isInline = false)),
                BeginSection(
                    name = "X", metadata = MetaData(row = 1, column = 2, isInline = false)),
                BeginArgument(metadata = MetaData(row = 2, column = 4, isInline = false)),
                BeginGroup(name = "Y", metadata = MetaData(row = 2, column = 4, isInline = false)),
                BeginSection(
                    name = "Y", metadata = MetaData(row = 2, column = 4, isInline = false)),
                BeginArgument(metadata = MetaData(row = 3, column = 6, isInline = false)),
                BeginGroup(name = "Z", metadata = MetaData(row = 3, column = 6, isInline = false)),
                BeginSection(
                    name = "Z", metadata = MetaData(row = 3, column = 6, isInline = false)),
                EndSection,
                EndGroup,
                EndArgument,
                EndSection,
                EndGroup,
                EndArgument,
                EndSection,
                BeginSection(
                    name = "B", metadata = MetaData(row = 4, column = 2, isInline = false)),
                EndSection,
                EndGroup,
                EndArgument,
                EndSection,
                BeginSection(
                    name = "C", metadata = MetaData(row = 5, column = 0, isInline = false)),
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses multi sections where first section has dot space argument`() =
        runTest(
            """
            Theorem:
            . a: x
            then: 'y'
    """.trimIndent(),
            listOf(
                BeginGroup(
                    name = "Theorem", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "Theorem", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 1, column = 2, isInline = false)),
                BeginGroup(name = "a", metadata = MetaData(row = 1, column = 2, isInline = false)),
                BeginSection(
                    name = "a", metadata = MetaData(row = 1, column = 2, isInline = false)),
                BeginArgument(metadata = MetaData(row = 1, column = 5, isInline = true)),
                Name(text = "x", metadata = MetaData(row = 1, column = 5, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup,
                EndArgument,
                EndSection,
                BeginSection(
                    name = "then", metadata = MetaData(row = 2, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 2, column = 6, isInline = true)),
                Statement(text = "y", metadata = MetaData(row = 2, column = 6, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses multiple top level groups`() =
        runTest(
            """
            Theorem: x

            Theorem: y

            Theorem: z
        """.trimIndent(),
            listOf(
                BeginGroup(
                    name = "Theorem", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "Theorem", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 0, column = 9, isInline = true)),
                Name(text = "x", metadata = MetaData(row = 0, column = 9, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup,
                BeginGroup(
                    name = "Theorem", metadata = MetaData(row = 2, column = 0, isInline = false)),
                BeginSection(
                    name = "Theorem", metadata = MetaData(row = 2, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 2, column = 9, isInline = true)),
                Name(text = "y", metadata = MetaData(row = 2, column = 9, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup,
                BeginGroup(
                    name = "Theorem", metadata = MetaData(row = 4, column = 0, isInline = false)),
                BeginSection(
                    name = "Theorem", metadata = MetaData(row = 4, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 4, column = 9, isInline = true)),
                Name(text = "z", metadata = MetaData(row = 4, column = 9, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses multiple top level groups and text blocks`() =
        runTest(
            """
            :: some text ::

            Theorem: x

            Theorem: y

            Theorem: z
        """.trimIndent(),
            listOf(
                TextBlock(
                    text = " some text ",
                    metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginGroup(
                    name = "Theorem", metadata = MetaData(row = 2, column = 0, isInline = false)),
                BeginSection(
                    name = "Theorem", metadata = MetaData(row = 2, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 2, column = 9, isInline = true)),
                Name(text = "x", metadata = MetaData(row = 2, column = 9, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup,
                BeginGroup(
                    name = "Theorem", metadata = MetaData(row = 4, column = 0, isInline = false)),
                BeginSection(
                    name = "Theorem", metadata = MetaData(row = 4, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 4, column = 9, isInline = true)),
                Name(text = "y", metadata = MetaData(row = 4, column = 9, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup,
                BeginGroup(
                    name = "Theorem", metadata = MetaData(row = 6, column = 0, isInline = false)),
                BeginSection(
                    name = "Theorem", metadata = MetaData(row = 6, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 6, column = 9, isInline = true)),
                Name(text = "z", metadata = MetaData(row = 6, column = 9, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses multiple top level groups with ids`() =
        runTest(
            """
            [id 1]
            Theorem: x

            [id 2]
            Theorem: y

            [id 3]
            Theorem: z
        """.trimIndent(),
            listOf(
                BeginGroup(
                    name = "Theorem", metadata = MetaData(row = 0, column = 0, isInline = false)),
                Id(text = "id 1", metadata = MetaData(row = 0, column = 0, isInline = false)),
                BeginSection(
                    name = "Theorem", metadata = MetaData(row = 1, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 1, column = 9, isInline = true)),
                Name(text = "x", metadata = MetaData(row = 1, column = 9, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup,
                BeginGroup(
                    name = "Theorem", metadata = MetaData(row = 3, column = 0, isInline = false)),
                Id(text = "id 2", metadata = MetaData(row = 3, column = 0, isInline = false)),
                BeginSection(
                    name = "Theorem", metadata = MetaData(row = 4, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 4, column = 9, isInline = true)),
                Name(text = "y", metadata = MetaData(row = 4, column = 9, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup,
                BeginGroup(
                    name = "Theorem", metadata = MetaData(row = 6, column = 0, isInline = false)),
                Id(text = "id 3", metadata = MetaData(row = 6, column = 0, isInline = false)),
                BeginSection(
                    name = "Theorem", metadata = MetaData(row = 7, column = 0, isInline = false)),
                BeginArgument(metadata = MetaData(row = 7, column = 9, isInline = true)),
                Name(text = "z", metadata = MetaData(row = 7, column = 9, isInline = true)),
                EndArgument,
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses multiple dot space group arguments`() = runTest("""
        [a]
        Defines: b
        satisfying:
        . forAll: c
          then: 'd'
        . forAll: e
          then: 'f'
        written: "g"
    """.trimIndent(), listOf(
        BeginGroup(name="Defines", metadata=MetaData(row=0, column=0, isInline=false)),
            Id(text="a", metadata=MetaData(row=0, column=0, isInline=false)),
            BeginSection(name="Defines", metadata=MetaData(row=1, column=0, isInline=false)),
                BeginArgument(metadata=MetaData(row=1, column=9, isInline=true)),
                    Name(text="b", metadata=MetaData(row=1, column=9, isInline=true)),
                EndArgument,
            EndSection,
            BeginSection(name="satisfying", metadata=MetaData(row=2, column=0, isInline=false)),
                BeginArgument(metadata=MetaData(row=3, column=2, isInline=false)),
                    BeginGroup(name="forAll", metadata=MetaData(row=3, column=2, isInline=false)),
                        BeginSection(name="forAll", metadata=MetaData(row=3, column=2, isInline=false)),
                            BeginArgument(metadata=MetaData(row=3, column=10, isInline=true)),
                                Name(text="c", metadata=MetaData(row=3, column=10, isInline=true)),
                            EndArgument,
                        EndSection,
                        BeginSection(name="then", metadata=MetaData(row=4, column=2, isInline=false)),
                            BeginArgument(metadata=MetaData(row=4, column=8, isInline=true)),
                                Statement(text="d", metadata=MetaData(row=4, column=8, isInline=true)),
                            EndArgument,
                        EndSection,
                    EndGroup,
                EndArgument,
                BeginArgument(metadata=MetaData(row=5, column=2, isInline=false)),
                    BeginGroup(name="forAll", metadata=MetaData(row=5, column=2, isInline=false)),
                        BeginSection(name="forAll", metadata=MetaData(row=5, column=2, isInline=false)),
                            BeginArgument(metadata=MetaData(row=5, column=10, isInline=true)),
                                Name(text="e", metadata=MetaData(row=5, column=10, isInline=true)),
                            EndArgument,
                        EndSection,
                        BeginSection(name="then", metadata=MetaData(row=6, column=2, isInline=false)),
                            BeginArgument(metadata=MetaData(row=6, column=8, isInline=true)),
                                Statement(text="f", metadata=MetaData(row=6, column=8, isInline=true)),
                            EndArgument,
                        EndSection,
                    EndGroup,
                EndArgument,
            EndSection,
            BeginSection(name="written", metadata=MetaData(row=7, column=0, isInline=false)),
                BeginArgument(metadata=MetaData(row=7, column=9, isInline=true)),
                    Text(text="g", metadata=MetaData(row=7, column=9, isInline=true)),
                EndArgument,
            EndSection,
        EndGroup
    ))

    @Test
    fun `correctly parses multiple simple dot space group arguments`() = runTest("""
        a:
        . b: x
        . c: y
        d: z
    """.trimIndent(), listOf(
        BeginGroup(name="a", metadata=MetaData(row=0, column=0, isInline=false)),
            BeginSection(name="a", metadata=MetaData(row=0, column=0, isInline=false)),
                BeginArgument(metadata=MetaData(row=1, column=2, isInline=false)),
                    BeginGroup(name="b", metadata=MetaData(row=1, column=2, isInline=false)),
                        BeginSection(name="b", metadata=MetaData(row=1, column=2, isInline=false)),
                            BeginArgument(metadata=MetaData(row=1, column=5, isInline=true)),
                                Name(text="x", metadata=MetaData(row=1, column=5, isInline=true)),
                            EndArgument,
                        EndSection,
                    EndGroup,
                EndArgument,
                BeginArgument(metadata=MetaData(row=2, column=2, isInline=false)),
                    BeginGroup(name="c", metadata=MetaData(row=2, column=2, isInline=false)),
                        BeginSection(name="c", metadata=MetaData(row=2, column=2, isInline=false)),
                            BeginArgument(metadata=MetaData(row=2, column=5, isInline=true)),
                                Name(text="y", metadata=MetaData(row=2, column=5, isInline=true)),
                            EndArgument,
                        EndSection,
                    EndGroup,
                EndArgument,
            EndSection,
            BeginSection(name="d", metadata=MetaData(row=3, column=0, isInline=false)),
                BeginArgument(metadata=MetaData(row=3, column=3, isInline=true)),
                    Name(text="z", metadata=MetaData(row=3, column=3, isInline=true)),
                EndArgument,
            EndSection,
        EndGroup
    ))
}

private fun runTest(text: String, expected: List<NodeLexerToken>) {
    val tokenLexer = newChalkTalkTokenLexer(text)
    val nodeLexer = newChalkTalkNodeLexer(tokenLexer)
    val nodes = mutableListOf<NodeLexerToken>()
    while (nodeLexer.hasNext()) {
        nodes.add(nodeLexer.next())
    }
    expect {
        that(nodeLexer.diagnostics()).isEmpty()
        that(nodes.toList()).isEqualTo(expected)
    }
}

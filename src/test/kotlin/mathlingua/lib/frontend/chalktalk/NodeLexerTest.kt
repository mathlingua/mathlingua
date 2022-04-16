package mathlingua.lib.frontend.chalktalk

import kotlin.test.Test
import mathlingua.lib.frontend.MetaData
import mathlingua.lib.frontend.ast.BeginArgument
import mathlingua.lib.frontend.ast.BeginGroup
import mathlingua.lib.frontend.ast.BeginSection
import mathlingua.lib.frontend.ast.ChalkTalkNode
import mathlingua.lib.frontend.ast.EndArgument
import mathlingua.lib.frontend.ast.EndGroup
import mathlingua.lib.frontend.ast.EndSection
import mathlingua.lib.frontend.ast.Name
import mathlingua.lib.frontend.ast.NameAssignment
import mathlingua.lib.frontend.ast.NameParam
import mathlingua.lib.frontend.ast.OperatorName
import mathlingua.lib.frontend.ast.RegularFunction
import mathlingua.lib.frontend.ast.Set
import mathlingua.lib.frontend.ast.SubAndRegularParamFunction
import mathlingua.lib.frontend.ast.SubAndRegularParamFunctionSequence
import mathlingua.lib.frontend.ast.SubParamFunction
import mathlingua.lib.frontend.ast.SubParamFunctionSequence
import mathlingua.lib.frontend.ast.TextBlock
import mathlingua.lib.frontend.ast.Tuple
import strikt.api.expect
import strikt.assertions.isEmpty
import strikt.assertions.isEqualTo

internal class NodeLexerTest {
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
            "someName:", listOf(BeginGroup, BeginSection(name = "someName"), EndSection, EndGroup))

    @Test
    fun `correctly parses a single section group with a name arg`() =
        runTest(
            "someName: xyz",
            listOf(
                BeginGroup,
                BeginSection(name = "someName"),
                BeginArgument,
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
                BeginGroup,
                BeginSection(name = "someName"),
                BeginArgument,
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
                BeginGroup,
                BeginSection(name = "someName"),
                BeginArgument,
                Name(text = "a", metadata = MetaData(row = 0, column = 10, isInline = true)),
                EndArgument,
                BeginArgument,
                Name(text = "b", metadata = MetaData(row = 0, column = 13, isInline = true)),
                EndArgument,
                BeginArgument,
                Name(text = "c", metadata = MetaData(row = 1, column = 2, isInline = false)),
                EndArgument,
                BeginArgument,
                Name(text = "d", metadata = MetaData(row = 1, column = 5, isInline = true)),
                EndArgument,
                BeginArgument,
                Name(text = "e", metadata = MetaData(row = 2, column = 2, isInline = false)),
                EndArgument,
                EndSection,
                EndGroup))

    @Test
    fun `correctly parses a single section group with an operator arg`() =
        runTest(
            "someName: *+",
            listOf(
                BeginGroup,
                BeginSection(name = "someName"),
                BeginArgument,
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
                BeginGroup,
                BeginSection(name = "someName"),
                BeginArgument,
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
                BeginGroup,
                BeginSection(name = "someName"),
                BeginArgument,
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
                BeginGroup,
                BeginSection(name = "someName"),
                BeginArgument,
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
                BeginGroup,
                BeginSection(name = "someName"),
                BeginArgument,
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
                BeginGroup,
                BeginSection(name = "someName"),
                BeginArgument,
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
                BeginGroup,
                BeginSection(name = "someName"),
                BeginArgument,
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
                BeginGroup,
                BeginSection(name = "someName"),
                BeginArgument,
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
                BeginGroup,
                BeginSection(name = "someName"),
                BeginArgument,
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
                BeginGroup,
                BeginSection(name = "someName"),
                BeginArgument,
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
                BeginGroup,
                BeginSection(name = "someName"),
                BeginArgument,
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
                BeginGroup,
                BeginSection(name = "sectionA"),
                EndSection,
                BeginSection(name = "sectionB"),
                EndSection,
                BeginSection(name = "sectionC"),
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
                BeginGroup,
                BeginSection(name = "A"),
                BeginArgument,
                BeginGroup,
                BeginSection(name = "X"),
                EndSection,
                EndGroup,
                EndArgument,
                EndSection,
                BeginSection(name = "B"),
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
                BeginGroup,
                BeginSection(name = "A"),
                BeginArgument,
                BeginGroup,
                BeginSection(name = "X"),
                BeginArgument,
                BeginGroup,
                BeginSection(name = "Y"),
                EndSection,
                EndGroup,
                EndArgument,
                EndSection,
                EndGroup,
                EndArgument,
                EndSection,
                BeginSection(name = "B"),
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
                BeginGroup,
                BeginSection(name = "A"),
                BeginArgument,
                BeginGroup,
                BeginSection(name = "X"),
                BeginArgument,
                BeginGroup,
                BeginSection(name = "Y"),
                BeginArgument,
                BeginGroup,
                BeginSection(name = "Z"),
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
                BeginSection(name = "B"),
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
                BeginGroup,
                BeginSection(name = "A"),
                BeginArgument,
                BeginGroup,
                BeginSection(name = "X"),
                BeginArgument,
                BeginGroup,
                BeginSection(name = "Y"),
                BeginArgument,
                BeginGroup,
                BeginSection(name = "Z"),
                EndSection,
                EndGroup,
                EndArgument,
                EndSection,
                EndGroup,
                EndArgument,
                EndSection,
                BeginSection(name = "B"),
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
                BeginGroup,
                BeginSection(name = "A"),
                BeginArgument,
                BeginGroup,
                BeginSection(name = "X"),
                BeginArgument,
                BeginGroup,
                BeginSection(name = "Y"),
                BeginArgument,
                BeginGroup,
                BeginSection(name = "Z"),
                EndSection,
                EndGroup,
                EndArgument,
                EndSection,
                EndGroup,
                EndArgument,
                EndSection,
                BeginSection(name = "B"),
                EndSection,
                EndGroup,
                EndArgument,
                EndSection,
                BeginSection(name = "C"),
                EndSection,
                EndGroup))
}

private fun runTest(text: String, expected: List<ChalkTalkNode>) {
    val tokenLexer = newTokenLexer(text)
    val nodeLexer = newNodeLexer(tokenLexer)
    val nodes = mutableListOf<ChalkTalkNode>()
    while (nodeLexer.hasNext()) {
        nodes.add(nodeLexer.next())
    }
    expect {
        that(nodeLexer.errors()).isEmpty()
        that(nodes.toList()).isEqualTo(expected)
    }
}

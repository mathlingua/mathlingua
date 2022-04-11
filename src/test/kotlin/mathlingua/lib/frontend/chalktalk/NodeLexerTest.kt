package mathlingua.lib.frontend.chalktalk

import mathlingua.lib.frontend.MetaData
import strikt.api.expect
import strikt.assertions.isEmpty
import strikt.assertions.isEqualTo
import kotlin.test.Test

internal class NodeLexerTest {
    @Test
    fun `correctly parses empty input`() = runTest("", emptyList())

    @Test
    fun `correctly parses single text block`() = runTest("::some text::", listOf(
        TextBlock(
            text = "some text",
            metadata = MetaData(
                row = 0,
                column = 0,
                isInline = true
            )
        )
    ))

    @Test
    fun `correctly parses single name`() = runTest("someName", listOf(
        BeginArgument,
        Name(
            text = "someName",
            metadata = MetaData(
                row = 0,
                column = 0,
                isInline = true
            )
        ),
        EndArgument
    ))

    @Test
    fun `correctly parses single operator`() = runTest("*+-", listOf(
        BeginArgument,
        Name(
            text = "*+-",
            metadata = MetaData(
                row = 0,
                column = 0,
                isInline = true
            )
        ),
        EndArgument
    ))

    @Test
    fun `correctly parses single section`() = runTest("""
        iff:
    """.trimIndent(), listOf(
        BeginGroup,
        BeginSection(name = "iff"),
        EndSection,
        EndGroup
    ))

    @Test
    fun `correctly parses single section with single name argument`() = runTest("""
        iff: someName
    """.trimIndent(), listOf(
        BeginGroup,
        BeginSection(name = "iff"),
        BeginArgument,
        Name(
            text = "someName",
            metadata = MetaData(
                row = 0,
                column = 5,
                isInline = true
            )
        ),
        EndArgument,
        EndSection,
        EndGroup
    ))

    /*
    @Test
    fun `correctly parses single section with single non-inline name argument`() = runTest("""
        iff:
        . someName
    """.trimIndent(), listOf(
        BeginGroup,
        BeginSection(name = "iff"),
        BeginArgument,
        Name(
            text = "someName",
            metadata = MetaData(
                row = 1,
                column = 2,
                isInline = false
            )
        ),
        EndArgument,
        EndSection,
        EndGroup
    ))
     */

    @Test
    fun `correctly parses multiple sections with single name argument`() = runTest("""
        iff: someName
        then: anotherName
    """.trimIndent(), listOf(
        BeginGroup,
        BeginSection(name = "iff"),
        BeginArgument,
        Name(
            text = "someName",
            metadata = MetaData(
                row = 0,
                column = 5,
                isInline = true
            )
        ),
        EndArgument,
        EndSection,
        BeginSection(name = "then"),
        BeginArgument,
        Name(
            text = "anotherName",
            metadata = MetaData(
                row = 1,
                column = 6,
                isInline = true
            )
        ),
        EndArgument,
        EndSection,
        EndGroup
    ))
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

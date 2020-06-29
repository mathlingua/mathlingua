package mathlingua.common

import assertk.assertThat
import assertk.assertions.isEqualTo
import mathlingua.common.textalk.Command
import mathlingua.common.textalk.CommandPart
import mathlingua.common.textalk.ExpressionTexTalkNode
import mathlingua.common.textalk.GroupTexTalkNode
import mathlingua.common.textalk.OperatorTexTalkNode
import mathlingua.common.textalk.ParametersTexTalkNode
import mathlingua.common.textalk.TexTalkNodeType
import mathlingua.common.textalk.TexTalkTokenType
import mathlingua.common.textalk.TextTexTalkNode
import org.junit.jupiter.api.Test

internal class MathLinguaTest {

    @Test
    fun expandWrittenAs() {
        val validation = MathLingua.parse("""
            [\or{a}{b}]
            Represents:
            that: "something"
            Metadata:
            . written: "a? \text{ or } b?"
        """.trimIndent())
        assertThat(validation is ValidationSuccess)
        val doc = (validation as ValidationSuccess).value
        val map = MathLingua.getPatternsToWrittenAs(emptyList(), doc.represents)
        val expectedCommand = Command(
            parts = listOf(
                CommandPart(
                    name = TextTexTalkNode(
                        type = TexTalkNodeType.Identifier,
                        tokenType = TexTalkTokenType.Identifier,
                        text = "or",
                        isVarArg = false
                    ),
                    square = null,
                    subSup = null,
                    groups = listOf(
                        GroupTexTalkNode(
                            type = TexTalkNodeType.CurlyGroup,
                            parameters = ParametersTexTalkNode(
                                items = listOf(
                                    ExpressionTexTalkNode(
                                        children = listOf(
                                            TextTexTalkNode(
                                                type = TexTalkNodeType.Identifier,
                                                tokenType = TexTalkTokenType.Identifier,
                                                text = "a",
                                                isVarArg = false
                                            )
                                        )
                                    )
                                )
                            ),
                            isVarArg = false
                        ),
                        GroupTexTalkNode(
                            type = TexTalkNodeType.CurlyGroup,
                            parameters = ParametersTexTalkNode(
                                items = listOf(
                                    ExpressionTexTalkNode(
                                        children = listOf(
                                            TextTexTalkNode(
                                                type = TexTalkNodeType.Identifier,
                                                tokenType = TexTalkTokenType.Identifier,
                                                text = "b",
                                                isVarArg = false
                                            )
                                        )
                                    )
                                )
                            ),
                            isVarArg = false
                        )
                    ),
                    namedGroups = emptyList()
                )
            )
        )
        val expected = mapOf(
            OperatorTexTalkNode(
                lhs = null,
                command = expectedCommand,
                rhs = null
            ) to "a? \\text{ or } b?"
        )
        assertThat(map).isEqualTo(expected)
    }
}

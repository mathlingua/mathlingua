package mathlingua.mathlingua

import assertk.assertThat
import assertk.assertions.isEmpty
import assertk.assertions.isEqualTo
import mathlingua.MathLingua
import mathlingua.Signature
import mathlingua.support.Location
import mathlingua.support.ValidationSuccess
import mathlingua.textalk.Command
import mathlingua.textalk.CommandPart
import mathlingua.textalk.ExpressionTexTalkNode
import mathlingua.textalk.GroupTexTalkNode
import mathlingua.textalk.OperatorTexTalkNode
import mathlingua.textalk.ParametersTexTalkNode
import mathlingua.textalk.TexTalkNodeType
import mathlingua.textalk.TexTalkTokenType
import mathlingua.textalk.TextTexTalkNode
import org.junit.jupiter.api.Test

internal class MathLinguaTest {

    @Test
    fun findDuplicateContentNoDuplicates() {
        val input =
            """
            [\finite.set]
            Defines: X
            means: '\something'

            [\infinite.set]
            Defines: X
            means: '\something.else'
        """.trimIndent()

        val supplemental =
            listOf(
                """
            [\set]
            Defines: X
            means:
            . if: X
              then: '\something.else'
        """.trimIndent(),
                """
            Theorem:
            then:
            . '\finite.set'
        """.trimIndent())

        val dups = MathLingua.findDuplicateContent(input, supplemental)
        assertThat(dups).isEmpty()
    }

    @Test
    fun findDuplicateContentDuplicatesInInput() {
        val input =
            """
            [\finite.set]
            Defines: X
            means: '\something'

            [\another.name]
            Defines: X
            means: '\something'

            [\infinite.set]
            Defines: X
            means: '\something.else'
        """.trimIndent()

        val supplemental =
            listOf(
                """
            [\set]
            Defines: X
            means:
            . if: X
              then: '\something.else'
        """.trimIndent(),
                """
            Theorem:
            then:
            . '\finite.set'
        """.trimIndent())

        val dups = MathLingua.findDuplicateContent(input, supplemental)
        assertThat(dups).isEqualTo(listOf(Location(row = 4, column = 1)))
    }

    @Test
    fun findDuplicateSignaturesDuplicatesInInput() {
        val input =
            """
            [\finite.set]
            Defines: X
            means: '\something'

            [\infinite.set]
            Defines: X
            means: '\something.else'

            [\finite.set]
            Defines: Y
            means: '\yet.something.else'
        """.trimIndent()

        val supplemental =
            listOf(
                """
            [\set]
            Defines: X
            means:
            . if: X
              then: '\something.else'
        """.trimIndent(),
                """
            Theorem:
            then:
            . '\finite.set'
        """.trimIndent())

        val dups = MathLingua.findDuplicateSignatures(input, supplemental)
        assertThat(dups)
            .isEqualTo(
                listOf(Signature(form = "\\finite.set", location = Location(row = 8, column = 1))))
    }

    @Test
    fun findDuplicateContentDuplicatesWithSupplemental() {
        val input =
            """
            [\finite.set]
            Defines: X
            means: '\something'

            [\infinite.set]
            Defines: X
            means: '\something.else'
        """.trimIndent()

        val supplemental =
            listOf(
                """
            [\set]
            Defines: X
            means:
            . if: X
              then: '\something.else'

            [\another.name]
            Defines: X
            means: '\something'
        """.trimIndent(),
                """
            Theorem:
            then:
            . '\finite.set'
        """.trimIndent())

        val dups = MathLingua.findDuplicateContent(input, supplemental)
        assertThat(dups).isEqualTo(listOf(Location(row = 0, column = 0)))
    }

    @Test
    fun findDuplicateSignaturesDuplicatesWithSupplemental() {
        val input =
            """
            [\finite.set]
            Defines: X
            means: '\something'

            [\infinite.set]
            Defines: X
            means: '\something.else'
        """.trimIndent()

        val supplemental =
            listOf(
                """
            [\set]
            Defines: X
            means:
            . if: X
              then: '\something.else'
        """.trimIndent(),
                """
            Theorem:
            then:
            . '\finite.set'

            [\finite.set]
            Defines: Y
            means: '\yet.something.else'
        """.trimIndent())

        val dups = MathLingua.findDuplicateSignatures(input, supplemental)
        assertThat(dups)
            .isEqualTo(
                listOf(Signature(form = "\\finite.set", location = Location(row = 0, column = 0))))
    }

    @Test
    fun findDuplicateContentDuplicatesAllInSupplemental() {
        val input =
            """
            [\finite.set]
            Defines: X
            means: '\something'

            [\infinite.set]
            Defines: X
            means: '\something.else'
        """.trimIndent()

        val supplemental =
            listOf(
                """
            [\set]
            Defines: X
            means:
            . if: X
              then: '\something.else'

            Theorem:
            then:
            . '\finite.set'
        """.trimIndent(),
                """
            Theorem:
            then:
            . '\finite.set'
        """.trimIndent())

        val dups = MathLingua.findDuplicateContent(input, supplemental)
        assertThat(dups).isEmpty()
    }

    @Test
    fun findDuplicateSignaturesDuplicatesAllInSupplemental() {
        val input =
            """
            [\finite.set]
            Defines: X
            means: '\something'

            [\infinite.set]
            Defines: X
            means: '\something.else'
        """.trimIndent()

        val supplemental =
            listOf(
                """
            [\set]
            Defines: X
            means:
            . if: X
              then: '\something.else'
        """.trimIndent(),
                """
            Theorem:
            then:
            . '\finite.set'

            [\set]
            Defines: Y
            means: '\yet.something.else'
        """.trimIndent())

        val dups = MathLingua.findDuplicateSignatures(input, supplemental)
        assertThat(dups).isEmpty()
    }

    @Test
    fun expandWrittenAs() {
        val validation =
            MathLingua.parse(
                """
            [\or{a}{b}]
            States:
            that: "something"
            written: "a? \text{ or } b?"
        """.trimIndent())
        assertThat(validation is ValidationSuccess)
        val doc = (validation as ValidationSuccess).value
        val map =
            MathLingua.getPatternsToWrittenAs(emptyList(), doc.states(), emptyList(), emptyList())
        val expectedCommand =
            Command(
                parts =
                    listOf(
                        CommandPart(
                            name =
                                TextTexTalkNode(
                                    type = TexTalkNodeType.Identifier,
                                    tokenType = TexTalkTokenType.Identifier,
                                    text = "or",
                                    isVarArg = false),
                            square = null,
                            subSup = null,
                            groups =
                                listOf(
                                    GroupTexTalkNode(
                                        type = TexTalkNodeType.CurlyGroup,
                                        parameters =
                                            ParametersTexTalkNode(
                                                items =
                                                    listOf(
                                                        ExpressionTexTalkNode(
                                                            children =
                                                                listOf(
                                                                    TextTexTalkNode(
                                                                        type =
                                                                            TexTalkNodeType
                                                                                .Identifier,
                                                                        tokenType =
                                                                            TexTalkTokenType
                                                                                .Identifier,
                                                                        text = "a",
                                                                        isVarArg = false))))),
                                        isVarArg = false),
                                    GroupTexTalkNode(
                                        type = TexTalkNodeType.CurlyGroup,
                                        parameters =
                                            ParametersTexTalkNode(
                                                items =
                                                    listOf(
                                                        ExpressionTexTalkNode(
                                                            children =
                                                                listOf(
                                                                    TextTexTalkNode(
                                                                        type =
                                                                            TexTalkNodeType
                                                                                .Identifier,
                                                                        tokenType =
                                                                            TexTalkTokenType
                                                                                .Identifier,
                                                                        text = "b",
                                                                        isVarArg = false))))),
                                        isVarArg = false)),
                            paren = null,
                            namedGroups = emptyList())))
        val expected =
            mapOf(
                OperatorTexTalkNode(lhs = null, command = expectedCommand, rhs = null) to
                    "a? \\text{ or } b?")
        assertThat(map).isEqualTo(expected)
    }
}

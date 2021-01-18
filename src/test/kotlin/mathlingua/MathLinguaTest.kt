package mathlingua.mathlingua

import assertk.assertThat
import assertk.assertions.isEmpty
import assertk.assertions.isEqualTo
import mathlingua.MathLingua
import mathlingua.Signature
import mathlingua.frontend.FrontEnd
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.textalk.Command
import mathlingua.frontend.textalk.CommandPart
import mathlingua.frontend.textalk.ExpressionTexTalkNode
import mathlingua.frontend.textalk.GroupTexTalkNode
import mathlingua.frontend.textalk.OperatorTexTalkNode
import mathlingua.frontend.textalk.ParametersTexTalkNode
import mathlingua.frontend.textalk.TexTalkNodeType
import mathlingua.frontend.textalk.TexTalkTokenType
import mathlingua.frontend.textalk.TextTexTalkNode
import org.junit.jupiter.api.Test

internal class MathLinguaTest {

    @Test
    fun findDuplicateContentNoDuplicates() {
        val input =
            """
            [\finite.set]
            Defines: X
            where: '\something'
            means: '\something'
            written: "something"

            [\infinite.set]
            Defines: X
            where: '\something'
            means: '\something.else'
            written: "something"
        """.trimIndent()

        val supplemental =
            listOf(
                """
            [\set]
            Defines: X
            where: 'something'
            means:
            . if: X
              then: '\something.else'
            written: "something"
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
            where: 'X is \something'
            means: '\something'
            written: "something"

            [\another.name]
            Defines: X
            where: 'X is \something'
            means: '\something'
            written: "something"

            [\infinite.set]
            Defines: X
            where: 'X is \something'
            means: '\something.else'
            written: "something"
        """.trimIndent()

        val supplemental =
            listOf(
                """
            [\set]
            Defines: X
            where: 'X is \something'
            means:
            . if: X
              then: '\something.else'
            written: "something"
        """.trimIndent(),
                """
            Theorem:
            then:
            . '\finite.set'
        """.trimIndent())

        val dups = MathLingua.findDuplicateContent(input, supplemental)
        assertThat(dups).isEqualTo(listOf(Location(row = 6, column = 1)))
    }

    @Test
    fun findDuplicateSignaturesDuplicatesInInput() {
        val input =
            """
            [\finite.set]
            Defines: X
            where: 'X is \something'
            means: '\something'
            written: "something"

            [\infinite.set]
            Defines: X
            where: 'X is \something'
            means: '\something.else'
            written: "something"

            [\finite.set]
            Defines: Y
            where: 'X is \something'
            means: '\yet.something.else'
            written: "something"
        """.trimIndent()

        val supplemental =
            listOf(
                """
            [\set]
            Defines: X
            where: 'X is \something'
            means:
            . if: X
              then: '\something.else'
            written: "something"
        """.trimIndent(),
                """
            Theorem:
            then:
            . '\finite.set'
        """.trimIndent())

        val dups = MathLingua.findDuplicateSignatures(input, supplemental)
        assertThat(dups)
            .isEqualTo(
                listOf(Signature(form = "\\finite.set", location = Location(row = 12, column = 1))))
    }

    @Test
    fun findDuplicateContentDuplicatesWithSupplemental() {
        val input =
            """
            [\finite.set]
            Defines: X
            where: 'X is \something'
            means: '\something'
            written: "something"

            [\infinite.set]
            Defines: X
            where: 'X is \something'
            means: '\something.else'
            written: "something"
        """.trimIndent()

        val supplemental =
            listOf(
                """
            [\set]
            Defines: X
            where: 'X is \something'
            means:
            . if: X
              then: '\something.else'
            written: "something"

            [\another.name]
            Defines: X
            where: 'X is \something'
            means: '\something'
            written: "something"
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
            where: 'X is \something'
            means: '\something'
            written: "something"

            [\infinite.set]
            Defines: X
            where: 'X is \something'
            means: '\something.else'
            written: "something"
        """.trimIndent()

        val supplemental =
            listOf(
                """
            [\set]
            Defines: X
            where: 'X is \something'
            means:
            . if: X
              then: '\something.else'
            written: "something"
        """.trimIndent(),
                """
            Theorem:
            then:
            . '\finite.set'

            [\finite.set]
            Defines: Y
            where: 'X is \something'
            means: '\yet.something.else'
            written: "something"
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
            where: 'X is \something'
            means: '\something'
            written: "something"

            [\infinite.set]
            Defines: X
            where: 'X is \something'
            means: '\something.else'
            written: "something"
        """.trimIndent()

        val supplemental =
            listOf(
                """
            [\set]
            Defines: X
            where: 'X is \something'
            means:
            . if: X
              then: '\something.else'
            written: "something"

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
            where: 'X is \something'
            means: '\something'
            written: "something"

            [\infinite.set]
            Defines: X
            where: 'X is \something'
            means: '\something.else'
            written: "something"
        """.trimIndent()

        val supplemental =
            listOf(
                """
            [\set]
            Defines: X
            where: 'X is \something'
            means:
            . if: X
              then: '\something.else'
            written: "something"
        """.trimIndent(),
                """
            Theorem:
            then:
            . '\finite.set'

            [\set]
            Defines: Y
            where: 'X is \something'
            means: '\yet.something.else'
            written: "something"
        """.trimIndent())

        val dups = MathLingua.findDuplicateSignatures(input, supplemental)
        assertThat(dups).isEmpty()
    }

    @Test
    fun expandWrittenAs() {
        val validation =
            FrontEnd.parse(
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
                            namedGroups = emptyList())),
                false)
        val expected =
            mapOf(
                OperatorTexTalkNode(lhs = null, command = expectedCommand, rhs = null) to
                    "a? \\text{ or } b?")
        assertThat(map).isEqualTo(expected)
    }
}

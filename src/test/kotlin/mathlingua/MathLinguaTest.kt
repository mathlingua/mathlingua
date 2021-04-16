package mathlingua.mathlingua

import assertk.assertThat
import assertk.assertions.isEmpty
import assertk.assertions.isEqualTo
import mathlingua.backend.getPatternsToWrittenAs
import mathlingua.backend.newSourceCollectionFromContent
import mathlingua.backend.transform.Signature
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
            listOf(
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
        """.trimIndent(),
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
        val sourceCollection = newSourceCollectionFromContent(input)
        assertThat(sourceCollection.getDuplicateContent()).isEmpty()
    }

    @Test
    fun findDuplicateContentDuplicatesInInput() {
        val input =
            listOf(
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
        """.trimIndent(),
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

        val sourceCollection = newSourceCollectionFromContent(input)
        val locations =
            sourceCollection.getDuplicateContent().mapNotNull {
                if (it.tracker != null) {
                    it.tracker!!.getLocationOf(it.value)
                } else {
                    null
                }
            }
        assertThat(locations).isEmpty()
    }

    @Test
    fun findDuplicateSignaturesDuplicatesInInput() {
        val input =
            listOf(
                """
            [\finite.set]
            Defines: X
            means: 'X is \something'
            satisfying: '\something'
            written: "something"

            [\infinite.set]
            Defines: X
            means: 'X is \something'
            satisfying: '\something.else'
            written: "something"

            [\finite.set]
            Defines: Y
            means: 'X is \something'
            satisfying: '\yet.something.else'
            written: "something"
        """.trimIndent(),
                """
            [\set]
            Defines: X
            means: 'X is \something'
            satisfying:
            . if: X
              then: '\something.else'
            written: "something"
        """.trimIndent(),
                """
            Theorem:
            then:
            . '\finite.set'
        """.trimIndent())

        val sourceCollection = newSourceCollectionFromContent(input)
        val dups = sourceCollection.getDuplicateDefinedSignatures().map { it.value }
        assertThat(dups)
            .isEqualTo(
                listOf(Signature(form = "\\finite.set", location = Location(row = 12, column = 0))))
    }

    @Test
    fun findDuplicateContentDuplicatesWithSupplemental() {
        val input =
            listOf(
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
        """.trimIndent(),
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

        val collection = newSourceCollectionFromContent(input)
        val dups =
            collection.getDuplicateContent().mapNotNull {
                if (it.tracker != null) {
                    it.tracker!!.getLocationOf(it.value)
                } else {
                    null
                }
            }
        assertThat(dups).isEmpty()
    }

    @Test
    fun findDuplicateSignaturesDuplicatesWithSupplemental() {
        val input =
            listOf(
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
        """.trimIndent(),
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

        val collection = newSourceCollectionFromContent(input)
        val dups =
            collection.getDuplicateContent().mapNotNull {
                if (it.tracker != null) {
                    it.tracker!!.getLocationOf(it.value)
                } else {
                    null
                }
            }
        assertThat(dups).isEmpty()
    }

    @Test
    fun findDuplicateContentDuplicatesAllInSupplemental() {
        val input =
            listOf(
                """
            [\finite.set]
            Defines: X
            means: 'X is \something'
            satisfying: '\something'
            written: "something"

            [\infinite.set]
            Defines: X
            means: 'X is \something'
            satisfying: '\something.else'
            written: "something"
        """.trimIndent(),
                """
            [\set]
            Defines: X
            means: 'X is \something'
            satisfying:
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

        val collection = newSourceCollectionFromContent(input)
        val dups = collection.getDuplicateContent().map { it.value.toCode(false, 0).getCode() }
        assertThat(dups)
            .isEqualTo(
                listOf(
                    """
            Theorem:
            then:
            . '\finite.set'
        """.trimIndent()))
    }

    @Test
    fun findDuplicateSignaturesDuplicatesAllInSupplemental() {
        val input =
            listOf(
                """
            [\finite.set]
            Defines: X
            means: 'X is \something'
            satisfying: '\something'
            written: "something"

            [\infinite.set]
            Defines: X
            means: 'X is \something'
            satisfying: '\something.else'
            written: "something"
        """.trimIndent(),
                """
            [\set]
            Defines: X
            means: 'X is \something'
            satisfying:
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
            means: 'X is \something'
            satisfying: '\yet.something.else'
            written: "something"
        """.trimIndent())

        val collection = newSourceCollectionFromContent(input)
        val dups = collection.getDuplicateDefinedSignatures().map { it.value }
        assertThat(dups)
            .isEqualTo(listOf(Signature(form = "\\set", location = Location(row = 4, column = 0))))
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
        val map = getPatternsToWrittenAs(emptyList(), doc.states(), emptyList(), emptyList())
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

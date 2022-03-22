package mathlingua.backend.transform

import assertk.assertThat
import assertk.assertions.isEqualTo
import assertk.assertions.isFalse
import assertk.assertions.isTrue
import mathlingua.frontend.FrontEnd
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import org.junit.jupiter.api.Test

internal class TypeManagerTest {

    @Test
    fun testIsSigDescendantOf() {
        val code =
            """
            [\A]
            Defines: A
            satisfying: "A"
            written: "A"


            [\B]
            Defines: B
            means: 'B is \A'
            satisfying: "B"
            written: "B"


            [\C]
            Defines: C
            means: 'C is \B'
            satisfying: "C"
            written: "C"


            [\D]
            Defines: D
            means: 'D is \A'
            satisfying: "D"
            written: "D"


            [\E]
            Defines: E
            satisfying: "E"
            written: "E"
            Providing:
            . view:
              as: '\A'
              via: 'E'
            . view:
              as: '\B'
              via: 'E'
        """.trimIndent()
        // C -> B -> A
        // D -> A
        // E
        val validation = FrontEnd.parse(code)
        if (validation is ValidationFailure) {
            validation.errors.forEach(::println)
        }
        assertThat(validation is ValidationSuccess)
        val doc = (validation as ValidationSuccess).value

        val man = newTypeManager()
        doc.defines().forEach { man.add(it) }
        assertThat(man.isSigDescendantOf("\\C", "\\A")).isTrue()
        assertThat(man.isSigDescendantOf("\\B", "\\A")).isTrue()
        assertThat(man.isSigDescendantOf("\\A", "\\A")).isTrue()
        assertThat(man.isSigDescendantOf("\\D", "\\A")).isTrue()
        assertThat(man.isSigDescendantOf("\\E", "\\A")).isFalse()
        assertThat(man.isSigDescendantOf("\\E", "\\B")).isFalse()
        assertThat(man.isSigDescendantOf("\\E", "\\C")).isFalse()
        assertThat(man.isSigDescendantOf("\\E", "\\D")).isFalse()
        assertThat(man.isSigDescendantOf("\\A", "\\E")).isFalse()
        assertThat(man.isSigDescendantOf("\\B", "\\E")).isFalse()
        assertThat(man.isSigDescendantOf("\\C", "\\E")).isFalse()
    }

    @Test
    fun testIsSigViewableAs() {
        val code =
            """
            [\A]
            Defines: A
            satisfying: "A"
            written: "A"


            [\B]
            Defines: B
            means: 'B is \A'
            satisfying: "B"
            written: "B"


            [\C]
            Defines: C
            means: 'C is \B'
            satisfying: "C"
            written: "C"


            [\D]
            Defines: D
            satisfying: "D"
            written: "D"
            Providing:
            . view:
              as: '\C'
              via: 'D'


            [\E]
            Defines: E
            satisfying: "E"
            written: "E"
            Providing:
            . view:
              as: '\D'
              via: 'E'

            [\F]
            Defines: F
            satisfying: "F"
            written: "F"
            Providing:
            . view:
              as: '\A'
              via: 'F'
        """.trimIndent()
        val validation = FrontEnd.parse(code)
        assertThat(validation is ValidationSuccess)
        val doc = (validation as ValidationSuccess).value

        val man = newTypeManager()
        doc.defines().forEach { man.add(it) }
        assertThat(man.isSigViewableAs("\\A", "\\A")).isTrue()
        assertThat(man.isSigViewableAs("\\B", "\\A")).isTrue()
        assertThat(man.isSigViewableAs("\\E", "\\D")).isTrue()
        assertThat(man.isSigViewableAs("\\D", "\\C")).isTrue()
        assertThat(man.isSigViewableAs("\\E", "\\C")).isTrue()
        assertThat(man.isSigViewableAs("\\E", "\\B")).isTrue()
        assertThat(man.isSigViewableAs("\\E", "\\A")).isTrue()
        assertThat(man.isSigViewableAs("\\E", "\\E")).isTrue()
        assertThat(man.isSigViewableAs("\\F", "\\A")).isTrue()
        assertThat(man.isSigViewableAs("\\F", "\\B")).isFalse()
        assertThat(man.isSigViewableAs("\\F", "\\C")).isFalse()
        assertThat(man.isSigViewableAs("\\F", "\\D")).isFalse()
        assertThat(man.isSigViewableAs("\\F", "\\E")).isFalse()
    }

    @Test
    fun testLineage() {
        val code =
            """
            [\A]
            Defines: A
            satisfying: "A"
            written: "A"


            [\B]
            Defines: B
            means: 'B is \A'
            satisfying: "B"
            written: "B"


            [\C]
            Defines: C
            means: 'C is \B'
            satisfying: "C"
            written: "C"


            [\D]
            Defines: D
            means: 'D is \A'
            satisfying: "D"
            written: "D"


            [\E]
            Defines: E
            satisfying: "E"
            written: "E"
            Providing:
            . view:
              as: '\A'
              via: 'E'
            . view:
              as: '\B'
              via: 'E'
        """.trimIndent()
        // C -> B -> A
        // D -> A
        // E
        val validation = FrontEnd.parse(code)
        assertThat(validation is ValidationSuccess)
        val doc = (validation as ValidationSuccess).value

        val man = newTypeManager()
        doc.defines().forEach { man.add(it) }

        assertThat(man.getLineage("\\C")).isEqualTo(listOf("\\C", "\\B", "\\A"))
        assertThat(man.getLineage("\\B")).isEqualTo(listOf("\\B", "\\A"))
        assertThat(man.getLineage("\\A")).isEqualTo(listOf("\\A"))
        assertThat(man.getLineage("\\D")).isEqualTo(listOf("\\D", "\\A"))
        // verify that lineage doesn't follow view:as: items
        assertThat(man.getLineage("\\E")).isEqualTo(listOf("\\E"))
    }

    @Test
    fun testLeastCommonAncestor() {
        val code =
            """
            [\A1]
            Defines: A1
            satisfying: "A1"
            written: "A1"


            [\A2]
            Defines: A2
            means: 'A2 is \A1'
            satisfying: "A2"
            written: "A2"


            [\A3]
            Defines: A3
            means: 'A3 is \A2'
            satisfying: "A3"
            written: "A3"


            [\A4]
            Defines: A4
            means: 'A4 is \A3'
            satisfying: "A4"
            written: "A4"


            [\B1]
            Defines: B1
            means: 'B1 is \A2'
            satisfying: "B1"
            written: "B1"


            [\B2]
            Defines: B2
            means: 'B2 is \B1'
            satisfying: "B2"
            written: "B2"


            [\C1]
            Defines: C1
            means: 'C1 is \A2'
            satisfying: "C1"
            written: "C1"


            [\D1]
            Defines: D1
            satisfying: "D1"
            written: "D1"


            [\D2]
            Defines: D2
            means: 'D2 is \D1'
            satisfying: "D2"
            written: "D2"
        """.trimIndent()
        // \A4 -> \A3 -> \A2 -> \A1
        // \B2 -> \B1 -> \A2
        // \C1 -> \A2
        // \D2 -> \D1
        val validation = FrontEnd.parse(code)
        assertThat(validation is ValidationSuccess)
        val doc = (validation as ValidationSuccess).value

        val man = newTypeManager()
        doc.defines().forEach { man.add(it) }

        assertThat(man.getLeastCommonAncestor(emptyList())).isEqualTo(null)

        assertThat(man.getLeastCommonAncestor(listOf("\\A4", "\\B2", "\\C1"))).isEqualTo("\\A2")
        assertThat(man.getLeastCommonAncestor(listOf("\\A3", "\\B2", "\\C1"))).isEqualTo("\\A2")
        assertThat(man.getLeastCommonAncestor(listOf("\\A2", "\\B2", "\\C1"))).isEqualTo("\\A2")

        assertThat(man.getLeastCommonAncestor(listOf("\\A4", "\\B1", "\\C1"))).isEqualTo("\\A2")
        assertThat(man.getLeastCommonAncestor(listOf("\\A3", "\\B1", "\\C1"))).isEqualTo("\\A2")
        assertThat(man.getLeastCommonAncestor(listOf("\\A2", "\\B1", "\\C1"))).isEqualTo("\\A2")

        assertThat(man.getLeastCommonAncestor(listOf("\\A4", "\\B2", "\\C1", "\\D2")))
            .isEqualTo(null)
        assertThat(man.getLeastCommonAncestor(listOf("\\A3", "\\B2", "\\C1", "\\D2")))
            .isEqualTo(null)
        assertThat(man.getLeastCommonAncestor(listOf("\\A2", "\\B2", "\\C1", "\\D2")))
            .isEqualTo(null)

        assertThat(man.getLeastCommonAncestor(listOf("\\A4", "\\B2", "\\C1", "\\D1")))
            .isEqualTo(null)
        assertThat(man.getLeastCommonAncestor(listOf("\\A3", "\\B2", "\\C1", "\\D1")))
            .isEqualTo(null)
        assertThat(man.getLeastCommonAncestor(listOf("\\A2", "\\B2", "\\C1", "\\D1")))
            .isEqualTo(null)
    }
}

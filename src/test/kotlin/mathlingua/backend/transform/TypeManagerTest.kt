package mathlingua.backend.transform

import assertk.assertThat
import assertk.assertions.isFalse
import assertk.assertions.isTrue
import mathlingua.frontend.FrontEnd
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import org.junit.jupiter.api.Test

internal class TypeManagerTest {

    @Test
    fun testIsSigDescendentOf() {
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
        assertThat(man.isSigDescendentOf("\\C", "\\A")).isTrue()
        assertThat(man.isSigDescendentOf("\\B", "\\A")).isTrue()
        assertThat(man.isSigDescendentOf("\\A", "\\A")).isTrue()
        assertThat(man.isSigDescendentOf("\\D", "\\A")).isTrue()
        assertThat(man.isSigDescendentOf("\\E", "\\A")).isFalse()
        assertThat(man.isSigDescendentOf("\\E", "\\B")).isFalse()
        assertThat(man.isSigDescendentOf("\\E", "\\C")).isFalse()
        assertThat(man.isSigDescendentOf("\\E", "\\D")).isFalse()
        assertThat(man.isSigDescendentOf("\\A", "\\E")).isFalse()
        assertThat(man.isSigDescendentOf("\\B", "\\E")).isFalse()
        assertThat(man.isSigDescendentOf("\\C", "\\E")).isFalse()
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
}

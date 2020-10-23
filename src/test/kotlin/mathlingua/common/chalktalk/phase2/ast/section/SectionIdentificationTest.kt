package mathlingua.common.chalktalk.phase2.ast.section

import assertk.assertThat
import assertk.assertions.isEqualTo
import mathlingua.common.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.common.chalktalk.phase1.ast.Phase1Token
import mathlingua.common.chalktalk.phase1.ast.Section
import org.junit.jupiter.api.Test

internal class SectionIdentificationTest {
    @Test
    fun identifiesUniqueRequiredSectionNames() {
        val sections = listOf(
            Section(Phase1Token("Theorem", ChalkTalkTokenType.Name, -1, -1), emptyList()),
            Section(Phase1Token("given", ChalkTalkTokenType.Name, -1, -1), emptyList()),
            Section(Phase1Token("where", ChalkTalkTokenType.Name, -1, -1), emptyList()),
            Section(Phase1Token("then", ChalkTalkTokenType.Name, -1, -1), emptyList()),
            Section(Phase1Token("using", ChalkTalkTokenType.Name, -1, -1), emptyList())
        )
        val ids = identifySections(sections, "Theorem", "given", "where", "then", "using")
        assertThat(ids.keys).isEqualTo(setOf("Theorem", "given", "where", "then", "using"))
    }

    @Test
    fun identifiesUniqueOptionalAllSuppliedSectionNames() {
        val sections = listOf(
            Section(Phase1Token("Theorem", ChalkTalkTokenType.Name, -1, -1), emptyList()),
            Section(Phase1Token("given", ChalkTalkTokenType.Name, -1, -1), emptyList()),
            Section(Phase1Token("where", ChalkTalkTokenType.Name, -1, -1), emptyList()),
            Section(Phase1Token("then", ChalkTalkTokenType.Name, -1, -1), emptyList()),
            Section(Phase1Token("using", ChalkTalkTokenType.Name, -1, -1), emptyList())
        )
        val ids = identifySections(sections, "Theorem", "given?", "where?", "then", "using?")
        assertThat(ids.keys).isEqualTo(setOf("Theorem", "given", "where", "then", "using"))
    }

    @Test
    fun identifiesUniqueOptionalSomeOmittedSectionNames() {
        val sections = listOf(
            Section(Phase1Token("Theorem", ChalkTalkTokenType.Name, -1, -1), emptyList()),
            Section(Phase1Token("then", ChalkTalkTokenType.Name, -1, -1), emptyList()),
            Section(Phase1Token("using", ChalkTalkTokenType.Name, -1, -1), emptyList())
        )
        val ids = identifySections(sections, "Theorem", "given?", "where?", "then", "using?")
        assertThat(ids.keys).isEqualTo(setOf("Theorem", "then", "using"))
    }

    @Test
    fun identifiesDuplicateOptionalAllSuppliedSectionNames() {
        val sections = listOf(
            Section(Phase1Token("Theorem", ChalkTalkTokenType.Name, -1, -1), emptyList()),
            Section(Phase1Token("given", ChalkTalkTokenType.Name, -1, -1), emptyList()),
            Section(Phase1Token("where", ChalkTalkTokenType.Name, 1, 1), emptyList()),
            Section(Phase1Token("then", ChalkTalkTokenType.Name, -1, -1), emptyList()),
            Section(Phase1Token("using", ChalkTalkTokenType.Name, -1, -1), emptyList()),
            Section(Phase1Token("where", ChalkTalkTokenType.Name, 2, 2), emptyList())
        )
        val ids = identifySections(sections, "Theorem", "given?", "where?", "then", "using?", "where?")
        assertThat(ids.keys).isEqualTo(setOf("Theorem", "given", "where", "then", "using", "where1"))
        assertThat(ids["where"]!!.name.row).isEqualTo(1)
        assertThat(ids["where1"]!!.name.row).isEqualTo(2)
    }

    @Test
    fun identifiesDuplicateOptionalFirstOmittedSectionNames() {
        val sections = listOf(
            Section(Phase1Token("Theorem", ChalkTalkTokenType.Name, -1, -1), emptyList()),
            Section(Phase1Token("then", ChalkTalkTokenType.Name, -1, -1), emptyList()),
            Section(Phase1Token("using", ChalkTalkTokenType.Name, -1, -1), emptyList()),
            Section(Phase1Token("where", ChalkTalkTokenType.Name, 2, 2), emptyList())
        )
        val ids = identifySections(sections, "Theorem", "given?", "where?", "then", "using?", "where?")
        assertThat(ids.keys).isEqualTo(setOf("Theorem", "then", "using", "where1"))
        assertThat(ids["where1"]!!.name.row).isEqualTo(2)
    }

    @Test
    fun identifiesDuplicateOptionalSecondOmittedSectionNames() {
        val sections = listOf(
            Section(Phase1Token("Theorem", ChalkTalkTokenType.Name, -1, -1), emptyList()),
            Section(Phase1Token("where", ChalkTalkTokenType.Name, 1, 1), emptyList()),
            Section(Phase1Token("then", ChalkTalkTokenType.Name, -1, -1), emptyList()),
            Section(Phase1Token("using", ChalkTalkTokenType.Name, -1, -1), emptyList())
        )
        val ids = identifySections(sections, "Theorem", "given?", "where?", "then", "using?", "where?")
        assertThat(ids.keys).isEqualTo(setOf("Theorem", "where", "then", "using"))
        assertThat(ids["where"]!!.name.row).isEqualTo(1)
    }
}

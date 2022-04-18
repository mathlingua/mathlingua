package mathlingua.lib.frontend.chalktalk

import mathlingua.lib.frontend.Diagnostic
import mathlingua.lib.frontend.DiagnosticType
import mathlingua.lib.frontend.ast.AndGroup
import mathlingua.lib.frontend.ast.AndSection
import mathlingua.lib.frontend.ast.AsSection
import mathlingua.lib.frontend.ast.AuthorGroup
import mathlingua.lib.frontend.ast.AuthorSection
import mathlingua.lib.frontend.ast.AxiomGroup
import mathlingua.lib.frontend.ast.AxiomSection
import mathlingua.lib.frontend.ast.BeginArgument
import mathlingua.lib.frontend.ast.BeginGroup
import mathlingua.lib.frontend.ast.BeginSection
import mathlingua.lib.frontend.ast.BetweenSection
import mathlingua.lib.frontend.ast.BySection
import mathlingua.lib.frontend.ast.CalledSection
import mathlingua.lib.frontend.ast.Clause
import mathlingua.lib.frontend.ast.ConjectureGroup
import mathlingua.lib.frontend.ast.ConjectureSection
import mathlingua.lib.frontend.ast.ContentSection
import mathlingua.lib.frontend.ast.DefinesGroup
import mathlingua.lib.frontend.ast.DefinesSection
import mathlingua.lib.frontend.ast.Document
import mathlingua.lib.frontend.ast.EndArgument
import mathlingua.lib.frontend.ast.EndGroup
import mathlingua.lib.frontend.ast.EndSection
import mathlingua.lib.frontend.ast.EqualityGroup
import mathlingua.lib.frontend.ast.EqualitySection
import mathlingua.lib.frontend.ast.ExistsGroup
import mathlingua.lib.frontend.ast.ExistsSection
import mathlingua.lib.frontend.ast.ExistsUniqueGroup
import mathlingua.lib.frontend.ast.ExistsUniqueSection
import mathlingua.lib.frontend.ast.ExpressingItem
import mathlingua.lib.frontend.ast.ExpressingSection
import mathlingua.lib.frontend.ast.ForAllGroup
import mathlingua.lib.frontend.ast.ForAllSection
import mathlingua.lib.frontend.ast.FromSection
import mathlingua.lib.frontend.ast.GeneratedGroup
import mathlingua.lib.frontend.ast.GeneratedSection
import mathlingua.lib.frontend.ast.GeneratedWhenSection
import mathlingua.lib.frontend.ast.GivenSection
import mathlingua.lib.frontend.ast.HomepageGroup
import mathlingua.lib.frontend.ast.HomepageSection
import mathlingua.lib.frontend.ast.Id
import mathlingua.lib.frontend.ast.IfGroup
import mathlingua.lib.frontend.ast.IfSection
import mathlingua.lib.frontend.ast.IffGroup
import mathlingua.lib.frontend.ast.IffSection
import mathlingua.lib.frontend.ast.IsSection
import mathlingua.lib.frontend.ast.MatchingGroup
import mathlingua.lib.frontend.ast.MatchingSection
import mathlingua.lib.frontend.ast.MeansSection
import mathlingua.lib.frontend.ast.MemberSymbolsGroup
import mathlingua.lib.frontend.ast.MemberSymbolsSection
import mathlingua.lib.frontend.ast.MemberSymbolsWhereSection
import mathlingua.lib.frontend.ast.MembershipGroup
import mathlingua.lib.frontend.ast.MembershipSection
import mathlingua.lib.frontend.ast.MetadataItem
import mathlingua.lib.frontend.ast.MetadataSection
import mathlingua.lib.frontend.ast.NameGroup
import mathlingua.lib.frontend.ast.NameSection
import mathlingua.lib.frontend.ast.NegativeFloatGroup
import mathlingua.lib.frontend.ast.NegativeFloatSection
import mathlingua.lib.frontend.ast.NegativeIntSection
import mathlingua.lib.frontend.ast.NodeLexerToken
import mathlingua.lib.frontend.ast.NotGroup
import mathlingua.lib.frontend.ast.NotSection
import mathlingua.lib.frontend.ast.NoteGroup
import mathlingua.lib.frontend.ast.NoteSection
import mathlingua.lib.frontend.ast.NoteTopLevelGroup
import mathlingua.lib.frontend.ast.NoteTopLevelSection
import mathlingua.lib.frontend.ast.OffsetGroup
import mathlingua.lib.frontend.ast.OffsetSection
import mathlingua.lib.frontend.ast.OrGroup
import mathlingua.lib.frontend.ast.OrSection
import mathlingua.lib.frontend.ast.PiecewiseElseSection
import mathlingua.lib.frontend.ast.PiecewiseGroup
import mathlingua.lib.frontend.ast.PiecewiseSection
import mathlingua.lib.frontend.ast.PiecewiseThenSection
import mathlingua.lib.frontend.ast.PiecewiseWhenSection
import mathlingua.lib.frontend.ast.PositiveFloatGroup
import mathlingua.lib.frontend.ast.PositiveFloatSection
import mathlingua.lib.frontend.ast.PositiveIntSection
import mathlingua.lib.frontend.ast.ProofSection
import mathlingua.lib.frontend.ast.ProvidedSection
import mathlingua.lib.frontend.ast.ProvidingItem
import mathlingua.lib.frontend.ast.ProvidingSection
import mathlingua.lib.frontend.ast.ReferenceGroup
import mathlingua.lib.frontend.ast.ReferenceSection
import mathlingua.lib.frontend.ast.ResourceGroup
import mathlingua.lib.frontend.ast.ResourceItem
import mathlingua.lib.frontend.ast.ResourceSection
import mathlingua.lib.frontend.ast.SatisfyingItem
import mathlingua.lib.frontend.ast.SatisfyingSection
import mathlingua.lib.frontend.ast.Section
import mathlingua.lib.frontend.ast.Spec
import mathlingua.lib.frontend.ast.SpecifyGroup
import mathlingua.lib.frontend.ast.SpecifyItem
import mathlingua.lib.frontend.ast.SpecifySection
import mathlingua.lib.frontend.ast.Statement
import mathlingua.lib.frontend.ast.StatesGroup
import mathlingua.lib.frontend.ast.StatesSection
import mathlingua.lib.frontend.ast.SuchThatSection
import mathlingua.lib.frontend.ast.SymbolsGroup
import mathlingua.lib.frontend.ast.SymbolsSection
import mathlingua.lib.frontend.ast.SymbolsWhereSection
import mathlingua.lib.frontend.ast.TagGroup
import mathlingua.lib.frontend.ast.TagSection
import mathlingua.lib.frontend.ast.Text
import mathlingua.lib.frontend.ast.TextBlock
import mathlingua.lib.frontend.ast.ThatItem
import mathlingua.lib.frontend.ast.ThatSection
import mathlingua.lib.frontend.ast.ThenSection
import mathlingua.lib.frontend.ast.TheoremGroup
import mathlingua.lib.frontend.ast.TheoremSection
import mathlingua.lib.frontend.ast.ThroughSection
import mathlingua.lib.frontend.ast.TopLevelGroup
import mathlingua.lib.frontend.ast.TopLevelGroupOrTextBlock
import mathlingua.lib.frontend.ast.TopicGroup
import mathlingua.lib.frontend.ast.TopicSection
import mathlingua.lib.frontend.ast.TypeGroup
import mathlingua.lib.frontend.ast.TypeSection
import mathlingua.lib.frontend.ast.UrlGroup
import mathlingua.lib.frontend.ast.UrlSection
import mathlingua.lib.frontend.ast.UsingSection
import mathlingua.lib.frontend.ast.ViaSection
import mathlingua.lib.frontend.ast.ViewGroup
import mathlingua.lib.frontend.ast.ViewSection
import mathlingua.lib.frontend.ast.WhenSection
import mathlingua.lib.frontend.ast.WhereSection
import mathlingua.lib.frontend.ast.WithSection
import mathlingua.lib.frontend.ast.WritingSection
import mathlingua.lib.frontend.ast.WrittenSection
import mathlingua.lib.frontend.ast.ZeroGroup
import mathlingua.lib.frontend.ast.ZeroSection

internal data class ParseResult(val doc: Document, val diagnostics: List<Diagnostic>)

internal interface ChalkTalkParser {
    fun parse(): ParseResult
}

internal fun newChalkTalkParser(lexer: ChalkTalkNodeLexer): ChalkTalkParser {
    return ChalkTalkParserImpl(lexer)
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private class ChalkTalkParserImpl(val lexer: ChalkTalkNodeLexer) : ChalkTalkParser {
    private val diagnostics = mutableListOf<Diagnostic>()

    override fun parse(): ParseResult {
        return null!!
    }

    private fun document(): Document {
        return null!!
    }

    private fun topLevelGroupOrTextBlock(): TopLevelGroupOrTextBlock? {
        return topLevelGroup() ?: textBlock()
    }

    private fun topLevelGroup(): TopLevelGroup? {
        return null
    }

    private fun textBlock(): TextBlock? {
        return null
    }

    private fun id(): Id? {
        return null
    }

    private fun statement(): Statement? {
        return null
    }

    private fun text(): Text? {
        return null
    }

    private fun clause(): Clause? {
        return andGroup()
            ?: notGroup() ?: orGroup() ?: existsGroup() ?: existsUniqueGroup() ?: forAllGroup()
                ?: ifGroup() ?: iffGroup() ?: text() ?: statement()
    }

    private fun spec(): Spec? {
        return statement()
    }

    private fun andSection(): AndSection? = section<AndSection?>("and") { null }

    private fun andGroup(): AndGroup? {
        return null
    }

    private fun notSection(): NotSection? {
        return null
    }

    private fun notGroup(): NotGroup? {
        return null
    }

    private fun orSection(): OrSection? {
        return null
    }

    private fun orGroup(): OrGroup? {
        return null
    }

    private fun existsSection(): ExistsSection? {
        return null
    }

    private fun whereSection(): WhereSection? {
        return null
    }

    private fun suchThatSection(): SuchThatSection? {
        return null
    }

    private fun existsGroup(): ExistsGroup? {
        return null
    }

    private fun existsUniqueSection(): ExistsUniqueSection? {
        return null
    }

    private fun existsUniqueGroup(): ExistsUniqueGroup? {
        return null
    }

    private fun forAllSection(): ForAllSection? {
        return null
    }

    private fun forAllGroup(): ForAllGroup? {
        return null
    }

    private fun thenSection(): ThenSection? {
        return null
    }

    private fun ifSection(): IfSection? {
        return null
    }

    private fun ifGroup(): IfGroup? {
        return null
    }

    private fun iffSection(): IffSection? {
        return null
    }

    private fun iffGroup(): IffGroup? {
        return null
    }

    private fun generatedSection(): GeneratedSection? {
        return null
    }

    private fun fromSection(): FromSection? {
        return null
    }

    private fun generatedWhenSection(): GeneratedWhenSection? {
        return null
    }

    private fun generatedGroup(): GeneratedGroup? {
        return null
    }

    private fun piecewiseSection(): PiecewiseSection? {
        return null
    }

    private fun piecewiseWhenSection(): PiecewiseWhenSection? {
        return null
    }

    private fun piecewiseThenSection(): PiecewiseThenSection? {
        return null
    }

    private fun piecewiseElseSection(): PiecewiseElseSection? {
        return null
    }

    private fun piecewiseGroup(): PiecewiseGroup? {
        return null
    }

    private fun matchingSection(): MatchingSection? {
        return null
    }

    private fun matchingGroup(): MatchingGroup? {
        return null
    }

    private fun equalitySection(): EqualitySection? {
        return null
    }

    private fun betweenSection(): BetweenSection? {
        return null
    }

    private fun providedSection(): ProvidedSection? {
        return null
    }

    private fun equalityGroup(): EqualityGroup? {
        return null
    }

    private fun membershipSection(): MembershipSection? {
        return null
    }

    private fun throughSection(): ThroughSection? {
        return null
    }

    private fun membershipGroup(): MembershipGroup? {
        return null
    }

    private fun viewSection(): ViewSection? {
        return null
    }

    private fun asSection(): AsSection? {
        return null
    }

    private fun viaSection(): ViaSection? {
        return null
    }

    private fun bySection(): BySection? {
        return null
    }

    private fun viewGroup(): ViewGroup? {
        return null
    }

    private fun symbolsSection(): SymbolsSection? {
        return null
    }

    private fun symbolsWhereSection(): SymbolsWhereSection? {
        return null
    }

    private fun symbolsGroup(): SymbolsGroup? {
        return null
    }

    private fun memberSymbolsSection(): MemberSymbolsSection? {
        return null
    }

    private fun memberSymbolsWhereSection(): MemberSymbolsWhereSection? {
        return null
    }

    private fun memberSymbolsGroup(): MemberSymbolsGroup? {
        return null
    }

    private fun noteSection(): NoteSection? {
        return null
    }

    private fun noteGroup(): NoteGroup? {
        return null
    }

    private fun authorSection(): AuthorSection? {
        return null
    }

    private fun authorGroup(): AuthorGroup? {
        return null
    }

    private fun tagSection(): TagSection? {
        return null
    }

    private fun tagGroup(): TagGroup? {
        return null
    }

    private fun referenceSection(): ReferenceSection? {
        return null
    }

    private fun referenceGroup(): ReferenceGroup? {
        return null
    }

    private fun definesSection(): DefinesSection? {
        return null
    }

    private fun withSection(): WithSection? {
        return null
    }

    private fun givenSection(): GivenSection? {
        return null
    }

    private fun whenSection(): WhenSection? {
        return null
    }

    private fun meansSection(): MeansSection? {
        return null
    }

    private fun satisfyingItem(): SatisfyingItem? {
        return null
    }

    private fun satisfyingSection(): SatisfyingSection? {
        return null
    }

    private fun expressingItem(): ExpressingItem? {
        return null
    }

    private fun expressingSection(): ExpressingSection? {
        return null
    }

    private fun usingSection(): UsingSection? {
        return null
    }

    private fun writingSection(): WritingSection? {
        return null
    }

    private fun writtenSection(): WrittenSection? {
        return null
    }

    private fun calledSection(): CalledSection? {
        return null
    }

    private fun providingItem(): ProvidingItem? {
        return null
    }

    private fun providingSection(): ProvidingSection? {
        return null
    }

    private fun metadataItem(): MetadataItem? {
        return null
    }

    private fun metadataSection(): MetadataSection? {
        return null
    }

    private fun definesGroup(): DefinesGroup? {
        return null
    }

    private fun statesSection(): StatesSection? {
        return null
    }

    private fun statesGroup(): StatesGroup? {
        return null
    }

    private fun thatItem(): ThatItem? {
        return null
    }

    private fun thatSection(): ThatSection? {
        return null
    }

    private fun resourceItem(): ResourceItem? {
        return null
    }

    private fun resourceSection(): ResourceSection? {
        return null
    }

    private fun typeSection(): TypeSection? {
        return null
    }

    private fun typeGroup(): TypeGroup? {
        return null
    }

    private fun nameSection(): NameSection? {
        return null
    }

    private fun nameGroup(): NameGroup? {
        return null
    }

    private fun homepageSection(): HomepageSection? {
        return null
    }

    private fun homepageGroup(): HomepageGroup? {
        return null
    }

    private fun urlSection(): UrlSection? {
        return null
    }

    private fun urlGroup(): UrlGroup? {
        return null
    }

    private fun offsetSection(): OffsetSection? {
        return null
    }

    private fun offsetGroup(): OffsetGroup? {
        return null
    }

    private fun resourceGroup(): ResourceGroup? {
        return null
    }

    private fun axiomSection(): AxiomSection? {
        return null
    }

    private fun axiomGroup(): AxiomGroup? {
        return null
    }

    private fun conjectureSection(): ConjectureSection? {
        return null
    }

    private fun conjectureGroup(): ConjectureGroup? {
        return null
    }

    private fun theoremSection(): TheoremSection? {
        return null
    }

    private fun theoremGroup(): TheoremGroup? {
        return null
    }

    private fun proofSection(): ProofSection? {
        return null
    }

    private fun topicSection(): TopicSection? {
        return null
    }

    private fun contentSection(): ContentSection? {
        return null
    }

    private fun topicGroup(): TopicGroup? {
        return null
    }

    private fun noteTopLevelSection(): NoteTopLevelSection? {
        return null
    }

    private fun noteTopLevelGroup(): NoteTopLevelGroup? {
        return null
    }

    private fun specifyItem(): SpecifyItem? {
        return null
    }

    private fun specifySection(): SpecifySection? {
        return null
    }

    private fun specifyGroup(): SpecifyGroup? {
        return null
    }

    private fun zeroSection(): ZeroSection? {
        return null
    }

    private fun zeroGroup(): ZeroGroup? {
        return null
    }

    private fun positiveIntSection(): PositiveIntSection? {
        return null
    }

    private fun negativeIntSection(): NegativeIntSection? {
        return null
    }

    private fun positiveFloatSection(): PositiveFloatSection? {
        return null
    }

    private fun positiveFloatGroup(): PositiveFloatGroup? {
        return null
    }

    private fun negativeFloatSection(): NegativeFloatSection? {
        return null
    }

    private fun negativeFloatGroup(): NegativeFloatGroup? {
        return null
    }

    private fun isSection(): IsSection? {
        return null
    }

    private fun <T : Section> require(value: T?, default: T): T {
        val peek =
            if (lexer.hasNext()) {
                lexer.peek()
            } else {
                null
            }
        diagnostics.add(
            Diagnostic(
                type = DiagnosticType.Error,
                message = "Expected a '${default.name}' section",
                row = peek?.metadata?.row ?: -1,
                column = peek?.metadata?.column ?: -1))
        return value ?: default
    }

    private fun <T> group(expectedName: String, fn: () -> T): T? {
        if (!has(BeginGroup) || !hasHasBeginSection(expectedName)) {
            return null
        }

        expect(BeginGroup)
        val result = fn()
        expect(EndGroup)

        return result
    }

    private fun <T> section(expectedName: String, fn: () -> T): T? {
        if (!lexer.hasNext() ||
            lexer.peek() !is BeginSection ||
            (lexer.peek() as BeginSection).name != expectedName) {
            return null
        }

        lexer.next() // move past the BeginSection
        val result = fn()
        expect(EndSection)

        return result
    }

    private fun <T> argument(fn: () -> T): T? {
        if (!has(BeginArgument)) {
            return null
        }

        expect(BeginArgument)
        val result = fn()
        expect(EndArgument)

        return result
    }

    private fun has(token: NodeLexerToken) = lexer.hasNext() && lexer.peek() == token

    private fun hasHasBeginSection(name: String) =
        lexer.hasNextNext() &&
            lexer.peekPeek() is BeginSection &&
            (lexer.peekPeek() as BeginSection).name == name

    private fun expect(token: NodeLexerToken) {
        if (!lexer.hasNext()) {
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    message = "Expected $token but found the end of stream",
                    row = -1,
                    column = -1))
            return
        }

        val next = lexer.next()
        if (next != token) {
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    message = "Expected $token but found $next",
                    row = next.metadata.row,
                    column = next.metadata.column))
        }
    }
}

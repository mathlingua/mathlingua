package mathlingua.lib.frontend.chalktalk

import java.util.LinkedList
import java.util.Queue
import mathlingua.lib.frontend.Diagnostic
import mathlingua.lib.frontend.DiagnosticType
import mathlingua.lib.frontend.MetaData
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
import mathlingua.lib.frontend.ast.DEFAULT_AND_SECTION
import mathlingua.lib.frontend.ast.DEFAULT_CLAUSE
import mathlingua.lib.frontend.ast.DEFAULT_CONJECTURE_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_EXISTS_SECTION
import mathlingua.lib.frontend.ast.DEFAULT_EXISTS_UNIQUE_SECTION
import mathlingua.lib.frontend.ast.DEFAULT_ID
import mathlingua.lib.frontend.ast.DEFAULT_NOT_SECTION
import mathlingua.lib.frontend.ast.DEFAULT_OR_SECTION
import mathlingua.lib.frontend.ast.DEFAULT_THEOREM_GROUP
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
import mathlingua.lib.frontend.ast.Target
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
        return ParseResult(doc = document(), diagnostics = diagnostics)
    }

    private fun document(): Document {
        val items = mutableListOf<TopLevelGroupOrTextBlock>()
        while (true) {
            val item = topLevelGroupOrTextBlock() ?: break
            items.add(item)
        }

        while (lexer.hasNext()) {
            val next = lexer.next()
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    message = "Unexpected item $next",
                    row = next.metadata.row,
                    column = next.metadata.column))
        }

        return Document(items = items)
    }

    private fun topLevelGroupOrTextBlock(): TopLevelGroupOrTextBlock? {
        return topLevelGroup() ?: textBlock()
    }

    private fun topLevelGroup(): TopLevelGroup? =
        definesGroup()
            ?: statesGroup() ?: axiomGroup() ?: conjectureGroup() ?: theoremGroup() ?: topicGroup()
                ?: resourceGroup() ?: specifyGroup() ?: noteTopLevelGroup()

    private fun textBlock(): TextBlock? = getNextIfCorrectType()

    private fun id(): Id? = getNextIfCorrectType()

    private fun statement(): Statement? = getNextIfCorrectType()

    private fun text(): Text? = getNextIfCorrectType()

    private fun clause(): Clause? =
        andGroup()
            ?: notGroup() ?: orGroup() ?: existsGroup() ?: existsUniqueGroup() ?: forAllGroup()
                ?: ifGroup() ?: iffGroup() ?: text() ?: statement()

    private fun spec(): Spec? = statement()

    private fun clauses(): List<Clause> {
        val clauses = mutableListOf<Clause>()
        while (lexer.hasNext()) {
            val arg = argument { clause() } ?: break
            clauses.add(arg)
        }
        return clauses
    }

    private fun andSection(): AndSection? =
        section("and") { AndSection(clauses = clauses(), metadata = it) }

    private fun andGroup(): AndGroup? =
        group("and") {
            AndGroup(andSection = requiredSection(andSection(), DEFAULT_AND_SECTION), metadata = it)
        }

    private fun notSection(): NotSection? =
        section("not") { NotSection(clause = required(clause(), DEFAULT_CLAUSE), metadata = it) }

    private fun notGroup(): NotGroup? =
        group("not") {
            NotGroup(notSection = requiredSection(notSection(), DEFAULT_NOT_SECTION), metadata = it)
        }

    private fun orSection(): OrSection? =
        section("or") { OrSection(clauses = clauses(), metadata = it) }

    private fun orGroup(): OrGroup? =
        group("or") {
            OrGroup(orSection = requiredSection(orSection(), DEFAULT_OR_SECTION), metadata = it)
        }

    private fun target(): Target? = null

    private fun targets(): List<Target> {
        val targets = mutableListOf<Target>()
        while (lexer.hasNext()) {
            val target = target() ?: break
            targets.add(target)
        }
        return targets
    }

    private fun existsSection(): ExistsSection? =
        section("exists") { ExistsSection(targets = targets(), metadata = it) }

    private fun specs(): List<Spec> {
        val specs = mutableListOf<Spec>()
        while (lexer.hasNext()) {
            val spec = spec() ?: break
            specs.add(spec)
        }
        return specs
    }

    private fun whereSection(): WhereSection? =
        section("where") { WhereSection(specs = specs(), metadata = it) }

    private fun suchThatSection(): SuchThatSection? =
        section("suchThat") { SuchThatSection(clauses = clauses(), metadata = it) }

    private fun existsGroup(): ExistsGroup? =
        group("exists") {
            ExistsGroup(
                existsSection = requiredSection(existsSection(), DEFAULT_EXISTS_SECTION),
                whereSection = whereSection(),
                suchThatSection = suchThatSection(),
                metadata = it)
        }

    private fun existsUniqueSection(): ExistsUniqueSection? =
        section("existsUnique") { ExistsUniqueSection(targets = targets(), metadata = it) }

    private fun existsUniqueGroup(): ExistsUniqueGroup? =
        group("existsUnique") {
            ExistsUniqueGroup(
                existsUniqueSection =
                    requiredSection(existsUniqueSection(), DEFAULT_EXISTS_UNIQUE_SECTION),
                whereSection = whereSection(),
                suchThatSection = suchThatSection(),
                metadata = it)
        }

    private fun forAllSection(): ForAllSection? {
        return null
    }

    private fun forAllGroup(): ForAllGroup? {
        return null
    }

    private fun thenSection(): ThenSection? =
        section("then") { ThenSection(clauses = clauses(), metadata = it) }

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

    private fun conjectureGroup(): ConjectureGroup? =
        group(
            idRequired = false,
            specs =
                listOf(
                    SectionSpec(name = "Conjecture", required = true) { this.conjectureSection() },
                    SectionSpec(name = "given", required = false) { this.givenSection() },
                    SectionSpec(name = "where", required = false) { this.whereSection() },
                    SectionSpec(name = "suchThat", required = false) { this.suchThatSection() },
                    SectionSpec(name = "then", required = true) { this.thenSection() },
                    SectionSpec(name = "iff", required = false) { this.iffSection() },
                    SectionSpec(name = "using", required = false) { this.usingSection() },
                    SectionSpec(name = "Metadata", required = false) { this.metadataSection() }),
            default = DEFAULT_CONJECTURE_GROUP) { id, sections, metadata ->
            ConjectureGroup(
                id = id,
                conjectureSection = sections["Conjecture"] as ConjectureSection,
                givenSection = sections["given"] as GivenSection?,
                whereSection = sections["where"] as WhereSection?,
                suchThatSection = sections["suchThat"] as SuchThatSection?,
                thenSection = sections["then"] as ThenSection,
                iffSection = sections["iff"] as IffSection?,
                usingSection = sections["using"] as UsingSection?,
                metadataSection = sections["Metadata"] as MetadataSection?,
                metadata = metadata)
        }

    private fun theoremSection(): TheoremSection? =
        section("Theorem") { TheoremSection(metadata = it) }

    private fun theoremGroup(): TheoremGroup? =
        group(
            idRequired = false,
            specs =
                listOf(
                    SectionSpec(name = "Theorem", required = true) { this.theoremSection() },
                    SectionSpec(name = "given", required = false) { this.givenSection() },
                    SectionSpec(name = "where", required = false) { this.whereSection() },
                    SectionSpec(name = "suchThat", required = false) { this.suchThatSection() },
                    SectionSpec(name = "then", required = true) { this.thenSection() },
                    SectionSpec(name = "iff", required = false) { this.iffSection() },
                    SectionSpec(name = "using", required = false) { this.usingSection() },
                    SectionSpec(name = "proof", required = false) { this.proofSection() },
                    SectionSpec(name = "Metadata", required = false) { this.metadataSection() }),
            default = DEFAULT_THEOREM_GROUP) { id, sections, metadata ->
            TheoremGroup(
                id = id,
                theoremSection = sections["Theorem"] as TheoremSection,
                givenSection = sections["given"] as GivenSection?,
                whereSection = sections["where"] as WhereSection?,
                suchThatSection = sections["suchThat"] as SuchThatSection?,
                thenSection = sections["then"] as ThenSection,
                iffSection = sections["iff"] as IffSection?,
                usingSection = sections["using"] as UsingSection?,
                proofSection = sections["proof"] as ProofSection?,
                metadataSection = sections["Metadata"] as MetadataSection?,
                metadata = metadata)
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

    private inline fun <reified T> getNextIfCorrectType(): T? =
        if (nextIs<T>()) {
            lexer.next() as T
        } else {
            null
        }

    private inline fun <reified T> nextIs() = lexer.hasNext() && lexer.peek() is T

    private fun <T : Section> requiredSection(value: T?, default: T): T {
        val peek =
            if (lexer.hasNext()) {
                lexer.peek()
            } else {
                null
            }
        if (value == null) {
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    message = "Expected a '${default.name}' section",
                    row = peek?.metadata?.row ?: -1,
                    column = peek?.metadata?.column ?: -1))
        }
        return value ?: default
    }

    private fun <T> required(value: T?, default: T): T {
        val peek =
            if (lexer.hasNext()) {
                lexer.peek()
            } else {
                null
            }
        if (value == null) {
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    message = "Expected a '$default'",
                    row = peek?.metadata?.row ?: -1,
                    column = peek?.metadata?.column ?: -1))
        }
        return value ?: default
    }

    private fun <T> group(expectedName: String, fn: (metadata: MetaData) -> T): T? {
        if (!nextIs<BeginGroup>() || !hasHasBeginSection(expectedName)) {
            return null
        }

        val begin = expectIs<BeginGroup>()
        val result = fn(begin?.metadata ?: MetaData(row = -1, column = -1, isInline = false))
        expect(EndGroup)

        return result
    }

    private inline fun <reified T> section(
        expectedName: String, fn: (metadata: MetaData) -> T
    ): T? {
        if (!lexer.hasNext() ||
            lexer.peek() !is BeginSection ||
            (lexer.peek() as BeginSection).name != expectedName) {
            return null
        }

        val section = lexer.next() // move past the BeginSection
        val result = fn(section.metadata)
        expect(EndSection)

        return result
    }

    private fun <T> argument(fn: (metadata: MetaData) -> T): T? {
        if (!nextIs<BeginArgument>()) {
            return null
        }

        val begin = expectIs<BeginArgument>()
        val result = fn(begin?.metadata ?: MetaData(row = -1, column = -1, isInline = false))
        expect(EndArgument)

        return result
    }

    private fun has(token: NodeLexerToken) = lexer.hasNext() && lexer.peek() == token

    private fun hasHasBeginSection(name: String) =
        lexer.hasNextNext() &&
            lexer.peekPeek() is BeginSection &&
            (lexer.peekPeek() as BeginSection).name == name

    private inline fun <reified T> expectIs(): NodeLexerToken? {
        if (!lexer.hasNext() || lexer.peek() !is T) {
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    message = "Found a token of the wrong type",
                    row = -1,
                    column = -1))
            return null
        }
        return lexer.next()
    }

    private fun expect(token: NodeLexerToken): NodeLexerToken? {
        if (!lexer.hasNext()) {
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    message = "Expected $token but found the end of stream",
                    row = -1,
                    column = -1))
            return null
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
        return next
    }

    private fun identifySections(
        sections: List<Section>, expected: List<String>
    ): Map<String, Section>? {
        val patternBuilder = StringBuilder()
        for (name in expected) {
            patternBuilder.append(name)
            patternBuilder.append(":\n")
        }

        // the pattern is used for error messages
        val pattern = patternBuilder.toString()

        val sectionQueue: Queue<Section> = LinkedList()
        for (s in sections) {
            sectionQueue.offer(s)
        }

        val expectedQueue: Queue<String> = LinkedList()
        for (e in expected) {
            expectedQueue.offer(e)
        }

        val usedSectionNames = mutableMapOf<String, Int>()
        val result = mutableMapOf<String, Section>()

        while (!sectionQueue.isEmpty() && !expectedQueue.isEmpty()) {
            val nextSection = sectionQueue.peek()
            val maybeName = expectedQueue.peek()

            val isOptional = maybeName.endsWith("?")
            val trueName =
                if (isOptional) maybeName.substring(0, maybeName.length - 1) else maybeName
            val key =
                if (usedSectionNames.containsKey(trueName)) {
                    "$trueName${usedSectionNames[trueName]}"
                } else {
                    trueName
                }
            usedSectionNames[trueName] = usedSectionNames.getOrDefault(trueName, 0) + 1
            if (nextSection.name == trueName) {
                result[key] = nextSection
                // the expected name and Section have both been used so move past them
                sectionQueue.poll()
                expectedQueue.poll()
            } else if (isOptional) {
                // The Section found doesn't match the expected name
                // but the expected name is optional.  So move past
                // the expected name but don't move past the Section
                // so it can be processed again in the next run of
                // the loop.
                expectedQueue.poll()
            } else {
                diagnostics.add(
                    Diagnostic(
                        type = DiagnosticType.Error,
                        message =
                            "For pattern:\n\n" +
                                pattern +
                                "\nExpected '" +
                                trueName +
                                "' but found '" +
                                nextSection.name +
                                "'",
                        row = nextSection.metadata.row,
                        column = nextSection.metadata.column))
                return null
            }
        }

        if (!sectionQueue.isEmpty()) {
            val peek = sectionQueue.peek()
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    message =
                        "For pattern:\n\n" + pattern + "\nUnexpected Section '" + peek.name + "'",
                    row = peek.metadata.row,
                    column = peek.metadata.column))
            return null
        }

        var nextExpected: String? = null
        for (exp in expectedQueue) {
            if (!exp.endsWith("?")) {
                // trim the ?
                nextExpected = exp
                break
            }
        }

        var startRow = -1
        var startColumn = -1
        if (sections.isNotEmpty()) {
            val sect = sections[0]
            startRow = sect.metadata.row
            startColumn = sect.metadata.column
        }

        if (nextExpected != null) {
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    message = "For pattern:\n\n$pattern\nExpected a $nextExpected",
                    row = startRow,
                    column = startColumn))
            return null
        }

        return result
    }

    private fun <T> group(
        idRequired: Boolean,
        specs: List<SectionSpec>,
        default: T,
        builder: (id: Id?, sections: Map<String, Section?>, metadata: MetaData) -> T
    ): T? {
        if (!nextIs<BeginGroup>()) {
            return null
        }

        val namesToSpec = mutableMapOf<String, SectionSpec>()
        for (spec in specs) {
            namesToSpec[spec.name] = spec
        }

        val beginGroup = expectIs<BeginGroup>()!!

        var id = id()
        if (idRequired && id == null) {
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    message = "Expected an id",
                    row = beginGroup.metadata.row,
                    column = beginGroup.metadata.column))
            id = DEFAULT_ID
        }

        val sections = mutableListOf<EvaluatedSection>()
        while (lexer.hasNext()) {
            val nextIsSection = lexer.hasNext() && lexer.peek() is BeginSection
            if (!nextIsSection) {
                break
            }
            val sect = lexer.peek() as BeginSection
            if (namesToSpec.containsKey(sect.name)) {
                val spec = namesToSpec[sect.name]!!
                val result = spec.builder()
                sections.add(
                    EvaluatedSection(name = sect.name, required = spec.required, section = result))
            } else {
                val beginSection = expectIs<BeginSection>()!!
                diagnostics.add(
                    Diagnostic(
                        type = DiagnosticType.Error,
                        message = "Unexpected section '${sect.name}'",
                        row = beginSection.metadata.row,
                        column = beginGroup.metadata.column))
                while (lexer.hasNext() && !nextIs<EndSection>()) {
                    lexer.next()
                }
                expectIs<EndSection>()
            }
        }

        while (lexer.hasNext() && !nextIs<EndGroup>()) {
            val next = lexer.next()
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    message = "Unexpected item $next",
                    row = next.metadata.row,
                    column = next.metadata.column))
        }

        val mapping =
            identifySections(
                sections.mapNotNull { it.section },
                specs.map { "${it.name}${if (it.required) { "" } else { "?" }}" })
                ?: return default

        expect(EndGroup)
        return builder(
            id,
            mapping,
            MetaData(
                row = beginGroup.metadata.row,
                column = beginGroup.metadata.column,
                isInline = false))
    }
}

private data class SectionSpec(
    val name: String, val required: Boolean, val builder: () -> Section?)

private data class EvaluatedSection(val name: String, val required: Boolean, val section: Section?)

fun main() {
    val text =
        """
        [some id]
        Theorem:
        then:
        . and:
          . "abc"
    """.trimIndent()
    val lexer1 = newChalkTalkTokenLexer(text)
    val lexer2 = newChalkTalkNodeLexer(lexer1)
    val parser = newChalkTalkParser(lexer2)
    val result = parser.parse()
    println("Doc:")
    println(result.doc)
    println("Diagnostics:")
    result.diagnostics.forEach(::println)
}

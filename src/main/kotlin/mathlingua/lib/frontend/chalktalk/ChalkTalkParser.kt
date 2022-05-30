/*
 * Copyright 2022 Dominic Kramer
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package mathlingua.lib.frontend.chalktalk

import java.util.LinkedList
import java.util.Queue
import mathlingua.lib.frontend.Diagnostic
import mathlingua.lib.frontend.DiagnosticOrigin
import mathlingua.lib.frontend.DiagnosticType
import mathlingua.lib.frontend.MetaData
import mathlingua.lib.frontend.ast.AndGroup
import mathlingua.lib.frontend.ast.AndSection
import mathlingua.lib.frontend.ast.AsSection
import mathlingua.lib.frontend.ast.Assignment
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
import mathlingua.lib.frontend.ast.ChalkTalkNode
import mathlingua.lib.frontend.ast.Clause
import mathlingua.lib.frontend.ast.ConjectureGroup
import mathlingua.lib.frontend.ast.ConjectureSection
import mathlingua.lib.frontend.ast.ContentSection
import mathlingua.lib.frontend.ast.DEFAULT_AND_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_AUTHOR_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_AXIOM_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_CLAUSE
import mathlingua.lib.frontend.ast.DEFAULT_CONJECTURE_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_DEFINES_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_EQUALITY_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_EXISTS_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_EXISTS_UNIQUE_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_FOR_ALL_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_GENERATED_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_HOMEPAGE_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_ID
import mathlingua.lib.frontend.ast.DEFAULT_IFF_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_IF_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_MATCHING_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_MEMBERSHIP_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_MEMBER_SYMBOLS_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_NAME_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_NEGATIVE_FLOAT_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_NEGATIVE_INT_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_NOTE_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_NOTE_TOP_LEVEL_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_NOT_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_OFFSET_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_OR_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_PIECEWISE_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_POSITIVE_FLOAT_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_POSITIVE_INT_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_REFERENCE_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_RESOURCE_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_RESOURCE_NAME
import mathlingua.lib.frontend.ast.DEFAULT_SPECIFY_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_STATEMENT
import mathlingua.lib.frontend.ast.DEFAULT_STATES_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_SYMBOLS_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_TAG_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_TARGET
import mathlingua.lib.frontend.ast.DEFAULT_TEXT
import mathlingua.lib.frontend.ast.DEFAULT_THEOREM_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_TOPIC_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_TOPIC_NAME
import mathlingua.lib.frontend.ast.DEFAULT_TYPE_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_URL_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_VIEW_GROUP
import mathlingua.lib.frontend.ast.DEFAULT_ZERO_GROUP
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
import mathlingua.lib.frontend.ast.Function
import mathlingua.lib.frontend.ast.FunctionAssignment
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
import mathlingua.lib.frontend.ast.Name
import mathlingua.lib.frontend.ast.NameAssignment
import mathlingua.lib.frontend.ast.NameGroup
import mathlingua.lib.frontend.ast.NameOrFunction
import mathlingua.lib.frontend.ast.NameSection
import mathlingua.lib.frontend.ast.NegativeFloatGroup
import mathlingua.lib.frontend.ast.NegativeFloatSection
import mathlingua.lib.frontend.ast.NegativeIntGroup
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
import mathlingua.lib.frontend.ast.OperatorName
import mathlingua.lib.frontend.ast.OrGroup
import mathlingua.lib.frontend.ast.OrSection
import mathlingua.lib.frontend.ast.PiecewiseElseSection
import mathlingua.lib.frontend.ast.PiecewiseGroup
import mathlingua.lib.frontend.ast.PiecewiseSection
import mathlingua.lib.frontend.ast.PiecewiseThenSection
import mathlingua.lib.frontend.ast.PiecewiseWhenSection
import mathlingua.lib.frontend.ast.PositiveFloatGroup
import mathlingua.lib.frontend.ast.PositiveFloatSection
import mathlingua.lib.frontend.ast.PositiveIntGroup
import mathlingua.lib.frontend.ast.PositiveIntSection
import mathlingua.lib.frontend.ast.ProofSection
import mathlingua.lib.frontend.ast.ProvidedSection
import mathlingua.lib.frontend.ast.ProvidingItem
import mathlingua.lib.frontend.ast.ProvidingSection
import mathlingua.lib.frontend.ast.ReferenceGroup
import mathlingua.lib.frontend.ast.ReferenceSection
import mathlingua.lib.frontend.ast.ResourceGroup
import mathlingua.lib.frontend.ast.ResourceItem
import mathlingua.lib.frontend.ast.ResourceName
import mathlingua.lib.frontend.ast.ResourceSection
import mathlingua.lib.frontend.ast.SatisfyingItem
import mathlingua.lib.frontend.ast.SatisfyingSection
import mathlingua.lib.frontend.ast.Section
import mathlingua.lib.frontend.ast.Sequence
import mathlingua.lib.frontend.ast.Set
import mathlingua.lib.frontend.ast.Spec
import mathlingua.lib.frontend.ast.SpecifyGroup
import mathlingua.lib.frontend.ast.SpecifyItem
import mathlingua.lib.frontend.ast.SpecifySection
import mathlingua.lib.frontend.ast.Statement
import mathlingua.lib.frontend.ast.StatesGroup
import mathlingua.lib.frontend.ast.StatesSection
import mathlingua.lib.frontend.ast.SubAndRegularParamCall
import mathlingua.lib.frontend.ast.SubAndRegularParamSequence
import mathlingua.lib.frontend.ast.SubParamCall
import mathlingua.lib.frontend.ast.SubParamSequence
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
import mathlingua.lib.frontend.ast.TopicName
import mathlingua.lib.frontend.ast.TopicSection
import mathlingua.lib.frontend.ast.Tuple
import mathlingua.lib.frontend.ast.TypeGroup
import mathlingua.lib.frontend.ast.TypeSection
import mathlingua.lib.frontend.ast.UrlGroup
import mathlingua.lib.frontend.ast.UrlSection
import mathlingua.lib.frontend.ast.UsingSection
import mathlingua.lib.frontend.ast.VariadicName
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

internal interface ChalkTalkParser {
    fun parse(): Document
    fun diagnostics(): List<Diagnostic>
}

internal fun newChalkTalkParser(lexer: ChalkTalkNodeLexer): ChalkTalkParser =
    ChalkTalkParserImpl(lexer)

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private class ChalkTalkParserImpl(val lexer: ChalkTalkNodeLexer) : ChalkTalkParser {
    private val diagnostics = mutableListOf<Diagnostic>()

    override fun parse() = document()

    override fun diagnostics() = diagnostics

    private fun document(): Document {
        val items = mutableListOf<TopLevelGroupOrTextBlock>()
        while (lexer.hasNext()) {
            val item = topLevelGroupOrTextBlock() ?: break
            items.add(item)
        }
        while (lexer.hasNext()) {
            maybeAddDiagnosticForMissingItem(lexer.next())
        }
        return Document(items = items)
    }

    private fun topLevelGroupOrTextBlock(): TopLevelGroupOrTextBlock? {
        return textBlock() ?: topLevelGroup()
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

    private fun clauses(): List<Clause> = collect { argument { clause() } }

    private fun andSection(): AndSection? =
        section("and") { AndSection(clauses = oneOrMore(clauses(), it), metadata = it) }

    private fun andGroup(): AndGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs = listOf(SectionSpec(name = "and", required = true) { this.andSection() }),
            default = DEFAULT_AND_GROUP) { _, sections, metadata ->
            AndGroup(andSection = sections["and"] as AndSection, metadata = metadata)
        }

    private fun notSection(): NotSection? =
        section("not") {
            NotSection(clause = required(argument { clause() }, DEFAULT_CLAUSE), metadata = it)
        }

    private fun notGroup(): NotGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs = listOf(SectionSpec(name = "not", required = true) { this.notSection() }),
            default = DEFAULT_NOT_GROUP) { _, sections, metadata ->
            NotGroup(notSection = sections["not"] as NotSection, metadata = metadata)
        }

    private fun orSection(): OrSection? =
        section("or") { OrSection(clauses = oneOrMore(clauses(), it), metadata = it) }

    private fun orGroup(): OrGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs = listOf(SectionSpec(name = "or", required = true) { this.orSection() }),
            default = DEFAULT_OR_GROUP,
        ) { _, sections, metadata ->
            OrGroup(orSection = sections["or"] as OrSection, metadata = metadata)
        }

    private fun operator(): OperatorName? = getNextIfCorrectType()

    private fun tuple(): Tuple? = getNextIfCorrectType()

    private fun set(): Set? = getNextIfCorrectType()

    private fun sequence(): Sequence? = getNextIfCorrectType()

    private fun target(): Target? =
        name()
            ?: operator() ?: tuple() ?: sequence()
                ?: function().let {
                if (it is Function) {
                    it
                } else {
                    if (it != null) {
                        diagnostics.add(
                            Diagnostic(
                                type = DiagnosticType.Error,
                                message = "Expected a target",
                                origin = DiagnosticOrigin.TexTalkParser,
                                row = it.metadata.row,
                                column = it.metadata.column))
                    }
                    null
                }
            }
                ?: set() ?: assignment()

    private fun targets(): List<Target> = collect { argument { target() } }

    private fun existsSection(): ExistsSection? =
        section("exists") { ExistsSection(targets = oneOrMore(targets(), it), metadata = it) }

    private fun specs(): List<Spec> = collect { argument { spec() } }

    private fun whereSection(): WhereSection? =
        section("where") { WhereSection(specs = oneOrMore(specs(), it), metadata = it) }

    private fun suchThatSection(): SuchThatSection? =
        section("suchThat") { SuchThatSection(clauses = oneOrMore(clauses(), it), metadata = it) }

    private fun existsGroup(): ExistsGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(
                    SectionSpec(name = "exists", required = true) { this.existsSection() },
                    SectionSpec(name = "where", required = false) { this.whereSection() },
                    SectionSpec(name = "suchThat", required = false) { this.suchThatSection() }),
            default = DEFAULT_EXISTS_GROUP) { _, sections, metadata ->
            ExistsGroup(
                existsSection = sections["exists"] as ExistsSection,
                whereSection = sections["where"] as WhereSection?,
                suchThatSection = sections["suchThat"] as SuchThatSection?,
                metadata = metadata)
        }

    private fun existsUniqueSection(): ExistsUniqueSection? =
        section("existsUnique") {
            ExistsUniqueSection(targets = oneOrMore(targets(), it), metadata = it)
        }

    private fun existsUniqueGroup(): ExistsUniqueGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(
                    SectionSpec(name = "existsUnique", required = true) {
                        this.existsUniqueSection()
                    },
                    SectionSpec(name = "where", required = false) { this.whereSection() },
                    SectionSpec(name = "suchThat", required = false) { this.suchThatSection() }),
            default = DEFAULT_EXISTS_UNIQUE_GROUP) { _, sections, metadata ->
            ExistsUniqueGroup(
                existsUniqueSection = sections["existsUnique"] as ExistsUniqueSection,
                whereSection = sections["where"] as WhereSection?,
                suchThatSection = sections["suchThat"] as SuchThatSection?,
                metadata = metadata)
        }

    private fun forAllSection(): ForAllSection? =
        section("forAll") { ForAllSection(targets = oneOrMore(targets(), it), metadata = it) }

    private fun forAllGroup(): ForAllGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(
                    SectionSpec(name = "forAll", required = true) { this.forAllSection() },
                    SectionSpec(name = "where", required = false) { this.whereSection() },
                    SectionSpec(name = "suchThat", required = false) { this.suchThatSection() },
                    SectionSpec(name = "then", required = true) { this.thenSection() }),
            default = DEFAULT_FOR_ALL_GROUP) { _, sections, metadata ->
            ForAllGroup(
                forAllSection = sections["forAll"] as ForAllSection,
                whereSection = sections["where"] as WhereSection?,
                suchThatSection = sections["suchThat"] as SuchThatSection?,
                thenSection = sections["then"] as ThenSection,
                metadata = metadata)
        }

    private fun thenSection(): ThenSection? =
        section("then") { ThenSection(clauses = oneOrMore(clauses(), it), metadata = it) }

    private fun ifSection(): IfSection? =
        section("if") { IfSection(clauses = oneOrMore(clauses(), it), metadata = it) }

    private fun ifGroup(): IfGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(
                    SectionSpec(name = "if", required = true) { this.ifSection() },
                    SectionSpec(name = "then", required = true) { this.thenSection() }),
            default = DEFAULT_IF_GROUP) { _, sections, metadata ->
            IfGroup(
                ifSection = sections["if"] as IfSection,
                thenSection = sections["then"] as ThenSection,
                metadata = metadata)
        }

    private fun iffSection(): IffSection? =
        section("iff") { IffSection(clauses = oneOrMore(clauses(), it), metadata = it) }

    private fun iffGroup(): IffGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(
                    SectionSpec(name = "iff", required = true) { this.iffSection() },
                    SectionSpec(name = "then", required = true) { this.thenSection() }),
            default = DEFAULT_IFF_GROUP) { _, sections, metadata ->
            IffGroup(
                iffSection = sections["iff"] as IffSection,
                thenSection = sections["then"] as ThenSection,
                metadata = metadata)
        }

    private fun generatedSection(): GeneratedSection? =
        section("generated") { GeneratedSection(metadata = it) }

    private fun name(): Name? = getNextIfCorrectType()

    private fun function(): Function? = getNextIfCorrectType()

    private fun nameOrFunction(): NameOrFunction? = name() ?: function()

    private fun nameOrFunctions(): List<NameOrFunction> = collect { argument { nameOrFunction() } }

    private fun fromSection(): FromSection? =
        section("from") { FromSection(items = oneOrMore(nameOrFunctions(), it), metadata = it) }

    private fun statements(): List<Statement> = collect { argument { statement() } }

    private fun generatedWhenSection(): GeneratedWhenSection? =
        section("when") {
            GeneratedWhenSection(statements = oneOrMore(statements(), it), metadata = it)
        }

    private fun generatedGroup(): GeneratedGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(
                    SectionSpec(name = "generated", required = true) { this.generatedSection() },
                    SectionSpec(name = "from", required = true) { this.fromSection() },
                    SectionSpec(name = "when", required = true) { this.generatedWhenSection() }),
            default = DEFAULT_GENERATED_GROUP) { _, sections, metadata ->
            GeneratedGroup(
                generatedSection = sections["generated"] as GeneratedSection,
                fromSection = sections["from"] as FromSection,
                whenSection = sections["when"] as GeneratedWhenSection,
                metadata = metadata)
        }

    private fun piecewiseSection(): PiecewiseSection? =
        section("piecewise") { PiecewiseSection(metadata = it) }

    private fun piecewiseWhenSection(): PiecewiseWhenSection? =
        section("when") { PiecewiseWhenSection(clauses = oneOrMore(clauses(), it), metadata = it) }

    private fun piecewiseThenSection(): PiecewiseThenSection? =
        section("then") {
            PiecewiseThenSection(statements = oneOrMore(statements(), it), metadata = it)
        }

    private fun piecewiseElseSection(): PiecewiseElseSection? =
        section("else") {
            PiecewiseElseSection(statements = oneOrMore(statements(), it), metadata = it)
        }

    private fun piecewiseGroup(): PiecewiseGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(
                    SectionSpec(name = "piecewise", required = true) { this.piecewiseSection() },
                    SectionSpec(name = "when", required = false) { this.piecewiseWhenSection() },
                    SectionSpec(name = "then", required = false) { this.piecewiseThenSection() },
                    SectionSpec(name = "else", required = false) { this.piecewiseElseSection() }),
            default = DEFAULT_PIECEWISE_GROUP) { _, sections, metadata ->
            PiecewiseGroup(
                piecewiseSection = sections["piecewise"] as PiecewiseSection,
                whenSection = sections["when"] as PiecewiseWhenSection?,
                thenSection = sections["then"] as PiecewiseThenSection?,
                piecewiseElseSection = sections["else"] as PiecewiseElseSection?,
                metadata = metadata)
        }

    private fun matchingSection(): MatchingSection? =
        section("matching") {
            MatchingSection(statements = oneOrMore(statements(), it), metadata = it)
        }

    private fun matchingGroup(): MatchingGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(SectionSpec(name = "matching", required = true) { this.matchingSection() }),
            default = DEFAULT_MATCHING_GROUP) { _, sections, metadata ->
            MatchingGroup(
                matchingSection = sections["matching"] as MatchingSection, metadata = metadata)
        }

    private fun equalitySection(): EqualitySection? =
        section("equality") { EqualitySection(metadata = it) }

    private fun betweenSection(): BetweenSection? =
        section("between") {
            BetweenSection(
                first = required(argument { target() }, DEFAULT_TARGET),
                second = required(argument { target() }, DEFAULT_TARGET),
                metadata = it)
        }

    private fun providedSection(): ProvidedSection? =
        section("provided") {
            ProvidedSection(
                statement = required(argument { statement() }, DEFAULT_STATEMENT), metadata = it)
        }

    private fun equalityGroup(): EqualityGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(
                    SectionSpec(name = "equality", required = true) { this.equalitySection() },
                    SectionSpec(name = "between", required = true) { this.betweenSection() },
                    SectionSpec(name = "provided", required = true) { this.providedSection() }),
            default = DEFAULT_EQUALITY_GROUP) { _, sections, metadata ->
            EqualityGroup(
                equalitySection = sections["equality"] as EqualitySection,
                betweenSection = sections["between"] as BetweenSection,
                providedSection = sections["provided"] as ProvidedSection,
                metadata = metadata)
        }

    private fun membershipSection(): MembershipSection? =
        section("membership") { MembershipSection(metadata = it) }

    private fun throughSection(): ThroughSection? =
        section("through") {
            ThroughSection(
                through = required(argument { statement() }, DEFAULT_STATEMENT), metadata = it)
        }

    private fun membershipGroup(): MembershipGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(
                    SectionSpec(name = "membership", required = true) { this.membershipSection() },
                    SectionSpec(name = "through", required = true) { this.throughSection() }),
            default = DEFAULT_MEMBERSHIP_GROUP) { _, sections, metadata ->
            MembershipGroup(
                membershipSection = sections["membership"] as MembershipSection,
                throughSection = sections["through"] as ThroughSection,
                metadata = metadata)
        }

    private fun viewSection(): ViewSection? = section("view") { ViewSection(metadata = it) }

    private fun asSection(): AsSection? =
        section("as") {
            AsSection(asText = required(argument { text() }, DEFAULT_TEXT), metadata = it)
        }

    private fun viaSection(): ViaSection? =
        section("via") {
            ViaSection(via = required(argument { statement() }, DEFAULT_STATEMENT), metadata = it)
        }

    private fun bySection(): BySection? =
        section("by") {
            BySection(by = required(argument { statement() }, DEFAULT_STATEMENT), metadata = it)
        }

    private fun viewGroup(): ViewGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(
                    SectionSpec(name = "view", required = true) { this.viewSection() },
                    SectionSpec(name = "as", required = true) { this.asSection() },
                    SectionSpec(name = "via", required = true) { this.viaSection() },
                    SectionSpec(name = "by", required = false) { this.bySection() }),
            default = DEFAULT_VIEW_GROUP) { _, sections, metadata ->
            ViewGroup(
                viewSection = sections["view"] as ViewSection,
                asSection = sections["as"] as AsSection,
                viaSection = sections["via"] as ViaSection,
                bySection = sections["by"] as BySection?,
                metadata = metadata)
        }

    private fun names(): List<Name> = collect { name() }

    private fun symbolsSection(): SymbolsSection? =
        section("symbols") { SymbolsSection(names = oneOrMore(names(), it), metadata = it) }

    private fun symbolsWhereSection(): SymbolsWhereSection? =
        section("where") {
            SymbolsWhereSection(statements = oneOrMore(statements(), it), metadata = it)
        }

    private fun symbolsGroup(): SymbolsGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(
                    SectionSpec(name = "symbols", required = true) { this.symbolsSection() },
                    SectionSpec(name = "where", required = true) { this.symbolsWhereSection() }),
            default = DEFAULT_SYMBOLS_GROUP) { _, sections, metadata ->
            SymbolsGroup(
                symbolsSection = sections["symbols"] as SymbolsSection,
                whereSection = sections["where"] as SymbolsWhereSection,
                metadata = metadata)
        }

    private fun memberSymbolsSection(): MemberSymbolsSection? =
        section("memberSymbols") {
            MemberSymbolsSection(names = oneOrMore(names(), it), metadata = it)
        }

    private fun memberSymbolsWhereSection(): MemberSymbolsWhereSection? =
        section("where") {
            MemberSymbolsWhereSection(statements = oneOrMore(statements(), it), metadata = it)
        }

    private fun memberSymbolsGroup(): MemberSymbolsGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(
                    SectionSpec(name = "memberSymbols", required = true) {
                        this.memberSymbolsSection()
                    },
                    SectionSpec(name = "where", required = true) {
                        this.memberSymbolsWhereSection()
                    }),
            default = DEFAULT_MEMBER_SYMBOLS_GROUP) { _, sections, metadata ->
            MemberSymbolsGroup(
                memberSymbolsSection = sections["memberSymbols"] as MemberSymbolsSection,
                whereSection = sections["where"] as MemberSymbolsWhereSection,
                metadata = metadata)
        }

    private fun texts(): List<Text> = collect { argument { text() } }

    private fun noteSection(): NoteSection? =
        section("note") { NoteSection(items = oneOrMore(texts(), it), metadata = it) }

    private fun noteGroup(): NoteGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs = listOf(SectionSpec(name = "note", required = true) { this.noteSection() }),
            default = DEFAULT_NOTE_GROUP) { _, sections, metadata ->
            NoteGroup(noteSection = sections["note"] as NoteSection, metadata = metadata)
        }

    private fun authorSection(): AuthorSection? =
        section("author") { AuthorSection(items = oneOrMore(texts(), it), metadata = it) }

    private fun authorGroup(): AuthorGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs = listOf(SectionSpec(name = "author", required = true) { this.authorSection() }),
            default = DEFAULT_AUTHOR_GROUP) { _, sections, metadata ->
            AuthorGroup(authorSection = sections["author"] as AuthorSection, metadata = metadata)
        }

    private fun tagSection(): TagSection? =
        section("tag") { TagSection(items = oneOrMore(texts(), it), metadata = it) }

    private fun tagGroup(): TagGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs = listOf(SectionSpec(name = "tag", required = true) { this.tagSection() }),
            default = DEFAULT_TAG_GROUP) { _, sections, metadata ->
            TagGroup(tagSection = sections["tag"] as TagSection, metadata = metadata)
        }

    private fun referenceSection(): ReferenceSection? =
        section("reference") { ReferenceSection(items = oneOrMore(texts(), it), metadata = it) }

    private fun referenceGroup(): ReferenceGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(
                    SectionSpec(name = "reference", required = true) { this.referenceSection() }),
            default = DEFAULT_REFERENCE_GROUP) { _, sections, metadata ->
            ReferenceGroup(
                referenceSection = sections["reference"] as ReferenceSection, metadata = metadata)
        }

    private fun definesSection(): DefinesSection? =
        section("Defines") {
            DefinesSection(target = required(argument { target() }, DEFAULT_TARGET), metadata = it)
        }

    private fun assignment(): Assignment? = getNextIfCorrectType()

    private fun assignments(): List<Assignment> = collect { argument { assignment() } }

    private fun withSection(): WithSection? =
        section("with") { WithSection(assignments = oneOrMore(assignments(), it), metadata = it) }

    private fun givenSection(): GivenSection? =
        section("given") { GivenSection(targets = oneOrMore(targets(), it), metadata = it) }

    private fun whenSection(): WhenSection? =
        section("when") { WhenSection(specs = oneOrMore(specs(), it), metadata = it) }

    private fun meansSection(): MeansSection? =
        section("means") {
            MeansSection(
                statement = required(argument { statement() }, DEFAULT_STATEMENT), metadata = it)
        }

    private fun satisfyingItem(): SatisfyingItem? =
        generatedGroup() ?: clause() ?: spec() ?: statement()

    private fun satisfyingItems(): List<SatisfyingItem> = collect { argument { satisfyingItem() } }

    private fun satisfyingSection(): SatisfyingSection? =
        section("satisfying") {
            SatisfyingSection(items = oneOrMore(satisfyingItems(), it), metadata = it)
        }

    private fun expressingItem(): ExpressingItem? =
        piecewiseGroup() ?: matchingGroup() ?: clause() ?: spec() ?: statement()

    private fun expressingItems(): List<ExpressingItem> = collect { argument { expressingItem() } }

    private fun expressingSection(): ExpressingSection? =
        section("expressing") {
            ExpressingSection(items = oneOrMore(expressingItems(), it), metadata = it)
        }

    private fun usingSection(): UsingSection? =
        section("using") { UsingSection(statements = oneOrMore(statements(), it), metadata = it) }

    private fun writingSection(): WritingSection? =
        section("writing") { WritingSection(items = oneOrMore(texts(), it), metadata = it) }

    private fun writtenSection(): WrittenSection? =
        section("written") { WrittenSection(items = oneOrMore(texts(), it), metadata = it) }

    private fun calledSection(): CalledSection? =
        section("called") { CalledSection(items = oneOrMore(texts(), it), metadata = it) }

    private fun providingItem(): ProvidingItem? =
        viewGroup()
            ?: symbolsGroup() ?: memberSymbolsGroup() ?: equalityGroup() ?: membershipGroup()

    private fun providingItems(): List<ProvidingItem> = collect { argument { providingItem() } }

    private fun providingSection(): ProvidingSection? =
        section("Providing") {
            ProvidingSection(items = oneOrMore(providingItems(), it), metadata = it)
        }

    private fun metadataItem(): MetadataItem? =
        noteGroup() ?: authorGroup() ?: tagGroup() ?: referenceGroup()

    private fun metadataItems(): List<MetadataItem> = collect { argument { metadataItem() } }

    private fun metadataSection(): MetadataSection? =
        section("Metadata") {
            MetadataSection(items = oneOrMore(metadataItems(), it), metadata = it)
        }

    private fun definesGroup(): DefinesGroup? =
        group(
            idSpec = IdRequirement.Required,
            specs =
                listOf(
                    SectionSpec(name = "Defines", required = true) { this.definesSection() },
                    SectionSpec(name = "with", required = false) { this.withSection() },
                    SectionSpec(name = "given", required = false) { this.givenSection() },
                    SectionSpec(name = "when", required = false) { this.whenSection() },
                    SectionSpec(name = "suchThat", required = false) { this.suchThatSection() },
                    SectionSpec(name = "means", required = false) { this.meansSection() },
                    SectionSpec(name = "satisfying", required = false) { this.satisfyingSection() },
                    SectionSpec(name = "expressing", required = false) { this.expressingSection() },
                    SectionSpec(name = "using", required = false) { this.usingSection() },
                    SectionSpec(name = "writing", required = false) { this.writingSection() },
                    SectionSpec(name = "written", required = true) { this.writtenSection() },
                    SectionSpec(name = "called", required = false) { this.calledSection() },
                    SectionSpec(name = "Providing", required = false) { this.providingSection() },
                    SectionSpec(name = "Metadata", required = false) { this.metadataSection() }),
            default = DEFAULT_DEFINES_GROUP) { id, sections, metadata ->
            DefinesGroup(
                id = id ?: DEFAULT_ID,
                definesSection = sections["Defines"] as DefinesSection,
                withSection = sections["with"] as WithSection?,
                givenSection = sections["given"] as GivenSection?,
                whenSection = sections["when"] as WhenSection?,
                suchThatSection = sections["suchThat"] as SuchThatSection?,
                meansSection = sections["means"] as MeansSection?,
                satisfyingSection = sections["satisfying"] as SatisfyingSection?,
                expressingSection = sections["expressing"] as ExpressingSection?,
                usingSection = sections["using"] as UsingSection?,
                writingSection = sections["writing"] as WritingSection?,
                writtenSection = sections["written"] as WrittenSection,
                calledSection = sections["called"] as CalledSection?,
                providingSection = sections["Providing"] as ProvidingSection?,
                metadataSection = sections["Metadata"] as MetadataSection?,
                metadata = metadata)
        }

    private fun statesSection(): StatesSection? = section("States") { StatesSection(metadata = it) }

    private fun statesGroup(): StatesGroup? =
        group(
            idSpec = IdRequirement.Required,
            specs =
                listOf(
                    SectionSpec(name = "States", required = true) { this.statesSection() },
                    SectionSpec(name = "given", required = false) { this.givenSection() },
                    SectionSpec(name = "when", required = false) { this.whenSection() },
                    SectionSpec(name = "suchThat", required = false) { this.suchThatSection() },
                    SectionSpec(name = "that", required = false) { this.thatSection() },
                    SectionSpec(name = "using", required = false) { this.usingSection() },
                    SectionSpec(name = "written", required = true) { this.writtenSection() },
                    SectionSpec(name = "called", required = false) { this.calledSection() },
                    SectionSpec(name = "Metadata", required = false) { this.metadataSection() }),
            default = DEFAULT_STATES_GROUP) { id, sections, metadata ->
            StatesGroup(
                id = id ?: DEFAULT_ID,
                statesSection = sections["States"] as StatesSection,
                givenSection = sections["given"] as GivenSection?,
                whenSection = sections["when"] as WhenSection?,
                suchThatSection = sections["suchThat"] as SuchThatSection?,
                thatSection = sections["that"] as ThatSection,
                usingSection = sections["using"] as UsingSection?,
                writtenSection = sections["written"] as WrittenSection,
                calledSection = sections["called"] as CalledSection?,
                metadataSection = sections["Metadata"] as MetadataSection?,
                metadata = metadata)
        }

    private fun thatItem(): ThatItem? = clause() ?: spec() ?: statement()

    private fun thatItems(): List<ThatItem> = collect { argument { thatItem() } }

    private fun thatSection(): ThatSection? =
        section("that") { ThatSection(items = oneOrMore(thatItems(), it), metadata = it) }

    private fun resourceItem(): ResourceItem? =
        typeGroup()
            ?: nameGroup() ?: authorGroup() ?: homepageGroup() ?: urlGroup() ?: offsetGroup()

    private fun resourceItems(): List<ResourceItem> = collect { argument { resourceItem() } }

    private fun resourceSection(): ResourceSection? =
        section("Resource") {
            ResourceSection(items = oneOrMore(resourceItems(), it), metadata = it)
        }

    private fun typeSection(): TypeSection? =
        section("type") {
            TypeSection(type = required(argument { text() }, DEFAULT_TEXT), metadata = it)
        }

    private fun typeGroup(): TypeGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs = listOf(SectionSpec(name = "type", required = true) { this.typeSection() }),
            default = DEFAULT_TYPE_GROUP) { _, sections, metadata ->
            TypeGroup(typeSection = sections["type"] as TypeSection, metadata = metadata)
        }

    private fun nameSection(): NameSection? =
        section("name") {
            NameSection(text = required(argument { text() }, DEFAULT_TEXT), metadata = it)
        }

    private fun nameGroup(): NameGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs = listOf(SectionSpec(name = "name", required = true) { this.nameSection() }),
            default = DEFAULT_NAME_GROUP) { _, sections, metadata ->
            NameGroup(nameSection = sections["name"] as NameSection, metadata = metadata)
        }

    private fun homepageSection(): HomepageSection? =
        section("homepage") {
            HomepageSection(homepage = required(argument { text() }, DEFAULT_TEXT), metadata = it)
        }

    private fun homepageGroup(): HomepageGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(SectionSpec(name = "homepage", required = true) { this.homepageSection() }),
            default = DEFAULT_HOMEPAGE_GROUP) { _, sections, metadata ->
            HomepageGroup(
                homepageSection = sections["homepage"] as HomepageSection, metadata = metadata)
        }

    private fun urlSection(): UrlSection? =
        section("url") {
            UrlSection(url = required(argument { text() }, DEFAULT_TEXT), metadata = it)
        }

    private fun urlGroup(): UrlGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs = listOf(SectionSpec(name = "url", required = true) { this.urlSection() }),
            default = DEFAULT_URL_GROUP) { _, sections, metadata ->
            UrlGroup(urlSection = sections["url"] as UrlSection, metadata = metadata)
        }

    private fun offsetSection(): OffsetSection? =
        section("offset") {
            OffsetSection(offset = required(argument { text() }, DEFAULT_TEXT), metadata = it)
        }

    private fun offsetGroup(): OffsetGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs = listOf(SectionSpec(name = "offset", required = true) { this.offsetSection() }),
            default = DEFAULT_OFFSET_GROUP) { _, sections, metadata ->
            OffsetGroup(offsetSection = sections["offset"] as OffsetSection, metadata = metadata)
        }

    private fun resourceGroup(): ResourceGroup? =
        group(
            idSpec = IdRequirement.Required,
            specs =
                listOf(SectionSpec(name = "Resource", required = true) { this.resourceSection() }),
            default = DEFAULT_RESOURCE_GROUP) { id, sections, metadata ->
            ResourceGroup(
                id =
                    if (id != null) {
                        ResourceName(name = id.text, metadata = id.metadata)
                    } else {
                        diagnostics.add(
                            Diagnostic(
                                type = DiagnosticType.Error,
                                origin = DiagnosticOrigin.ChalkTalkParser,
                                message = "Expected a resource name",
                                row = sections["Resource"]?.metadata?.row ?: -1,
                                column = sections["Resource"]?.metadata?.column ?: -1))
                        DEFAULT_RESOURCE_NAME
                    },
                resourceSection = sections["Resource"] as ResourceSection,
                metadata = metadata)
        }

    private fun axiomSection(): AxiomSection? =
        section("Axiom") { AxiomSection(names = texts(), metadata = it) }

    private fun axiomGroup(): AxiomGroup? =
        group(
            idSpec = IdRequirement.Optional,
            specs =
                listOf(
                    SectionSpec(name = "Axiom", required = true) { this.axiomSection() },
                    SectionSpec(name = "given", required = false) { this.givenSection() },
                    SectionSpec(name = "where", required = false) { this.whereSection() },
                    SectionSpec(name = "suchThat", required = false) { this.suchThatSection() },
                    SectionSpec(name = "then", required = true) { this.thenSection() },
                    SectionSpec(name = "iff", required = false) { this.iffSection() },
                    SectionSpec(name = "using", required = false) { this.usingSection() },
                    SectionSpec(name = "Metadata", required = false) { this.metadataSection() }),
            default = DEFAULT_AXIOM_GROUP) { id, sections, metadata ->
            AxiomGroup(
                id = id,
                axiomSection = sections["Axiom"] as AxiomSection,
                givenSection = sections["given"] as GivenSection?,
                whereSection = sections["where"] as WhereSection?,
                suchThatSection = sections["suchThat"] as SuchThatSection?,
                thenSection = sections["then"] as ThenSection,
                iffSection = sections["iff"] as IffSection?,
                usingSection = sections["using"] as UsingSection?,
                metadataSection = sections["Metadata"] as MetadataSection?,
                metadata = metadata)
        }

    private fun conjectureSection(): ConjectureSection? =
        section("Conjecture") { ConjectureSection(names = texts(), metadata = it) }

    private fun conjectureGroup(): ConjectureGroup? =
        group(
            idSpec = IdRequirement.Optional,
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
        section("Theorem") { TheoremSection(names = texts(), metadata = it) }

    private fun theoremGroup(): TheoremGroup? =
        group(
            idSpec = IdRequirement.Optional,
            specs =
                listOf(
                    SectionSpec(name = "Theorem", required = true) { this.theoremSection() },
                    SectionSpec(name = "given", required = false) { this.givenSection() },
                    SectionSpec(name = "where", required = false) { this.whereSection() },
                    SectionSpec(name = "suchThat", required = false) { this.suchThatSection() },
                    SectionSpec(name = "then", required = true) { this.thenSection() },
                    SectionSpec(name = "iff", required = false) { this.iffSection() },
                    SectionSpec(name = "using", required = false) { this.usingSection() },
                    SectionSpec(name = "Proof", required = false) { this.proofSection() },
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
                proofSection = sections["Proof"] as ProofSection?,
                metadataSection = sections["Metadata"] as MetadataSection?,
                metadata = metadata)
        }

    private fun proofSection(): ProofSection? =
        section("Proof") { ProofSection(proofs = oneOrMore(texts(), it), metadata = it) }

    private fun topicSection(): TopicSection? =
        section("Topic") { TopicSection(names = texts(), metadata = it) }

    private fun contentSection(): ContentSection? =
        section("content") {
            ContentSection(content = required(argument { text() }, DEFAULT_TEXT), metadata = it)
        }

    private fun topicGroup(): TopicGroup? =
        group(
            idSpec = IdRequirement.Required,
            specs =
                listOf(
                    SectionSpec(name = "Topic", required = true) { this.topicSection() },
                    SectionSpec(name = "content", required = true) { this.contentSection() },
                    SectionSpec(name = "Metadata", required = false) { this.metadataSection() }),
            default = DEFAULT_TOPIC_GROUP) { id, sections, metadata ->
            TopicGroup(
                id =
                    if (id != null) {
                        TopicName(name = id.text, metadata = id.metadata)
                    } else {
                        diagnostics.add(
                            Diagnostic(
                                type = DiagnosticType.Error,
                                origin = DiagnosticOrigin.ChalkTalkParser,
                                message = "Expected a topic name",
                                row = sections["Topic"]?.metadata?.row ?: -1,
                                column = sections["Topic"]?.metadata?.column ?: -1))
                        DEFAULT_TOPIC_NAME
                    },
                topicSection = sections["Topic"] as TopicSection,
                contentSection = sections["content"] as ContentSection,
                metadataSection = sections["Metadata"] as MetadataSection?,
                metadata = metadata)
        }

    private fun noteTopLevelSection(): NoteTopLevelSection? =
        section("Note") { NoteTopLevelSection(metadata = it) }

    private fun noteTopLevelGroup(): NoteTopLevelGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(
                    SectionSpec(name = "Note", required = true) { this.noteTopLevelSection() },
                    SectionSpec(name = "content", required = true) { this.contentSection() },
                    SectionSpec(name = "Metadata", required = false) { this.metadataSection() }),
            default = DEFAULT_NOTE_TOP_LEVEL_GROUP) { _, sections, metadata ->
            NoteTopLevelGroup(
                noteSection = sections["Note"] as NoteTopLevelSection,
                contentSection = sections["content"] as ContentSection,
                metadataSection = sections["Metadata"] as MetadataSection?,
                metadata = metadata)
        }

    private fun specifyItem(): SpecifyItem? =
        zeroGroup()
            ?: positiveIntGroup() ?: negativeIntGroup() ?: positiveFloatGroup()
                ?: negativeFloatGroup()

    private fun specifyItems(): List<SpecifyItem> = collect { argument { specifyItem() } }

    private fun specifySection(): SpecifySection? =
        section("Specify") { SpecifySection(items = oneOrMore(specifyItems(), it), metadata = it) }

    private fun specifyGroup(): SpecifyGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(SectionSpec(name = "Specify", required = true) { this.specifySection() }),
            default = DEFAULT_SPECIFY_GROUP) { _, sections, metadata ->
            SpecifyGroup(
                specifySection = sections["Specify"] as SpecifySection, metadata = metadata)
        }

    private fun zeroSection(): ZeroSection? = section("zero") { ZeroSection(metadata = it) }

    private fun zeroGroup(): ZeroGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(
                    SectionSpec(name = "zero", required = true) { this.zeroSection() },
                    SectionSpec(name = "is", required = true) { this.isSection() }),
            default = DEFAULT_ZERO_GROUP) { _, sections, metadata ->
            ZeroGroup(
                zeroSection = sections["zero"] as ZeroSection,
                isSection = sections["is"] as IsSection,
                metadata = metadata)
        }

    private fun positiveIntSection(): PositiveIntSection? =
        section("positiveInt") { PositiveIntSection(metadata = it) }

    private fun positiveIntGroup(): PositiveIntGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(
                    SectionSpec(name = "positiveInt", required = true) {
                        this.positiveIntSection()
                    },
                    SectionSpec(name = "is", required = true) { this.isSection() }),
            default = DEFAULT_POSITIVE_INT_GROUP) { _, sections, metadata ->
            PositiveIntGroup(
                positiveIntSection = sections["positiveInt"] as PositiveIntSection,
                isSection = sections["is"] as IsSection,
                metadata = metadata)
        }

    private fun negativeIntSection(): NegativeIntSection? =
        section("negativeInt") { NegativeIntSection(metadata = it) }

    private fun negativeIntGroup(): NegativeIntGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(
                    SectionSpec(name = "negativeInt", required = true) {
                        this.negativeIntSection()
                    },
                    SectionSpec(name = "is", required = true) { this.isSection() }),
            default = DEFAULT_NEGATIVE_INT_GROUP) { _, sections, metadata ->
            NegativeIntGroup(
                negativeIntSection = sections["negativeInt"] as NegativeIntSection,
                isSection = sections["is"] as IsSection,
                metadata = metadata)
        }

    private fun positiveFloatSection(): PositiveFloatSection? =
        section("positiveFloat") { PositiveFloatSection(metadata = it) }

    private fun positiveFloatGroup(): PositiveFloatGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(
                    SectionSpec(name = "positiveFloat", required = true) {
                        this.positiveFloatSection()
                    },
                    SectionSpec(name = "is", required = true) { this.isSection() }),
            default = DEFAULT_POSITIVE_FLOAT_GROUP) { _, sections, metadata ->
            PositiveFloatGroup(
                positiveFloatSection = sections["positiveFloat"] as PositiveFloatSection,
                isSection = sections["is"] as IsSection,
                metadata = metadata)
        }

    private fun negativeFloatSection(): NegativeFloatSection? =
        section("negativeFloat") { NegativeFloatSection(metadata = it) }

    private fun negativeFloatGroup(): NegativeFloatGroup? =
        group(
            idSpec = IdRequirement.NotAllowed,
            specs =
                listOf(
                    SectionSpec(name = "negativeFloat", required = true) {
                        this.negativeFloatSection()
                    },
                    SectionSpec(name = "is", required = true) { this.isSection() }),
            default = DEFAULT_NEGATIVE_FLOAT_GROUP) { _, sections, metadata ->
            NegativeFloatGroup(
                negativeFloatSection = sections["negativeFloat"] as NegativeFloatSection,
                isSection = sections["is"] as IsSection,
                metadata = metadata)
        }

    private fun isSection(): IsSection? =
        section("is") {
            IsSection(form = required(argument { text() }, DEFAULT_TEXT), metadata = it)
        }

    private fun <T> collect(fn: () -> T?): List<T> {
        val result = mutableListOf<T>()
        while (lexer.hasNext()) {
            val item = fn() ?: break
            result.add(item)
        }
        return result
    }

    private inline fun <reified T> getNextIfCorrectType(): T? =
        if (nextIs<T>()) {
            lexer.next() as T
        } else {
            null
        }

    private inline fun <reified T> nextIs() = lexer.hasNext() && lexer.peek() is T

    private fun <T : ChalkTalkNode> required(value: T?, default: T): T {
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
                    origin = DiagnosticOrigin.ChalkTalkParser,
                    message = "Expected a '${default.javaClass.simpleName}'",
                    row = peek?.metadata?.row ?: -1,
                    column = peek?.metadata?.column ?: -1))
        }
        return value ?: default
    }

    private fun <T> oneOrMore(value: List<T>, metadata: MetaData): List<T> {
        if (value.isEmpty()) {
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    origin = DiagnosticOrigin.ChalkTalkParser,
                    message = "Expected at least one argument",
                    row = metadata.row,
                    column = metadata.column))
        }
        return value
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
        while (lexer.hasNext() && !nextIs<EndSection>()) {
            maybeAddDiagnosticForMissingItem(lexer.next())
        }
        expectIs<EndSection>()
        return result
    }

    private fun <T> argument(fn: (metadata: MetaData) -> T): T? {
        if (!nextIs<BeginArgument>()) {
            return null
        }

        val begin = expectIs<BeginArgument>()
        val result = fn(begin?.metadata ?: MetaData(row = -1, column = -1, isInline = false))
        expectIs<EndArgument>()

        return result
    }

    private inline fun <reified T : NodeLexerToken> expectIs(): NodeLexerToken? {
        if (!lexer.hasNext() || lexer.peek() !is T) {
            val peek =
                if (lexer.hasNext()) {
                    lexer.peek()
                } else {
                    null
                }
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    origin = DiagnosticOrigin.ChalkTalkParser,
                    message = "Found a token of the wrong type",
                    row = peek?.metadata?.row ?: -1,
                    column = peek?.metadata?.column ?: -1))
            return null
        }
        return lexer.next()
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
                        origin = DiagnosticOrigin.ChalkTalkParser,
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
                    origin = DiagnosticOrigin.ChalkTalkParser,
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
                    origin = DiagnosticOrigin.ChalkTalkParser,
                    message = "For pattern:\n\n$pattern\nExpected a $nextExpected",
                    row = startRow,
                    column = startColumn))
            return null
        }

        return result
    }

    private fun <T> group(
        idSpec: IdRequirement,
        specs: List<SectionSpec>,
        default: T,
        builder: (id: Id?, sections: Map<String, Section?>, metadata: MetaData) -> T
    ): T? {
        if (!nextIs<BeginGroup>() || (lexer.peek() as BeginGroup).name != specs.first().name) {
            return null
        }

        val namesToSpec = mutableMapOf<String, SectionSpec>()
        for (spec in specs) {
            namesToSpec[spec.name] = spec
        }

        val beginGroup = expectIs<BeginGroup>()!!

        val id = id()
        if (id == null) {
            if (idSpec == IdRequirement.Required) {
                diagnostics.add(
                    Diagnostic(
                        type = DiagnosticType.Error,
                        origin = DiagnosticOrigin.ChalkTalkParser,
                        message = "Expected an id",
                        row = beginGroup.metadata.row,
                        column = beginGroup.metadata.column))
            }
        } else {
            // report an error if the id is specified but is not allowed
            if (idSpec == IdRequirement.NotAllowed) {
                diagnostics.add(
                    Diagnostic(
                        type = DiagnosticType.Error,
                        origin = DiagnosticOrigin.ChalkTalkParser,
                        message = "An id cannot be specified here",
                        row = beginGroup.metadata.row,
                        column = beginGroup.metadata.column))
            }
        }

        val sections = mutableListOf<Section>()
        while (lexer.hasNext()) {
            val nextIsSection = lexer.hasNext() && lexer.peek() is BeginSection
            if (!nextIsSection) {
                break
            }
            val sect = lexer.peek() as BeginSection
            if (namesToSpec.containsKey(sect.name)) {
                val spec = namesToSpec[sect.name]!!
                val result = spec.builder()
                if (result != null) {
                    sections.add(result)
                }
            } else {
                val beginSection = expectIs<BeginSection>()!!
                var count = 1
                diagnostics.add(
                    Diagnostic(
                        type = DiagnosticType.Error,
                        origin = DiagnosticOrigin.ChalkTalkParser,
                        message = "Unexpected section '${sect.name}'",
                        row = beginSection.metadata.row,
                        column = beginGroup.metadata.column))
                while (lexer.hasNext() && count > 0) {
                    val peek = lexer.peek()
                    if (peek is BeginSection) {
                        count++
                    } else if (peek is EndSection) {
                        count--
                    }

                    if (count > 0) {
                        lexer.next()
                    }
                }
                expectIs<EndSection>()
            }
        }

        while (lexer.hasNext() && !nextIs<EndGroup>()) {
            maybeAddDiagnosticForMissingItem(lexer.next())
        }
        expectIs<EndGroup>()

        val mapping =
            identifySections(
                sections, specs.map { "${it.name}${if (it.required) { "" } else { "?" }}" })

        return if (mapping == null) {
            default
        } else {
            builder(
                id,
                mapping,
                MetaData(
                    row = beginGroup.metadata.row,
                    column = beginGroup.metadata.column,
                    isInline = false))
        }
    }

    private fun getNodeName(item: NodeLexerToken) =
        when (item) {
            is BeginGroup -> {
                // don't report about this to the user since it is an implementation detail
                null
            }
            is EndGroup -> {
                // don't report about this to the user since it is an implementation detail
                null
            }
            is BeginSection -> {
                // don't report about this to the user since it is an implementation detail
                null
            }
            is EndSection -> {
                // don't report about this to the user since it is an implementation detail
                null
            }
            is BeginArgument -> {
                // don't report about this to the user since it is an implementation detail
                null
            }
            is EndArgument -> {
                // don't report about this to the user since it is an implementation detail
                null
            }
            is Statement -> "statement"
            is FunctionAssignment -> "function assignment"
            is NameAssignment -> "name assignment"
            is Function -> "function"
            is SubAndRegularParamCall -> "function"
            is SubParamCall -> "function"
            is Name -> "name"
            is VariadicName -> "name"
            is OperatorName -> "name"
            is SubAndRegularParamSequence -> "sequence"
            is SubParamSequence -> "sequence"
            is Set -> "set"
            is Tuple -> "tuple"
            is Text -> "text"
            is NameOrFunction -> "name or function"
            is Id -> "id"
            is TextBlock -> "text block"
        }

    private fun maybeAddDiagnosticForMissingItem(token: NodeLexerToken) {
        val text = getNodeName(token)
        if (text != null) {
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    origin = DiagnosticOrigin.ChalkTalkParser,
                    message = "Unexpected $text item",
                    row = token.metadata.row,
                    column = token.metadata.column))
        }
    }
}

private enum class IdRequirement {
    NotAllowed, // it cannot be specified
    Optional, // it can be specified but is not required
    Required // it must be specified
}

private data class SectionSpec(
    val name: String, val required: Boolean, val builder: () -> Section?)

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

package mathlingua.lib.frontend.ast

import mathlingua.lib.frontend.HasMetaData
import mathlingua.lib.frontend.MetaData

internal sealed interface ToCode {
    fun toCode(): String
}

internal sealed interface ChalkTalkNode : HasMetaData, ToCode

internal data class TextBlock(val text: String, override val metadata: MetaData) :
    ChalkTalkNode, TopLevelGroupOrTextBlock, NodeLexerToken {
    override fun toCode() = "::$text::"
}

internal data class Id(val text: String, override val metadata: MetaData) :
    ChalkTalkNode, NodeLexerToken {
    override fun toCode() = text
}

internal sealed interface IdForm : ChalkTalkNode

internal data class Formulation(val text: String, override val metadata: MetaData) :
    Argument, Spec, ProvidedItem, Clause {
    override fun toCode() =
        if (this.text.contains("'")) {
            "`${this.text}`"
        } else {
            "'${this.text}'"
        }
}

internal data class Text(val text: String, override val metadata: MetaData) : Argument, Clause {
    override fun toCode() = "\"$text\""
}

/*
 * <clause> ::= and: |
 *              not: |
 *              or: |
 *              exists: |
 *              existsUnique: |
 *              forAll: |
 *              if: |
 *              iff: |
 *              <text>[.*] |
 *              <statement>[<value textalk exp>]
 */
internal interface Clause : ThatItem, SatisfyingItem, ExpressingItem

/*
 * <spec> ::= <statement>[<is textalk exp>] |
 *            <statement>[<in textalk exp>]
 */
internal interface Spec : Clause, SatisfyingItem, ExpressingItem, ThatItem

internal open class Section(val name: String, override val metadata: MetaData) : ChalkTalkNode {
    override fun toCode(): String {
        throw Exception("Not implemented")
    }
}

internal open class Group(override val metadata: MetaData) : ChalkTalkNode {
    override fun toCode(): String {
        throw Exception("Not implemented")
    }
}

// internal data class Group(, override val metadata: MetaData) : Group(metadata)

internal data class AndSection(val clauses: List<Clause>, override val metadata: MetaData) :
    Section("and", metadata) {
    override fun toCode() = sectionToCode(this, *clauses.toTypedArray())
}

internal data class AndGroup(val andSection: AndSection, override val metadata: MetaData) :
    Group(metadata), Clause {
    override fun toCode() = groupToCode(null, andSection)
}

internal data class NotSection(val clause: Clause, override val metadata: MetaData) :
    Section("not", metadata) {
    override fun toCode() = sectionToCode(this, clause)
}

internal data class NotGroup(val notSection: NotSection, override val metadata: MetaData) :
    Group(metadata), Clause {
    override fun toCode() = groupToCode(null, notSection)
}

internal data class OrSection(val clauses: List<Clause>, override val metadata: MetaData) :
    Section("or", metadata) {
    override fun toCode() = sectionToCode(this, *clauses.toTypedArray())
}

internal data class OrGroup(val orSection: OrSection, override val metadata: MetaData) :
    Group(metadata), Clause {
    override fun toCode() = groupToCode(null, orSection)
}

internal data class ExistsSection(val targets: List<Target>, override val metadata: MetaData) :
    Section("exists", metadata) {
    override fun toCode() = sectionToCode(this, *targets.toTypedArray())
}

internal data class WhereSection(val specs: List<Spec>, override val metadata: MetaData) :
    Section("where", metadata) {
    override fun toCode() = sectionToCode(this, *specs.toTypedArray())
}

internal data class SuchThatSection(val clauses: List<Clause>, override val metadata: MetaData) :
    Section("suchThat", metadata) {
    override fun toCode() = sectionToCode(this, *clauses.toTypedArray())
}

internal data class ExistsGroup(
    val existsSection: ExistsSection,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection?,
    override val metadata: MetaData
) : Group(metadata), Clause {
    override fun toCode() = groupToCode(null, existsSection, whereSection, suchThatSection)
}

internal data class ExistsUniqueSection(
    val targets: List<Target>, override val metadata: MetaData
) : Section("existsUnique", metadata) {
    override fun toCode() = sectionToCode(this, *targets.toTypedArray())
}

internal data class ExistsUniqueGroup(
    val existsUniqueSection: ExistsUniqueSection,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection?,
    override val metadata: MetaData
) : Group(metadata), Clause {
    override fun toCode() = groupToCode(null, existsUniqueSection, whereSection, suchThatSection)
}

internal data class ForAllSection(val targets: List<Target>, override val metadata: MetaData) :
    Section("forAll", metadata) {
    override fun toCode() = sectionToCode(this, *targets.toTypedArray())
}

internal data class ForAllGroup(
    val forAllSection: ForAllSection,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection?,
    val thenSection: ThenSection,
    override val metadata: MetaData
) : Group(metadata), Clause {
    override fun toCode() =
        groupToCode(null, forAllSection, whereSection, suchThatSection, thenSection)
}

internal data class ThenSection(val clauses: List<Clause>, override val metadata: MetaData) :
    Section("then", metadata) {
    override fun toCode() = sectionToCode(this, *clauses.toTypedArray())
}

internal data class IfSection(val clauses: List<Clause>, override val metadata: MetaData) :
    Section("if", metadata) {
    override fun toCode() = sectionToCode(this, *clauses.toTypedArray())
}

internal data class IfGroup(
    val ifSection: IfSection, val thenSection: ThenSection, override val metadata: MetaData
) : Group(metadata), Clause {
    override fun toCode() = groupToCode(null, ifSection, thenSection)
}

internal data class IffSection(val clauses: List<Clause>, override val metadata: MetaData) :
    Section("iff", metadata) {
    override fun toCode() = sectionToCode(this, *clauses.toTypedArray())
}

internal data class IffGroup(
    val iffSection: IffSection, val thenSection: ThenSection, override val metadata: MetaData
) : Group(metadata), Clause {
    override fun toCode() = groupToCode(null, iffSection, thenSection)
}

internal data class GeneratedSection(override val metadata: MetaData) :
    Section("generated", metadata) {
    override fun toCode() = sectionToCode(this)
}

internal data class FromSection(val items: List<NameOrFunction>, override val metadata: MetaData) :
    Section("from", metadata) {
    override fun toCode() = sectionToCode(this, *items.toTypedArray())
}

internal data class GeneratedWhenSection(
    val formulations: List<Formulation>, override val metadata: MetaData
) : Section("when", metadata) {
    override fun toCode() = sectionToCode(this, *formulations.toTypedArray())
}

internal data class GeneratedGroup(
    val generatedSection: GeneratedSection,
    val fromSection: FromSection,
    val generatedWhenSection: GeneratedWhenSection,
    override val metadata: MetaData
) : Group(metadata), SatisfyingItem {
    override fun toCode() = groupToCode(null, generatedSection, fromSection, generatedWhenSection)
}

internal data class PiecewiseSection(override val metadata: MetaData) :
    Section("piecewise", metadata) {
    override fun toCode() = sectionToCode(this)
}

internal data class PiecewiseWhenSection(
    val clauses: List<Clause>, override val metadata: MetaData
) : Section("when", metadata) {
    override fun toCode() = sectionToCode(this, *clauses.toTypedArray())
}

internal data class PiecewiseThenSection(
    val formulations: List<Formulation>, override val metadata: MetaData
) : Section("then", metadata) {
    override fun toCode() = sectionToCode(this, *formulations.toTypedArray())
}

internal data class PiecewiseElseSection(
    val formulations: List<Formulation>, override val metadata: MetaData
) : Section("else", metadata) {
    override fun toCode() = sectionToCode(this, *formulations.toTypedArray())
}

internal data class PiecewiseGroup(
    val piecewiseSection: PiecewiseSection,
    val piecewiseWhenSection: PiecewiseWhenSection?,
    val piecewiseThenSection: PiecewiseThenSection?,
    val piecewiseElseSection: PiecewiseElseSection?,
    override val metadata: MetaData
) : Group(metadata), ExpressingItem {
    override fun toCode() =
        groupToCode(
            null,
            piecewiseSection,
            piecewiseWhenSection,
            piecewiseThenSection,
            piecewiseElseSection)
}

internal data class MatchingSection(
    val formulations: List<Formulation>, override val metadata: MetaData
) : Section("matching", metadata) {
    override fun toCode() = sectionToCode(this, *formulations.toTypedArray())
}

internal data class MatchingGroup(
    val matchingSection: MatchingSection, override val metadata: MetaData
) : Group(metadata), ExpressingItem {
    override fun toCode() = groupToCode(null, matchingSection)
}

internal data class EqualitySection(override val metadata: MetaData) :
    Section("equality", metadata) {
    override fun toCode() = sectionToCode(this)
}

internal data class BetweenSection(
    val first: Target, val second: Target, override val metadata: MetaData
) : Section("between", metadata) {
    override fun toCode() = sectionToCode(this, first, second)
}

internal interface ProvidedItem : ChalkTalkNode

internal data class ProvidedSection(val statement: ProvidedItem, override val metadata: MetaData) :
    Section("provided", metadata) {
    override fun toCode() = sectionToCode(this, statement)
}

internal data class EqualityGroup(
    val equalitySection: EqualitySection,
    val betweenSection: BetweenSection,
    val providedSection: ProvidedSection,
    override val metadata: MetaData
) : Group(metadata), ProvidingItem {
    override fun toCode() = groupToCode(null, equalitySection, betweenSection, providedSection)
}

internal data class MembershipSection(override val metadata: MetaData) :
    Section("membership", metadata) {
    override fun toCode() = sectionToCode(this)
}

internal data class ThroughSection(val through: Formulation, override val metadata: MetaData) :
    Section("through", metadata) {
    override fun toCode() = sectionToCode(this, through)
}

internal data class MembershipGroup(
    val membershipSection: MembershipSection,
    val throughSection: ThroughSection,
    override val metadata: MetaData
) : Group(metadata), ProvidingItem {
    override fun toCode() = groupToCode(null, membershipSection, throughSection)
}

internal data class ViewSection(override val metadata: MetaData) : Section("view", metadata) {
    override fun toCode() = sectionToCode(this)
}

internal data class AsSection(val asText: Text, override val metadata: MetaData) :
    Section("as", metadata) {
    override fun toCode() = sectionToCode(this, asText)
}

internal data class ViaSection(val via: Formulation, override val metadata: MetaData) :
    Section("via", metadata) {
    override fun toCode() = sectionToCode(this, via)
}

internal data class BySection(val by: Formulation, override val metadata: MetaData) :
    Section("by", metadata) {
    override fun toCode() = sectionToCode(this, by)
}

internal data class ViewGroup(
    val viewSection: ViewSection,
    val asSection: AsSection,
    val viaSection: ViaSection,
    val bySection: BySection?,
    override val metadata: MetaData
) : Group(metadata), ProvidingItem {
    override fun toCode() = groupToCode(null, viewSection, asSection, viaSection, bySection)
}

internal data class SymbolsSection(val names: List<Name>, override val metadata: MetaData) :
    Section("symbols", metadata) {
    override fun toCode() = sectionToCode(this, *names.toTypedArray())
}

internal data class SymbolsWhereSection(
    val formulations: List<Formulation>, override val metadata: MetaData
) : Section("where", metadata) {
    override fun toCode() = sectionToCode(this, *formulations.toTypedArray())
}

internal data class SymbolsGroup(
    val symbolsSection: SymbolsSection,
    val symbolsWhereSection: SymbolsWhereSection,
    override val metadata: MetaData
) : Group(metadata), ProvidingItem {
    override fun toCode() = groupToCode(null, symbolsSection, symbolsWhereSection)
}

internal data class SymbolsAsGroup(
    val symbolsSection: SymbolsSection,
    val symbolsAsSection: SymbolsAsSection,
    override val metadata: MetaData
) : Group(metadata), ProvidingItem {
    override fun toCode() = groupToCode(null, symbolsSection, symbolsAsSection)
}

internal data class SymbolsAsSection(val asSection: Text, override val metadata: MetaData) :
    Section("as", metadata) {
    override fun toCode() = sectionToCode(this, asSection)
}

internal data class MemberSymbolsSection(val names: List<Name>, override val metadata: MetaData) :
    Section("memberSymbols", metadata) {
    override fun toCode() = sectionToCode(this, *names.toTypedArray())
}

internal data class MemberSymbolsWhereSection(
    val formulations: List<Formulation>, override val metadata: MetaData
) : Section("where", metadata) {
    override fun toCode() = sectionToCode(this, *formulations.toTypedArray())
}

internal data class MemberSymbolsAsSection(val asSection: Text, override val metadata: MetaData) :
    Section("as", metadata) {
    override fun toCode() = sectionToCode(this, asSection)
}

internal data class MemberSymbolsGroup(
    val memberSymbolsSection: MemberSymbolsSection,
    val memberSymbolsWhereSection: MemberSymbolsWhereSection,
    override val metadata: MetaData
) : Group(metadata), ProvidingItem {
    override fun toCode() = groupToCode(null, memberSymbolsSection, memberSymbolsWhereSection)
}

internal data class MemberSymbolsAsGroup(
    val memberSymbolsSection: MemberSymbolsSection,
    val memberSymbolsAsSection: MemberSymbolsAsSection,
    override val metadata: MetaData
) : Group(metadata), ProvidingItem {
    override fun toCode() = groupToCode(null, memberSymbolsSection, memberSymbolsAsSection)
}

internal data class ContributorSection(val items: List<Text>, override val metadata: MetaData) :
    Section("author", metadata) {
    override fun toCode() = sectionToCode(this, *items.toTypedArray())
}

internal data class ContributorGroup(
    val contributorSection: ContributorSection, override val metadata: MetaData
) : Group(metadata), MetadataItem {
    override fun toCode() = groupToCode(null, contributorSection)
}

internal data class AuthorSection(val items: List<Text>, override val metadata: MetaData) :
    Section("author", metadata) {
    override fun toCode() = sectionToCode(this, *items.toTypedArray())
}

internal data class AuthorGroup(val authorSection: AuthorSection, override val metadata: MetaData) :
    Group(metadata), ResourceItem {
    override fun toCode() = groupToCode(null, authorSection)
}

internal data class TagSection(val items: List<Text>, override val metadata: MetaData) :
    Section("tag", metadata) {
    override fun toCode() = sectionToCode(this, *items.toTypedArray())
}

internal data class TagGroup(val tagSection: TagSection, override val metadata: MetaData) :
    Group(metadata), MetadataItem {
    override fun toCode() = groupToCode(null, tagSection)
}

internal data class ReferencesSection(val items: List<Text>, override val metadata: MetaData) :
    Section("References", metadata) {
    override fun toCode() = sectionToCode(this, *items.toTypedArray())
}

internal data class DefinesSection(val target: Target, override val metadata: MetaData) :
    Section("Defines", metadata) {
    override fun toCode() = sectionToCode(this, target)
}

internal data class WithSection(
    val assignments: List<NameAssignment>, override val metadata: MetaData
) : Section("with", metadata) {
    override fun toCode() = sectionToCode(this, *assignments.toTypedArray())
}

internal data class GivenSection(val targets: List<Target>, override val metadata: MetaData) :
    Section("given", metadata) {
    override fun toCode() = sectionToCode(this, *targets.toTypedArray())
}

internal data class WhenSection(val specs: List<Spec>, override val metadata: MetaData) :
    Section("when", metadata) {
    override fun toCode() = sectionToCode(this, *specs.toTypedArray())
}

internal data class MeansSection(val statement: Formulation, override val metadata: MetaData) :
    Section("means", metadata) {
    override fun toCode() = sectionToCode(this, statement)
}

internal interface SatisfyingItem : ChalkTalkNode

internal data class SatisfyingSection(
    val items: List<SatisfyingItem>, override val metadata: MetaData
) : Section("satisfying", metadata) {
    override fun toCode() = sectionToCode(this, *items.toTypedArray())
}

internal interface ExpressingItem : ChalkTalkNode

internal data class ExpressingSection(
    val items: List<ExpressingItem>, override val metadata: MetaData
) : Section("expressing", metadata) {
    override fun toCode() = sectionToCode(this, *items.toTypedArray())
}

internal data class UsingSection(
    val formulations: List<Formulation>, override val metadata: MetaData
) : Section("Using", metadata) {
    override fun toCode() = sectionToCode(this, *formulations.toTypedArray())
}

internal sealed interface CodifiedItem : ChalkTalkNode

internal data class WritingGroup(
    val writingSection: WritingSection, override val metadata: MetaData
) : Group(metadata), CodifiedItem {
    override fun toCode() = groupToCode(null, writingSection)
}

internal data class WritingSection(val items: List<Text>, override val metadata: MetaData) :
    Section("writing", metadata) {
    override fun toCode() = sectionToCode(this, *items.toTypedArray())
}

internal data class WrittenGroup(
    val writtenSection: WrittenSection, override val metadata: MetaData
) : Group(metadata), CodifiedItem {
    override fun toCode() = groupToCode(null, writtenSection)
}

internal data class WrittenSection(val items: List<Text>, override val metadata: MetaData) :
    Section("written", metadata) {
    override fun toCode() = sectionToCode(this, *items.toTypedArray())
}

internal data class CalledGroup(val calledSection: CalledSection, override val metadata: MetaData) :
    Group(metadata), CodifiedItem {
    override fun toCode() = groupToCode(null, calledSection)
}

internal data class CalledSection(val items: List<Text>, override val metadata: MetaData) :
    Section("called", metadata) {
    override fun toCode() = sectionToCode(this, *items.toTypedArray())
}

internal data class CodifiedSection(
    val items: List<CodifiedItem>, override val metadata: MetaData
) : Section("Codified", metadata) {
    override fun toCode() = sectionToCode(this, *items.toTypedArray())
}

internal interface ProvidingItem : ChalkTalkNode

internal data class ProvidingSection(
    val items: List<ProvidingItem>, override val metadata: MetaData
) : Section("Providing", metadata) {
    override fun toCode() = sectionToCode(this, *items.toTypedArray())
}

internal interface MetadataItem : ChalkTalkNode

internal data class MetadataSection(
    val items: List<MetadataItem>, override val metadata: MetaData
) : Section("Metadata", metadata) {
    override fun toCode() = sectionToCode(this, *items.toTypedArray())
}

internal data class DefinesGroup(
    val id: Id,
    val definesSection: DefinesSection,
    val withSection: WithSection?,
    val givenSection: GivenSection?,
    val whenSection: WhenSection?,
    val suchThatSection: SuchThatSection?,
    val meansSection: MeansSection?,
    val satisfyingSection: SatisfyingSection?,
    val expressingSection: ExpressingSection?,
    val providingSection: ProvidingSection?,
    val usingSection: UsingSection?,
    val codifiedSection: CodifiedSection,
    val documentedSection: DocumentedSection?,
    val referencesSection: ReferencesSection?,
    val metadataSection: MetadataSection?,
    override val metadata: MetaData
) : Group(metadata), TopLevelGroup {
    override fun toCode() =
        groupToCode(
            id,
            definesSection,
            withSection,
            givenSection,
            whenSection,
            suchThatSection,
            meansSection,
            satisfyingSection,
            expressingSection,
            providingSection,
            usingSection,
            codifiedSection,
            documentedSection,
            referencesSection,
            metadataSection)
}

internal data class StatesSection(override val metadata: MetaData) : Section("States", metadata) {
    override fun toCode() = sectionToCode(this)
}

internal sealed interface ThatItem : ChalkTalkNode

internal data class ThatSection(val items: List<ThatItem>, override val metadata: MetaData) :
    Section("that", metadata) {
    override fun toCode() = sectionToCode(this, *items.toTypedArray())
}

internal data class StatesGroup(
    val id: Id,
    val statesSection: StatesSection,
    val givenSection: GivenSection?,
    val whenSection: WhenSection?,
    val suchThatSection: SuchThatSection?,
    val thatSection: ThatSection,
    val usingSection: UsingSection?,
    val codifiedSection: CodifiedSection,
    val documentedSection: DocumentedSection?,
    val referencesSection: ReferencesSection?,
    val metadataSection: MetadataSection?,
    override val metadata: MetaData
) : Group(metadata), TopLevelGroup {
    override fun toCode() =
        groupToCode(
            id,
            statesSection,
            givenSection,
            whenSection,
            suchThatSection,
            thatSection,
            usingSection,
            codifiedSection,
            documentedSection,
            referencesSection,
            metadataSection)
}

internal interface ResourceItem : ChalkTalkNode

internal data class ResourceSection(
    val items: List<ResourceItem>, override val metadata: MetaData
) : Section("Resource", metadata) {
    override fun toCode() = sectionToCode(this, *items.toTypedArray())
}

internal data class TypeSection(val type: Text, override val metadata: MetaData) :
    Section("type", metadata) {
    override fun toCode() = sectionToCode(this, type)
}

internal data class TypeGroup(val typeSection: TypeSection, override val metadata: MetaData) :
    Group(metadata), ResourceItem {
    override fun toCode() = groupToCode(null, typeSection)
}

internal data class NameSection(val text: Text, override val metadata: MetaData) :
    Section("name", metadata) {
    override fun toCode() = sectionToCode(this, text)
}

internal data class NameGroup(val nameSection: NameSection, override val metadata: MetaData) :
    Group(metadata), ResourceItem {
    override fun toCode() = groupToCode(null, nameSection)
}

internal data class HomepageSection(val homepage: Text, override val metadata: MetaData) :
    Section("homepage", metadata) {
    override fun toCode() = sectionToCode(this, homepage)
}

internal data class HomepageGroup(
    val homepageSection: HomepageSection, override val metadata: MetaData
) : Group(metadata), ResourceItem {
    override fun toCode() = groupToCode(null, homepageSection)
}

internal data class UrlSection(val url: Text, override val metadata: MetaData) :
    Section("url", metadata) {
    override fun toCode() = sectionToCode(this, url)
}

internal data class UrlGroup(val urlSection: UrlSection, override val metadata: MetaData) :
    Group(metadata), ResourceItem {
    override fun toCode() = groupToCode(null, urlSection)
}

internal data class OffsetSection(val offset: Text, override val metadata: MetaData) :
    Section("offset", metadata) {
    override fun toCode() = sectionToCode(this, offset)
}

internal data class OffsetGroup(val offsetSection: OffsetSection, override val metadata: MetaData) :
    Group(metadata), ResourceItem {
    override fun toCode() = groupToCode(null, offsetSection)
}

internal data class ResourceName(val name: String, override val metadata: MetaData) :
    ChalkTalkNode {
    override fun toCode() = "@$name"
}

internal data class ResourceGroup(
    val id: ResourceName, val resourceSection: ResourceSection, override val metadata: MetaData
) : Group(metadata), TopLevelGroup {
    override fun toCode() = groupToCode(null, resourceSection)
}

internal data class AxiomSection(val names: List<Text>, override val metadata: MetaData) :
    Section("Axiom", metadata) {
    override fun toCode() = sectionToCode(this)
}

internal data class AxiomGroup(
    val id: Id?,
    val axiomSection: AxiomSection,
    val givenSection: GivenSection?,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection?,
    val thenSection: ThenSection,
    val iffSection: IffSection?,
    val usingSection: UsingSection?,
    val documentedSection: DocumentedSection?,
    val referencesSection: ReferencesSection?,
    val metadataSection: MetadataSection?,
    override val metadata: MetaData
) : Group(metadata), TopLevelGroup {
    override fun toCode() =
        groupToCode(
            id,
            axiomSection,
            givenSection,
            whereSection,
            suchThatSection,
            thenSection,
            iffSection,
            usingSection,
            documentedSection,
            referencesSection,
            metadataSection)
}

internal data class ConjectureSection(val names: List<Text>, override val metadata: MetaData) :
    Section("Conjecture", metadata) {
    override fun toCode() = sectionToCode(this)
}

internal data class ConjectureGroup(
    val id: Id?,
    val conjectureSection: ConjectureSection,
    val givenSection: GivenSection?,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection?,
    val thenSection: ThenSection,
    val iffSection: IffSection?,
    val usingSection: UsingSection?,
    val documentedSection: DocumentedSection?,
    val referencesSection: ReferencesSection?,
    val metadataSection: MetadataSection?,
    override val metadata: MetaData
) : Group(metadata), TopLevelGroup {
    override fun toCode() =
        groupToCode(
            id,
            conjectureSection,
            givenSection,
            whereSection,
            suchThatSection,
            thenSection,
            iffSection,
            usingSection,
            documentedSection,
            referencesSection,
            metadataSection)
}

internal data class TheoremSection(val names: List<Text>, override val metadata: MetaData) :
    Section("Theorem", metadata) {
    override fun toCode() = sectionToCode(this)
}

internal data class TheoremGroup(
    val id: Id?,
    val theoremSection: TheoremSection,
    val givenSection: GivenSection?,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection?,
    val thenSection: ThenSection,
    val iffSection: IffSection?,
    val usingSection: UsingSection?,
    val proofSection: ProofSection?,
    val documentedSection: DocumentedSection?,
    val referencesSection: ReferencesSection?,
    val metadataSection: MetadataSection?,
    override val metadata: MetaData
) : Group(metadata), TopLevelGroup {
    override fun toCode() =
        groupToCode(
            id,
            theoremSection,
            givenSection,
            whereSection,
            suchThatSection,
            thenSection,
            iffSection,
            usingSection,
            proofSection,
            documentedSection,
            referencesSection,
            metadataSection)
}

internal data class ProofSection(val proofs: List<Text>, override val metadata: MetaData) :
    Section("Proof", metadata) {
    override fun toCode() = sectionToCode(this, *proofs.toTypedArray())
}

internal data class TopicSection(val names: List<Text>, override val metadata: MetaData) :
    Section("Topic", metadata) {
    override fun toCode() = sectionToCode(this)
}

internal data class ContentSection(val content: Text, override val metadata: MetaData) :
    Section("content", metadata) {
    override fun toCode() = sectionToCode(this, content)
}

internal data class TopicName(val name: String, override val metadata: MetaData) : ChalkTalkNode {
    override fun toCode() = "#$name"
}

internal data class TopicGroup(
    val id: TopicName,
    val topicSection: TopicSection,
    val contentSection: ContentSection,
    val metadataSection: MetadataSection?,
    override val metadata: MetaData
) : Group(metadata), TopLevelGroup {
    override fun toCode() =
        groupToCodeWithStringId(id, topicSection, contentSection, metadataSection)
}

internal data class NoteTopLevelSection(override val metadata: MetaData) :
    Section("Note", metadata) {
    override fun toCode() = sectionToCode(this)
}

internal data class NoteTopLevelGroup(
    val noteTopLevelSection: NoteTopLevelSection,
    val contentSection: ContentSection,
    val metadataSection: MetadataSection?,
    override val metadata: MetaData
) : Group(metadata), TopLevelGroup {
    override fun toCode() = groupToCode(null, noteTopLevelSection, contentSection, metadataSection)
}

internal interface SpecifyItem : ChalkTalkNode

internal data class SpecifySection(val items: List<SpecifyItem>, override val metadata: MetaData) :
    Section("Specify", metadata) {
    override fun toCode() = sectionToCode(this, *items.toTypedArray())
}

internal data class SpecifyGroup(
    val specifySection: SpecifySection, override val metadata: MetaData
) : Group(metadata), TopLevelGroup {
    override fun toCode() = groupToCode(null, specifySection)
}

internal data class ZeroSection(override val metadata: MetaData) : Section("zero", metadata) {
    override fun toCode() = sectionToCode(this)
}

internal data class ZeroGroup(
    val zeroSection: ZeroSection, val isSection: IsSection, override val metadata: MetaData
) : Group(metadata), SpecifyItem {
    override fun toCode() = groupToCode(null, zeroSection, isSection)
}

internal data class PositiveIntSection(override val metadata: MetaData) :
    Section("positiveInt", metadata) {
    override fun toCode() = sectionToCode(this)
}

internal data class PositiveIntGroup(
    val positiveIntSection: PositiveIntSection,
    val isSection: IsSection,
    override val metadata: MetaData
) : Group(metadata), SpecifyItem {
    override fun toCode() = groupToCode(null, positiveIntSection, isSection)
}

internal data class NegativeIntSection(override val metadata: MetaData) :
    Section("negativeInt", metadata) {
    override fun toCode() = sectionToCode(this)
}

internal data class NegativeIntGroup(
    val negativeIntSection: NegativeIntSection,
    val isSection: IsSection,
    override val metadata: MetaData
) : Group(metadata), SpecifyItem {
    override fun toCode() = groupToCode(null, negativeIntSection, isSection)
}

internal data class PositiveFloatSection(override val metadata: MetaData) :
    Section("positiveFloat", metadata) {
    override fun toCode() = sectionToCode(this)
}

internal data class PositiveFloatGroup(
    val positiveFloatSection: PositiveFloatSection,
    val isSection: IsSection,
    override val metadata: MetaData
) : Group(metadata), SpecifyItem {
    override fun toCode() = groupToCode(null, positiveFloatSection, isSection)
}

internal data class NegativeFloatSection(override val metadata: MetaData) :
    Section("negativeFloat", metadata) {
    override fun toCode() = sectionToCode(this)
}

internal data class NegativeFloatGroup(
    val negativeFloatSection: NegativeFloatSection,
    val isSection: IsSection,
    override val metadata: MetaData
) : Group(metadata), SpecifyItem {
    override fun toCode() = groupToCode(null, negativeFloatSection, isSection)
}

internal data class IsSection(val form: Text, override val metadata: MetaData) :
    Section("is", metadata) {
    override fun toCode() = sectionToCode(this, form)
}

internal interface DocumentedItem : ChalkTalkNode

internal data class LooselySection(val content: Text, override val metadata: MetaData) :
    Section("loosely", metadata) {
    override fun toCode() = sectionToCode(this, content)
}

internal data class LooselyGroup(
    val looselySection: LooselySection, override val metadata: MetaData
) : Group(metadata), DocumentedItem {
    override fun toCode() = groupToCode(null, looselySection)
}

internal data class OverviewSection(val content: Text, override val metadata: MetaData) :
    Section("overview", metadata) {
    override fun toCode() = sectionToCode(this, content)
}

internal data class OverviewGroup(
    val overviewSection: OverviewSection, override val metadata: MetaData
) : Group(metadata), DocumentedItem {
    override fun toCode() = groupToCode(null, overviewSection)
}

internal data class MotivationSection(val content: Text, override val metadata: MetaData) :
    Section("motivation", metadata) {
    override fun toCode() = sectionToCode(this, content)
}

internal data class MotivationGroup(
    val motivationSection: MotivationSection, override val metadata: MetaData
) : Group(metadata), DocumentedItem {
    override fun toCode() = groupToCode(null, motivationSection)
}

internal data class HistorySection(val content: Text, override val metadata: MetaData) :
    Section("history", metadata) {
    override fun toCode() = sectionToCode(this, content)
}

internal data class HistoryGroup(
    val historySection: HistorySection, override val metadata: MetaData
) : Group(metadata), DocumentedItem {
    override fun toCode() = groupToCode(null, historySection)
}

internal data class ExamplesSection(val items: List<Text>, override val metadata: MetaData) :
    Section("examples", metadata) {
    override fun toCode() = sectionToCode(this, *items.toTypedArray())
}

internal data class ExamplesGroup(
    val examplesSection: ExamplesSection, override val metadata: MetaData
) : Group(metadata), DocumentedItem {
    override fun toCode() = groupToCode(null, examplesSection)
}

internal data class RelatedSection(val items: List<Text>, override val metadata: MetaData) :
    Section("related", metadata) {
    override fun toCode() = sectionToCode(this, *items.toTypedArray())
}

internal data class RelatedGroup(
    val relatedSection: RelatedSection, override val metadata: MetaData
) : Group(metadata), DocumentedItem {
    override fun toCode() = groupToCode(null, relatedSection)
}

internal data class DiscoveredSection(val items: List<Text>, override val metadata: MetaData) :
    Section("discovered", metadata) {
    override fun toCode() = sectionToCode(this, *items.toTypedArray())
}

internal data class DiscoveredGroup(
    val discoveredSection: DiscoveredSection, override val metadata: MetaData
) : Group(metadata), DocumentedItem {
    override fun toCode() = groupToCode(null, discoveredSection)
}

internal data class NotesSection(val items: List<Text>, override val metadata: MetaData) :
    Section("notes", metadata) {
    override fun toCode() = sectionToCode(this, *items.toTypedArray())
}

internal data class NotesGroup(val notesSection: NotesSection, override val metadata: MetaData) :
    Group(metadata), DocumentedItem {
    override fun toCode() = groupToCode(null, notesSection)
}

internal data class DocumentedSection(
    val items: List<DocumentedItem>, override val metadata: MetaData
) : Section("Documented", metadata) {
    override fun toCode() = sectionToCode(this, *items.toTypedArray())
}

internal interface TopLevelGroup : TopLevelGroupOrTextBlock

internal interface TopLevelGroupOrTextBlock : ChalkTalkNode

internal data class Document(val items: List<TopLevelGroupOrTextBlock>) {
    fun toCode(): String {
        val builder = StringBuilder()
        for (i in items.indices) {
            if (i > 0) {
                builder.append("\n\n\n")
            }
            builder.append(items[i].toCode())
        }
        return builder.toString()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// adds a `. ` to the start of the code and indents the subsequent lines
private fun codeToDotSpaceArg(code: String): String {
    val builder = StringBuilder()
    val lines = code.split("\n")
    if (lines.isNotEmpty()) {
        builder.append(". ")
        builder.append(lines[0])
    }
    for (i in 1 until lines.size) {
        builder.append("\n  ")
        builder.append(lines[i])
    }
    return builder.toString()
}

private fun sectionToCode(section: Section, vararg args: ChalkTalkNode): String {
    val builder = StringBuilder()
    builder.append(section.name)
    builder.append(":")
    var i = 0
    while (i < args.size) {
        val arg = args[i++]
        if (arg.metadata.isInline == false) {
            builder.append("\n")
            builder.append(codeToDotSpaceArg(arg.toCode()))
            while (i < args.size && args[i].metadata.isInline == true) {
                builder.append(", ")
                builder.append(args[i++].toCode())
            }
        } else {
            builder.append(" ")
            builder.append(arg.toCode())
            while (i < args.size && args[i].metadata.isInline == true) {
                builder.append(", ")
                builder.append(args[i++].toCode())
            }
        }
    }
    return builder.toString()
}

private fun groupToCode(id: ToCode?, vararg sections: Section?) =
    groupToCodeWithStringId(id, *sections)

private fun groupToCodeWithStringId(id: ToCode?, vararg sections: Section?): String {
    val builder = StringBuilder()
    if (id != null) {
        builder.append("[")
        builder.append(id.toCode())
        builder.append("]\n")
    }
    var isFirstPrinted = true
    for (sect in sections) {
        if (sect != null) {
            if (!isFirstPrinted) {
                builder.append("\n")
            }
            builder.append(sect.toCode())
            isFirstPrinted = false
        }
    }
    return builder.toString()
}

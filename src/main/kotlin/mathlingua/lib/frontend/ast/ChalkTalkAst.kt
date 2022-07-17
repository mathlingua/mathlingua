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

internal sealed interface ChalkTalkNode : HasMetaData, Node

internal data class TextBlock(val text: String, override val metadata: MetaData) :
    ChalkTalkNode, TopLevelGroupOrTextBlock, NodeLexerToken

internal data class Id(val text: String, override val metadata: MetaData) :
    ChalkTalkNode, NodeLexerToken

internal sealed interface IdForm : ChalkTalkNode

internal data class Formulation(val text: String, override val metadata: MetaData) :
    Argument, Spec, ProvidedItem, Clause

internal data class Text(val text: String, override val metadata: MetaData) : Argument, Clause

internal sealed interface Clause : ThatItem, SatisfyingItem, ExpressingItem

internal sealed interface Spec : Clause, SatisfyingItem, ExpressingItem, ThatItem

internal sealed interface Section : ChalkTalkNode {
    val name: String
}

internal sealed interface Group : ChalkTalkNode

internal data class AndSection(val clauses: List<Clause>, override val metadata: MetaData) :
    Section {
    override val name = "and"
}

internal data class AndGroup(val andSection: AndSection, override val metadata: MetaData) :
    Group, Clause

internal data class NotSection(val clause: Clause, override val metadata: MetaData) : Section {
    override val name = "not"
}

internal data class NotGroup(val notSection: NotSection, override val metadata: MetaData) :
    Group, Clause

internal data class OrSection(val clauses: List<Clause>, override val metadata: MetaData) :
    Section {
    override val name = "or"
}

internal data class OrGroup(val orSection: OrSection, override val metadata: MetaData) :
    Group, Clause

internal data class ExistsSection(val targets: List<Target>, override val metadata: MetaData) :
    Section {
    override val name = "exists"
}

internal data class WhereSection(val specs: List<Spec>, override val metadata: MetaData) : Section {
    override val name = "where"
}

internal data class SuchThatSection(val clauses: List<Clause>, override val metadata: MetaData) :
    Section {
    override val name = "suchThat"
}

internal data class ExistsGroup(
    val existsSection: ExistsSection,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection?,
    override val metadata: MetaData
) : Group, Clause

internal data class ExistsUniqueSection(
    val targets: List<Target>, override val metadata: MetaData
) : Section {
    override val name = "existsUnique"
}

internal data class ExistsUniqueGroup(
    val existsUniqueSection: ExistsUniqueSection,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection?,
    override val metadata: MetaData
) : Group, Clause

internal data class ForAllSection(val targets: List<Target>, override val metadata: MetaData) :
    Section {
    override val name = "forAll"
}

internal data class ForAllGroup(
    val forAllSection: ForAllSection,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection?,
    val thenSection: ThenSection,
    override val metadata: MetaData
) : Group, Clause

internal data class ThenSection(val clauses: List<Clause>, override val metadata: MetaData) :
    Section {
    override val name = "then"
}

internal data class IfSection(val clauses: List<Clause>, override val metadata: MetaData) :
    Section {
    override val name = "if"
}

internal data class IfGroup(
    val ifSection: IfSection, val thenSection: ThenSection, override val metadata: MetaData
) : Group, Clause

internal data class IffSection(val clauses: List<Clause>, override val metadata: MetaData) :
    Section {
    override val name = "iff"
}

internal data class IffGroup(
    val iffSection: IffSection, val thenSection: ThenSection, override val metadata: MetaData
) : Group, Clause

internal data class GeneratedSection(override val metadata: MetaData) : Section {
    override val name = "generated"
}

internal data class FromSection(val items: List<NameOrFunction>, override val metadata: MetaData) :
    Section {
    override val name = "from"
}

internal data class GeneratedWhenSection(
    val formulations: List<Formulation>, override val metadata: MetaData
) : Section {
    override val name = "when"
}

internal data class GeneratedGroup(
    val generatedSection: GeneratedSection,
    val fromSection: FromSection,
    val generatedWhenSection: GeneratedWhenSection,
    override val metadata: MetaData
) : Group, SatisfyingItem

internal data class PiecewiseSection(override val metadata: MetaData) : Section {
    override val name = "piecewise"
}

internal data class PiecewiseWhenSection(
    val clauses: List<Clause>, override val metadata: MetaData
) : Section {
    override val name = "when"
}

internal data class PiecewiseThenSection(
    val formulations: List<Formulation>, override val metadata: MetaData
) : Section {
    override val name = "then"
}

internal data class PiecewiseElseSection(
    val formulations: List<Formulation>, override val metadata: MetaData
) : Section {
    override val name = "else"
}

internal data class PiecewiseGroup(
    val piecewiseSection: PiecewiseSection,
    val piecewiseWhenSection: PiecewiseWhenSection?,
    val piecewiseThenSection: PiecewiseThenSection?,
    val piecewiseElseSection: PiecewiseElseSection?,
    override val metadata: MetaData
) : Group, ExpressingItem

internal data class MatchingSection(
    val formulations: List<Formulation>, override val metadata: MetaData
) : Section {
    override val name = "matching"
}

internal data class MatchingGroup(
    val matchingSection: MatchingSection, override val metadata: MetaData
) : Group, ExpressingItem

internal data class EqualitySection(override val metadata: MetaData) : Section {
    override val name = "equality"
}

internal data class BetweenSection(
    val first: Target, val second: Target, override val metadata: MetaData
) : Section {
    override val name = "between"
}

internal sealed interface ProvidedItem : ChalkTalkNode

internal data class ProvidedSection(val statement: ProvidedItem, override val metadata: MetaData) :
    Section {
    override val name = "provided"
}

internal data class EqualityGroup(
    val equalitySection: EqualitySection,
    val betweenSection: BetweenSection,
    val providedSection: ProvidedSection,
    override val metadata: MetaData
) : Group, ProvidingItem

internal data class MembershipSection(override val metadata: MetaData) : Section {
    override val name = "membership"
}

internal data class ThroughSection(val through: Formulation, override val metadata: MetaData) :
    Section {
    override val name = "through"
}

internal data class MembershipGroup(
    val membershipSection: MembershipSection,
    val throughSection: ThroughSection,
    override val metadata: MetaData
) : Group, ProvidingItem

internal data class ViewSection(override val metadata: MetaData) : Section {
    override val name = "view"
}

internal data class AsSection(val asText: Text, override val metadata: MetaData) : Section {
    override val name = "as"
}

internal data class ViaSection(val via: Formulation, override val metadata: MetaData) : Section {
    override val name = "via"
}

internal data class BySection(val by: Formulation, override val metadata: MetaData) : Section {
    override val name = "by"
}

internal data class ViewGroup(
    val viewSection: ViewSection,
    val asSection: AsSection,
    val viaSection: ViaSection,
    val bySection: BySection?,
    override val metadata: MetaData
) : Group, ProvidingItem

internal data class SymbolsSection(val names: List<Name>, override val metadata: MetaData) :
    Section {
    override val name = "symbols"
}

internal data class SymbolsWhereSection(
    val formulations: List<Formulation>, override val metadata: MetaData
) : Section {
    override val name = "where"
}

internal data class SymbolsGroup(
    val symbolsSection: SymbolsSection,
    val symbolsWhereSection: SymbolsWhereSection,
    override val metadata: MetaData
) : Group, ProvidingItem

internal data class SymbolsAsGroup(
    val symbolsSection: SymbolsSection,
    val symbolsAsSection: SymbolsAsSection,
    override val metadata: MetaData
) : Group, ProvidingItem

internal data class SymbolsAsSection(val asSection: Text, override val metadata: MetaData) :
    Section {
    override val name = "as"
}

internal data class MemberSymbolsSection(val names: List<Name>, override val metadata: MetaData) :
    Section {
    override val name = "memberSymbols"
}

internal data class MemberSymbolsWhereSection(
    val formulations: List<Formulation>, override val metadata: MetaData
) : Section {
    override val name = "where"
}

internal data class MemberSymbolsAsSection(val asSection: Text, override val metadata: MetaData) :
    Section {
    override val name = "as"
}

internal data class MemberSymbolsGroup(
    val memberSymbolsSection: MemberSymbolsSection,
    val memberSymbolsWhereSection: MemberSymbolsWhereSection,
    override val metadata: MetaData
) : Group, ProvidingItem

internal data class MemberSymbolsAsGroup(
    val memberSymbolsSection: MemberSymbolsSection,
    val memberSymbolsAsSection: MemberSymbolsAsSection,
    override val metadata: MetaData
) : Group, ProvidingItem

internal data class ContributorSection(val items: List<Text>, override val metadata: MetaData) :
    Section {
    override val name = "author"
}

internal data class ContributorGroup(
    val contributorSection: ContributorSection, override val metadata: MetaData
) : Group, MetadataItem

internal data class AuthorSection(val items: List<Text>, override val metadata: MetaData) :
    Section {
    override val name = "author"
}

internal data class AuthorGroup(val authorSection: AuthorSection, override val metadata: MetaData) :
    Group, ResourceItem

internal data class TagSection(val items: List<Text>, override val metadata: MetaData) : Section {
    override val name = "tag"
}

internal data class TagGroup(val tagSection: TagSection, override val metadata: MetaData) :
    Group, MetadataItem

internal data class ReferencesSection(val items: List<Text>, override val metadata: MetaData) :
    Section {
    override val name = "References"
}

internal data class DefinesSection(val target: Target, override val metadata: MetaData) : Section {
    override val name = "Defines"
}

internal data class WithSection(
    val assignments: List<NameAssignment>, override val metadata: MetaData
) : Section {
    override val name = "with"
}

internal data class GivenSection(val targets: List<Target>, override val metadata: MetaData) :
    Section {
    override val name = "given"
}

internal data class WhenSection(val specs: List<Spec>, override val metadata: MetaData) : Section {
    override val name = "when"
}

internal data class MeansSection(val statement: Formulation, override val metadata: MetaData) :
    Section {
    override val name = "means"
}

internal sealed interface SatisfyingItem : ChalkTalkNode

internal data class SatisfyingSection(
    val items: List<SatisfyingItem>, override val metadata: MetaData
) : Section {
    override val name = "satisfying"
}

internal sealed interface ExpressingItem : ChalkTalkNode

internal data class ExpressingSection(
    val items: List<ExpressingItem>, override val metadata: MetaData
) : Section {
    override val name = "expressing"
}

internal data class UsingSection(
    val formulations: List<Formulation>, override val metadata: MetaData
) : Section {
    override val name = "Using"
}

internal sealed interface CodifiedItem : ChalkTalkNode

internal data class WritingGroup(
    val writingSection: WritingSection, override val metadata: MetaData
) : Group, CodifiedItem

internal data class WritingSection(val items: List<Text>, override val metadata: MetaData) :
    Section {
    override val name = "writing"
}

internal data class WrittenGroup(
    val writtenSection: WrittenSection, override val metadata: MetaData
) : Group, CodifiedItem

internal data class WrittenSection(val items: List<Text>, override val metadata: MetaData) :
    Section {
    override val name = "written"
}

internal data class CalledGroup(val calledSection: CalledSection, override val metadata: MetaData) :
    Group, CodifiedItem

internal data class CalledSection(val items: List<Text>, override val metadata: MetaData) :
    Section {
    override val name = "called"
}

internal data class CodifiedSection(
    val items: List<CodifiedItem>, override val metadata: MetaData
) : Section {
    override val name = "Codified"
}

internal sealed interface ProvidingItem : ChalkTalkNode

internal data class ProvidingSection(
    val items: List<ProvidingItem>, override val metadata: MetaData
) : Section {
    override val name = "Providing"
}

internal sealed interface MetadataItem : ChalkTalkNode

internal data class MetadataSection(
    val items: List<MetadataItem>, override val metadata: MetaData
) : Section {
    override val name = "Metadata"
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
) : Group, TopLevelGroup

internal data class StatesSection(override val metadata: MetaData) : Section {
    override val name = "States"
}

internal sealed interface ThatItem : ChalkTalkNode

internal data class ThatSection(val items: List<ThatItem>, override val metadata: MetaData) :
    Section {
    override val name = "that"
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
) : Group, TopLevelGroup

internal sealed interface ResourceItem : ChalkTalkNode

internal data class ResourceSection(
    val items: List<ResourceItem>, override val metadata: MetaData
) : Section {
    override val name = "Resource"
}

internal data class TypeSection(val type: Text, override val metadata: MetaData) : Section {
    override val name = "type"
}

internal data class TypeGroup(val typeSection: TypeSection, override val metadata: MetaData) :
    Group, ResourceItem

internal data class NameSection(val text: Text, override val metadata: MetaData) : Section {
    override val name = "name"
}

internal data class NameGroup(val nameSection: NameSection, override val metadata: MetaData) :
    Group, ResourceItem

internal data class HomepageSection(val homepage: Text, override val metadata: MetaData) : Section {
    override val name = "homepage"
}

internal data class HomepageGroup(
    val homepageSection: HomepageSection, override val metadata: MetaData
) : Group, ResourceItem

internal data class UrlSection(val url: Text, override val metadata: MetaData) : Section {
    override val name = "url"
}

internal data class UrlGroup(val urlSection: UrlSection, override val metadata: MetaData) :
    Group, ResourceItem

internal data class OffsetSection(val offset: Text, override val metadata: MetaData) : Section {
    override val name = "offset"
}

internal data class OffsetGroup(val offsetSection: OffsetSection, override val metadata: MetaData) :
    Group, ResourceItem

internal data class ResourceName(val name: String, override val metadata: MetaData) : ChalkTalkNode

internal data class ResourceGroup(
    val id: ResourceName, val resourceSection: ResourceSection, override val metadata: MetaData
) : Group, TopLevelGroup

internal data class AxiomSection(val names: List<Text>, override val metadata: MetaData) : Section {
    override val name = "Axiom"
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
) : Group, TopLevelGroup

internal data class ConjectureSection(val names: List<Text>, override val metadata: MetaData) :
    Section {
    override val name = "Conjecture"
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
) : Group, TopLevelGroup

internal data class TheoremSection(val names: List<Text>, override val metadata: MetaData) :
    Section {
    override val name = "Theorem"
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
) : Group, TopLevelGroup

internal data class ProofSection(val proofs: List<Text>, override val metadata: MetaData) :
    Section {
    override val name = "Proof"
}

internal data class TopicSection(val names: List<Text>, override val metadata: MetaData) : Section {
    override val name = "Topic"
}

internal data class ContentSection(val content: Text, override val metadata: MetaData) : Section {
    override val name = "content"
}

internal data class TopicName(val name: String, override val metadata: MetaData) : ChalkTalkNode

internal data class TopicGroup(
    val id: TopicName,
    val topicSection: TopicSection,
    val contentSection: ContentSection,
    val metadataSection: MetadataSection?,
    override val metadata: MetaData
) : Group, TopLevelGroup

internal data class NoteTopLevelSection(override val metadata: MetaData) : Section {
    override val name = "Note"
}

internal data class NoteTopLevelGroup(
    val noteTopLevelSection: NoteTopLevelSection,
    val contentSection: ContentSection,
    val metadataSection: MetadataSection?,
    override val metadata: MetaData
) : Group, TopLevelGroup

internal sealed interface SpecifyItem : ChalkTalkNode

internal data class SpecifySection(val items: List<SpecifyItem>, override val metadata: MetaData) :
    Section {
    override val name = "Specify"
}

internal data class SpecifyGroup(
    val specifySection: SpecifySection, override val metadata: MetaData
) : Group, TopLevelGroup

internal data class ZeroSection(override val metadata: MetaData) : Section {
    override val name = "zero"
}

internal data class ZeroGroup(
    val zeroSection: ZeroSection, val isSection: IsSection, override val metadata: MetaData
) : Group, SpecifyItem

internal data class PositiveIntSection(override val metadata: MetaData) : Section {
    override val name = "positiveInt"
}

internal data class PositiveIntGroup(
    val positiveIntSection: PositiveIntSection,
    val isSection: IsSection,
    override val metadata: MetaData
) : Group, SpecifyItem

internal data class NegativeIntSection(override val metadata: MetaData) : Section {
    override val name = "negativeInt"
}

internal data class NegativeIntGroup(
    val negativeIntSection: NegativeIntSection,
    val isSection: IsSection,
    override val metadata: MetaData
) : Group, SpecifyItem

internal data class PositiveFloatSection(override val metadata: MetaData) : Section {
    override val name = "positiveFloat"
}

internal data class PositiveFloatGroup(
    val positiveFloatSection: PositiveFloatSection,
    val isSection: IsSection,
    override val metadata: MetaData
) : Group, SpecifyItem

internal data class NegativeFloatSection(override val metadata: MetaData) : Section {
    override val name = "negativeFloat"
}

internal data class NegativeFloatGroup(
    val negativeFloatSection: NegativeFloatSection,
    val isSection: IsSection,
    override val metadata: MetaData
) : Group, SpecifyItem

internal data class IsSection(val form: Text, override val metadata: MetaData) : Section {
    override val name = "is"
}

internal sealed interface DocumentedItem : ChalkTalkNode

internal data class LooselySection(val content: Text, override val metadata: MetaData) : Section {
    override val name = "loosely"
}

internal data class LooselyGroup(
    val looselySection: LooselySection, override val metadata: MetaData
) : Group, DocumentedItem

internal data class OverviewSection(val content: Text, override val metadata: MetaData) : Section {
    override val name = "overview"
}

internal data class OverviewGroup(
    val overviewSection: OverviewSection, override val metadata: MetaData
) : Group, DocumentedItem

internal data class MotivationSection(val content: Text, override val metadata: MetaData) :
    Section {
    override val name = "motivation"
}

internal data class MotivationGroup(
    val motivationSection: MotivationSection, override val metadata: MetaData
) : Group, DocumentedItem

internal data class HistorySection(val content: Text, override val metadata: MetaData) : Section {
    override val name = "history"
}

internal data class HistoryGroup(
    val historySection: HistorySection, override val metadata: MetaData
) : Group, DocumentedItem

internal data class ExamplesSection(val items: List<Text>, override val metadata: MetaData) :
    Section {
    override val name = "examples"
}

internal data class ExamplesGroup(
    val examplesSection: ExamplesSection, override val metadata: MetaData
) : Group, DocumentedItem

internal data class RelatedSection(val items: List<Text>, override val metadata: MetaData) :
    Section {
    override val name = "related"
}

internal data class RelatedGroup(
    val relatedSection: RelatedSection, override val metadata: MetaData
) : Group, DocumentedItem

internal data class DiscoveredSection(val items: List<Text>, override val metadata: MetaData) :
    Section {
    override val name = "discovered"
}

internal data class DiscoveredGroup(
    val discoveredSection: DiscoveredSection, override val metadata: MetaData
) : Group, DocumentedItem

internal data class NotesSection(val items: List<Text>, override val metadata: MetaData) : Section {
    override val name = "notes"
}

internal data class NotesGroup(val notesSection: NotesSection, override val metadata: MetaData) :
    Group, DocumentedItem

internal data class DocumentedSection(
    val items: List<DocumentedItem>, override val metadata: MetaData
) : Section {
    override val name = "Documented"
}

internal sealed interface TopLevelGroup : TopLevelGroupOrTextBlock

internal sealed interface TopLevelGroupOrTextBlock : ChalkTalkNode

internal data class Document(val items: List<TopLevelGroupOrTextBlock>)

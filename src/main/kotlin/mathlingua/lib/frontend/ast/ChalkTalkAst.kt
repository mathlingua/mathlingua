package mathlingua.lib.frontend.ast

import mathlingua.lib.frontend.HasMetaData
import mathlingua.lib.frontend.MetaData

internal sealed interface ChalkTalkNode : HasMetaData

internal data class TextBlock(val text: String, override val metadata: MetaData) :
    ChalkTalkNode, TopLevelGroupOrTextBlock

internal data class Id(val text: String, override val metadata: MetaData) : ChalkTalkNode

internal data class Statement(val text: String, override val metadata: MetaData) : Argument

internal data class Text(val text: String, override val metadata: MetaData) : Argument

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

internal object BeginGroup : ChalkTalkNode {
    override val metadata = MetaData(row = -1, column = -1, isInline = false)
    override fun toString() = javaClass.simpleName
}

internal object EndGroup : ChalkTalkNode {
    override val metadata = MetaData(row = -1, column = -1, isInline = false)
    override fun toString() = javaClass.simpleName
}

internal data class BeginSection(val name: String) : ChalkTalkNode {
    override val metadata = MetaData(row = -1, column = -1, isInline = false)
}

internal object EndSection : ChalkTalkNode {
    override val metadata = MetaData(row = -1, column = -1, isInline = false)
    override fun toString() = javaClass.simpleName
}

internal object BeginArgument : ChalkTalkNode {
    override val metadata = MetaData(row = -1, column = -1, isInline = false)
    override fun toString() = javaClass.simpleName
}

internal object EndArgument : ChalkTalkNode {
    override val metadata = MetaData(row = -1, column = -1, isInline = false)
    override fun toString() = javaClass.simpleName
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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
internal interface Spec : Clause

internal open class Section(val name: String, override val metadata: MetaData) : ChalkTalkNode

internal open class Group(override val metadata: MetaData) : ChalkTalkNode

// internal data class Group(, override val metadata: MetaData) : Group(metadata)

internal data class AndSection(val clauses: List<Clause>, override val metadata: MetaData) :
    Section("and", metadata)

internal data class AndGroup(val andSection: AndSection, override val metadata: MetaData) :
    Group(metadata), Clause

internal data class NotSection(val clause: Clause, override val metadata: MetaData) :
    Section("not", metadata)

internal data class NotGroup(val notSection: NotSection, override val metadata: MetaData) :
    Group(metadata), Clause

internal data class OrSection(val clauses: List<Clause>, override val metadata: MetaData) :
    Section("or", metadata)

internal data class OrGroup(val orSection: OrSection, override val metadata: MetaData) :
    Group(metadata), Clause

internal data class ExistsSection(val targets: List<Target>, override val metadata: MetaData) :
    Section("exists", metadata)

internal data class WhereSection(val specs: List<Spec>, override val metadata: MetaData) :
    Section("where", metadata)

internal data class SuchThatSection(val clauses: List<Clause>, override val metadata: MetaData) :
    Section("suchThat", metadata)

internal data class ExistsGroup(
    val existsSection: ExistsSection,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection?,
    override val metadata: MetaData
) : Group(metadata), Clause

internal data class ExistsUniqueSection(
    val targets: List<Target>, override val metadata: MetaData
) : Section("existsUnique", metadata)

internal data class ExistsUniqueGroup(
    val existsUniqueSection: ExistsUniqueSection,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection?,
    override val metadata: MetaData
) : Group(metadata), Clause

internal data class ForAllSection(val targets: List<Target>, override val metadata: MetaData) :
    Section("forAll", metadata)

internal data class ForAllGroup(
    val forAllSection: ForAllSection,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection?,
    val thenSection: ThenSection,
    override val metadata: MetaData
) : Group(metadata), Clause

internal data class ThenSection(val clauses: List<Clause>, override val metadata: MetaData) :
    Section("then", metadata)

internal data class IfSection(val clauses: List<Clause>, override val metadata: MetaData) :
    Section("if", metadata)

internal data class IfGroup(
    val ifSection: IfSection, val thenSection: ThenSection, override val metadata: MetaData
) : Group(metadata), Clause

internal data class IffSection(val clauses: List<Clause>, override val metadata: MetaData) :
    Section("iff", metadata)

internal data class IffGroup(
    val iffSection: IffSection, val thenSection: ThenSection, override val metadata: MetaData
) : Group(metadata), Clause

internal data class GeneratedSection(override val metadata: MetaData) :
    Section("generated", metadata)

internal data class FromSection(val items: List<NameOrFunction>, override val metadata: MetaData) :
    Section("from", metadata)

internal data class GeneratedWhenSection(
    val statements: List<Statement>, override val metadata: MetaData
) : Section("when", metadata)

internal data class GeneratedGroup(
    val fromSection: FromSection, val whenSection: WhenSection, override val metadata: MetaData
) : Group(metadata), SatisfyingItem

internal data class PiecewiseSection(override val metadata: MetaData) :
    Section("piecewise", metadata)

internal data class PiecewiseWhenSection(
    val clauses: List<Clause>, override val metadata: MetaData
) : Section("when", metadata)

internal data class PiecewiseThenSection(
    val statements: List<Statement>, override val metadata: MetaData
) : Section("then", metadata)

internal data class PiecewiseElseSection(
    val statements: List<Statement>, override val metadata: MetaData
) : Section("else", metadata)

internal data class PiecewiseGroup(
    val piecewiseSection: PiecewiseSection,
    val whenSection: PiecewiseWhenSection?,
    val thenSection: PiecewiseThenSection?,
    val piecewiseElseSection: PiecewiseElseSection?,
    override val metadata: MetaData
) : Group(metadata), ExpressingItem

internal data class MatchingSection(
    val statements: List<Statement>, override val metadata: MetaData
) : Section("matching", metadata)

internal data class MatchingGroup(
    val matchingSection: MatchingSection, override val metadata: MetaData
) : Group(metadata), ExpressingItem

internal data class EqualitySection(override val metadata: MetaData) :
    Section("equality", metadata)

internal data class BetweenSection(
    val first: Target, val second: Target, override val metadata: MetaData
) : Section("between", metadata)

internal data class ProvidedSection(val statement: Statement, override val metadata: MetaData) :
    Section("provided", metadata)

internal data class EqualityGroup(
    val equalitySection: EqualitySection,
    val betweenSection: BetweenSection,
    val providedSection: ProvidedSection,
    override val metadata: MetaData
) : Group(metadata), ProvidingItem

internal data class MembershipSection(override val metadata: MetaData) :
    Section("membership", metadata)

internal data class ThroughSection(val through: Name, override val metadata: MetaData) :
    Section("through", metadata)

internal data class MembershipGroup(
    val membershipSection: MembershipSection,
    val throughSection: ThroughSection,
    override val metadata: MetaData
) : Group(metadata), ProvidingItem

internal data class ViewSection(override val metadata: MetaData) : Section("view", metadata)

internal data class AsSection(val asText: Text, override val metadata: MetaData) :
    Section("as", metadata)

internal data class ViaSection(val via: Statement, override val metadata: MetaData) :
    Section("via", metadata)

internal data class BySection(val by: Statement, override val metadata: MetaData) :
    Section("by", metadata)

internal data class ViewGroup(
    val viewSection: ViewSection,
    val asSection: AsSection,
    val viaSection: ViaSection,
    val bySection: BySection?,
    override val metadata: MetaData
) : Group(metadata), ProvidingItem

internal data class SymbolsSection(val names: List<Name>, override val metadata: MetaData) :
    Section("symbols", metadata)

internal data class SymbolsWhereSection(
    val statements: List<Statement>, override val metadata: MetaData
) : Section("where", metadata)

internal data class SymbolsGroup(
    val symbolsSection: SymbolsSection,
    val whereSection: SymbolsWhereSection,
    override val metadata: MetaData
) : Group(metadata), ProvidingItem

internal data class MemberSymbolsSection(val names: List<Name>, override val metadata: MetaData) :
    Section("memberSymbols", metadata)

internal data class MemberSymbolsWhereSection(
    val statements: List<Statement>, override val metadata: MetaData
) : Section("where", metadata)

internal data class MemberSymbolsGroup(
    val memberSymbolsSection: MemberSymbolsSection,
    val whereSection: MemberSymbolsWhereSection,
    override val metadata: MetaData
) : Group(metadata), ProvidingItem

internal data class NoteSection(val items: List<Text>, override val metadata: MetaData) :
    Section("note", metadata)

internal data class NoteGroup(val noteSection: NoteSection, override val metadata: MetaData) :
    Group(metadata), MetadataItem

internal data class AuthorSection(val items: List<Text>, override val metadata: MetaData) :
    Section("author", metadata)

internal data class AuthorGroup(val authorSection: AuthorSection, override val metadata: MetaData) :
    Group(metadata), MetadataItem

internal data class TagSection(val items: List<Text>, override val metadata: MetaData) :
    Section("tag", metadata)

internal data class TagGroup(val tagSection: TagSection, override val metadata: MetaData) :
    Group(metadata), MetadataItem

internal data class ReferenceSection(val items: List<Text>, override val metadata: MetaData) :
    Section("reference", metadata)

internal data class ReferenceGroup(
    val referenceSection: ReferenceSection, override val metadata: MetaData
) : Group(metadata), MetadataItem

internal data class DefinesSection(val target: Target, override val metadata: MetaData) :
    Section("Defines", metadata)

internal data class WithSection(
    val assignments: List<Assignment>, override val metadata: MetaData
) : Section("with", metadata)

internal data class GivenSection(val targets: List<Target>, override val metadata: MetaData) :
    Section("given", metadata)

internal data class WhenSection(val specs: List<Spec>, override val metadata: MetaData) :
    Section("when", metadata)

internal data class MeansSection(val statement: Statement, override val metadata: MetaData) :
    Section("means", metadata)

internal interface SatisfyingItem

internal data class SatisfyingSection(
    val items: List<SatisfyingItem>, override val metadata: MetaData
) : Section("satisfying", metadata)

internal interface ExpressingItem

internal data class ExpressingSection(
    val items: List<ExpressingItem>, override val metadata: MetaData
) : Section("expressing", metadata)

internal data class UsingSection(val statements: List<Statement>, override val metadata: MetaData) :
    Section("using", metadata)

internal data class WritingSection(val items: List<Text>, override val metadata: MetaData) :
    Section("writing", metadata)

internal data class WrittenSection(val items: List<Text>, override val metadata: MetaData) :
    Section("written", metadata)

internal data class CalledSection(val items: List<Text>, override val metadata: MetaData) :
    Section("called", metadata)

internal interface ProvidingItem

internal data class ProvidingSection(
    val items: List<ProvidingItem>, override val metadata: MetaData
) : Section("Providing", metadata)

internal interface MetadataItem

internal data class MetadataSection(
    val items: List<MetadataItem>, override val metadata: MetaData
) : Section("Metadata", metadata)

internal data class DefinesGroup(
    val id: Id,
    val definesSection: DefinesSection,
    val withSection: WithSection?,
    val givenSection: GivenSection?,
    val whenSection: WhenSection?,
    val meansSection: MeansSection?,
    val satisfyingSection: SatisfyingSection?,
    val expressingSection: ExpressingSection?,
    val usingSection: UsingSection?,
    val writingSection: WritingSection?,
    val writtenSection: WrittenSection,
    val calledSection: CalledSection?,
    val providingSection: ProvidingSection?,
    val metadataSection: MetadataSection?,
    override val metadata: MetaData
) : Group(metadata), TopLevelGroup

internal data class StatesSection(override val metadata: MetaData) : Section("States", metadata)

internal sealed interface ThatItem : ChalkTalkNode

internal data class ThatSection(val items: List<ThatItem>, override val metadata: MetaData) :
    Section("that", metadata)

internal data class StatesGroup(
    val id: Id,
    val statesSection: StatesSection,
    val givenSection: GivenSection?,
    val whenSection: WhenSection?,
    val thatSection: ThatSection,
    val usingSection: UsingSection?,
    val writtenSection: WrittenSection,
    val calledSection: CalledSection?,
    val metadataSection: MetadataSection?,
    override val metadata: MetaData
) : Group(metadata), TopLevelGroup

internal interface ResourceItem

internal data class ResourceSection(
    val items: List<ResourceItem>, override val metadata: MetaData
) : Section("Resource", metadata)

internal data class TypeSection(val type: Text, override val metadata: MetaData) :
    Section("type", metadata)

internal data class TypeGroup(val typeSection: TypeSection, override val metadata: MetaData) :
    Group(metadata), ResourceItem

internal data class NameSection(val text: Text, override val metadata: MetaData) :
    Section("name", metadata)

internal data class NameGroup(val nameSection: NameSection, override val metadata: MetaData) :
    Group(metadata), ResourceItem

internal data class HomepageSection(val homepage: Text, override val metadata: MetaData) :
    Section("homepage", metadata)

internal data class HomepageGroup(
    val homepageSection: HomepageSection, override val metadata: MetaData
) : Group(metadata), ResourceItem

internal data class UrlSection(val url: Text, override val metadata: MetaData) :
    Section("url", metadata)

internal data class UrlGroup(val urlSection: UrlSection, override val metadata: MetaData) :
    Group(metadata), ResourceItem

internal data class OffsetSection(val offset: Text, override val metadata: MetaData) :
    Section("offset", metadata)

internal data class OffsetGroup(val offsetSection: OffsetSection, override val metadata: MetaData) :
    Group(metadata), ResourceItem

internal data class ResourceGroup(
    val id: String, val resourceSection: ResourceSection, override val metadata: MetaData
) : Group(metadata), TopLevelGroup

internal data class AxiomSection(override val metadata: MetaData) : Section("Axiom", metadata)

internal data class AxiomGroup(
    val id: Id,
    val axiomSection: AxiomSection,
    val givenSection: GivenSection?,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection?,
    val thenSection: ThenSection,
    val iffSection: IffSection?,
    val usingSection: UsingSection?,
    val metadataSection: MetadataSection?,
    override val metadata: MetaData
) : Group(metadata), TopLevelGroup

internal data class ConjectureSection(override val metadata: MetaData) :
    Section("Conjecture", metadata)

internal data class ConjectureGroup(
    val id: Id,
    val conjectureSection: ConjectureSection,
    val givenSection: GivenSection?,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection?,
    val thenSection: ThenSection,
    val iffSection: IffSection?,
    val usingSection: UsingSection?,
    val metadataSection: MetadataSection?,
    override val metadata: MetaData
) : Group(metadata), TopLevelGroup

internal data class TheoremSection(override val metadata: MetaData) : Section("Theorem", metadata)

internal data class TheoremGroup(
    val id: Id,
    val theoremSection: TheoremSection,
    val givenSection: GivenSection?,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection?,
    val thenSection: ThenSection,
    val iffSection: IffSection?,
    val usingSection: UsingSection?,
    val proofSection: ProofSection?,
    val metadataSection: MetadataSection?,
    override val metadata: MetaData
) : Group(metadata), TopLevelGroup

internal data class ProofSection(val proofs: List<Text>, override val metadata: MetaData) :
    Section("Proof", metadata)

internal data class TopicSection(override val metadata: MetaData) : Section("Topic", metadata)

internal data class ContentSection(val content: Text, override val metadata: MetaData) :
    Section("content", metadata)

internal data class TopicGroup(
    val id: String,
    val topicSection: TopicSection,
    val contentSection: ContentSection,
    val metadataSection: MetadataSection?,
    override val metadata: MetaData
) : Group(metadata), TopLevelGroup

internal data class NoteTopLevelSection(override val metadata: MetaData) :
    Section("Note", metadata)

internal data class NoteTopLevelGroup(
    val noteSection: NoteTopLevelSection,
    val contentSection: ContentSection,
    val metadataSection: MetadataSection?,
    override val metadata: MetaData
) : Group(metadata), TopLevelGroup

internal interface SpecifyItem

internal data class SpecifySection(val items: List<SpecifyItem>, override val metadata: MetaData) :
    Section("Specify", metadata)

internal data class SpecifyGroup(
    val specifySection: SpecifySection, override val metadata: MetaData
) : Group(metadata), TopLevelGroup

internal data class ZeroSection(override val metadata: MetaData) : Section("zero", metadata)

internal data class ZeroGroup(
    val zeroSection: ZeroSection, val isSection: IsSection, override val metadata: MetaData
) : Group(metadata), SpecifyItem

internal data class PositiveIntSection(override val metadata: MetaData) :
    Section("positiveInt", metadata)

internal data class PositiveIntGroup(
    val positiveIntSection: PositiveIntSection,
    val isSection: IsSection,
    override val metadata: MetaData
) : Group(metadata), SpecifyItem

internal data class NegativeIntSection(override val metadata: MetaData) :
    Section("negativeInt", metadata)

internal data class NegativeIntGroup(
    val negativeIntSection: NegativeIntSection,
    val isSection: IsSection,
    override val metadata: MetaData
) : Group(metadata), SpecifyItem

internal data class PositiveFloatSection(override val metadata: MetaData) :
    Section("positiveFloat", metadata)

internal data class PositiveFloatGroup(
    val positiveFloatSection: PositiveFloatSection,
    val isSection: IsSection,
    override val metadata: MetaData
) : Group(metadata), SpecifyItem

internal data class NegativeFloatSection(override val metadata: MetaData) :
    Section("negativeFloat", metadata)

internal data class NegativeFloatGroup(
    val negativeFloatSection: NegativeFloatSection,
    val isSection: IsSection,
    override val metadata: MetaData
) : Group(metadata), SpecifyItem

internal data class IsSection(val form: Text, override val metadata: MetaData) :
    Section("is", metadata)

internal interface TopLevelGroup

internal interface TopLevelGroupOrTextBlock : TopLevelGroup

internal data class Document(val items: List<TopLevelGroupOrTextBlock>)

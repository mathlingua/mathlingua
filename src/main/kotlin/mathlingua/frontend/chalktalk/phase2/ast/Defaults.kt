/*
 * Copyright 2022 The MathLingua Authors
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

package mathlingua.frontend.chalktalk.phase2.ast

import mathlingua.frontend.chalktalk.phase1.ast.Abstraction
import mathlingua.frontend.chalktalk.phase1.ast.Assignment
import mathlingua.frontend.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Token
import mathlingua.frontend.chalktalk.phase1.ast.Tuple
import mathlingua.frontend.chalktalk.phase2.ast.clause.AbstractionNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.AssignmentNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.ClauseListNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Identifier
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Text
import mathlingua.frontend.chalktalk.phase2.ast.clause.TupleNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.have.HaveGroup
import mathlingua.frontend.chalktalk.phase2.ast.clause.have.HaveSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.ElseSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.IfGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.IfSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.ThenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.and.AndGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.and.AndSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.exists.ExistsGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.exists.ExistsSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.exists.SuchThatSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.existsUnique.ExistsUniqueGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.existsUnique.ExistsUniqueSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.forAll.ForAllGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.forAll.ForAllSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.generated.GeneratedFromSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.generated.GeneratedGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.generated.GeneratedSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.iff.IffGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.iff.IffSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.matching.MatchingGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.matching.MatchingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.matching.WithSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.not.NotGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.not.NotSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.or.OrGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.or.OrSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.piecewise.PiecewiseGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.piecewise.PiecewiseSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.CalledSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.MeansSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.ExpressingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.ProvidedSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.SatisfyingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.BySection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.ProvidingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.equality.BetweenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.equality.EqualityGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.equality.EqualitySection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.memberSymbols.MemberSymbolsGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.memberSymbols.MemberSymbolsSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.membership.MembershipGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.membership.MembershipSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.symbols.SymbolsGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.symbols.SymbolsSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.symbols.WhereSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.viewing.ThroughSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.viewing.ViaSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.viewing.ViewAsSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.viewing.ViewGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.viewing.ViewSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.ThatSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.note.NoteGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.note.NoteSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resource.ResourceGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resource.ResourceSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.ConjectureGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.ConjectureSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.newIfOrIffSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.GivenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.ProofSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.RelatedGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.ResourcesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.SiteGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.SourceItemGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.StringItem
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.StringSectionGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.TopicsGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.ContentItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.NameItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.OffsetItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.PageItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.RelatedSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.ResourcesSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.SiteItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.SourceItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.StringSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.TopicsSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.IsSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.SpecifyGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.SpecifySection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.negativeFloat.NegativeFloatGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.negativeFloat.NegativeFloatSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.negativeInt.NegativeIntGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.negativeInt.NegativeIntSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.positiveFloat.PositiveFloatGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.positiveFloat.PositiveFloatSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.positiveInt.PositiveIntGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.positiveInt.PositiveIntSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.zero.ZeroGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.specify.zero.ZeroSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topic.ContentSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topic.TopicGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topic.TopicSection
import mathlingua.frontend.support.ValidationFailure

internal val DEFAULT_ABSTRACTION =
    AbstractionNode(
        abstraction =
            Abstraction(
                isEnclosed = false,
                isVarArgs = false,
                parts = emptyList(),
                subParams = emptyList(),
                row = -1,
                column = -1),
        row = -1,
        column = -1,
        isInline = false)

internal val DEFAULT_TOKEN =
    Phase1Token(text = "INVALID", type = ChalkTalkTokenType.Invalid, row = -1, column = -1)

internal val DEFAULT_ASSIGNMENT =
    AssignmentNode(
        assignment = Assignment(lhs = DEFAULT_TOKEN, rhs = DEFAULT_TOKEN, row = -1, column = -1),
        row = -1,
        column = -1,
        isInline = false)

internal val DEFAULT_IDENTIFIER =
    Identifier(name = "INVALID", isVarArgs = false, row = -1, column = -1, isInline = false)

internal val DEFAULT_ID_STATEMENT =
    IdStatement(
        text = "INVALID", texTalkRoot = ValidationFailure(emptyList()), row = -1, column = -1)

internal val DEFAULT_STATEMENT =
    Statement(
        text = "INVALID",
        texTalkRoot = ValidationFailure(emptyList()),
        row = -1,
        column = -1,
        isInline = false)

internal val DEFAULT_TEXT = Text(text = "INVALID", row = -1, column = -1, isInline = false)

internal val DEFAULT_TUPLE =
    TupleNode(
        tuple = Tuple(items = emptyList(), row = -1, column = -1),
        row = -1,
        column = -1,
        isInline = false)

internal val DEFAULT_CLAUSE_LIST_NODE = ClauseListNode(clauses = emptyList(), row = -1, column = -1)

internal val DEFAULT_SUCH_THAT_SECTION =
    SuchThatSection(clauses = DEFAULT_CLAUSE_LIST_NODE, row = -1, column = -1)

internal val DEFAULT_ELSE_SECTION =
    ElseSection(clauses = DEFAULT_CLAUSE_LIST_NODE, row = -1, column = -1)

internal val DEFAULT_IF_SECTION =
    IfSection(clauses = DEFAULT_CLAUSE_LIST_NODE, row = -1, column = -1)

internal val DEFAULT_THEN_SECTION =
    ThenSection(clauses = DEFAULT_CLAUSE_LIST_NODE, row = -1, column = -1)

internal val DEFAULT_IFF_SECTION =
    IffSection(clauses = DEFAULT_CLAUSE_LIST_NODE, row = -1, column = -1)

internal val DEFAULT_NOT_SECTION =
    NotSection(clauses = DEFAULT_CLAUSE_LIST_NODE, row = -1, column = -1)

internal val DEFAULT_OR_SECTION =
    OrSection(clauses = DEFAULT_CLAUSE_LIST_NODE, row = -1, column = -1)

internal val DEFAULT_AND_SECTION =
    AndSection(clauses = DEFAULT_CLAUSE_LIST_NODE, row = -1, column = -1)

internal val DEFAULT_AND_GROUP = AndGroup(andSection = DEFAULT_AND_SECTION, row = -1, column = -1)

internal val DEFAULT_PIECEWISE_SECTION = PiecewiseSection(row = -1, column = -1)

internal val DEFAULT_NOTE_SECTION = NoteSection(row = -1, column = -1)

internal val DEFAULT_EXPRESSING_SECTION =
    ExpressingSection(clauses = DEFAULT_CLAUSE_LIST_NODE, row = -1, column = -1)

internal val DEFAULT_PROVIDED_SECTION =
    ProvidedSection(clauses = DEFAULT_CLAUSE_LIST_NODE, row = -1, column = -1)

internal val DEFAULT_THAT_SECTION =
    ThatSection(clauses = DEFAULT_CLAUSE_LIST_NODE, row = -1, column = -1)

internal val DEFAULT_USING_SECTION =
    UsingSection(clauses = DEFAULT_CLAUSE_LIST_NODE, row = -1, column = -1)

internal val DEFAULT_WHEN_SECTION =
    WhenSection(clauses = DEFAULT_CLAUSE_LIST_NODE, row = -1, column = -1)

internal val DEFAULT_WHERE_SECTION =
    WhereSection(clauses = DEFAULT_CLAUSE_LIST_NODE, row = -1, column = -1)

internal val DEFAULT_SATISFYING_SECTION =
    SatisfyingSection(clauses = DEFAULT_CLAUSE_LIST_NODE, row = -1, column = -1)

internal val DEFAULT_STATES_SECTION = StatesSection(row = -1, column = -1)

internal val DEFAULT_EXISTS_SECTION =
    ExistsSection(identifiers = emptyList(), row = -1, column = -1)

internal val DEFAULT_EXISTS_UNIQUE_SECTION =
    ExistsUniqueSection(identifiers = emptyList(), row = -1, column = -1)

internal val DEFAULT_EXISTS_GROUP =
    ExistsGroup(
        existsSection = DEFAULT_EXISTS_SECTION,
        whereSection = DEFAULT_WHERE_SECTION,
        suchThatSection = DEFAULT_SUCH_THAT_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_EXISTS_UNIQUE_GROUP =
    ExistsUniqueGroup(
        existsUniqueSection = DEFAULT_EXISTS_UNIQUE_SECTION,
        whereSection = DEFAULT_WHERE_SECTION,
        suchThatSection = DEFAULT_SUCH_THAT_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_FOR_ALL_SECTION = ForAllSection(targets = emptyList(), row = -1, column = -1)

internal val DEFAULT_DEFINES_SECTION = DefinesSection(targets = emptyList(), row = -1, column = -1)

internal val DEFAULT_WRITTEN_SECTION = WrittenSection(forms = emptyList(), row = -1, column = -1)

internal val DEFAULT_CALLED_SECTION = CalledSection(forms = emptyList(), row = -1, column = -1)

internal val DEFAULT_META_DATA_SECTION = MetaDataSection(items = emptyList(), row = -1, column = -1)

internal val DEFAULT_FOR_ALL_GROUP =
    ForAllGroup(
        forAllSection = DEFAULT_FOR_ALL_SECTION,
        whereSection = DEFAULT_WHERE_SECTION,
        suchThatSection = DEFAULT_SUCH_THAT_SECTION,
        thenSection = DEFAULT_THEN_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_IF_GROUP =
    IfGroup(
        ifSection = DEFAULT_IF_SECTION, thenSection = DEFAULT_THEN_SECTION, row = -1, column = -1)

internal val DEFAULT_IFF_GROUP =
    IffGroup(
        iffSection = DEFAULT_IFF_SECTION, thenSection = DEFAULT_THEN_SECTION, row = -1, column = -1)

internal val DEFAULT_NOT_GROUP = NotGroup(notSection = DEFAULT_NOT_SECTION, row = -1, column = -1)

internal val DEFAULT_OR_GROUP = OrGroup(orSection = DEFAULT_OR_SECTION, row = -1, column = -1)

internal val DEFAULT_GIVEN_SECTION = GivenSection(targets = listOf(), row = -1, column = -1)

internal val DEFAULT_STATES_GROUP =
    StatesGroup(
        signature = null,
        id = DEFAULT_ID_STATEMENT,
        statesSection = DEFAULT_STATES_SECTION,
        givenSection = DEFAULT_GIVEN_SECTION,
        whenSection = DEFAULT_WHEN_SECTION,
        thatSection = DEFAULT_THAT_SECTION,
        usingSection = DEFAULT_USING_SECTION,
        writtenSection = DEFAULT_WRITTEN_SECTION,
        calledSection = DEFAULT_CALLED_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_ENTRY_SECTION = TopicSection(names = emptyList(), row = -1, column = -1)

internal val DEFAULT_CONTENT_SECTION = ContentSection(text = "", row = -1, column = -1)

internal val DEFAULT_TOPIC_GROUP =
    TopicGroup(
        id = DEFAULT_ID_STATEMENT.text,
        topicSection = DEFAULT_ENTRY_SECTION,
        contentSection = DEFAULT_CONTENT_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_NOTE_GROUP =
    NoteGroup(
        id = DEFAULT_ID_STATEMENT.text,
        noteSection = DEFAULT_NOTE_SECTION,
        contentSection = DEFAULT_CONTENT_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_AXIOM_SECTION = AxiomSection(names = emptyList(), row = -1, column = -1)

internal val DEFAULT_CONJECTURE_SECTION =
    ConjectureSection(names = emptyList(), row = -1, column = -1)

internal val DEFAULT_THEOREM_SECTION = TheoremSection(names = emptyList(), row = -1, column = -1)

internal val DEFAULT_EXTENDING_SECTION =
    MeansSection(statements = emptyList(), row = -1, column = -1)

internal val DEFAULT_PROOF_SECTION = ProofSection(text = "", row = -1, column = -1)

internal val DEFAULT_IF_OR_IFF_SECTION = newIfOrIffSection(DEFAULT_IF_SECTION)

internal val DEFAULT_AXIOM_GROUP =
    AxiomGroup(
        signature = null,
        id = DEFAULT_ID_STATEMENT,
        axiomSection = DEFAULT_AXIOM_SECTION,
        givenSection = DEFAULT_GIVEN_SECTION,
        whenSection = DEFAULT_WHEN_SECTION,
        thenSection = DEFAULT_THEN_SECTION,
        iffSection = DEFAULT_IFF_SECTION,
        usingSection = DEFAULT_USING_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_CONJECTURE_GROUP =
    ConjectureGroup(
        signature = null,
        id = DEFAULT_ID_STATEMENT,
        conjectureSection = DEFAULT_CONJECTURE_SECTION,
        givenSection = DEFAULT_GIVEN_SECTION,
        whenSection = DEFAULT_WHEN_SECTION,
        thenSection = DEFAULT_THEN_SECTION,
        iffSection = DEFAULT_IFF_SECTION,
        usingSection = DEFAULT_USING_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_THEOREM_GROUP =
    TheoremGroup(
        signature = null,
        id = DEFAULT_ID_STATEMENT,
        theoremSection = DEFAULT_THEOREM_SECTION,
        givenSection = DEFAULT_GIVEN_SECTION,
        whenSection = DEFAULT_WHEN_SECTION,
        thenSection = DEFAULT_THEN_SECTION,
        iffSection = DEFAULT_IFF_SECTION,
        usingSection = DEFAULT_USING_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION,
        proofSection = DEFAULT_PROOF_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_RESOURCE_SECTION = ResourceSection(items = emptyList(), row = -1, column = -1)

internal val DEFAULT_RESOURCE_GROUP =
    ResourceGroup(
        id = "",
        resourceSection = DEFAULT_RESOURCE_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_PIECEWISE_GROUP =
    PiecewiseGroup(
        piecewiseSection = DEFAULT_PIECEWISE_SECTION,
        whenThen = emptyList(),
        elseSection = DEFAULT_ELSE_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_SOURCE_ITEM_SECTION =
    SourceItemSection(sourceReference = "", row = -1, column = -1)

internal val DEFAULT_CONTENT_ITEM_SECTION = ContentItemSection(content = "", row = -1, column = -1)

internal val DEFAULT_OFFSET_ITEM_SECTION = OffsetItemSection(offset = "", row = -1, column = -1)

internal val DEFAULT_SITE_ITEM_SECTION = SiteItemSection(url = "", row = -1, column = -1)

internal val DEFAULT_NAME_ITEM_SECTION = NameItemSection(name = "", row = -1, column = -1)

internal val DEFAULT_SITE_GROUP =
    SiteGroup(
        siteItemSection = DEFAULT_SITE_ITEM_SECTION,
        nameItemSection = DEFAULT_NAME_ITEM_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_PAGE_ITEM_SECTION = PageItemSection(page = "", row = -1, column = -1)

internal val DEFAULT_RESOURCES_SECTION =
    ResourcesSection(items = emptyList(), row = -1, column = -1)

internal val DEFAULT_TOPICS_SECTION = TopicsSection(items = emptyList(), row = -1, column = -1)

internal val DEFAULT_RELATED_SECTION = RelatedSection(items = emptyList(), row = -1, column = -1)

internal val DEFAULT_RESOURCES_GROUP =
    ResourcesGroup(resourcesSection = DEFAULT_RESOURCES_SECTION, row = -1, column = -1)

internal val DEFAULT_TOPICS_GROUP =
    TopicsGroup(topicsSection = DEFAULT_TOPICS_SECTION, row = -1, column = -1)

internal val DEFAULT_RELATED_GROUP =
    RelatedGroup(relatedSection = DEFAULT_RELATED_SECTION, row = -1, column = -1)

internal val DEFAULT_STRING_SECTION =
    StringSection(name = "", values = emptyList(), row = -1, column = -1)

internal val DEFAULT_STRING_RESOURCE_ITEM = StringItem(text = "", row = -1, column = -1)

internal val DEFAULT_SOURCE_ITEM_GROUP =
    SourceItemGroup(
        sourceSection = DEFAULT_SOURCE_ITEM_SECTION,
        pageSection = DEFAULT_PAGE_ITEM_SECTION,
        offsetSection = DEFAULT_OFFSET_ITEM_SECTION,
        contentSection = DEFAULT_CONTENT_ITEM_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_META_DATA_ITEM =
    StringSectionGroup(
        section = StringSection(name = "", values = emptyList(), row = -1, column = -1),
        row = -1,
        column = -1)

internal val DEFAULT_VIEW_AS_SECTION =
    ViewAsSection(statement = DEFAULT_STATEMENT, row = -1, column = -1)

internal val DEFAULT_VIA_SECTION = ViaSection(statement = DEFAULT_STATEMENT, row = -1, column = -1)

internal val DEFAULT_BY_SECTION = BySection(statement = DEFAULT_STATEMENT, row = -1, column = -1)

internal val DEFAULT_THROUGH_SECTION =
    ThroughSection(statement = DEFAULT_STATEMENT, row = -1, column = -1)

internal val DEFAULT_MEMBERSHIP_SECTION = MembershipSection(row = -1, column = -1)

internal val DEFAULT_MEMBERSHIP_GROUP =
    MembershipGroup(
        membershipSection = DEFAULT_MEMBERSHIP_SECTION,
        throughSection = DEFAULT_THROUGH_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_VIEW_SECTION = ViewSection(row = -1, column = -1)

internal val DEFAULT_VIEW_GROUP =
    ViewGroup(
        viewSection = DEFAULT_VIEW_SECTION,
        viewAsSection = DEFAULT_VIEW_AS_SECTION,
        viaSection = DEFAULT_VIA_SECTION,
        bySection = DEFAULT_BY_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_HAVE_SECTION =
    HaveSection(clauses = DEFAULT_CLAUSE_LIST_NODE, row = -1, column = -1)

internal val DEFAULT_HAVE_GROUP =
    HaveGroup(
        haveSection = DEFAULT_HAVE_SECTION, bySection = DEFAULT_BY_SECTION, row = -1, column = -1)

internal val DEFAULT_PROVIDING_SECTION =
    ProvidingSection(clauses = DEFAULT_CLAUSE_LIST_NODE, row = -1, column = -1)

internal val DEFAULT_EQUALITY_SECTION = EqualitySection(row = -1, column = -1)

internal val DEFAULT_BETWEEN_SECTION = BetweenSection(emptyList(), row = -1, column = -1)

internal val DEFAULT_EQUALITY_GROUP =
    EqualityGroup(
        equalitySection = DEFAULT_EQUALITY_SECTION,
        betweenSection = DEFAULT_BETWEEN_SECTION,
        providedSection = DEFAULT_PROVIDED_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_DEFINES_GROUP =
    DefinesGroup(
        signature = null,
        id = DEFAULT_ID_STATEMENT,
        definesSection = DEFAULT_DEFINES_SECTION,
        givenSection = DEFAULT_GIVEN_SECTION,
        whenSection = DEFAULT_WHEN_SECTION,
        meansSection = DEFAULT_EXTENDING_SECTION,
        satisfyingSection = DEFAULT_SATISFYING_SECTION,
        expressingSection = DEFAULT_EXPRESSING_SECTION,
        providingSection = DEFAULT_PROVIDING_SECTION,
        usingSection = DEFAULT_USING_SECTION,
        writtenSection = DEFAULT_WRITTEN_SECTION,
        calledSection = DEFAULT_CALLED_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_GENERATED_SECTION = GeneratedSection(row = -1, column = -1)

internal val DEFAULT_GENERATED_FROM_SECTION =
    GeneratedFromSection(forms = emptyList(), row = -1, column = -1)

internal val DEFAULT_GENERATED_GROUP =
    GeneratedGroup(
        generatedSection = DEFAULT_GENERATED_SECTION,
        generatedFromSection = DEFAULT_GENERATED_FROM_SECTION,
        whenSection = DEFAULT_WHEN_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_WITH_SECTION =
    WithSection(clauses = DEFAULT_CLAUSE_LIST_NODE, row = -1, column = -1)

internal val DEFAULT_MATCHING_SECTION =
    MatchingSection(targets = emptyList(), row = -1, column = -1)

internal val DEFAULT_MATCHING_GROUP =
    MatchingGroup(
        matchingSection = DEFAULT_MATCHING_SECTION,
        withSection = DEFAULT_WITH_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_NEGATIVE_FLOAT_SECTION = NegativeFloatSection(row = -1, column = -1)

internal val DEFAULT_POSITIVE_FLOAT_SECTION = PositiveFloatSection(row = -1, column = -1)

internal val DEFAULT_POSITIVE_INT_SECTION = PositiveIntSection(row = -1, column = -1)

internal val DEFAULT_NEGATIVE_INT_SECTION = NegativeIntSection(row = -1, column = -1)

internal val DEFAULT_ZERO_SECTION = ZeroSection(row = -1, column = -1)

internal val DEFAULT_IS_SECTION = IsSection(statement = DEFAULT_STATEMENT, row = -1, column = -1)

internal val DEFAULT_ZERO_GROUP =
    ZeroGroup(
        zeroSection = DEFAULT_ZERO_SECTION, isSection = DEFAULT_IS_SECTION, row = -1, column = -1)

internal val DEFAULT_POSITIVE_INT_GROUP =
    PositiveIntGroup(
        positiveIntSection = DEFAULT_POSITIVE_INT_SECTION,
        isSection = DEFAULT_IS_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_NEGATIVE_INT_GROUP =
    NegativeIntGroup(
        negativeIntSection = DEFAULT_NEGATIVE_INT_SECTION,
        isSection = DEFAULT_IS_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_POSITIVE_FLOAT_GROUP =
    PositiveFloatGroup(
        positiveFloatSection = DEFAULT_POSITIVE_FLOAT_SECTION,
        isSection = DEFAULT_IS_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_NEGATIVE_FLOAT_GROUP =
    NegativeFloatGroup(
        negativeFloatSection = DEFAULT_NEGATIVE_FLOAT_SECTION,
        isSection = DEFAULT_IS_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_SPECIFY_SECTION =
    SpecifySection(numberGroups = emptyList(), row = -1, column = -1)

internal val DEFAULT_SPECIFY_GROUP =
    SpecifyGroup(specifySection = DEFAULT_SPECIFY_SECTION, row = -1, column = -1)

internal val DEFAULT_SYMBOLS_SECTION = SymbolsSection(targets = emptyList(), row = -1, column = -1)

internal val DEFAULT_SYMBOLS_GROUP =
    SymbolsGroup(
        symbolsSection = DEFAULT_SYMBOLS_SECTION,
        whereSection = DEFAULT_WHERE_SECTION,
        row = -1,
        column = -1)

internal val DEFAULT_MEMBER_SYMBOLS_SECTION =
    MemberSymbolsSection(targets = emptyList(), row = -1, column = -1)

internal val DEFAULT_MEMBER_SYMBOLS_GROUP =
    MemberSymbolsGroup(
        memberSymbolsSection = DEFAULT_MEMBER_SYMBOLS_SECTION,
        whereSection = DEFAULT_WHERE_SECTION,
        row = -1,
        column = -1)

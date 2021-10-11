/*
 * Copyright 2019 The MathLingua Authors
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
import mathlingua.frontend.chalktalk.phase1.ast.Group
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Token
import mathlingua.frontend.chalktalk.phase1.ast.Section
import mathlingua.frontend.chalktalk.phase1.ast.Tuple
import mathlingua.frontend.chalktalk.phase1.ast.getColumn
import mathlingua.frontend.chalktalk.phase1.ast.getRow
import mathlingua.frontend.chalktalk.phase2.ast.clause.AbstractionNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.AssignmentNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.ClauseListNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Identifier
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Target
import mathlingua.frontend.chalktalk.phase2.ast.clause.Text
import mathlingua.frontend.chalktalk.phase2.ast.clause.TupleNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.validateClause
import mathlingua.frontend.chalktalk.phase2.ast.clause.validateIdStatement
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
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
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.iff.IffGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.iff.IffSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.not.NotGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.not.NotSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.or.OrGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.or.OrSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.piecewise.PiecewiseGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.piecewise.PiecewiseSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.CalledSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.EvaluatedSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.MeansSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.ProvidedSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.RequiringSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.ThatSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.BySection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.ViewingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.equality.BetweenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.equality.EqualityGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.equality.EqualitySection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.membership.MembershipGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.membership.MembershipSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.viewingas.ThroughSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.viewingas.ViaSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.viewingas.ViewingAsGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.viewingas.ViewingAsSection
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
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topic.ContentSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topic.TopicGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topic.TopicSection
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.validationFailure

fun <T : Phase2Node> track(node: Phase1Node, tracker: MutableLocationTracker, builder: () -> T): T {
    val phase2Node = builder()
    tracker.setLocationOf(phase2Node, Location(row = getRow(node), column = getColumn(node)))
    return phase2Node
}

fun <T, U> validateByTransform(
    node: Phase1Node,
    errors: MutableList<ParseError>,
    default: T,
    message: String,
    transform: (node: Phase1Node) -> U?,
    builder: (node: U) -> T
): T {
    val newNode = transform(node)
    return if (newNode != null) {
        builder(newNode)
    } else {
        errors.add(ParseError(message = message, row = getRow(node), column = getColumn(node)))
        default
    }
}

fun <T> validateGroup(
    node: Phase1Node,
    errors: MutableList<ParseError>,
    name: String,
    default: T,
    builder: (group: Group) -> T
) =
    validateByTransform(
        node,
        errors,
        default,
        "Expected group '$name'",
        {
            if (it is Group && it.sections.isNotEmpty() && it.sections[0].name.text == name) {
                it
            } else {
                null
            }
        },
        builder)

fun <T> validateSection(
    node: Phase1Node,
    errors: MutableList<ParseError>,
    name: String,
    default: T,
    builder: (section: Section) -> T
) = validateSectionImpl(node, errors, name, default, builder)

fun <T> validateSection(
    node: Phase1Node, errors: MutableList<ParseError>, default: T, builder: (section: Section) -> T
) = validateSectionImpl(node, errors, null, default, builder)

private fun <T> validateSectionImpl(
    node: Phase1Node,
    errors: MutableList<ParseError>,
    name: String?,
    default: T,
    builder: (section: Section) -> T
) =
    validateByTransform(
        node,
        errors,
        default,
        if (name == null) {
            "Expected a section but found $node"
        } else {
            "Expected a section '$name'"
        },
        {
            if (it is Section && (name == null || it.name.text == name)) {
                it
            } else {
                null
            }
        },
        builder)

fun <T> validateTargetSection(
    node: Phase1Node,
    errors: MutableList<ParseError>,
    name: String,
    default: T,
    tracker: MutableLocationTracker,
    builder: (targets: List<Target>) -> T
) =
    validateSection(node, errors, name, default) { section ->
        if (section.args.isEmpty()) {
            errors.add(
                ParseError(
                    message = "Section '$name' requires at least one argument.",
                    row = getRow(section),
                    column = getColumn(section)))
            default
        } else {
            val targets = mutableListOf<Target>()
            for (arg in section.args) {
                var shouldContinue = false
                val clause = validateClause(arg, errors, tracker)
                if (clause is Target) {
                    targets.add(clause)
                    shouldContinue = true
                }

                if (shouldContinue) {
                    continue
                }

                errors.add(ParseError("Expected an Target", getRow(arg), getColumn(arg)))
            }
            builder(targets.toList())
        }
    }

fun getOptionalId(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
): IdStatement? {
    val group = node.resolve()
    return if (group is Group && group.id != null) {
        val (rawText, _, row, column) = group.id
        // The id token is of type Id and the text is of the form "[...]"
        // Convert it to look like a statement.
        val statementText = "'" + rawText.substring(1, rawText.length - 1) + "'"
        val stmtToken = Phase1Token(statementText, ChalkTalkTokenType.Statement, row, column)
        validateIdStatement(stmtToken, errors, tracker)
    } else {
        null
    }
}

fun getId(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
): IdStatement {
    val id = getOptionalId(node, errors, tracker)
    return if (id != null) {
        id
    } else {
        errors.add(ParseError("Expected an Id", getRow(node), getColumn(node)))
        DEFAULT_ID_STATEMENT
    }
}

fun <T> validateSingleArg(
    section: Section,
    errors: MutableList<ParseError>,
    default: T,
    type: String,
    builder: (arg: Phase1Node) -> T
) =
    if (section.args.size != 1) {
        errors.add(
            ParseError(
                message = "Expected a single $type argument",
                row = getRow(section),
                column = getColumn(section)))
        default
    } else {
        builder(section.args[0])
    }

val DEFAULT_ABSTRACTION =
    AbstractionNode(
        abstraction =
            Abstraction(
                isEnclosed = false,
                isVarArgs = false,
                parts = emptyList(),
                subParams = emptyList()))

val DEFAULT_TOKEN =
    Phase1Token(text = "INVALID", type = ChalkTalkTokenType.Invalid, row = -1, column = -1)

val DEFAULT_ASSIGNMENT =
    AssignmentNode(assignment = Assignment(lhs = DEFAULT_TOKEN, rhs = DEFAULT_TOKEN))

val DEFAULT_IDENTIFIER = Identifier(name = "INVALID", isVarArgs = false)

val DEFAULT_ID_STATEMENT =
    IdStatement(text = "INVALID", texTalkRoot = validationFailure(emptyList()))

val DEFAULT_STATEMENT = Statement(text = "INVALID", texTalkRoot = validationFailure(emptyList()))

val DEFAULT_TEXT = Text(text = "INVALID")

val DEFAULT_TUPLE = TupleNode(tuple = Tuple(items = emptyList()))

val DEFAULT_CLAUSE_LIST_NODE = ClauseListNode(clauses = emptyList())

val DEFAULT_SUCH_THAT_SECTION = SuchThatSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_ELSE_SECTION = ElseSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_IF_SECTION = IfSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_THEN_SECTION = ThenSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_IFF_SECTION = IffSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_NOT_SECTION = NotSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_OR_SECTION = OrSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_AND_SECTION = AndSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_AND_GROUP = AndGroup(andSection = DEFAULT_AND_SECTION)

val DEFAULT_PIECEWISE_SECTION = PiecewiseSection()

val DEFAULT_NOTE_SECTION = NoteSection()

val DEFAULT_EVALUATED_SECTION = EvaluatedSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_PROVIDED_SECTION = ProvidedSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_THAT_SECTION = ThatSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_USING_SECTION = UsingSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_WHEN_SECTION = WhenSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_MEANS_SECTION = MeansSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_STATES_SECTION = StatesSection()

val DEFAULT_EXISTS_SECTION = ExistsSection(identifiers = emptyList())

val DEFAULT_EXISTS_UNIQUE_SECTION = ExistsUniqueSection(identifiers = emptyList())

val DEFAULT_EXISTS_GROUP =
    ExistsGroup(existsSection = DEFAULT_EXISTS_SECTION, suchThatSection = DEFAULT_SUCH_THAT_SECTION)

val DEFAULT_EXISTS_UNIQUE_GROUP =
    ExistsUniqueGroup(
        existsUniqueSection = DEFAULT_EXISTS_UNIQUE_SECTION,
        suchThatSection = DEFAULT_SUCH_THAT_SECTION)

val DEFAULT_FOR_ALL_SECTION = ForAllSection(targets = emptyList())

val DEFAULT_DEFINES_SECTION = DefinesSection(targets = emptyList())

val DEFAULT_WRITTEN_SECTION = WrittenSection(forms = emptyList())

val DEFAULT_CALLED_SECTION = CalledSection(forms = emptyList())

val DEFAULT_META_DATA_SECTION = MetaDataSection(items = emptyList())

val DEFAULT_FOR_ALL_GROUP =
    ForAllGroup(
        forAllSection = DEFAULT_FOR_ALL_SECTION,
        suchThatSection = DEFAULT_SUCH_THAT_SECTION,
        thenSection = DEFAULT_THEN_SECTION)

val DEFAULT_IF_GROUP = IfGroup(ifSection = DEFAULT_IF_SECTION, thenSection = DEFAULT_THEN_SECTION)

val DEFAULT_IFF_GROUP =
    IffGroup(iffSection = DEFAULT_IFF_SECTION, thenSection = DEFAULT_THEN_SECTION)

val DEFAULT_NOT_GROUP = NotGroup(notSection = DEFAULT_NOT_SECTION)

val DEFAULT_OR_GROUP = OrGroup(orSection = DEFAULT_OR_SECTION)

val DEFAULT_REQUIRING_SECTION = RequiringSection(targets = listOf())

val DEFAULT_STATES_GROUP =
    StatesGroup(
        signature = null,
        id = DEFAULT_ID_STATEMENT,
        statesSection = DEFAULT_STATES_SECTION,
        requiringSection = DEFAULT_REQUIRING_SECTION,
        whenSection = DEFAULT_WHEN_SECTION,
        thatSection = DEFAULT_THAT_SECTION,
        usingSection = DEFAULT_USING_SECTION,
        writtenSection = DEFAULT_WRITTEN_SECTION,
        calledSection = DEFAULT_CALLED_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION)

val DEFAULT_ENTRY_SECTION = TopicSection(names = emptyList())

val DEFAULT_CONTENT_SECTION = ContentSection(text = "")

val DEFAULT_TOPIC_GROUP =
    TopicGroup(
        id = DEFAULT_ID_STATEMENT.text,
        topicSection = DEFAULT_ENTRY_SECTION,
        contentSection = DEFAULT_CONTENT_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION)

val DEFAULT_NOTE_GROUP =
    NoteGroup(
        id = DEFAULT_ID_STATEMENT.text,
        noteSection = DEFAULT_NOTE_SECTION,
        contentSection = DEFAULT_CONTENT_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION)

val DEFAULT_AXIOM_SECTION = AxiomSection(names = emptyList())

val DEFAULT_CONJECTURE_SECTION = ConjectureSection(names = emptyList())

val DEFAULT_THEOREM_SECTION = TheoremSection(names = emptyList())

val DEFAULT_PROOF_SECTION = ProofSection(text = "")

val DEFAULT_GIVEN_SECTION = GivenSection(targets = listOf())

val DEFAULT_IF_OR_IFF_SECTION = newIfOrIffSection(DEFAULT_IF_SECTION)

val DEFAULT_AXIOM_GROUP =
    AxiomGroup(
        signature = null,
        id = DEFAULT_ID_STATEMENT,
        axiomSection = DEFAULT_AXIOM_SECTION,
        givenSection = DEFAULT_GIVEN_SECTION,
        ifOrIffSection = DEFAULT_IF_OR_IFF_SECTION,
        thenSection = DEFAULT_THEN_SECTION,
        usingSection = DEFAULT_USING_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION)

val DEFAULT_CONJECTURE_GROUP =
    ConjectureGroup(
        signature = null,
        id = DEFAULT_ID_STATEMENT,
        conjectureSection = DEFAULT_CONJECTURE_SECTION,
        givenSection = DEFAULT_GIVEN_SECTION,
        ifOrIffSection = DEFAULT_IF_OR_IFF_SECTION,
        thenSection = DEFAULT_THEN_SECTION,
        usingSection = DEFAULT_USING_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION)

val DEFAULT_THEOREM_GROUP =
    TheoremGroup(
        signature = null,
        id = DEFAULT_ID_STATEMENT,
        theoremSection = DEFAULT_THEOREM_SECTION,
        givenSection = DEFAULT_GIVEN_SECTION,
        ifOrIffSection = DEFAULT_IF_OR_IFF_SECTION,
        thenSection = DEFAULT_THEN_SECTION,
        usingSection = DEFAULT_USING_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION,
        proofSection = DEFAULT_PROOF_SECTION)

val DEFAULT_RESOURCE_SECTION = ResourceSection(items = emptyList())

val DEFAULT_RESOURCE_GROUP =
    ResourceGroup(
        id = "",
        resourceSection = DEFAULT_RESOURCE_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION)

val DEFAULT_PIECEWISE_GROUP =
    PiecewiseGroup(
        piecewiseSection = DEFAULT_PIECEWISE_SECTION,
        whenThen = emptyList(),
        elseSection = DEFAULT_ELSE_SECTION)

val DEFAULT_SOURCE_ITEM_SECTION = SourceItemSection(sourceReference = "")

val DEFAULT_CONTENT_ITEM_SECTION = ContentItemSection(content = "")

val DEFAULT_OFFSET_ITEM_SECTION = OffsetItemSection(offset = "")

val DEFAULT_SITE_ITEM_SECTION = SiteItemSection(url = "")

val DEFAULT_NAME_ITEM_SECTION = NameItemSection(name = "")

val DEFAULT_SITE_GROUP =
    SiteGroup(
        siteItemSection = DEFAULT_SITE_ITEM_SECTION, nameItemSection = DEFAULT_NAME_ITEM_SECTION)

val DEFAULT_PAGE_ITEM_SECTION = PageItemSection(page = "")

val DEFAULT_RESOURCES_SECTION = ResourcesSection(items = emptyList())

val DEFAULT_TOPICS_SECTION = TopicsSection(items = emptyList())

val DEFAULT_RELATED_SECTION = RelatedSection(items = emptyList())

val DEFAULT_RESOURCES_GROUP = ResourcesGroup(resourcesSection = DEFAULT_RESOURCES_SECTION)

val DEFAULT_TOPICS_GROUP = TopicsGroup(topicsSection = DEFAULT_TOPICS_SECTION)

val DEFAULT_RELATED_GROUP = RelatedGroup(relatedSection = DEFAULT_RELATED_SECTION)

val DEFAULT_STRING_SECTION = StringSection(name = "", values = emptyList())

val DEFAULT_STRING_RESOURCE_ITEM = StringItem(text = "")

val DEFAULT_SOURCE_ITEM_GROUP =
    SourceItemGroup(
        sourceSection = DEFAULT_SOURCE_ITEM_SECTION,
        pageSection = DEFAULT_PAGE_ITEM_SECTION,
        offsetSection = DEFAULT_OFFSET_ITEM_SECTION,
        contentSection = DEFAULT_CONTENT_ITEM_SECTION)

val DEFAULT_META_DATA_ITEM =
    StringSectionGroup(section = StringSection(name = "", values = emptyList()))

val DEFAULT_VIEWED_AS_SECTION = ViewingAsSection(statement = DEFAULT_STATEMENT)

val DEFAULT_VIA_SECTION = ViaSection(statement = DEFAULT_STATEMENT)

val DEFAULT_BY_SECTION = BySection(statement = DEFAULT_STATEMENT)

val DEFAULT_THROUGH_SECTION = ThroughSection(statement = DEFAULT_STATEMENT)

val DEFAULT_MEMBERSHIP_SECTION = MembershipSection()

val DEFAULT_MEMBERSHIP_GROUP =
    MembershipGroup(
        membershipSection = DEFAULT_MEMBERSHIP_SECTION, throughSection = DEFAULT_THROUGH_SECTION)

val DEFAULT_VIEWED_AS_GROUP =
    ViewingAsGroup(
        viewingAsSection = DEFAULT_VIEWED_AS_SECTION,
        viaSection = DEFAULT_VIA_SECTION,
        bySection = DEFAULT_BY_SECTION)

val DEFAULT_VIEWING_SECTION = ViewingSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_EQUALITY_SECTION = EqualitySection()

val DEFAULT_BETWEEN_SECTION = BetweenSection(emptyList())

val DEFAULT_EQUALITY_GROUP =
    EqualityGroup(
        equalitySection = DEFAULT_EQUALITY_SECTION,
        betweenSection = DEFAULT_BETWEEN_SECTION,
        providedSection = DEFAULT_PROVIDED_SECTION)

val DEFAULT_DEFINES_GROUP =
    DefinesGroup(
        signature = null,
        id = DEFAULT_ID_STATEMENT,
        definesSection = DEFAULT_DEFINES_SECTION,
        requiringSection = DEFAULT_REQUIRING_SECTION,
        whenSection = DEFAULT_WHEN_SECTION,
        meansSection = DEFAULT_MEANS_SECTION,
        evaluatedSection = DEFAULT_EVALUATED_SECTION,
        viewingSection = DEFAULT_VIEWING_SECTION,
        usingSection = DEFAULT_USING_SECTION,
        writtenSection = DEFAULT_WRITTEN_SECTION,
        calledSection = DEFAULT_CALLED_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION)

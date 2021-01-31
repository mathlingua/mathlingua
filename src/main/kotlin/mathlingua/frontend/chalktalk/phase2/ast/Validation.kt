/*
 * Copyright 2019 Google LLC
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
import mathlingua.frontend.chalktalk.phase1.ast.Mapping
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
import mathlingua.frontend.chalktalk.phase2.ast.clause.MappingNode
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
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.collection.CollectionGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.collection.CollectionSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.collection.InSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.collection.OfSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.exists.ExistsGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.exists.ExistsSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.exists.SuchThatSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.expands.AsSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.expands.ExpandsGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.expands.ExpandsSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.forAll.ForAllGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.forAll.ForAllSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.from.FromGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.given.AllSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.given.GivenGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.iff.IffGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.iff.IffSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.inductively.ConstantGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.inductively.ConstantSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.inductively.ConstructorGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.inductively.ConstructorSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.inductively.InductivelyFromSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.inductively.InductivelyGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.inductively.InductivelySection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.mapping.FromSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.mapping.MappingGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.mapping.MappingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.matching.MatchingGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.matching.MatchingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.not.NotGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.not.NotSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.or.OrGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.or.OrSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.piecewise.PiecewiseGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.piecewise.PiecewiseSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.CollectsSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesCollectsGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesEvaluatedGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGeneratedGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesInstantiatedGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesMapsGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesMeansGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.EvaluatedSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.GeneratedSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.InstantiatedSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.MapsSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.MeansSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.ProvidedSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.RequiringSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.SpecifiesSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.evaluates.EvaluatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.evaluates.EvaluatesSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.FoundationGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.FoundationSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.mutually.MutuallyGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.mutually.MutuallySection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.ThatSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.views.SingleAsSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.views.SingleFromSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.views.SingleToSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.views.ViewsGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.views.ViewsSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.entry.ContentSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.entry.EntryGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.entry.EntrySection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.entry.TypeSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resource.ResourceGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resource.ResourceSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.ConjectureGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.ConjectureSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.newIfOrIffSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.GivenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.WhereSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.ReferenceGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.SourceItemGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.StringSectionGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.ContentItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.OffsetItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.PageItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.ReferenceSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.SourceItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.StringSection
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
    node: Phase1Node,
    errors: MutableList<ParseError>,
    default: IdStatement,
    tracker: MutableLocationTracker
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

val DEFAULT_MAPPING_NODE = MappingNode(mapping = Mapping(lhs = DEFAULT_TOKEN, rhs = DEFAULT_TOKEN))

val DEFAULT_TEXT = Text(text = "INVALID")

val DEFAULT_TUPLE = TupleNode(tuple = Tuple(items = emptyList()))

val DEFAULT_CLAUSE_LIST_NODE = ClauseListNode(clauses = emptyList())

val DEFAULT_SUCH_THAT_SECTION = SuchThatSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_ELSE_SECTION = ElseSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_IF_SECTION = IfSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_THEN_SECTION = ThenSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_AS_SECTION = AsSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_IFF_SECTION = IffSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_CONSTANT_SECTION = ConstantSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_COLLECTION_SECTION = CollectionSection()

val DEFAULT_INDUCTIVELY_FROM_SECTION = InductivelyFromSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_INDUCTIVELY_SECTION = InductivelySection()

val DEFAULT_MAPPING_SECTION = MappingSection()

val DEFAULT_MATCHING_SECTION = MatchingSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_NOT_SECTION = NotSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_OR_SECTION = OrSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_AND_SECTION = AndSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_AND_GROUP = AndGroup(andSection = DEFAULT_AND_SECTION)

val DEFAULT_PIECEWISE_SECTION = PiecewiseSection()

val DEFAULT_EVALUATED_SECTION = EvaluatedSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_PROVIDED_SECTION = ProvidedSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_THAT_SECTION = ThatSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_SINGLE_TO_SECTION = SingleToSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_USING_SECTION = UsingSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_WHEN_SECTION = WhenSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_WHERE_SECTION = WhereSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_SPECIFIES_SECTION = SpecifiesSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_ALL_SECTION = AllSection(statement = DEFAULT_STATEMENT)

val DEFAULT_OF_SECTION = OfSection(statement = DEFAULT_STATEMENT)

val DEFAULT_FROM_SECTION = FromSection(statements = emptyList())

val DEFAULT_TO_SECTION =
    mathlingua.frontend.chalktalk.phase2.ast.group.clause.mapping.ToSection(
        statements = emptyList())

val DEFAULT_MEANS_SECTION = MeansSection(clauses = DEFAULT_CLAUSE_LIST_NODE)

val DEFAULT_INSTANTIATED_SECTION = InstantiatedSection(statements = emptyList())

val DEFAULT_STATES_SECTION = StatesSection()

val DEFAULT_SINGLE_AS_SECTION = SingleAsSection(statement = DEFAULT_STATEMENT)

val DEFAULT_SINGLE_FROM_SECTION = SingleFromSection(statement = DEFAULT_STATEMENT)

val DEFAULT_EXISTS_SECTION = ExistsSection(identifiers = emptyList())

val DEFAULT_EXISTS_GROUP =
    ExistsGroup(
        existsSection = DEFAULT_EXISTS_SECTION,
        whereSection = null,
        suchThatSection = DEFAULT_SUCH_THAT_SECTION)

val DEFAULT_IN_SECTION = InSection(statement = DEFAULT_STATEMENT)

val DEFAULT_FOR_ALL_SECTION = ForAllSection(targets = emptyList())

val DEFAULT_CONSTRUCTOR_SECTION = ConstructorSection(targets = emptyList())

val DEFAULT_DEFINES_SECTION = DefinesSection(targets = emptyList())

val DEFAULT_EVALUATES_SECTION = EvaluatesSection()

val DEFAULT_WRITTEN_SECTION = WrittenSection(forms = emptyList())

val DEFAULT_META_DATA_SECTION = MetaDataSection(items = emptyList())

val DEFAULT_MUTUALLY_SECTION = MutuallySection(items = emptyList())

val DEFAULT_VIEWS_SECTION = ViewsSection(targets = emptyList())

val DEFAULT_COLLECTION_GROUP =
    CollectionGroup(
        collectionSection = DEFAULT_COLLECTION_SECTION,
        ofSection = DEFAULT_OF_SECTION,
        inSection = DEFAULT_IN_SECTION,
        forAllSection = DEFAULT_FOR_ALL_SECTION,
        whereSection = DEFAULT_WHERE_SECTION)

val DEFAULT_EXPANDS_SECTION = ExpandsSection(targets = emptyList())

val DEFAULT_EXPANDS_GROUP =
    ExpandsGroup(expandsSection = DEFAULT_EXPANDS_SECTION, asSection = DEFAULT_AS_SECTION)

val DEFAULT_FOR_ALL_GROUP =
    ForAllGroup(
        forAllSection = DEFAULT_FOR_ALL_SECTION,
        whereSection = DEFAULT_WHERE_SECTION,
        suchThatSection = DEFAULT_SUCH_THAT_SECTION,
        thenSection = DEFAULT_THEN_SECTION)

val DEFAULT_IF_GROUP = IfGroup(ifSection = DEFAULT_IF_SECTION, thenSection = DEFAULT_THEN_SECTION)

val DEFAULT_IFF_GROUP =
    IffGroup(iffSection = DEFAULT_IFF_SECTION, thenSection = DEFAULT_THEN_SECTION)

val DEFAULT_CONSTANT_GROUP = ConstantGroup(constantSection = DEFAULT_CONSTANT_SECTION)

val DEFAULT_CONSTRUCTOR_GROUP =
    ConstructorGroup(
        constructorSection = DEFAULT_CONSTRUCTOR_SECTION, fromSection = DEFAULT_FROM_SECTION)

val DEFAULT_INDUCTIVELY_GROUP =
    InductivelyGroup(
        inductivelySection = DEFAULT_INDUCTIVELY_SECTION,
        fromSection = DEFAULT_INDUCTIVELY_FROM_SECTION)

val DEFAULT_MAPPING_GROUP =
    MappingGroup(
        mappingSection = DEFAULT_MAPPING_SECTION,
        fromSection = DEFAULT_FROM_SECTION,
        thenSection = DEFAULT_TO_SECTION,
        asSection = DEFAULT_AS_SECTION)

val DEFAULT_NOT_GROUP = NotGroup(notSection = DEFAULT_NOT_SECTION)

val DEFAULT_OR_GROUP = OrGroup(orSection = DEFAULT_OR_SECTION)

val DEFAULT_MATCHING_GROUP = MatchingGroup(matchingSection = DEFAULT_MATCHING_SECTION)

val DEFAULT_MUTUALLY_GROUP =
    MutuallyGroup(
        mutuallySection = DEFAULT_MUTUALLY_SECTION, metaDataSection = DEFAULT_META_DATA_SECTION)

val DEFAULT_STATES_GROUP =
    StatesGroup(
        signature = null,
        id = DEFAULT_ID_STATEMENT,
        statesSection = DEFAULT_STATES_SECTION,
        whenSection = DEFAULT_WHEN_SECTION,
        thatSection = DEFAULT_THAT_SECTION,
        usingSection = DEFAULT_USING_SECTION,
        writtenSection = DEFAULT_WRITTEN_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION)

val DEFAULT_ENTRY_SECTION = EntrySection(names = emptyList())

val DEFAULT_TYPE_SECTION = TypeSection(text = "")

val DEFAULT_CONTENT_SECTION = ContentSection(text = "")

val DEFAULT_ENTRY_GROUP =
    EntryGroup(
        entrySection = DEFAULT_ENTRY_SECTION,
        typeSection = DEFAULT_TYPE_SECTION,
        contentSection = DEFAULT_CONTENT_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION)

val DEFAULT_AXIOM_SECTION = AxiomSection(names = emptyList())

val DEFAULT_CONJECTURE_SECTION = ConjectureSection(names = emptyList())

val DEFAULT_THEOREM_SECTION = TheoremSection(names = emptyList())

val DEFAULT_GIVEN_SECTION = GivenSection(targets = listOf())

val DEFAULT_REQUIRING_SECTION = RequiringSection(targets = listOf())

val DEFAULT_IF_OR_IFF_SECTION = newIfOrIffSection(DEFAULT_IF_SECTION)

val DEFAULT_AXIOM_GROUP =
    AxiomGroup(
        signature = null,
        id = DEFAULT_ID_STATEMENT,
        axiomSection = DEFAULT_AXIOM_SECTION,
        givenSection = DEFAULT_GIVEN_SECTION,
        whereSection = DEFAULT_WHERE_SECTION,
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
        whereSection = DEFAULT_WHERE_SECTION,
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
        givenWhereSection = DEFAULT_WHERE_SECTION,
        ifOrIffSection = DEFAULT_IF_OR_IFF_SECTION,
        thenSection = DEFAULT_THEN_SECTION,
        usingSection = DEFAULT_USING_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION)

val DEFAULT_VIEWS_GROUP =
    ViewsGroup(
        signature = null,
        id = DEFAULT_ID_STATEMENT,
        viewsSection = DEFAULT_VIEWS_SECTION,
        singleFromSection = DEFAULT_SINGLE_FROM_SECTION,
        singleToSection = DEFAULT_SINGLE_TO_SECTION,
        asSection = DEFAULT_SINGLE_AS_SECTION,
        usingSection = DEFAULT_USING_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION)

val DEFAULT_RESOURCE_SECTION = ResourceSection(items = emptyList())

val DEFAULT_RESOURCE_GROUP =
    ResourceGroup(
        id = "",
        sourceSection = DEFAULT_RESOURCE_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION)

val DEFAULT_PIECEWISE_GROUP =
    PiecewiseGroup(
        piecewiseSection = DEFAULT_PIECEWISE_SECTION,
        whenThen = emptyList(),
        elseSection = DEFAULT_ELSE_SECTION)

val DEFAULT_EVALUATES_GROUP =
    EvaluatesGroup(
        signature = null,
        id = DEFAULT_ID_STATEMENT,
        evaluatesSection = DEFAULT_EVALUATES_SECTION,
        whenThen = emptyList(),
        elseSection = DEFAULT_ELSE_SECTION,
        usingSection = DEFAULT_USING_SECTION,
        writtenSection = DEFAULT_WRITTEN_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION)

val DEFAULT_SOURCE_ITEM_SECTION = SourceItemSection(sourceReference = "")

val DEFAULT_CONTENT_ITEM_SECTION = ContentItemSection(content = "")

val DEFAULT_OFFSET_ITEM_SECTION = OffsetItemSection(offset = "")

val DEFAULT_PAGE_ITEM_SECTION = PageItemSection(page = "")

val DEFAULT_REFERENCE_SECTION = ReferenceSection(sourceItems = emptyList())

val DEFAULT_REFERENCE_GROUP = ReferenceGroup(referenceSection = DEFAULT_REFERENCE_SECTION)

val DEFAULT_STRING_SECTION = StringSection(name = "", values = emptyList())

val DEFAULT_SOURCE_ITEM_GROUP =
    SourceItemGroup(
        sourceSection = DEFAULT_SOURCE_ITEM_SECTION,
        pageSection = DEFAULT_PAGE_ITEM_SECTION,
        offsetSection = DEFAULT_OFFSET_ITEM_SECTION,
        contentSection = DEFAULT_CONTENT_ITEM_SECTION)

val DEFAULT_META_DATA_ITEM =
    StringSectionGroup(section = StringSection(name = "", values = emptyList()))

val DEFAULT_GIVEN_GROUP =
    GivenGroup(
        givenSection = DEFAULT_GIVEN_SECTION,
        whereSection = DEFAULT_WHERE_SECTION,
        allSection = DEFAULT_ALL_SECTION,
        suchThatSection = DEFAULT_SUCH_THAT_SECTION)

val DEFAULT_FROM_GROUP =
    FromGroup(fromSection = DEFAULT_FROM_SECTION, toSection = DEFAULT_TO_SECTION)

val DEFAULT_COLLECTS_SECTION = CollectsSection(givenGroup = DEFAULT_GIVEN_GROUP)

val DEFAULT_MAPS_SECTION = MapsSection(fromGroup = DEFAULT_FROM_GROUP)

val DEFAULT_GENERATED_SECTION = GeneratedSection(inductivelyGroup = DEFAULT_INDUCTIVELY_GROUP)

val DEFAULT_DEFINES_MEANS_GROUP =
    DefinesMeansGroup(
        signature = null,
        id = DEFAULT_ID_STATEMENT,
        definesSection = DEFAULT_DEFINES_SECTION,
        requiringSection = DEFAULT_REQUIRING_SECTION,
        whenSection = DEFAULT_WHEN_SECTION,
        specifiesSection = DEFAULT_SPECIFIES_SECTION,
        meansSection = DEFAULT_MEANS_SECTION,
        usingSection = DEFAULT_USING_SECTION,
        writtenSection = DEFAULT_WRITTEN_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION)

val DEFAULT_DEFINES_INSTANTIATED_GROUP =
    DefinesInstantiatedGroup(
        signature = null,
        id = DEFAULT_ID_STATEMENT,
        definesSection = DEFAULT_DEFINES_SECTION,
        requiringSection = DEFAULT_REQUIRING_SECTION,
        whenSection = DEFAULT_WHEN_SECTION,
        instantiatedSection = DEFAULT_INSTANTIATED_SECTION,
        usingSection = DEFAULT_USING_SECTION,
        writtenSection = DEFAULT_WRITTEN_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION)

val DEFAULT_DEFINES_EVALUATED_GROUP =
    DefinesEvaluatedGroup(
        signature = null,
        id = DEFAULT_ID_STATEMENT,
        definesSection = DEFAULT_DEFINES_SECTION,
        requiringSection = DEFAULT_REQUIRING_SECTION,
        whenSection = DEFAULT_WHEN_SECTION,
        specifiesSection = DEFAULT_SPECIFIES_SECTION,
        evaluatedSection = DEFAULT_EVALUATED_SECTION,
        usingSection = DEFAULT_USING_SECTION,
        writtenSection = DEFAULT_WRITTEN_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION)

val DEFAULT_DEFINES_COLLECTS_GROUP =
    DefinesCollectsGroup(
        signature = null,
        id = DEFAULT_ID_STATEMENT,
        definesSection = DEFAULT_DEFINES_SECTION,
        requiringSection = DEFAULT_REQUIRING_SECTION,
        whenSection = DEFAULT_WHEN_SECTION,
        specifiesSection = DEFAULT_SPECIFIES_SECTION,
        collectsSection = DEFAULT_COLLECTS_SECTION,
        usingSection = DEFAULT_USING_SECTION,
        writtenSection = DEFAULT_WRITTEN_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION)

val DEFAULT_DEFINES_MAPS_GROUP =
    DefinesMapsGroup(
        signature = null,
        id = DEFAULT_ID_STATEMENT,
        definesSection = DEFAULT_DEFINES_SECTION,
        requiringSection = DEFAULT_REQUIRING_SECTION,
        whenSection = DEFAULT_WHEN_SECTION,
        specifiesSection = DEFAULT_SPECIFIES_SECTION,
        mapsSection = DEFAULT_MAPS_SECTION,
        usingSection = DEFAULT_USING_SECTION,
        writtenSection = DEFAULT_WRITTEN_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION)

val DEFAULT_DEFINES_GENERATED_GROUP =
    DefinesGeneratedGroup(
        signature = null,
        id = DEFAULT_ID_STATEMENT,
        definesSection = DEFAULT_DEFINES_SECTION,
        requiringSection = DEFAULT_REQUIRING_SECTION,
        whenSection = DEFAULT_WHEN_SECTION,
        specifiesSection = DEFAULT_SPECIFIES_SECTION,
        generatedSection = DEFAULT_GENERATED_SECTION,
        usingSection = DEFAULT_USING_SECTION,
        writtenSection = DEFAULT_WRITTEN_SECTION,
        metaDataSection = DEFAULT_META_DATA_SECTION)

val DEFAULT_FOUNDATION_SECTION = FoundationSection(content = DEFAULT_DEFINES_MEANS_GROUP)

val DEFAULT_FOUNDATION_GROUP =
    FoundationGroup(
        foundationSection = DEFAULT_FOUNDATION_SECTION, metaDataSection = DEFAULT_META_DATA_SECTION)

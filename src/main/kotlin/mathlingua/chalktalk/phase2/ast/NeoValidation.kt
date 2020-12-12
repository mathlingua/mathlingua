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

package mathlingua.chalktalk.phase2.ast

import mathlingua.chalktalk.phase1.ast.Abstraction
import mathlingua.chalktalk.phase1.ast.Assignment
import mathlingua.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.chalktalk.phase1.ast.Group
import mathlingua.chalktalk.phase1.ast.Mapping
import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase1.ast.Phase1Token
import mathlingua.chalktalk.phase1.ast.Section
import mathlingua.chalktalk.phase1.ast.Tuple
import mathlingua.chalktalk.phase1.ast.getColumn
import mathlingua.chalktalk.phase1.ast.getRow
import mathlingua.chalktalk.phase2.ast.clause.AbstractionNode
import mathlingua.chalktalk.phase2.ast.clause.AssignmentNode
import mathlingua.chalktalk.phase2.ast.clause.ClauseListNode
import mathlingua.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.chalktalk.phase2.ast.clause.Identifier
import mathlingua.chalktalk.phase2.ast.clause.MappingNode
import mathlingua.chalktalk.phase2.ast.clause.Statement
import mathlingua.chalktalk.phase2.ast.clause.Text
import mathlingua.chalktalk.phase2.ast.clause.TupleNode
import mathlingua.chalktalk.phase2.ast.group.clause.If.ElseIfSection
import mathlingua.chalktalk.phase2.ast.group.clause.If.ElseSection
import mathlingua.chalktalk.phase2.ast.group.clause.If.IfSection
import mathlingua.chalktalk.phase2.ast.group.clause.If.ThenSection
import mathlingua.chalktalk.phase2.ast.group.clause.collection.CollectionSection
import mathlingua.chalktalk.phase2.ast.group.clause.collection.OfSection
import mathlingua.chalktalk.phase2.ast.group.clause.exists.ExistsGroup
import mathlingua.chalktalk.phase2.ast.group.clause.exists.ExistsSection
import mathlingua.chalktalk.phase2.ast.group.clause.exists.SuchThatSection
import mathlingua.chalktalk.phase2.ast.group.clause.expands.AsSection
import mathlingua.chalktalk.phase2.ast.group.clause.iff.IffSection
import mathlingua.chalktalk.phase2.ast.group.clause.inductively.ConstantSection
import mathlingua.chalktalk.phase2.ast.group.clause.inductively.InductivelyFromSection
import mathlingua.chalktalk.phase2.ast.group.clause.inductively.InductivelySection
import mathlingua.chalktalk.phase2.ast.group.clause.mapping.FromSection
import mathlingua.chalktalk.phase2.ast.group.clause.mapping.MappingSection
import mathlingua.chalktalk.phase2.ast.group.clause.mapping.ToSection
import mathlingua.chalktalk.phase2.ast.group.clause.matching.MatchingSection
import mathlingua.chalktalk.phase2.ast.group.clause.not.NotSection
import mathlingua.chalktalk.phase2.ast.group.clause.or.OrSection
import mathlingua.chalktalk.phase2.ast.group.clause.piecewise.PiecewiseSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.EvaluatedSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.MeansSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.ProvidedSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.states.ThatSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.views.SingleAsSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.views.SingleFromSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.views.SingleToSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.WhereSection
import mathlingua.support.ParseError
import mathlingua.support.ValidationFailure
import mathlingua.support.validationFailure

fun <T, U> neoValidateByTransform(node: Phase1Node, errors: MutableList<ParseError>,
                               default: T, message: String,
                               transform: (node: Phase1Node) -> U?, builder: (node: U) -> T): T {
    val newNode = transform(node)
    return if (newNode != null) {
        builder(newNode)
    } else {
        errors.add(
            ParseError(
                message = message,
                row = getRow(node),
                column = getColumn(node)
            )
        )
        default
    }
}

fun <T> neoValidateGroup(node: Phase1Node, errors: MutableList<ParseError>,
                         name: String, default: T, builder: (group: Group) -> T) =
    neoValidateByTransform(node, errors, default, "Expected a group",
    {
        if (it is Group && it.sections.isNotEmpty() && it.sections[0].name.text == name) {
            it
        } else {
            null
        }
    }, builder)

fun <T> neoValidateSection(node: Phase1Node, errors: MutableList<ParseError>,
    name: String, default: T, builder: (section: Section) -> T) =
    neoValidateByTransform(node, errors, default, "Expected a section '$name'",
        {
            if (it is Section && it.name.text == name) {
                it
            } else {
                null
            }
        }, builder)



val DEFAULT_ABSTRACTION = AbstractionNode(
    abstraction = Abstraction(
        isEnclosed = false,
        isVarArgs = false,
        parts = emptyList(),
        subParams = emptyList()
    )
)

val DEFAULT_TOKEN = Phase1Token(
    text = "INVALID",
    type = ChalkTalkTokenType.Invalid,
    row = -1,
    column = -1
)

val DEFAULT_ASSIGNMENT = AssignmentNode(
    assignment = Assignment(
        lhs = DEFAULT_TOKEN,
        rhs = DEFAULT_TOKEN
    )
)

val DEFAULT_IDENTIFIER = Identifier(
    name = "INVALID",
    isVarArgs = false
)

val DEFAULT_ID_STATEMENT = IdStatement(
    text = "INVALID",
    texTalkRoot = validationFailure(emptyList())
)

val DEFAULT_STATEMENT = Statement(
    text = "INVALID",
    texTalkRoot = validationFailure(emptyList())
)

val DEFAULT_MAPPING = MappingNode(
    mapping = Mapping(
        lhs = DEFAULT_TOKEN,
        rhs = DEFAULT_TOKEN
    )
)

val DEFAULT_TEXT = Text(
    text = "INVALID"
)

val DEFAULT_TUPLE = TupleNode(
    tuple = Tuple(
        items = emptyList()
    )
)

val DEFAULT_CLAUSE_LIST_NODE = ClauseListNode(
    clauses = emptyList()
)

val DEFAULT_SUCH_THAT_SECTION = SuchThatSection(
    clauses = DEFAULT_CLAUSE_LIST_NODE
)

val DEFAULT_ELSE_SECTION = ElseSection(
    clauses = DEFAULT_CLAUSE_LIST_NODE
)

val DEFAULT_ELSE_IF_SECTION = ElseIfSection(
    clauses = DEFAULT_CLAUSE_LIST_NODE
)

val DEFAULT_IF_SECTION = IfSection(
    clauses = DEFAULT_CLAUSE_LIST_NODE
)

val DEFAULT_THEN_SECTION = ThenSection(
    clauses = DEFAULT_CLAUSE_LIST_NODE
)

val DEFAULT_AS_SECTION = AsSection(
    clauses = DEFAULT_CLAUSE_LIST_NODE
)

val DEFAULT_IFF_SECTION = IffSection(
    clauses = DEFAULT_CLAUSE_LIST_NODE
)

val DEFAULT_CONSTANT_SECTION = ConstantSection(
    clauses = DEFAULT_CLAUSE_LIST_NODE
)

val DEFAULT_COLLECTION_SECTION = CollectionSection()

val DEFAULT_INDUCTIVELY_FROM_SECTION = InductivelyFromSection(
    clauses = DEFAULT_CLAUSE_LIST_NODE
)

val DEFAULT_INDUCTIVELY_SECTION = InductivelySection()

val DEFAULT_MAPPING_SECTION = MappingSection()

val DEFAULT_MATCHING_SECTION = MatchingSection(
    clauses = DEFAULT_CLAUSE_LIST_NODE
)

val DEFAULT_NOT_SECTION = NotSection(
    clauses = DEFAULT_CLAUSE_LIST_NODE
)

val DEFAULT_OR_SECTION = OrSection(
    clauses = DEFAULT_CLAUSE_LIST_NODE
)

val DEFAULT_PIECEWISE_SECTION = PiecewiseSection()

val DEFAULT_EVALUATED_SECTION = EvaluatedSection(
    clauses = DEFAULT_CLAUSE_LIST_NODE
)

val DEFAULT_PROVIDED_SECTION = ProvidedSection(
    clauses = DEFAULT_CLAUSE_LIST_NODE
)

val DEFAULT_THAT_SECTION = ThatSection(
    clauses = DEFAULT_CLAUSE_LIST_NODE
)

val DEFAULT_SINGLE_TO_SECTION = SingleToSection(
    clauses = DEFAULT_CLAUSE_LIST_NODE
)

val DEFAULT_USING_SECTION = UsingSection(
    clauses = DEFAULT_CLAUSE_LIST_NODE
)

val DEFAULT_WHEN_SECTION = WhenSection(
    clauses = DEFAULT_CLAUSE_LIST_NODE
)

val DEFAULT_WHERE_SECTION = WhereSection(
    clauses = DEFAULT_CLAUSE_LIST_NODE
)

val DEFAULT_OF_SECTION = OfSection(
    statement = DEFAULT_STATEMENT
)

val DEFAULT_FROM_SECTION = FromSection(
    statements = emptyList()
)

val DEFAULT_TO_SECTION = ToSection(
    statements = emptyList()
)

val DEFAULT_MEANS_SECTION = MeansSection(
    clauses = DEFAULT_CLAUSE_LIST_NODE
)

val DEFAULT_STATES_SECTION = StatesSection()

val DEFAULT_SINGLE_AS_SECTION = SingleAsSection(
    statement = DEFAULT_STATEMENT
)

val DEFAULT_SINGLE_FROM_SECTION = SingleFromSection(
    statement = DEFAULT_STATEMENT
)

val DEFAULT_EXISTS_SECTION = ExistsSection(
    identifiers = emptyList()
)

val DEFAULT_EXISTS_GROUP = ExistsGroup(
    existsSection = DEFAULT_EXISTS_SECTION,
    whereSection = null,
    suchThatSection = DEFAULT_SUCH_THAT_SECTION
)

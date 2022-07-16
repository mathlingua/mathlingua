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

import mathlingua.lib.frontend.MetaData

internal sealed interface Node

internal sealed interface CommonNode : ChalkTalkNode, TexTalkNode, NodeLexerToken, Node

internal sealed interface NameOrFunction : CommonNode

internal sealed interface NameOrVariadicName : CommonNode

internal data class Name(val text: String, override val metadata: MetaData) :
    SquareTargetItem,
    NameOrNameAssignment,
    NameAssignmentItem,
    NameOrFunction,
    NameOrCommandExpressionCall,
    NameOrVariadicName,
    Expression,
    Target

internal sealed interface VariadicRhs : ChalkTalkNode, TexTalkNode

internal sealed interface VariadicIsRhs : ChalkTalkNode

internal sealed interface VariadicTargetForm : VariadicRhs

internal sealed interface Target : Argument

internal data class OperatorName(val text: String, override val metadata: MetaData) :
    Target, NameAssignmentItem, Operator

internal data class VariadicName(val name: Name, override val metadata: MetaData) :
    VariadicTargetForm, VariadicIsRhs, NameOrVariadicName

internal data class VariadicFunctionForm(
    val function: FunctionFormCall, override val metadata: MetaData
) : VariadicTargetForm

internal data class VariadicSequenceForm(
    val sequence: SequenceForm, override val metadata: MetaData
) : VariadicTargetForm

internal sealed interface FunctionFormCall : NameOrFunction, Expression

internal data class FunctionForm(
    val name: Name, val params: ParenNodeList<NameOrVariadicName>, override val metadata: MetaData
) : FunctionFormCall, NameOrFunction, Target, NameAssignmentItem, SquareTargetItem, Expression

internal data class SubParamFormCall(
    val name: Name,
    val subParams: ParenNodeList<NameOrVariadicName>,
    override val metadata: MetaData
) : FunctionFormCall

internal data class SubAndRegularParamFormCall(
    val name: Name,
    val subParams: ParenNodeList<NameOrVariadicName>,
    val params: ParenNodeList<NameOrVariadicName>,
    override val metadata: MetaData
) : FunctionFormCall

internal sealed interface SequenceForm : SquareTargetItem, NameAssignmentItem, Expression, Target

internal data class SubParamSequenceForm(
    val func: SubParamFormCall, override val metadata: MetaData
) : SequenceForm

internal data class SubAndRegularParamSequenceForm(
    val func: SubAndRegularParamFormCall, override val metadata: MetaData
) : SequenceForm

internal sealed interface NameAssignmentItem : CommonNode

internal data class NameAssignment(
    val lhs: Name, val rhs: NameAssignmentItem, override val metadata: MetaData
) : Target, NameOrNameAssignment

internal sealed interface Argument : CommonNode

internal sealed interface SquareTargetItem : CommonNode

internal data class TupleForm(val targets: ParenNodeList<Target>, override val metadata: MetaData) :
    SquareTargetItem, NameAssignmentItem, Target, Expression

internal sealed interface NameOrNameAssignment : CommonNode

internal data class SetForm(
    val items: CurlyNodeList<NameOrNameAssignment>, override val metadata: MetaData
) : SquareTargetItem, NameAssignmentItem, Target, Expression

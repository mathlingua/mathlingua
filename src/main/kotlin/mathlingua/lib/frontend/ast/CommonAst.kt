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

internal sealed interface CommonNode : ChalkTalkNode, TexTalkNode, NodeLexerToken

internal interface NameOrFunction : CommonNode

// names
internal data class Name(val text: String, override val metadata: MetaData) :
    Target, NameOrNameAssignment, NameAssignmentItem, NameOrFunction, NameOrCommand, Expression

internal data class OperatorName(val text: String, override val metadata: MetaData) :
    Target, NameAssignmentItem, NameOrFunction

internal data class NameParam(val name: Name, val isVarArgs: Boolean)

// functions
internal sealed interface Function : Target, NameAssignmentItem, NameOrFunction, Expression

internal data class RegularFunction(
    val name: Name, val params: List<NameParam>, override val metadata: MetaData
) : Function

internal data class SubParamFunction(
    val name: Name, val subParams: List<NameParam>, override val metadata: MetaData
) : Function

internal data class SubAndRegularParamFunction(
    val name: Name,
    val subParams: List<NameParam>,
    val params: List<NameParam>,
    override val metadata: MetaData
) : Function

// sequences
internal sealed interface Sequence : Target, NameAssignmentItem, Expression

internal data class SubParamFunctionSequence(
    val func: SubParamFunction, override val metadata: MetaData
) : Sequence

internal data class SubAndRegularParamFunctionSequence(
    val func: SubAndRegularParamFunction, override val metadata: MetaData
) : Sequence

// assignments
internal sealed interface Assignment : Target
// <name> | <operator> | <tuple> | <sequence> | <function> | <set>
internal sealed interface NameAssignmentItem : CommonNode

internal data class NameAssignment(
    val lhs: Name, val rhs: NameAssignmentItem, override val metadata: MetaData
) : Assignment, NameOrNameAssignment

internal data class FunctionAssignment(
    val lhs: Function, val rhs: Function, override val metadata: MetaData
) : Assignment

// <target> | <text> | <statement>
internal sealed interface Argument : CommonNode

// targets
// <assignment> | <name> | <operator> | <tuple> | <sequence> | <function> | <set>
internal sealed interface Target : Argument

internal data class Tuple(val targets: List<Target>, override val metadata: MetaData) :
    Target, NameAssignmentItem, Expression

// <name> | <name assignment>
internal sealed interface NameOrNameAssignment : CommonNode

internal data class Set(val items: List<NameOrNameAssignment>, override val metadata: MetaData) :
    Target, NameAssignmentItem, Expression

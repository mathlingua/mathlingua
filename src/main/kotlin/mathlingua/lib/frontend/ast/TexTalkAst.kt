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

internal sealed interface TexTalkNode : HasMetaData, TexTalkNodeOrToken

internal sealed interface Expression : TexTalkNode

internal sealed interface TexTalkNodeOrToken

internal enum class TexTalkTokenType {
    LParen,
    RParen,
    LSquare,
    RSquare,
    LCurly,
    RCurly,
    Comma,
    ColonEquals,
    Is,
    In,
    NotIn,
    As,
    ColonTypeColon,
    ColonStatementColon,
    ColonExpressionColon,
    Underscore,
    Caret,
    DotDotDot,
    Dot,
    Backslash,
    Slash,
    Name,
    Operator,
    Colon,
    Equals,
    NotEquals
}

internal data class TexTalkToken(
    val type: TexTalkTokenType, val text: String, val row: Int, val column: Int
) : TexTalkNodeOrToken

internal data class NamedParameterExpression(
    val name: Name, val params: List<Expression>, override val metadata: MetaData
) : TexTalkNode

internal data class CommandExpression(
    val names: List<Name>,
    val squareParams: List<Name>?,
    val subParams: List<Expression>?,
    val supParams: List<Expression>?,
    val curlyParams: List<Expression>?,
    val namedParams: List<NamedParameterExpression>?,
    val parenParams: List<Expression>?,
    override val metadata: MetaData
) : TexTalkNode, NameOrCommand, Expression

internal data class NamedParameterForm(
    val name: Name, val params: List<Name>, override val metadata: MetaData
) : TexTalkNode

internal data class CommandForm(
    val names: List<Name>,
    val squareParams: List<Name>?,
    val subParams: List<Name>?,
    val supParams: List<Name>?,
    val curlyParams: List<Name>?,
    val namedParams: List<NamedParameterForm>?,
    val parenParams: List<Name>?,
    override val metadata: MetaData
) : TexTalkNode

internal sealed interface OperationExpression : Expression

internal data class InfixCommandExpression(
    val lhs: Expression,
    val command: CommandExpression,
    val rhs: Expression,
    override val metadata: MetaData
) : OperationExpression

internal data class InfixCommandForm(val command: CommandForm, override val metadata: MetaData) :
    TexTalkNode

internal sealed interface NameOrCommand : TexTalkNode

internal data class IsExpression(
    val lhs: List<Target>, val rhs: List<NameOrCommand>, override val metadata: MetaData
) : Expression

internal data class SignatureExpression(
    val names: List<Name>, val colonNames: List<Name>, override val metadata: MetaData
) : TexTalkNode

internal data class AsExpression(
    val lhs: Expression, val rhs: SignatureExpression, override val metadata: MetaData
) : Expression

internal data class InExpression(
    val lhs: List<Target>, val rhs: NameOrCommand, override val metadata: MetaData
) : Expression

internal data class NotInExpression(
    val lhs: List<Target>, val rhs: NameOrCommand, override val metadata: MetaData
) : Expression

internal data class ColonEqualsExpression(
    val lhs: Target, val rhs: Expression, override val metadata: MetaData
) : Expression

internal data class EqualsExpression(
    val lhs: Expression, val rhs: Expression, override val metadata: MetaData
) : Expression

internal data class NotEqualsExpression(
    val lhs: Expression, val rhs: Expression, override val metadata: MetaData
) : Expression

internal enum class GroupingType {
    Paren,
    Curly
}

internal data class GroupingExpression(
    val type: GroupingType, val expression: Expression, override val metadata: MetaData
) : Expression

internal sealed interface Operator : TexTalkNode

internal data class MemberScopedOperatorName(
    val prefixes: List<Name>, val name: Name, override val metadata: MetaData
) : Operator

internal data class TypeScopedOperatorName(
    val signature: SignatureExpression, val name: OperatorName, override val metadata: MetaData
) : Operator

internal data class MemberScopedName(
    val prefixes: List<Name>, val name: Name, override val metadata: MetaData
) : Expression

internal data class InfixOperatorExpression(
    val operator: Operator,
    val lhs: Expression,
    val rhs: Expression,
    override val metadata: MetaData
) : OperationExpression

internal data class PrefixOperatorExpression(
    val operator: Operator, val value: Expression, override val metadata: MetaData
) : OperationExpression

internal data class PostfixOperatorExpression(
    val operator: Operator, val value: Expression, override val metadata: MetaData
) : OperationExpression

internal sealed interface FunctionCall : Expression

internal data class RegularFunctionCall(
    val name: Name,
    val args: List<Expression>,
    override val metadata: MetaData
) : FunctionCall

internal data class SubParamFunctionCall(
    val name: Name,
    val subArgs: List<Expression>,
    override val metadata: MetaData
) : FunctionCall

internal data class SubAndRegularParamFunctionCall(
    val name: Name,
    val subArgs: List<Expression>,
    val args: List<Expression>,
    override val metadata: MetaData
) : FunctionCall

internal data class TupleCall(
    val args: List<Expression>,
    override val metadata: MetaData
) : Expression

internal sealed interface AssignmentExpression : Expression

internal data class NameAssignmentExpression(
    val lhs: Name,
    val rhs: Expression,
    override val metadata: MetaData
) : AssignmentExpression

internal data class FunctionAssignmentExpression(
    val lhs: Function,
    val rhs: Expression,
    override val metadata: MetaData
) : AssignmentExpression

internal data class SetAssignmentExpression(
    val lhs: Set,
    val rhs: Expression,
    override val metadata: MetaData
) : AssignmentExpression

internal data class SequenceAssignmentExpression(
    val lhs: Sequence,
    val rhs: Expression,
    override val metadata: MetaData
) : AssignmentExpression

internal data class TupleAssignmentExpression(
    val lhs: Tuple,
    val rhs: Expression,
    override val metadata: MetaData
) : AssignmentExpression

internal data class NameAssignmentAssignmentExpression(
    val lhs: NameAssignment,
    val rhs: Expression,
    override val metadata: MetaData
) : AssignmentExpression

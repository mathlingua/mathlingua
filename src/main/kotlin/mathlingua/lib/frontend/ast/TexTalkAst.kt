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

internal sealed interface Expression : TexTalkNode, VariadicRhs

internal sealed interface TexTalkNodeOrToken

internal enum class TexTalkTokenType {
    LParen,
    RParen,
    LSquare,
    RSquare,
    LSquareColon,
    ColonRSquare,
    LCurly,
    RCurly,
    Comma,
    ColonEqual,
    Is,
    In,
    NotIn,
    As,
    Underscore,
    Caret,
    DotDotDot,
    Dot,
    Backslash,
    Slash,
    Name,
    Operator,
    Colon,
    ColonColon,
    Equals,
    NotEqual
}

internal data class TexTalkToken(
    val type: TexTalkTokenType, val text: String, val row: Int, val column: Int
) : TexTalkNodeOrToken

internal data class NamedParameterExpression(
    val name: Name, val params: List<Expression>, override val metadata: MetaData
) : TexTalkNode

internal class SquareParams private constructor(private val value: Any) {
    constructor(items: List<SquareTargetItem>) : this(items as Any)
    constructor(variadicName: VariadicName) : this(variadicName as Any)

    fun isSquareTargetItems() = value is List<*>

    @Suppress("UNCHECKED_CAST")
    fun asSquareTargetItems(): List<SquareTargetItem> = value as List<SquareTargetItem>

    fun isNameParam() = value is VariadicName
    fun asNameParam() = value as VariadicName

    override fun toString() = value.toString()
}

internal data class CommandExpression(
    val names: List<Name>,
    val squareParams: SquareParams?,
    val subParams: List<Expression>?,
    val supParams: List<Expression>?,
    val curlyParams: List<Expression>?,
    val namedParams: List<NamedParameterExpression>?,
    val parenParams: List<Expression>?,
    override val metadata: MetaData
) : TexTalkNode, NameOrCommand, Expression, VariadicIsRhs {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class NamedParameterForm(
    val name: Name, val params: List<Name>, override val metadata: MetaData
) : TexTalkNode

internal data class CommandForm(
    val names: List<Name>,
    val squareParams: SquareParams?,
    val subParams: List<Name>?,
    val supParams: List<Name>?,
    val curlyParams: List<Name>?,
    val namedParams: List<NamedParameterForm>?,
    val parenParams: List<Name>?,
    override val metadata: MetaData
) : IdForm, TexTalkNode {
    override fun toCode(): String {
        TODO("Not yet implemented")
    }
}

internal sealed interface OperationExpression : Expression

internal data class InfixCommandExpression(
    val lhs: Expression,
    val command: CommandExpression,
    val rhs: Expression,
    override val metadata: MetaData
) : OperationExpression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class InfixCommandForm(val command: CommandForm, override val metadata: MetaData) :
    TexTalkNode

internal sealed interface NameOrCommand : TexTalkNode

internal data class VariadicIsExpression(
    val lhs: VariadicTarget, val rhs: VariadicIsRhs, override val metadata: MetaData
) : Expression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class IsExpression(
    val lhs: List<Target>, val rhs: List<NameOrCommand>, override val metadata: MetaData
) : Expression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class SignatureExpression(
    val names: List<Name>, val colonNames: List<Name>, override val metadata: MetaData
) : TexTalkNode

internal data class AsExpression(
    val lhs: Expression, val rhs: SignatureExpression, override val metadata: MetaData
) : Expression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class VariadicInExpression(
    val lhs: VariadicTarget, val rhs: VariadicRhs, override val metadata: MetaData
) : Expression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class InExpression(
    val lhs: List<Target>, val rhs: Expression, override val metadata: MetaData
) : Expression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class VariadicNotInExpression(
    val lhs: VariadicTarget, val rhs: VariadicRhs, override val metadata: MetaData
) : Expression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class NotInExpression(
    val lhs: List<Target>, val rhs: Expression, override val metadata: MetaData
) : Expression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class ColonEqualsExpression(
    val lhs: Target, val rhs: Expression, override val metadata: MetaData
) : Expression, SatisfyingItem, ExpressingItem, ThatItem {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class VariadicColonEqualsExpression(
    val lhs: VariadicTarget, val rhs: VariadicRhs, override val metadata: MetaData
) : Expression, SatisfyingItem, ExpressingItem {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class EqualsExpression(
    val lhs: Expression, val rhs: Expression, override val metadata: MetaData
) : Expression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class NotEqualsExpression(
    val lhs: Expression, val rhs: Expression, override val metadata: MetaData
) : Expression {
    override fun toCode() = TODO("Not yet implemented")
}

internal interface GroupingExpression : Expression

internal data class CurlyGroupingExpression(
    val expression: Expression, override val metadata: MetaData
) : GroupingExpression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class ParenGroupingExpression(
    val expression: Expression, override val metadata: MetaData
) : GroupingExpression {
    override fun toCode() = TODO("Not yet implemented")
}

internal sealed interface Operator : TexTalkNode

internal data class MemberScopedOperatorName(
    val prefixes: List<Name>, val name: Name, override val metadata: MetaData
) : Operator

internal data class TypeScopedInfixOperatorName(
    val signature: SignatureExpression, val name: OperatorName, override val metadata: MetaData
) : Operator

internal data class TypeScopedOperatorName(
    val signature: SignatureExpression, val name: OperatorName, override val metadata: MetaData
) : Operator

internal data class MemberScopedName(
    val prefixes: List<Name>, val name: Name, override val metadata: MetaData
) : Expression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class InfixOperatorExpression(
    val operator: Operator,
    val lhs: Expression,
    val rhs: Expression,
    override val metadata: MetaData
) : OperationExpression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class PrefixOperatorExpression(
    val operator: Operator, val value: Expression, override val metadata: MetaData
) : OperationExpression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class PostfixOperatorExpression(
    val operator: Operator, val value: Expression, override val metadata: MetaData
) : OperationExpression {
    override fun toCode() = TODO("Not yet implemented")
}

internal sealed interface CallExpression : Expression

internal data class FunctionCallExpression(
    val name: Name, val args: List<Expression>, override val metadata: MetaData
) : CallExpression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class SubParamCallExpression(
    val name: Name, val subArgs: List<Expression>, override val metadata: MetaData
) : CallExpression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class SubAndRegularParamCallExpression(
    val name: Name,
    val subArgs: List<Expression>,
    val args: List<Expression>,
    override val metadata: MetaData
) : CallExpression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class TupleExpression(val args: List<Expression>, override val metadata: MetaData) :
    Expression {
    override fun toCode() = TODO("Not yet implemented")
}

internal sealed interface AssignmentExpression : Expression

internal data class NameAssignmentExpression(
    val lhs: Name, val rhs: Expression, override val metadata: MetaData
) : AssignmentExpression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class FunctionAssignmentExpression(
    val lhs: FunctionCall, val rhs: Expression, override val metadata: MetaData
) : AssignmentExpression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class SetAssignmentExpression(
    val lhs: Set, val rhs: Expression, override val metadata: MetaData
) : AssignmentExpression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class SequenceAssignmentExpression(
    val lhs: Sequence, val rhs: Expression, override val metadata: MetaData
) : AssignmentExpression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class TupleAssignmentExpression(
    val lhs: Tuple, val rhs: Expression, override val metadata: MetaData
) : AssignmentExpression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class NameAssignmentAssignmentExpression(
    val lhs: NameAssignment, val rhs: Expression, override val metadata: MetaData
) : AssignmentExpression {
    override fun toCode() = TODO("Not yet implemented")
}

internal data class OperationAssignmentExpression(
    val lhs: NameAssignment, val rhs: Expression, override val metadata: MetaData
) : AssignmentExpression {
    override fun toCode() = TODO("Not yet implemented")
}

internal interface MetaIsFormItem : TexTalkNode

internal data class StatementIsFormItem(override val metadata: MetaData) : MetaIsFormItem

internal data class AssignmentIsFormItem(override val metadata: MetaData) : MetaIsFormItem

internal data class SpecificationIsFormItem(override val metadata: MetaData) : MetaIsFormItem

internal data class ExpressionIsFormItem(override val metadata: MetaData) : MetaIsFormItem

internal data class DefinitionIsFormItem(override val metadata: MetaData) : MetaIsFormItem

internal data class MetaIsForm(val items: List<MetaIsFormItem>, override val metadata: MetaData) :
    TexTalkNode

internal data class IdInfixOperatorCall(
    val lhs: Name, val center: OperatorName, val rhs: Name, override val metadata: MetaData
) : IdForm {
    override fun toCode(): String {
        TODO("Not yet implemented")
    }
}

internal data class IdPostfixOperatorCall(
    val lhs: Name, val center: OperatorName, override val metadata: MetaData
) : IdForm {
    override fun toCode(): String {
        TODO("Not yet implemented")
    }
}

internal data class IdPrefixOperatorCall(
    val center: OperatorName, val rhs: Name, override val metadata: MetaData
) : IdForm {
    override fun toCode(): String {
        TODO("Not yet implemented")
    }
}

internal data class InfixCommandFormCall(
    val lhs: Name, val center: InfixCommandForm, val rhs: Name, override val metadata: MetaData
) : IdForm {
    override fun toCode(): String {
        TODO("Not yet implemented")
    }
}

internal data class InfixCommandExpressionForm(
    val expression: CommandExpression, override val metadata: MetaData
) : TexTalkNode {}

internal object EmptyTexTalkNode : TexTalkNode {
    override val metadata = MetaData(row = -1, column = -1, isInline = null)
}

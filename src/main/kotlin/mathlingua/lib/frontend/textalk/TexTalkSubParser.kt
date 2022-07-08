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

package mathlingua.lib.frontend.textalk

import mathlingua.lib.frontend.Diagnostic
import mathlingua.lib.frontend.MetaData
import mathlingua.lib.frontend.ast.CommandExpressionCall
import mathlingua.lib.frontend.ast.CommandFormCall
import mathlingua.lib.frontend.ast.CurlyNodeList
import mathlingua.lib.frontend.ast.DEFAULT_NAME
import mathlingua.lib.frontend.ast.Expression
import mathlingua.lib.frontend.ast.FunctionForm
import mathlingua.lib.frontend.ast.FunctionFormCall
import mathlingua.lib.frontend.ast.Name
import mathlingua.lib.frontend.ast.NameOrCommandExpressionCall
import mathlingua.lib.frontend.ast.NameOrVariadicName
import mathlingua.lib.frontend.ast.NamedParameterExpression
import mathlingua.lib.frontend.ast.NamedParameterForm
import mathlingua.lib.frontend.ast.NonBracketNodeList
import mathlingua.lib.frontend.ast.OperatorName
import mathlingua.lib.frontend.ast.ParenNodeList
import mathlingua.lib.frontend.ast.SquareNodeList
import mathlingua.lib.frontend.ast.SquareParams
import mathlingua.lib.frontend.ast.SubAndRegularParamFormCall
import mathlingua.lib.frontend.ast.SubParamFormCall
import mathlingua.lib.frontend.ast.TexTalkNode
import mathlingua.lib.frontend.ast.TexTalkToken
import mathlingua.lib.frontend.ast.TexTalkTokenNode
import mathlingua.lib.frontend.ast.TexTalkTokenType
import mathlingua.lib.frontend.ast.VariadicFunctionForm
import mathlingua.lib.frontend.ast.VariadicIsRhs
import mathlingua.lib.frontend.ast.VariadicName
import mathlingua.lib.frontend.ast.defaultCurlyNodeList
import mathlingua.lib.frontend.ast.defaultParenNodeList

internal interface TexTalkSubParser {
    fun parse(): TexTalkNode?
}

internal fun newTexTalkSubParser(
    nodes: MutableList<TreeNode>, diagnostics: MutableList<Diagnostic>
): TexTalkSubParser = TexTalkSubParserImpl(nodes, diagnostics)

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private class TexTalkSubParserImpl(
    val treeNodes: MutableList<TreeNode>, private val diagnostics: MutableList<Diagnostic>
) : TexTalkSubParser {
    private val nodes = treeNodes.mapNotNull { it.toTexTalkNode(diagnostics) }.toMutableList()

    override fun parse(): TexTalkNode? = null // parseIntoSingleNode(diagnostics)

    private fun token(type: TexTalkTokenType): TexTalkToken? =
        if (has(type)) {
            pollToken()
        } else {
            null
        }

    private fun name(): Name? =
        if (this.has(TexTalkTokenType.Name)) {
            val token = this.pollToken()
            Name(
                text = token.text,
                metadata = MetaData(row = token.row, column = token.column, isInline = null))
        } else {
            null
        }

    private fun variadicName(diagnostics: MutableList<Diagnostic>): VariadicName? =
        if (this.hasHas(TexTalkTokenType.Name, TexTalkTokenType.DotDotDot)) {
            val name = name()!!
            this.expect(TexTalkTokenType.DotDotDot)
            this.pollToken() // move past the ...
            VariadicName(name = name, metadata = name.metadata.copy())
        } else {
            null
        }

    private fun nameOrVariadicName(diagnostics: MutableList<Diagnostic>): NameOrVariadicName? =
        this.name() ?: this.variadicName(diagnostics)

    private fun operatorName(): OperatorName? =
        if (this.has(TexTalkTokenType.Operator)) {
            val token = this.pollToken()
            OperatorName(
                text = token.text,
                metadata = MetaData(row = token.row, column = token.column, isInline = null))
        } else {
            null
        }

    private fun parenNodeList(): ParenNodeList<*>? = pollIfHasNode()

    private fun squareNodeList(): SquareNodeList<*>? = pollIfHasNode()

    private fun curlyNodeList(): CurlyNodeList<*>? = pollIfHasNode()

    private fun functionForm(diagnostics: MutableList<Diagnostic>): FunctionForm? =
        if (this.hasHas(TexTalkTokenType.Name, TexTalkTokenType.LParen)) {
            val name = name()!!
            FunctionForm(
                name = name,
                params =
                    parenNodeList()
                        ?.asParenNodeListOfType<NameOrVariadicName>()
                        ?.errorIfNullOrEmpty(
                            diagnostics = diagnostics,
                            row = name.metadata.row,
                            column = name.metadata.column)
                        ?: defaultParenNodeList(),
                metadata = name.metadata.copy())
        } else {
            null
        }

    private fun variadicFunctionForm(diagnostics: MutableList<Diagnostic>): VariadicFunctionForm? {
        val funcForm = this.functionForm(diagnostics) ?: return null
        this.expect(TexTalkTokenType.DotDotDot)
        return VariadicFunctionForm(function = funcForm, metadata = funcForm.metadata.copy())
    }

    private fun subParamFormCallOrSubAndRegularParamFormCall(
        diagnostics: MutableList<Diagnostic>
    ): FunctionFormCall? =
        if (this.hasHasHas(
            TexTalkTokenType.Name, TexTalkTokenType.Underscore, TexTalkTokenType.LParen)) {
            val name = name()!!
            this.expect(TexTalkTokenType.Underscore)
            val subParams =
                parenNodeList()
                    ?.asParenNodeListOfType<NameOrVariadicName>()
                    ?.errorIfNullOrEmpty(
                        diagnostics = diagnostics,
                        row = name.metadata.row,
                        column = name.metadata.column)
                    ?: defaultParenNodeList()
            val params =
                parenNodeList()
                    ?.asParenNodeListOfType<NameOrVariadicName>()
                    ?.errorIfNullOrEmpty(
                        diagnostics = diagnostics,
                        row = name.metadata.row,
                        column = name.metadata.column)
            if (params != null) {
                SubAndRegularParamFormCall(
                    name = name,
                    subParams = subParams,
                    params = params,
                    metadata = name.metadata.copy())
            } else {
                SubParamFormCall(
                    name = name, subParams = subParams, metadata = name.metadata.copy())
            }
        } else {
            null
        }

    private fun functionFormCall(diagnostics: MutableList<Diagnostic>): FunctionFormCall? =
        this.functionForm(diagnostics)
            ?: this.subParamFormCallOrSubAndRegularParamFormCall(diagnostics)

    private fun squareParams(diagnostics: MutableList<Diagnostic>): SquareParams? {
        val peek = this.nodes.firstOrNull()
        val row = peek?.metadata?.row ?: -1
        val column = peek?.metadata?.column ?: -1

        val squareNodes = this.squareNodeList() ?: return null
        if (squareNodes.nodes.isEmpty()) {
            diagnostics.add(
                error(message = "Expected at least one parameter", row = row, column = column))
            return SquareParams(
                items =
                    SquareNodeList(
                        nodes = emptyList(),
                        metadata = MetaData(row = row, column = column, isInline = null)))
        }

        return if (nodes.first() is VariadicName) {
            for (i in 1 until nodes.size) {
                diagnostics.add(
                    error(
                        message = "At most one VariadicName can be specified in a [...]",
                        row = nodes[i].metadata.row,
                        column = nodes[i].metadata.column))
            }
            SquareParams(nodes.first() as VariadicName)
        } else {
            SquareParams(
                items =
                    SquareNodeList(
                        nodes = nodes.filterAndError(diagnostics),
                        metadata = MetaData(row = row, column = column, isInline = null)))
        }
    }

    private fun namedParameterForm(): NamedParameterForm? {
        if (!this.hasHas(TexTalkTokenType.Colon, TexTalkTokenType.Name)) {
            return null
        }
        val colon = this.token(TexTalkTokenType.Colon) ?: return null
        val name = this.name().orError("Expected a Name", colon.row, colon.column, DEFAULT_NAME)
        val curlyExpression =
            curlyNodeList()
                ?.asCurlyNodeListOfType<NameOrVariadicName>()
                ?.errorIfNullOrEmpty(
                    diagnostics = diagnostics,
                    row = name.metadata.row,
                    column = name.metadata.column)
        return NamedParameterForm(
            name = name,
            params =
                curlyExpression.orError(
                    message = "Expected a {...}",
                    row = colon.row,
                    column = colon.column,
                    default = defaultCurlyNodeList()),
            metadata = MetaData(row = colon.row, column = colon.column, isInline = null))
    }

    private fun namedParameterExpression(
        diagnostics: MutableList<Diagnostic>
    ): NamedParameterExpression? {
        if (!this.hasHas(TexTalkTokenType.Colon, TexTalkTokenType.Name)) {
            return null
        }
        val colon = this.token(TexTalkTokenType.Colon) ?: return null
        val name = this.name().orError("Expected a Name", colon.row, colon.column, DEFAULT_NAME)
        val curlyExpression =
            curlyNodeList()
                ?.asCurlyNodeListOfType<Expression>()
                ?.errorIfNullOrEmpty(
                    diagnostics = diagnostics,
                    row = name.metadata.row,
                    column = name.metadata.column)
        return NamedParameterExpression(
            name = name,
            params =
                curlyExpression.orError(
                    message = "Expected a {...}",
                    row = colon.row,
                    column = colon.column,
                    default = defaultCurlyNodeList()),
            metadata = MetaData(row = colon.row, column = colon.column, isInline = null))
    }

    private fun namedParameterForm(diagnostics: MutableList<Diagnostic>): NamedParameterForm? {
        if (!this.hasHas(TexTalkTokenType.Colon, TexTalkTokenType.Name)) {
            return null
        }
        val colon = this.token(TexTalkTokenType.Colon) ?: return null
        val name = this.name().orError("Expected a Name", colon.row, colon.column, DEFAULT_NAME)
        val params =
            curlyNodeList()
                ?.asCurlyNodeListOfType<NameOrVariadicName>()
                ?.errorIfNullOrEmpty(
                    diagnostics = diagnostics,
                    row = name.metadata.row,
                    column = name.metadata.column)
                ?: defaultCurlyNodeList()
        return NamedParameterForm(
            name = name,
            params = params,
            metadata = MetaData(row = colon.row, column = colon.column, isInline = null))
    }

    private fun nameOrCommandExpressionCall(
        diagnostics: MutableList<Diagnostic>
    ): NameOrCommandExpressionCall? = this.name() ?: this.commandExpressionCall(diagnostics)

    private fun variadicIsRhs(): VariadicIsRhs? =
        this.variadicName(diagnostics) ?: this.commandExpressionCall(diagnostics)

    private fun subExpressionParams(): CurlyNodeList<Expression>? =
        if (hasHas(TexTalkTokenType.Underscore, TexTalkTokenType.LCurly)) {
            expect(TexTalkTokenType.Underscore)
            curlyNodeList()?.asCurlyNodeListOfType()
        } else {
            null
        }

    private fun subNameOrVariadicNameParams(): CurlyNodeList<NameOrVariadicName>? =
        if (hasHas(TexTalkTokenType.Underscore, TexTalkTokenType.LCurly)) {
            expect(TexTalkTokenType.Underscore)
            curlyNodeList()?.asCurlyNodeListOfType()
        } else {
            null
        }

    private fun supExpressionParams(): CurlyNodeList<Expression>? =
        if (hasHas(TexTalkTokenType.Caret, TexTalkTokenType.LCurly)) {
            expect(TexTalkTokenType.Caret)
            curlyNodeList()?.asCurlyNodeListOfType()
        } else {
            null
        }

    private fun supNameOrVariadicNameParams(): CurlyNodeList<NameOrVariadicName>? =
        if (hasHas(TexTalkTokenType.Caret, TexTalkTokenType.LCurly)) {
            expect(TexTalkTokenType.Caret)!!
            curlyNodeList()?.asCurlyNodeListOfType()
        } else {
            null
        }

    private fun commandExpressionCall(
        diagnostics: MutableList<Diagnostic>
    ): CommandExpressionCall? =
        if (!this.has(TexTalkTokenType.Backslash)) {
            null
        } else {
            val backslash = this.expect(TexTalkTokenType.Backslash)!!
            val names = mutableListOf<Name>()
            while (this.nodes.isNotEmpty()) {
                val name = name() ?: break
                names.add(name)
                if (this.has(TexTalkTokenType.Dot)) {
                    this.expect(TexTalkTokenType.Dot)
                } else {
                    break
                }
            }
            if (names.isEmpty()) {
                diagnostics.add(
                    error(
                        message = "Expected at least one Name",
                        row = backslash.row,
                        column = backslash.column))
            }
            val squareParams = this.squareParams(diagnostics)
            val subParams = this.subExpressionParams()
            val supParams = this.supExpressionParams()
            val curlyParams = curlyNodeList()?.asCurlyNodeListOfType<Expression>()
            val namedParams = mutableListOf<NamedParameterExpression>()
            while (this.nodes.isNotEmpty()) {
                val namedParam = this.namedParameterExpression(diagnostics) ?: break
                namedParams.add(namedParam)
            }
            val parenParams = parenNodeList()?.asParenNodeListOfType<Expression>()

            CommandExpressionCall(
                names =
                    NonBracketNodeList(
                        nodes = names,
                        metadata =
                            MetaData(
                                row = backslash.row, column = backslash.column, isInline = null)),
                squareParams = squareParams,
                subParams = subParams,
                supParams = supParams,
                curlyParams = curlyParams,
                namedParams =
                    NonBracketNodeList(
                        nodes = namedParams,
                        metadata =
                            MetaData(
                                row = backslash.row, column = backslash.column, isInline = null)),
                parenParams = parenParams,
                metadata =
                    MetaData(row = backslash.row, column = backslash.column, isInline = null))
        }

    private fun commandFormCall(diagnostics: MutableList<Diagnostic>): CommandFormCall? =
        if (!this.has(TexTalkTokenType.Backslash)) {
            null
        } else {
            val backslash = this.expect(TexTalkTokenType.Backslash)!!
            val names = mutableListOf<Name>()
            while (this.nodes.isNotEmpty()) {
                val name = name() ?: break
                names.add(name)
                if (this.has(TexTalkTokenType.Dot)) {
                    this.expect(TexTalkTokenType.Dot)
                } else {
                    break
                }
            }
            if (names.isEmpty()) {
                diagnostics.add(
                    error(
                        message = "Expected at least one Name",
                        row = backslash.row,
                        column = backslash.column))
            }
            val squareParams = this.squareParams(diagnostics)
            val subParams = this.subNameOrVariadicNameParams()
            val supParams = this.supNameOrVariadicNameParams()
            val curlyParams = curlyNodeList()?.asCurlyNodeListOfType<NameOrVariadicName>()
            val namedParams = mutableListOf<NamedParameterForm>()
            while (this.nodes.isNotEmpty()) {
                val namedParam = this.namedParameterForm(diagnostics) ?: break
                namedParams.add(namedParam)
            }
            val parenParams = parenNodeList()?.asParenNodeListOfType<NameOrVariadicName>()

            CommandFormCall(
                names =
                    NonBracketNodeList(
                        nodes = names,
                        metadata =
                            MetaData(
                                row = backslash.row, column = backslash.column, isInline = null)),
                squareParams = squareParams,
                subParams = subParams,
                supParams = supParams,
                curlyParams = curlyParams,
                namedParams =
                    NonBracketNodeList(
                        nodes = namedParams,
                        metadata =
                            MetaData(
                                row = backslash.row, column = backslash.column, isInline = null)),
                parenParams = parenParams,
                metadata =
                    MetaData(row = backslash.row, column = backslash.column, isInline = null))
        }

    private fun commandForm(diagnostics: MutableList<Diagnostic>): CommandFormCall? =
        if (this.has(TexTalkTokenType.Backslash)) {
            val backslash = this.expect(TexTalkTokenType.Backslash)!!
            val nameList = mutableListOf<Name>()
            while (this.nodes.isNotEmpty()) {
                val name = name() ?: break
                nameList.add(name)
                if (this.has(TexTalkTokenType.Dot)) {
                    this.expect(TexTalkTokenType.Dot)
                } else {
                    break
                }
            }
            if (nameList.isEmpty()) {
                diagnostics.add(
                    error(
                        message = "Expected at least one Name",
                        row = backslash.row,
                        column = backslash.column))
            }
            val squareParams = this.squareParams(diagnostics)
            val subParams = this.subNameOrVariadicNameParams()
            val supParams = this.supNameOrVariadicNameParams()
            val curlyParams = curlyNodeList()?.asCurlyNodeListOfType<NameOrVariadicName>()
            val namedParamList = mutableListOf<NamedParameterForm>()
            while (this.nodes.isNotEmpty()) {
                val namedParam = this.namedParameterForm(diagnostics) ?: break
                namedParamList.add(namedParam)
            }
            val firstNamedParam = namedParamList.firstOrNull()
            val parenParams = parenNodeList()?.asParenNodeListOfType<NameOrVariadicName>()
            CommandFormCall(
                names =
                    NonBracketNodeList(
                        nodes = nameList,
                        metadata =
                            MetaData(
                                row = backslash.row, column = backslash.column, isInline = null)),
                squareParams = squareParams,
                subParams = subParams,
                supParams = supParams,
                curlyParams = curlyParams,
                namedParams =
                    NonBracketNodeList(
                        nodes = namedParamList,
                        metadata =
                            MetaData(
                                row = firstNamedParam?.metadata?.row ?: -1,
                                column = firstNamedParam?.metadata?.column ?: -1,
                                isInline = null)),
                parenParams = parenParams,
                metadata =
                    MetaData(row = backslash.row, column = backslash.column, isInline = null))
        } else {
            null
        }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    private fun expect(type: TexTalkTokenType): TexTalkToken? =
        if (has(type)) {
            pollToken()
        } else {
            val peek = nodes.firstOrNull()
            diagnostics.add(
                error(
                    message = "Expected a ${type.name}",
                    row = peek?.metadata?.row ?: -1,
                    column = peek?.metadata?.column ?: -1))
            null
        }

    private inline fun <reified T> T?.orError(
        message: String, row: Int, column: Int, default: T
    ): T =
        if (this != null) {
            this!!
        } else {
            diagnostics.add(error(message, row, column))
            default
        }

    private inline fun <reified T : TexTalkNode> ParenNodeList<T>?.errorIfNullOrEmpty(
        diagnostics: MutableList<Diagnostic>, row: Int, column: Int
    ): ParenNodeList<T>? =
        if (this == null || this.nodes.isEmpty()) {
            diagnostics.add(
                error(
                    message = "Expected at least one ${T::class.java.simpleName}",
                    row = row,
                    column = column))
            this
        } else {
            this
        }

    private inline fun <reified T : TexTalkNode> SquareNodeList<T>?.errorIfNullOrEmpty(
        diagnostics: MutableList<Diagnostic>, row: Int, column: Int
    ): SquareNodeList<T>? =
        if (this == null || this.nodes.isEmpty()) {
            diagnostics.add(
                error(
                    message = "Expected at least one ${T::class.java.simpleName}",
                    row = row,
                    column = column))
            this
        } else {
            this
        }

    private inline fun <reified T : TexTalkNode> CurlyNodeList<T>?.errorIfNullOrEmpty(
        diagnostics: MutableList<Diagnostic>, row: Int, column: Int
    ): CurlyNodeList<T>? =
        if (this == null || this.nodes.isEmpty()) {
            diagnostics.add(
                error(
                    message = "Expected at least one ${T::class.java.simpleName}",
                    row = row,
                    column = column))
            this
        } else {
            this
        }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    private inline fun <reified T : TexTalkNode> hasNode() = nodes.isNotEmpty() && nodes[0] is T

    private fun peekNode(): TexTalkNode = nodes[0]

    private fun pollNode(): TexTalkNode = nodes.removeAt(0)

    private inline fun <reified T : TexTalkNode> pollIfHasNode(): T? =
        if (hasNode<T>()) {
            pollNode() as T
        } else {
            null
        }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    private fun has(type: TexTalkTokenType) =
        if (nodes.isEmpty()) {
            false
        } else if (nodes[0] !is TexTalkTokenNode) {
            false
        } else {
            (nodes[0] as TexTalkTokenNode).token.type == type
        }

    private fun hasHas(type1: TexTalkTokenType, type2: TexTalkTokenType) =
        if (nodes.size < 2) {
            false
        } else if (nodes[0] !is TexTalkTokenNode || nodes[1] !is TexTalkTokenNode) {
            false
        } else {
            (nodes[0] as TexTalkTokenNode).token.type == type1 &&
                (nodes[1] as TexTalkTokenNode).token.type == type2
        }

    private fun hasHasHas(
        type1: TexTalkTokenType, type2: TexTalkTokenType, type3: TexTalkTokenType
    ) =
        if (nodes.size < 3) {
            false
        } else if (nodes[0] !is TexTalkTokenNode ||
            nodes[1] !is TexTalkTokenNode ||
            nodes[2] !is TexTalkTokenNode) {
            false
        } else {
            (nodes[0] as TexTalkTokenNode).token.type == type1 &&
                (nodes[1] as TexTalkTokenNode).token.type == type2 &&
                (nodes[2] as TexTalkTokenNode).token.type == type3
        }

    private fun peekToken() = (nodes[0] as TexTalkTokenNode).token

    private fun pollToken() = (nodes.removeAt(0) as TexTalkTokenNode).token

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    private inline fun <reified T : TexTalkNode> ParenNodeList<*>.asParenNodeListOfType():
        ParenNodeList<T> =
        ParenNodeList(
            nodes =
                this.nodes.mapNotNull {
                    if (it is T) {
                        it
                    } else {
                        diagnostics.add(
                            error(
                                message = "Expected an item of type ${T::class.java.simpleName}",
                                row = this.metadata.row,
                                column = this.metadata.column))
                        null
                    }
                },
            metadata = this.metadata)

    private inline fun <reified T : TexTalkNode> SquareNodeList<*>.asSquareNodeListOfType():
        SquareNodeList<T> =
        SquareNodeList(
            nodes =
                this.nodes.mapNotNull {
                    if (it is T) {
                        it
                    } else {
                        diagnostics.add(
                            error(
                                message = "Expected an item of type ${T::class.java.simpleName}",
                                row = this.metadata.row,
                                column = this.metadata.column))
                        null
                    }
                },
            metadata = this.metadata)

    private inline fun <reified T : TexTalkNode> CurlyNodeList<*>.asCurlyNodeListOfType():
        CurlyNodeList<T> =
        CurlyNodeList(
            nodes =
                this.nodes.mapNotNull {
                    if (it is T) {
                        it
                    } else {
                        diagnostics.add(
                            error(
                                message = "Expected an item of type ${T::class.java.simpleName}",
                                row = this.metadata.row,
                                column = this.metadata.column))
                        null
                    }
                },
            metadata = this.metadata)
}

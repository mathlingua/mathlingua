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
import mathlingua.lib.frontend.ast.AssignmentIsFormItem
import mathlingua.lib.frontend.ast.CommandExpression
import mathlingua.lib.frontend.ast.CommandForm
import mathlingua.lib.frontend.ast.DEFAULT_NAME
import mathlingua.lib.frontend.ast.DefinitionIsFormItem
import mathlingua.lib.frontend.ast.Expression
import mathlingua.lib.frontend.ast.ExpressionIsFormItem
import mathlingua.lib.frontend.ast.Function
import mathlingua.lib.frontend.ast.FunctionCall
import mathlingua.lib.frontend.ast.MetaIsForm
import mathlingua.lib.frontend.ast.MetaIsFormItem
import mathlingua.lib.frontend.ast.Name
import mathlingua.lib.frontend.ast.NameOrVariadicName
import mathlingua.lib.frontend.ast.NamedParameterExpression
import mathlingua.lib.frontend.ast.NamedParameterForm
import mathlingua.lib.frontend.ast.SignatureExpression
import mathlingua.lib.frontend.ast.SpecificationIsFormItem
import mathlingua.lib.frontend.ast.SquareParams
import mathlingua.lib.frontend.ast.StatementIsFormItem
import mathlingua.lib.frontend.ast.SubAndRegularParamCall
import mathlingua.lib.frontend.ast.SubParamCall
import mathlingua.lib.frontend.ast.Target
import mathlingua.lib.frontend.ast.TexTalkNode
import mathlingua.lib.frontend.ast.TexTalkToken
import mathlingua.lib.frontend.ast.TexTalkTokenType
import mathlingua.lib.frontend.ast.Tuple
import mathlingua.lib.frontend.ast.TupleExpression
import mathlingua.lib.frontend.ast.VariadicName
import mathlingua.lib.util.BiUnion
import mathlingua.lib.util.BiUnionFirst
import mathlingua.lib.util.BiUnionSecond

internal interface TexTalkSubParser {
    fun parse(): TexTalkNode?
}

internal fun newTexTalkSubParser(
    nodes: MutableList<TreeNode>, diagnostics: MutableList<Diagnostic>
): TexTalkSubParser = TexTalkSubParserImpl(nodes, diagnostics)

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private class TexTalkSubParserImpl(
    private val nodes: MutableList<TreeNode>, private val diagnostics: MutableList<Diagnostic>
) : TexTalkSubParser {

    override fun parse(): TexTalkNode? = parseIntoSingleNode(diagnostics)

    private fun parseIntoSingleNode(diagnostics: MutableList<Diagnostic>): TexTalkNode? {
        // process the list one by one and then run shunting-yard on the result
        // this is where a list of tokens is constructed into a list of parsed nodes that only need
        // to
        // be processed into a tree based on operator usage using the shunting-yard algorithm
        val nodesToProcess = mutableListOf<TexTalkNode>()
        while (this.nodes.isNotEmpty()) {
            val node =
                this.commandExpressionOrCommandForm(diagnostics)?.value
                    ?: this.variadicName(diagnostics) ?: this.name() ?: break
            nodesToProcess.add(node)
        }
        // TODO: FINISH THIS
        return shuntingYard(diagnostics, nodesToProcess)
    }

    private fun shuntingYard(
        diagnostics: MutableList<Diagnostic>, nodes: MutableList<TexTalkNode>
    ): TexTalkNode? {
        // TODO: FINISH THIS
        return nodes.firstOrNull()
    }

    private fun token(type: TexTalkTokenType): TexTalkToken? =
        if (this.has(type)) {
            this.pollToken()
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
            this.expect(diagnostics, TexTalkTokenType.DotDotDot)
            this.pollToken() // move past the ...
            VariadicName(name = name, metadata = name.metadata.copy())
        } else {
            null
        }

    private fun nameOrVariadicName(diagnostics: MutableList<Diagnostic>): NameOrVariadicName? =
        this.name() ?: this.variadicName(diagnostics)

    private fun function(diagnostics: MutableList<Diagnostic>): Function? =
        if (this.hasHas(TexTalkTokenType.Name, TexTalkTokenType.LParen)) {
            val name = name()!!
            Function(
                name = name,
                params =
                    this.parenNameOrVariadicNameParameterList(diagnostics)
                        ?.errorIfNullOrEmpty(
                            diagnostics = diagnostics,
                            row = name.metadata.row,
                            column = name.metadata.column)
                        ?: emptyList(),
                metadata = name.metadata.copy())
        } else {
            null
        }

    private fun subParamCallOrSubAndRegularParamCall(
        diagnostics: MutableList<Diagnostic>
    ): FunctionCall? =
        if (this.hasHasHas(
            TexTalkTokenType.Name, TexTalkTokenType.Underscore, TexTalkTokenType.LParen)) {
            val name = name()!!
            this.expect(diagnostics, TexTalkTokenType.Underscore)
            val subParams =
                this.parenNameOrVariadicNameParameterList(diagnostics)
                    ?.errorIfNullOrEmpty(
                        diagnostics = diagnostics,
                        row = name.metadata.row,
                        column = name.metadata.column)
                    ?: emptyList()
            val params = this.parenNameOrVariadicNameParameterList(diagnostics)
            if (params != null) {
                SubAndRegularParamCall(
                    name = name,
                    subParams = subParams,
                    params = params,
                    metadata = name.metadata.copy())
            } else {
                SubParamCall(name = name, subParams = subParams, metadata = name.metadata.copy())
            }
        } else {
            null
        }

    private fun functionCall(diagnostics: MutableList<Diagnostic>): FunctionCall? =
        this.function(diagnostics) ?: this.subParamCallOrSubAndRegularParamCall(diagnostics)

    private fun squareParams(diagnostics: MutableList<Diagnostic>): SquareParams? {
        val peek = this.nodes.firstOrNull()
        val row = peek?.row() ?: -1
        val column = peek?.column() ?: -1

        val nodes = this.squareNodeList(diagnostics) ?: return null
        if (nodes.isEmpty()) {
            diagnostics.add(
                error(message = "Expected at least one parameter", row = row, column = column))
            return SquareParams(emptyList())
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
            SquareParams(nodes.filterAndError(diagnostics))
        }
    }

    private fun nameOrNamedParameterExpression(
        diagnostics: MutableList<Diagnostic>
    ): BiUnion<Name, NamedParameterExpression, TexTalkNode>? {
        if (!this.hasHas(TexTalkTokenType.Colon, TexTalkTokenType.Name)) {
            return null
        }
        val colon = this.token(TexTalkTokenType.Colon) ?: return null
        val name =
            this.name()
                .orError(diagnostics, "Expected a Name", colon.row, colon.column, DEFAULT_NAME)
        val curlyExpression = this.curlyExpressionParameterList(diagnostics)
        return if (curlyExpression == null) {
            BiUnionFirst(name)
        } else {
            BiUnionSecond(
                NamedParameterExpression(
                    name = name,
                    params =
                        curlyExpression.orError(
                            diagnostics = diagnostics,
                            message = "Expected a {...}",
                            row = colon.row,
                            column = colon.column,
                            default = emptyList()),
                    metadata = MetaData(row = colon.row, column = colon.column, isInline = null)))
        }
    }

    private fun namedParameterForm(diagnostics: MutableList<Diagnostic>): NamedParameterForm? {
        if (!this.hasHas(TexTalkTokenType.Colon, TexTalkTokenType.Name)) {
            return null
        }
        val colon = this.token(TexTalkTokenType.Colon) ?: return null
        val name =
            this.name()
                .orError(diagnostics, "Expected a Name", colon.row, colon.column, DEFAULT_NAME)
        val params =
            this.curlyNameOrVariadicNameParameterList(diagnostics)
                ?.orError(
                    diagnostics = diagnostics,
                    message = "Expected a {...}",
                    row = colon.row,
                    column = colon.column,
                    default = emptyList())
                ?: emptyList()
        return NamedParameterForm(
            name = name,
            params = params,
            metadata = MetaData(row = colon.row, column = colon.column, isInline = null))
    }

    private fun subExpressionParams(diagnostics: MutableList<Diagnostic>): List<Expression>? =
        if (this.hasHas(TexTalkTokenType.Underscore, TexTalkTokenType.LCurly)) {
            val underscore = this.expect(diagnostics, TexTalkTokenType.Underscore)!!
            this.curlyExpressionParameterList(diagnostics)
                ?.errorIfNullOrEmpty(
                    diagnostics = diagnostics, row = underscore.row, column = underscore.column)
                ?: emptyList()
        } else {
            null
        }

    private fun subNameOrVariadicNameParams(
        diagnostics: MutableList<Diagnostic>
    ): List<NameOrVariadicName>? =
        if (this.hasHas(TexTalkTokenType.Underscore, TexTalkTokenType.LCurly)) {
            val underscore = this.expect(diagnostics, TexTalkTokenType.Underscore)!!
            this.curlyNameOrVariadicNameParameterList(diagnostics)
                ?.errorIfNullOrEmpty(
                    diagnostics = diagnostics, row = underscore.row, column = underscore.column)
                ?: emptyList()
        } else {
            null
        }

    private fun supExpressionParams(diagnostics: MutableList<Diagnostic>): List<Expression>? =
        if (this.hasHas(TexTalkTokenType.Caret, TexTalkTokenType.LCurly)) {
            val caret = this.expect(diagnostics, TexTalkTokenType.Caret)!!
            this.curlyExpressionParameterList(diagnostics)
                ?.errorIfNullOrEmpty(
                    diagnostics = diagnostics, row = caret.row, column = caret.column)
                ?: emptyList()
        } else {
            null
        }

    private fun supNameOrVariadicNameParams(
        diagnostics: MutableList<Diagnostic>
    ): List<NameOrVariadicName>? =
        if (this.hasHas(TexTalkTokenType.Caret, TexTalkTokenType.LCurly)) {
            val caret = this.expect(diagnostics, TexTalkTokenType.Caret)!!
            this.curlyNameOrVariadicNameParameterList(diagnostics)
                ?.errorIfNullOrEmpty(
                    diagnostics = diagnostics, row = caret.row, column = caret.column)
                ?: emptyList()
        } else {
            null
        }

    private fun commandExpressionOrCommandForm(
        diagnostics: MutableList<Diagnostic>
    ): BiUnion<CommandExpression, SignatureExpression, TexTalkNode>? =
        if (this.has(TexTalkTokenType.Backslash)) {
            val backslash = this.expect(diagnostics, TexTalkTokenType.Backslash)!!
            val names = mutableListOf<Name>()
            while (this.nodes.isNotEmpty()) {
                val name = name() ?: break
                names.add(name)
                if (this.has(TexTalkTokenType.Dot)) {
                    this.expect(diagnostics, TexTalkTokenType.Dot)
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
            val subParams = this.subExpressionParams(diagnostics)
            val supParams = this.supExpressionParams(diagnostics)
            val curlyParams = this.curlyExpressionParameterList(diagnostics)
            val namesOrNamedParams =
                mutableListOf<BiUnion<Name, NamedParameterExpression, TexTalkNode>>()
            while (this.nodes.isNotEmpty()) {
                val namedParam = this.nameOrNamedParameterExpression(diagnostics) ?: break
                namesOrNamedParams.add(namedParam)
            }
            val parenParams = this.parenExpressionParameterList(diagnostics)

            if (squareParams != null ||
                subParams != null ||
                supParams != null ||
                curlyParams != null ||
                namesOrNamedParams.any { it.value is Expression } ||
                parenParams != null) {
                // process as a CommandExpression
                BiUnionFirst(
                    CommandExpression(
                        names = names,
                        squareParams = squareParams,
                        subParams = subParams,
                        supParams = supParams,
                        curlyParams = curlyParams,
                        namedParams =
                            namesOrNamedParams.mapNotNull {
                                val value = it.value
                                if (value is NamedParameterExpression) {
                                    value
                                } else {
                                    diagnostics.add(
                                        error(
                                            message = "Expected a NamedParameterExpression",
                                            row = backslash.row,
                                            column = backslash.column))
                                    null
                                }
                            },
                        parenParams = parenParams,
                        metadata =
                            MetaData(
                                row = backslash.row, column = backslash.column, isInline = null)))
            } else {
                // process as a CommandForm
                BiUnionSecond(
                    SignatureExpression(
                        names = names,
                        colonNames =
                            namesOrNamedParams.mapNotNull {
                                val value = it.value
                                if (value is Name) {
                                    value
                                } else {
                                    diagnostics.add(
                                        error(
                                            message = "Expected a Named",
                                            row = backslash.row,
                                            column = backslash.column))
                                    null
                                }
                            },
                        metadata =
                            MetaData(
                                row = backslash.row, column = backslash.column, isInline = null)))
            }
        } else {
            null
        }

    private fun commandForm(diagnostics: MutableList<Diagnostic>): CommandForm? =
        if (this.has(TexTalkTokenType.Backslash)) {
            val backslash = this.expect(diagnostics, TexTalkTokenType.Backslash)!!
            val names = mutableListOf<Name>()
            while (this.nodes.isNotEmpty()) {
                val name = name() ?: break
                names.add(name)
                if (this.has(TexTalkTokenType.Dot)) {
                    this.expect(diagnostics, TexTalkTokenType.Dot)
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
            val subParams = this.subNameOrVariadicNameParams(diagnostics)
            val supParams = this.supNameOrVariadicNameParams(diagnostics)
            val curlyParams = this.curlyNameOrVariadicNameParameterList(diagnostics)
            val namedParams = mutableListOf<NamedParameterForm>()
            while (this.nodes.isNotEmpty()) {
                val namedParam = this.namedParameterForm(diagnostics) ?: break
                namedParams.add(namedParam)
            }
            val parenParams = this.parenNameOrVariadicNameParameterList(diagnostics)
            CommandForm(
                names = names,
                squareParams = squareParams,
                subParams = subParams,
                supParams = supParams,
                curlyParams = curlyParams,
                namedParams = namedParams,
                parenParams = parenParams,
                metadata =
                    MetaData(row = backslash.row, column = backslash.column, isInline = null))
        } else {
            null
        }

    /*
    // TODO: FIX THIS
    private fun nameOrCommand(
        diagnostics: MutableList<Diagnostic>
    ): NameOrCommand? = this.name() ?: this.commandExpression(diagnostics)
    */

    // returns `null` if the next node in the list is not a ParenTreeNode
    private fun bracketedNodeList(
        diagnostics: MutableList<Diagnostic>, bracketType: TexTalkTokenType
    ): List<TexTalkNode>? {
        if (this.nodes.isEmpty() || this.nodes.first() !is ParenTreeNode) {
            return null
        }
        val peek = this.nodes.first() as ParenTreeNode
        if (peek.prefix?.type != bracketType) {
            return null
        }
        if (peek.suffix == null) {
            diagnostics.add(
                error(
                    message = "Expected a closing bracket",
                    row = peek.prefix!!.row,
                    column = peek.prefix!!.column))
        }
        val parenTreeNode = this.nodes.removeAt(0) as ParenTreeNode
        return parenTreeNode.treeNodeToTexTalkNodeList(diagnostics)
    }

    private fun parenNodeList(diagnostics: MutableList<Diagnostic>) =
        bracketedNodeList(diagnostics, TexTalkTokenType.LParen)

    private fun squareNodeList(diagnostics: MutableList<Diagnostic>) =
        bracketedNodeList(diagnostics, TexTalkTokenType.LSquare)

    private fun curlyNodeList(diagnostics: MutableList<Diagnostic>) =
        bracketedNodeList(diagnostics, TexTalkTokenType.LCurly)

    private fun squareColonNodeList(diagnostics: MutableList<Diagnostic>) =
        bracketedNodeList(diagnostics, TexTalkTokenType.LSquareColon)

    private fun parenExpressionParameterList(
        diagnostics: MutableList<Diagnostic>
    ): MutableList<Expression>? = this.parenNodeList(diagnostics)?.filterAndError(diagnostics)

    private fun squareExpressionParameterList(
        diagnostics: MutableList<Diagnostic>
    ): MutableList<Expression>? = this.squareNodeList(diagnostics)?.filterAndError(diagnostics)

    private fun curlyExpressionParameterList(
        diagnostics: MutableList<Diagnostic>
    ): MutableList<Expression>? = this.curlyNodeList(diagnostics)?.filterAndError(diagnostics)

    private fun squareColonExpressionParameterList(
        diagnostics: MutableList<Diagnostic>
    ): MutableList<Expression>? = this.squareColonNodeList(diagnostics)?.filterAndError(diagnostics)

    private fun parenNameOrVariadicNameParameterList(
        diagnostics: MutableList<Diagnostic>
    ): MutableList<NameOrVariadicName>? =
        this.parenNodeList(diagnostics)?.filterAndError(diagnostics)

    private fun squareNameOrVariadicNameParameterList(
        diagnostics: MutableList<Diagnostic>
    ): MutableList<NameOrVariadicName>? =
        this.squareNodeList(diagnostics)?.filterAndError(diagnostics)

    private fun curlyNameOrVariadicNameParameterList(
        diagnostics: MutableList<Diagnostic>
    ): MutableList<NameOrVariadicName>? =
        this.curlyNodeList(diagnostics)?.filterAndError(diagnostics)

    private fun squareColonMetaIsFormItemParameterList(
        diagnostics: MutableList<Diagnostic>
    ): MutableList<MetaIsFormItem>? =
        this.squareColonNodeList(diagnostics)?.filterAndError(diagnostics)

    /*
    // TODO: FIX THIS
    private fun variadicIsRhs(
        diagnostics: MutableList<Diagnostic>
    ): VariadicIsRhs? = this.variadicName(diagnostics) ?: this.commandExpression(diagnostics)
     */

    private fun statementIsFormItem(): StatementIsFormItem? =
        if (this.hasNameText("statement")) {
            val next = pollToken()
            StatementIsFormItem(
                metadata = MetaData(row = next.row, column = next.column, isInline = null))
        } else {
            null
        }

    private fun assignmentIsFormItem(): AssignmentIsFormItem? =
        if (this.hasNameText("assignment")) {
            val next = pollToken()
            AssignmentIsFormItem(
                metadata = MetaData(row = next.row, column = next.column, isInline = null))
        } else {
            null
        }

    private fun specificationIsFormItem(): SpecificationIsFormItem? =
        if (this.hasNameText("specification")) {
            val next = pollToken()
            SpecificationIsFormItem(
                metadata = MetaData(row = next.row, column = next.column, isInline = null))
        } else {
            null
        }

    private fun expressionIsFormItem(): ExpressionIsFormItem? =
        if (this.hasNameText("expression")) {
            val next = pollToken()
            ExpressionIsFormItem(
                metadata = MetaData(row = next.row, column = next.column, isInline = null))
        } else {
            null
        }

    private fun definitionIsFormItem(): DefinitionIsFormItem? =
        if (this.hasNameText("definition")) {
            val next = pollToken()
            DefinitionIsFormItem(
                metadata = MetaData(row = next.row, column = next.column, isInline = null))
        } else {
            null
        }

    private fun metaIsFormItem(): MetaIsFormItem? =
        this.statementIsFormItem()
            ?: this.assignmentIsFormItem() ?: this.specificationIsFormItem()
                ?: this.expressionIsFormItem() ?: this.definitionIsFormItem()

    private fun metaIsForm(diagnostics: MutableList<Diagnostic>): MetaIsForm? =
        if (this.has(TexTalkTokenType.LSquareColon)) {
            val prefix = this.peekParenPrefix()
            MetaIsForm(
                items = this.squareColonMetaIsFormItemParameterList(diagnostics) ?: emptyList(),
                metadata =
                    MetaData(
                        row = prefix?.row ?: -1, column = prefix?.column ?: -1, isInline = null))
        } else {
            null
        }

    // TODO: IMPLEMENT THIS
    private fun expression(diagnostics: MutableList<Diagnostic>): Expression? = null

    // TODO: IMPLEMENT THIS
    private fun target(diagnostics: MutableList<Diagnostic>): Target? = null

    private fun tupleOrTupleExpression(
        diagnostics: MutableList<Diagnostic>
    ): BiUnion<Tuple, TupleExpression, TexTalkNode>? {
        val paren = this.peekParenPrefix()
        val nodes = this.parenNodeList(diagnostics) ?: return null

        return if (nodes.all { it is Target }) {
            BiUnionFirst(
                Tuple(
                    targets = nodes.filterAndError(diagnostics),
                    metadata =
                        MetaData(
                            row = paren?.row ?: -1, column = paren?.column ?: -1, isInline = null)))
        } else {
            BiUnionSecond(
                TupleExpression(
                    args = nodes.filterAndError(diagnostics),
                    metadata =
                        MetaData(
                            row = paren?.row ?: -1, column = paren?.column ?: -1, isInline = null)))
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    private inline fun <reified T> T?.orError(
        diagnostics: MutableList<Diagnostic>, message: String, row: Int, column: Int, default: T
    ): T =
        if (this != null) {
            this
        } else {
            diagnostics.add(error(message, row, column))
            default
        }

    private inline fun <reified T> MutableList<T>?.errorIfNullOrEmpty(
        diagnostics: MutableList<Diagnostic>, row: Int, column: Int
    ): MutableList<T>? =
        if (this == null || this.isEmpty()) {
            diagnostics.add(
                error(
                    message = "Expected at least one ${T::class.java.simpleName}",
                    row = row,
                    column = column))
            this
        } else {
            this
        }

    private fun expect(
        diagnostics: MutableList<Diagnostic>, type: TexTalkTokenType
    ): TexTalkToken? =
        if (this.has(type)) {
            this.pollToken()
        } else {
            val peek = nodes.firstOrNull()
            diagnostics.add(
                error(
                    message = "Expected a ${type.name}",
                    row = peek?.row() ?: -1,
                    column = peek?.column() ?: -1))
            null
        }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    private fun has(type: TexTalkTokenType) = this.nodes.has(type)

    private fun hasNameText(text: String) =
        has(TexTalkTokenType.Name) && (this.nodes[0] as AtomTreeNode).token.text == text

    private fun hasHas(type1: TexTalkTokenType, type2: TexTalkTokenType) =
        this.nodes.size >= 2 &&
            this.nodes.elementAtOrNull(0)?.isAtomOfType(type1) == true &&
            this.nodes.elementAtOrNull(1)?.startsWith(type2) == true

    private fun hasHasHas(
        type1: TexTalkTokenType, type2: TexTalkTokenType, type3: TexTalkTokenType
    ) =
        this.nodes.size >= 3 &&
            this.nodes.elementAtOrNull(0)?.isAtomOfType(type1) == true &&
            this.nodes.elementAtOrNull(1)?.isAtomOfType(type2) == true &&
            this.nodes.elementAtOrNull(2)?.startsWith(type3) == true

    private fun pollToken() = (this.nodes.removeAt(0) as AtomTreeNode).token

    private fun peekParenPrefix() =
        if (this.nodes.isNotEmpty() && this.nodes.first() is ParenTreeNode) {
            (this.nodes.first() as ParenTreeNode).prefix
        } else {
            null
        }
}

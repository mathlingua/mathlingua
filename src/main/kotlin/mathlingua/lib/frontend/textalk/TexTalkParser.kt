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
import mathlingua.lib.frontend.DiagnosticOrigin
import mathlingua.lib.frontend.DiagnosticType
import mathlingua.lib.frontend.MetaData
import mathlingua.lib.frontend.ast.Expression
import mathlingua.lib.frontend.ast.Function
import mathlingua.lib.frontend.ast.FunctionAssignment
import mathlingua.lib.frontend.ast.FunctionCall
import mathlingua.lib.frontend.ast.Name
import mathlingua.lib.frontend.ast.NameAssignment
import mathlingua.lib.frontend.ast.NameAssignmentItem
import mathlingua.lib.frontend.ast.NameOrNameAssignment
import mathlingua.lib.frontend.ast.NameOrVariadicName
import mathlingua.lib.frontend.ast.OperatorName
import mathlingua.lib.frontend.ast.Set
import mathlingua.lib.frontend.ast.SubAndRegularParamCall
import mathlingua.lib.frontend.ast.SubAndRegularParamSequence
import mathlingua.lib.frontend.ast.SubParamCall
import mathlingua.lib.frontend.ast.SubParamSequence
import mathlingua.lib.frontend.ast.Target
import mathlingua.lib.frontend.ast.TexTalkNode
import mathlingua.lib.frontend.ast.TexTalkToken
import mathlingua.lib.frontend.ast.TexTalkTokenType
import mathlingua.lib.frontend.ast.Tuple
import mathlingua.lib.frontend.ast.VariadicName

internal interface TexTalkParser {
    fun parse(): TexTalkNode
    fun diagnostics(): List<Diagnostic>
}

internal fun newTexTalkParser(lexer: TexTalkLexer): TexTalkParser = TexTalkParserImpl(lexer)

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private data class TexTalkParserImpl(private val lexer: TexTalkLexer) : TexTalkParser {
    private val diagnostics = mutableListOf<Diagnostic>()

    override fun parse() =
        expression()
            ?: Name(text = "", metadata = MetaData(row = -1, column = -1, isInline = false))

    override fun diagnostics() = diagnostics

    private fun expression(): Expression? =
        function() as Expression?
            ?: tuple() ?: name() ?: setOrSequence() as Expression?

    private fun name(): Name? {
        if (!lexer.has(TexTalkTokenType.Name)) {
            return null
        }
        val next = lexer.next()
        return Name(
            text = next.text,
            metadata = MetaData(row = next.row, column = next.column, isInline = null))
    }

    private fun nameParam(): NameOrVariadicName? {
        val name = name() ?: return null
        val hasDotDotDot =
            if (lexer.has(TexTalkTokenType.DotDotDot)) {
                expect(TexTalkTokenType.DotDotDot)
                true
            } else {
                false
            }
        return if (hasDotDotDot) {
            VariadicName(name = name, metadata = name.metadata)
        } else {
            name
        }
    }

    private fun nameParamList(expectedEnd: TexTalkTokenType): List<NameOrVariadicName> {
        val result = mutableListOf<NameOrVariadicName>()
        while (lexer.hasNext() && !lexer.has(expectedEnd)) {
            val nameParam = nameParam() ?: break
            result.add(nameParam)
            if (!lexer.has(expectedEnd)) {
                expect(TexTalkTokenType.Comma)
            }
        }
        while (lexer.hasNext() && lexer.peek().type != expectedEnd) {
            val next = lexer.next()
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    origin = DiagnosticOrigin.TexTalkParser,
                    message = "Unexpected token ${next.text}",
                    row = next.row,
                    column = next.column))
        }
        return result
    }

    private fun subParams(): List<NameOrVariadicName>? {
        if (!lexer.hasHas(TexTalkTokenType.Underscore, TexTalkTokenType.LCurly)) {
            return null
        }
        expect(TexTalkTokenType.Underscore)
        expect(TexTalkTokenType.LCurly)
        val result = nameParamList(TexTalkTokenType.RCurly)
        expect(TexTalkTokenType.RCurly)
        return result
    }

    private fun regularParams(): List<NameOrVariadicName>? {
        if (!lexer.has(TexTalkTokenType.LParen)) {
            return null
        }
        expect(TexTalkTokenType.LParen)
        val result = nameParamList(TexTalkTokenType.RParen)
        expect(TexTalkTokenType.RParen)
        return result
    }

    private fun operatorName(): OperatorName? {
        if (!lexer.has(TexTalkTokenType.Operator)) {
            return null
        }
        val next = lexer.next()
        return OperatorName(
            text = next.text,
            metadata = MetaData(row = next.row, column = next.column, isInline = null))
    }

    private fun function(): TexTalkNode? {
        // to be a function, the next tokens must be either `<name> "("` or `<name> "_"`
        if (!lexer.hasHas(TexTalkTokenType.Name, TexTalkTokenType.LParen) &&
            !lexer.hasHas(TexTalkTokenType.Name, TexTalkTokenType.Underscore)) {
            return null
        }

        val name = name()!!
        val subParams = subParams()
        val regularParams = regularParams()
        return if (subParams != null && regularParams != null) {
            SubAndRegularParamCall(
                name = name,
                subParams = subParams,
                params = regularParams,
                metadata =
                    MetaData(
                        row = name.metadata.row,
                        column = name.metadata.column,
                        isInline = name.metadata.isInline))
        } else if (subParams != null && regularParams == null) {
            SubParamCall(
                name = name,
                subParams = subParams,
                metadata =
                    MetaData(
                        row = name.metadata.row,
                        column = name.metadata.column,
                        isInline = name.metadata.isInline))
        } else if (subParams == null && regularParams != null) {
            Function(
                name = name,
                params = regularParams,
                metadata =
                    MetaData(
                        row = name.metadata.row,
                        column = name.metadata.column,
                        isInline = name.metadata.isInline))
        } else {
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    origin = DiagnosticOrigin.ChalkTalkParser,
                    message = "Expected a function",
                    row = name.metadata.row,
                    column = name.metadata.column))
            null
        }
    }

    private fun target(): TexTalkNode? {
        val func = function()
        if (func != null) {
            return if (lexer.has(TexTalkTokenType.ColonEqual)) {
                expect(TexTalkTokenType.ColonEqual)
                val rhs = function()
                if (rhs == null) {
                    diagnostics.add(
                        Diagnostic(
                            type = DiagnosticType.Error,
                            origin = DiagnosticOrigin.TexTalkParser,
                            message =
                                "The right hand side of a := must be a function if the left hand side is a function",
                            row = func.metadata.row,
                            column = func.metadata.column))
                    null
                } else if (func is FunctionCall && rhs is FunctionCall) {
                    FunctionAssignment(
                        lhs = func,
                        rhs = rhs,
                        metadata =
                            MetaData(
                                row = func.metadata.row,
                                column = func.metadata.column,
                                isInline = null))
                } else {
                    if (func !is FunctionCall) {
                        diagnostics.add(
                            Diagnostic(
                                type = DiagnosticType.Error,
                                origin = DiagnosticOrigin.ChalkTalkNodeLexer,
                                message = "Expected a FunctionCall",
                                row = func.metadata.row,
                                column = func.metadata.column))
                    }

                    if (rhs !is FunctionCall) {
                        diagnostics.add(
                            Diagnostic(
                                type = DiagnosticType.Error,
                                origin = DiagnosticOrigin.ChalkTalkNodeLexer,
                                message = "Expected a FunctionCall",
                                row = rhs.metadata.row,
                                column = rhs.metadata.column))
                    }

                    null
                }
            } else {
                func
            }
        }

        val name = name()
        if (name != null) {
            return if (lexer.has(TexTalkTokenType.ColonEqual)) {
                expect(TexTalkTokenType.ColonEqual)
                val rhs = nameOrAssignmentItem()
                if (rhs == null) {
                    null
                } else {
                    NameAssignment(
                        lhs = name,
                        rhs = rhs,
                        metadata =
                            MetaData(
                                row = name.metadata.row,
                                column = name.metadata.column,
                                isInline = null))
                }
            } else {
                name
            }
        }

        return nameOrAssignmentItem() as Target?
    }

    private fun targets(expectedEnd: TexTalkTokenType): List<TexTalkNode> {
        val targets = mutableListOf<TexTalkNode>()
        while (lexer.hasNext() && !lexer.has(expectedEnd)) {
            val target = target() ?: break
            targets.add(target)
            if (!lexer.has(expectedEnd)) {
                expect(TexTalkTokenType.Comma)
            }
        }
        while (lexer.hasNext() && !lexer.has(expectedEnd)) {
            val next = lexer.next()
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    origin = DiagnosticOrigin.TexTalkParser,
                    message = "Expected a target",
                    row = next.row,
                    column = next.column))
        }
        return targets
    }

    private fun tuple(): Tuple? {
        if (!lexer.has(TexTalkTokenType.LParen)) {
            return null
        }
        val lParen = expect(TexTalkTokenType.LParen)!!
        val targets = targets(TexTalkTokenType.RParen)
        expect(TexTalkTokenType.RParen)
        return Tuple(
            targets =
                targets
                    .map {
                        if (it !is Target) {
                            diagnostics.add(
                                Diagnostic(
                                    type = DiagnosticType.Error,
                                    message = "Expected a target",
                                    origin = DiagnosticOrigin.TexTalkParser,
                                    row = it.metadata.row,
                                    column = it.metadata.column))
                        }
                        it
                    }
                    .filterIsInstance<Target>(),
            metadata = MetaData(row = lParen.row, column = lParen.column, isInline = null))
    }

    private fun setOrSequence(): TexTalkNode? {
        if (!lexer.has(TexTalkTokenType.LCurly)) {
            return null
        }
        val lCurly = expect(TexTalkTokenType.LCurly)!!
        val targets = targets(TexTalkTokenType.RCurly)
        expect(TexTalkTokenType.RCurly)
        val subParams = subParams()
        return if (subParams != null) {
            // it is a sequence
            if (targets.isEmpty()) {
                diagnostics.add(
                    Diagnostic(
                        type = DiagnosticType.Error,
                        origin = DiagnosticOrigin.TexTalkParser,
                        message = "Expected a function with sub params",
                        row = lCurly.row,
                        column = lCurly.column))
                null
            } else if (targets.size != 1) {
                diagnostics.add(
                    Diagnostic(
                        type = DiagnosticType.Error,
                        origin = DiagnosticOrigin.TexTalkParser,
                        message =
                            "Expected exactly one function with sub params but found ${targets.size}",
                        row = lCurly.row,
                        column = lCurly.column))
                null
            } else {
                when (val first = targets.first()
                ) {
                    is SubParamCall -> {
                        SubParamSequence(
                            func = first,
                            metadata =
                                MetaData(
                                    row = lCurly.row, column = lCurly.column, isInline = null))
                    }
                    is SubAndRegularParamCall -> {
                        SubAndRegularParamSequence(
                            func = first,
                            metadata =
                                MetaData(
                                    row = lCurly.row, column = lCurly.column, isInline = null))
                    }
                    else -> {
                        diagnostics.add(
                            Diagnostic(
                                type = DiagnosticType.Error,
                                origin = DiagnosticOrigin.ChalkTalkNodeLexer,
                                message = "The given function must have sub params",
                                row = lCurly.row,
                                column = lCurly.column))
                        null
                    }
                }
            }
        } else {
            // it is a set
            for (t in targets) {
                if (t !is NameOrNameAssignment) {
                    diagnostics.add(
                        Diagnostic(
                            type = DiagnosticType.Error,
                            origin = DiagnosticOrigin.ChalkTalkNodeLexer,
                            message = "Expected a name or name assignment",
                            row = t.metadata.row,
                            column = t.metadata.column))
                }
            }
            Set(
                items = targets.filterIsInstance<NameOrNameAssignment>(),
                metadata = MetaData(row = lCurly.row, column = lCurly.column, isInline = null))
        }
    }

    private fun nameOrAssignmentItem(): NameAssignmentItem? {
        val func = function()
        if (func != null) {
            return if (func is NameAssignmentItem) {
                func
            } else {
                diagnostics.add(
                    Diagnostic(
                        type = DiagnosticType.Error,
                        origin = DiagnosticOrigin.TexTalkParser,
                        message = "Expected a name assignment item",
                        row = func.metadata.row,
                        column = func.metadata.column))
                null
            }
        }

        val name = name()
        if (name != null) {
            return name
        }

        val op = operatorName()
        if (op != null) {
            return op
        }

        val tuple = tuple()
        if (tuple != null) {
            return tuple
        }

        val setOrSequence = setOrSequence()
        if (setOrSequence != null) {
            return setOrSequence as NameAssignmentItem
        }

        val nextText =
            if (lexer.hasNext()) {
                lexer.next().text
            } else {
                "end of text"
            }

        diagnostics.add(
            Diagnostic(
                type = DiagnosticType.Error,
                origin = DiagnosticOrigin.TexTalkParser,
                message = "Expected a name assignment item but found $nextText",
                row = -1,
                column = -1))

        return null
    }

    private fun expect(type: TexTalkTokenType): TexTalkToken? {
        if (!lexer.hasNext()) {
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    origin = DiagnosticOrigin.TexTalkParser,
                    message = "Expected a $type token but found the end of text",
                    row = -1,
                    column = -1))
            return null
        }
        val next = lexer.next()
        if (next.type != type) {
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    origin = DiagnosticOrigin.TexTalkParser,
                    message = "Expected a $type but found ${next.type}",
                    row = next.row,
                    column = next.column))
        }
        return next
    }
}

private fun TexTalkLexer.has(type: TexTalkTokenType) = this.hasNext() && this.peek().type == type

private fun TexTalkLexer.hasHas(type1: TexTalkTokenType, type2: TexTalkTokenType) =
    this.hasNext() &&
        this.hasNextNext() &&
        this.peek().type == type1 &&
        this.peekPeek().type == type2


fun main() {
    val text = """
        x + y
    """.trimIndent()
    val lexer = newTexTalkLexer(text)
    val parser = newTexTalkParser(lexer)
    val node = parser.parse()
    println(node)
    lexer.diagnostics().forEach(::println)
    parser.diagnostics().forEach(::println)
}

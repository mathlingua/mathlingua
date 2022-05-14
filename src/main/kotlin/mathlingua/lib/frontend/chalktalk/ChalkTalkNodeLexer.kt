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

package mathlingua.lib.frontend.chalktalk

import java.util.LinkedList
import java.util.Stack
import mathlingua.lib.frontend.Diagnostic
import mathlingua.lib.frontend.DiagnosticOrigin
import mathlingua.lib.frontend.DiagnosticType
import mathlingua.lib.frontend.MetaData
import mathlingua.lib.frontend.ast.Argument
import mathlingua.lib.frontend.ast.BeginArgument
import mathlingua.lib.frontend.ast.BeginGroup
import mathlingua.lib.frontend.ast.BeginSection
import mathlingua.lib.frontend.ast.ChalkTalkNode
import mathlingua.lib.frontend.ast.EndArgument
import mathlingua.lib.frontend.ast.EndGroup
import mathlingua.lib.frontend.ast.EndSection
import mathlingua.lib.frontend.ast.Function
import mathlingua.lib.frontend.ast.FunctionAssignment
import mathlingua.lib.frontend.ast.Id
import mathlingua.lib.frontend.ast.Name
import mathlingua.lib.frontend.ast.NameAssignment
import mathlingua.lib.frontend.ast.NameAssignmentItem
import mathlingua.lib.frontend.ast.NameOrNameAssignment
import mathlingua.lib.frontend.ast.NameParam
import mathlingua.lib.frontend.ast.NodeLexerToken
import mathlingua.lib.frontend.ast.OperatorName
import mathlingua.lib.frontend.ast.RegularFunction
import mathlingua.lib.frontend.ast.Set
import mathlingua.lib.frontend.ast.Statement
import mathlingua.lib.frontend.ast.SubAndRegularParamFunction
import mathlingua.lib.frontend.ast.SubAndRegularParamFunctionSequence
import mathlingua.lib.frontend.ast.SubParamFunction
import mathlingua.lib.frontend.ast.SubParamFunctionSequence
import mathlingua.lib.frontend.ast.Target
import mathlingua.lib.frontend.ast.Text
import mathlingua.lib.frontend.ast.TextBlock
import mathlingua.lib.frontend.ast.Tuple

internal interface ChalkTalkNodeLexer {
    fun hasNext(): Boolean
    fun peek(): NodeLexerToken
    fun next(): NodeLexerToken

    fun hasNextNext(): Boolean
    fun peekPeek(): NodeLexerToken
    fun nextNext(): NodeLexerToken

    fun fastForward(): List<NodeLexerToken>

    fun diagnostics(): List<Diagnostic>
}

internal fun newChalkTalkNodeLexer(lexer: ChalkTalkTokenLexer): ChalkTalkNodeLexer =
    ChalkTalkNodeLexerImpl(lexer)

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private class ChalkTalkNodeLexerImpl(private val lexer: ChalkTalkTokenLexer) : ChalkTalkNodeLexer {
    private val tokens = LinkedList<NodeLexerToken>()
    private val beginStack = Stack<NodeLexerToken>()
    private val diagnostics = mutableListOf<Diagnostic>()

    init {
        processTopLevel()
    }

    override fun hasNext() = tokens.isNotEmpty()

    override fun peek() = tokens.element()!!

    override fun next(): NodeLexerToken {
        val next = tokens.remove()!!
        when (next) {
            is BeginGroup, is BeginSection, is BeginArgument -> {
                beginStack.push(next)
            }
            is EndGroup -> {
                if (beginStack.isEmpty() || beginStack.peek() !is BeginGroup) {
                    throw Exception(
                        "Unmatched EndGroup to ${if (beginStack.isEmpty()) {
                        "and empty stack"
                    } else {
                        beginStack.peek().toString()
                    }}")
                }
                beginStack.pop()
            }
            is EndSection -> {
                if (beginStack.isEmpty() || beginStack.peek() !is BeginSection) {
                    throw Exception(
                        "Unmatched EndSection to ${if (beginStack.isEmpty()) {
                        "and empty stack"
                    } else {
                        beginStack.peek().toString()
                    }}")
                }
                beginStack.pop()
            }
            is EndArgument -> {
                if (beginStack.isEmpty() || beginStack.peek() !is BeginArgument) {
                    throw Exception(
                        "Unmatched EndArgument to ${if (beginStack.isEmpty()) {
                        "and empty stack"
                    } else {
                        beginStack.peek().toString()
                    }}")
                }
                beginStack.pop()
            }
            else -> {
                // for other tokens, the stack doesn't need to be modified
            }
        }
        return next
    }

    override fun hasNextNext() = tokens.size >= 2

    override fun peekPeek() = tokens[1]

    override fun nextNext(): NodeLexerToken {
        next()
        return next()
    }

    override fun fastForward(): List<NodeLexerToken> {
        val result = mutableListOf<NodeLexerToken>()
        val initStackSize = beginStack.size
        while (hasNext() && beginStack.size >= initStackSize) {
            result.add(next())
        }
        return result
    }

    override fun diagnostics(): List<Diagnostic> = diagnostics

    private fun expect(type: ChalkTalkTokenType): ChalkTalkToken? {
        if (!lexer.hasNext()) {
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    origin = DiagnosticOrigin.ChalkTalkNodeLexer,
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
                    origin = DiagnosticOrigin.ChalkTalkNodeLexer,
                    message = "Expected a $type but found ${next.type}",
                    row = next.row,
                    column = next.column))
        }
        return next
    }

    private fun statement(isInline: Boolean): Statement? {
        if (!lexer.has(ChalkTalkTokenType.Statement)) {
            return null
        }
        val next = lexer.next()
        return Statement(
            text = next.text,
            metadata = MetaData(row = next.row, column = next.column, isInline = isInline))
    }

    private fun text(isInline: Boolean): Text? {
        if (!lexer.has(ChalkTalkTokenType.Text)) {
            return null
        }
        val next = lexer.next()
        return Text(
            text = next.text,
            metadata = MetaData(row = next.row, column = next.column, isInline = isInline))
    }

    private fun argument(isInline: Boolean): Argument? {
        if (!lexer.hasNext() || lexer.has(ChalkTalkTokenType.Newline)) {
            return null
        }

        val statement = statement(isInline)
        if (statement != null) {
            return statement
        }

        val text = text(isInline)
        if (text != null) {
            return text
        }

        return target(isInline)
    }

    private fun name(isInline: Boolean): Name? {
        if (!lexer.has(ChalkTalkTokenType.Name)) {
            return null
        }
        val next = lexer.next()
        return Name(
            text = next.text,
            metadata = MetaData(row = next.row, column = next.column, isInline = isInline))
    }

    private fun nameParam(isInline: Boolean): NameParam? {
        val name =
            name(isInline)
                ?: if (lexer.has(ChalkTalkTokenType.Underscore)) {
                    val underscore = lexer.next()
                    Name(
                        text = underscore.text,
                        metadata =
                            MetaData(
                                row = underscore.row,
                                column = underscore.column,
                                isInline = isInline))
                } else {
                    null
                }
                    ?: return null
        val hasDotDotDot =
            if (lexer.has(ChalkTalkTokenType.DotDotDot)) {
                expect(ChalkTalkTokenType.DotDotDot)
                true
            } else {
                false
            }
        return NameParam(name = name, isVarArgs = hasDotDotDot)
    }

    private fun nameParamList(isInline: Boolean, expectedEnd: ChalkTalkTokenType): List<NameParam> {
        val result = mutableListOf<NameParam>()
        while (lexer.hasNext() && !lexer.has(expectedEnd)) {
            val nameParam = nameParam(isInline) ?: break
            result.add(nameParam)
            if (!lexer.has(expectedEnd)) {
                expect(ChalkTalkTokenType.Comma)
            }
        }
        while (lexer.hasNext() && lexer.peek().type != expectedEnd) {
            val next = lexer.next()
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    origin = DiagnosticOrigin.ChalkTalkNodeLexer,
                    message = "Unexpected token ${next.text}",
                    row = next.row,
                    column = next.column))
        }
        return result
    }

    private fun subParams(isInline: Boolean): List<NameParam>? {
        if (!lexer.hasHas(ChalkTalkTokenType.Underscore, ChalkTalkTokenType.LCurly)) {
            return null
        }
        expect(ChalkTalkTokenType.Underscore)
        expect(ChalkTalkTokenType.LCurly)
        val result = nameParamList(isInline, ChalkTalkTokenType.RCurly)
        expect(ChalkTalkTokenType.RCurly)
        return result
    }

    private fun regularParams(isInline: Boolean): List<NameParam>? {
        if (!lexer.has(ChalkTalkTokenType.LParen)) {
            return null
        }
        expect(ChalkTalkTokenType.LParen)
        val result = nameParamList(isInline, ChalkTalkTokenType.RParen)
        expect(ChalkTalkTokenType.RParen)
        return result
    }

    private fun operatorName(isInline: Boolean): OperatorName? {
        if (!lexer.has(ChalkTalkTokenType.Operator)) {
            return null
        }
        val next = lexer.next()
        return OperatorName(
            text = next.text,
            metadata = MetaData(row = next.row, column = next.column, isInline = isInline))
    }

    private fun function(isInline: Boolean): Function? {
        // to be a function, the next tokens must be either `<name> "("` or `<name> "_"`
        if (!lexer.hasHas(ChalkTalkTokenType.Name, ChalkTalkTokenType.LParen) &&
            !lexer.hasHas(ChalkTalkTokenType.Name, ChalkTalkTokenType.Underscore)) {
            return null
        }

        val name = name(isInline)!!
        val subParams = subParams(isInline)
        val regularParams = regularParams(isInline)
        return if (subParams != null && regularParams != null) {
            SubAndRegularParamFunction(
                name = name,
                subParams = subParams,
                params = regularParams,
                metadata =
                    MetaData(
                        row = name.metadata.row,
                        column = name.metadata.column,
                        isInline = name.metadata.isInline))
        } else if (subParams != null && regularParams == null) {
            SubParamFunction(
                name = name,
                subParams = subParams,
                metadata =
                    MetaData(
                        row = name.metadata.row,
                        column = name.metadata.column,
                        isInline = name.metadata.isInline))
        } else if (subParams == null && regularParams != null) {
            RegularFunction(
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
                    origin = DiagnosticOrigin.ChalkTalkNodeLexer,
                    message = "Expected a function",
                    row = name.metadata.row,
                    column = name.metadata.column))
            null
        }
    }

    private fun nameOrAssignmentItem(isInline: Boolean): NameAssignmentItem? {
        val func = function(isInline)
        if (func != null) {
            return func
        }

        val name = name(isInline)
        if (name != null) {
            return name
        }

        val op = operatorName(isInline)
        if (op != null) {
            return op
        }

        val tuple = tuple(isInline)
        if (tuple != null) {
            return tuple
        }

        val setOrSequence = setOrSequence(isInline)
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
                origin = DiagnosticOrigin.ChalkTalkNodeLexer,
                message = "Expected a name assignment item but found $nextText",
                row = -1,
                column = -1))

        return null
    }

    private fun target(isInline: Boolean): Target? {
        val func = function(isInline)
        if (func != null) {
            return if (lexer.has(ChalkTalkTokenType.ColonEqual)) {
                expect(ChalkTalkTokenType.ColonEqual)
                val rhs = function(isInline)
                if (rhs == null) {
                    diagnostics.add(
                        Diagnostic(
                            type = DiagnosticType.Error,
                            origin = DiagnosticOrigin.ChalkTalkNodeLexer,
                            message =
                                "The right hand side of a := must be a function if the left hand side is a function",
                            row = func.metadata.row,
                            column = func.metadata.column))
                    null
                } else {
                    FunctionAssignment(
                        lhs = func,
                        rhs = rhs,
                        metadata =
                            MetaData(
                                row = func.metadata.row,
                                column = func.metadata.column,
                                isInline = isInline))
                }
            } else {
                func
            }
        }

        val name = name(isInline)
        if (name != null) {
            return if (lexer.has(ChalkTalkTokenType.ColonEqual)) {
                expect(ChalkTalkTokenType.ColonEqual)
                val rhs = nameOrAssignmentItem(isInline)
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
                                isInline = isInline))
                }
            } else {
                name
            }
        }

        return nameOrAssignmentItem(isInline) as Target?
    }

    private fun targets(isInline: Boolean, expectedEnd: ChalkTalkTokenType): List<Target> {
        val targets = mutableListOf<Target>()
        while (lexer.hasNext() && !lexer.has(expectedEnd)) {
            val target = target(isInline) ?: break
            targets.add(target)
            if (!lexer.has(expectedEnd)) {
                expect(ChalkTalkTokenType.Comma)
            }
        }
        while (lexer.hasNext() && !lexer.has(expectedEnd)) {
            val next = lexer.next()
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    origin = DiagnosticOrigin.ChalkTalkNodeLexer,
                    message = "Expected a target",
                    row = next.row,
                    column = next.column))
        }
        return targets
    }

    private fun tuple(isInline: Boolean): Tuple? {
        if (!lexer.has(ChalkTalkTokenType.LParen)) {
            return null
        }
        val lParen = expect(ChalkTalkTokenType.LParen)!!
        val targets = targets(isInline, ChalkTalkTokenType.RParen)
        expect(ChalkTalkTokenType.RParen)
        return Tuple(
            targets = targets,
            metadata = MetaData(row = lParen.row, column = lParen.column, isInline = isInline))
    }

    private fun setOrSequence(isInline: Boolean): ChalkTalkNode? {
        if (!lexer.has(ChalkTalkTokenType.LCurly)) {
            return null
        }
        val lCurly = expect(ChalkTalkTokenType.LCurly)!!
        val targets = targets(isInline, ChalkTalkTokenType.RCurly)
        expect(ChalkTalkTokenType.RCurly)
        val subParams = subParams(isInline)
        return if (subParams != null) {
            // it is a sequence
            if (targets.isEmpty()) {
                diagnostics.add(
                    Diagnostic(
                        type = DiagnosticType.Error,
                        origin = DiagnosticOrigin.ChalkTalkNodeLexer,
                        message = "Expected a function with sub params",
                        row = lCurly.row,
                        column = lCurly.column))
                null
            } else if (targets.size != 1) {
                diagnostics.add(
                    Diagnostic(
                        type = DiagnosticType.Error,
                        origin = DiagnosticOrigin.ChalkTalkNodeLexer,
                        message =
                            "Expected exactly one function with sub params but found ${targets.size}",
                        row = lCurly.row,
                        column = lCurly.column))
                null
            } else {
                when (val first = targets.first()
                ) {
                    is SubParamFunction -> {
                        SubParamFunctionSequence(
                            func = first,
                            metadata =
                                MetaData(
                                    row = lCurly.row, column = lCurly.column, isInline = isInline))
                    }
                    is SubAndRegularParamFunction -> {
                        SubAndRegularParamFunctionSequence(
                            func = first,
                            metadata =
                                MetaData(
                                    row = lCurly.row, column = lCurly.column, isInline = isInline))
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
                metadata = MetaData(row = lCurly.row, column = lCurly.column, isInline = isInline))
        }
    }

    private fun processTopLevel() {
        var first = true
        while (lexer.hasNext()) {
            if (first || lexer.has(ChalkTalkTokenType.LineBreak)) {
                if (lexer.has(ChalkTalkTokenType.LineBreak)) {
                    lexer.next() // move past the line break
                }

                if (lexer.has(ChalkTalkTokenType.TextBlock)) {
                    val next = lexer.next()
                    tokens.add(
                        TextBlock(
                            text = next.text,
                            metadata =
                                MetaData(row = next.row, column = next.column, isInline = false)))
                    if (lexer.has(ChalkTalkTokenType.Newline)) {
                        lexer.next() // move past the newline
                    }
                } else {
                    processGroup()
                }
            } else {
                val next = lexer.next()
                diagnostics.add(
                    Diagnostic(
                        type = DiagnosticType.Error,
                        origin = DiagnosticOrigin.ChalkTalkNodeLexer,
                        message = "Unexpected text '${next.text}'",
                        row = next.row,
                        column = next.column))
            }
            first = false
        }
    }

    private fun isNextSectionHeader() =
        lexer.hasHas(ChalkTalkTokenType.Name, ChalkTalkTokenType.Colon)

    // continues to the terminator token and absorbs it
    private fun processGroup() {
        var startToken: ChalkTalkToken? = null
        var id: Id? = null

        if (lexer.has(ChalkTalkTokenType.Id)) {
            val next = lexer.next()
            startToken = next
            id =
                Id(
                    text = next.text,
                    metadata = MetaData(row = next.row, column = next.column, isInline = false))
            expect(ChalkTalkTokenType.Newline)
        }

        if (startToken == null && lexer.has(ChalkTalkTokenType.Name)) {
            startToken = lexer.peek()
        }

        val firstSectionName =
            if (lexer.has(ChalkTalkTokenType.Name)) {
                lexer.peek().text
            } else {
                null
            }

        val beginGroup =
            BeginGroup(
                name = firstSectionName,
                metadata =
                    if (startToken != null) {
                        MetaData(row = startToken.row, column = startToken.column, isInline = false)
                    } else {
                        MetaData(row = -1, column = -1, isInline = false)
                    })
        tokens.add(beginGroup)

        if (id != null) {
            tokens.add(id)
        }

        while (lexer.hasNext() && isNextSectionHeader()) {
            processSection()
        }

        if (lexer.has(ChalkTalkTokenType.Newline)) {
            lexer.next()
        }

        tokens.add(EndGroup(metadata = beginGroup.metadata.copy()))
    }

    // processes a section including absorbing any trailing newlines or other tokens at the end of
    // the section
    private fun processSection() {
        val name =
            expect(ChalkTalkTokenType.Name)
                ?: throw Exception("Reached a section header without a name")
        expect(ChalkTalkTokenType.Colon)
            ?: throw Exception("Reached a section header without a colon")

        val beginSection =
            BeginSection(
                name = name.text,
                metadata = MetaData(row = name.row, column = name.column, isInline = false))
        tokens.add(beginSection)

        // handle arguments on the same line
        processArgumentList(startsInline = true)

        if (lexer.has(ChalkTalkTokenType.Indent)) {
            expect(ChalkTalkTokenType.Indent)
            // handle arguments on new lines
            while (lexer.hasNext() && !lexer.has(ChalkTalkTokenType.UnIndent)) {
                val dotSpace = expect(ChalkTalkTokenType.DotSpace)!! // move past the dot space
                if (isNextSectionHeader()) {
                    val beginArgument =
                        BeginArgument(
                            metadata =
                                MetaData(
                                    row = dotSpace.row,
                                    column = dotSpace.column + 2,
                                    isInline = false))
                    tokens.add(beginArgument)
                    processGroup()
                    tokens.add(EndArgument(metadata = beginArgument.metadata.copy()))
                } else {
                    processArgumentList(startsInline = false)
                }
            }

            if (lexer.hasNext()) {
                expect(ChalkTalkTokenType.UnIndent)
            }
        }

        tokens.add(EndSection(metadata = beginSection.metadata.copy()))
    }

    // processes a comma separated list of arguments on a single line and absorbs the trailing
    // newline
    private fun processArgumentList(startsInline: Boolean) {
        var isInline = startsInline
        while (lexer.hasNext()) {
            val firstToken = lexer.peek()

            val arg = argument(isInline)
            if (arg != null) {
                val beginArgument =
                    BeginArgument(
                        metadata =
                            MetaData(
                                row = firstToken.row,
                                column = firstToken.column,
                                isInline = isInline))
                tokens.add(beginArgument)
                tokens.add(arg)
                tokens.add(EndArgument(metadata = beginArgument.metadata.copy()))
            }

            isInline = true

            // a comma is not present for the last of the sections arguments
            if (lexer.has(ChalkTalkTokenType.Comma)) {
                lexer.next() // move past the comma
            } else {
                break
            }
        }

        while (lexer.hasNext() && !lexer.has(ChalkTalkTokenType.Newline)) {
            val next = lexer.next()
            diagnostics.add(
                Diagnostic(
                    type = DiagnosticType.Error,
                    origin = DiagnosticOrigin.ChalkTalkNodeLexer,
                    message = "Unexpected text ${next.text}",
                    row = next.row,
                    column = next.column))
        }

        // a section might not end in a newline if the end of the section is the end of the input
        if (lexer.hasNext()) {
            expect(ChalkTalkTokenType.Newline)
        }
    }
}

private fun ChalkTalkTokenLexer.has(type: ChalkTalkTokenType) =
    this.hasNext() && this.peek().type == type

private fun ChalkTalkTokenLexer.hasHas(type1: ChalkTalkTokenType, type2: ChalkTalkTokenType) =
    this.hasNext() &&
        this.hasNextNext() &&
        this.peek().type == type1 &&
        this.peekPeek().type == type2

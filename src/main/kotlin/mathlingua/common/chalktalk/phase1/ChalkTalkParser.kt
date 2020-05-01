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

package mathlingua.common.chalktalk.phase1

import mathlingua.common.ParseError
import mathlingua.common.chalktalk.phase1.ast.*

interface ChalkTalkParser {
    fun parse(chalkTalkLexer: ChalkTalkLexer): ChalkTalkParseResult
}

data class ChalkTalkParseResult(val root: Root?, val errors: List<ParseError>)

fun newChalkTalkParser(): ChalkTalkParser {
    return ChalkTalkParserImpl()
}

// ------------------------------------------------------------------------------------------------------------------ //

private val INVALID = Phase1Token("INVALID", ChalkTalkTokenType.Invalid, -1, -1)

private class ChalkTalkParserImpl : ChalkTalkParser {

    override fun parse(chalkTalkLexer: ChalkTalkLexer): ChalkTalkParseResult {
        val worker = ParserWorker(chalkTalkLexer)
        val errors = worker.errors
        val root: Root? = worker.root()
        return ChalkTalkParseResult(root, errors)
    }

    private class ParserWorker(private val chalkTalkLexer: ChalkTalkLexer) {
        val errors = mutableListOf<ParseError>()

        fun root(): Root {
            val groups = mutableListOf<Group>()
            while (hasNext()) {
                val grp = group()
                grp ?: break
                groups.add(grp)
            }

            while (hasNext()) {
                val next = next()
                addError("Unrecognized token '" + next.text, next)
            }

            return Root(groups)
        }

        private fun group(): Group? {
            if (has(ChalkTalkTokenType.Linebreak)) {
                next() // absorb the line break
            }

            var id: Phase1Token? = null
            if (has(ChalkTalkTokenType.Id)) {
                id = next()
                expect(ChalkTalkTokenType.Begin)
                expect(ChalkTalkTokenType.End)
            }

            val sections = mutableListOf<Section>()
            while (hasNext()) {
                val sec = section()
                sec ?: break
                sections.add(sec)
            }

            return if (sections.isEmpty()) null else Group(sections, id)
        }

        private fun section(): Section? {
            if (!hasHas(ChalkTalkTokenType.Name, ChalkTalkTokenType.Colon)) {
                return null
            }

            val name = expect(ChalkTalkTokenType.Name)
            expect(ChalkTalkTokenType.Colon)

            val args = mutableListOf<Argument>()
            while (hasNext() && !has(ChalkTalkTokenType.Begin)) {
                val arg = argument()
                arg ?: break
                args.add(arg)

                if (hasNext() && !has(ChalkTalkTokenType.Begin)) {
                    expect(ChalkTalkTokenType.Comma)
                }
            }

            expect(ChalkTalkTokenType.Begin)

            while (hasNext()) {
                val argList = argumentList()
                argList ?: break
                args.addAll(argList)
            }

            expect(ChalkTalkTokenType.End)

            return Section(name, args)
        }

        private fun argumentList(): List<Argument>? {
            if (!hasNext() || !has(ChalkTalkTokenType.DotSpace)) {
                return null
            }

            expect(ChalkTalkTokenType.DotSpace)

            val grp = group()
            if (grp != null) {
                return listOf(Argument(grp))
            }

            val argList = mutableListOf<Argument>()
            val valueArg = argument()
            if (valueArg != null) {
                argList.add(valueArg)

                while (has(ChalkTalkTokenType.Comma)) {
                    next() // absorb the comma
                    val v = argument()
                    v ?: break
                    argList.add(v)
                }
            }

            expect(ChalkTalkTokenType.Begin)
            expect(ChalkTalkTokenType.End)
            return if (argList.isNotEmpty()) argList else null
        }

        private fun token(type: ChalkTalkTokenType): Phase1Token? {
            return if (has(type)) {
                next()
            } else {
                null
            }
        }

        private fun argument(): Argument? {
            val literal = token(ChalkTalkTokenType.Statement)
                ?: token(ChalkTalkTokenType.String)
            if (literal != null) {
                return Argument(literal)
            }

            val mapping = mapping()
            if (mapping != null) {
                return Argument(mapping)
            }

            val target = tupleItem()
            if (target == null) {
                addError("Expected a name, abstraction, tuple, aggregate, or assignment")
                return Argument(INVALID)
            }
            return Argument(target)
        }

        private fun mapping(): Mapping? {
            if (!hasHas(ChalkTalkTokenType.Name, ChalkTalkTokenType.Equals)) {
                return null
            }

            val name = next()
            val equals = next()
            val rhs = if (!hasNext()) {
                addError("A = must be followed by an argument", equals)
                INVALID
            } else {
                val maybeRhs = next()
                if (maybeRhs.type == ChalkTalkTokenType.String) {
                    maybeRhs
                } else {
                    addError("The right hand side of a = must be a string", equals)
                    INVALID
                }
            }
            return Mapping(name, rhs)
        }

        private fun assignment(): Assignment? {
            if (!hasHas(ChalkTalkTokenType.Name, ChalkTalkTokenType.ColonEquals)) {
                return null
            }

            val name = next()
            val colonEquals = next()
            var rhs = assignmentRhs()
            if (rhs == null) {
                addError("A := must be followed by a argument", colonEquals)
                rhs = INVALID
            }
            return Assignment(name, rhs)
        }

        private fun abstraction(): Abstraction? {
            var isEnclosed = false
            var openCurly: Phase1Token? = null
            if (has(ChalkTalkTokenType.LCurly)) {
                isEnclosed = true
                openCurly = next() // skip the {
            }

            val parts = mutableListOf<AbstractionPart>()
            while (hasNext() && !has(ChalkTalkTokenType.RCurly)) {
                if (parts.isNotEmpty()) {
                    expect(ChalkTalkTokenType.Comma)
                }
                val part = abstractionPart()
                if (part == null && isEnclosed) {
                    addError("Expected an abstraction after a {", openCurly)
                }
                part ?: break
                parts.add(part)
                if (!isEnclosed) {
                    // break after adding the first AbstractionPart
                    // if the abstraction is not enclosed, because only
                    // enclosed abstractions can contain multiple parts
                    break
                }
            }

            if (isEnclosed) {
                expect(ChalkTalkTokenType.RCurly)
            }

            return if (!isEnclosed && parts.isEmpty()) {
                null
            } else {
                Abstraction(isEnclosed, parts)
            }
        }

        private fun abstractionPart(): AbstractionPart? {
            if (!has(ChalkTalkTokenType.Name)) {
                return null
            }

            val id = expect(ChalkTalkTokenType.Name)

            var subParams: List<Phase1Token>? = null
            if (has(ChalkTalkTokenType.Underscore)) {
                expect(ChalkTalkTokenType.Underscore)
                if (has(ChalkTalkTokenType.LCurly)) {
                    expect(ChalkTalkTokenType.LCurly)
                    subParams = nameList(ChalkTalkTokenType.RCurly)
                    expect(ChalkTalkTokenType.RCurly)
                } else {
                    subParams = listOf(expect(ChalkTalkTokenType.Name))
                }
            }

            var names: List<Phase1Token>? = null
            if (has(ChalkTalkTokenType.LParen)) {
                expect(ChalkTalkTokenType.LParen)
                names = nameList(ChalkTalkTokenType.RParen)
                expect(ChalkTalkTokenType.RParen)
            }

            return AbstractionPart(id, subParams, names)
        }

        private fun name(): Phase1Token {
            if (!has(ChalkTalkTokenType.Name)) {
                if (hasNext()) {
                    val peek = next()
                    addError("Expected a name, but found ${peek.text}", peek)
                } else {
                    addError("Expected a name, but found the end of input")
                }
                return INVALID
            }
            return next()
        }

        private fun tuple(): Tuple? {
            if (!has(ChalkTalkTokenType.LParen)) {
                return null
            }

            val items = mutableListOf<TupleItem>()

            val leftParen = expect(ChalkTalkTokenType.LParen)
            while (hasNext() && !has(ChalkTalkTokenType.RParen)) {
                if (items.isNotEmpty()) {
                    expect(ChalkTalkTokenType.Comma)
                }

                val item = tupleItem()
                if (item == null) {
                    addError("Encountered a non-tuple item in a tuple", leftParen)
                } else {
                    items.add(item)
                }
            }
            expect(ChalkTalkTokenType.RParen)

            return Tuple(items)
        }

        private fun assignmentRhs(): AssignmentRhs? {
            return tuple() ?: abstraction() ?: name()
        }

        private fun tupleItem(): TupleItem? {
            return assignment() ?: abstraction() ?: assignmentRhs()
        }

        private fun hasNext() = chalkTalkLexer.hasNext()

        private fun next() = chalkTalkLexer.next()

        private fun addError(message: String, token: Phase1Token? = null) {
            val row = token?.row ?: -1
            val column = token?.column ?: -1
            errors.add(ParseError(message, row, column))
        }

        private fun nameList(stopType: ChalkTalkTokenType): List<Phase1Token> {
            val names = mutableListOf<Phase1Token>()
            while (hasNext() && !has(stopType)) {
                var comma: Phase1Token? = null
                if (names.isNotEmpty()) {
                    comma = expect(ChalkTalkTokenType.Comma)
                }

                if (!hasNext()) {
                    addError("Expected a name to follow a comma", comma)
                    break
                }

                val tok = next()
                if (tok.type == ChalkTalkTokenType.Name) {
                    names.add(tok)
                } else {
                    addError("Expected a name but found '${tok.text}'", tok)
                }
            }
            return names
        }

        private fun has(type: ChalkTalkTokenType) = hasNext() && chalkTalkLexer.peek().type == type

        private fun hasHas(
            type: ChalkTalkTokenType,
            thenType: ChalkTalkTokenType
        ) = has(type) &&
                chalkTalkLexer.hasNextNext() && chalkTalkLexer.peekPeek().type == thenType

        private fun expect(type: ChalkTalkTokenType): Phase1Token {
            if (!hasNext() || chalkTalkLexer.peek().type !== type) {
                val peek = if (hasNext()) {
                    chalkTalkLexer.peek()
                } else {
                    INVALID
                }
                addError("Expected a token of type $type but found ${peek.type}", peek)
                return INVALID
            }
            return next()
        }
    }
}

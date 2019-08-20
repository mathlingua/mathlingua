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

package mathlingua.chalktalk.phase1

import mathlingua.chalktalk.phase1.ast.*
import mathlingua.common.ParseError

interface ChalkTalkParser {
  fun parse(chalkTalkLexer: ChalkTalkLexer): ChalkTalkParseResult
}

data class ChalkTalkParseResult(val root: Root?, val errors: List<ParseError>)

fun newChalkTalkParser(): ChalkTalkParser {
  return ChalkTalkParserImpl()
}

private class ChalkTalkParserImpl : ChalkTalkParser {

  override fun parse(chalkTalkLexer: ChalkTalkLexer): ChalkTalkParseResult {
    val worker = ParserWorker(chalkTalkLexer)
    val errors = worker.errors
    val root: Root? = worker.root()
    return ChalkTalkParseResult(root, errors)
  }

  private class ParserWorker(private val chalkTalkLexer: ChalkTalkLexer) {
    val errors: MutableList<ParseError>

    init {
      this.errors = ArrayList()
    }

    fun root(): Root {
      val groups = ArrayList<Group>()
      while (true) {
        val grp = group()
        grp ?: break
        groups.add(grp)
      }

      while (this.chalkTalkLexer.hasNext()) {
        val next = this.chalkTalkLexer.next()
        this.errors.add(
          ParseError(
            "Unrecognized token '" + next.text, next.row, next.column
          )
        )
      }

      return Root(groups)
    }

    private fun group(): Group? {
      if (this.chalkTalkLexer.hasNext() && this.chalkTalkLexer.peek().type === ChalkTalkTokenType.Linebreak) {
        this.chalkTalkLexer.next() // absorb the line break
      }

      var id: ChalkTalkToken? = null
      if (this.chalkTalkLexer.hasNext() && this.chalkTalkLexer.peek().type === ChalkTalkTokenType.Id) {
        id = this.chalkTalkLexer.next()
        expect(ChalkTalkTokenType.Begin)
        expect(ChalkTalkTokenType.End)
      }

      val sections = ArrayList<Section>()
      while (true) {
        val sec = section()
        sec ?: break
        sections.add(sec)
      }

      return if (sections.isEmpty()) null else Group(sections, id)

    }

    private fun section(): Section? {
      val isSec = (this.chalkTalkLexer.hasNext()
        && this.chalkTalkLexer.hasNextNext()
        && this.chalkTalkLexer.peek().type === ChalkTalkTokenType.Name
        && this.chalkTalkLexer.peekPeek().type === ChalkTalkTokenType.Colon)
      if (!isSec) {
        return null
      }

      val name = expect(ChalkTalkTokenType.Name)
      expect(ChalkTalkTokenType.Colon)

      val args = ArrayList<Argument>()
      while (this.chalkTalkLexer.hasNext() && this.chalkTalkLexer.peek().type !== ChalkTalkTokenType.Begin) {
        val arg = argument()
        arg ?: break
        args.add(arg)

        if (this.chalkTalkLexer.hasNext() && this.chalkTalkLexer.peek().type !== ChalkTalkTokenType.Begin) {
          expect(ChalkTalkTokenType.Comma)
        }
      }

      expect(ChalkTalkTokenType.Begin)

      while (true) {
        val argList = argumentList()
        argList ?: break
        args.addAll(argList)
      }

      expect(ChalkTalkTokenType.End)

      return Section(name, args)
    }

    private fun argumentList(): List<Argument>? {
      if (!this.chalkTalkLexer.hasNext() || this.chalkTalkLexer.peek().type !== ChalkTalkTokenType.DotSpace) {
        return null
      }

      expect(ChalkTalkTokenType.DotSpace)

      val grp = group()
      if (grp != null) {
        return listOf(Argument(grp))
      }

      val argList = ArrayList<Argument>()
      val valueArg = argument()
      if (valueArg != null) {
        argList.add(valueArg)

        while (this.chalkTalkLexer.hasNext() && this.chalkTalkLexer.peek().type === ChalkTalkTokenType.Comma) {
          this.chalkTalkLexer.next() // absorb the comma
          val v = argument()
          v ?: break
          argList.add(v)
        }
      }

      expect(ChalkTalkTokenType.Begin)
      expect(ChalkTalkTokenType.End)
      return if (argList.isNotEmpty()) argList else null
    }

    private fun token(type: ChalkTalkTokenType): ChalkTalkToken? {
      return if (has(type)) {
        this.chalkTalkLexer.next()
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
        errors.add(
          ParseError(
            "Expected a name, abstraction, tuple, aggregate, or assignment",
            -1, -1
          )
        )
        val tok = ChalkTalkToken("INVALID", ChalkTalkTokenType.Invalid, -1, -1)
        return Argument(tok)
      }
      return Argument(target)
    }

    private fun mapping(): Mapping? {
      if (!hasHas(ChalkTalkTokenType.Name, ChalkTalkTokenType.Equals)) {
        return null
      }

      val name = this.chalkTalkLexer.next()
      val equals = this.chalkTalkLexer.next()
      var rhs: ChalkTalkToken?
      if (!this.chalkTalkLexer.hasNext()) {
        errors.add(
          ParseError(
            "A = must be followed by an argument",
            equals.row, equals.column
          )
        )
        rhs = ChalkTalkToken("INVALID", ChalkTalkTokenType.Invalid, -1, -1)
      } else {
        val maybeRhs = this.chalkTalkLexer.next()
        if (maybeRhs.type == ChalkTalkTokenType.String) {
          rhs = maybeRhs
        } else {
          ParseError(
            "The right hand side of a = must be a string",
            equals.row, equals.column
          )
          rhs = ChalkTalkToken("INVALID", ChalkTalkTokenType.Invalid, -1, -1)
        }
      }
      return Mapping(name, rhs)
    }

    private fun assignment(): Assignment? {
      if (!hasHas(ChalkTalkTokenType.Name, ChalkTalkTokenType.ColonEquals)) {
        return null
      }

      val name = this.chalkTalkLexer.next()
      val colonEquals = this.chalkTalkLexer.next()
      var rhs = assignmentRhs()
      if (rhs == null) {
        errors.add(
          ParseError(
            "A := must be followed by a argument",
            colonEquals.row, colonEquals.column
          )
        )
        rhs = ChalkTalkToken("INVALID", ChalkTalkTokenType.Invalid, -1, -1)
      }
      return Assignment(name, rhs)
    }

    private fun aggregate(): Aggregate? {
      if (!has(ChalkTalkTokenType.LCurly)) {
        return null
      }

      expect(ChalkTalkTokenType.LCurly)
      val names = nameList(ChalkTalkTokenType.RCurly)
      expect(ChalkTalkTokenType.RCurly)

      return Aggregate(names)
    }

    private fun abstraction(): Abstraction? {
      if (!hasHas(ChalkTalkTokenType.Name, ChalkTalkTokenType.LParen)) {
        return null
      }

      val id = expect(ChalkTalkTokenType.Name)
      expect(ChalkTalkTokenType.LParen)
      val names = nameList(ChalkTalkTokenType.RParen)
      expect(ChalkTalkTokenType.RParen)
      return Abstraction(id, names)
    }

    private fun name(): ChalkTalkToken {
      if (!has(ChalkTalkTokenType.Name)) {
        if (this.chalkTalkLexer.hasNext()) {
          val peek = this.chalkTalkLexer.next()
          this.errors.add(
            ParseError("Expected a name, but found ${peek.text}", peek.row, peek.column)
          )
        } else {
          this.errors.add(ParseError("Expected a name, but found the end of input", -1, -1))
        }
        return ChalkTalkToken("", ChalkTalkTokenType.Invalid, -1, -1)
      }
      return this.chalkTalkLexer.next()
    }

    private fun tuple(): Tuple? {
      if (!has(ChalkTalkTokenType.LParen)) {
        return null
      }

      val items = ArrayList<TupleItem>()

      val leftParen = expect(ChalkTalkTokenType.LParen)
      while (this.chalkTalkLexer.hasNext() &&
        this.chalkTalkLexer.peek().type != ChalkTalkTokenType.RParen
      ) {
        if (items.isNotEmpty()) {
          expect(ChalkTalkTokenType.Comma)
        }

        val item = tupleItem()
        if (item == null) {
          this.errors.add(
            ParseError(
              "Encountered a non-tuple item in a tuple",
              leftParen.row, leftParen.column
            )
          )
        } else {
          items.add(item)
        }
      }
      expect(ChalkTalkTokenType.RParen)

      return Tuple(items)
    }

    private fun assignmentRhs(): AssignmentRhs? {
      return tuple() ?: aggregate() ?: name()
    }

    private fun tupleItem(): TupleItem? {
      return assignment() ?: abstraction() ?: assignmentRhs()
    }

    private fun nameList(stopType: ChalkTalkTokenType): List<ChalkTalkToken> {
      val names = ArrayList<ChalkTalkToken>()
      while (this.chalkTalkLexer.hasNext() && this.chalkTalkLexer.peek().type != stopType) {
        var comma: ChalkTalkToken? = null
        if (names.isNotEmpty()) {
          comma = expect(ChalkTalkTokenType.Comma)
        }

        if (!this.chalkTalkLexer.hasNext()) {
          this.errors.add(
            ParseError("Expected a name to follow a comma", comma!!.row, comma.column)
          )
          break
        }

        val tok = this.chalkTalkLexer.next()
        if (tok.type == ChalkTalkTokenType.Name) {
          names.add(tok)
        } else {
          this.errors.add(
            ParseError("Expected a name but found '${tok.text}'", tok.row, tok.column)
          )
        }
      }
      return names
    }

    private fun has(type: ChalkTalkTokenType): Boolean {
      return this.chalkTalkLexer.hasNext() && this.chalkTalkLexer.peek().type == type
    }

    private fun hasHas(type: ChalkTalkTokenType, thenType: ChalkTalkTokenType): Boolean {
      return has(type) &&
        this.chalkTalkLexer.hasNextNext() && this.chalkTalkLexer.peekPeek().type == thenType
    }

    private fun expect(type: ChalkTalkTokenType): ChalkTalkToken {
      if (!this.chalkTalkLexer.hasNext() || this.chalkTalkLexer.peek().type !== type) {
        val peek = if (this.chalkTalkLexer.hasNext()) {
          this.chalkTalkLexer.peek()
        } else {
          ChalkTalkToken("", ChalkTalkTokenType.Invalid, -1, -1)
        }
        errors.add(
          ParseError(
            "Expected a token of type "
              + type
              + " but "
              + "found "
              + peek.type, peek.row, peek.column
          )
        )
        return ChalkTalkToken("INVALID", ChalkTalkTokenType.Invalid, -1, -1)
      }
      return this.chalkTalkLexer.next()
    }
  }
}

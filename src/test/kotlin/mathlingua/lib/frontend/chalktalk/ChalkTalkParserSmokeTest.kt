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

import kotlin.test.Test
import kotlin.test.assertEquals
import mathlingua.lib.frontend.ast.toCode

class ChalkTalkParserSmokeTest {
    @Test
    fun `correctly parses`() {
        val tokenLexer = newChalkTalkTokenLexer(MATHLINGUA_CODE)
        val nodeLexer = newChalkTalkNodeLexer(tokenLexer)
        val parser = newChalkTalkParser(nodeLexer)
        val doc = parser.parse()
        assertEquals(
            expected = "",
            actual =
                parser.diagnostics().joinToString("\n\n") {
                    // add to the row so that the starting row lines up with the line
                    // numbers in the Kotlin code
                    "${it.type}(${it.row + 40}, ${it.column}, ${it.origin}): ${it.message}"
                })
        assertEquals(expected = MATHLINGUA_CODE, actual = doc.toCode())
    }
}

private val MATHLINGUA_CODE =
    """
[\some.theorem]
Theorem:
given: a, b
where: 'a, b is \something'
suchThat: 'a + b > 0'
then: 'a > 0'
iff: 'b > 0'
Using: 'x + y := \something'
Proof: "Some proof"
Documented:
. loosely: "some description"
. overview: "some overview"
. motivation: "some motivation"
. history: "some history"
. examples: "some example"
. related: "some related"
. discovered: "some discovered"
. notes: "some notes"
References: "@some.reference"


[\some.theorem.without.iff]
Theorem:
given: a, b
where: 'a, b is \something'
suchThat: 'a + b > 0'
then: 'a > 0'


Theorem:
given: x
where: 'x is \something'
suchThat: 'x > 0'
then: 'x > 0'


[\some.axiom]
Axiom:
given: a, b
where: 'a, b is \something'
suchThat: 'a + b > 0'
then: 'a > 0'


Axiom:
given: x
where: 'x is \something'
suchThat: 'x > 0'
then: 'x > 0'


[\some.conjecture]
Conjecture:
given: a, b
where: 'a, b is \something'
suchThat: 'a + b > 0'
then: 'a > 0'


Conjecture:
given: x
where: 'x is \something'
suchThat: 'x > 0'
then: 'x > 0'


[\some.definition]
Defines: X
with: X := (x, y)
given: a, b
when: 'a, b is \something'
suchThat: 'x + y > 0'
means: 'X is \something'
satisfying: '\something'
Providing:
. view:
  as: "\something.else"
  via: 'X'
  by: '\some.theorem'
. equality:
  between: x, y
  provided: 'x = y'
. membership:
  through: 'X'
Codified:
. written: "some written"
. writing: "some writing"
. called: "some called"


[\some.definition]
Defines: n
satisfying:
. generated:
  from: x, f(y)
  when: 'x is \something'
Codified:
. written: "some definition"


[\some.expressing.definition(x)]
Defines: f(x)
means: 'f is \function'
expressing:
. 'f(x) := x'
. matching: 'f(x) := x'
Codified:
. written: "f(x)"


[\some.other.expressing(x)]
Defines: f(x)
means: '\something'
expressing:
. piecewise:
  when: 'x > 0'
  then: 'f(x) := x'
  else: 'f(x) := 0'
Codified:
. written: "f(x)"


[a > b]
States:
given: a, b
when: 'a, b is \something'
suchThat: 'a >= b'
that: '\something'
Codified:
. written: "a > b"


[#some.topic]
Topic:
content: "some content"


Note:
content: "some note"


Specify:
. zero:
  is: "\something"
. positiveInt:
  is: "\something"
. negativeInt:
  is: "\something"
. positiveFloat:
  is: "\something"
. negativeFloat:
  is: "\something"


Theorem:
given: x
then:
. if: 'x > 0'
  then: 'x > 0'


Theorem:
given: x
then:
. iff: 'x > 0'
  then: 'x > 0'


Theorem:
given: x
then:
. not: 'x > 0'


Theorem:
given: x
then:
. and:
  . 'a'
  . 'b'


Theorem:
given: x
then:
. or:
  . 'a'
  . 'b'


Theorem:
given: x
then:
. exists: y
  where: 'y is \something'
  suchThat: 'y > 0'


Theorem:
given: x
then:
. existsUnique: y
  where: 'y is \something'
  suchThat: 'y > 0'


Theorem:
given: x
then:
. forAll: y
  where: 'y is \something'
  suchThat: 'y > 0'
  then: 'y >= 0'


::
some text block
::
""".trimIndent()

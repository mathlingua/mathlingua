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

package mathlingua.textalk


internal class TexTalkParserImplTest {
  /*
  @Test
  fun `parses single name commands`() {
    val text = "\\real::continuous:on{A}to{B}"
    val lexer = newTexTalkLexer(text)
    val parser = newTexTalkParser()
    val result = parser.parse(lexer)
    val expected = TexTalkParseResult(
      root=ExpressionNode(
        children=listOf(
          CommandNode(
            scope=NameNode(
              parts=listOf("real"))
            ,
            name=NameNode(
                parts=listOf("continuous")),
            square=null,
            subSup=null,
            groups= emptyList(),
            namedGroups=listOf(
              NamedGroupNode(
                name=TextNode(
                  type=NodeType.Identifier,
                  text="on"),
                group=GroupNode(
                  type=NodeType.CurlyGroup,
                  expressions=listOf(
                    ExpressionNode(
                      children=listOf(
                        TextNode(
                          type=NodeType.Identifier,
                          text="A")
                      )
                    )
                  )
                )
              ),
              NamedGroupNode(
                name=TextNode(
                  type=NodeType.Identifier,
                  text="to"),
                group=GroupNode(
                  type=NodeType.CurlyGroup,
                  expressions=listOf(
                    ExpressionNode(
                      children=listOf(
                        TextNode(
                          type=NodeType.Identifier,
                          text="B")
                      )
                    )
                  )
                )
              )
            )
          )
        )
      ),
      errors = emptyList()
    )
    Assertions.assertEquals(expected, result)
  }

  @Test
  fun `parses single name commands`() {
    val text = "\\real.analytic::uniformly.continuous:on{A}to{B}"
    val lexer = newTexTalkLexer(text)
    val parser = newTexTalkParser()
    val result = parser.parse(lexer)
    println(prettyPrint(result.toString()))
  }
   */
}

fun prettyPrint(text: String): String {
  var index = 0
  val builder = StringBuffer()
  for (c in text) {
    builder.append(c)
    var printIndent = false
    if (c == '(' || c == '[' || c == '{') {
      index += 2
      printIndent = true
    } else if (c == ')' || c == ']' || c == '}') {
      index -= 2
      printIndent = true
    } else if (c == ',') {
      index -= 1 // the , is already followed by a space
      printIndent = true
    }
    if (printIndent) {
      builder.append('\n')
      for (i in 0 until index) {
        builder.append(' ')
      }
    }
  }
  return builder.toString()
}

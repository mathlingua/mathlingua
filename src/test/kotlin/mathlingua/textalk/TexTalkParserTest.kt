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

import assertk.assertThat
import assertk.assertions.isDataClassEqualTo
import assertk.assertions.isEqualTo
import org.junit.jupiter.api.Test

internal class TexTalkParserTest {
    @Test
    fun `correctly parses an identifier`() {
        val text = "y"
        val parser = newTexTalkParser()
        val lexer = newTexTalkLexer(text)
        val result = parser.parse(lexer)
        assertThat(result.errors.size).isEqualTo(0)
        assertThat(result.root).isDataClassEqualTo(
            ExpressionNode(
                children = listOf(TextNode(type = NodeType.Identifier, text = "y"))
            )
        )
    }
}

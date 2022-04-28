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
import mathlingua.lib.frontend.ast.TexTalkNode

internal data class TexTalkParseResult(val node: TexTalkNode, val diagnostics: List<Diagnostic>)

internal interface TexTalkParser {
    fun parse(): TexTalkParseResult
}

internal fun newTexTalkParser(lexer: TexTalkLexer): TexTalkParser = TexTalkParserImpl(lexer)

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private data class TexTalkParserImpl(private val lexer: TexTalkLexer) : TexTalkParser {
    override fun parse(): TexTalkParseResult {
        TODO("Not yet implemented")
    }
}

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

package mathlingua.lib.frontend

import mathlingua.lib.frontend.chalktalk.ChalkTalkParseResult
import mathlingua.lib.frontend.chalktalk.newChalkTalkNodeLexer
import mathlingua.lib.frontend.chalktalk.newChalkTalkParser
import mathlingua.lib.frontend.chalktalk.newChalkTalkTokenLexer

enum class DiagnosticType {
    Error
}

data class Diagnostic(val type: DiagnosticType, val message: String, val row: Int, val column: Int)

internal data class MetaData(var row: Int, var column: Int, var isInline: Boolean)

internal interface HasMetaData {
    val metadata: MetaData
}

internal object Frontend {
    fun parse(text: String): ChalkTalkParseResult {
        val tokenLexer = newChalkTalkTokenLexer(text)
        val nodeLexer = newChalkTalkNodeLexer(tokenLexer)
        val parser = newChalkTalkParser(nodeLexer)
        return parser.parse()
    }
}

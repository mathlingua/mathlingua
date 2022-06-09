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

import mathlingua.lib.frontend.ast.Document
import mathlingua.lib.frontend.chalktalk.newChalkTalkNodeLexer
import mathlingua.lib.frontend.chalktalk.newChalkTalkParser
import mathlingua.lib.frontend.chalktalk.newChalkTalkTokenLexer

enum class DiagnosticType {
    Error
}

enum class DiagnosticOrigin {
    ChalkTalkTokenLexer,
    ChalkTalkNodeLexer,
    ChalkTalkParser,
    TexTalkLexer,
    TexTalkParser
}

data class Diagnostic(
    val type: DiagnosticType,
    val origin: DiagnosticOrigin,
    val message: String,
    val row: Int,
    val column: Int)

internal data class MetaData(var row: Int, var column: Int, var isInline: Boolean?)

internal interface HasMetaData {
    val metadata: MetaData
}

internal data class FrontendResult(val doc: Document, val diagnostics: List<Diagnostic>)

internal object Frontend {
    fun parse(text: String): FrontendResult {
        val tokenLexer = newChalkTalkTokenLexer(text)
        val nodeLexer = newChalkTalkNodeLexer(tokenLexer)
        val parser = newChalkTalkParser(nodeLexer)

        val doc = parser.parse()
        val diagnostics = mutableListOf<Diagnostic>()
        diagnostics.addAll(tokenLexer.diagnostics())
        diagnostics.addAll(nodeLexer.diagnostics())
        diagnostics.addAll(parser.diagnostics())

        return FrontendResult(doc = doc, diagnostics = diagnostics)
    }
}

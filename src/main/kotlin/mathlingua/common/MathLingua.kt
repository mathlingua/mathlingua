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

package mathlingua.common

import mathlingua.chalktalk.phase1.newChalkTalkLexer
import mathlingua.chalktalk.phase1.newChalkTalkParser
import mathlingua.chalktalk.phase2.Document

data class MathLinguaResult(val document: Document?, val errors: List<ParseError>)

object MathLingua {
    fun parse(input: String): MathLinguaResult {
        val lexer = newChalkTalkLexer(input)

        val allErrors = mutableListOf<ParseError>()
        allErrors.addAll(lexer.errors())

        val parser = newChalkTalkParser()
        val (root, errors) = parser.parse(lexer)
        allErrors.addAll(errors)

        if (root == null) {
            return MathLinguaResult(null, allErrors)
        }

        val documentValidation = Document.validate(root)
        allErrors.addAll(documentValidation.errors)

        return MathLinguaResult(documentValidation.value, allErrors)
    }
}

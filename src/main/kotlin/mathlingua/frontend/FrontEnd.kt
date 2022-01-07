/*
 * Copyright 2020 The MathLingua Authors
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

package mathlingua.frontend

import mathlingua.frontend.chalktalk.phase1.newChalkTalkLexer
import mathlingua.frontend.chalktalk.phase1.newChalkTalkParser
import mathlingua.frontend.chalktalk.phase2.ast.Document
import mathlingua.frontend.chalktalk.phase2.ast.validateDocument
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.Validation
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.support.newLocationTracker
import mathlingua.frontend.support.validationFailure
import mathlingua.frontend.support.validationSuccess

internal data class Parse(val document: Document, val tracker: MutableLocationTracker)

/*
 * Represents a front end of MathLingua processing, specifically the lexing
 * and parsing of text to produce an AST.  A FrontEnd works on a single piece
 * of text.  It does not need to know dependencies between a collection of files
 * or text since that information is not needed to produce and validate an AST.
 * The BackEnd handles such dependencies.
 */
internal object FrontEnd {
    fun parse(input: String): Validation<Document> =
        when (val validation = parseWithLocations(input)
        ) {
            is ValidationSuccess -> validationSuccess(validation.value.document)
            is ValidationFailure -> validationFailure(validation.errors)
        }

    fun parseWithLocations(input: String): Validation<Parse> {
        val lexer = newChalkTalkLexer(input)

        val allErrors = mutableListOf<ParseError>()
        allErrors.addAll(lexer.errors())

        val parser = newChalkTalkParser()
        val (root, errors) = parser.parse(lexer)
        allErrors.addAll(errors)

        if (root == null || allErrors.isNotEmpty()) {
            return validationFailure(allErrors)
        }

        val tracker = newLocationTracker()
        return when (val documentValidation = validateDocument(root, tracker)
        ) {
            is ValidationSuccess ->
                validationSuccess(Parse(document = documentValidation.value, tracker = tracker))
            is ValidationFailure -> {
                allErrors.addAll(documentValidation.errors)
                validationFailure(allErrors)
            }
        }
    }
}

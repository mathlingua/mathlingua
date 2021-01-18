/*
 * Copyright 2020 Google LLC
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

package mathlingua.frontend.chalktalk.phase2.ast.clause

import mathlingua.frontend.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Token
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_STATEMENT
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateByTransform
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.Validation
import mathlingua.frontend.support.validationFailure
import mathlingua.frontend.support.validationSuccess
import mathlingua.frontend.textalk.ExpressionTexTalkNode
import mathlingua.frontend.textalk.newTexTalkLexer
import mathlingua.frontend.textalk.newTexTalkParser

data class Statement(val text: String, val texTalkRoot: Validation<ExpressionTexTalkNode>) :
    Clause {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeStatement(text, texTalkRoot)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(this)
}

fun isStatement(node: Phase1Node) =
    node is Phase1Token && node.type === ChalkTalkTokenType.Statement

fun validateStatement(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateByTransform(
            node = node.resolve(),
            errors = errors,
            default = DEFAULT_STATEMENT,
            message = "Expected a statement",
            transform = {
                if (it is Phase1Token && it.type == ChalkTalkTokenType.Statement) {
                    it
                } else {
                    null
                }
            }) {
            // the text is of the form '...'
            // so the open and closing ' need to be trimmed
            val text = it.text.removeSurrounding("'", "'")

            val texTalkErrors = ArrayList<ParseError>()

            val lexer = newTexTalkLexer(text)
            texTalkErrors.addAll(lexer.errors)

            val parser = newTexTalkParser()
            val result = parser.parse(lexer)
            texTalkErrors.addAll(result.errors)

            val validation: Validation<ExpressionTexTalkNode> =
                if (texTalkErrors.isEmpty()) {
                    validationSuccess(result.root)
                } else {
                    validationFailure(texTalkErrors)
                }

            errors.addAll(texTalkErrors)
            Statement(text = text, texTalkRoot = validation)
        }
    }

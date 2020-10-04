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

package mathlingua.common.chalktalk.phase2.ast.clause

import mathlingua.common.chalktalk.phase1.ast.*
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.support.MutableLocationTracker
import mathlingua.common.support.ParseError
import mathlingua.common.support.Validation
import mathlingua.common.support.validationFailure
import mathlingua.common.support.validationSuccess
import mathlingua.common.textalk.ExpressionTexTalkNode
import mathlingua.common.textalk.newTexTalkLexer
import mathlingua.common.textalk.newTexTalkParser

data class Statement(
    val text: String,
    val texTalkRoot: Validation<ExpressionTexTalkNode>
) : Clause {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeStatement(text, texTalkRoot)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) = chalkTransformer(this)
}

fun isStatement(node: Phase1Node) = node is Phase1Token && node.type === ChalkTalkTokenType.Statement

fun validateStatement(rawNode: Phase1Node, tracker: MutableLocationTracker): Validation<Statement> {
    val node = rawNode.resolve()

    val errors = ArrayList<ParseError>()
    if (node !is Phase1Token) {
        errors.add(
                ParseError(
                        "Cannot convert a to a ChalkTalkToken",
                        getRow(node), getColumn(node)
                )
        )
    }

    val (rawText, type, row, column) = node as Phase1Token
    if (type !== ChalkTalkTokenType.Statement) {
        errors.add(
                ParseError(
                        "Cannot convert a " + node.toCode() + " to a Statement",
                        row, column
                )
        )
        return validationFailure(errors)
    }

    // the text is of the form '...'
    // so the open and closing ' need to be trimmed
    val text = rawText.substring(1, rawText.length - 1)

    val texTalkErrors = ArrayList<ParseError>()

    val lexer = newTexTalkLexer(text)
    texTalkErrors.addAll(lexer.errors)

    val parser = newTexTalkParser()
    val result = parser.parse(lexer)
    texTalkErrors.addAll(result.errors)

    val validation: Validation<ExpressionTexTalkNode> = if (texTalkErrors.isEmpty()) {
        validationSuccess(result.root)
    } else {
        validationFailure(texTalkErrors)
    }

    return validationSuccess(tracker, rawNode, Statement(text, validation))
}

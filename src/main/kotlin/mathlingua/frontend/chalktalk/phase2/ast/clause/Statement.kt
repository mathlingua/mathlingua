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

package mathlingua.frontend.chalktalk.phase2.ast.clause

import mathlingua.frontend.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.frontend.chalktalk.phase1.ast.Inlineable
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Token
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_STATEMENT
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.validateByInlineableTransform
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.Validation
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.textalk.ColonEqualsTexTalkNode
import mathlingua.frontend.textalk.ExpressionTexTalkNode
import mathlingua.frontend.textalk.InTexTalkNode
import mathlingua.frontend.textalk.IsTexTalkNode
import mathlingua.frontend.textalk.newTexTalkLexer
import mathlingua.frontend.textalk.newTexTalkParser

internal data class Statement(
    val text: String,
    val texTalkRoot: Validation<ExpressionTexTalkNode>,
    override val row: Int,
    override val column: Int,
    override val isInline: Boolean
) : Clause, Inlineable {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeStatement(text, texTalkRoot)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(this)
}

internal fun isStatement(node: Phase1Node) =
    node is Phase1Token && node.type === ChalkTalkTokenType.Statement

internal fun validateStatement(
    node: Phase1Node, errors: MutableList<ParseError>, isInline: Boolean
): Statement =
    validateByInlineableTransform(
        node = node.resolve(),
        errors = errors,
        default = DEFAULT_STATEMENT,
        message = "Expected a statement",
        isInline = isInline,
        transform = {
            if (it is Phase1Token && it.type == ChalkTalkTokenType.Statement) {
                it
            } else {
                null
            }
        }) { n, row, column, inline ->
        // the text is of the form '...'
        // so the open and closing ' need to be trimmed
        val text = n.text.removeSurrounding("'", "'")

        val texTalkErrors = ArrayList<ParseError>()

        val lexer = newTexTalkLexer(text)
        texTalkErrors.addAll(lexer.errors)

        val parser = newTexTalkParser()
        val result = parser.parse(lexer)
        texTalkErrors.addAll(result.errors)

        val validation: Validation<ExpressionTexTalkNode> =
            if (texTalkErrors.isEmpty()) {
                ValidationSuccess(result.root)
            } else {
                ValidationFailure(texTalkErrors)
            }

        errors.addAll(texTalkErrors)
        Statement(text = text, texTalkRoot = validation, row, column, inline)
    }

internal fun Statement.isIsStatement() =
    when (val validation = this.texTalkRoot
    ) {
        is ValidationSuccess -> {
            val root = validation.value
            root.children.size == 1 && root.children[0] is IsTexTalkNode
        }
        is ValidationFailure -> false
    }

internal fun Statement.isInStatement() =
    when (val validation = this.texTalkRoot
    ) {
        is ValidationSuccess -> {
            val root = validation.value
            root.children.size == 1 && root.children[0] is InTexTalkNode
        }
        is ValidationFailure -> false
    }

internal fun Statement.isColonEqualsStatement() =
    when (val validation = this.texTalkRoot
    ) {
        is ValidationSuccess -> {
            val root = validation.value
            root.children.size == 1 && root.children[0] is ColonEqualsTexTalkNode
        }
        is ValidationFailure -> false
    }

internal fun Statement.getLeftHandSideTargets(): List<Target>? =
    if (this.isIsStatement()) {
        val isStatement = (this.texTalkRoot as ValidationSuccess).value.children[0] as IsTexTalkNode
        val result = mutableListOf<Target>()
        for (exp in isStatement.lhs.items) {
            for (c in exp.children) {
                if (c is Target) {
                    result.add(c)
                }
            }
        }
        result
    } else if (this.isInStatement()) {
        val inStatement = (this.texTalkRoot as ValidationSuccess).value.children[0] as InTexTalkNode
        val result = mutableListOf<Target>()
        for (exp in inStatement.lhs.items) {
            for (c in exp.children) {
                if (c is Target) {
                    result.add(c)
                }
            }
        }
        result
    } else if (this.isColonEqualsStatement()) {
        val colonEqualsStatement =
            (this.texTalkRoot as ValidationSuccess).value.children[0] as ColonEqualsTexTalkNode
        val result = mutableListOf<Target>()
        for (exp in colonEqualsStatement.lhs.items) {
            for (c in exp.children) {
                if (c is Target) {
                    result.add(c)
                }
            }
        }
        result
    } else {
        null
    }

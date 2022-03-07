/*
 * Copyright 2022 The MathLingua Authors
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

package mathlingua.frontend.chalktalk.phase2.ast

import mathlingua.backend.transform.Signature
import mathlingua.backend.transform.signature
import mathlingua.frontend.chalktalk.phase2.ast.clause.ClauseListNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.HasUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.CalledSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.SatisfyingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.viewing.ViewAsSection
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.textalk.AsTexTalkNode
import mathlingua.frontend.textalk.ColonEqualsTexTalkNode
import mathlingua.frontend.textalk.ExpressionTexTalkNode
import mathlingua.frontend.textalk.InTexTalkNode
import mathlingua.frontend.textalk.IsTexTalkNode
import mathlingua.frontend.textalk.TexTalkNode
import mathlingua.frontend.textalk.findColonEqualsLhsSymbols
import mathlingua.frontend.textalk.findColonEqualsRhsSignatures
import mathlingua.frontend.textalk.findInRhsSignatures
import mathlingua.frontend.textalk.findIsLhsSymbols
import mathlingua.frontend.textalk.findIsRhsSignatures

internal fun Phase2Node.deepForEachTopDown(fn: (node: Phase2Node) -> Boolean) {
    val doContinue = fn(this)
    if (doContinue) {
        this.forEach { it.deepForEachTopDown(fn) }
    }
}

internal fun Phase2Node.getNonIsNonInNonAsStatementsNonInAsSections():
    List<Pair<Statement, TexTalkNode>> {
    val result = mutableListOf<Pair<Statement, TexTalkNode>>()
    this.deepForEachTopDown { n ->
        if (n is Statement) {
            when (val validation = n.texTalkRoot
            ) {
                is ValidationSuccess -> {
                    val exp = validation.value
                    if (exp.children.size != 1 ||
                        (exp.children[0] !is IsTexTalkNode &&
                            exp.children[0] !is InTexTalkNode &&
                            exp.children[0] !is AsTexTalkNode)) {
                        result.add(
                            Pair(
                                n,
                                exp.transform {
                                    if (it is IsTexTalkNode || it is InTexTalkNode) {
                                        ExpressionTexTalkNode(children = emptyList())
                                    } else {
                                        it
                                    }
                                }))
                    }
                }
                else -> {
                    // invalid statements are not processed since it cannot be determined
                    // if they are of the form `... is ...`
                }
            }
        }
        n !is ViewAsSection
    }
    return result
}

internal fun Phase2Node.findAllTexTalkNodes(): List<TexTalkNode> {
    val result = mutableListOf<TexTalkNode>()
    this.deepForEachTopDown { n ->
        if (n is Statement) {
            when (val validation = n.texTalkRoot
            ) {
                is ValidationSuccess -> {
                    result.add(validation.value)
                }
            }
        }
        true
    }
    return result
}

internal fun Phase2Node.findAllStatements(): List<Pair<Statement, List<DefinesGroup>>> {
    val pairs = mutableListOf<Pair<Statement, HasUsingSection?>>()
    findAllStatementsImpl(
        this,
        if (this is HasUsingSection) {
            this
        } else {
            null
        },
        pairs)
    return pairs.map {
        val aliases = mutableListOf<DefinesGroup>()
        val usingSection = it.second?.usingSection
        if (usingSection != null) {
            for (clause in usingSection.clauses.clauses) {
                if (clause is Statement &&
                    clause.texTalkRoot is ValidationSuccess &&
                    clause.texTalkRoot.value.children.firstOrNull() is ColonEqualsTexTalkNode) {

                    val colonEquals =
                        clause.texTalkRoot.value.children.first() as ColonEqualsTexTalkNode
                    val lhsItems = colonEquals.lhs.items
                    val rhsItems = colonEquals.rhs.items
                    if (lhsItems.size != rhsItems.size) {
                        throw RuntimeException(
                            "The left-hand-side and right-hand-side of a := must have the same number of " +
                                "comma separated expressions")
                    }
                    for (i in lhsItems.indices) {
                        val lhs = lhsItems[i]
                        val rhs = rhsItems[i]
                        val id =
                            IdStatement(
                                text = lhs.toCode(),
                                texTalkRoot = ValidationSuccess(lhs),
                                row = -1,
                                column = -1)
                        val syntheticDefines =
                            DefinesGroup(
                                signature = id.signature(),
                                id = id,
                                definesSection =
                                    DefinesSection(targets = emptyList(), row = -1, column = -1),
                                whereSection = null,
                                givenSection = null,
                                whenSection = null,
                                meansSection = null,
                                satisfyingSection =
                                    SatisfyingSection(
                                        clauses =
                                            ClauseListNode(
                                                clauses = emptyList(), row = -1, column = -1),
                                        row = -1,
                                        column = -1),
                                expressingSection = null,
                                providingSection = null,
                                usingSection = null,
                                writingSection = null,
                                writtenSection =
                                    WrittenSection(
                                        forms = listOf(rhs.toCode()), row = -1, column = -1),
                                calledSection =
                                    CalledSection(forms = emptyList(), row = -1, column = -1),
                                metaDataSection = null,
                                row = -1,
                                column = -1)
                        aliases.add(syntheticDefines)
                    }
                }
            }
        }
        Pair(first = it.first, second = aliases)
    }
}

private fun findAllStatementsImpl(
    n: Phase2Node,
    hasUsingNode: HasUsingSection?,
    result: MutableList<Pair<Statement, HasUsingSection?>>
) {
    if (n is Statement) {
        result.add(Pair(n, hasUsingNode))
    }
    n.forEach {
        findAllStatementsImpl(
            it,
            if (hasUsingNode != null) {
                hasUsingNode
            } else {
                if (n is HasUsingSection) {
                    n
                } else {
                    null
                }
            },
            result)
    }
}

internal fun Phase2Node.findIsRhsSignatures(): List<Signature> {
    val result = mutableListOf<Signature>()
    this.deepForEachTopDown { n ->
        if (n is Statement) {
            when (n.texTalkRoot) {
                is ValidationSuccess -> {
                    val location = Location(n.row, n.column)
                    n.texTalkRoot.value.findIsRhsSignatures(location, result)
                }
                else -> {
                    // ignore statements that do not parse
                }
            }
        }
        true
    }
    return result
}

internal fun Phase2Node.findInRhsSignatures(): List<Signature> {
    val result = mutableListOf<Signature>()
    this.deepForEachTopDown { n ->
        if (n is Statement) {
            when (n.texTalkRoot) {
                is ValidationSuccess -> {
                    val location = Location(n.row, n.column)
                    n.texTalkRoot.value.findInRhsSignatures(location, result)
                }
                else -> {
                    // ignore statements that do not parse
                }
            }
        }
        true
    }
    return result
}

internal fun Phase2Node.findColonEqualsRhsSignatures(): List<Signature> {
    val result = mutableListOf<Signature>()
    this.deepForEachTopDown { n ->
        if (n is Statement) {
            when (n.texTalkRoot) {
                is ValidationSuccess -> {
                    val location = Location(n.row, n.column)
                    n.texTalkRoot.value.findColonEqualsRhsSignatures(location, result)
                }
                else -> {
                    // ignore statements that do not parse
                }
            }
        }
        true
    }
    return result
}

internal fun Phase2Node.findIsLhsSymbols(): List<Pair<String, Location>> {
    val result = mutableListOf<Pair<String, Location>>()
    this.deepForEachTopDown { n ->
        if (n is Statement) {
            when (n.texTalkRoot) {
                is ValidationSuccess -> {
                    val location = Location(n.row, n.column)
                    n.texTalkRoot.value.findIsLhsSymbols(location, result)
                }
                else -> {
                    // ignore statements that do not parse
                }
            }
        }
        true
    }
    return result
}

internal fun Phase2Node.findColonEqualsLhsSymbols(): List<Pair<String, Location>> {
    val result = mutableListOf<Pair<String, Location>>()
    this.deepForEachTopDown { n ->
        if (n is Statement) {
            when (n.texTalkRoot) {
                is ValidationSuccess -> {
                    val location = Location(n.row, n.column)
                    n.texTalkRoot.value.findColonEqualsLhsSymbols(location, result)
                }
                else -> {
                    // ignore statements that do not parse
                }
            }
        }
        true
    }
    return result
}

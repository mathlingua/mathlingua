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

package mathlingua.common.transform

import mathlingua.common.Validation
import mathlingua.common.chalktalk.phase2.Clause
import mathlingua.common.chalktalk.phase2.ClauseListNode
import mathlingua.common.chalktalk.phase2.DefinesGroup
import mathlingua.common.chalktalk.phase2.ForGroup
import mathlingua.common.chalktalk.phase2.ForSection
import mathlingua.common.chalktalk.phase2.Identifier
import mathlingua.common.chalktalk.phase2.IfGroup
import mathlingua.common.chalktalk.phase2.IfSection
import mathlingua.common.chalktalk.phase2.MeansSection
import mathlingua.common.chalktalk.phase2.Phase2Node
import mathlingua.common.chalktalk.phase2.RepresentsGroup
import mathlingua.common.chalktalk.phase2.Statement
import mathlingua.common.chalktalk.phase2.ThenSection
import mathlingua.common.chalktalk.phase2.WhereSection
import mathlingua.common.textalk.Command
import mathlingua.common.textalk.ExpressionTexTalkNode
import mathlingua.common.textalk.IsTexTalkNode
import mathlingua.common.textalk.ParametersTexTalkNode
import mathlingua.common.textalk.TexTalkNode
import mathlingua.common.textalk.TexTalkNodeType
import mathlingua.common.textalk.TextTexTalkNode
import mathlingua.common.textalk.populateParents

private fun hasIsNodeParent(node: TexTalkNode?): Boolean {
    if (node == null) {
        return false
    }

    if (node is IsTexTalkNode) {
        return true
    }

    return hasIsNodeParent(node.parent)
}

fun moveInlineCommandsToIsNode(
    node: Clause,
    sigToName: Map<String, String>,
    shouldProcessChalk: (node: Phase2Node) -> Boolean,
    shouldProcessTex: (node: TexTalkNode) -> Boolean
): Clause {
    if (!shouldProcessChalk(node)) {
        return node
    }

    fun shouldProcessTexNodes(node: TexTalkNode): Boolean {
        if (!shouldProcessTex(node)) {
            return false
        }

        return !hasIsNodeParent(node)
    }

    val sigsFound = mutableSetOf<String>()
    val newNode = replaceCommands(node, sigToName, sigsFound, shouldProcessChalk, ::shouldProcessTexNodes) as Clause

    if (sigsFound.isEmpty()) {
        return node
    }

    return ForGroup(
        forSection = ForSection(
            targets = sigsFound.map { Identifier(name = sigToName[it]!!) }
        ),
        whereSection = WhereSection(
            clauses = ClauseListNode(
                clauses = sigsFound.map {
                    val isNode = IsTexTalkNode(
                        parent = null,
                        lhs = ParametersTexTalkNode(
                            parent = null,
                            items = listOf(
                                ExpressionTexTalkNode(
                                    parent = null,
                                    children = listOf(
                                        TextTexTalkNode(
                                            parent = null,
                                            type = TexTalkNodeType.Identifier,
                                            text = sigToName[it]!!
                                        )
                                    )
                                )
                            )
                        ),
                        rhs = ParametersTexTalkNode(
                            parent = null,
                            items = listOf(
                                ExpressionTexTalkNode(
                                    parent = null,
                                    children = listOf(
                                        TextTexTalkNode(
                                            parent = null,
                                            type = TexTalkNodeType.Identifier,
                                            text = it
                                        )
                                    )
                                )
                            )
                        )
                    )

                    Statement(
                        text = isNode.toCode(),
                        texTalkRoot = Validation.success(
                            populateParents(
                                ExpressionTexTalkNode(
                                    parent = null,
                                   children = listOf(isNode)
                                )
                            )
                        )
                    )
                }
            )
        ),
        thenSection = ThenSection(
            clauses = ClauseListNode(
                clauses = listOf(newNode)
            )
        )
    )
}

fun replaceRepresents(
    node: Phase2Node,
    represents: List<RepresentsGroup>,
    filter: (node: Phase2Node) -> Boolean = { true }
): Phase2Node {
    val repMap = mutableMapOf<String, RepresentsGroup>()
    for (rep in represents) {
        val sig = rep.signature
        if (sig != null) {
            repMap[sig] = rep
        }
    }

    fun chalkTransformer(node: Phase2Node): Phase2Node {
        if (!filter(node)) {
            return node
        }

        if (node !is Statement) {
            return node
        }

        if (!node.texTalkRoot.isSuccessful ||
            node.texTalkRoot.value!!.children.size != 1 ||
            node.texTalkRoot.value.children[0] !is Command) {
            return node
        }

        val command = node.texTalkRoot.value.children[0] as Command
        val sig = getCommandSignature(command).toCode()

        if (!repMap.containsKey(sig)) {
            return node
        }

        val rep = repMap[sig]!!
        val cmdVars = getVars(command)
        val defIndirectVars = getRepresentsIdVars(rep)

        val map = mutableMapOf<String, String>()
        for (i in cmdVars.indices) {
            map[defIndirectVars[i]] = cmdVars[i]
        }

        return renameVars(buildIfThen(rep), map)
    }

    return node.transform(::chalkTransformer)
}

fun toCanonicalForm(def: DefinesGroup): DefinesGroup {
    return DefinesGroup(
        signature = def.signature,
        id = def.id,
        definesSection = def.definesSection,
        assumingSection = null,
        meansSection = MeansSection(
            clauses = ClauseListNode(
                clauses = listOf(buildIfThen(def))
            )
        ),
        aliasSection = def.aliasSection,
        metaDataSection = def.metaDataSection
    )
}

fun buildIfThen(def: DefinesGroup): IfGroup {
    return IfGroup(
        ifSection = IfSection(
            clauses = def.assumingSection?.clauses ?: ClauseListNode(emptyList())
        ),
        thenSection = ThenSection(
            clauses = def.meansSection.clauses
        )
    )
}

fun buildIfThen(rep: RepresentsGroup): IfGroup {
    return IfGroup(
        ifSection = IfSection(
            clauses = rep.assumingSection?.clauses ?: ClauseListNode(emptyList())
        ),
        thenSection = ThenSection(
            clauses = rep.thatSection.clauses
        )
    )
}

fun getDefinesDirectVars(def: DefinesGroup): List<String> {
    val vars = mutableListOf<String>()
    for (target in def.definesSection.targets) {
        vars.addAll(getVars(target))
    }
    return vars
}

fun getDefinesIdVars(def: DefinesGroup): List<String> {
    val vars = mutableListOf<String>()
    if (def.id.texTalkRoot.isSuccessful) {
        vars.addAll(getVars(def.id.texTalkRoot.value!!))
    }
    return vars
}

fun getRepresentsIdVars(rep: RepresentsGroup): List<String> {
    val vars = mutableListOf<String>()
    if (rep.id.texTalkRoot.isSuccessful) {
        vars.addAll(getVars(rep.id.texTalkRoot.value!!))
    }
    return vars
}

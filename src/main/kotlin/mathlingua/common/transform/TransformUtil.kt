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
import mathlingua.common.textalk.getAncestry

fun moveInlineCommandsToIsNode(
    stmt: Statement,
    sigToName: Map<String, String>,
    shouldProcessChalk: (node: Phase2Node) -> Boolean,
    shouldProcessTex: (node: TexTalkNode) -> Boolean
): Clause {
    val validation = stmt.texTalkRoot
    if (!validation.isSuccessful) {
        return stmt
    }
    val root = validation.value!!

    if (!shouldProcessChalk(stmt)) {
        return stmt
    }

    fun shouldProcessTexNodes(node: TexTalkNode): Boolean {
        if (!shouldProcessTex(node)) {
            return false
        }

        return !getAncestry(root, node).any { it is IsTexTalkNode }
    }

    val commandsFound = mutableListOf<Command>()
    val newNode = replaceCommands(stmt, sigToName, commandsFound, shouldProcessChalk, ::shouldProcessTexNodes) as Clause

    if (commandsFound.isEmpty()) {
        return stmt
    }

    val uniqueSignatures = commandsFound.map {
        getCommandSignature(it).toCode()
    }.distinct().filter { sigToName.containsKey(it) }

    if (uniqueSignatures.isEmpty()) {
        return stmt
    }

    val tmp = mutableSetOf<String>()
    val distinctCommands = mutableListOf<Command>()
    for (cmd in commandsFound) {
        val code = cmd.toCode()
        if (!tmp.contains(code)) {
            distinctCommands.add(cmd)
            tmp.add(code)
        }
    }

    println(sigToName)

    return ForGroup(
        forSection = ForSection(
            targets = uniqueSignatures.map { println("it=" + it); Identifier(name = sigToName[it]!!) }
        ),
        whereSection = WhereSection(
            clauses = ClauseListNode(
                clauses = distinctCommands.map {
                    val isNode = IsTexTalkNode(
                        lhs = ParametersTexTalkNode(
                            items = listOf(
                                ExpressionTexTalkNode(
                                    children = listOf(
                                        TextTexTalkNode(
                                            type = TexTalkNodeType.Identifier,
                                            text = sigToName[getCommandSignature(it).toCode()]!!
                                        )
                                    )
                                )
                            )
                        ),
                        rhs = ParametersTexTalkNode(
                            items = listOf(
                                ExpressionTexTalkNode(
                                    children = listOf(it)
                                )
                            )
                        )
                    )

                    Statement(
                        text = isNode.toCode(),
                        texTalkRoot = Validation.success(
                            ExpressionTexTalkNode(
                                children = listOf(isNode)
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

fun replaceIsNodes(
    node: Phase2Node,
    defs: List<DefinesGroup>,
    filter: (node: Phase2Node) -> Boolean = { true }
): Phase2Node {
    val defMap = mutableMapOf<String, DefinesGroup>()
    for (def in defs) {
        val sig = def.signature
        if (sig != null) {
            defMap[sig] = def
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
            node.texTalkRoot.value.children[0] !is IsTexTalkNode) {
            return node
        }

        val isNode = node.texTalkRoot.value.children[0] as IsTexTalkNode
        if (isNode.rhs.items.size != 1 ||
            isNode.rhs.items[0].children.size != 1 ||
            isNode.rhs.items[0].children[0] !is Command) {
            return node
        }

        val command = isNode.rhs.items[0].children[0] as Command
        val sig = getCommandSignature(command).toCode()

        if (!defMap.containsKey(sig)) {
            return node
        }

        val def = defMap[sig]!!
        val cmdVars = getVars(command)

        val defDirectVars = getDefinesDirectVars(def)
        val defIndirectVars = getDefinesIdVars(def)

        if (cmdVars.size != defIndirectVars.size) {
            return node
        }

        val map = mutableMapOf<String, String>()
        for (i in cmdVars.indices) {
            map[defIndirectVars[i]] = cmdVars[i]
        }

        val lhsVars = getVars(isNode.lhs)
        if (lhsVars.size != defDirectVars.size) {
            return node
        }

        for (i in lhsVars.indices) {
            map[defDirectVars[i]] = lhsVars[i]
        }

        return renameVars(buildIfThen(def), map)
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

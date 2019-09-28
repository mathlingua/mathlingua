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

import mathlingua.common.ValidationFailure
import mathlingua.common.ValidationSuccess
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase1.ast.Phase1Token
import mathlingua.common.chalktalk.phase2.AbstractionNode
import mathlingua.common.chalktalk.phase2.AggregateNode
import mathlingua.common.chalktalk.phase2.AssignmentNode
import mathlingua.common.chalktalk.phase2.Identifier
import mathlingua.common.chalktalk.phase2.Phase2Node
import mathlingua.common.chalktalk.phase2.Statement
import mathlingua.common.chalktalk.phase2.Text
import mathlingua.common.chalktalk.phase2.TupleNode
import mathlingua.common.textalk.ExpressionTexTalkNode
import mathlingua.common.textalk.ParametersTexTalkNode
import mathlingua.common.textalk.TexTalkNode
import mathlingua.common.textalk.TextTexTalkNode

fun getVars(node: Phase1Node): List<String> {
    val vars = mutableListOf<String>()
    getVarsImpl(node, vars)
    return vars
}

fun getVars(node: Phase2Node): List<String> {
    val vars = mutableListOf<String>()
    getVarsImpl(node, vars)
    return vars
}

fun getVars(texTalkNode: TexTalkNode): List<String> {
    val vars = mutableListOf<String>()
    getVarsImpl(texTalkNode, vars, false)
    return vars
}

fun renameVars(texTalkNode: TexTalkNode,
               map: Map<String, String>) = texTalkNode.transform {
    if (it is TextTexTalkNode) {
        it.copy(text = map[it.text] ?: it.text)
    } else {
        it
    }
}

fun renameVars(root: Phase2Node, map: Map<String, String>): Phase2Node {
    fun chalkTransformer(node: Phase2Node): Phase2Node {
        if (node is Identifier) {
            return node.copy(name = map[node.name] ?: node.name)
        }

        if (node is Statement) {
            return when (val validation = node.texTalkRoot) {
                is ValidationSuccess -> {
                    val exp = renameVars(validation.value, map) as ExpressionTexTalkNode
                    return Statement(
                            row = -1,
                            column = -1,
                        text = exp.toCode(),
                        texTalkRoot = ValidationSuccess(exp)
                    )
                }
                is ValidationFailure -> node
            }
        } else if (node is Text) {
            var newText = node.text
            val keysLongToShort = map.keys.toList().sortedBy { it.length }.reversed()
            for (key in keysLongToShort) {
                newText = newText.replace("%$key", map[key]!!)
            }
            return Text(
                text = newText,
                row = -1,
                column = -1
            )
        }

        return node
    }

    return root.transform(::chalkTransformer)
}

private fun getVarsImpl(node: Phase1Node,
                        vars: MutableList<String>) {
    if (node is Phase1Token) {
        vars.add(node.text)
    } else {
        node.forEach { getVarsImpl(it, vars) }
    }
}

private fun getVarsImpl(node: Phase2Node,
                        vars: MutableList<String>) {
    if (node is Identifier) {
        vars.add(node.name)
    } else if (node is TupleNode) {
        getVarsImpl(node.tuple, vars)
    } else if (node is AggregateNode) {
        getVarsImpl(node.aggregate, vars)
    } else if (node is AbstractionNode) {
        getVarsImpl(node.abstraction, vars)
    } else if (node is AssignmentNode) {
        vars.add(node.assignment.lhs.text)
        getVarsImpl(node.assignment.rhs, vars)
    } else {
        node.forEach { getVarsImpl(it, vars) }
    }
}

private fun getVarsImpl(texTalkNode: TexTalkNode, vars: MutableList<String>, inParams: Boolean) {
    if (inParams && texTalkNode is TextTexTalkNode) {
        vars.add(texTalkNode.text)
    } else if (texTalkNode is ParametersTexTalkNode) {
        texTalkNode.forEach { getVarsImpl(it, vars, true) }
    } else {
        texTalkNode.forEach { getVarsImpl(it, vars, inParams) }
    }
}

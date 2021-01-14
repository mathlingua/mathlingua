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

package mathlingua.chalktalk.phase2

import java.lang.StringBuilder
import mathlingua.ValueSourceTracker
import mathlingua.chalktalk.phase2.ast.clause.AbstractionNode
import mathlingua.chalktalk.phase2.ast.clause.AssignmentNode
import mathlingua.chalktalk.phase2.ast.clause.Identifier
import mathlingua.chalktalk.phase2.ast.clause.Statement
import mathlingua.chalktalk.phase2.ast.clause.Target
import mathlingua.chalktalk.phase2.ast.clause.TupleNode
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError
import mathlingua.support.ValidationSuccess
import mathlingua.support.newLocationTracker
import mathlingua.textalk.Command
import mathlingua.textalk.ExpressionTexTalkNode
import mathlingua.textalk.IsTexTalkNode
import mathlingua.textalk.TexTalkNode
import mathlingua.textalk.TextTexTalkNode
import mathlingua.transform.normalize
import mathlingua.transform.signature

class SymbolAnalyzer(defines: List<ValueSourceTracker<DefinesGroup>>) {
    private val signatureMap = mutableMapOf<String, Set<String>>()

    init {
        for (vst in defines) {
            val sig = vst.value.signature
            if (sig != null) {
                val name = getTargetName(vst.value.definesSection.targets[0])
                signatureMap[sig] =
                    getDefSignatures(vst.value, name, vst.tracker ?: newLocationTracker())
            }
        }
    }

    private fun getTargetName(target: Target): String {
        return when (target) {
            is Identifier -> target.name
            is AbstractionNode -> target.abstraction.parts.joinToString(".") { it.name.text }
            is AssignmentNode -> target.assignment.lhs.text
            is TupleNode -> target.tuple.toCode()
            else -> target.toCode(false, 0).getCode()
        }
    }

    private fun getAllTypes(
        node: Phase2Node, target: String, tracker: MutableLocationTracker
    ): List<Command> {
        val result = mutableListOf<Command>()
        getAllTypesImpl(normalize(node, tracker), target, result)
        return result
    }

    private fun getAllTypesImpl(node: Phase2Node, target: String, result: MutableList<Command>) {
        if (node is Statement) {
            when (val validation = node.texTalkRoot
            ) {
                is ValidationSuccess -> {
                    getAllTypesImpl(validation.value, target, result)
                }
            }
        }
        node.forEach { getAllTypesImpl(it, target, result) }
    }

    private fun matchesName(node: ExpressionTexTalkNode, target: String): Boolean {
        return node.children.size == 1 &&
            ((node.children[0] is TextTexTalkNode &&
                (node.children[0] as TextTexTalkNode).text == target) ||
                node.children[0].toCode().replace(" ", "").startsWith("$target("))
    }

    private fun getAllTypesImpl(node: TexTalkNode, target: String, result: MutableList<Command>) {
        if (node is IsTexTalkNode && node.lhs.items.find { matchesName(it, target) } != null) {
            node.rhs.items.forEach {
                if (it.children.size == 1 && it.children[0] is Command) {
                    result.add(it.children[0] as Command)
                }
            }
        }
        node.forEach { getAllTypesImpl(it, target, result) }
    }

    data class NameAndStatement(val name: String, val statement: Statement)

    private fun findAllNames(node: Phase2Node): List<NameAndStatement> {
        val result = mutableListOf<NameAndStatement>()
        findAllNamesImpl(node, result)
        return result
    }

    private fun findAllNamesImpl(node: Phase2Node, result: MutableList<NameAndStatement>) {
        if (node is Statement) {
            when (val validation = node.texTalkRoot
            ) {
                is ValidationSuccess -> {
                    findAllNamesImpl(node, validation.value, result)
                }
            }
        }
        node.forEach { findAllNamesImpl(it, result) }
    }

    private fun findAllNamesImpl(
        stmt: Statement, texTalkNode: TexTalkNode, result: MutableList<NameAndStatement>
    ) {
        if (texTalkNode is TextTexTalkNode) {
            result.add(NameAndStatement(name = texTalkNode.text, statement = stmt))
        }
        texTalkNode.forEach {
            when (it) {
                is Command -> {
                    for (part in it.parts) {
                        for (grp in part.groups) {
                            findAllNamesImpl(stmt, grp, result)
                        }
                        for (grp in part.namedGroups) {
                            findAllNamesImpl(stmt, grp, result)
                        }
                        if (part.paren != null) {
                            findAllNamesImpl(stmt, part.paren, result)
                        }
                        if (part.square != null) {
                            findAllNamesImpl(stmt, part.square, result)
                        }
                        if (part.subSup != null) {
                            findAllNamesImpl(stmt, part.subSup, result)
                        }
                    }
                }
                else -> findAllNamesImpl(stmt, it, result)
            }
        }
    }

    fun findInvalidTypes(grp: TopLevelGroup, tracker: MutableLocationTracker): List<ParseError> {
        val group = normalize(grp, tracker)
        val errors = mutableListOf<ParseError>()
        val names = findAllNames(group)
        for (name in names) {
            val allPaths = mutableListOf<List<String>>()
            for (sig in getDefSignatures(group, name.name, tracker)) {
                allPaths.addAll(getTypePaths(sig))
            }
            val baseTypes = mutableSetOf<String>()
            for (path in allPaths) {
                baseTypes.add(path.last())
            }
            if (baseTypes.size > 1) {
                val location = tracker.getLocationOf(name.statement)
                val builder = StringBuilder()
                builder.append(
                    "'${name.name}' has more than one base type: {${baseTypes.toList().joinToString(", ")}}\n")
                builder.append("Found type paths:\n")
                builder.append(allPaths.joinToString(", ") { "{${it.joinToString(" -> ")}}" })
                builder.append("\n")
                errors.add(
                    ParseError(
                        message = builder.toString(),
                        row = location?.row ?: -1,
                        column = location?.column ?: -1))
            }
        }
        return errors
    }

    private fun getDefSignatures(
        node: Phase2Node, name: String, tracker: MutableLocationTracker
    ): Set<String> {
        val commands = getAllTypes(node, name, tracker)
        val typeSet = mutableSetOf<String>()
        for (cmd in commands) {
            typeSet.add(cmd.signature())
        }
        return typeSet
    }

    fun getTypePaths(startSignature: String): List<List<String>> {
        val path = mutableListOf<String>()
        val result = mutableListOf<List<String>>()
        getTypePathsImpl(startSignature, path, result)
        return result
    }

    private fun getTypePathsImpl(
        startSignature: String, path: MutableList<String>, result: MutableList<List<String>>
    ) {
        path.add(startSignature)
        if (signatureMap.containsKey(startSignature) &&
            signatureMap[startSignature]!!.isNotEmpty()) {
            for (subSignature in signatureMap[startSignature]!!) {
                if (path.contains(subSignature)) {
                    throw RuntimeException("Type cycle detected")
                }
                getTypePathsImpl(subSignature, path, result)
            }
        } else {
            result.add(listOf(*path.toTypedArray()))
        }
        path.removeLast()
    }
}

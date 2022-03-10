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

package mathlingua.backend

import mathlingua.backend.transform.Signature
import mathlingua.backend.transform.getVarsPhase2Node
import mathlingua.backend.transform.normalize
import mathlingua.backend.transform.signature
import mathlingua.frontend.chalktalk.phase1.ast.Tuple
import mathlingua.frontend.chalktalk.phase1.ast.TupleItem
import mathlingua.frontend.chalktalk.phase2.ast.clause.AbstractionNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.AssignmentNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.Identifier
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Target
import mathlingua.frontend.chalktalk.phase2.ast.clause.TupleNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.getLeftHandSideTargets
import mathlingua.frontend.chalktalk.phase2.ast.clause.isColonEqualsStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.isInStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.isIsStatement
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.viewing.ViewGroup
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.textalk.Command
import mathlingua.frontend.textalk.ExpressionTexTalkNode
import mathlingua.frontend.textalk.IsTexTalkNode
import mathlingua.frontend.textalk.ParametersTexTalkNode
import mathlingua.frontend.textalk.TexTalkNode
import mathlingua.frontend.textalk.TexTalkTokenType
import mathlingua.frontend.textalk.TextTexTalkNode

internal interface SymbolAnalyzer {
    fun findInvalidTypes(grp: TopLevelGroup): List<ParseError>
    fun findInvalidMeansSection(defines: DefinesGroup): List<ParseError>
    fun getTypePaths(startSignature: String): List<List<String>>
}

internal fun newSymbolAnalyzer(
    defines: List<Pair<ValueAndSource<Signature>, DefinesGroup>>
): SymbolAnalyzer {
    return SymbolAnalyzerImpl(defines)
}

// -----------------------------------------------------------------------------

private fun List<TupleItem>.startsWith(prefix: List<ExpressionTexTalkNode>): Boolean {
    var i = 0
    while (i < this.size && i < prefix.size) {
        if (this[i].toCode() != prefix[i].toCode()) {
            return false
        }
        i++
    }
    return i >= prefix.size
}

private class SymbolAnalyzerImpl(defines: List<Pair<ValueAndSource<Signature>, DefinesGroup>>) :
    SymbolAnalyzer {
    private val signatureMap = mutableMapOf<String, Set<String>>()

    init {
        for (pair in defines) {
            val sig = pair.first.value
            val def = pair.second
            val name = getTargetName(def.definesSection.targets[0])
            val defSignatures = mutableSetOf<String>()
            if (name != null) {
                defSignatures.addAll(getDefSignatures(def, name))
            }

            if (def.definesSection.targets.size == 1 && def.meansSection != null) {
                val targetTuple =
                    when (val first = def.definesSection.targets.first()
                    ) {
                        is TupleNode -> {
                            first.tuple
                        }
                        is AssignmentNode -> {
                            if (first.assignment.rhs is Tuple) {
                                first.assignment.rhs
                            } else {
                                null
                            }
                        }
                        else -> {
                            null
                        }
                    }

                fun ParametersTexTalkNode.first() =
                    if (this.items.size == 1 && this.items.first().children.size == 1) {
                        this.items.first().children.first()
                    } else {
                        null
                    }

                if (targetTuple != null) {
                    for (stmt in def.meansSection.statements) {
                        when (val validation = stmt.texTalkRoot
                        ) {
                            is ValidationSuccess -> {
                                val exp = validation.value
                                if (exp.children.size == 1) {
                                    val expFirst = exp.children.first()
                                    if (expFirst is IsTexTalkNode) {
                                        val lhsFirst = expFirst.lhs.first()
                                        val rhsFirst = expFirst.rhs.first()
                                        if (lhsFirst is mathlingua.frontend.textalk.TupleNode &&
                                            rhsFirst is Command &&
                                            targetTuple.items.startsWith(lhsFirst.params.items)) {
                                            defSignatures.add(rhsFirst.signature())
                                        } else if (lhsFirst is TextTexTalkNode &&
                                            lhsFirst.tokenType == TexTalkTokenType.Identifier &&
                                            rhsFirst is Command &&
                                            targetTuple.items.isNotEmpty() &&
                                            lhsFirst.toCode() ==
                                                targetTuple.items.first().toCode()) {
                                            defSignatures.add(rhsFirst.signature())
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if (def.providingSection != null) {
                var viewGroup: ViewGroup? = null
                for (clause in def.providingSection.clauses.clauses) {
                    if (clause is ViewGroup) {
                        viewGroup = clause
                        break
                    }
                }
                if (viewGroup != null) {
                    when (val root = viewGroup.viewAsSection.statement.texTalkRoot
                    ) {
                        is ValidationSuccess -> {
                            if (root.value.children.size == 1 &&
                                root.value.children[0] is Command) {
                                val cmd = root.value.children[0] as Command
                                defSignatures.add(cmd.signature())
                            }
                            // TODO: have an error reported if the viewing:as: section doesn't
                            // contain a
                            // Command
                        }
                        else -> {
                            // TODO: have an error reported if the viewing:as: section doesn't
                            // have valid
                            // textalk code
                        }
                    }
                }
            }
            signatureMap[sig.form] = defSignatures
        }
    }

    override fun findInvalidTypes(grp: TopLevelGroup): List<ParseError> {
        val group = grp.normalize()
        val errors = mutableListOf<ParseError>()
        val names = findAllNames(group)
        for (name in names) {
            val allPaths = mutableListOf<List<String>>()
            for (sig in getDefSignatures(group, name.name)) {
                allPaths.addAll(getTypePaths(sig))
            }
            val baseTypes = mutableSetOf<String>()
            for (path in allPaths) {
                baseTypes.add(path.last())
            }
            if (baseTypes.size > 1) {
                val builder = StringBuilder()
                builder.append(
                    "'${name.name}' has more than one base type: {${baseTypes.toList().joinToString(", ")}}\n")
                builder.append("Found type paths:\n")
                builder.append(allPaths.joinToString(", ") { "{${it.joinToString(" -> ")}}" })
                builder.append("\n")
                errors.add(
                    ParseError(
                        message = builder.toString(),
                        row = name.statement.row,
                        column = name.statement.column))
            }
        }
        return errors
    }

    override fun findInvalidMeansSection(defines: DefinesGroup): List<ParseError> {
        val means = defines.meansSection ?: return emptyList()

        val result = mutableListOf<ParseError>()
        if (means.statements.isEmpty()) {
            return listOf(
                ParseError(
                    message = "Expected at least one statement",
                    row = means.row,
                    column = means.column))
        }

        for (stmt in means.statements) {
            if (!stmt.isIsStatement() && !stmt.isInStatement() && !stmt.isColonEqualsStatement()) {
                result.add(
                    ParseError(
                        message = "Expected an 'is', 'in', or ':=' statement",
                        row = stmt.row,
                        column = stmt.column))
            }
        }

        if (result.isNotEmpty()) {
            return result
        }

        val defVars = mutableSetOf<String>()
        for (target in defines.definesSection.targets) {
            defVars.addAll(target.getVarsPhase2Node().map { it.name })
        }

        for (stmt in means.statements) {
            for (target in stmt.getLeftHandSideTargets() ?: emptyList()) {
                for (name in target.getVarsPhase2Node().map { it.name }) {
                    result.add(
                        ParseError(
                            message =
                                "Cannot define name $name in an `is`, `in`, or `:=` statement that was not introduced in a `means:` section.",
                            row = target.row,
                            column = target.column))
                }
            }
        }

        return result
    }

    private fun getDefSignatures(node: Phase2Node, name: String): Set<String> {
        val commands = getAllTypes(node, name)
        val typeSet = mutableSetOf<String>()
        for (cmd in commands) {
            typeSet.add(cmd.signature())
        }
        return typeSet
    }

    override fun getTypePaths(startSignature: String): List<List<String>> {
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
                    throw RuntimeException("Type cycle detected for signature $subSignature")
                }
                getTypePathsImpl(subSignature, path, result)
            }
        } else {
            result.add(listOf(*path.toTypedArray()))
        }
        path.removeLast()
    }
}

private fun getTargetName(target: Target): String? {
    return when (target) {
        is AbstractionNode -> {
            if (!target.abstraction.isEnclosed &&
                target.abstraction.parts.size == 1 &&
                !target.abstraction.isVarArgs &&
                target.abstraction.subParams == null) {
                target.abstraction.parts.first().name.text
            } else {
                null
            }
        }
        is Identifier -> target.name
        is AssignmentNode -> target.assignment.lhs.text
        else -> null
    }
}

private fun getAllTypes(node: Phase2Node, target: String): List<Command> {
    val result = mutableListOf<Command>()
    getAllTypesImpl(node.normalize(), target, result)
    return result
}

private fun getAllTypesImpl(node: Phase2Node, target: String, result: MutableList<Command>) {
    if (node is Statement) {
        when (val validation = node.texTalkRoot
        ) {
            is ValidationSuccess -> {
                getAllTypesImpl(validation.value, target, result)
            }
            is ValidationFailure -> {
                // ignore statements that are invalid
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

private data class NameAndStatement(val name: String, val statement: Statement)

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

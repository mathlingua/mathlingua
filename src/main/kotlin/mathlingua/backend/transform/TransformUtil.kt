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

package mathlingua.backend.transform

import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.ClauseListNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.hasChild
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.support.validationSuccess
import mathlingua.frontend.textalk.Command
import mathlingua.frontend.textalk.ExpressionTexTalkNode
import mathlingua.frontend.textalk.IsTexTalkNode
import mathlingua.frontend.textalk.ParametersTexTalkNode
import mathlingua.frontend.textalk.TexTalkNode
import mathlingua.frontend.textalk.TexTalkNodeType
import mathlingua.frontend.textalk.TexTalkTokenType
import mathlingua.frontend.textalk.TextTexTalkNode

internal fun normalize(node: Phase2Node, tracker: MutableLocationTracker): Phase2Node {
    var result = node
    result = commaSeparateCompoundCommands(result, result, tracker).root
    result = separateIsStatements(result, result, tracker).root
    result = separateInfixOperatorStatements(result, result, tracker).root
    return glueCommands(result, result, tracker).root
}

// replaces anything of the form `x is \a \b \c` as `x \a.b.c`
internal fun normalize(node: TexTalkNode, location: Location): TexTalkNode {
    return node.transform {
        if (it is ExpressionTexTalkNode) {
            val allCmd = it.children.all { it is Command }
            if (allCmd) {
                ExpressionTexTalkNode(children = getCommandsToGlue(it, location))
            } else {
                it
            }
        } else {
            it
        }
    }
}

internal fun separateInfixOperatorStatements(
    root: Phase2Node, follow: Phase2Node, tracker: MutableLocationTracker
): RootTarget<Phase2Node, Phase2Node> {
    var newFollow: Phase2Node? = null
    val newRoot =
        root.transform {
            val result =
                if (it is ClauseListNode) {
                    val newClauses = mutableListOf<Clause>()
                    for (c in it.clauses) {
                        if (c is Statement) {
                            when (val validation = c.texTalkRoot
                            ) {
                                is ValidationSuccess -> {
                                    val expRoot = validation.value
                                    for (expanded in getExpandedInfixOperators(expRoot)) {
                                        val stmt =
                                            Statement(
                                                text = expanded.toCode(),
                                                texTalkRoot = validationSuccess(expanded))
                                        val location = tracker.getLocationOf(c)
                                        if (location != null) {
                                            tracker.setLocationOf(stmt, location)
                                        }
                                        newClauses.add(stmt)
                                    }
                                }
                                is ValidationFailure -> newClauses.add(c)
                            }
                        } else {
                            newClauses.add(c)
                        }
                    }
                    val result = ClauseListNode(clauses = newClauses)
                    if (newFollow == null && hasChild(it, follow)) {
                        newFollow = result
                    }
                    result
                } else {
                    it
                }
            result
        }
    return RootTarget(root = newRoot, target = newFollow ?: follow)
}

internal fun commaSeparateCompoundCommands(
    root: Phase2Node, follow: Phase2Node, tracker: MutableLocationTracker
): RootTarget<Phase2Node, Phase2Node> {
    var newFollow: Phase2Node? = null
    val newRoot =
        root.transform {
            val result =
                if (it is Statement) {
                    if (it.texTalkRoot is ValidationSuccess) {
                        val newRoot =
                            it.texTalkRoot.value.transform { texTalkNode ->
                                if (texTalkNode is IsTexTalkNode) {
                                    val newExpressions = mutableListOf<ExpressionTexTalkNode>()
                                    val location = tracker.getLocationOf(root) ?: Location(-1, -1)
                                    for (exp in texTalkNode.rhs.items) {
                                        newExpressions.addAll(
                                            getCommandsToGlue(exp, location).map { cmd ->
                                                ExpressionTexTalkNode(children = listOf(cmd))
                                            })
                                    }
                                    IsTexTalkNode(
                                        lhs = texTalkNode.lhs,
                                        rhs = texTalkNode.rhs.copy(items = newExpressions))
                                } else {
                                    texTalkNode
                                }
                            }
                        val newStatement =
                            Statement(
                                text = newRoot.toCode(),
                                texTalkRoot = validationSuccess(newRoot as ExpressionTexTalkNode))
                        if (newFollow == null && hasChild(it, follow)) {
                            newFollow = newStatement
                        }
                        newStatement
                    } else {
                        it
                    }
                } else {
                    it
                }
            val location = tracker.getLocationOf(it)
            if (location != null) {
                tracker.setLocationOf(result, location)
            }
            result
        }
    return RootTarget(root = newRoot, target = newFollow ?: follow)
}

internal fun separateIsStatements(
    root: Phase2Node, follow: Phase2Node, tracker: MutableLocationTracker
): RootTarget<Phase2Node, Phase2Node> {
    var newFollow: Phase2Node? = null
    val newRoot =
        root.transform {
            val result =
                if (it is ClauseListNode) {
                    val newClauses = mutableListOf<Clause>()
                    for (clause in it.clauses) {
                        if (clause is Statement) {
                            val separated = findSeparatedIsNodes(clause)
                            if (separated == null) {
                                newClauses.add(clause)
                            } else {
                                newClauses.addAll(
                                    separated.map {
                                        val expRoot = ExpressionTexTalkNode(children = listOf(it))
                                        val stmt =
                                            Statement(
                                                text = expRoot.toCode(),
                                                texTalkRoot = validationSuccess(expRoot))
                                        val location = tracker.getLocationOf(clause)
                                        if (location != null) {
                                            tracker.setLocationOf(stmt, location)
                                        }
                                        stmt
                                    })
                            }
                        } else {
                            newClauses.add(clause)
                        }
                    }
                    val result = ClauseListNode(clauses = newClauses)
                    if (newFollow == null && hasChild(it, follow)) {
                        newFollow = result
                    }
                    result
                } else {
                    it
                }
            result
        }
    return RootTarget(root = newRoot, target = newFollow ?: follow)
}

internal fun replaceSignatures(texTalkNode: TexTalkNode, signature: String, replacement: String) =
    texTalkNode.transform {
        if (it is Command && it.signature() == signature) {
            TextTexTalkNode(
                type = TexTalkNodeType.Identifier,
                tokenType = TexTalkTokenType.Identifier,
                text = replacement,
                isVarArg = false)
        } else {
            texTalkNode
        }
    }

// -----------------------------------------------------------------------------

private fun findSeparatedIsNodes(node: Statement) =
    when (val validation = node.texTalkRoot
    ) {
        is ValidationFailure -> null
        is ValidationSuccess -> {
            val root = validation.value
            if (root.children.size == 1 && root.children[0] is IsTexTalkNode) {
                val isNode = root.children[0] as IsTexTalkNode
                separateIsStatementsUnder(isNode)
            } else {
                null
            }
        }
    }

private fun separateIsStatementsUnder(isNode: IsTexTalkNode): List<IsTexTalkNode> {
    val result = mutableListOf<IsTexTalkNode>()
    for (left in isNode.lhs.items) {
        for (right in isNode.rhs.items) {
            result.add(
                IsTexTalkNode(
                    lhs = ParametersTexTalkNode(items = listOf(left)),
                    rhs = ParametersTexTalkNode(items = listOf(right))))
        }
    }
    return result
}

private fun getSingleInfixOperatorIndex(exp: ExpressionTexTalkNode): Int {
    for (i in 1 until exp.children.size - 1) {
        val prev = exp.children[i - 1]
        val cur = exp.children[i]
        val next = exp.children[i + 1]
        if (!isOperator(prev) && cur is Command && !isOperator(next)) {
            return i
        }
    }

    return -1
}

private fun isComma(node: TexTalkNode) = node is TextTexTalkNode && node.text == ","

private fun isOperator(node: TexTalkNode): Boolean {
    if (node !is TextTexTalkNode) {
        return false
    }

    if (node.text.isBlank()) {
        return false
    }

    for (c in node.text) {
        if (!isOpChar(c)) {
            return false
        }
    }

    return true
}

private fun isOpChar(c: Char) =
    (c == '!' ||
        c == '@' ||
        c == '%' ||
        c == '&' ||
        c == '*' ||
        c == '-' ||
        c == '+' ||
        c == '=' ||
        c == '|' ||
        c == '/' ||
        c == '<' ||
        c == '>')

private fun getArguments(exp: ExpressionTexTalkNode, start: Int, end: Int): List<TexTalkNode> {
    val result = mutableListOf<TexTalkNode>()
    var i = start
    while (i < end) {
        val argChildren = mutableListOf<TexTalkNode>()
        while (i < end && !isComma(exp.children[i])) {
            argChildren.add(exp.children[i++])
        }

        if (i < end && isComma(exp.children[i])) {
            i++ // skip the comma
        }

        if (argChildren.size == 1) {
            result.add(argChildren[0])
        } else {
            result.add(ExpressionTexTalkNode(children = argChildren))
        }
    }
    return result
}

private fun getExpandedInfixOperators(exp: ExpressionTexTalkNode): List<ExpressionTexTalkNode> {
    val opIndex = getSingleInfixOperatorIndex(exp)
    if (opIndex < 0) {
        return listOf(exp)
    }

    val leftArgs = getArguments(exp, 0, opIndex)
    val rightArgs = getArguments(exp, opIndex + 1, exp.children.size)

    val result = mutableListOf<ExpressionTexTalkNode>()

    val op = exp.children[opIndex]
    for (left in leftArgs) {
        for (right in rightArgs) {
            result.add(ExpressionTexTalkNode(children = listOf(left, op, right)))
        }
    }

    return result
}

/*
 * The following code has not been tested since changes
 * have been made to the AST.

internal fun moveInlineCommandsToIsNode(
    defs: List<DefinesGroup>,
    root: Phase2Node,
    target: Phase2Node // non-null targets a specific node
// and null targets all ClauseListNodes
): RootTarget<Phase2Node, Phase2Node> {
    val knownDefSigs = defs.map { it.signature }.filterNotNull().toSet()
    fun realShouldProcessTex(root: TexTalkNode, node: TexTalkNode): Boolean {
        if (node is Command && !knownDefSigs.contains(node.signature())) {
            return false
        }

        val parents = getTexTalkAncestry(root, node)
        for (p in parents) {
            if (p is IsTexTalkNode) {
                return false
            }
        }

        return true
    }

    var seed = 0
    var newTarget: Phase2Node? = null
    val newRoot =
        root.transform {
            val result =
                if (it is ClauseListNode && (hasChild(it, target) || hasChild(target, it))) {
                    val newClauses = mutableListOf<Clause>()
                    for (c in it.clauses) {
                        if (c is Statement) {
                            val transformed =
                                moveStatementInlineCommandsToIsNode(
                                    seed++, c, { true }, ::realShouldProcessTex)
                            newClauses.add(transformed)
                        } else {
                            newClauses.add(c)
                        }
                    }
                    val result = ClauseListNode(clauses = newClauses)
                    if (newTarget == null && hasChild(it, target)) {
                        newTarget = result
                    }
                    result
                } else {
                    it
                }
            result
        }
    return RootTarget(root = newRoot, target = newTarget ?: target)
}

internal fun moveStatementInlineCommandsToIsNode(
    seed: Int,
    stmt: Statement,
    shouldProcessChalk: (node: Phase2Node) -> Boolean,
    shouldProcessTex: (root: TexTalkNode, node: TexTalkNode) -> Boolean
): Clause {
    val validation = stmt.texTalkRoot
    if (validation is ValidationFailure) {
        return stmt
    }
    val root = (validation as ValidationSuccess).value

    if (!shouldProcessChalk(stmt)) {
        return stmt
    }

    fun shouldProcessTexNodes(root: TexTalkNode, node: TexTalkNode): Boolean {
        if (!shouldProcessTex(root, node)) {
            return false
        }

        return !getTexTalkAncestry(root, node).any { it is IsTexTalkNode }
    }

    val commandsFound = findCommands(root)
    val cmdToReplacement = mutableMapOf<Command, String>()
    var count = seed
    for (cmd in commandsFound) {
        if (shouldProcessTex(root, cmd)) {
            cmdToReplacement[cmd] = "\$${count++}"
        }
    }

    val cmdsToProcess = cmdToReplacement.keys
    val newNode =
        replaceCommands(
            stmt, cmdToReplacement, shouldProcessChalk, ::shouldProcessTexNodes) as Clause

    if (commandsFound.isEmpty()) {
        return stmt
    }

    if (cmdsToProcess.isEmpty()) {
        return stmt
    }

    return ForAllGroup(
        forAllSection =
            ForAllSection(
                targets =
                    cmdsToProcess.map {
                        Identifier(name = cmdToReplacement[it]!!, isVarArgs = false)
                    }),
        whereSection =
            WhereSection(
                clauses =
                    ClauseListNode(
                        clauses =
                            cmdsToProcess.map {
                                val isNode =
                                    IsTexTalkNode(
                                        lhs =
                                            ParametersTexTalkNode(
                                                items =
                                                    listOf(
                                                        ExpressionTexTalkNode(
                                                            children =
                                                                listOf(
                                                                    TextTexTalkNode(
                                                                        type =
                                                                            TexTalkNodeType
                                                                                .Identifier,
                                                                        tokenType =
                                                                            TexTalkTokenType
                                                                                .Identifier,
                                                                        text =
                                                                            cmdToReplacement[it]!!,
                                                                        isVarArg = false))))),
                                        rhs =
                                            ParametersTexTalkNode(
                                                items =
                                                    listOf(
                                                        ExpressionTexTalkNode(
                                                            children = listOf(it)))))

                                Statement(
                                    text = isNode.toCode(),
                                    texTalkRoot =
                                        validationSuccess(
                                            ExpressionTexTalkNode(children = listOf(isNode))))
                            })),
        suchThatSection = null,
        thenSection = ThenSection(clauses = ClauseListNode(clauses = listOf(newNode))))
}

internal fun replaceStates(
    root: Phase2Node, states: List<StatesGroup>, target: Phase2Node
): RootTarget<Phase2Node, Phase2Node> {
    val statesMap = mutableMapOf<String, StatesGroup>()
    for (s in states) {
        val sig = s.signature
        if (sig != null) {
            statesMap[sig] = s
        }
    }

    var newTarget: Phase2Node? = null
    fun chalkTransformer(node: Phase2Node): Phase2Node {
        if (node !is ClauseListNode || (!hasChild(node, target) && !hasChild(target, node))) {
            return node
        }

        val newClauses = mutableListOf<Clause>()

        for (clause in node.clauses) {
            if (clause !is Statement) {
                newClauses.add(clause)
                continue
            }

            if (clause.texTalkRoot is ValidationSuccess &&
                clause.texTalkRoot.value.children.size == 1 &&
                clause.texTalkRoot.value.children[0] is Command) {
                // a prefix command
                val command = clause.texTalkRoot.value.children[0] as Command
                val sig = command.signature()

                if (!statesMap.containsKey(sig)) {
                    return node
                }

                val rep = statesMap[sig]!!
                val cmdVars = getVars(command)
                val defIndirectVars = getRepresentsIdVars(rep)

                val map = mutableMapOf<String, String>()
                for (i in cmdVars.indices) {
                    map[defIndirectVars[i]] = cmdVars[i]
                }

                val ifThen = buildIfThens(rep)
                if (ifThen.ifSection.clauses.clauses.isEmpty()) {
                    for (c in ifThen.thenSection.clauses.clauses) {
                        newClauses.add(renameVars(c, map) as Clause)
                    }
                } else {
                    newClauses.add(renameVars(ifThen, map) as Clause)
                }
            } else if (clause.texTalkRoot is ValidationSuccess &&
                clause.texTalkRoot.value.children.size == 3 &&
                clause.texTalkRoot.value.children[0] is TextTexTalkNode &&
                clause.texTalkRoot.value.children[1] is Command &&
                clause.texTalkRoot.value.children[2] is TextTexTalkNode) {
                // and infix command
                val left = clause.texTalkRoot.value.children[0] as TextTexTalkNode
                val op = clause.texTalkRoot.value.children[1] as Command
                val right = clause.texTalkRoot.value.children[2] as TextTexTalkNode

                val sig = op.signature()

                if (!statesMap.containsKey(sig)) {
                    return node
                }

                val rep = statesMap[sig]!!
                val cmdVars = listOf(left.text, right.text)

                if (rep.id.texTalkRoot is ValidationFailure) {
                    return node
                }

                val validation = rep.id.texTalkRoot as ValidationSuccess
                if (validation.value.children.size != 3 ||
                    validation.value.children[0] !is TextTexTalkNode ||
                    validation.value.children[1] !is Command ||
                    validation.value.children[2] !is TextTexTalkNode) {
                    return node
                }

                val repLeftOpRight = validation.value.children
                val repLeft = (repLeftOpRight[0] as TextTexTalkNode).text
                val repRight = (repLeftOpRight[2] as TextTexTalkNode).text

                val defIndirectVars = listOf(repLeft, repRight)

                val map = mutableMapOf<String, String>()
                for (i in cmdVars.indices) {
                    map[defIndirectVars[i]] = cmdVars[i]
                }

                val ifThen = buildIfThens(rep)
                if (ifThen.ifSection.clauses.clauses.isEmpty()) {
                    for (c in ifThen.thenSection.clauses.clauses) {
                        newClauses.add(renameVars(c, map) as Clause)
                    }
                } else {
                    newClauses.add(renameVars(ifThen, map) as Clause)
                }
            } else {
                newClauses.add(clause)
            }
        }
        val result = ClauseListNode(clauses = newClauses)
        if (newTarget == null && hasChild(node, target)) {
            newTarget = result
        }
        return result
    }

    val newRoot = root.transform(::chalkTransformer)
    return RootTarget(root = newRoot, target = newTarget ?: target)
}

private fun buildIfThens(def: StatesGroup) =
    IfGroup(
        ifSection =
        IfSection(clauses = def.whenSection?.clauses ?: ClauseListNode(clauses = emptyList())),
        thenSection =
        ThenSection(clauses = def.thatSection.clauses))

private fun getRepresentsIdVars(rep: StatesGroup): List<String> {
    val vars = mutableListOf<String>()
    if (rep.id.texTalkRoot is ValidationSuccess) {
        vars.addAll(getVars(rep.id.texTalkRoot.value))
    }
    return vars
}

 */

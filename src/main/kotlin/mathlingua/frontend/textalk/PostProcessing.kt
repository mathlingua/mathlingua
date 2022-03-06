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

package mathlingua.frontend.textalk

import mathlingua.frontend.support.ParseError
import mathlingua.newStack

/*
 * The following is the algorithm for parsing operators and identifying their arguments.
 *
 * The algorithm will be described using the following example input:
 *
 *   +\x! * y! \plus x! + -y \times z
 *
 * Definitions:
 * - Special operator: An operator composed of one or more special characters (*,+,-,/,...)
 * - Command: A sequence starting with \ (for example, \plus, \in, \times{X})
 * - Command operator: A command that is interpreted as an operator.  For example '\in'
 *                     in the input '\pi \in X'
 * - Command value: A command that is interpreted as a value. For example, '\pi' in
 *                  the input '\pi \in X'
 * - Value: A sequence that can be the right-hand-side or left-hand-side of an operator.
 *
 * 1. ** identify identifier function calls **
 *    Reading left to right, identify any identifier functions calls such as `f(x)`.
 *    These consist of a TextTexTalkNode and a GroupTexTalkNode side-by-side and are
 *    different from `\f{x}` in that that is a command.  Those two nodes should be
 *    grouped together in a synthetic group so that, for example,
 *       f(x) > 0
 *    is treated as
 *       {f(x)} > 0
 * 2. ** identify prefix special operators **
 *    Reading the input from left to right (technically, scan the children of an
 *    ExpressionTexTalkNode from left to right), for any special operator where there is a
 *    non-special operator to the right of it, group the special operator and non-special
 *    operator such that the special operator is a prefix operator and the other is its value.
 *
 *    In the example, the input is interpreted as:
 *
 *       {+\x}! * y! \plus x! + {-y} \times z
 *
 * 3. ** identify postfix special operators **
 *    Reading the input from left to right, at any special operator, if there is either nothing
 *    to the left of that operator, or there is a non-special operator, group the special operator
 *    and the non-special operator so that the special operator is a postfix operator with the
 *    non-special value its value.
 *
 *    In the example, the input is next interpreted as:
 *
 *       {{+\x}!} * {y!} \plus {x!} + {-y} \times z
 *
 * 4. ** identify infix command operators **
 *    Note that an command (\pi or \in for example), can only be interpreted as a value or
 *    an infix operator.  Prefix and postfix operators are not supported.
 *
 *    The items between two special operators are viewed as a unit and commands are
 *    identified as values or commands using the rules below.
 *
 *    For example, given the input:
 *
 *      {{+\x}!} * {y!} \plus {x!} + {-y} \times z
 *
 *    the groups that are processed are:
 *    - {{+\x}!}
 *    - {y!} \plus {x!}
 *    - {-y} \times z
 *
 *    If the input contains no commands then:
 *    - Any sequence of two or more non-operators is a parse error since operators can
 *      only accept at most one argument on the left and at most one on the right.
 *      Example: 'a b + c d * x'
 *      Then: ERROR since + can only accept a single argument on the left, not 'a' and 'b'
 *            The same is true for + and 'c' and 'd' and '*' with its left argument.
 *
 *    Given the first command in the input (scanning from left to right), either
 *    - Case: There is nothing to the left or right of that command:
 *      Example: '\something{x}'
 *      Then: The command is interpreted as a value (\something{x} is a value)
 *    - Case: There is something to the left of the command but nothing to the right
 *      Example: 'x \in'
 *      Then: ERROR since postfix operators are not supported.
 *    - Case: There is nothing to the left of the command but is something to the right.
 *      Example: '\f x' or '\f \g'
 *      Then: ERROR since prefix operators are not supported.
 *    - Case: There is no value to the left of the command, one command to the right,
 *            and a value to the right of the command.
 *      Example: '\pi \in X' or '\pi \in \reals'
 *      Then: The middle command is an infix operator with the left command as its left
 *            value and the value to the right of the middle command its right value.
 *
 *            In the example '\pi \in X', then the '\in' is an infix operator with '\pi'
 *            its left value and 'X' its right value.
 *
 *            In the example '\pi \in \reals', then the '\in' is an infix operator with
 *            '\pi' as its left value and '\reals' as its right value.
 *    - Case: There is no value to the left of the command, and more than two non-operators
 *            to the right of the command.
 *      Then: ERROR since there isn't a way to parse the input.  The user needs to use
 *            parentheses to group items to allow the parse to proceed.
 *
 *            For example, '\pi \plus \x \times \y' needs parentheses to be explicitly added
 *            by the user to make its parse clear.  For example, the parentheses could be
 *            '(\pi \plus \x) \times \y' or '\pi \plus (\x \times \y)'.
 *
 *            Note that with the parentheses, the ambiguity is gone with respect to the cases
 *            described in the algorithm.
 *    - Case: There is exactly one non-operator to the left of the command and exactly one
 *            non-operator to the right of the command.
 *      Example: 'x \in \reals' or 'x \in X'
 *      Then: The command is an infix operator.  The value to the left is the operator's left
 *            value and the item to the right is its right operator.
 *
 *            For example, in 'x \in \reals' then '\in' is an infix operator with 'x' its left
 *            value and '\reals' its right value.
 *
 *            Next, in 'x \n X' then '\in' is also an infix operator with 'x' its left value
 *            and 'X' its right value.
 *    - Case: There is exactly one non-operator to the left of the command and more than one
 *            non-operator to the right.
 *      Example: 'x \in a b' or 'x \in \a b'
 *      Then: ERROR since an infix operator can only accept a single value on the left and right.
 *
 *    For the example, the groups are interpreted as:
 *    - {{+\x}!}
 *      Interpretation: a value
 *    - {y!} \plus {x!}
 *      Interpretation: '\plus' is an infix operator with {y!} its left value and {x!}
 *                      its right value.
 *    - {-y} \times z
 *      Interpretation: '\times' is an infix operator with {-y} its left value and z
 *                      its right value.
 * 5. ** identify infix special operators **
 *    At this point there is only a single value (either in parentheses or an individual item)
 *    between any two special operators in the input.  Perform the shunting hard algorithm
 *    to parse the input into a tree structure using the following precedence and associativity.
 *
 *    Precedence from most binding to least binding
 *      :=
 *      =
 *      ...
 *      _
 *      ^
 *      * /
 *      + -
 *      any custom special operator (such as !, **, ++, \plus etc.)
 *
 *    Any operator with ... as a prefix or suffix has its precedence determined by the
 *    associated operator.  That is, ...+ will have the precedence of + while ...* will
 *    have the precedence of *.
 *
 *    Associativity:
 *      :=, =, ^          -> right associative
 *      everything else   -> left associative
 *
 *    Note custom special operators are assumed to be left associative and have the highest
 *    binding after := and =
 *
 *    Also note, at this point the only operators that need evaluation from an associativity
 *    or precedence perspective are special operators since command operators have already
 *    been identified and the algorithm requires the user to explicitly use parentheses to
 *    describe how they are parsed from an associativity and precedence perspective.
 *    That is '\a \plus \b \times c' must be explicitly marked as either
 *    '(\a \plus \b) \times c' or '\a \plus (\b \times \c)'.
 */

internal fun parseOperators(root: ExpressionTexTalkNode) =
    try {
        val isRhsExpressions = findIsRhsExpressions(root)
        val funcCallRoot = identifyIdentifierFunctionCalls(root)
        val idPrefixOpRoot = identifySpecialPrefixOperators(funcCallRoot, isRhsExpressions)
        val idPostfixOpRoot = identifySpecialPostfixOperators(idPrefixOpRoot, isRhsExpressions)
        val idInfixOpRoot = identifyInfixCommandOperators(idPostfixOpRoot, isRhsExpressions)
        val final = runShuntingYard(idInfixOpRoot, isRhsExpressions)

        val resultRoot = final as ExpressionTexTalkNode
        TexTalkParseResult(root = resultRoot, errors = emptyList())
    } catch (e: ParseException) {
        TexTalkParseResult(
            root = ExpressionTexTalkNode(children = emptyList()), errors = listOf(e.parseError))
    }

// -----------------------------------------------------------------------------

private class ParseException(val parseError: ParseError) : Exception(parseError.message)

private fun findIsRhsExpressions(root: TexTalkNode): Set<ExpressionTexTalkNode> {
    val result = mutableSetOf<ExpressionTexTalkNode>()
    findIsRhsExpressionsImpl(root, result)
    return result
}

private fun findIsRhsExpressionsImpl(node: TexTalkNode, result: MutableSet<ExpressionTexTalkNode>) {
    if (node is IsTexTalkNode) {
        result.addAll(node.rhs.items)
    }

    node.forEach { findIsRhsExpressionsImpl(it, result) }
}

private fun identifyIdentifierFunctionCalls(root: TexTalkNode) =
    root.transform {
        if (it is ExpressionTexTalkNode) {
            val newChildren = mutableListOf<TexTalkNode>()
            var i = 0
            while (i < it.children.size) {
                val cur = it.children[i]
                val next = it.children.getOrNull(i + 1)
                // is if `f   (x)`
                //        ^   ^
                //        cur next
                if (cur is TextTexTalkNode &&
                    cur.tokenType == TexTalkTokenType.Identifier &&
                    next != null &&
                    next is GroupTexTalkNode &&
                    next.type == TexTalkNodeType.ParenGroup) {
                    newChildren.add(
                        GroupTexTalkNode(
                            type = TexTalkNodeType.SyntheticGroup,
                            isVarArg = false,
                            parameters =
                                ParametersTexTalkNode(
                                    items =
                                        listOf(
                                            ExpressionTexTalkNode(children = listOf(cur, next))))))
                    i += 2
                } else {
                    newChildren.add(cur)
                    i++
                }
            }
            ExpressionTexTalkNode(children = newChildren)
        } else {
            it
        }
    } as ExpressionTexTalkNode

private fun isSpecialOperator(node: TexTalkNode?) = getCommandOperatorSymbolText(node) != null

private fun getCommandOperatorSymbolText(node: TexTalkNode?): String? {
    return if (node != null &&
        node is TextTexTalkNode &&
        (node.tokenType == TexTalkTokenType.Operator ||
            node.tokenType == TexTalkTokenType.Caret ||
            node.tokenType == TexTalkTokenType.Underscore ||
            node.tokenType == TexTalkTokenType.DotDotDot)) {
        node.text
    } else if (node != null &&
        node is Command &&
        node.parts.isNotEmpty() &&
        isOperatorText(node.parts.last().name.text)) {
        node.parts.last().name.text
    } else if (node != null &&
        node is TextTexTalkNode &&
        node.tokenType == TexTalkTokenType.Identifier &&
        node.text.contains(".") &&
        isOperatorText(node.text)) {
        // An identifier of the form G.* is always an operator
        // since it is introduced from a Defines: target of the form
        // G := (X, *, e), for example, and thus G.* refers to the
        // operator and is only used in an expression context, i.e.
        // of the form 'x G.* y'.
        node.text
    } else {
        null
    }
}

private fun isOperatorText(name: String): Boolean {
    if (name.isEmpty()) {
        return false
    }

    var i = 0
    if (name.contains(".")) {
        i = name.indexOf(".") + 1
    }

    while (i < name.length) {
        if (!isOpChar(name[i++])) {
            return false
        }
    }

    return true
}

private fun identifySpecialPrefixOperators(
    root: ExpressionTexTalkNode, isNodeRhsExpressions: Set<ExpressionTexTalkNode>
) =
    root.transform {
        if (it is ExpressionTexTalkNode && !isNodeRhsExpressions.contains(it)) {
            val newChildren = mutableListOf<TexTalkNode>()
            var i = 0
            while (i < it.children.size) {
                val prev = it.children.getOrNull(i - 1)
                val cur = it.children[i]
                val next = it.children.getOrNull(i + 1)
                i++
                if (prev == null && isSpecialOperator(cur) && next == null) {
                    throw ParseException(
                        ParseError(
                            message =
                                "An operator needs to have at least a left or " +
                                    "right side argument ('${cur.toCode()}')",
                            row = -1,
                            column = -1))
                } else if ((isSpecialOperator(prev) || prev == null) &&
                    isSpecialOperator(cur) &&
                    !isSpecialOperator(next)) {
                    // for example  * +x
                    //   prev -> *
                    //   cur  -> +
                    //   next -> x
                    newChildren.add(
                        GroupTexTalkNode(
                            type = TexTalkNodeType.SyntheticGroup,
                            parameters =
                                ParametersTexTalkNode(
                                    items =
                                        listOf(
                                            ExpressionTexTalkNode(
                                                children =
                                                    listOf(
                                                        OperatorTexTalkNode(
                                                            lhs = null,
                                                            command = cur,
                                                            rhs = next))))),
                            isVarArg = false))
                    // move past the next element since it was already used
                    i++
                } else {
                    newChildren.add(cur)
                }
            }
            ExpressionTexTalkNode(children = newChildren)
        } else {
            it
        }
    } as ExpressionTexTalkNode

private fun identifySpecialPostfixOperators(
    root: ExpressionTexTalkNode, isNodeRhsExpressions: Set<ExpressionTexTalkNode>
) =
    root.transform {
        if (it is ExpressionTexTalkNode && !isNodeRhsExpressions.contains(it)) {
            val newChildren = mutableListOf<TexTalkNode>()
            var i = 0
            while (i < it.children.size) {
                val cur = it.children[i]
                val next = it.children.getOrNull(i + 1)
                val nextNext = it.children.getOrNull(i + 2)
                i++
                if (!isSpecialOperator(cur) &&
                    isSpecialOperator(next) &&
                    (isSpecialOperator(nextNext) || nextNext == null)) {
                    // for example  x! +
                    //   cur -> x
                    //   next -> !
                    //   nextNext -> +
                    newChildren.add(
                        GroupTexTalkNode(
                            type = TexTalkNodeType.SyntheticGroup,
                            parameters =
                                ParametersTexTalkNode(
                                    items =
                                        listOf(
                                            ExpressionTexTalkNode(
                                                children =
                                                    listOf(
                                                        OperatorTexTalkNode(
                                                            lhs = cur,
                                                            command = next!!,
                                                            rhs = null))))),
                            isVarArg = false))
                    // move past the next element since it was already used
                    i++
                } else {
                    newChildren.add(cur)
                }
            }
            ExpressionTexTalkNode(children = newChildren)
        } else {
            it
        }
    } as ExpressionTexTalkNode

private fun identifyInfixCommandOperators(
    root: TexTalkNode, isNodeRhsExpressions: Set<ExpressionTexTalkNode>
) =
    root.transform {
        if (it is ExpressionTexTalkNode && !isNodeRhsExpressions.contains(it)) {
            val newChildren = mutableListOf<TexTalkNode>()
            val sections = splitBetweenInfixOperators(it.children)
            for (section in sections) {
                if (section.isEmpty()) {
                    throw ParseException(
                        ParseError(
                            message =
                                "Two infix operators cannot be side by side ('${it.toCode()}')",
                            row = -1,
                            column = -1))
                } else if (section.size == 1) {
                    newChildren.add(section[0])
                } else if (section.size == 3) {
                    val lhs = section[0]
                    val cmd = section[1]
                    val rhs = section[2]
                    if (cmd !is Command || !cmd.hasSuffix) {
                        throw ParseException(
                            ParseError(
                                message = "Expected an argument but found ${cmd.toCode()}",
                                row = -1,
                                column = -1))
                    } else {
                        newChildren.add(OperatorTexTalkNode(lhs = lhs, command = cmd, rhs = rhs))
                    }
                } else if (section.size == 2 &&
                    section[1] is GroupTexTalkNode &&
                    section[1].type == TexTalkNodeType.ParenGroup) {
                    // then the section is // f(x) or \f(x) so just keep the existing nodes as is
                    // TODO: Determine why these need to be added in reverse order to be
                    //       handled correctly.  I think this is because a stack is used
                    //       when running the Shunting-Yard algorithm using the data from
                    //       this function, that reverses the order of children here so
                    //       they need to be reversed here so the Shunting-Yard algorithm
                    //       reverses them back.
                    newChildren.add(section[1])
                    newChildren.add(section[0])
                } else {
                    newChildren.addAll(section)
                }
            }
            ExpressionTexTalkNode(children = newChildren)
        } else {
            it
        }
    }

private fun splitBetweenInfixOperators(nodes: List<TexTalkNode>): List<List<TexTalkNode>> {
    val result = mutableListOf<List<TexTalkNode>>()
    var i = 0
    while (i < nodes.size) {
        val inner = mutableListOf<TexTalkNode>()
        while (i < nodes.size && !isSpecialOperator(nodes[i])) {
            inner.add(nodes[i++])
        }
        result.add(inner)

        if (i < nodes.size && isSpecialOperator(nodes[i])) {
            result.add(mutableListOf(nodes[i++]))
        }
    }
    return result
}

private enum class Associativity {
    Left,
    Right,
    Unknown
}

private fun getPrecedence(op: String): Int {
    return when {
        (op == "+" || op == "-") -> 1
        (op == "*" || op == "/") -> 2
        op == "^" -> 3
        op == "_" -> 4
        // the ... operator is special and has high precedence
        op == "..." -> 5
        op == "=" -> 6
        op == "!=" -> 7
        op == ":=" -> 8
        // operator ...+ has the precedence +
        // and ...* has the precedence of *
        op.contains("...") -> getPrecedence(op.replace("...", ""))
        else -> 0
    }
}

private fun getPrecedence(node: TexTalkNode) =
    when {
        isSpecialOperator(node) -> getPrecedence(getCommandOperatorSymbolText(node)!!)
        else ->
            throw ParseException(
                ParseError(
                    message = "Cannot get precedence of node '${node.toCode()}'",
                    row = -1,
                    column = -1))
    }

private fun getAssociativity(node: TexTalkNode) =
    when {
        isSpecialOperator(node) -> {
            val op = getCommandOperatorSymbolText(node)
            when {
                (op == "+" || op == "-" || op == "*" || op == "/" || op == "_") ->
                    Associativity.Left
                op == "^" -> Associativity.Right
                else -> Associativity.Unknown
            }
        }
        else -> Associativity.Unknown
    }

private fun runShuntingYard(root: TexTalkNode, isNodeRhsExpressions: Set<ExpressionTexTalkNode>) =
    root.transform {
        if (it is ExpressionTexTalkNode && !isNodeRhsExpressions.contains(it)) {
            val postfix = toPostfixForm(it.children)
            ExpressionTexTalkNode(children = postfixToTree(postfix))
        } else {
            it
        }
    }

private fun toPostfixForm(nodes: List<TexTalkNode>): List<TexTalkNode> {
    val argStack = newStack<TexTalkNode>()
    val opStack = newStack<TexTalkNode>()

    for (a in nodes) {
        if (!isSpecialOperator(a)) {
            argStack.push(a)
            continue
        }

        // 'a' is an operator
        if (opStack.isEmpty()) {
            opStack.push(a)
            continue
        }

        val topOp = opStack.peek()
        val topPrec = getPrecedence(topOp)
        val curPrec = getPrecedence(a)
        when {
            topPrec < curPrec -> {
                opStack.push(a)
            }
            topPrec > curPrec -> {
                argStack.push(opStack.pop())
                opStack.push(a)
            }
            else -> {
                // the operator on the top of the operator stack
                // and the current operator being viewed have the
                // same precedence
                when {
                    (getAssociativity(topOp) == Associativity.Left ||
                        getAssociativity(topOp) == Associativity.Unknown) -> {
                        // treat unknown operators as left associative
                        argStack.push(opStack.pop())
                        opStack.push(a)
                    }
                    getAssociativity(topOp) == Associativity.Right -> {
                        opStack.push(a)
                    }
                }
            }
        }
    }

    while (!opStack.isEmpty()) {
        argStack.push(opStack.pop())
    }

    val result = mutableListOf<TexTalkNode>()
    while (!argStack.isEmpty()) {
        result.add(argStack.pop())
    }
    result.reverse()
    return result
}

private fun postfixToTree(nodes: List<TexTalkNode>): List<TexTalkNode> {
    val stack = newStack<TexTalkNode>()
    for (n in nodes) {
        if (isSpecialOperator(n)) {
            if (stack.isEmpty()) {
                throw ParseException(
                    ParseError(
                        message =
                            "Expected two arguments for operator ${n.toCode()} but found none",
                        row = -1,
                        column = -1))
            }
            val rhs = stack.pop()
            if (stack.isEmpty()) {
                throw ParseException(
                    ParseError(
                        message = "Expected two arguments for operator ${n.toCode()} but found one",
                        row = -1,
                        column = -1))
            }
            val lhs = stack.pop()
            stack.push(OperatorTexTalkNode(lhs = lhs, command = n, rhs = rhs))
        } else {
            stack.push(n)
        }
    }

    val result = mutableListOf<TexTalkNode>()
    while (!stack.isEmpty()) {
        result.add(stack.pop())
    }

    return result
}

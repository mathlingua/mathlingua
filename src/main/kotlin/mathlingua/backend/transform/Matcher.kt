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

package mathlingua.backend.transform

import kotlin.math.max
import mathlingua.frontend.chalktalk.phase2.WrittenAsForm
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.support.Location
import mathlingua.frontend.textalk.ColonEqualsTexTalkNode
import mathlingua.frontend.textalk.Command
import mathlingua.frontend.textalk.CommandPart
import mathlingua.frontend.textalk.ExpressionTexTalkNode
import mathlingua.frontend.textalk.GroupTexTalkNode
import mathlingua.frontend.textalk.MappingNode
import mathlingua.frontend.textalk.OperatorTexTalkNode
import mathlingua.frontend.textalk.SequenceNode
import mathlingua.frontend.textalk.TexTalkNode
import mathlingua.frontend.textalk.TexTalkNodeType
import mathlingua.frontend.textalk.TexTalkTokenType
import mathlingua.frontend.textalk.TextTexTalkNode

internal data class Expansion(
    val text: String?, val errors: List<String>, val matchedTarget: Boolean)

internal fun OperatorTexTalkNode.signature() =
    when (this.command) {
        is Command -> this.command.signature()
        is TextTexTalkNode -> this.command.text
        else ->
            throw RuntimeException(
                "Cannot get a signature of an " + "operator with command ${this.command.toCode()}")
    }

internal fun Command.signature(): String {
    val builder = StringBuilder()
    builder.append('\\')
    for (i in this.parts.indices) {
        if (i > 0) {
            builder.append('.')
        }
        builder.append(this.parts[i].signature())
    }
    return builder.toString()
}

internal fun IdStatement.signature(): Signature? {
    val signatures = findAllStatementSignatures(stmt = this.toStatement(), ignoreLhsEqual = false)
    val form = signatures.toList().firstOrNull()?.form ?: return null
    return Signature(form = form, location = Location(this.row, this.column))
}

internal data class Substitutions(
    val doesMatch: Boolean,
    val substitutions: Map<String, List<TexTalkNode>>,
    val errors: List<String>)

internal fun getSubstitutions(
    pattern: OperatorTexTalkNode, value: OperatorTexTalkNode
): Substitutions {
    val subs =
        MutableSubstitutions(
            doesMatch = true, substitutions = mutableMapOf(), errors = mutableListOf())

    if ((pattern.lhs == null) != (value.lhs == null)) {
        subs.doesMatch = false
        return subs.toImmutable()
    }

    if (pattern.lhs != null) {
        if (pattern.lhs !is TextTexTalkNode) {
            subs.doesMatch = false
            subs.errors.add(
                "The left-hand-side of an infix operator " +
                    "pattern must be an identifier but found ${pattern.lhs.toCode()}")
            return subs.toImmutable()
        }

        subs.substitutions[pattern.lhs.text] = mutableListOf(value.lhs!!)
    }

    if ((pattern.rhs == null) != (value.rhs == null)) {
        subs.doesMatch = false
        return subs.toImmutable()
    }

    if (pattern.rhs != null) {
        if (pattern.rhs !is TextTexTalkNode) {
            subs.doesMatch = false
            subs.errors.add(
                "The right-hand-side of an infix operator " +
                    "pattern must be an identifier but found ${pattern.rhs.toCode()}")
            return subs.toImmutable()
        }

        subs.substitutions[pattern.rhs.text] = mutableListOf(value.rhs!!)
    }

    if ((pattern.command is TextTexTalkNode) && (value.command is TextTexTalkNode)) {
        if (pattern.command.text != value.command.text) {
            subs.doesMatch = false
        }
        return subs.toImmutable()
    }

    if ((pattern.command is Command) && (value.command is Command)) {
        val cmdSubs = getSubstitutions(pattern.command, value.command)
        subs.errors.addAll(cmdSubs.errors)
        subs.doesMatch = subs.doesMatch && cmdSubs.doesMatch
        for ((name, values) in cmdSubs.substitutions) {
            subs.substitutions[name] = values.toMutableList()
        }
        return subs.toImmutable()
    }

    throw RuntimeException(
        "Encountered a pattern or value operator with a " +
            "command that is an expression.  Pattern: ${pattern.toCode()}, Value: ${value.toCode()}")
}

// if there is a Command -> String pattern match and 'cmd' is the command
// then the match is store in 'operatorPatternToExpansion' as
//   OperatorTexTalkNode(lhs = null, command = cmd, rhs = null) -> String
// That is Command patterns, are OperatorTexTalkNode pattern with a
// left-hand-side or right-hand-side argument.
internal fun expandAsWritten(
    target: String?,
    node: TexTalkNode,
    operatorPatternToExpansion: Map<OperatorTexTalkNode, WrittenAsForm>
): Expansion {
    val sigToPatternExpansion = mutableMapOf<String, PatternExpansion>()
    for ((opPattern, expansion) in operatorPatternToExpansion) {
        sigToPatternExpansion[opPattern.signature()] =
            PatternExpansion(pattern = opPattern, expansion = expansion)
    }
    return expandAsWrittenImpl(target, node, sigToPatternExpansion, SurroundingKind.DoNothing)
}

// -----------------------------------------------------------------------------

private fun getSubstitutions(pattern: Command, value: Command): Substitutions {
    val errors = validatePattern(pattern)
    if (errors.isNotEmpty()) {
        return Substitutions(doesMatch = false, substitutions = emptyMap(), errors = errors)
    }

    val subs =
        MutableSubstitutions(
            doesMatch = true, substitutions = mutableMapOf(), errors = mutableListOf())

    if (pattern.parts.size == value.parts.size) {
        for (i in pattern.parts.indices) {
            findSubstitutions(pattern.parts[i], value.parts[i], subs)
        }
    }

    return subs.toImmutable()
}

private data class MutableSubstitutions(
    var doesMatch: Boolean,
    val substitutions: MutableMap<String, MutableList<TexTalkNode>>,
    val errors: MutableList<String>
) {
    fun toImmutable() =
        Substitutions(doesMatch = doesMatch, substitutions = substitutions, errors = errors)
}

private fun CommandPart.signature(): String {
    val builder = StringBuilder()
    builder.append(this.name.text)
    for (grp in this.namedGroups) {
        builder.append(':')
        builder.append(grp.name.text)
    }
    return builder.toString()
}

private fun findSubstitutions(
    pattern: GroupTexTalkNode?, value: GroupTexTalkNode?, subs: MutableSubstitutions
) {
    if ((pattern == null) != (value == null)) {
        subs.doesMatch = false
        if (pattern == null) {
            subs.errors.add("Unexpected group '${value?.toCode()}'")
        } else {
            subs.errors.add("Unexpected a match for '${pattern.toCode()}'")
        }
        return
    }

    if ((pattern == null) || (value == null)) {
        return
    }

    // this assumes that each parameters is either a TextTexTalkNode, MappingNode, or SequenceNode
    // with only the last one possibly variadic (and must be a TextTexTalkNode in that case)
    // That is, a signature can only have `X`, `f(x)`, `{x_{i}}_{i}` or `{f_{i}(x)}_{i}` as
    // parameters.
    // Thus, parameters of the form `X := ...`, `f(x) := ...` etc. are not supported.
    val paramNames =
        pattern.parameters.items
            .map {
                if (it.children[0] is MappingNode) {
                    (it.children[0] as MappingNode).name
                } else if (it.children[0] is SequenceNode) {
                    (it.children[0] as SequenceNode).mapping.name
                } else {
                    it.children[0] as TextTexTalkNode
                }
            }
            .map { it.text }
    val isVariadic =
        pattern.parameters.items.isNotEmpty() &&
            (pattern.parameters.items.last().children[0] is TextTexTalkNode) &&
            (pattern.parameters.items.last().children[0] as TextTexTalkNode).isVarArg

    val values = value.parameters.items
    if (isVariadic) {
        val numRequired = max(0, paramNames.size - 1)
        if (values.size < numRequired) {
            subs.doesMatch = false
            subs.errors.add(
                "Expected at least $numRequired arguments but found ${values.size} for '${value.toCode()}'")
            return
        }

        for (i in 0 until numRequired) {
            val exps: MutableList<TexTalkNode> = mutableListOf(values[i])
            subs.substitutions[paramNames[i]] = exps
        }

        val varArgs = mutableListOf<TexTalkNode>()
        for (i in numRequired until values.size) {
            varArgs.add(values[i])
        }
        subs.substitutions[paramNames.last()] = varArgs
    } else {
        val numRequired = paramNames.size
        if (values.size != numRequired) {
            subs.doesMatch = false
            subs.errors.add(
                "Expected exactly $numRequired arguments but found ${values.size} for '${value.toCode()}'")
            return
        }

        for (i in paramNames.indices) {
            val exps: MutableList<TexTalkNode> = mutableListOf(values[i])
            if (asDotDotDotOperator(exps) != null) {
                // a non-variadic variable (i.e. x as compared to x?)
                // cannot match a sequence `a...b` so if exps describes
                // such as sequence then there isn't a match
                subs.doesMatch = false
            } else {
                subs.substitutions[paramNames[i]] = exps
            }
        }
    }
}

private fun handleVariadicGroupSubstitutions(
    patternGroups: List<GroupTexTalkNode>,
    valueGroups: List<GroupTexTalkNode>,
    subs: MutableSubstitutions
) {
    val isLastVarArg = patternGroups.isNotEmpty() && patternGroups.last().isVarArg
    if (isLastVarArg) {
        if (valueGroups.size >= patternGroups.size) {
            for (i in patternGroups.indices) {
                findSubstitutions(patternGroups[i], valueGroups[i], subs)
            }

            val variadicName =
                (patternGroups.last().parameters.items[0].children[0] as TextTexTalkNode).text
            // this function assumes if there is a variadic group it is only the last group and that
            // group
            // has a single parameter that is a TextTexTalk node that is not variadic itself
            for (i in patternGroups.size until valueGroups.size) {
                val items = valueGroups[i].parameters.items
                if (items.size != 1) {
                    subs.doesMatch = false
                    subs.errors.add(
                        "A variadic group can only contain a single item but found ${items.size} for '${valueGroups[i]}")
                    continue
                }

                if (!subs.substitutions.containsKey(variadicName)) {
                    subs.substitutions[variadicName] = mutableListOf()
                }

                if (items.size != 1) {
                    subs.errors.add(
                        "A variadic group can only contain a single item but found ${items.size} for '${valueGroups[i].toCode()}'")
                    subs.doesMatch = false
                } else {
                    subs.substitutions[variadicName]!!.add(valueGroups[i].parameters.items[0])
                }
            }
        } else {
            subs.doesMatch = false
            val groupOrGroups =
                if (patternGroups.size == 1) {
                    "group"
                } else {
                    "groups"
                }
            subs.errors.add(
                "Expected at least ${patternGroups.size} curly brace parameter $groupOrGroups but found ${valueGroups.size}")
        }
    } else {
        if (valueGroups.size == patternGroups.size) {
            for (i in patternGroups.indices) {
                findSubstitutions(patternGroups[i], valueGroups[i], subs)
            }
        } else {
            subs.doesMatch = false
            val groupOrGroups =
                if (patternGroups.size == 1) {
                    "group"
                } else {
                    "groups"
                }
            subs.errors.add(
                "Expected exactly ${patternGroups.size} curly brace parameter $groupOrGroups but found ${valueGroups.size}")
        }
    }
}

private fun findSubstitutions(
    pattern: CommandPart, value: CommandPart, subs: MutableSubstitutions
) {
    if (pattern.name != value.name) {
        subs.doesMatch = false
        subs.errors.add("Name mismatch.  Expected ${pattern.name} but found ${value.name}")
        return
    }

    findSubstitutions(pattern.square, value.square, subs)
    findSubstitutions(pattern.subSup?.sub, value.subSup?.sub, subs)
    findSubstitutions(pattern.subSup?.sup, value.subSup?.sup, subs)

    handleVariadicGroupSubstitutions(pattern.groups, value.groups, subs)

    if (pattern.paren == null && value.paren != null) {
        // if the pattern doesn't accept a paren but the value has one,
        // then it isn't a match
        subs.doesMatch = false
    } else if (pattern.paren != null && value.paren != null) {
        // if the value and pattern both have parens, they have to match
        // in the number of parameters
        findSubstitutions(pattern.paren, value.paren, subs)
    }

    // it is fine if the pattern has a paren but the value does not
    // because then the value is assumed to have (?,?) with the number
    // of ? matching the number of parameters that are expected

    // it is also fine if neither the pattern or the value have parens
    // in that case it is a match but there are no substitutions needed

    if (pattern.namedGroups.size == value.namedGroups.size) {
        for (i in pattern.namedGroups.indices) {
            val patternGrp = pattern.namedGroups[i]
            val valGrp = value.namedGroups[i]
            if (patternGrp.name != valGrp.name) {
                subs.doesMatch = false
                subs.errors.add(
                    "Mismatched named group: Expected ${patternGrp.name} groups but found ${valGrp.name} for '${value.toCode()}'")
            } else {
                handleVariadicGroupSubstitutions(patternGrp.groups, valGrp.groups, subs)
            }
        }
    } else {
        subs.doesMatch = false
        subs.errors.add(
            "Expected exactly ${pattern.namedGroups.size} named groups but found ${value.namedGroups.size} for '${value.toCode()}'")
    }
}

private fun validatePatternGroupImpl(
    group: GroupTexTalkNode?, canBeVarArg: Boolean, description: String, errors: MutableList<String>
) {
    group ?: return

    if (group.isVarArg && !canBeVarArg) {
        errors.add("$description cannot be variadic")
    }

    if (group.isVarArg &&
        (group.parameters.items.size != 1 ||
            (group.parameters.items[0].children.size != 1) ||
            (group.parameters.items[0].children[0] !is TextTexTalkNode) ||
            (group.parameters.items[0].children[0] as TextTexTalkNode).isVarArg)) {
        errors.add(
            "A variadic group can only have a single identifier parameter that is not variadic")
        return
    }

    for (i in group.parameters.items.indices) {
        val expression = group.parameters.items[i]
        if (expression.children.size != 1) {
            errors.add("Cannot have a parameter with more than one value: '${expression.toCode()}'")
            continue
        }

        val item = expression.children[0]
        if (item !is TextTexTalkNode && item !is MappingNode && item !is SequenceNode) {
            errors.add(
                "Parameters can only be of the form X, f(x), {x_{i}}_{i}, or {f_{i}(x)}_{i} but found '${item.toCode()}'")
            continue
        }

        if (item is TextTexTalkNode && item.isVarArg && i != group.parameters.items.size - 1) {
            errors.add("Only the last parameter in a group can be variadic: '${item.toCode()}'")
        }
    }
}

private fun validatePatternImpl(part: CommandPart, errors: MutableList<String>) {
    validatePatternGroupImpl(part.square, false, "A square group", errors)
    validatePatternGroupImpl(part.subSup?.sub, false, "A ^ group", errors)
    validatePatternGroupImpl(part.subSup?.sup, false, "A _ group", errors)
    validatePatternGroupImpl(part.paren, true, "A paren group", errors)

    for (i in part.groups.indices) {
        val canBeVarArg = i == part.groups.size - 1
        val description =
            if (i == part.groups.size - 1) {
                "The last group"
            } else {
                "A group"
            }
        validatePatternGroupImpl(part.groups[i], canBeVarArg, description, errors)
    }
    for (i in part.namedGroups.indices) {
        val namedGroup = part.namedGroups[i]
        for (j in namedGroup.groups.indices) {
            val canBeVarArg = j == namedGroup.groups.size - 1
            val description =
                if (canBeVarArg) {
                    "The last group of a named group"
                } else {
                    "A named group"
                }
            validatePatternGroupImpl(namedGroup.groups[j], canBeVarArg, description, errors)
        }
    }
}

private fun validatePattern(command: Command): List<String> {
    val errors = mutableListOf<String>()
    for (part in command.parts) {
        validatePatternImpl(part, errors)
    }
    return errors
}

private data class PatternExpansion(val pattern: OperatorTexTalkNode, val expansion: WrittenAsForm)

private fun expandAsWrittenImplImpl(
    fromTarget: String?, cmd: Command, sigToPatternExpansion: Map<String, PatternExpansion>
) =
    expandAsWrittenImplImpl(
        fromTarget,
        OperatorTexTalkNode(lhs = null, command = cmd, rhs = null),
        sigToPatternExpansion)

private fun asDotDotDotOperator(nodes: List<TexTalkNode>): OperatorTexTalkNode? {
    if (nodes.size != 1 || nodes[0] !is ExpressionTexTalkNode) {
        return null
    }

    val exp = nodes[0] as ExpressionTexTalkNode
    if (exp.children.size != 1 || exp.children[0] !is OperatorTexTalkNode) {
        return null
    }

    val op = exp.children[0] as OperatorTexTalkNode
    if (op.command !is TextTexTalkNode) {
        return null
    }

    return if (op.command.text == "...") {
        op
    } else {
        null
    }
}

private fun getNameFromTarget(target: String): String {
    var result = target.trim()
    val colonEqualsIndex = result.indexOf(":=")
    if (colonEqualsIndex >= 0) {
        result = result.substring(0, colonEqualsIndex)
    }
    val leftParenIndex = result.indexOf('(')
    if (leftParenIndex >= 0) {
        result = result.substring(0, leftParenIndex)
    }
    return result.trim()
}

private fun expandAsWrittenImplImpl(
    fromTarget: String?,
    op: OperatorTexTalkNode,
    sigToPatternExpansion: Map<String, PatternExpansion>
): Expansion {
    val patternExpansion = sigToPatternExpansion[op.signature()]

    if (patternExpansion == null &&
        op.command is TextTexTalkNode &&
        op.command.tokenType == TexTalkTokenType.Operator) {
        // There isn't a matching pattern.  However, the operator is a non-backslash
        // command such as +, -, ++, etc. and so it is not an error to not find the
        // matching pattern.  Instead, the operator itself can be rendered.
        return Expansion(text = null, errors = emptyList(), matchedTarget = false)
    } else if (patternExpansion == null) {
        return Expansion(
            text = null,
            errors = listOf("No matching definition found for ${op.toCode()}"),
            matchedTarget = false)
    }

    val subs = getSubstitutions(patternExpansion.pattern, op)

    val toTarget = patternExpansion.expansion.target
    val substitutions = subs.substitutions.toMutableMap()
    var expansion = patternExpansion.expansion.form

    if (toTarget != null) {
        if (fromTarget == null) {
            expansion = expansion.replace("$toTarget?", "?")
        } else {
            val fromName = getNameFromTarget(fromTarget)
            val toName = getNameFromTarget(toTarget)
            substitutions[toName] =
                listOf(
                    TextTexTalkNode(
                        type = TexTalkNodeType.Identifier,
                        tokenType = TexTalkTokenType.Identifier,
                        text = fromName,
                        isVarArg = false))
        }
    }

    if (!subs.doesMatch) {
        return Expansion(text = null, errors = subs.errors, matchedTarget = false)
    }

    var matchedTarget = false
    for ((name, exp) in substitutions) {
        // replace any 'name?' with the expansions of the list of expressions 'exp'
        // separated by a space
        if (exp.isNotEmpty()) {
            val expToStringParen =
                exp.joinToString(" ") {
                    expandAsWrittenImpl(
                            fromTarget, it, sigToPatternExpansion, SurroundingKind.AddParens)
                        .text
                        ?: it.toCode()
                }
            matchedTarget = matchedTarget || (name == fromTarget && expansion.contains("$name+?"))
            expansion = expansion.replace("$name+?", expToStringParen)

            val expToStringNoParen =
                exp.joinToString(" ") {
                    expandAsWrittenImpl(
                            fromTarget, it, sigToPatternExpansion, SurroundingKind.RemoveParens)
                        .text
                        ?: it.toCode()
                }
            matchedTarget = matchedTarget || (name == fromTarget && expansion.contains("$name-?"))
            expansion = expansion.replace("$name-?", expToStringNoParen)

            val expToString =
                exp.joinToString(" ") {
                    expandAsWrittenImpl(
                            fromTarget, it, sigToPatternExpansion, SurroundingKind.DoNothing)
                        .text
                        ?: it.toCode()
                }
            matchedTarget = matchedTarget || (name == fromTarget && expansion.contains("$name?"))
            expansion = expansion.replace("$name?", expToString)
        }

        // now handle the case where the substitution is specified as 'name{...}?'
        // instead of just 'name?'
        var hasPlusQuestionMark = false
        var hasMinusQuestionMark = false
        var startIndex = 0
        val target = "$name{"
        while (true) {
            val index = expansion.indexOf(target, startIndex)
            if (index < 0) {
                break
            }

            // If 'name{' is found in expansion, but the full string in expansion is
            // not of the form 'name{...}?' update the startIndex to point past the
            // index of the 'name{' found.  Otherwise, there will be an infinite loop
            // where the loop will keep processing the invalid 'name{'.
            startIndex = index + target.length

            val innerTextBuffer = StringBuilder()
            var isValid = false
            var leftCurlyCount = 1
            var i = index + target.length
            while (i < expansion.length) {
                val c = expansion[i]
                val prevNotBackslash = i - 1 < 0 || expansion[i - 1] != '\\'
                if (c == '{' && prevNotBackslash) {
                    leftCurlyCount++
                } else if (c == '}' && prevNotBackslash) {
                    leftCurlyCount--
                }
                i++
                if (c == '}' &&
                    leftCurlyCount == 0 &&
                    i < expansion.length &&
                    expansion[i] == '+' &&
                    i + 1 < expansion.length &&
                    expansion[i + 1] == '?') {
                    isValid = true
                    i += 2 // move past the + and ?
                    hasPlusQuestionMark = true
                    break
                } else if (c == '}' &&
                    leftCurlyCount == 0 &&
                    i < expansion.length &&
                    expansion[i] == '-' &&
                    i + 1 < expansion.length &&
                    expansion[i + 1] == '?') {
                    isValid = true
                    i += 2 // move past the - and ?
                    hasMinusQuestionMark = true
                    break
                } else if (c == '}' &&
                    leftCurlyCount == 0 &&
                    i < expansion.length &&
                    expansion[i] == '?') {
                    isValid = true
                    break
                } else {
                    innerTextBuffer.append(c)
                }
            }

            val surroundingKind =
                if (hasPlusQuestionMark) {
                    SurroundingKind.AddParens
                } else if (hasMinusQuestionMark) {
                    SurroundingKind.RemoveParens
                } else {
                    SurroundingKind.DoNothing
                }

            if (isValid) {
                var innerText = innerTextBuffer.toString()
                val expansionPrefix = expansion.substring(0, index)
                // add 2 at the end for the trailing }?
                // or  3 at the end for the trailing }+? or }-?
                val delta =
                    if (hasPlusQuestionMark || hasMinusQuestionMark) {
                        3
                    } else {
                        2
                    }
                val expansionSuffix =
                    expansion.substring(index + target.length + innerText.length + delta)

                val prefixRegex = Regex("(.*)\\.\\.\\.")
                val infixRegex = Regex("(.*)\\.\\.\\.(.*)\\.\\.\\.(.*)")
                val suffixRegex = Regex("\\.\\.\\.(.*)")

                // if the pattern is name{}?? then the user has requested parens be added around
                // a match if it is of a complex form (i.e. 'a + b' but not 'a')
                if (infixRegex.matches(innerText)) {
                    val args =
                        exp.map {
                            expandAsWrittenImpl(
                                    fromTarget, it, sigToPatternExpansion, surroundingKind)
                                .text
                                ?: it.toCode()
                        }
                    val result = infixRegex.find(innerText)
                    if (result != null && result.groupValues.size >= 4) {
                        val prefix = result.groupValues[1]
                        val separator = result.groupValues[2]
                        val suffix = result.groupValues[3]
                        val opNode = asDotDotDotOperator(exp)
                        val joinedArgs =
                            if (opNode != null) {
                                "${opNode.lhs?.toCode() ?: ""}$separator${opNode.command.toCode()}$separator${opNode.rhs?.toCode() ?: ""}"
                            } else {
                                args.joinToString(separator)
                            }
                        val pattern = result.groupValues[0]
                        // pattern is of the form name{prefix...separator...suffix}?
                        // Note: if 'name{...a...b}? is given, for example, then
                        //       'prefix' above will be the empty string
                        //       'separator' will be 'a' and
                        //       'suffix' will be 'b'
                        innerText = innerText.replace(pattern, joinedArgs)
                        innerText = "$prefix$innerText$suffix"
                    }
                } else if (prefixRegex.matches(innerText)) {
                    val args =
                        exp.map {
                            expandAsWrittenImpl(
                                    fromTarget, it, sigToPatternExpansion, surroundingKind)
                                .text
                                ?: it.toCode()
                        }
                    val result = prefixRegex.find(innerText)
                    if (result != null && result.groupValues.size >= 2) {
                        val separator = result.groupValues[1]
                        val joinedArgsBuilder = StringBuilder()
                        for (a in args) {
                            joinedArgsBuilder.append(separator)
                            joinedArgsBuilder.append(a)
                        }
                        val joinedArgs = joinedArgsBuilder.toString()
                        val pattern = result.groupValues[0]
                        // pattern is of the form name{separator...}?
                        innerText = innerText.replace(pattern, joinedArgs)
                    }
                } else if (suffixRegex.matches(innerText)) {
                    val args =
                        exp.map {
                            expandAsWrittenImpl(
                                    fromTarget, it, sigToPatternExpansion, surroundingKind)
                                .text
                                ?: it.toCode()
                        }
                    val result = suffixRegex.find(innerText)
                    if (result != null && result.groupValues.size >= 2) {
                        val separator = result.groupValues[1]
                        val joinedArgsBuilder = StringBuilder()
                        for (a in args) {
                            joinedArgsBuilder.append(a)
                            joinedArgsBuilder.append(separator)
                        }
                        val joinedArgs = joinedArgsBuilder.toString()
                        val pattern = result.groupValues[0]
                        // pattern is of the form name{...separator}?
                        innerText = innerText.replace(pattern, joinedArgs)
                    }
                }

                // replace 'name{...}?' (or 'name{}??') with 'innerText'
                expansion = expansionPrefix + innerText + expansionSuffix
            }
        }
    }

    return Expansion(text = expansion, errors = emptyList(), matchedTarget = matchedTarget)
}

private fun shouldHaveParen(node: TexTalkNode): Boolean {
    if (node is TextTexTalkNode || node is Command) {
        return false
    }

    if (node is OperatorTexTalkNode) {
        return node.lhs != null || node.rhs != null
    }

    if (node is ExpressionTexTalkNode) {
        if (node.children.isEmpty()) {
            return false
        }
        return node.children.size > 1 || shouldHaveParen(node.children[0])
    }

    return true
}

private enum class SurroundingKind {
    AddParens,
    RemoveParens,
    DoNothing
}

private fun expandAsWrittenImpl(
    fromTarget: String?,
    node: TexTalkNode,
    sigToPatternExpansion: Map<String, PatternExpansion>,
    surroundingKind: SurroundingKind
): Expansion {
    val errors = mutableListOf<String>()
    var matchedTarget = false
    val code =
        node.toCode {
            when (it) {
                is Command -> {
                    val result = expandAsWrittenImplImpl(fromTarget, it, sigToPatternExpansion)
                    errors.addAll(result.errors)
                    matchedTarget = matchedTarget || result.matchedTarget
                    result.text
                }
                is OperatorTexTalkNode -> {
                    val result = expandAsWrittenImplImpl(fromTarget, it, sigToPatternExpansion)
                    errors.addAll(result.errors)
                    matchedTarget = matchedTarget || result.matchedTarget
                    result.text
                }
                is ColonEqualsTexTalkNode -> {
                    val lhsText = it.lhs.toCode()
                    val rhsResult =
                        expandAsWrittenImpl(
                            fromTarget, it.rhs, sigToPatternExpansion, surroundingKind)
                    errors.addAll(rhsResult.errors)
                    matchedTarget = matchedTarget || rhsResult.matchedTarget
                    "$lhsText := ${rhsResult.text}"
                }
                else -> null
            }
        }

    return Expansion(
        text =
            if (surroundingKind == SurroundingKind.AddParens &&
                shouldHaveParen(node) &&
                !code.startsWith("(") &&
                !code.endsWith(")")) {
                "\\left ( $code \\right )"
            } else if (surroundingKind == SurroundingKind.RemoveParens) {
                code.trim().removeSurrounding("(", ")")
            } else {
                code
            },
        errors = errors,
        matchedTarget = matchedTarget)
}

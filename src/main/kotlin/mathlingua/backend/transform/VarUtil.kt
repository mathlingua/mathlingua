/*
 * Copyright 2019 The MathLingua Authors
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

import mathlingua.MutableMultiSet
import mathlingua.backend.isOperatorName
import mathlingua.frontend.chalktalk.phase1.ast.Abstraction
import mathlingua.frontend.chalktalk.phase1.ast.AbstractionPart
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Token
import mathlingua.frontend.chalktalk.phase2.ast.clause.AbstractionNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.AssignmentNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Identifier
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.clause.TupleNode
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.exists.ExistsGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.existsUnique.ExistsUniqueGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.forAll.ForAllGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.HasUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.RequiringSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.equality.EqualityGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.ConjectureGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.GivenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.LocationTracker
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.textalk.ColonColonEqualsTexTalkNode
import mathlingua.frontend.textalk.ColonEqualsTexTalkNode
import mathlingua.frontend.textalk.Command
import mathlingua.frontend.textalk.CommandPart
import mathlingua.frontend.textalk.GroupTexTalkNode
import mathlingua.frontend.textalk.MappingNode
import mathlingua.frontend.textalk.OperatorTexTalkNode
import mathlingua.frontend.textalk.ParametersTexTalkNode
import mathlingua.frontend.textalk.TexTalkNode
import mathlingua.frontend.textalk.TexTalkNodeType
import mathlingua.frontend.textalk.TexTalkTokenType
import mathlingua.frontend.textalk.TextTexTalkNode
import mathlingua.newMutableMultiSet

internal data class Var(val name: String, val isPlaceholder: Boolean) {
    override fun toString(): String {
        return name
    }
}

internal fun getVarsPhase2Node(node: Phase2Node): List<Var> {
    val vars = mutableListOf<Var>()
    getVarsImplPhase2Node(node, vars)
    return vars
}

internal fun getVarsTexTalkNode(
    texTalkNode: TexTalkNode,
    isInLhsColonEquals: Boolean,
    groupScope: GroupScope,
    isInIdStatement: Boolean
): List<Var> {
    val vars = mutableListOf<Var>()
    getVarsImplTexTalkNode(texTalkNode, vars, isInLhsColonEquals, groupScope, isInIdStatement)
    return vars
}

internal fun renameVarsTexTalkNode(texTalkNode: TexTalkNode, map: Map<String, String>) =
    texTalkNode.transform {
        if (it is TextTexTalkNode) {
            it.copy(text = map[it.text] ?: it.text)
        } else {
            it
        }
    }

internal fun checkVarsPhase2Node(
    topLevel: TopLevelGroup, node: Phase2Node, tracker: LocationTracker
): Set<ParseError> {
    val errors = mutableListOf<ParseError>()
    checkVarsImplPhase2Node(topLevel, node, VarMultiSet(), tracker, errors)
    return errors.toSet()
}

// -----------------------------------------------------------------------------

private fun getVarsImplPhase1Node(
    node: Phase1Node, vars: MutableList<Var>, isInPlaceholderScope: Boolean
) {
    if (node is Phase1Token) {
        val index = node.text.indexOf('_')
        if (index >= 0) {
            // If a Phase1Node is of the form `x_i` then record `x` as a variable name
            // This will occur if a Defines: has a target of the form {x_i}_i
            vars.add(
                Var(name = node.text.substring(0, index), isPlaceholder = isInPlaceholderScope))
        } else {
            // If the variable is variadic, store the non-variadic version of the name.
            // That is if it is `x...` store `x`.
            vars.add(
                Var(name = node.text.removeSuffix("..."), isPlaceholder = isInPlaceholderScope))
        }
    } else if (node is AbstractionPart) {
        getVarsImplPhase1Node(node.name, vars, isInPlaceholderScope = false)
        if (node.params != null) {
            for (param in node.params) {
                getVarsImplPhase1Node(param, vars, isInPlaceholderScope = true)
            }
        }

        if (node.subParams != null) {
            for (param in node.subParams) {
                getVarsImplPhase1Node(param, vars, isInPlaceholderScope = true)
            }
        }

        if (node.tail != null) {
            getVarsImplPhase1Node(node.tail, vars, isInPlaceholderScope = false)
        }
    } else if (node is Abstraction) {
        if (node.subParams != null) {
            for (token in node.subParams) {
                getVarsImplPhase1Node(token, vars, isInPlaceholderScope = true)
            }
        }
        for (part in node.parts) {
            getVarsImplPhase1Node(part, vars, isInPlaceholderScope = false)
        }
    } else {
        node.forEach { getVarsImplPhase1Node(it, vars, isInPlaceholderScope) }
    }
}

private fun getVarsImplPhase2Node(node: Phase2Node, vars: MutableList<Var>) {
    if (node is Identifier) {
        vars.add(Var(name = node.name.removeSuffix("..."), isPlaceholder = false))
    } else if (node is TupleNode) {
        getVarsImplPhase1Node(node.tuple, vars, isInPlaceholderScope = false)
    } else if (node is AbstractionNode) {
        getVarsImplPhase1Node(node.abstraction, vars, isInPlaceholderScope = false)
    } else if (node is AssignmentNode) {
        getVarsImplPhase1Node(node.assignment, vars, isInPlaceholderScope = false)
    } else if (node is IdStatement) {
        getVarsImplPhase2Node(node.toStatement(), vars)
    } else if (node is Statement) {
        when (node.texTalkRoot) {
            is ValidationSuccess -> {
                getVarsImplTexTalkNode(
                    texTalkNode = node.texTalkRoot.value,
                    vars = vars,
                    isInLhsColonEquals = false,
                    groupScope = GroupScope.InNone,
                    isInIdStatement = false)
            }
        }
    } else {
        node.forEach { getVarsImplPhase2Node(it, vars) }
    }
}

internal enum class GroupScope {
    InSquare,
    InParen,
    InCurly,
    InNone
}

private fun getVarsImplTexTalkNode(
    texTalkNode: TexTalkNode,
    vars: MutableList<Var>,
    isInLhsColonEquals: Boolean,
    groupScope: GroupScope,
    isInIdStatement: Boolean
) {
    if (texTalkNode is TextTexTalkNode) {
        if (texTalkNode.tokenType != TexTalkTokenType.ColonEquals &&
            texTalkNode.tokenType != TexTalkTokenType.ColonColonEquals) {
            vars.add(
                Var(
                    name = texTalkNode.text.removeSuffix("..."),
                    isPlaceholder =
                        groupScope == GroupScope.InSquare ||
                            (groupScope == GroupScope.InParen && isInIdStatement) ||
                            (groupScope != GroupScope.InNone && isInLhsColonEquals)))
        }
    } else if (texTalkNode is OperatorTexTalkNode) {
        if (texTalkNode.lhs != null) {
            getVarsImplTexTalkNode(
                texTalkNode.lhs, vars, isInLhsColonEquals, GroupScope.InParen, isInIdStatement)
        }

        if (texTalkNode.rhs != null) {
            getVarsImplTexTalkNode(
                texTalkNode.rhs, vars, isInLhsColonEquals, GroupScope.InParen, isInIdStatement)
        }

        getVarsImplTexTalkNode(
            texTalkNode.command, vars, isInLhsColonEquals, groupScope, isInIdStatement)
    } else if (texTalkNode is ColonEqualsTexTalkNode) {
        getVarsImplTexTalkNode(texTalkNode.lhs, vars, true, groupScope, isInIdStatement)
        getVarsImplTexTalkNode(texTalkNode.rhs, vars, false, groupScope, isInIdStatement)
    } else if (texTalkNode is GroupTexTalkNode) {
        getVarsImplTexTalkNode(
            texTalkNode.parameters,
            vars,
            isInLhsColonEquals = isInLhsColonEquals,
            groupScope =
                when (texTalkNode.type) {
                    TexTalkNodeType.SquareGroup -> {
                        GroupScope.InSquare
                    }
                    TexTalkNodeType.CurlyGroup -> {
                        GroupScope.InCurly
                    }
                    TexTalkNodeType.ParenGroup -> {
                        GroupScope.InParen
                    }
                    else -> {
                        GroupScope.InNone
                    }
                },
            isInIdStatement)
    } else if (texTalkNode is CommandPart) {
        // The following line is commented out since otherwise the
        // signature `\some.function` would have `some` and `function` as
        // valid variables, which is not expected.  Instead, `\some.function`
        // should be viewed as a valid signature.
        // vars.add(Var(name = texTalkNode.name.text, isPlaceholder = false))
        for (grp in texTalkNode.groups) {
            getVarsImplTexTalkNode(
                grp,
                vars,
                isInLhsColonEquals,
                when (grp.type) {
                    TexTalkNodeType.CurlyGroup -> {
                        GroupScope.InCurly
                    }
                    TexTalkNodeType.SquareGroup -> {
                        GroupScope.InSquare
                    }
                    TexTalkNodeType.ParenGroup -> {
                        GroupScope.InParen
                    }
                    else -> {
                        GroupScope.InNone
                    }
                },
                isInIdStatement)
        }
        for (grp in texTalkNode.namedGroups) {
            grp.groups.forEach {
                getVarsImplTexTalkNode(
                    it,
                    vars,
                    isInLhsColonEquals,
                    when (grp.type) {
                        TexTalkNodeType.CurlyGroup -> {
                            GroupScope.InCurly
                        }
                        TexTalkNodeType.SquareGroup -> {
                            GroupScope.InSquare
                        }
                        TexTalkNodeType.ParenGroup -> {
                            GroupScope.InParen
                        }
                        else -> {
                            GroupScope.InNone
                        }
                    },
                    isInIdStatement)
            }
        }
        if (texTalkNode.paren != null) {
            getVarsImplTexTalkNode(
                texTalkNode.paren, vars, isInLhsColonEquals, GroupScope.InParen, isInIdStatement)
        }
        if (texTalkNode.square != null) {
            getVarsImplTexTalkNode(
                texTalkNode.square, vars, isInLhsColonEquals, GroupScope.InSquare, isInIdStatement)
        }
        if (texTalkNode.subSup != null) {
            getVarsImplTexTalkNode(
                texTalkNode.subSup, vars, isInLhsColonEquals, groupScope, isInIdStatement)
        }
    } else if (texTalkNode is ParametersTexTalkNode) {
        texTalkNode.forEach {
            getVarsImplTexTalkNode(it, vars, isInLhsColonEquals, groupScope, isInIdStatement)
        }
    } else {
        texTalkNode.forEach {
            getVarsImplTexTalkNode(it, vars, isInLhsColonEquals, groupScope, isInIdStatement)
        }
    }
}

// -----------------------------------------------------------------------------

private fun checkVarsImplPhase2Node(
    topLevel: TopLevelGroup,
    node: Phase2Node,
    vars: VarMultiSet,
    tracker: LocationTracker,
    errors: MutableList<ParseError>
): List<Var> {
    val varsToRemove = VarMultiSet()
    val location =
        tracker.getLocationOf(node) ?: tracker.getLocationOf(topLevel) ?: Location(-1, -1)

    val requiringSection =
        when (node) {
            is DefinesGroup -> node.requiringSection
            is StatesGroup -> node.requiringSection
            else -> null
        }

    if (requiringSection != null) {
        varsToRemove.addAll(
            checkRequiringSectionVars(topLevel, requiringSection, vars, tracker, errors))
    }

    if (node is DefinesGroup) {
        val whenSection = node.whenSection
        if (whenSection != null) {
            varsToRemove.addAll(
                checkWhenSectionVars(
                    node = whenSection, vars = vars, tracker = tracker, errors = errors))
        }
        varsToRemove.addAll(checkVarsImplIdStatement(topLevel, node.id, vars, tracker, errors))
        varsToRemove.addAll(
            checkDefineSectionVars(topLevel, node.definesSection, vars, tracker, errors))
    }

    if (node is HasUsingSection && node.usingSection != null) {
        for (clause in node.usingSection!!.clauses.clauses) {
            val clauseLocation = tracker.getLocationOf(clause) ?: Location(-1, -1)
            if (clause !is Statement) {
                errors.add(
                    ParseError(
                        message = "A using section can only contain statements",
                        row = clauseLocation.row,
                        column = clauseLocation.column))
                continue
            }

            val statement = clause
            val validation = statement.texTalkRoot
            if (validation is ValidationSuccess) {
                val root = validation.value
                if (root.children.size != 1 || root.children[0] !is ColonEqualsTexTalkNode) {
                    errors.add(
                        ParseError(
                            message =
                                "The statements in a using section must be of the form '... := ...'",
                            row = clauseLocation.row,
                            column = clauseLocation.column))
                    continue
                }

                val colonEquals = root.children[0] as ColonEqualsTexTalkNode
                val params = colonEquals.lhs
                if (params.items.size != 1) {
                    errors.add(
                        ParseError(
                            message =
                                "The left-hand side of an operator must have exactly one argument",
                            row = clauseLocation.row,
                            column = clauseLocation.column))
                } else {
                    val exp = params.items[0]
                    if (exp.children.isNotEmpty()) {
                        val child = exp.children[0]
                        if (child is MappingNode) {
                            // handle forms of the type `f(x) := ...`
                            val v = Var(name = child.name.text, isPlaceholder = false)
                            vars.add(v)
                            varsToRemove.add(v)
                        } else if (child is OperatorTexTalkNode) {
                            if (child.command is TextTexTalkNode) {
                                val cmd = child.command.text
                                val v = Var(name = cmd, isPlaceholder = true)
                                vars.add(v)
                                varsToRemove.add(v)
                            }

                            if (child.lhs != null && child.lhs is TextTexTalkNode) {
                                val v = Var(name = child.lhs.text, isPlaceholder = true)
                                vars.add(v)
                                varsToRemove.add(v)
                            }

                            if (child.rhs != null && child.rhs is TextTexTalkNode) {
                                val v = Var(name = child.rhs.text, isPlaceholder = true)
                                vars.add(v)
                                varsToRemove.add(v)
                            }
                        } else if (child is GroupTexTalkNode &&
                            child.parameters.items.isNotEmpty() &&
                            child.parameters.items[0].children.isNotEmpty() &&
                            child.parameters.items[0].children[0] is TextTexTalkNode) {
                            val text =
                                (child.parameters.items[0].children[0] as TextTexTalkNode).text
                            val v = Var(name = text, isPlaceholder = true)
                            vars.add(v)
                            varsToRemove.add(v)
                        } else if (child is TextTexTalkNode) {
                            val name = child.text
                            val v = Var(name = name, isPlaceholder = true)
                            vars.add(v)
                            varsToRemove.add(v)
                        }
                    }
                }
            }
        }
    }

    when (node) {
        is StatesGroup -> {
            varsToRemove.addAll(checkVarsImplIdStatement(topLevel, node.id, vars, tracker, errors))
        }
        is TheoremGroup -> {
            if (node.givenSection != null) {
                varsToRemove.addAll(
                    checkGivenSectionVars(topLevel, node.givenSection, vars, tracker, errors))
            }
        }
        is AxiomGroup -> {
            if (node.givenSection != null) {
                varsToRemove.addAll(
                    checkGivenSectionVars(topLevel, node.givenSection, vars, tracker, errors))
            }
        }
        is ConjectureGroup -> {
            if (node.givenSection != null) {
                varsToRemove.addAll(
                    checkGivenSectionVars(topLevel, node.givenSection, vars, tracker, errors))
            }
        }
        is ForAllGroup -> {
            val forAllVars =
                node.forAllSection.targets.map { getVarsPhase2Node(node = it) }.flatten()
            for (v in forAllVars) {
                if (vars.hasConflict(v)) {
                    errors.add(
                        ParseError(
                            message = "Duplicate defined symbol '$v' in `forAll:`",
                            row = location.row,
                            column = location.column))
                }
                vars.add(v)
            }
            varsToRemove.addAll(forAllVars)
        }
        is EqualityGroup -> {
            val betweenVars =
                node.betweenSection.targets.map { getVarsPhase2Node(node = it) }.flatten()
            for (v in betweenVars) {
                if (vars.hasConflict(v)) {
                    errors.add(
                        ParseError(
                            message = "Duplicate defined symbol '$v' in `equality:`",
                            row = location.row,
                            column = location.column))
                }
                vars.add(v)
            }
            varsToRemove.addAll(betweenVars)
        }
        is ExistsGroup -> {
            val existsVars =
                node.existsSection.identifiers.map { getVarsPhase2Node(node = it) }.flatten()
            for (v in existsVars) {
                if (vars.hasConflict(v)) {
                    errors.add(
                        ParseError(
                            message = "Duplicate defined symbol '$v' in `exists:`",
                            row = location.row,
                            column = location.column))
                }
                vars.add(v)
            }
            varsToRemove.addAll(existsVars)
        }
        is ExistsUniqueGroup -> {
            val existsVars =
                node.existsUniqueSection.identifiers.map { getVarsPhase2Node(node = it) }.flatten()
            for (v in existsVars) {
                if (vars.hasConflict(v)) {
                    errors.add(
                        ParseError(
                            message = "Duplicate defined symbol '$v' in `existsUnique:`",
                            row = location.row,
                            column = location.column))
                }
                vars.add(v)
            }
            varsToRemove.addAll(existsVars)
        }
        is Statement -> {
            varsToRemove.addAll(checkVarsImplStatement(topLevel, node, vars, tracker, errors))
        }
    }

    if (node is HasUsingSection && node.usingSection != null) {
        // a `using:` section cannot reference other symbols
        // defined in a group.  However, if `x := y`, for
        // example, is defined in a statement, then the
        // statements in the `using:` section after `x := y`
        // can reference the symbol `x`.
        val usingVars = VarMultiSet()
        if (node is DefinesGroup) {
            usingVars.addAll(getVarsPhase2Node(node.id))
            for (target in node.definesSection.targets) {
                usingVars.addAll(getVarsPhase2Node(target))
            }
            if (node.requiringSection != null) {
                for (target in node.requiringSection.targets) {
                    usingVars.addAll(getVarsPhase2Node(target))
                }
            }
            if (node.whenSection != null) {
                usingVars.addAll(getVarsIntroducedInWhenSection(node.whenSection))
            }
        } else if (node is StatesGroup) {
            usingVars.addAll(getVarsPhase2Node(node.id))
            if (node.requiringSection != null) {
                for (target in node.requiringSection.targets) {
                    usingVars.addAll(getVarsPhase2Node(target))
                }
            }
            if (node.whenSection != null) {
                usingVars.addAll(getVarsIntroducedInWhenSection(node.whenSection))
            }
        } else if (node is TheoremGroup) {
            if (node.givenSection != null) {
                for (target in node.givenSection.targets) {
                    usingVars.addAll(getVarsPhase2Node(target))
                }
            }
        } else if (node is ConjectureGroup) {
            if (node.givenSection != null) {
                for (target in node.givenSection.targets) {
                    usingVars.addAll(getVarsPhase2Node(target))
                }
            }
        } else if (node is AxiomGroup) {
            if (node.givenSection != null) {
                for (target in node.givenSection.targets) {
                    usingVars.addAll(getVarsPhase2Node(target))
                }
            }
        }

        for (clause in node.usingSection!!.clauses.clauses) {
            if (clause is Statement &&
                clause.texTalkRoot is ValidationSuccess &&
                clause.texTalkRoot.value.children.size == 1 &&
                clause.texTalkRoot.value.children[0] is ColonEqualsTexTalkNode) {
                val colonEquals = clause.texTalkRoot.value.children[0] as ColonEqualsTexTalkNode
                val thisValidVars = usingVars.copy()
                for (v in
                    getVarsTexTalkNode(
                        texTalkNode = colonEquals.lhs,
                        isInLhsColonEquals = true,
                        groupScope = GroupScope.InNone,
                        isInIdStatement = false)) {
                    thisValidVars.add(v)
                    if (!v.isPlaceholder) {
                        usingVars.add(v)
                    }
                }
                val clauseLocation = tracker.getLocationOf(clause) ?: Location(-1, -1)
                for (v in
                    checkVarsImplTexTalkNode(
                        topLevel = topLevel,
                        texTalkNode = colonEquals.rhs,
                        location = clauseLocation,
                        vars = thisValidVars,
                        errors = errors,
                        isInLhsColonEquals = false,
                        groupScope = GroupScope.InNone)) {
                    usingVars.add(v)
                }
            } else {
                for (v in checkVarsImplPhase2Node(topLevel, clause, usingVars, tracker, errors)) {
                    usingVars.add(v)
                }
            }
        }

        node.forEach {
            for (v in checkVarsImplPhase2Node(topLevel, it, usingVars, tracker, errors)) {
                usingVars.add(v)
            }
        }
    } else {
        node.forEach { checkVarsImplPhase2Node(topLevel, it, vars, tracker, errors) }
        for (v in varsToRemove.toList()) {
            vars.remove(v)
        }
    }

    return varsToRemove.toList()
}

private fun getVarsIntroducedInWhenSection(whenSection: WhenSection): List<Var> {
    val result = mutableListOf<Var>()
    for (clause in whenSection.clauses.clauses) {
        if (clause is Statement) {
            result.addAll(getLeftHandSideVars(clause))
        }
    }
    return result
}

private fun getLeftHandSideVars(statement: Statement): List<Var> {
    val result = mutableListOf<Var>()
    if (statement.texTalkRoot is ValidationSuccess) {
        val exp = statement.texTalkRoot.value
        if (exp.children.size == 1 && exp.children[0] is ColonEqualsTexTalkNode) {
            val colonEquals = exp.children[0] as ColonEqualsTexTalkNode
            result.addAll(
                getVarsTexTalkNode(
                    texTalkNode = colonEquals.lhs,
                    isInLhsColonEquals = true,
                    groupScope = GroupScope.InNone,
                    isInIdStatement = false))
        }
    }
    return result
}

private fun checkWhenSectionVars(
    node: WhenSection, vars: VarMultiSet, tracker: LocationTracker, errors: MutableList<ParseError>
): List<Var> {
    val whenVars = VarMultiSet()
    for (clause in node.clauses.clauses) {
        whenVars.addAll(
            checkColonOrColonColonEqualsRhsSymbols(
                clause, tracker = tracker, vars = vars, errors = errors))
    }
    return whenVars.toList()
}

private fun checkColonOrColonColonEqualsRhsSymbols(
    node: Phase2Node, tracker: LocationTracker, vars: VarMultiSet, errors: MutableList<ParseError>
): List<Var> {
    val result = VarMultiSet()
    if (node is Statement) {
        result.addAll(
            checkColonOrColonColonEqualsRhsSymbols(
                statement = node, tracker = tracker, vars = vars, errors = errors))
    }
    node.forEach { checkColonOrColonColonEqualsRhsSymbols(it, tracker, vars, errors) }
    return result.toList()
}

private fun checkColonOrColonColonEqualsRhsSymbols(
    statement: Statement,
    tracker: LocationTracker,
    vars: VarMultiSet,
    errors: MutableList<ParseError>
): List<Var> {
    val result = VarMultiSet()
    val validation = statement.texTalkRoot
    if (validation is ValidationSuccess) {
        val location = tracker.getLocationOf(statement) ?: Location(-1, -1)
        result.addAll(
            checkColonOrColonColonEqualsRhsSymbols(
                node = validation.value, location = location, vars = vars, errors = errors))
    }
    return result.toList()
}

private fun checkColonOrColonColonEqualsRhsSymbols(
    node: TexTalkNode, location: Location, vars: VarMultiSet, errors: MutableList<ParseError>
): List<Var> {
    val result = VarMultiSet()
    val params =
        if (node is ColonEqualsTexTalkNode) {
            node.rhs
        } else if (node is ColonColonEqualsTexTalkNode) {
            node.rhs
        } else {
            null
        }
    if (params != null) {
        for (v in
            getVarsTexTalkNode(
                texTalkNode = params,
                isInLhsColonEquals = false,
                groupScope = GroupScope.InNone,
                isInIdStatement = false)) {
            if (!vars.contains(v)) {
                errors.add(
                    ParseError(
                        message = "Undefined symbol '$v' in `:=`",
                        row = location.row,
                        column = location.column))
            }
        }
    }
    node.forEach {
        result.addAll(checkColonOrColonColonEqualsRhsSymbols(it, location, vars, errors))
    }
    return result.toList()
}

private fun checkDefineSectionVars(
    topLevel: TopLevelGroup,
    node: DefinesSection,
    vars: VarMultiSet,
    tracker: LocationTracker,
    errors: MutableList<ParseError>
): List<Var> {
    val location =
        tracker.getLocationOf(node) ?: tracker.getLocationOf(topLevel) ?: Location(-1, -1)
    val givenVars = node.targets.map { getVarsPhase2Node(node = it) }.flatten()
    for (v in givenVars) {
        if (vars.hasConflict(v)) {
            errors.add(
                ParseError(
                    message = "Duplicate defined symbol '$v' in `Defines:`",
                    row = location.row,
                    column = location.column))
        }
        vars.add(v)
    }
    return givenVars
}

private fun checkRequiringSectionVars(
    topLevel: TopLevelGroup,
    node: RequiringSection,
    vars: VarMultiSet,
    tracker: LocationTracker,
    errors: MutableList<ParseError>
): List<Var> {
    val location =
        tracker.getLocationOf(node) ?: tracker.getLocationOf(topLevel) ?: Location(-1, -1)
    val givenVars = node.targets.map { getVarsPhase2Node(node = it) }.flatten()
    for (v in givenVars) {
        if (vars.hasConflict(v)) {
            errors.add(
                ParseError(
                    message = "Duplicate defined symbol '$v' in `requiring:`",
                    row = location.row,
                    column = location.column))
        }
        vars.add(v)
    }
    return givenVars
}

private fun checkGivenSectionVars(
    topLevel: TopLevelGroup,
    node: GivenSection,
    vars: VarMultiSet,
    tracker: LocationTracker,
    errors: MutableList<ParseError>
): List<Var> {
    val location =
        tracker.getLocationOf(node) ?: tracker.getLocationOf(topLevel) ?: Location(-1, -1)
    val givenVars = node.targets.map { getVarsPhase2Node(node = it) }.flatten()
    for (v in givenVars) {
        if (vars.hasConflict(v)) {
            errors.add(
                ParseError(
                    message = "Duplicate defined symbol '$v' in `given:`",
                    row = location.row,
                    column = location.column))
        }
        vars.add(v)
    }
    return givenVars
}

private fun checkVarsImplStatement(
    topLevel: TopLevelGroup,
    statement: Statement,
    vars: VarMultiSet,
    tracker: LocationTracker,
    errors: MutableList<ParseError>
): List<Var> {
    return if (statement.texTalkRoot is ValidationSuccess) {
        val location = tracker.getLocationOf(statement) ?: Location(-1, -1)
        // ensure statements like 'f(x, y) := x + y' do not use
        // undefined symbols.  In this case, the left-hand side of
        // := introduces new symbols for the right hand-side
        val root = statement.texTalkRoot.value
        if (root.children.size == 1 && root.children[0] is ColonEqualsTexTalkNode) {
            val colonEquals = root.children[0] as ColonEqualsTexTalkNode
            val names = mutableListOf<Var>()
            val definedSymbols = mutableListOf<Var>()
            val child = colonEquals.lhs.items.getOrNull(0)?.children?.getOrNull(0)
            if (child != null) {
                if (child is OperatorTexTalkNode && child.command is TextTexTalkNode) {
                    val cmd = child.command.text
                    definedSymbols.add(Var(name = cmd, isPlaceholder = false))
                } else if (child is GroupTexTalkNode &&
                    child.parameters.items.isNotEmpty() &&
                    child.parameters.items[0].children.isNotEmpty() &&
                    child.parameters.items[0].children[0] is TextTexTalkNode) {
                    val text = (child.parameters.items[0].children[0] as TextTexTalkNode).text
                    definedSymbols.add(Var(name = text, isPlaceholder = true))
                } else if (child is TextTexTalkNode) {
                    val name = child.text
                    definedSymbols.add(Var(name = name, isPlaceholder = true))
                }

                // only process the left hand side if its of the form
                // x + y := ...
                // x := ...
                // f(x) := ...
                if (child is OperatorTexTalkNode) {
                    if (child.lhs != null) {
                        names.addAll(
                            getVarsTexTalkNode(
                                texTalkNode = child.lhs,
                                isInLhsColonEquals = true,
                                groupScope = GroupScope.InParen,
                                isInIdStatement = false))
                    }

                    if (child.rhs != null) {
                        names.addAll(
                            getVarsTexTalkNode(
                                texTalkNode = child.rhs,
                                isInLhsColonEquals = true,
                                groupScope = GroupScope.InParen,
                                isInIdStatement = false))
                    }
                } else if (child is GroupTexTalkNode) {
                    names.addAll(
                        getVarsTexTalkNode(
                            texTalkNode = child.parameters,
                            isInLhsColonEquals = true,
                            groupScope =
                                when (child.type) {
                                    TexTalkNodeType.CurlyGroup -> {
                                        GroupScope.InCurly
                                    }
                                    TexTalkNodeType.ParenGroup -> {
                                        GroupScope.InParen
                                    }
                                    TexTalkNodeType.SquareGroup -> {
                                        GroupScope.InSquare
                                    }
                                    else -> {
                                        GroupScope.InNone
                                    }
                                },
                            isInIdStatement = false))
                } else if (child is TextTexTalkNode) {
                    names.add(Var(name = child.text, isPlaceholder = true))
                } else {
                    // handle cases like `x \op/ y` and `\f(x)`
                    names.addAll(
                        getVarsTexTalkNode(
                            texTalkNode = child,
                            isInLhsColonEquals = true,
                            groupScope = GroupScope.InNone,
                            isInIdStatement = false))
                }
            }

            for (sym in definedSymbols) {
                vars.add(sym)
            }

            val varsCopy = vars.copy()
            for (n in names) {
                varsCopy.add(n)
            }

            val subFound =
                checkVarsImplTexTalkNode(
                    topLevel = topLevel,
                    texTalkNode = colonEquals.rhs,
                    location = location,
                    vars = varsCopy,
                    errors = errors,
                    isInLhsColonEquals = false,
                    groupScope = GroupScope.InNone)
            val result = mutableListOf<Var>()
            result.addAll(definedSymbols)
            result.addAll(subFound)
            result
        } else {
            // otherwise there aren't any additional symbols to add before
            // checking for the use of undefined symbols
            checkVarsImplTexTalkNode(
                topLevel = topLevel,
                texTalkNode = statement.texTalkRoot.value,
                location = location,
                vars = vars,
                errors = errors,
                isInLhsColonEquals = false,
                groupScope = GroupScope.InNone)
        }
    } else {
        emptyList()
    }
}

private fun isNumberLiteral(text: String) = Regex("[+-]?\\d+(\\.\\d+)?").matchEntire(text) != null

private fun checkVarsImplTexTalkNode(
    topLevel: TopLevelGroup,
    texTalkNode: TexTalkNode,
    location: Location,
    vars: VarMultiSet,
    errors: MutableList<ParseError>,
    isInLhsColonEquals: Boolean,
    groupScope: GroupScope
): List<Var> {
    val varsToRemove = VarMultiSet()
    if (texTalkNode is ColonEqualsTexTalkNode) {
        checkVarsImplTexTalkNode(
            topLevel = topLevel,
            texTalkNode = texTalkNode.lhs,
            location = location,
            vars = vars,
            errors = errors,
            isInLhsColonEquals = true,
            groupScope = groupScope)
        checkVarsImplTexTalkNode(
            topLevel = topLevel,
            texTalkNode = texTalkNode.rhs,
            location = location,
            vars = vars,
            errors = errors,
            isInLhsColonEquals = false,
            groupScope = groupScope)
    } else if (texTalkNode is TextTexTalkNode) {
        val name = texTalkNode.text
        // Note: operators are treated as signatures, not symbols
        // val v = Var(
        //     name = name,
        //     isPlaceholder = groupScope == GroupScope.InSquare || (isInLhsColonEquals &&
        // groupScope != GroupScope.InNone)
        // )
        if (name != "=" &&
            !isNumberLiteral(name) &&
            !vars.contains(Var(name = name, isPlaceholder = true)) &&
            !vars.contains(Var(name = name, isPlaceholder = false)) &&
            !isOperatorName(name)) {
            errors.add(
                ParseError(
                    message = "Undefined symbol '$name'",
                    row = location.row,
                    column = location.column))
        }
    } else if (texTalkNode is Command) {
        for (part in texTalkNode.parts) {
            // process the square braces first because any symbols
            // specified there are then available in the other groups
            if (part.square != null) {
                val squareVars =
                    getVarsTexTalkNode(
                        texTalkNode = part.square,
                        isInLhsColonEquals = isInLhsColonEquals,
                        groupScope = GroupScope.InSquare,
                        isInIdStatement = false)
                varsToRemove.addAll(squareVars)
                for (v in squareVars) {
                    if (vars.hasConflict(v)) {
                        errors.add(
                            ParseError(
                                message = "Duplicate defined symbol '$v' within square parens",
                                row = location.row,
                                column = location.column))
                    }
                    vars.add(v)
                }
                checkVarsImplTexTalkNode(
                    topLevel = topLevel,
                    texTalkNode = part.square,
                    location = location,
                    vars = vars,
                    errors = errors,
                    isInLhsColonEquals = isInLhsColonEquals,
                    groupScope = GroupScope.InSquare)
            }
            for (grp in part.groups) {
                checkVarsImplTexTalkNode(
                    topLevel = topLevel,
                    texTalkNode = grp,
                    location = location,
                    vars = vars,
                    errors = errors,
                    isInLhsColonEquals = isInLhsColonEquals,
                    groupScope =
                        when (grp.type) {
                            TexTalkNodeType.SquareGroup -> {
                                GroupScope.InSquare
                            }
                            TexTalkNodeType.CurlyGroup -> {
                                GroupScope.InCurly
                            }
                            TexTalkNodeType.ParenGroup -> {
                                GroupScope.InParen
                            }
                            else -> {
                                GroupScope.InNone
                            }
                        })
            }
            for (grp in part.namedGroups) {
                grp.groups.forEach {
                    checkVarsImplTexTalkNode(
                        topLevel = topLevel,
                        texTalkNode = it,
                        location = location,
                        vars = vars,
                        errors = errors,
                        isInLhsColonEquals = isInLhsColonEquals,
                        groupScope =
                            when (grp.type) {
                                TexTalkNodeType.SquareGroup -> {
                                    GroupScope.InSquare
                                }
                                TexTalkNodeType.CurlyGroup -> {
                                    GroupScope.InCurly
                                }
                                TexTalkNodeType.ParenGroup -> {
                                    GroupScope.InParen
                                }
                                else -> {
                                    GroupScope.InNone
                                }
                            })
                }
            }
            if (part.paren != null) {
                checkVarsImplTexTalkNode(
                    topLevel = topLevel,
                    texTalkNode = part.paren,
                    location = location,
                    vars = vars,
                    errors = errors,
                    isInLhsColonEquals = isInLhsColonEquals,
                    groupScope = GroupScope.InParen)
            }
            if (part.subSup != null) {
                checkVarsImplTexTalkNode(
                    topLevel = topLevel,
                    texTalkNode = part.subSup,
                    location = location,
                    vars = vars,
                    errors = errors,
                    isInLhsColonEquals = isInLhsColonEquals,
                    groupScope = groupScope)
            }
        }
    } else {
        texTalkNode.forEach {
            checkVarsImplTexTalkNode(
                topLevel = topLevel,
                texTalkNode = it,
                location = location,
                vars = vars,
                errors = errors,
                isInLhsColonEquals = isInLhsColonEquals,
                groupScope = groupScope)
        }
    }
    for (v in varsToRemove.toList()) {
        vars.remove(v)
    }
    return varsToRemove.toList()
}

private fun checkVarsImplIdStatement(
    topLevel: TopLevelGroup,
    id: IdStatement,
    vars: VarMultiSet,
    tracker: LocationTracker,
    errors: MutableList<ParseError>
): List<Var> {
    return if (id.texTalkRoot is ValidationSuccess) {
        val location =
            tracker.getLocationOf(id) ?: tracker.getLocationOf(topLevel) ?: Location(-1, -1)
        // Do not add variables in parens because they will be added
        // in the description in a Defines section
        //
        // That is consider:
        //
        // [\some.function(x)]
        // Defines: f(x)
        // ...
        //
        // Then the `x` in `f(x)` will be marked as a variable.  If the `x`
        // in `\some.function(x)` is also marked as a variable, then the `x`
        // in `f(x)` will be found to be a duplicate defined variable.  Thus,
        // skip variables in parens in ids (in \some.function(x)) in this
        // example.
        val idVars =
            getVarsTexTalkNode(
                texTalkNode = id.texTalkRoot.value,
                isInLhsColonEquals = false,
                groupScope = GroupScope.InNone,
                isInIdStatement = true)
        for (v in idVars) {
            if (vars.hasConflict(v)) {
                errors.add(
                    ParseError(
                        message = "Duplicate defined symbol '$v' in `[...]`",
                        row = location.row,
                        column = location.column))
            }
            vars.add(v)
        }
        idVars
    } else {
        emptyList()
    }
}

private class VarMultiSet : MutableMultiSet<Var> {
    private var placeholders = newMutableMultiSet<Var>()
    private var nonPlaceholders = newMutableMultiSet<Var>()

    override fun add(value: Var) {
        if (value.isPlaceholder) {
            placeholders.add(value)
        } else {
            nonPlaceholders.add(value)
        }
    }

    override fun addAll(values: Collection<Var>) {
        for (v in values) {
            add(v)
        }
    }

    override fun remove(value: Var) {
        if (value.isPlaceholder) {
            placeholders.remove(value)
        } else {
            nonPlaceholders.remove(value)
        }
    }

    override fun contains(key: Var): Boolean {
        return if (key.isPlaceholder) {
            placeholders.contains(key)
        } else {
            nonPlaceholders.contains(key)
        }
    }

    override fun toList(): List<Var> {
        val result = mutableListOf<Var>()
        result.addAll(placeholders.toList())
        result.addAll(nonPlaceholders.toList())
        return result
    }

    override fun isEmpty(): Boolean {
        return placeholders.isEmpty() && nonPlaceholders.isEmpty()
    }

    override fun toSet(): Set<Var> {
        val result = mutableSetOf<Var>()
        result.addAll(placeholders.toSet())
        result.addAll(nonPlaceholders.toSet())
        return result
    }

    override fun copy(): VarMultiSet {
        val copy = VarMultiSet()
        copy.placeholders = this.placeholders.copy()
        copy.nonPlaceholders = this.nonPlaceholders.copy()
        return copy
    }

    override fun toString(): String {
        return "{\n  placeholders: $placeholders\n  nonplaceholders: $nonPlaceholders}"
    }

    private fun hasPlaceholderWithName(v: Var) =
        placeholders.contains(Var(name = v.name, isPlaceholder = true))

    private fun hasNonPlaceholderWithName(v: Var) =
        nonPlaceholders.contains(Var(name = v.name, isPlaceholder = false))

    fun hasConflict(v: Var): Boolean {
        // two variables conflict if they have the same name and
        // at least one is a non-placeholder var

        return if (v.isPlaceholder) {
            // if v is a placeholder it can only conflict with another var if
            // there is a non-placeholder var with the same name as v
            hasNonPlaceholderWithName(v)
        } else {
            // if v is not a placeholder var then its name cannot overlap
            // with any placeholder vars or any other non-placeholder vars
            hasPlaceholderWithName(v) || hasNonPlaceholderWithName(v)
        }
    }
}

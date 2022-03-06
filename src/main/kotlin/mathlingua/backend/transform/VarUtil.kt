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
import mathlingua.frontend.chalktalk.phase1.ast.Abstraction
import mathlingua.frontend.chalktalk.phase1.ast.AbstractionPart
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Token
import mathlingua.frontend.chalktalk.phase1.ast.isOperatorName
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
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.generated.GeneratedGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.HasUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.WhereTargetSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.equality.EqualityGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.ConjectureGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.GivenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.textalk.ColonEqualsTexTalkNode
import mathlingua.frontend.textalk.Command
import mathlingua.frontend.textalk.CommandPart
import mathlingua.frontend.textalk.GroupTexTalkNode
import mathlingua.frontend.textalk.InTexTalkNode
import mathlingua.frontend.textalk.IsTexTalkNode
import mathlingua.frontend.textalk.OperatorTexTalkNode
import mathlingua.frontend.textalk.ParametersTexTalkNode
import mathlingua.frontend.textalk.SequenceNode
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

internal fun Phase2Node.getVarsPhase2Node(): List<Var> {
    val vars = mutableListOf<Var>()
    getVarsImplPhase2Node(this, vars)
    return vars
}

internal fun Phase1Node.getVarsPhase1Node(isInPlaceholderScope: Boolean): List<Var> {
    val vars = mutableListOf<Var>()
    getVarsImplPhase1Node(this, vars, isInPlaceholderScope)
    return vars
}

internal fun TexTalkNode.getVarsTexTalkNode(
    isInLhsOfColonEqualsIsOrIn: Boolean,
    groupScope: GroupScope,
    isInIdStatement: Boolean,
    forceIsPlaceholder: Boolean
): List<Var> {
    val vars = mutableListOf<Var>()
    getVarsImplTexTalkNode(
        this, vars, isInLhsOfColonEqualsIsOrIn, groupScope, isInIdStatement, forceIsPlaceholder)
    return vars
}

internal fun TexTalkNode.renameVarsTexTalkNode(map: Map<String, String>) =
    this.transform {
        if (it is TextTexTalkNode) {
            it.copy(text = map[it.text] ?: it.text)
        } else {
            it
        }
    }

internal fun TopLevelGroup.checkVarsPhase2Node(node: Phase2Node): Set<ParseError> {
    val errors = mutableListOf<ParseError>()
    val vars = VarMultiSet()
    checkVarsImplPhase2Node(this, node, vars, errors)
    return errors.toSet()
}

// -----------------------------------------------------------------------------

private fun getVarsImplPhase1Node(
    node: Phase1Node, vars: MutableList<Var>, isInPlaceholderScope: Boolean
) {
    if (node is Phase1Token) {
        // If the variable is variadic, store the non-variadic version of the name.
        // That is if it is `x...` store `x`.
        vars.add(Var(name = node.text.removeSuffix("..."), isPlaceholder = isInPlaceholderScope))
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
                    isInIdStatement = false,
                    forceIsPlaceholder = false)
            }
            else -> {
                // if the parsing fails, then the vars cannot be determined
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
    isInIdStatement: Boolean,
    forceIsPlaceholder: Boolean
) {
    if (texTalkNode is TextTexTalkNode) {
        if (texTalkNode.tokenType != TexTalkTokenType.ColonEquals) {
            vars.add(
                Var(
                    name = texTalkNode.text.removeSuffix("..."),
                    isPlaceholder =
                        forceIsPlaceholder ||
                            groupScope == GroupScope.InSquare ||
                            (groupScope == GroupScope.InParen && isInIdStatement) ||
                            (groupScope != GroupScope.InNone && isInLhsColonEquals)))
        }
    } else if (texTalkNode is OperatorTexTalkNode) {
        if (texTalkNode.lhs != null) {
            getVarsImplTexTalkNode(
                texTalkNode.lhs,
                vars,
                isInLhsColonEquals,
                GroupScope.InParen,
                isInIdStatement,
                forceIsPlaceholder)
        }

        if (texTalkNode.rhs != null) {
            getVarsImplTexTalkNode(
                texTalkNode.rhs,
                vars,
                isInLhsColonEquals,
                GroupScope.InParen,
                isInIdStatement,
                forceIsPlaceholder)
        }

        // if the texTalkNode is an operator of the form `x * y` where the `*` is a
        // TextTexTalkNode that is an operator then don't add `*` as a variable because
        // it should be treated as a signature
        if (texTalkNode.command !is TextTexTalkNode || !isOperatorName(texTalkNode.command.text)) {
            getVarsImplTexTalkNode(
                texTalkNode.command,
                vars,
                isInLhsColonEquals,
                groupScope,
                isInIdStatement,
                forceIsPlaceholder)
        }
    } else if (texTalkNode is ColonEqualsTexTalkNode) {
        getVarsImplTexTalkNode(
            texTalkNode.lhs, vars, true, groupScope, isInIdStatement, forceIsPlaceholder)
        getVarsImplTexTalkNode(
            texTalkNode.rhs, vars, false, groupScope, isInIdStatement, forceIsPlaceholder)
    } else if (texTalkNode is IsTexTalkNode) {
        getVarsImplTexTalkNode(
            texTalkNode.lhs, vars, true, groupScope, isInIdStatement, forceIsPlaceholder)
        getVarsImplTexTalkNode(
            texTalkNode.rhs, vars, false, groupScope, isInIdStatement, forceIsPlaceholder)
    } else if (texTalkNode is InTexTalkNode) {
        getVarsImplTexTalkNode(
            texTalkNode.lhs, vars, true, groupScope, isInIdStatement, forceIsPlaceholder)
        getVarsImplTexTalkNode(
            texTalkNode.rhs, vars, false, groupScope, isInIdStatement, forceIsPlaceholder)
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
            isInIdStatement,
            forceIsPlaceholder)
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
                isInIdStatement,
                forceIsPlaceholder)
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
                    isInIdStatement,
                    forceIsPlaceholder)
            }
        }
        if (texTalkNode.paren != null) {
            getVarsImplTexTalkNode(
                texTalkNode.paren,
                vars,
                isInLhsColonEquals,
                GroupScope.InParen,
                isInIdStatement,
                forceIsPlaceholder)
        }
        if (texTalkNode.square != null) {
            getVarsImplTexTalkNode(
                texTalkNode.square,
                vars,
                isInLhsColonEquals,
                GroupScope.InSquare,
                isInIdStatement,
                forceIsPlaceholder)
        }
        if (texTalkNode.subSup != null) {
            getVarsImplTexTalkNode(
                texTalkNode.subSup,
                vars,
                isInLhsColonEquals,
                groupScope,
                isInIdStatement,
                forceIsPlaceholder)
        }
    } else if (texTalkNode is ParametersTexTalkNode) {
        texTalkNode.forEach {
            getVarsImplTexTalkNode(
                it, vars, isInLhsColonEquals, groupScope, isInIdStatement, forceIsPlaceholder)
        }
    } else if (texTalkNode is SequenceNode) {
        getVarsImplTexTalkNode(
            texTalkNode.mapping.name,
            vars,
            isInLhsColonEquals,
            groupScope,
            isInIdStatement,
            forceIsPlaceholder)
        if (texTalkNode.mapping.parenGroup != null) {
            getVarsImplTexTalkNode(
                texTalkNode.mapping.parenGroup,
                vars,
                isInLhsColonEquals,
                GroupScope.InParen,
                isInIdStatement,
                forceIsPlaceholder = true)
        }
        if (texTalkNode.mapping.subGroup != null) {
            // treat the parameters like they are in squares to force them to be considered
            // placeholders
            getVarsImplTexTalkNode(
                texTalkNode.mapping.subGroup,
                vars,
                isInLhsColonEquals,
                GroupScope.InSquare,
                isInIdStatement,
                forceIsPlaceholder = true)
        }
        // treat the parameters like they are in squares to force them to be considered placeholders
        getVarsImplTexTalkNode(
            texTalkNode.subGroup,
            vars,
            isInLhsColonEquals,
            GroupScope.InSquare,
            isInIdStatement,
            forceIsPlaceholder = true)
    } else {
        texTalkNode.forEach {
            getVarsImplTexTalkNode(
                it, vars, isInLhsColonEquals, groupScope, isInIdStatement, forceIsPlaceholder)
        }
    }
}

// -----------------------------------------------------------------------------

private fun checkVarsImplPhase2Node(
    topLevel: TopLevelGroup, node: Phase2Node, vars: VarMultiSet, errors: MutableList<ParseError>
): List<Var> {
    val varsToRemove = VarMultiSet()
    val location = Location(node.row, node.column)

    if (node is DefinesGroup) {
        varsToRemove.addAll(checkVarsImplIdStatement(node.id, vars, errors))
        varsToRemove.addAll(checkDefineSectionVars(node.definesSection, vars, errors))
        if (node.whereSection != null) {
            varsToRemove.addAll(
                checkWhereTargetSectionVars(node.whereSection, node.definesSection, vars, errors))
        }
    }

    val givenSection =
        when (node) {
            is DefinesGroup -> node.givenSection
            is StatesGroup -> node.givenSection
            is TheoremGroup -> node.givenSection
            is AxiomGroup -> node.givenSection
            is ConjectureGroup -> node.givenSection
            else -> null
        }

    if (givenSection != null) {
        varsToRemove.addAll(checkGivenSectionVars(givenSection, vars, errors))
    }

    val whenSection =
        when (node) {
            is DefinesGroup -> node.whenSection
            is StatesGroup -> node.whenSection
            is AxiomGroup -> node.whenSection
            is ConjectureGroup -> node.whenSection
            is TheoremGroup -> node.whenSection
            else -> null
        }

    if (whenSection != null) {
        varsToRemove.addAll(checkWhenSectionVars(node = whenSection, vars = vars, errors = errors))
    }

    // verify the `using:` section clauses are of the correct form and add the left-hand-side
    // symbols to the known symbols in `vars`
    if (node is HasUsingSection && node.usingSection != null) {
        for (clause in node.usingSection!!.clauses.clauses) {
            val clauseLocation = Location(clause.row, clause.column)
            if (clause !is Statement) {
                errors.add(
                    ParseError(
                        message = "A using section can only contain statements",
                        row = clauseLocation.row,
                        column = clauseLocation.column))
                continue
            }

            val lhsVars = getLeftHandSideVars(clause)
            for (v in lhsVars) {
                if (vars.hasConflict(v)) {
                    errors.add(
                        ParseError(
                            message = "Duplicate defined symbol '${v.name}' in `using:`",
                            row = clauseLocation.row,
                            column = clauseLocation.column))
                }
                vars.add(v)
                varsToRemove.add(v)
            }

            val validation = clause.texTalkRoot
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
            }
        }
    }

    when (node) {
        is StatesGroup -> {
            varsToRemove.addAll(checkVarsImplIdStatement(node.id, vars, errors))
        }
        is ForAllGroup -> {
            val forAllVars = node.forAllSection.targets.map { it.getVarsPhase2Node() }.flatten()
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
        is GeneratedGroup -> {
            val fromVars = node.generatedFromSection.forms.map { it.getVarsPhase2Node() }.flatten()
            for (v in fromVars) {
                if (vars.hasConflict(v)) {
                    errors.add(
                        ParseError(
                            message = "Duplicate defined symbol '$v' in `from:`",
                            row = location.row,
                            column = location.column))
                }
                vars.add(v)
            }
            varsToRemove.addAll(fromVars)
        }
        is EqualityGroup -> {
            val betweenVars = node.betweenSection.targets.map { it.getVarsPhase2Node() }.flatten()
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
            val existsVars = node.existsSection.identifiers.map { it.getVarsPhase2Node() }.flatten()
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
                node.existsUniqueSection.identifiers.map { it.getVarsPhase2Node() }.flatten()
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
            varsToRemove.addAll(checkVarsImplStatement(topLevel, node, vars, errors))
        }
    }

    node.forEach { checkVarsImplPhase2Node(topLevel, it, vars, errors) }
    for (v in varsToRemove.toList()) {
        vars.remove(v)
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
                colonEquals.lhs.getVarsTexTalkNode(
                    isInLhsOfColonEqualsIsOrIn = true,
                    groupScope = GroupScope.InNone,
                    isInIdStatement = false,
                    forceIsPlaceholder = false))
        }
    }
    return result
}

private fun checkWhenSectionVars(
    node: WhenSection, vars: VarMultiSet, errors: MutableList<ParseError>
): List<Var> {
    val whenVars = VarMultiSet()
    for (clause in node.clauses.clauses) {
        // if a clause is a forAll:, exists:, or existsUnique: group
        // then make sure `vars` is updated to include the introduced symbols
        val clauseVars =
            when (clause) {
                is ForAllGroup -> clause.forAllSection.getVarsPhase2Node()
                is ExistsGroup -> clause.existsSection.getVarsPhase2Node()
                is ExistsUniqueGroup -> clause.existsUniqueSection.getVarsPhase2Node()
                else -> emptyList()
            }
        vars.addAll(clauseVars)
        whenVars.addAll(checkColonEqualsRhsSymbols(clause, vars = vars, errors = errors))
        vars.removeAll(clauseVars)
    }
    return whenVars.toList()
}

private fun checkColonEqualsRhsSymbols(
    node: Phase2Node, vars: VarMultiSet, errors: MutableList<ParseError>
): List<Var> {
    val result = VarMultiSet()
    if (node is Statement) {
        result.addAll(checkColonEqualsRhsSymbols(statement = node, vars = vars, errors = errors))
    }
    node.forEach { checkColonEqualsRhsSymbols(it, vars, errors) }
    return result.toList()
}

private fun checkColonEqualsRhsSymbols(
    statement: Statement, vars: VarMultiSet, errors: MutableList<ParseError>
): List<Var> {
    val result = VarMultiSet()
    val validation = statement.texTalkRoot
    if (validation is ValidationSuccess) {
        val location = Location(statement.row, statement.column)
        result.addAll(
            checkColonEqualsRhsSymbols(
                node = validation.value, location = location, vars = vars, errors = errors))
    }
    return result.toList()
}

private fun checkColonEqualsRhsSymbols(
    node: TexTalkNode, location: Location, vars: VarMultiSet, errors: MutableList<ParseError>
): List<Var> {
    val result = VarMultiSet()
    if (node is ColonEqualsTexTalkNode) {
        for (v in
            node.rhs.getVarsTexTalkNode(
                isInLhsOfColonEqualsIsOrIn = false,
                groupScope = GroupScope.InNone,
                isInIdStatement = false,
                forceIsPlaceholder = false)) {
            if (!vars.contains(v) && !isNumberLiteral(v.name)) {
                errors.add(
                    ParseError(
                        message = "Undefined symbol '$v' in `:=`",
                        row = location.row,
                        column = location.column))
            }
        }
    }
    node.forEach { result.addAll(checkColonEqualsRhsSymbols(it, location, vars, errors)) }
    return result.toList()
}

private fun checkDefineSectionVars(
    node: DefinesSection, vars: VarMultiSet, errors: MutableList<ParseError>
): List<Var> {
    val location = Location(node.row, node.column)
    val givenVars = node.targets.map { it.getVarsPhase2Node() }.flatten().toMutableList()
    /*
     * If the target is of the form G := (X, *, e)
     * then add G.X, G.*, and G.e as vars introduced.
     */
    if (node.targets.isNotEmpty() && node.targets.first() is AssignmentNode) {
        val assign = (node.targets.first() as AssignmentNode).assignment
        val left = assign.lhs.text
        for (right in assign.rhs.getVarsPhase1Node(isInPlaceholderScope = false)) {
            if (!right.isPlaceholder) {
                givenVars.add(Var(name = "$left.${right.name}", isPlaceholder = false))
            }
        }
    }
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

private fun checkWhereTargetSectionVars(
    node: WhereTargetSection,
    definesSection: DefinesSection,
    vars: VarMultiSet,
    errors: MutableList<ParseError>
): List<Var> {
    val location = Location(node.row, node.column)
    // all var names in the Defines: section
    val defVarNames =
        definesSection
            .targets
            .map { it.getVarsPhase2Node() }
            .flatten()
            .filter { !it.isPlaceholder }
            .map { it.name }
    // var names in the left-hand-side of := statements in where: section that
    // also var names in the Defines: section
    val lhsWhereNames =
        node.targets
            .filterIsInstance<AssignmentNode>()
            .map { it.assignment.lhs.text }
            .filter { defVarNames.contains(it) }
            .toSet()

    // the new variables introduced in the where: section
    val newWhereVars =
        node.targets
            .map { it.getVarsPhase2Node() }
            .flatten()
            .filter { !lhsWhereNames.contains(it.name) }
            .toMutableList()
    /*
     * If the target is of the form G := (X, *, e)
     * then add G.X, G.*, and G.e as vars introduced.
     */
    if (node.targets.isNotEmpty() && node.targets.first() is AssignmentNode) {
        val assign = (node.targets.first() as AssignmentNode).assignment
        val left = assign.lhs.text
        for (right in assign.rhs.getVarsPhase1Node(isInPlaceholderScope = false)) {
            if (!right.isPlaceholder) {
                newWhereVars.add(Var(name = "$left.${right.name}", isPlaceholder = false))
            }
        }
    }
    for (v in newWhereVars) {
        if (vars.hasConflict(v)) {
            errors.add(
                ParseError(
                    message = "Duplicate defined symbol '$v' in `where:`",
                    row = location.row,
                    column = location.column))
        }
        vars.add(v)
    }
    return newWhereVars
}

private fun checkGivenSectionVars(
    node: GivenSection, vars: VarMultiSet, errors: MutableList<ParseError>
): List<Var> {
    val location = Location(node.row, node.column)
    val givenVars = node.targets.map { it.getVarsPhase2Node() }.flatten()
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
    errors: MutableList<ParseError>
): List<Var> {
    return if (statement.texTalkRoot is ValidationSuccess) {
        val location = Location(statement.row, statement.column)
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
                            child.lhs.getVarsTexTalkNode(
                                isInLhsOfColonEqualsIsOrIn = true,
                                groupScope = GroupScope.InParen,
                                isInIdStatement = false,
                                forceIsPlaceholder = false))
                    }

                    if (child.rhs != null) {
                        names.addAll(
                            child.rhs.getVarsTexTalkNode(
                                isInLhsOfColonEqualsIsOrIn = true,
                                groupScope = GroupScope.InParen,
                                isInIdStatement = false,
                                forceIsPlaceholder = false))
                    }
                } else if (child is GroupTexTalkNode) {
                    names.addAll(
                        child.parameters.getVarsTexTalkNode(
                            isInLhsOfColonEqualsIsOrIn = true,
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
                            isInIdStatement = false,
                            forceIsPlaceholder = false))
                } else if (child is TextTexTalkNode) {
                    names.add(Var(name = child.text, isPlaceholder = true))
                } else {
                    // handle cases like `x \op/ y` and `\f(x)`
                    names.addAll(
                        child.getVarsTexTalkNode(
                            isInLhsOfColonEqualsIsOrIn = true,
                            groupScope = GroupScope.InNone,
                            isInIdStatement = false,
                            forceIsPlaceholder = false))
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
                    isInLhsOfColonEqualsIsOrIn = false,
                    groupScope = GroupScope.InNone)
            val result = mutableListOf<Var>()
            result.addAll(definedSymbols)
            result.addAll(subFound)
            result
        } else if (root.children.size == 1 && root.children[0] is IsTexTalkNode) {
            val isNode = root.children[0] as IsTexTalkNode
            val toRemove = mutableListOf<Var>()
            for (v in
                isNode.lhs.getVarsTexTalkNode(
                    isInLhsOfColonEqualsIsOrIn = true,
                    groupScope = GroupScope.InNone,
                    isInIdStatement = false,
                    forceIsPlaceholder = false)) {
                if (v.isPlaceholder) {
                    vars.add(v)
                    toRemove.add(v)
                }
            }
            toRemove.addAll(
                checkVarsImplTexTalkNode(
                    topLevel = topLevel,
                    texTalkNode = root,
                    location = location,
                    vars = vars,
                    errors = errors,
                    isInLhsOfColonEqualsIsOrIn = false,
                    groupScope = GroupScope.InNone))
            toRemove
        } else if (root.children.size == 1 && root.children[0] is InTexTalkNode) {
            val isNode = root.children[0] as InTexTalkNode
            val toRemove = mutableListOf<Var>()
            for (v in
                isNode.lhs.getVarsTexTalkNode(
                    isInLhsOfColonEqualsIsOrIn = true,
                    groupScope = GroupScope.InNone,
                    isInIdStatement = false,
                    forceIsPlaceholder = false)) {
                if (v.isPlaceholder) {
                    vars.add(v)
                    toRemove.add(v)
                }
            }
            toRemove.addAll(
                checkVarsImplTexTalkNode(
                    topLevel = topLevel,
                    texTalkNode = root,
                    location = location,
                    vars = vars,
                    errors = errors,
                    isInLhsOfColonEqualsIsOrIn = false,
                    groupScope = GroupScope.InNone))
            toRemove
        } else {
            // otherwise there aren't any additional symbols to add before
            // checking for the use of undefined symbols
            checkVarsImplTexTalkNode(
                topLevel = topLevel,
                texTalkNode = statement.texTalkRoot.value,
                location = location,
                vars = vars,
                errors = errors,
                isInLhsOfColonEqualsIsOrIn = false,
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
    isInLhsOfColonEqualsIsOrIn: Boolean,
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
            isInLhsOfColonEqualsIsOrIn = true,
            groupScope = groupScope)
        checkVarsImplTexTalkNode(
            topLevel = topLevel,
            texTalkNode = texTalkNode.rhs,
            location = location,
            vars = vars,
            errors = errors,
            isInLhsOfColonEqualsIsOrIn = false,
            groupScope = groupScope)
    } else if (texTalkNode is IsTexTalkNode) {
        checkVarsImplTexTalkNode(
            topLevel = topLevel,
            texTalkNode = texTalkNode.lhs,
            location = location,
            vars = vars,
            errors = errors,
            isInLhsOfColonEqualsIsOrIn = true,
            groupScope = groupScope)
        checkVarsImplTexTalkNode(
            topLevel = topLevel,
            texTalkNode = texTalkNode.rhs,
            location = location,
            vars = vars,
            errors = errors,
            isInLhsOfColonEqualsIsOrIn = false,
            groupScope = groupScope)
    } else if (texTalkNode is InTexTalkNode) {
        checkVarsImplTexTalkNode(
            topLevel = topLevel,
            texTalkNode = texTalkNode.lhs,
            location = location,
            vars = vars,
            errors = errors,
            isInLhsOfColonEqualsIsOrIn = true,
            groupScope = groupScope)
        checkVarsImplTexTalkNode(
            topLevel = topLevel,
            texTalkNode = texTalkNode.rhs,
            location = location,
            vars = vars,
            errors = errors,
            isInLhsOfColonEqualsIsOrIn = false,
            groupScope = groupScope)
    } else if (texTalkNode is TextTexTalkNode) {
        val name = texTalkNode.text
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
                    part.square.getVarsTexTalkNode(
                        isInLhsOfColonEqualsIsOrIn = isInLhsOfColonEqualsIsOrIn,
                        groupScope = GroupScope.InSquare,
                        isInIdStatement = false,
                        forceIsPlaceholder = false)
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
                    isInLhsOfColonEqualsIsOrIn = isInLhsOfColonEqualsIsOrIn,
                    groupScope = GroupScope.InSquare)
            }
            for (grp in part.groups) {
                checkVarsImplTexTalkNode(
                    topLevel = topLevel,
                    texTalkNode = grp,
                    location = location,
                    vars = vars,
                    errors = errors,
                    isInLhsOfColonEqualsIsOrIn = isInLhsOfColonEqualsIsOrIn,
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
                        isInLhsOfColonEqualsIsOrIn = isInLhsOfColonEqualsIsOrIn,
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
                    isInLhsOfColonEqualsIsOrIn = isInLhsOfColonEqualsIsOrIn,
                    groupScope = GroupScope.InParen)
            }
            if (part.subSup != null) {
                checkVarsImplTexTalkNode(
                    topLevel = topLevel,
                    texTalkNode = part.subSup,
                    location = location,
                    vars = vars,
                    errors = errors,
                    isInLhsOfColonEqualsIsOrIn = isInLhsOfColonEqualsIsOrIn,
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
                isInLhsOfColonEqualsIsOrIn = isInLhsOfColonEqualsIsOrIn,
                groupScope = groupScope)
        }
    }
    for (v in varsToRemove.toList()) {
        vars.remove(v)
    }
    return varsToRemove.toList()
}

private fun checkVarsImplIdStatement(
    id: IdStatement, vars: VarMultiSet, errors: MutableList<ParseError>
): List<Var> {
    return if (id.texTalkRoot is ValidationSuccess) {
        val location = Location(id.row, id.column)
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
            id.texTalkRoot.value.getVarsTexTalkNode(
                isInLhsOfColonEqualsIsOrIn = false,
                groupScope = GroupScope.InNone,
                isInIdStatement = true,
                forceIsPlaceholder = false)
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

    override fun removeAll(values: Collection<Var>) {
        for (v in values) {
            remove(v)
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

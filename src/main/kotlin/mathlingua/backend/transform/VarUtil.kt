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

import mathlingua.backend.isOperatorName
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Token
import mathlingua.frontend.chalktalk.phase2.ast.clause.AbstractionNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.AssignmentNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Identifier
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Text
import mathlingua.frontend.chalktalk.phase2.ast.clause.TupleNode
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.exists.ExistsGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.forAll.ForAllGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.given.GivenGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.HasUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.RequiringSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.evaluates.EvaluatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.equality.EqualityGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.ConjectureGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.GivenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.LocationTracker
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.support.validationSuccess
import mathlingua.frontend.textalk.ColonColonEqualsTexTalkNode
import mathlingua.frontend.textalk.ColonEqualsTexTalkNode
import mathlingua.frontend.textalk.Command
import mathlingua.frontend.textalk.CommandPart
import mathlingua.frontend.textalk.ExpressionTexTalkNode
import mathlingua.frontend.textalk.GroupTexTalkNode
import mathlingua.frontend.textalk.OperatorTexTalkNode
import mathlingua.frontend.textalk.ParametersTexTalkNode
import mathlingua.frontend.textalk.TexTalkNode
import mathlingua.frontend.textalk.TexTalkTokenType
import mathlingua.frontend.textalk.TextTexTalkNode

internal fun getVars(node: Phase1Node): List<String> {
    val vars = mutableListOf<String>()
    getVarsImpl(node, vars)
    return vars
}

internal fun getVars(node: Phase2Node, ignoreParen: Boolean): List<String> {
    val vars = mutableListOf<String>()
    getVarsImpl(node, vars, ignoreParen)
    return vars
}

internal fun getVars(texTalkNode: TexTalkNode, ignoreParens: Boolean): List<String> {
    val vars = mutableListOf<String>()
    getVarsImpl(texTalkNode, vars, ignoreParens)
    return vars
}

internal fun renameVars(texTalkNode: TexTalkNode, map: Map<String, String>) =
    texTalkNode.transform {
        if (it is TextTexTalkNode) {
            it.copy(text = map[it.text] ?: it.text)
        } else {
            it
        }
    }

internal fun renameVars(root: Phase2Node, map: Map<String, String>): Phase2Node {
    fun chalkTransformer(node: Phase2Node): Phase2Node {
        if (node is Identifier) {
            return Identifier(name = map[node.name] ?: node.name, isVarArgs = node.isVarArgs)
        }

        if (node is Statement) {
            return when (val validation = node.texTalkRoot
            ) {
                is ValidationSuccess -> {
                    val exp = renameVars(validation.value, map) as ExpressionTexTalkNode
                    return Statement(text = exp.toCode(), texTalkRoot = validationSuccess(exp))
                }
                is ValidationFailure -> node
            }
        } else if (node is Text) {
            var newText = node.text
            val keysLongToShort = map.keys.toList().sortedBy { it.length }.reversed()
            for (key in keysLongToShort) {
                newText = newText.replace("$key&", map[key]!!)
            }
            return Text(text = newText)
        }

        return node
    }

    return root.transform(::chalkTransformer)
}

internal fun checkVars(node: Phase2Node, tracker: LocationTracker): Set<ParseError> {
    val errors = mutableListOf<ParseError>()
    checkVarsImpl(node, MultiSet(), tracker, errors, ignoreParen = false)
    return errors.toSet()
}

// -----------------------------------------------------------------------------

private fun getVarsImpl(node: Phase1Node, vars: MutableList<String>) {
    if (node is Phase1Token) {
        vars.add(node.text.removeSuffix("..."))
    } else {
        node.forEach { getVarsImpl(it, vars) }
    }
}

private fun getVarsImpl(node: Phase2Node, vars: MutableList<String>, ignoreParen: Boolean) {
    if (node is Identifier) {
        vars.add(node.name.removeSuffix("..."))
    } else if (node is TupleNode) {
        getVarsImpl(node.tuple, vars)
    } else if (node is AbstractionNode) {
        getVarsImpl(node.abstraction, vars)
    } else if (node is AssignmentNode) {
        vars.add(node.assignment.lhs.text.removeSuffix("..."))
        getVarsImpl(node.assignment.rhs, vars)
    } else if (node is Statement) {
        when (node.texTalkRoot) {
            is ValidationSuccess -> {
                getVarsImpl(node.texTalkRoot.value, vars, ignoreParen)
            }
        }
    } else {
        node.forEach { getVarsImpl(it, vars, ignoreParen) }
    }
}

private fun getVarsImpl(texTalkNode: TexTalkNode, vars: MutableList<String>, ignoreParen: Boolean) {
    if (texTalkNode is TextTexTalkNode) {
        if (texTalkNode.tokenType != TexTalkTokenType.ColonEquals &&
            texTalkNode.tokenType != TexTalkTokenType.ColonColonEquals) {
            vars.add(texTalkNode.text.removeSuffix("..."))
        }
    } else if (texTalkNode is CommandPart) {
        for (grp in texTalkNode.groups) {
            getVarsImpl(grp, vars, ignoreParen)
        }
        for (grp in texTalkNode.namedGroups) {
            grp.groups.forEach { getVarsImpl(it, vars, ignoreParen) }
        }
        if (texTalkNode.paren != null && !ignoreParen) {
            getVarsImpl(texTalkNode.paren, vars, ignoreParen)
        }
        if (texTalkNode.square != null) {
            getVarsImpl(texTalkNode.square, vars, ignoreParen)
        }
        if (texTalkNode.subSup != null) {
            getVarsImpl(texTalkNode.subSup, vars, ignoreParen)
        }
    } else if (texTalkNode is ParametersTexTalkNode) {
        texTalkNode.forEach { getVarsImpl(it, vars, ignoreParen) }
    } else {
        texTalkNode.forEach { getVarsImpl(it, vars, ignoreParen) }
    }
}

// -----------------------------------------------------------------------------

private fun checkVarsImpl(
    node: Phase2Node,
    vars: MultiSet<String>,
    tracker: LocationTracker,
    errors: MutableList<ParseError>,
    ignoreParen: Boolean
): List<String> {
    val varsToRemove = MultiSet<String>()
    val location = tracker.getLocationOf(node) ?: Location(-1, -1)
    if (node is DefinesGroup) {
        val whenSection = node.whenSection
        if (whenSection != null) {
            varsToRemove.addAll(
                checkWhenSectionVars(
                    node = whenSection, vars = vars, tracker = tracker, errors = errors))
        }
        varsToRemove.addAll(checkVarsImpl(node.id, vars, tracker, errors))
        varsToRemove.addAll(
            checkDefineSectionVars(node.definesSection, vars, tracker, errors, ignoreParen))
        if (node.requiringSection != null) {
            varsToRemove.addAll(
                checkRequiringSectionVars(
                    node.requiringSection!!, vars, tracker, errors, ignoreParen))
        }
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

                val op = root.children[0] as ColonEqualsTexTalkNode
                val params = op.lhs
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
                        if (child is OperatorTexTalkNode && child.command is TextTexTalkNode) {
                            val cmd = child.command.text
                            vars.add(cmd)
                            varsToRemove.add(cmd)
                        } else if (child is GroupTexTalkNode &&
                            child.parameters.items.isNotEmpty() &&
                            child.parameters.items[0].children.isNotEmpty() &&
                            child.parameters.items[0].children[0] is TextTexTalkNode) {
                            val text =
                                (child.parameters.items[0].children[0] as TextTexTalkNode).text
                            vars.add(text)
                            varsToRemove.add(text)
                        } else if (child is TextTexTalkNode) {
                            val name = child.text
                            vars.add(name)
                            varsToRemove.add(name)
                        }
                    }
                }
            }
        }
    }

    when (node) {
        is EvaluatesGroup -> {
            varsToRemove.addAll(checkVarsImpl(node.id, vars, tracker, errors))
        }
        is StatesGroup -> {
            varsToRemove.addAll(checkVarsImpl(node.id, vars, tracker, errors))
        }
        is TheoremGroup -> {
            if (node.givenSection != null) {
                varsToRemove.addAll(
                    checkGivenSectionVars(node.givenSection, vars, tracker, errors, ignoreParen))
            }
        }
        is AxiomGroup -> {
            if (node.givenSection != null) {
                varsToRemove.addAll(
                    checkGivenSectionVars(node.givenSection, vars, tracker, errors, ignoreParen))
            }
        }
        is ConjectureGroup -> {
            if (node.givenSection != null) {
                varsToRemove.addAll(
                    checkGivenSectionVars(node.givenSection, vars, tracker, errors, ignoreParen))
            }
        }
        is ForAllGroup -> {
            val forAllVars = node.forAllSection.targets.map { getVars(it, ignoreParen) }.flatten()
            for (v in forAllVars) {
                if (vars.contains(v)) {
                    errors.add(
                        ParseError(
                            message = "Duplicate defined symbol '$v'",
                            row = location.row,
                            column = location.column))
                }
                vars.add(v)
            }
            varsToRemove.addAll(forAllVars)
        }
        is EqualityGroup -> {
            val betweenVars = node.betweenSection.targets.map { getVars(it, ignoreParen) }.flatten()
            for (v in betweenVars) {
                if (vars.contains(v)) {
                    errors.add(
                        ParseError(
                            message = "Duplicate defined symbol '$v'",
                            row = location.row,
                            column = location.column))
                }
                vars.add(v)
            }
            varsToRemove.addAll(betweenVars)
        }
        is ExistsGroup -> {
            val existsVars =
                node.existsSection.identifiers.map { getVars(it, ignoreParen) }.flatten()
            for (v in existsVars) {
                if (vars.contains(v)) {
                    errors.add(
                        ParseError(
                            message = "Duplicate defined symbol '$v'",
                            row = location.row,
                            column = location.column))
                }
                vars.add(v)
            }
            varsToRemove.addAll(existsVars)
        }
        is GivenGroup -> {
            val givenVars = node.givenSection.targets.map { getVars(it, ignoreParen) }.flatten()
            for (v in givenVars) {
                if (vars.contains(v)) {
                    errors.add(
                        ParseError(
                            message = "Duplicate defined symbol '$v'",
                            row = location.row,
                            column = location.column))
                }
                vars.add(v)
            }
            varsToRemove.addAll(givenVars)
        }
        is Statement -> {
            varsToRemove.addAll(checkVarsImpl(node, vars, tracker, errors, ignoreParen))
        }
    }

    if (node is UsingSection) {
        // a `using:` section cannot reference other symbols
        // defined in a group.  However, if `x := y`, for
        // example, is defined in a statement, then the
        // statements in the `using:` section after `x := y`
        // can reference the symbol `x`.
        val usingVars = MultiSet<String>()
        node.forEach {
            for (v in checkVarsImpl(it, usingVars, tracker, errors, ignoreParen)) {
                usingVars.add(v)
            }
        }
    } else {
        node.forEach { checkVarsImpl(it, vars, tracker, errors, ignoreParen) }
        for (v in varsToRemove.toList()) {
            vars.remove(v)
        }
    }

    return varsToRemove.toList()
}

private fun checkWhenSectionVars(
    node: WhenSection,
    vars: MultiSet<String>,
    tracker: LocationTracker,
    errors: MutableList<ParseError>
): List<String> {
    val whenVars = mutableListOf<String>()
    for (clause in node.clauses.clauses) {
        whenVars.addAll(
            checkColonOrColonColonEqualsRhsSymbols(
                clause, tracker = tracker, vars = vars, errors = errors))
    }
    return whenVars
}

private fun checkColonOrColonColonEqualsRhsSymbols(
    node: Phase2Node,
    tracker: LocationTracker,
    vars: MultiSet<String>,
    errors: MutableList<ParseError>
): List<String> {
    val result = mutableListOf<String>()
    if (node is Statement) {
        result.addAll(
            checkColonOrColonColonEqualsRhsSymbols(
                statement = node, tracker = tracker, vars = vars, errors = errors))
    }
    node.forEach { checkColonOrColonColonEqualsRhsSymbols(it, tracker, vars, errors) }
    return result
}

private fun checkColonOrColonColonEqualsRhsSymbols(
    statement: Statement,
    tracker: LocationTracker,
    vars: MultiSet<String>,
    errors: MutableList<ParseError>
): List<String> {
    val result = mutableListOf<String>()
    val validation = statement.texTalkRoot
    if (validation is ValidationSuccess) {
        val location = tracker.getLocationOf(statement) ?: Location(-1, -1)
        result.addAll(
            checkColonOrColonColonEqualsRhsSymbols(
                node = validation.value, location = location, vars = vars, errors = errors))
    }
    return result
}

private fun checkColonOrColonColonEqualsRhsSymbols(
    node: TexTalkNode, location: Location, vars: MultiSet<String>, errors: MutableList<ParseError>
): List<String> {
    val result = mutableListOf<String>()
    val params =
        if (node is ColonEqualsTexTalkNode) {
            node.rhs
        } else if (node is ColonColonEqualsTexTalkNode) {
            node.rhs
        } else {
            null
        }
    if (params != null) {
        for (v in getVars(params, false)) {
            if (vars.contains(v)) {
                errors.add(
                    ParseError(
                        message = "Duplicate defined symbol '$v'",
                        row = location.row,
                        column = location.column))
            }
            vars.add(v)
        }
    }
    node.forEach {
        result.addAll(checkColonOrColonColonEqualsRhsSymbols(it, location, vars, errors))
    }
    return result
}

private fun checkDefineSectionVars(
    node: DefinesSection,
    vars: MultiSet<String>,
    tracker: LocationTracker,
    errors: MutableList<ParseError>,
    ignoreParen: Boolean
): List<String> {
    val location = tracker.getLocationOf(node) ?: Location(-1, -1)
    val givenVars = node.targets.map { getVars(it, ignoreParen) }.flatten()
    for (v in givenVars) {
        if (vars.contains(v)) {
            errors.add(
                ParseError(
                    message = "Duplicate defined symbol '$v'",
                    row = location.row,
                    column = location.column))
        }
        vars.add(v)
    }
    return givenVars
}

private fun checkRequiringSectionVars(
    node: RequiringSection,
    vars: MultiSet<String>,
    tracker: LocationTracker,
    errors: MutableList<ParseError>,
    ignoreParen: Boolean
): List<String> {
    val location = tracker.getLocationOf(node) ?: Location(-1, -1)
    val givenVars = node.targets.map { getVars(it, ignoreParen) }.flatten()
    for (v in givenVars) {
        if (vars.contains(v)) {
            errors.add(
                ParseError(
                    message = "Duplicate defined symbol '$v'",
                    row = location.row,
                    column = location.column))
        }
        vars.add(v)
    }
    return givenVars
}

private fun checkGivenSectionVars(
    node: GivenSection,
    vars: MultiSet<String>,
    tracker: LocationTracker,
    errors: MutableList<ParseError>,
    ignoreParen: Boolean
): List<String> {
    val location = tracker.getLocationOf(node) ?: Location(-1, -1)
    val givenVars = node.targets.map { getVars(it, ignoreParen) }.flatten()
    for (v in givenVars) {
        if (vars.contains(v)) {
            errors.add(
                ParseError(
                    message = "Duplicate defined symbol '$v'",
                    row = location.row,
                    column = location.column))
        }
        vars.add(v)
    }
    return givenVars
}

private fun checkVarsImpl(
    statement: Statement,
    vars: MultiSet<String>,
    tracker: LocationTracker,
    errors: MutableList<ParseError>,
    ignoreParen: Boolean
): List<String> {
    return if (statement.texTalkRoot is ValidationSuccess) {
        val location = tracker.getLocationOf(statement) ?: Location(-1, -1)
        // ensure statements like 'f(x, y) := x + y' do not use
        // undefined symbols.  In this case, the left-hand side of
        // := introduces new symbols for the right hand-side
        val root = statement.texTalkRoot.value
        if (root.children.size == 1 && root.children[0] is ColonEqualsTexTalkNode) {
            val colonEquals = root.children[0] as ColonEqualsTexTalkNode
            val names = mutableListOf<String>()
            val definedSymbols = mutableListOf<String>()
            val child = colonEquals.lhs.items.getOrNull(0)?.children?.getOrNull(0)
            if (child != null) {
                if (child is OperatorTexTalkNode && child.command is TextTexTalkNode) {
                    val cmd = child.command.text
                    definedSymbols.add(cmd)
                } else if (child is GroupTexTalkNode &&
                    child.parameters.items.isNotEmpty() &&
                    child.parameters.items[0].children.isNotEmpty() &&
                    child.parameters.items[0].children[0] is TextTexTalkNode) {
                    val text = (child.parameters.items[0].children[0] as TextTexTalkNode).text
                    definedSymbols.add(text)
                } else if (child is TextTexTalkNode) {
                    val name = child.text
                    definedSymbols.add(name)
                }

                // only process the left hand side if its of the form
                // x + y := ...
                // x := ...
                // f(x) := ...
                if (child is OperatorTexTalkNode) {
                    if (child.lhs != null) {
                        names.addAll(getVars(child.lhs, ignoreParen))
                    }

                    if (child.rhs != null) {
                        names.addAll(getVars(child.rhs, ignoreParen))
                    }
                } else if (child is GroupTexTalkNode) {
                    names.addAll(getVars(child.parameters, ignoreParen))
                } else if (child is TextTexTalkNode) {
                    names.add(child.text)
                }
            }

            for (sym in definedSymbols) {
                vars.add(sym)
            }

            val varsCopy = MultiSet.copy(vars)
            for (n in names) {
                varsCopy.add(n)
            }

            val subFound = checkVarsImpl(colonEquals.rhs, location, varsCopy, errors, ignoreParen)
            val result = mutableListOf<String>()
            result.addAll(definedSymbols)
            result.addAll(subFound)
            result
        } else {
            // otherwise there aren't any additional symbols to add before
            // checking for the use of undefined symbols
            checkVarsImpl(statement.texTalkRoot.value, location, vars, errors, ignoreParen)
        }
    } else {
        emptyList()
    }
}

private fun isNumberLiteral(text: String) = Regex("[+-]?\\d+(\\.\\d+)?").matchEntire(text) != null

private fun checkVarsImpl(
    texTalkNode: TexTalkNode,
    location: Location,
    vars: MultiSet<String>,
    errors: MutableList<ParseError>,
    ignoreParen: Boolean
): List<String> {
    val varsToRemove = MultiSet<String>()
    if (texTalkNode is TextTexTalkNode) {
        val name = texTalkNode.text
        // Note: operators are treated as signatures, not symbols
        if (name != "=" &&
            !isNumberLiteral(name) &&
            !vars.contains(name) &&
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
                val squareVars = getVars(part.square, ignoreParen)
                varsToRemove.addAll(squareVars)
                for (v in squareVars) {
                    if (vars.contains(v)) {
                        errors.add(
                            ParseError(
                                message = "Duplicate defined symbol '$v'",
                                row = location.row,
                                column = location.column))
                    }
                    vars.add(v)
                }
                checkVarsImpl(part.square, location, vars, errors, ignoreParen)
            }
            for (grp in part.groups) {
                checkVarsImpl(grp, location, vars, errors, ignoreParen)
            }
            for (grp in part.namedGroups) {
                grp.groups.forEach { checkVarsImpl(it, location, vars, errors, ignoreParen) }
            }
            if (part.paren != null) {
                checkVarsImpl(part.paren, location, vars, errors, ignoreParen)
            }
            if (part.subSup != null) {
                checkVarsImpl(part.subSup, location, vars, errors, ignoreParen)
            }
        }
    } else {
        texTalkNode.forEach { checkVarsImpl(it, location, vars, errors, ignoreParen) }
    }
    for (v in varsToRemove.toList()) {
        vars.remove(v)
    }
    return varsToRemove.toList()
}

private fun checkVarsImpl(
    id: IdStatement,
    vars: MultiSet<String>,
    tracker: LocationTracker,
    errors: MutableList<ParseError>
): List<String> {
    return if (id.texTalkRoot is ValidationSuccess) {
        val location = tracker.getLocationOf(id) ?: Location(-1, -1)
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
        val idVars = getVars(id.texTalkRoot.value, ignoreParens = true)
        for (v in idVars) {
            if (vars.contains(v)) {
                errors.add(
                    ParseError(
                        message = "Duplicate defined symbol '$v'",
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

class MultiSet<T> {
    private val data = mutableMapOf<T, Int>()

    fun add(value: T) {
        data[value] = 1 + data.getOrDefault(value, 0)
    }

    fun addAll(values: Collection<T>) {
        for (v in values) {
            add(v)
        }
    }

    fun remove(value: T) {
        if (data.containsKey(value)) {
            val newCount = data[value]!! - 1
            data[value] = newCount
            if (newCount <= 0) {
                data.remove(value)
            }
        }
    }

    fun contains(key: T) = data.getOrDefault(key, 0) > 0

    fun toList() = data.keys.toList()

    companion object {
        fun <T> copy(set: MultiSet<T>): MultiSet<T> {
            val copy = MultiSet<T>()
            for (entry in set.data.entries) {
                copy.data[entry.key] = entry.value
            }
            return copy
        }
    }
}

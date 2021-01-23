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
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.evaluates.EvaluatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.views.ViewsGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.ConjectureGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.GivenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremGroup
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.LocationTracker
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.support.validationSuccess
import mathlingua.frontend.textalk.Command
import mathlingua.frontend.textalk.CommandPart
import mathlingua.frontend.textalk.ExpressionTexTalkNode
import mathlingua.frontend.textalk.ParametersTexTalkNode
import mathlingua.frontend.textalk.TexTalkNode
import mathlingua.frontend.textalk.TextTexTalkNode

internal fun getVars(node: Phase1Node): List<String> {
    val vars = mutableListOf<String>()
    getVarsImpl(node, vars)
    return vars
}

internal fun getVars(node: Phase2Node): List<String> {
    val vars = mutableListOf<String>()
    getVarsImpl(node, vars)
    return vars
}

internal fun getVars(texTalkNode: TexTalkNode): List<String> {
    val vars = mutableListOf<String>()
    getVarsImpl(texTalkNode, vars, false)
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
    checkVarsImpl(node, mutableSetOf(), tracker, errors)
    return errors.toSet()
}

// -----------------------------------------------------------------------------

private fun getVarsImpl(node: Phase1Node, vars: MutableList<String>) {
    if (node is Phase1Token) {
        vars.add(node.text)
    } else {
        node.forEach { getVarsImpl(it, vars) }
    }
}

private fun getVarsImpl(node: Phase2Node, vars: MutableList<String>) {
    if (node is Identifier) {
        vars.add(node.name)
    } else if (node is TupleNode) {
        getVarsImpl(node.tuple, vars)
    } else if (node is AbstractionNode) {
        getVarsImpl(node.abstraction, vars)
    } else if (node is AssignmentNode) {
        vars.add(node.assignment.lhs.text)
        getVarsImpl(node.assignment.rhs, vars)
    } else if (node is Statement) {
        when (node.texTalkRoot) {
            is ValidationSuccess -> {
                getVarsImpl(node.texTalkRoot.value, vars)
            }
        }
    } else {
        node.forEach { getVarsImpl(it, vars) }
    }
}

private fun getVarsImpl(texTalkNode: TexTalkNode, vars: MutableList<String>) =
    getVarsImpl(texTalkNode, vars, false)

private fun getVarsImpl(texTalkNode: TexTalkNode, vars: MutableList<String>, inParams: Boolean) {
    if (inParams && texTalkNode is TextTexTalkNode) {
        vars.add(
            texTalkNode.text +
                if (texTalkNode.isVarArg) {
                    "..."
                } else {
                    ""
                })
    } else if (texTalkNode is CommandPart) {
        for (grp in texTalkNode.groups) {
            getVarsImpl(grp, vars, false)
        }
        for (grp in texTalkNode.namedGroups) {
            grp.groups.forEach { getVarsImpl(it, vars, false) }
        }
        if (texTalkNode.paren != null) {
            getVarsImpl(texTalkNode.paren, vars, false)
        }
        if (texTalkNode.square != null) {
            getVarsImpl(texTalkNode.square, vars, false)
        }
        if (texTalkNode.subSup != null) {
            getVarsImpl(texTalkNode.subSup, vars, false)
        }
    } else if (texTalkNode is ParametersTexTalkNode) {
        texTalkNode.forEach { getVarsImpl(it, vars, true) }
    } else {
        texTalkNode.forEach { getVarsImpl(it, vars, inParams) }
    }
}

// -----------------------------------------------------------------------------

private fun checkVarsImpl(
    node: Phase2Node,
    vars: MutableSet<String>,
    tracker: LocationTracker,
    errors: MutableList<ParseError>
) {
    val varsToRemove = mutableSetOf<String>()
    val location = tracker.getLocationOf(node) ?: Location(-1, -1)
    when (node) {
        is EvaluatesGroup -> {
            varsToRemove.addAll(checkVarsImpl(node.id, vars, tracker, errors))
        }
        is ViewsGroup -> {
            varsToRemove.addAll(checkVarsImpl(node.id, vars, tracker, errors))
        }
        is StatesGroup -> {
            varsToRemove.addAll(checkVarsImpl(node.id, vars, tracker, errors))
        }
        is DefinesGroup -> {
            varsToRemove.addAll(checkVarsImpl(node.id, vars, tracker, errors))
            varsToRemove.addAll(checkDefineSectionVars(node.definesSection, vars, tracker, errors))
        }
        is TheoremGroup -> {
            if (node.givenSection != null) {
                varsToRemove.addAll(checkGivenSectionVars(node.givenSection, vars, tracker, errors))
            }
        }
        is AxiomGroup -> {
            if (node.givenSection != null) {
                varsToRemove.addAll(checkGivenSectionVars(node.givenSection, vars, tracker, errors))
            }
        }
        is ConjectureGroup -> {
            if (node.givenSection != null) {
                varsToRemove.addAll(checkGivenSectionVars(node.givenSection, vars, tracker, errors))
            }
        }
        is ForAllGroup -> {
            val forAllVars = node.forAllSection.targets.map { getVars(it) }.flatten()
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
        is ExistsGroup -> {
            val existsVars = node.existsSection.identifiers.map { getVars(it) }.flatten()
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
        is Statement -> {
            checkVarsImpl(node, vars, tracker, errors)
        }
    }

    node.forEach { checkVarsImpl(it, vars, tracker, errors) }
    vars.removeAll(varsToRemove)
}

private fun checkDefineSectionVars(
    node: DefinesSection,
    vars: MutableSet<String>,
    tracker: LocationTracker,
    errors: MutableList<ParseError>
): List<String> {
    val location = tracker.getLocationOf(node) ?: Location(-1, -1)
    val givenVars = node.targets.map { getVars(it) }.flatten()
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
    vars: MutableSet<String>,
    tracker: LocationTracker,
    errors: MutableList<ParseError>
): List<String> {
    val location = tracker.getLocationOf(node) ?: Location(-1, -1)
    val givenVars = node.targets.map { getVars(it) }.flatten()
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
    vars: MutableSet<String>,
    tracker: LocationTracker,
    errors: MutableList<ParseError>
) {
    if (statement.texTalkRoot is ValidationSuccess) {
        val location = tracker.getLocationOf(statement) ?: Location(-1, -1)
        checkVarsImpl(statement.texTalkRoot.value, location, vars, errors)
    }
}

private fun checkVarsImpl(
    texTalkNode: TexTalkNode,
    location: Location,
    vars: MutableSet<String>,
    errors: MutableList<ParseError>
) {
    val varsToRemove = mutableSetOf<String>()
    if (texTalkNode is TextTexTalkNode) {
        val name = texTalkNode.text
        if (name != "=" && !vars.contains(name)) {
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
                val squareVars = getVars(part.square)
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
                checkVarsImpl(part.square, location, vars, errors)
            }
            for (grp in part.groups) {
                checkVarsImpl(grp, location, vars, errors)
            }
            for (grp in part.namedGroups) {
                grp.groups.forEach { checkVarsImpl(it, location, vars, errors) }
            }
            if (part.paren != null) {
                checkVarsImpl(part.paren, location, vars, errors)
            }
            if (part.subSup != null) {
                checkVarsImpl(part.subSup, location, vars, errors)
            }
        }
    } else {
        texTalkNode.forEach { checkVarsImpl(it, location, vars, errors) }
    }
    vars.removeAll(varsToRemove)
}

private fun checkVarsImpl(
    id: IdStatement,
    vars: MutableSet<String>,
    tracker: LocationTracker,
    errors: MutableList<ParseError>
): List<String> {
    return if (id.texTalkRoot is ValidationSuccess) {
        val location = tracker.getLocationOf(id) ?: Location(-1, -1)
        val idVars = getVars(id.texTalkRoot.value)
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

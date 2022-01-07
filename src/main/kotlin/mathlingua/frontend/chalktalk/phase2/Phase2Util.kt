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

package mathlingua.frontend.chalktalk.phase2

import mathlingua.backend.WrittenAsForm
import mathlingua.backend.isOperatorName
import mathlingua.backend.transform.Signature
import mathlingua.backend.transform.signature
import mathlingua.frontend.chalktalk.phase1.ast.Abstraction
import mathlingua.frontend.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Token
import mathlingua.frontend.chalktalk.phase1.ast.Tuple
import mathlingua.frontend.chalktalk.phase2.ast.clause.AbstractionNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.AssignmentNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Identifier
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Target
import mathlingua.frontend.chalktalk.phase2.ast.clause.TupleNode
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resource.ResourceGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.ConjectureGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topic.TopicGroup
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.LocationTracker
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.support.newLocationTracker
import mathlingua.frontend.support.validationSuccess
import mathlingua.frontend.textalk.ColonEqualsTexTalkNode
import mathlingua.frontend.textalk.Command
import mathlingua.frontend.textalk.ExpressionTexTalkNode
import mathlingua.frontend.textalk.GroupTexTalkNode
import mathlingua.frontend.textalk.OperatorTexTalkNode
import mathlingua.frontend.textalk.TexTalkNode
import mathlingua.frontend.textalk.TexTalkTokenType
import mathlingua.frontend.textalk.TextTexTalkNode

internal fun hasChild(node: Phase2Node, child: Phase2Node): Boolean {
    if (node == child) {
        return true
    }

    var found = false
    node.forEach {
        if (!found) {
            found = hasChild(it, child)
        }
    }

    return found
}

internal fun TopLevelGroup.getCalledNames() =
    when (this) {
        is TheoremGroup -> {
            this.theoremSection.names
        }
        is AxiomGroup -> {
            this.axiomSection.names
        }
        is ConjectureGroup -> {
            this.conjectureSection.names
        }
        is DefinesGroup -> {
            this.getCalled()
        }
        is StatesGroup -> {
            this.getCalled()
        }
        is TopicGroup -> {
            this.topicSection.names
        }
        is ResourceGroup -> {
            val result = mutableListOf<String>()
            for (values in this.resourceSection.items.map { it.section.values }) {
                result.addAll(values)
            }
            result
        }
        else -> emptyList()
    }

internal fun getPatternsToWrittenAs(
    defines: List<DefinesGroup>, states: List<StatesGroup>, axioms: List<AxiomGroup>
): Map<OperatorTexTalkNode, WrittenAsForm> {
    val allDefines = mutableListOf<DefinesGroup>()
    allDefines.addAll(defines)

    val allStates = mutableListOf<StatesGroup>()
    allStates.addAll(states)

    val result = mutableMapOf<OperatorTexTalkNode, WrittenAsForm>()
    for (rep in allStates) {
        val writtenAs =
            rep.writtenSection.forms.getOrNull(0)?.removeSurrounding("\"", "\"") ?: continue

        val validation = rep.id.texTalkRoot
        if (validation is ValidationSuccess) {
            val exp = validation.value
            if (exp.children.size == 1 && exp.children[0] is OperatorTexTalkNode) {
                result[exp.children[0] as OperatorTexTalkNode] =
                    WrittenAsForm(target = null, form = writtenAs)
            } else if (exp.children.size == 1 && exp.children[0] is Command) {
                result[
                    OperatorTexTalkNode(
                        lhs = null, command = exp.children[0] as Command, rhs = null)] =
                    WrittenAsForm(target = null, form = writtenAs)
            }
        }
    }

    for (axiom in axioms) {
        val validation = axiom.id?.texTalkRoot
        if (validation is ValidationSuccess) {
            val exp = validation.value
            val name = axiom.axiomSection.names.getOrNull(0)
            val writtenAs =
                if (name != null) {
                    "\\textrm{${name.removeSurrounding("\"", "\"")}}"
                } else {
                    exp.toCode()
                }
            if (exp.children.size == 1 && exp.children[0] is OperatorTexTalkNode) {
                result[exp.children[0] as OperatorTexTalkNode] =
                    WrittenAsForm(target = null, form = writtenAs)
            } else if (exp.children.size == 1 && exp.children[0] is Command) {
                val cmd = exp.children[0] as Command
                result[OperatorTexTalkNode(lhs = null, command = cmd, rhs = null)] =
                    WrittenAsForm(target = null, form = writtenAs)
            }
        }
    }

    for (def in allDefines) {
        val writtenAs =
            def.writtenSection.forms.getOrNull(0)?.removeSurrounding("\"", "\"") ?: continue

        val validation = def.id.texTalkRoot
        val target =
            if (def.definesSection.targets.isNotEmpty()) {
                def.definesSection.targets[0].toCode(false, 0).getCode()
            } else {
                null
            }
        if (validation is ValidationSuccess) {
            val exp = validation.value
            if (exp.children.size == 1 && exp.children[0] is OperatorTexTalkNode) {
                result[exp.children[0] as OperatorTexTalkNode] =
                    WrittenAsForm(target = target, form = writtenAs)
            } else if (exp.children.size == 1 && exp.children[0] is Command) {
                val cmd = exp.children[0] as Command
                result[OperatorTexTalkNode(lhs = null, command = cmd, rhs = null)] =
                    WrittenAsForm(target = target, form = writtenAs)
            }
        }
    }

    return result
}

internal fun getInnerDefinedSignatures(clauses: List<Clause>): Set<String> {
    val result = mutableSetOf<String>()
    for (clause in clauses) {
        if (clause is Statement) {
            when (val validation = clause.texTalkRoot
            ) {
                is ValidationSuccess -> {
                    getInnerDefinedSignaturesImpl(validation.value, false, result)
                }
            }
        }
    }
    return result
}

private fun getInnerDefinedSignaturesImpl(
    node: TexTalkNode, isInColonEqualsRhs: Boolean, result: MutableSet<String>
) {
    if (node is ColonEqualsTexTalkNode) {
        getInnerDefinedSignaturesImpl(
            node = node.lhs, isInColonEqualsRhs = isInColonEqualsRhs, result)
        getInnerDefinedSignaturesImpl(node = node.rhs, isInColonEqualsRhs = true, result)
    } else if (!isInColonEqualsRhs && node is TextTexTalkNode && isOperatorName(node.text)) {
        result.add(node.text)
    } else {
        node.forEach { getInnerDefinedSignaturesImpl(it, isInColonEqualsRhs, result) }
    }
}

// an 'inner' signature is a signature that is only within scope of the given top level group
internal fun getInnerDefinedSignatures(
    group: TopLevelGroup, tracker: LocationTracker?
): Set<Signature> {
    val result = mutableSetOf<Signature>()

    val usingSection =
        when (group) {
            is DefinesGroup -> {
                group.usingSection
            }
            is StatesGroup -> {
                group.usingSection
            }
            is TheoremGroup -> {
                group.usingSection
            }
            is ConjectureGroup -> {
                group.usingSection
            }
            is AxiomGroup -> {
                group.usingSection
            }
            else -> {
                null
            }
        }

    if (usingSection != null) {
        for (clause in usingSection.clauses.clauses) {
            if (clause is Statement) {
                val location = tracker?.getLocationOf(clause) ?: Location(-1, -1)
                when (val validation = clause.texTalkRoot
                ) {
                    is ValidationSuccess -> {
                        for (sigForm in getUsingDefinedSignature(validation.value)) {
                            result.add(Signature(form = sigForm, location = location))
                        }
                    }
                }
            }
        }
    }

    val requiringSection =
        when (group) {
            is DefinesGroup -> group.givenSection
            is StatesGroup -> group.givenSection
            else -> null
        }

    if (requiringSection != null) {
        for (target in requiringSection.targets) {
            result.addAll(
                findOperatorNamesWithin(target).map {
                    Signature(
                        form = it,
                        location = tracker?.getLocationOf(target)
                                ?: Location(row = -1, column = -1))
                })
        }
    }

    if (group is DefinesGroup) {
        if (group.whenSection != null) {
            val location = tracker?.getLocationOf(group.whenSection) ?: Location(-1, -1)
            result.addAll(
                getInnerDefinedSignatures(group.whenSection.clauses.clauses).map {
                    Signature(form = it, location = location)
                })
        }

        result.addAll(getOperatorIdentifiersFromTargets(group.definesSection.targets, tracker))
    } else if (group is StatesGroup) {
        if (group.whenSection != null) {
            val location = tracker?.getLocationOf(group.whenSection) ?: Location(-1, -1)
            result.addAll(
                getInnerDefinedSignatures(group.whenSection.clauses.clauses).map {
                    Signature(form = it, location = location)
                })
        }
    } else if (group is TheoremGroup) {
        if (group.givenSection != null) {
            result.addAll(getOperatorIdentifiersFromTargets(group.givenSection.targets, tracker))
        }
    } else if (group is AxiomGroup) {
        if (group.givenSection != null) {
            result.addAll(getOperatorIdentifiersFromTargets(group.givenSection.targets, tracker))
        }
    } else if (group is ConjectureGroup) {
        if (group.givenSection != null) {
            result.addAll(getOperatorIdentifiersFromTargets(group.givenSection.targets, tracker))
        }
    }
    return result
}

private fun findOperatorNamesWithin(target: Target): List<String> {
    val result = mutableListOf<String>()
    findOperatorNamesWithinImpl(target, result)
    return result
}

private fun findOperatorNamesWithinImpl(node: Phase2Node, result: MutableList<String>) {
    if (node is Identifier && isOperatorName(node.name)) {
        result.add(node.name)
    } else if (node is AssignmentNode) {
        findOperatorNamesWithinImpl(node.assignment, result)
    } else if (node is TupleNode) {
        findOperatorNamesWithinImpl(node.tuple, result)
    } else if (node is AbstractionNode) {
        findOperatorNamesWithinImpl(node.abstraction, result)
    }
    node.forEach { findOperatorNamesWithinImpl(it, result) }
}

private fun findOperatorNamesWithinImpl(node: Phase1Node, result: MutableList<String>) {
    if (node is Phase1Token && isOperatorName(node.text)) {
        result.add(node.text)
    }
    node.forEach { findOperatorNamesWithinImpl(it, result) }
}

private fun getOperatorIdentifiersFromTargets(
    targets: List<Target>, tracker: LocationTracker?
): List<Signature> {
    val result = mutableListOf<Signature>()
    for (target in targets) {
        for (op in getOperatorIdentifiers(target)) {
            result.add(
                Signature(
                    form = op,
                    location = tracker?.getLocationOf(target) ?: Location(row = -1, column = -1)))
        }
    }
    return result
}

private fun getOperatorIdentifiers(node: Phase2Node): Set<String> {
    val result = mutableSetOf<String>()
    getOperatorIdentifiersImpl(node, result)
    return result
}

private fun getOperatorIdentifiersImpl(node: Phase2Node, result: MutableSet<String>) {
    if (node is AbstractionNode) {
        maybeAddAbstractionAsOperatorIdentifier(node.abstraction, result)
    } else if (node is TupleNode) {
        maybeAddTupleAsOperator(node.tuple, result)
    } else if (node is AssignmentNode) {
        val assign = node.assignment
        if (assign.rhs is Abstraction) {
            maybeAddAbstractionAsOperatorIdentifier(assign.rhs, result)
        } else if (assign.rhs is Tuple) {
            maybeAddTupleAsOperator(assign.rhs, result)
        }

        if (assign.lhs.type == ChalkTalkTokenType.Name && isOperatorName(assign.lhs.text)) {
            result.add(assign.lhs.text)
        }
    }
    node.forEach { getOperatorIdentifiersImpl(it, result) }
}

private fun getUsingDefinedSignature(node: ExpressionTexTalkNode): List<String> {
    return if (node.children.size == 1 &&
        node.children[0] is ColonEqualsTexTalkNode &&
        (node.children[0] as ColonEqualsTexTalkNode).lhs.items.size == 1 &&
        (node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children.size == 1 &&
        (node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[0] is GroupTexTalkNode &&
        ((node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[0] as GroupTexTalkNode)
            .parameters
            .items
            .size == 1 &&
        ((node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[0] as GroupTexTalkNode)
                .parameters
                .items[0]
            .children
            .size == 1 &&
        ((node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[0] as GroupTexTalkNode)
                .parameters
                .items[0]
            .children[0] is OperatorTexTalkNode &&
        (((node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[0] as GroupTexTalkNode)
                    .parameters
                    .items[0]
                .children[0] as OperatorTexTalkNode)
            .command is TextTexTalkNode &&
        ((((node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
                            0] as GroupTexTalkNode)
                        .parameters
                        .items[0]
                    .children[0] as OperatorTexTalkNode)
                .command as TextTexTalkNode)
            .tokenType == TexTalkTokenType.Operator) {
        // match -f := ...
        listOf(
            ((((node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
                                0] as GroupTexTalkNode)
                            .parameters
                            .items[0]
                        .children[0] as OperatorTexTalkNode)
                    .command as TextTexTalkNode)
                .text)
    } else if (node.children.isNotEmpty() &&
        node.children[0] is ColonEqualsTexTalkNode &&
        (node.children[0] as ColonEqualsTexTalkNode).lhs.items.isNotEmpty() &&
        (node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children.isNotEmpty() &&
        (node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
            0] is OperatorTexTalkNode &&
        ((node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
                0] as OperatorTexTalkNode)
            .command is TextTexTalkNode) {
        // match `a + b := ...`
        listOf(
            (((node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
                        0] as OperatorTexTalkNode)
                    .command as TextTexTalkNode)
                .text)
    } else if (node.children.size == 1 && node.children[0] is ColonEqualsTexTalkNode) {
        val colonEquals = node.children[0] as ColonEqualsTexTalkNode
        val lhs = colonEquals.lhs.items.firstOrNull()
        if (lhs != null &&
            lhs.children.size == 1 &&
            lhs.children[0] is mathlingua.frontend.textalk.TupleNode) {
            val tuple = lhs.children[0] as mathlingua.frontend.textalk.TupleNode
            tuple.params.items
                .filter { it.children.size == 1 && it.children[0] is TextTexTalkNode }
                .map { (it.children[0] as TextTexTalkNode).text }
        } else if (lhs != null) {
            val id =
                IdStatement(text = lhs.toCode(), texTalkRoot = validationSuccess(lhs))
                    .signature(newLocationTracker())
                    ?.form
            if (id == null) {
                emptyList()
            } else {
                listOf(id)
            }
        } else {
            emptyList()
        }
    } else {
        emptyList()
    }
}

private fun maybeAddAbstractionAsOperatorIdentifier(abs: Abstraction, result: MutableSet<String>) {
    if (!abs.isEnclosed &&
        !abs.isVarArgs &&
        abs.subParams == null &&
        abs.parts.size == 1 &&
        abs.parts[0].params == null &&
        abs.parts[0].subParams == null &&
        abs.parts[0].tail == null &&
        abs.parts[0].name.type == ChalkTalkTokenType.Name &&
        isOperatorName(abs.parts[0].name.text)) {
        // it is of a 'simple' form like *, **, etc.
        result.add(abs.parts[0].name.text)
    }
}

private fun maybeAddTupleAsOperator(tuple: Tuple, result: MutableSet<String>) {
    for (item in tuple.items) {
        if (item is Abstraction) {
            maybeAddAbstractionAsOperatorIdentifier(item, result)
        }
    }
}

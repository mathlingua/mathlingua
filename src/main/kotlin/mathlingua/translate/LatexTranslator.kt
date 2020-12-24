/*
 * Copyright 2020 Dominic Kramer
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *    http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package mathlingua.translate

import mathlingua.MathLingua
import mathlingua.chalktalk.phase2.ast.Document
import mathlingua.chalktalk.phase2.ast.clause.AbstractionNode
import mathlingua.chalktalk.phase2.ast.clause.AssignmentNode
import mathlingua.chalktalk.phase2.ast.clause.Clause
import mathlingua.chalktalk.phase2.ast.clause.ClauseListNode
import mathlingua.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.chalktalk.phase2.ast.clause.Identifier
import mathlingua.chalktalk.phase2.ast.clause.Statement
import mathlingua.chalktalk.phase2.ast.clause.Target
import mathlingua.chalktalk.phase2.ast.clause.Text
import mathlingua.chalktalk.phase2.ast.clause.TupleNode
import mathlingua.chalktalk.phase2.ast.group.clause.If.IfGroup
import mathlingua.chalktalk.phase2.ast.group.clause.If.IfSection
import mathlingua.chalktalk.phase2.ast.group.clause.If.ThenSection
import mathlingua.chalktalk.phase2.ast.group.clause.exists.ExistsGroup
import mathlingua.chalktalk.phase2.ast.group.clause.exists.ExistsSection
import mathlingua.chalktalk.phase2.ast.group.clause.exists.SuchThatSection
import mathlingua.chalktalk.phase2.ast.group.clause.expands.AsSection
import mathlingua.chalktalk.phase2.ast.group.clause.expands.ExpandsGroup
import mathlingua.chalktalk.phase2.ast.group.clause.expands.ExpandsSection
import mathlingua.chalktalk.phase2.ast.group.clause.forAll.ForAllGroup
import mathlingua.chalktalk.phase2.ast.group.clause.forAll.ForAllSection
import mathlingua.chalktalk.phase2.ast.group.clause.iff.IffGroup
import mathlingua.chalktalk.phase2.ast.group.clause.iff.IffSection
import mathlingua.chalktalk.phase2.ast.group.clause.not.NotGroup
import mathlingua.chalktalk.phase2.ast.group.clause.not.NotSection
import mathlingua.chalktalk.phase2.ast.group.clause.or.OrGroup
import mathlingua.chalktalk.phase2.ast.group.clause.or.OrSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.MeansSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.ProvidedSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.FoundationGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.mutually.MutuallyGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.states.ThatSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.resource.ResourceGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resource.ResourceSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.ConjectureGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.ConjectureSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.WhenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.WhereSection
import mathlingua.chalktalk.phase2.ast.section.TextSection
import mathlingua.support.ValidationSuccess

class LatexTranslator(
    val defines: List<DefinesGroup>,
    val represents: List<StatesGroup>,
    val foundations: List<FoundationGroup>,
    val mutuallyGroups: List<MutuallyGroup>
) {
    val buffer = mutableListOf<String>()

    fun translate(doc: Document?) {
        if (doc != null) {
            for (group in doc.groups) {
                translate(group)
                append("\n\n\n")
            }
        }
    }

    fun translate(ifGroup: IfGroup?) {
        translate(ifGroup?.ifSection)
        translate(ifGroup?.thenSection)
    }

    fun translate(iffGroup: IffGroup?) {
        translate(iffGroup?.iffSection)
        translate(iffGroup?.thenSection)
    }

    fun translate(existsGroup: ExistsGroup?) {
        translate(existsGroup?.existsSection)
        translate(existsGroup?.suchThatSection)
    }

    fun translate(forAllGroup: ForAllGroup?) {
        translate(forAllGroup?.forAllSection)
        translate(forAllGroup?.whereSection)
        translate(forAllGroup?.thenSection)
    }

    fun translate(notGroup: NotGroup?) {
        translate(notGroup?.notSection)
    }

    fun translate(orGroup: OrGroup?) {
        translate(orGroup?.orSection)
    }

    fun translate(statement: Statement?) {
        if (statement != null) {
            val text =
                if (statement.texTalkRoot is ValidationSuccess) {
                    MathLingua.expandWrittenAs(
                            statement.texTalkRoot.value,
                            defines,
                            represents,
                            foundations,
                            mutuallyGroups)
                        .text
                } else {
                    statement.text
                }
            append("\\($text\\)")
        }
    }

    fun translate(abstractionNode: AbstractionNode?) {
        if (abstractionNode != null) {
            append(abstractionNode.abstraction.toCode())
        }
    }

    fun translate(assignmentNode: AssignmentNode?) {
        if (assignmentNode != null) {
            append(assignmentNode.toCode(false, 0).getCode())
        }
    }

    fun translate(expandsGroup: ExpandsGroup?) {
        if (expandsGroup != null) {
            translate(expandsGroup.expandsSection)
            translate(expandsGroup.asSection)
        }
    }

    fun translate(identifier: Identifier?) {
        if (identifier != null) {
            append(identifier.name)
        }
    }

    fun translate(tupleNode: TupleNode?) {
        if (tupleNode != null) {
            append(tupleNode.toCode(false, 0).getCode())
        }
    }

    fun translate(theoremGroup: TheoremGroup?) {
        if (theoremGroup != null) {
            append("\\textbf{Theorem}")
            translate(theoremGroup.theoremSection)
        }
    }

    fun translate(axiomGroup: AxiomGroup?) {
        if (axiomGroup != null) {
            append("\\textbf{Axiom}")
            translate(axiomGroup.axiomSection)
        }
    }

    fun translate(conjectureGroup: ConjectureGroup?) {
        if (conjectureGroup != null) {
            append("\\textbf{Conjecture}")
            translate(conjectureGroup.conjectureSection)
        }
    }

    fun translate(definesGroup: DefinesGroup?) {
        /*
        if (definesGroup != null) {
            append("\\textbf{Definition} A")
            val written = definesGroup.writtenSection?.forms?.getOrNull(0)
            if (written != null) {
                append("\\(${written.removeSurrounding("\"", "\"")}\\)")
            } else {
                append("\\verb   '")
                append(definesGroup.id.text)
                append("'")
            }
            translate(definesGroup.definesSection)
            translate(definesGroup.providedSection)
            translate(definesGroup.meansSection)
        }
         */
    }

    fun translate(representsGroup: StatesGroup?) {
        if (representsGroup != null) {
            append(representsGroup.id.text)
            append("represents")
            translate(representsGroup.thatSection)
        }
    }

    fun translate(resourceGroup: ResourceGroup?) {
        if (resourceGroup != null) {
            append("\\textbf{${resourceGroup.id}}\\\\")
            translate(resourceGroup.sourceSection)
        }
    }

    fun translate(topLevelGroup: TopLevelGroup?) {
        when (topLevelGroup) {
            is DefinesGroup -> translate(topLevelGroup)
            is StatesGroup -> translate(topLevelGroup)
            is TheoremGroup -> translate(topLevelGroup)
            is AxiomGroup -> translate(topLevelGroup)
            is ConjectureGroup -> translate(topLevelGroup)
        }
    }

    fun translate(clause: Clause?) {
        when (clause) {
            is AbstractionNode -> translate(clause)
            is AssignmentNode -> translate(clause)
            is ExistsGroup -> translate(clause)
            is ExpandsGroup -> translate(clause)
            is ForAllGroup -> translate(clause)
            is Identifier -> translate(clause)
            is IffGroup -> translate(clause)
            is IfGroup -> translate(clause)
            is NotGroup -> translate(clause)
            is OrGroup -> translate(clause)
            is Statement -> translate(clause)
            is IdStatement -> translate(clause)
            is Text -> translate(clause)
            is TupleNode -> translate(clause)
            else -> throw RuntimeException("Unknown clause ${clause?.toCode(false, 0)?.getCode()}")
        }
    }

    fun translate(idStatement: IdStatement?) {
        translate(idStatement?.toStatement())
    }

    fun translate(text: Text) {
        append(text.text.replace("?", "").removeSurrounding("\"", "\""))
    }

    // TODO: Have this correctly generate LaTeX
    fun translate(theoremSection: TheoremSection?) {
        // if (theoremSection != null) {
        //     translate(theoremSection.clauses)
        // }
    }

    fun translate(asSection: AsSection?) {
        if (asSection != null) {
            append("as")
            translate(asSection.clauses)
        }
    }

    fun translate(whenSection: WhenSection?) {
        if (whenSection != null) {
            append("when")
            translate(whenSection.clauses)
        }
    }

    fun translate(providedSection: ProvidedSection?) {
        if (providedSection != null) {
            append("provided")
            translate(providedSection.clauses)
        }
    }

    // TODO: Have this correctly generate LaTeX
    fun translate(axiomSection: AxiomSection?) {
        // if (axiomSection != null) {
        //     translate(axiomSection.clauses)
        // }
    }

    // TODO: Have this correctly generate LaTeX
    fun translate(conjectureSection: ConjectureSection?) {
        // if (conjectureSection != null) {
        //     translate(conjectureSection.clauses)
        // }
    }

    fun translate(definesSection: DefinesSection?) {
        if (definesSection != null) {
            append("defines")
            translate(definesSection.targets)
        }
    }

    fun translate(existsSection: ExistsSection?) {
        if (existsSection != null) {
            append("there exists")
            translate(existsSection.identifiers)
        }
    }

    fun translate(expandsSection: ExpandsSection?) {
        if (expandsSection != null) {
            append("expands")
            translate(expandsSection.targets)
        }
    }

    fun translate(forAllSection: ForAllSection?) {
        if (forAllSection != null) {
            append("for all")
            translate(forAllSection.targets)
        }
    }

    fun translate(iffSection: IffSection?) {
        if (iffSection != null) {
            append("if and only if")
            translate(iffSection.clauses)
        }
    }

    fun translate(ifSection: IfSection?) {
        if (ifSection != null) {
            append("if")
            translate(ifSection.clauses)
        }
    }

    fun translate(meansSection: MeansSection?) {
        if (meansSection != null) {
            append("and means")
            translate(meansSection.clauses)
        }
    }

    fun translate(thatSection: ThatSection?) {
        if (thatSection != null) {
            translate(thatSection.clauses)
        }
    }

    fun translate(notSection: NotSection?) {
        if (notSection != null) {
            append("not")
            translate(notSection.clauses)
        }
    }

    fun translate(orSection: OrSection?) {
        if (orSection != null) {
            append("or")
            translate(orSection.clauses)
        }
    }

    fun translate(statesSection: StatesSection?) {
        if (statesSection != null) {
            append("represents")
        }
    }

    fun translate(resourceSection: ResourceSection?) {
        if (resourceSection != null) {
            append("\\begin{enumerate}")
            for (item in resourceSection.items) {
                append(
                    "\\item \\textbf{${item.section.name}}: ${item.section.values.joinToString(",")}")
            }
            append("\\end{enumerate}")
        }
    }

    fun translate(suchThatSection: SuchThatSection?) {
        if (suchThatSection != null) {
            append("such that")
            translate(suchThatSection.clauses)
        }
    }

    fun translate(textSection: TextSection?) {
        if (textSection != null) {
            append(textSection.text)
        }
    }

    fun translate(thenSection: ThenSection?) {
        if (thenSection != null) {
            append("then")
            translate(thenSection.clauses)
        }
    }

    fun translate(whereSection: WhereSection?) {
        append("where")
        translate(whereSection?.clauses)
    }

    fun translate(clauseListNode: ClauseListNode?) {
        if (clauseListNode == null) {
            return
        }

        for (c in clauseListNode.clauses) {
            translate(c)
        }
    }

    fun translate(targets: List<Target>) {
        if (targets.isEmpty()) {
            return
        }

        append("\\(")
        translate(targets[0])
        append("\\)")

        for (i in 1 until targets.size) {
            append("and \\(")
            translate(targets[i])
            append("\\)")
        }
    }

    fun toText() = buffer.joinToString(" ")

    private fun append(text: String) = buffer.add(text)
}
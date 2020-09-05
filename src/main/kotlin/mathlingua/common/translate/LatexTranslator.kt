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

package mathlingua.common.translate

import mathlingua.common.MathLingua
import mathlingua.common.ValidationSuccess
import mathlingua.common.chalktalk.phase2.ast.Document
import mathlingua.common.chalktalk.phase2.ast.clause.AbstractionNode
import mathlingua.common.chalktalk.phase2.ast.clause.AssignmentNode
import mathlingua.common.chalktalk.phase2.ast.clause.Clause
import mathlingua.common.chalktalk.phase2.ast.clause.ClauseListNode
import mathlingua.common.chalktalk.phase2.ast.clause.ExistsGroup
import mathlingua.common.chalktalk.phase2.ast.clause.ExpandsGroup
import mathlingua.common.chalktalk.phase2.ast.clause.ForGroup
import mathlingua.common.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.common.chalktalk.phase2.ast.clause.Identifier
import mathlingua.common.chalktalk.phase2.ast.clause.IfGroup
import mathlingua.common.chalktalk.phase2.ast.clause.IffGroup
import mathlingua.common.chalktalk.phase2.ast.clause.LatexGroup
import mathlingua.common.chalktalk.phase2.ast.clause.LeanGroup
import mathlingua.common.chalktalk.phase2.ast.clause.NotGroup
import mathlingua.common.chalktalk.phase2.ast.clause.OrGroup
import mathlingua.common.chalktalk.phase2.ast.clause.Statement
import mathlingua.common.chalktalk.phase2.ast.clause.Target
import mathlingua.common.chalktalk.phase2.ast.clause.Text
import mathlingua.common.chalktalk.phase2.ast.clause.TupleNode
import mathlingua.common.chalktalk.phase2.ast.metadata.item.StringSectionGroup
import mathlingua.common.chalktalk.phase2.ast.section.AsSection
import mathlingua.common.chalktalk.phase2.ast.section.WhenSection
import mathlingua.common.chalktalk.phase2.ast.section.AxiomSection
import mathlingua.common.chalktalk.phase2.ast.section.ConjectureSection
import mathlingua.common.chalktalk.phase2.ast.section.DefinesSection
import mathlingua.common.chalktalk.phase2.ast.section.DefinitionSection
import mathlingua.common.chalktalk.phase2.ast.section.ExistsSection
import mathlingua.common.chalktalk.phase2.ast.section.ExpandsSection
import mathlingua.common.chalktalk.phase2.ast.section.ForSection
import mathlingua.common.chalktalk.phase2.ast.section.IfSection
import mathlingua.common.chalktalk.phase2.ast.section.IffSection
import mathlingua.common.chalktalk.phase2.ast.section.LatexSection
import mathlingua.common.chalktalk.phase2.ast.section.MeansSection
import mathlingua.common.chalktalk.phase2.ast.section.NotSection
import mathlingua.common.chalktalk.phase2.ast.section.NoteSection
import mathlingua.common.chalktalk.phase2.ast.section.OrSection
import mathlingua.common.chalktalk.phase2.ast.section.ProblemSection
import mathlingua.common.chalktalk.phase2.ast.section.RefinesSection
import mathlingua.common.chalktalk.phase2.ast.section.RepresentsSection
import mathlingua.common.chalktalk.phase2.ast.section.ResourceSection
import mathlingua.common.chalktalk.phase2.ast.section.SuchThatSection
import mathlingua.common.chalktalk.phase2.ast.section.TextSection
import mathlingua.common.chalktalk.phase2.ast.section.ThatSection
import mathlingua.common.chalktalk.phase2.ast.section.ThenSection
import mathlingua.common.chalktalk.phase2.ast.section.TheoremSection
import mathlingua.common.chalktalk.phase2.ast.section.WhereSection
import mathlingua.common.chalktalk.phase2.ast.toplevel.AxiomGroup
import mathlingua.common.chalktalk.phase2.ast.toplevel.ConjectureGroup
import mathlingua.common.chalktalk.phase2.ast.toplevel.DefinesGroup
import mathlingua.common.chalktalk.phase2.ast.toplevel.DefinitionGroup
import mathlingua.common.chalktalk.phase2.ast.toplevel.NoteGroup
import mathlingua.common.chalktalk.phase2.ast.toplevel.ProblemGroup
import mathlingua.common.chalktalk.phase2.ast.toplevel.RepresentsGroup
import mathlingua.common.chalktalk.phase2.ast.toplevel.ResourceGroup
import mathlingua.common.chalktalk.phase2.ast.toplevel.TheoremGroup
import mathlingua.common.chalktalk.phase2.ast.toplevel.TopLevelGroup

class LatexTranslator(
    val defines: List<DefinesGroup>,
    val represents: List<RepresentsGroup>
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

    fun translate(forGroup: ForGroup?) {
        translate(forGroup?.forSection)
        translate(forGroup?.whereSection)
        translate(forGroup?.thenSection)
    }

    fun translate(notGroup: NotGroup?) {
        translate(notGroup?.notSection)
    }

    fun translate(orGroup: OrGroup?) {
        translate(orGroup?.orSection)
    }

    fun translate(statement: Statement?) {
        if (statement != null) {
            val text = if (statement.texTalkRoot is ValidationSuccess) {
                MathLingua.expandWrittenAs(statement.texTalkRoot.value, defines, represents).text
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

    fun translate(latexGroup: LatexGroup?) {
        if (latexGroup != null) {
            translate(latexGroup.latexSection)
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

    fun translate(definitionGroup: DefinitionGroup?) {
        if (definitionGroup != null) {
            append("\\textbf{Definition}")
            translate(definitionGroup.definitionSection)
        }
    }

    fun translate(definesGroup: DefinesGroup?) {
        if (definesGroup != null) {
            append("\\textbf{Definition} A")
            val written = definesGroup.metaDataSection?.items?.find {
                it is StringSectionGroup && it.section.name == "written"
            } as StringSectionGroup?
            if (written != null && written.section.values.isNotEmpty()) {
                append("\\(${written.section.values[0].removeSurrounding("\"", "\"")}\\)")
            } else {
                append("\\verb   '")
                append(definesGroup.id.text)
                append("'")
            }
            translate(definesGroup.definesSection)
            translate(definesGroup.whenSection)
            translate(definesGroup.meansSection)
        }
    }

    fun translate(representsGroup: RepresentsGroup?) {
        if (representsGroup != null) {
            append(representsGroup.id.text)
            append("represents")
            for (ts in representsGroup.thatSections) {
                translate(ts)
            }
        }
    }

    fun translate(noteGroup: NoteGroup?) {
        if (noteGroup != null) {
            append("\\textbf{Note}")
            translate(noteGroup.noteSection)
        }
    }

    fun translate(problemGroup: ProblemGroup?) {
        if (problemGroup != null) {
            append("\\textbf{Problem}")
            translate(problemGroup.problemSection)
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
            is DefinitionGroup -> translate(topLevelGroup)
            is DefinesGroup -> translate(topLevelGroup)
            is RepresentsGroup -> translate(topLevelGroup)
            is TheoremGroup -> translate(topLevelGroup)
            is AxiomGroup -> translate(topLevelGroup)
            is ConjectureGroup -> translate(topLevelGroup)
            is ProblemGroup -> translate(topLevelGroup)
            is NoteGroup -> translate(topLevelGroup)
        }
    }

    fun translate(clause: Clause?) {
        when (clause) {
            is AbstractionNode -> translate(clause)
            is AssignmentNode -> translate(clause)
            is ExistsGroup -> translate(clause)
            is ExpandsGroup -> translate(clause)
            is ForGroup -> translate(clause)
            is Identifier -> translate(clause)
            is IffGroup -> translate(clause)
            is IfGroup -> translate(clause)
            is NotGroup -> translate(clause)
            is OrGroup -> translate(clause)
            is Statement -> translate(clause)
            is IdStatement -> translate(clause)
            is Text -> translate(clause)
            is TupleNode -> translate(clause)
            is LatexGroup -> translate(clause)
            is LeanGroup -> translate(clause)
            else -> throw RuntimeException("Unknown clause ${clause?.toCode(false, 0)?.getCode()}")
        }
    }

    fun translate(leanGroup: LeanGroup?) {
        if (leanGroup != null) {
            append("\\begin{verbatim}\n")
            if (leanGroup.importSection != null) {
                for (imp in leanGroup.importSection.imports) {
                    append("import $imp\n")
                }
                append("\n")
            }
            if (leanGroup.variableSection != null) {
                for (v in leanGroup.variableSection.variables) {
                    append("variables $v\n")
                }
                append("\n")
            }
            append(leanGroup.leanSection.text)
            append("\n")
            append("\\end{verbatim}\n")
        }
    }

    fun translate(idStatement: IdStatement?) {
        translate(idStatement?.toStatement())
    }

    fun translate(text: Text) {
        append(text.text.replace("?", "").removeSurrounding("\"", "\""))
    }

    fun translate(theoremSection: TheoremSection?) {
        if (theoremSection != null) {
            translate(theoremSection.clauses)
        }
    }

    fun translate(asSection: AsSection?) {
        if (asSection != null) {
            append("as")
            translate(asSection.clauses)
        }
    }

    fun translate(whenSection: WhenSection?) {
        if (whenSection != null) {
            append("assuming")
            translate(whenSection.clauses)
        }
    }

    fun translate(axiomSection: AxiomSection?) {
        if (axiomSection != null) {
            translate(axiomSection.clauses)
        }
    }

    fun translate(conjectureSection: ConjectureSection?) {
        if (conjectureSection != null) {
            translate(conjectureSection.clauses)
        }
    }

    fun translate(definesSection: DefinesSection?) {
        if (definesSection != null) {
            append("defines")
            translate(definesSection.targets)
        }
    }

    fun translate(definitionSection: DefinitionSection?) {
        if (definitionSection != null) {
            translate(definitionSection.clauses)
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

    fun translate(forSection: ForSection?) {
        if (forSection != null) {
            append("for all")
            translate(forSection.targets)
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

    fun translate(latexSection: LatexSection?) {
        if (latexSection != null) {
            append(latexSection.text.removeSurrounding("\"", "\"").replace("?", ""))
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

    fun translate(noteSection: NoteSection?) {
        if (noteSection != null) {
            translate(noteSection.clauses)
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

    fun translate(problemSection: ProblemSection?) {
        if (problemSection != null) {
            translate(problemSection.clauses)
        }
    }

    fun translate(refinesSection: RefinesSection?) {
        if (refinesSection != null) {
            translate(refinesSection.targets)
        }
    }

    fun translate(representsSection: RepresentsSection?) {
        if (representsSection != null) {
            append("represents")
        }
    }

    fun translate(resourceSection: ResourceSection?) {
        if (resourceSection != null) {
            append("\\begin{enumerate}")
            for (item in resourceSection.items) {
                append("\\item \\textbf{${item.section.name}}: ${item.section.values.joinToString(",")}")
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

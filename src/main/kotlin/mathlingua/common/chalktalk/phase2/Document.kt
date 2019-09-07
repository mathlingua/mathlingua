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

package mathlingua.common.chalktalk.phase2

import mathlingua.common.ParseError
import mathlingua.common.Validation
import mathlingua.common.chalktalk.phase1.ast.ChalkTalkNode
import mathlingua.common.chalktalk.phase1.ast.Root
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow

interface Phase2Node {
    fun forEach(fn: (node: Phase2Node) -> Unit)
    fun toCode(isArg: Boolean, indent: Int): String
    fun renameVars(map: Map<String, String>): Phase2Node
}

data class Document(
    val defines: List<DefinesGroup>,
    val represents: List<RepresentsGroup>,
    val results: List<ResultGroup>,
    val axioms: List<AxiomGroup>,
    val conjectures: List<ConjectureGroup>,
    val sources: List<SourceGroup>
) : Phase2Node {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        defines.forEach(fn)
        represents.forEach(fn)
        results.forEach(fn)
        axioms.forEach(fn)
        conjectures.forEach(fn)
        sources.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()

        for (grp in defines) {
            builder.append(grp.toCode(false, 0))
            builder.append("\n\n\n")
        }

        for (grp in represents) {
            builder.append(grp.toCode(false, 0))
            builder.append("\n\n\n")
        }

        for (grp in axioms) {
            builder.append(grp.toCode(false, 0))
            builder.append("\n\n\n")
        }

        for (grp in conjectures) {
            builder.append(grp.toCode(false, 0))
            builder.append("\n\n\n")
        }

        for (grp in results) {
            builder.append(grp.toCode(false, 0))
            builder.append("\n\n\n")
        }

        for (src in sources) {
            builder.append(src.toCode(false, 0))
            builder.append("\n\n\n")
        }

        return builder.toString()
    }

    override fun renameVars(map: Map<String, String>): Phase2Node {
        return Document(
            defines = defines.map { it.renameVars(map) as DefinesGroup },
            axioms = axioms.map { it.renameVars(map) as AxiomGroup },
            conjectures = conjectures.map { it.renameVars(map) as ConjectureGroup },
            represents = represents.map { it.renameVars(map) as RepresentsGroup },
            results = results.map { it.renameVars(map) as ResultGroup },
            sources = sources.map { it.renameVars(map) as SourceGroup }
        )
    }
}

fun validateDocument(rawNode: ChalkTalkNode): Validation<Document> {
    val node = rawNode.resolve()

    val errors = ArrayList<ParseError>()
    if (node !is Root) {
        errors.add(
            ParseError(
                "Expected a Root",
                getRow(node),
                getColumn(node)
            )
        )
        return Validation.failure(errors)
    }

    val defines = ArrayList<DefinesGroup>()
    val represents = ArrayList<RepresentsGroup>()
    val results = ArrayList<ResultGroup>()
    val axioms = ArrayList<AxiomGroup>()
    val conjectures = ArrayList<ConjectureGroup>()
    val sources = ArrayList<SourceGroup>()

    val (groups) = node
    for (group in groups) {
        if (isResultGroup(group)) {
            val resultValidation = validateResultGroup(group)
            if (resultValidation.isSuccessful) {
                results.add(resultValidation.value!!)
            } else {
                errors.addAll(resultValidation.errors)
            }
        } else if (isAxiomGroup(group)) {
            val axiomValidation = validateAxiomGroup(group)
            if (axiomValidation.isSuccessful) {
                axioms.add(axiomValidation.value!!)
            } else {
                errors.addAll(axiomValidation.errors)
            }
        } else if (isConjectureGroup(group)) {
            val conjectureValidation = validateConjectureGroup(group)
            if (conjectureValidation.isSuccessful) {
                conjectures.add(conjectureValidation.value!!)
            } else {
                errors.addAll(conjectureValidation.errors)
            }
        } else if (isDefinesGroup(group)) {
            val definesValidation = validateDefinesGroup(group)
            if (definesValidation.isSuccessful) {
                defines.add(definesValidation.value!!)
            } else {
                errors.addAll(definesValidation.errors)
            }
        } else if (isRepresentsGroup(group)) {
            val representsValidation = validateRepresentsGroup(group)
            if (representsValidation.isSuccessful) {
                represents.add(representsValidation.value!!)
            } else {
                errors.addAll(representsValidation.errors)
            }
        } else if (isSourceGroup(group)) {
            val sourceValidation = validateSourceGroup(group)
            if (sourceValidation.isSuccessful) {
                sources.add(sourceValidation.value!!)
            } else {
                errors.addAll(sourceValidation.errors)
            }
        } else {
            errors.add(
                ParseError(
                    "Expected a top level group but found " + group.toCode(),
                    getRow(group), getColumn(group)
                )
            )
        }
    }

    return if (errors.isNotEmpty()) {
        Validation.failure(errors)
    } else Validation.success(
        Document(
            defines,
            represents,
            results,
            axioms,
            conjectures,
            sources
        )
    )
}

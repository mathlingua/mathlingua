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

package mathlingua.chalktalk.phase2.ast

import mathlingua.chalktalk.phase1.ast.AstUtils
import mathlingua.chalktalk.phase1.ast.ChalkTalkNode
import mathlingua.chalktalk.phase1.ast.Root
import mathlingua.common.ParseError
import mathlingua.common.Validation

interface Phase2Node {
    fun forEach(fn: (node: Phase2Node) -> Unit)
    fun toCode(isArg: Boolean, indent: Int): String
}

data class Document(
    val defines: List<DefinesGroup>,
    val refines: List<RefinesGroup>,
    val represents: List<RepresentsGroup>,
    val results: List<ResultGroup>,
    val axioms: List<AxiomGroup>,
    val conjectures: List<ConjectureGroup>
) : Phase2Node {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        defines.forEach(fn)
        refines.forEach(fn)
        represents.forEach(fn)
        results.forEach(fn)
        axioms.forEach(fn)
        conjectures.forEach(fn)
    }

    override fun toCode(isArg: Boolean, indent: Int): String {
        val builder = StringBuilder()

        for (grp in defines) {
            builder.append(grp.toCode(false, 0))
            builder.append("\n\n\n")
        }

        for (grp in refines) {
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

        return builder.toString()
    }

    companion object {

        fun validate(rawNode: ChalkTalkNode): Validation<Document> {
            val node = rawNode.resolve()

            val errors = ArrayList<ParseError>()
            if (node !is Root) {
                errors.add(
                    ParseError(
                        "Expected a Root but found " + node.javaClass.simpleName,
                        AstUtils.getRow(node),
                        AstUtils.getColumn(node)
                    )
                )
                return Validation.failure(errors)
            }

            val defines = ArrayList<DefinesGroup>()
            val refines = ArrayList<RefinesGroup>()
            val represents = ArrayList<RepresentsGroup>()
            val results = ArrayList<ResultGroup>()
            val axioms = ArrayList<AxiomGroup>()
            val conjectures = ArrayList<ConjectureGroup>()

            val (groups) = node
            for (group in groups) {
                if (ResultGroup.isResultGroup(group)) {
                    val resultValidation = ResultGroup.validate(group)
                    if (resultValidation.isSuccessful) {
                        results.add(resultValidation.value!!)
                    } else {
                        errors.addAll(resultValidation.errors)
                    }
                } else if (AxiomGroup.isAxiomGroup(group)) {
                    val axiomValidation = AxiomGroup.validate(group)
                    if (axiomValidation.isSuccessful) {
                        axioms.add(axiomValidation.value!!)
                    } else {
                        errors.addAll(axiomValidation.errors)
                    }
                } else if (ConjectureGroup.isConjectureGroup(group)) {
                    val conjectureValidation = ConjectureGroup.validate(group)
                    if (conjectureValidation.isSuccessful) {
                        conjectures.add(conjectureValidation.value!!)
                    } else {
                        errors.addAll(conjectureValidation.errors)
                    }
                } else if (DefinesGroup.isDefinesGroup(group)) {
                    val definesValidation = DefinesGroup.validate(group)
                    if (definesValidation.isSuccessful) {
                        defines.add(definesValidation.value!!)
                    } else {
                        errors.addAll(definesValidation.errors)
                    }
                } else if (RefinesGroup.isRefinesGroup(group)) {
                    val refinesValidation = RefinesGroup.validate(group)
                    if (refinesValidation.isSuccessful) {
                        refines.add(refinesValidation.value!!)
                    } else {
                        errors.addAll(refinesValidation.errors)
                    }
                } else if (RepresentsGroup.isRepresentsGroup(group)) {
                    val representsValidation = RepresentsGroup.validate(group)
                    if (representsValidation.isSuccessful) {
                        represents.add(representsValidation.value!!)
                    } else {
                        errors.addAll(representsValidation.errors)
                    }
                } else {
                    errors.add(
                        ParseError(
                            "Expected a Result or Defines but found " + group.toCode(),
                            AstUtils.getRow(group), AstUtils.getColumn(group)
                        )
                    )
                }
            }

            return if (!errors.isEmpty()) {
                Validation.failure(errors)
            } else Validation.success(Document(defines, refines, represents, results, axioms, conjectures))
        }
    }
}

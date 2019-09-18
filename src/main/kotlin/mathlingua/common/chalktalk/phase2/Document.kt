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
import mathlingua.common.ValidationFailure
import mathlingua.common.ValidationSuccess
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase1.ast.Root
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow

interface Phase2Node {
    var row: Int
    var column: Int
    fun forEach(fn: (node: Phase2Node) -> Unit)
    fun toCode(isArg: Boolean, indent: Int): String
    fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node
}

data class Document(
    val defines: List<DefinesGroup>,
    val represents: List<RepresentsGroup>,
    val results: List<ResultGroup>,
    val axioms: List<AxiomGroup>,
    val conjectures: List<ConjectureGroup>,
    val sources: List<SourceGroup>,
    override var row: Int,
    override var column: Int
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

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node {
        return chalkTransformer(
            Document(
                defines = defines.map { it.transform(chalkTransformer) as DefinesGroup },
                axioms = axioms.map { it.transform(chalkTransformer) as AxiomGroup },
                conjectures = conjectures.map { it.transform(chalkTransformer) as ConjectureGroup },
                represents = represents.map { it.transform(chalkTransformer) as RepresentsGroup },
                results = results.map { it.transform(chalkTransformer) as ResultGroup },
                sources = sources.map { it.transform(chalkTransformer) as SourceGroup },
                row = row,
                column = column
            )
        )
    }
}

fun validateDocument(rawNode: Phase1Node): Validation<Document> {
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
        return ValidationFailure(errors)
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
            when (val resultValidation = validateResultGroup(group)) {
                is ValidationSuccess -> results.add(resultValidation.value)
                is ValidationFailure -> errors.addAll(resultValidation.errors)
            }
        } else if (isAxiomGroup(group)) {
            when (val axiomValidation = validateAxiomGroup(group)) {
                is ValidationSuccess -> axioms.add(axiomValidation.value)
                is ValidationFailure -> errors.addAll(axiomValidation.errors)
            }
        } else if (isConjectureGroup(group)) {
            when (val conjectureValidation = validateConjectureGroup(group)) {
                is ValidationSuccess -> conjectures.add(conjectureValidation.value)
                is ValidationFailure -> errors.addAll(conjectureValidation.errors)
            }
        } else if (isDefinesGroup(group)) {
            when (val definesValidation = validateDefinesGroup(group)) {
                is ValidationSuccess -> defines.add(definesValidation.value)
                is ValidationFailure -> errors.addAll(definesValidation.errors)
            }
        } else if (isRepresentsGroup(group)) {
            when (val representsValidation = validateRepresentsGroup(group)) {
                is ValidationSuccess -> represents.add(representsValidation.value)
                is ValidationFailure -> errors.addAll(representsValidation.errors)
            }
        } else if (isSourceGroup(group)) {
            when (val sourceValidation = validateSourceGroup(group)) {
                is ValidationSuccess -> sources.add(sourceValidation.value)
                is ValidationFailure -> errors.addAll(sourceValidation.errors)
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

    var row = Integer.MAX_VALUE
    var column = Integer.MAX_VALUE

    fun updateRowCol(nodes: List<Phase2Node>) {
        for (n in nodes) {
            row = min(row, n.row)
            column = min(column, n.column)
        }
    }

    updateRowCol(defines)
    updateRowCol(represents)
    updateRowCol(results)
    updateRowCol(axioms)
    updateRowCol(conjectures)
    updateRowCol(sources)

    return if (errors.isNotEmpty()) {
        ValidationFailure(errors)
    } else ValidationSuccess(
        Document(
            defines,
            represents,
            results,
            axioms,
            conjectures,
            sources,
            row,
            column
        )
    )
}

fun min(x: Int, y: Int): Int {
    return if (x < y) x else y
}

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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel

import mathlingua.backend.SourceCollection
import mathlingua.backend.transform.Signature
import mathlingua.backend.transform.getVarsPhase2Node
import mathlingua.cli.EntityResult
import mathlingua.cli.fixClassNameBug
import mathlingua.cli.getAllWords
import mathlingua.frontend.chalktalk.phase1.ast.BlockComment
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.ConjectureGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.UsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.frontend.chalktalk.phase2.getCalledNames
import mathlingua.getRandomUuid
import mathlingua.md5Hash

internal abstract class TopLevelGroup(open val metaDataSection: MetaDataSection?) : Phase2Node

internal data class TopLevelBlockComment(
    val blockComment: BlockComment, override val row: Int, override val column: Int
) : TopLevelGroup(null) {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeBlockComment(blockComment.text)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(this)
}

internal fun isBlockComment(node: Phase1Node) = node is BlockComment

internal fun topLevelToCode(
    topLevelGroup: TopLevelGroup?,
    writer: CodeWriter,
    isArg: Boolean,
    indent: Int,
    id: IdStatement?,
    vararg sections: Phase2Node?
): CodeWriter {
    if (topLevelGroup != null) {
        writer.beginTopLevel(topLevelGroup, getRandomUuid())
    }
    var useAsArg = isArg
    if (id != null) {
        writer.writeIndent(isArg, indent)
        writer.writeId(id)
        writer.writeNewline()
        useAsArg = false
    }

    val nonNullSections = sections.filterNotNull()
    for (i in nonNullSections.indices) {
        val sect = nonNullSections[i]
        writer.append(sect, useAsArg, indent)
        useAsArg = false
        if (i != nonNullSections.size - 1) {
            writer.writeNewline()
        }
    }
    if (topLevelGroup != null) {
        writer.endTopLevel(0)
    }
    return writer
}

internal interface HasUsingSection {
    val usingSection: UsingSection?
}

internal interface HasSignature {
    val signature: Signature?
    val id: IdStatement?
}

internal fun TopLevelGroup.getWhenSection() =
    when (this) {
        is DefinesGroup -> this.whenSection
        is StatesGroup -> this.whenSection
        else -> null
    }

internal fun TopLevelGroup.getMeansExpressesSections() =
    when (this) {
        is DefinesGroup -> {
            val result = mutableListOf<Phase2Node>()
            if (this.satisfyingSection != null) {
                result.add(this.satisfyingSection)
            }
            if (this.expressingSection != null) {
                result.add(this.expressingSection)
            }
            result
        }
        else -> emptyList()
    }

internal fun TopLevelGroup.getInputSymbols(): Set<String> {
    val result = mutableSetOf<String>()
    when (this) {
        is DefinesGroup -> {
            result.addAll(this.id.getVarsPhase2Node().map { it.name })
            if (this.givenSection != null) {
                result.addAll(this.givenSection.getVarsPhase2Node().map { it.name })
            }
        }
        is StatesGroup -> {
            result.addAll(this.id.getVarsPhase2Node().map { it.name })
            if (this.givenSection != null) {
                result.addAll(this.givenSection.getVarsPhase2Node().map { it.name })
            }
        }
        is TheoremGroup -> {
            if (this.id != null) {
                result.addAll(this.id.getVarsPhase2Node().map { it.name })
            }
            if (this.givenSection != null) {
                result.addAll(this.givenSection.getVarsPhase2Node().map { it.name })
            }
        }
        is AxiomGroup -> {
            if (this.id != null) {
                result.addAll(this.id.getVarsPhase2Node().map { it.name })
            }
            if (this.givenSection != null) {
                result.addAll(this.givenSection.getVarsPhase2Node().map { it.name })
            }
        }
        is ConjectureGroup -> {
            if (this.id != null) {
                result.addAll(this.id.getVarsPhase2Node().map { it.name })
            }
            if (this.givenSection != null) {
                result.addAll(this.givenSection.getVarsPhase2Node().map { it.name })
            }
        }
    }
    return result
}

internal fun TopLevelGroup.getOutputSymbols(): Set<String> {
    val result = mutableSetOf<String>()
    when (this) {
        is DefinesGroup -> {
            result.addAll(this.definesSection.getVarsPhase2Node().map { it.name })
        }
    }
    return result
}

internal fun TopLevelGroup.toEntityResult(
    relativePath: String, sourceCollection: SourceCollection
): EntityResult {
    val renderedHtml =
        sourceCollection.prettyPrint(node = this, html = true, literal = false, doExpand = true)
    val rawHtml =
        sourceCollection.prettyPrint(node = this, html = true, literal = true, doExpand = false)
    return EntityResult(
        id = md5Hash(this.toCode(false, 0).getCode()),
        type = this.javaClass.simpleName,
        signature =
            if (this is HasSignature) {
                this.signature?.form
            } else {
                null
            },
        rawHtml = fixClassNameBug(rawHtml),
        renderedHtml = fixClassNameBug(renderedHtml),
        words = getAllWords(this).toList(),
        relativePath = relativePath,
        called = this.getCalledNames())
}

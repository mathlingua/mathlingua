/*
 * Copyright 2020 The MathLingua Authors
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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.Section
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_IF_OR_IFF_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.IfSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.validateIfSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.iff.IffSection
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.iff.validateIffSection
import mathlingua.frontend.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.frontend.support.ParseError

internal interface IfOrIffSection {
    fun isIf(): Boolean
    fun asIf(): IfSection
    fun isIff(): Boolean
    fun asIff(): IffSection
    fun resolve(): Phase2Node
    fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): IfOrIffSection
}

internal fun newIfOrIffSection(ifSection: IfSection): IfOrIffSection =
    IfOrIffSectionImpl(ifSection, null)

internal fun newIfOrIffSection(iffSection: IffSection): IfOrIffSection =
    IfOrIffSectionImpl(null, iffSection)

internal fun validateIfOrIffSection(
    root: Phase1Node, sections: Map<String, Section>, errors: MutableList<ParseError>
): IfOrIffSection? {
    val ifSection = ifNonNull(sections["if"]) { validateIfSection(it, errors) }
    val iffSection = ifNonNull(sections["iff"]) { validateIffSection(it, errors) }

    return if (ifSection == null && iffSection == null) {
        null
    } else if (ifSection != null && iffSection != null) {
        errors.add(
            ParseError(
                message = "Either an 'if' or 'iff' section must be specified but not both",
                row = root.row,
                column = root.column))
        DEFAULT_IF_OR_IFF_SECTION
    } else if (ifSection != null) {
        newIfOrIffSection(ifSection)
    } else {
        newIfOrIffSection(iffSection!!)
    }
}

private data class IfOrIffSectionImpl(val ifSection: IfSection?, val iffSection: IffSection?) :
    IfOrIffSection {
    init {
        if ((ifSection == null) == (iffSection == null)) {
            throw IllegalArgumentException(
                "Either an 'if' or an 'iff' section must be specified but not both.")
        }
    }

    override fun isIf(): Boolean {
        return ifSection != null
    }

    override fun asIf(): IfSection {
        return ifSection!!
    }

    override fun isIff(): Boolean {
        return iffSection != null
    }

    override fun asIff(): IffSection {
        return iffSection!!
    }

    override fun resolve(): Phase2Node =
        if (isIf()) {
            asIf()
        } else {
            asIff()
        }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): IfOrIffSection =
        if (isIf()) {
            newIfOrIffSection(chalkTransformer(asIf().transform(chalkTransformer)) as IfSection)
        } else {
            newIfOrIffSection(chalkTransformer(asIff().transform(chalkTransformer)) as IffSection)
        }
}

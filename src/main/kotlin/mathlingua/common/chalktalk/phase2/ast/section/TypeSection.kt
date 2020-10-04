/*
 * Copyright 2020 Google LLC
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

package mathlingua.common.chalktalk.phase2.ast.section

import mathlingua.common.support.MutableLocationTracker
import mathlingua.common.support.Validation
import mathlingua.common.support.ValidationFailure
import mathlingua.common.support.ValidationSuccess
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase2.CodeWriter
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.support.validationFailure
import mathlingua.common.support.validationSuccess

data class TypeSection(val text: String) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter): CodeWriter {
        writer.writeIndent(isArg, indent)
        writer.writeHeader("type")
        writer.writeIndent(false, 1)
        writer.writeDirect(text)
        return writer
    }

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(this)
}

fun validateTypeSection(
    rawNode: Phase1Node,
    tracker: MutableLocationTracker
): Validation<TypeSection> =
    when (val validation = validateTextSection(rawNode, "type", tracker)) {
        is ValidationFailure -> validationFailure(validation.errors)
        is ValidationSuccess -> validationSuccess(tracker, rawNode,
            TypeSection(validation.value.text)
        )
    }

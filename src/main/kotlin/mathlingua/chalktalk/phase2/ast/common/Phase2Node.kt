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

package mathlingua.chalktalk.phase2.ast.common

import mathlingua.chalktalk.phase2.CodeWriter
import mathlingua.chalktalk.phase2.MathLinguaCodeWriter

interface Phase2Node {
    fun forEach(fn: (node: Phase2Node) -> Unit)
    fun toCode(
        isArg: Boolean,
        indent: Int,
        writer: CodeWriter =
            MathLinguaCodeWriter(
                defines = emptyList(),
                states = emptyList(),
                foundations = emptyList(),
                mutuallyGroups = emptyList())
    ): CodeWriter
    fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node): Phase2Node
}

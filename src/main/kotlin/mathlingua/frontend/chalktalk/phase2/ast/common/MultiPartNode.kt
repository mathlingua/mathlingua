/*
 * Copyright 2020 The MathLingua Authors
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

package mathlingua.frontend.chalktalk.phase2.ast.common

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.CodeWriter

internal abstract class MultiPartNode(
    private val sections: List<Phase2Node?>,
    private val builder: (sections: List<Phase2Node?>) -> Phase2Node
) : Phase2Node {
    override fun forEach(fn: (node: Phase2Node) -> Unit) {
        for (sec in sections) {
            if (sec != null) {
                fn(sec)
            }
        }
    }

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
        mathlingua.frontend.chalktalk.phase2.ast.clause.toCode(
            writer, isArg, indent, *sections.toTypedArray())

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(builder(sections))
}

internal abstract class ZeroPartNode(val node: Phase1Node) : Phase2Node {

    override fun forEach(fn: (node: Phase2Node) -> Unit) {}

    override fun toCode(isArg: Boolean, indent: Int, writer: CodeWriter) =
        mathlingua.frontend.chalktalk.phase2.ast.clause.toCode(writer, isArg, indent, node)

    override fun transform(chalkTransformer: (node: Phase2Node) -> Phase2Node) =
        chalkTransformer(this)
}

internal abstract class OnePartNode<S : Phase2Node?>(
    s: S, row: Int, column: Int, builder: (s: S, row: Int, column: Int) -> Phase2Node
) :
    MultiPartNode(
        listOf(s),
        {
            @Suppress("UNCHECKED_CAST")
            builder(it[0] as S, row, column)
        })

internal abstract class TwoPartNode<S1 : Phase2Node?, S2 : Phase2Node?>(
    s1: S1,
    s2: S2,
    row: Int,
    column: Int,
    builder: (s1: S1, s2: S2, row: Int, column: Int) -> Phase2Node
) :
    MultiPartNode(
        listOf(s1, s2),
        {
            @Suppress("UNCHECKED_CAST")
            builder(it[0] as S1, it[1] as S2, row, column)
        })

internal abstract class ThreePartNode<S1 : Phase2Node?, S2 : Phase2Node?, S3 : Phase2Node?>(
    s1: S1,
    s2: S2,
    s3: S3,
    row: Int,
    column: Int,
    builder: (s1: S1, s2: S2, s3: S3, row: Int, column: Int) -> Phase2Node
) :
    MultiPartNode(
        listOf(s1, s2, s3),
        {
            @Suppress("UNCHECKED_CAST")
            builder(it[0] as S1, it[1] as S2, it[2] as S3, row, column)
        })

internal abstract class FourPartNode<
    S1 : Phase2Node?, S2 : Phase2Node?, S3 : Phase2Node?, S4 : Phase2Node?>(
    s1: S1,
    s2: S2,
    s3: S3,
    s4: S4,
    row: Int,
    column: Int,
    builder: (s1: S1, s2: S2, s3: S3, s4: S4, row: Int, column: Int) -> Phase2Node
) :
    MultiPartNode(
        listOf(s1, s2, s3, s4),
        {
            @Suppress("UNCHECKED_CAST")
            builder(it[0] as S1, it[1] as S2, it[2] as S3, it[3] as S4, row, column)
        })

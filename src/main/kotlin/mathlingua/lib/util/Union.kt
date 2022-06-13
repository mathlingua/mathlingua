/*
 * Copyright 2022 Dominic Kramer
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

package mathlingua.lib.util

internal sealed interface BiUnion<A : Parent, B : Parent, Parent> {
    val value: Parent
}

internal data class BiUnionFirst<A : Parent, B : Parent, Parent>(override val value: A) :
    BiUnion<A, B, Parent>

internal data class BiUnionSecond<A : Parent, B : Parent, Parent>(override val value: B) :
    BiUnion<A, B, Parent>

internal sealed interface TriUnion<A : Parent, B : Parent, C : Parent, Parent> {
    val value: Parent
}

internal data class TriUnionFirst<A : Parent, B : Parent, C : Parent, Parent>(
    override val value: A
) : TriUnion<A, B, C, Parent>

internal data class TriUnionSecond<A : Parent, B : Parent, C : Parent, Parent>(
    override val value: B
) : TriUnion<A, B, C, Parent>

internal data class TriUnionThird<A : Parent, B : Parent, C : Parent, Parent>(
    override val value: C
) : TriUnion<A, B, C, Parent>

/*
internal sealed interface QuadUnion<A, B, C, D>
internal data class QuadUnionFirst<A, B, C, D>(val value: A) : QuadUnion<A, B, C, D>
internal data class QuadUnionSecond<A, B, C, D>(val value: B) : QuadUnion<A, B, C, D>
internal data class QuadUnionThird<A, B, C, D>(val value: C) : QuadUnion<A, B, C, D>
internal data class QuadUnionFourth<A, B, C, D>(val value: C) : QuadUnion<A, B, C, D>
*/

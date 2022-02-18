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

package mathlingua.frontend.chalktalk.phase1.ast

internal enum class ChalkTalkTokenType {
    DotSpace,
    Name,
    Colon,
    String,
    Statement,
    Id,
    Comma,
    Begin,
    End,
    Linebreak,
    Invalid,
    Equals,
    ColonEquals,
    LParen,
    RParen,
    LCurly,
    RCurly,
    Underscore,
    DotDotDot,
    BlockComment
}

internal sealed class Phase1Target : Phase1Node

internal interface GroupOrBlockComment : Phase1Node

internal data class BlockComment(
    val text: String, override val row: Int, override val column: Int
) : GroupOrBlockComment {
    override fun forEach(fn: (node: Phase1Node) -> Unit) {}

    override fun toCode() = text

    override fun resolve() = this

    override fun transform(transformer: (node: Phase1Node) -> Phase1Node) = transformer(this)

    fun print(buffer: StringBuilder) {
        buffer.append(toCode())
    }
}

internal data class Group(
    val sections: List<Section>,
    val id: Phase1Token?,
    override val row: Int,
    override val column: Int
) : Phase1Target(), GroupOrBlockComment {

    override fun forEach(fn: (node: Phase1Node) -> Unit) {
        if (id != null) {
            fn(id)
        }
        sections.forEach(fn)
    }

    fun print(buffer: StringBuilder, level: Int, fromArg: Boolean) {
        if (id != null) {
            buffer.append(id.text)
            buffer.append("\n")
        }
        var first = true
        for (sect in sections) {
            if (first) {
                sect.print(buffer, level, fromArg && id == null)
            } else {
                sect.print(buffer, level, false)
            }
            first = false
        }
    }

    override fun toCode(): String {
        val buffer = StringBuilder()
        print(buffer, 0, false)
        return buffer.toString()
    }

    override fun resolve() = this

    override fun transform(transformer: (node: Phase1Node) -> Phase1Node) =
        transformer(
            Group(
                sections = sections.map { it.transform(transformer) as Section },
                id = id?.transform(transformer) as Phase1Token,
                row,
                column))
}

// ------------------------------------------------------------------------------------------------------------------ //

internal sealed class TupleItem : Phase1Target()

internal data class Assignment(
    val lhs: Phase1Token, val rhs: AssignmentRhs, override val row: Int, override val column: Int
) : TupleItem() {

    override fun forEach(fn: (node: Phase1Node) -> Unit) {
        fn(lhs)
        fn(rhs)
    }

    override fun toCode() = lhs.toCode() + " := " + rhs.toCode()

    override fun resolve() = this

    override fun transform(transformer: (node: Phase1Node) -> Phase1Node) =
        transformer(
            Assignment(
                lhs = lhs.transform(transformer) as Phase1Token,
                rhs = rhs.transform(transformer) as AssignmentRhs,
                row,
                column))
}

// ------------------------------------------------------------------------------------------------------------------ //

internal sealed class AssignmentRhs : TupleItem()

internal data class Phase1Token(
    val text: String, val type: ChalkTalkTokenType, override val row: Int, override val column: Int
) : AssignmentRhs() {

    override fun forEach(fn: (node: Phase1Node) -> Unit) {}

    override fun toCode() = text

    override fun resolve() = this

    override fun transform(transformer: (node: Phase1Node) -> Phase1Node) = transformer(this)
}

internal data class Tuple(
    val items: List<TupleItem>, override val row: Int, override val column: Int
) : AssignmentRhs() {

    override fun forEach(fn: (node: Phase1Node) -> Unit) = items.forEach(fn)

    override fun toCode(): String {
        val builder = StringBuilder()
        builder.append('(')
        for (i in items.indices) {
            builder.append(items[i].toCode())
            if (i != items.size - 1) {
                builder.append(", ")
            }
        }
        builder.append(')')
        return builder.toString()
    }

    override fun resolve() = this

    override fun transform(transformer: (node: Phase1Node) -> Phase1Node) =
        transformer(
            Tuple(items = items.map { it.transform(transformer) as TupleItem }, row, column))
}

internal data class Abstraction(
    val isEnclosed: Boolean,
    val isVarArgs: Boolean,
    val parts: List<AbstractionPart>,
    val subParams: List<Phase1Token>?,
    override val row: Int,
    override val column: Int
) : AssignmentRhs() {

    override fun forEach(fn: (node: Phase1Node) -> Unit) = parts.forEach(fn)

    override fun toCode(): String {
        val builder = StringBuilder()
        if (isEnclosed) {
            builder.append('{')
        }
        for (i in parts.indices) {
            builder.append(parts[i].toCode())
            if (i != parts.size - 1) {
                builder.append(", ")
            }
        }
        if (isEnclosed) {
            builder.append('}')
            if (subParams != null) {
                builder.append('_')
                if (subParams.size == 1) {
                    builder.append(subParams[0].toCode())
                } else {
                    builder.append('{')
                    for (i in subParams.indices) {
                        builder.append(subParams[i].toCode())
                        if (i != subParams.size - 1) {
                            builder.append(", ")
                        }
                    }
                    builder.append('}')
                }
            }
        }

        if (isVarArgs) {
            builder.append("...")
        }

        return builder.toString()
    }

    override fun resolve() = this

    override fun transform(transformer: (node: Phase1Node) -> Phase1Node) =
        transformer(
            Abstraction(
                isEnclosed = isEnclosed,
                isVarArgs = isVarArgs,
                parts = parts.map { it.transform(transformer) as AbstractionPart },
                subParams = subParams?.map { it.transform(transformer) as Phase1Token },
                row,
                column))
}

internal data class AbstractionPart(
    val name: Phase1Token,
    val subParams: List<Phase1Token>?,
    val params: List<Phase1Token>?,
    val tail: AbstractionPart?,
    override val row: Int,
    override val column: Int
) : AssignmentRhs() {

    override fun forEach(fn: (node: Phase1Node) -> Unit) {
        fn(name)
        subParams?.forEach(fn)
        params?.forEach(fn)
        if (tail != null) {
            fn(tail)
        }
    }

    override fun toCode(): String {
        val builder = StringBuilder()
        builder.append(name.toCode())
        if (subParams != null) {
            builder.append('_')
            if (subParams.size == 1) {
                builder.append(subParams[0].toCode())
            } else {
                builder.append('{')
                for (i in subParams.indices) {
                    builder.append(subParams[i].toCode())
                    if (i != subParams.size - 1) {
                        builder.append(", ")
                    }
                }
                builder.append('}')
            }
        }

        if (params != null) {
            builder.append('(')
            for (i in params.indices) {
                builder.append(params[i].toCode())
                if (i != params.size - 1) {
                    builder.append(", ")
                }
            }
            builder.append(')')
        }

        if (tail != null) {
            builder.append(" ... ")
            builder.append(tail.toCode())
        }

        return builder.toString()
    }

    override fun resolve() = this

    override fun transform(transformer: (node: Phase1Node) -> Phase1Node): Phase1Node {
        return transformer(
            AbstractionPart(
                name = name.transform(transformer) as Phase1Token,
                subParams = subParams?.map { it.transform(transformer) as Phase1Token },
                params = params?.map { it.transform(transformer) as Phase1Token },
                tail = tail?.transform(transformer) as AbstractionPart?,
                row,
                column))
    }
}

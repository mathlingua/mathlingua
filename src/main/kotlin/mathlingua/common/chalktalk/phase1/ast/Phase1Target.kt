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

package mathlingua.common.chalktalk.phase1.ast

enum class ChalkTalkTokenType {
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
    Underscore
}

sealed class Phase1Target : Phase1Node

data class Mapping(val lhs: Phase1Token, val rhs: Phase1Token) : Phase1Target() {

    override fun forEach(fn: (node: Phase1Node) -> Unit) {
        fn(lhs)
        fn(rhs)
    }

    override fun toCode() = lhs.toCode() + " = " + rhs.toCode()

    override fun resolve() = this

    override fun transform(transformer: (node: Phase1Node) -> Phase1Node) = transformer(Mapping(
            lhs = lhs.transform(transformer) as Phase1Token,
            rhs = rhs.transform(transformer) as Phase1Token
    ))
}

data class Group(val sections: List<Section>, val id: Phase1Token?) : Phase1Target() {

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

    override fun transform(transformer: (node: Phase1Node) -> Phase1Node) = transformer(Group(
            sections = sections.map { it.transform(transformer) as Section },
            id = id?.transform(transformer) as Phase1Token
    ))
}

// ------------------------------------------------------------------------------------------------------------------ //

sealed class TupleItem : Phase1Target()

data class Assignment(val lhs: Phase1Token, val rhs: AssignmentRhs) : TupleItem() {

    override fun forEach(fn: (node: Phase1Node) -> Unit) {
        fn(lhs)
        fn(rhs)
    }

    override fun toCode() = lhs.toCode() + " := " + rhs.toCode()

    override fun resolve() = this

    override fun transform(transformer: (node: Phase1Node) -> Phase1Node) = transformer(Assignment(
            lhs = lhs.transform(transformer) as Phase1Token,
            rhs = rhs.transform(transformer) as Phase1Token
    ))
}

// ------------------------------------------------------------------------------------------------------------------ //

sealed class AssignmentRhs : TupleItem()

data class Phase1Token(val text: String, val type: ChalkTalkTokenType, val row: Int, val column: Int) :
    AssignmentRhs() {

    override fun forEach(fn: (node: Phase1Node) -> Unit) {
    }

    override fun toCode() = text

    override fun resolve() = this

    override fun transform(transformer: (node: Phase1Node) -> Phase1Node) = transformer(this)
}

data class Tuple(val items: List<TupleItem>) : AssignmentRhs() {

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

    override fun transform(transformer: (node: Phase1Node) -> Phase1Node) = transformer(Tuple(
        items = items.map { it.transform(transformer) as TupleItem }
    ))
}

data class Abstraction(val isEnclosed: Boolean, val parts: List<AbstractionPart>, val subParams: List<Phase1Token>?) : AssignmentRhs() {

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

        return builder.toString()
    }

    override fun resolve() = this

    override fun transform(transformer: (node: Phase1Node) -> Phase1Node) = transformer(Abstraction(
        isEnclosed = isEnclosed,
        parts = parts.map { it.transform(transformer) as AbstractionPart },
        subParams = subParams?.map { it.transform(transformer) as Phase1Token }
    ))
}

data class AbstractionPart(val name: Phase1Token, val subParams: List<Phase1Token>?, val params: List<Phase1Token>?) :
    AssignmentRhs() {

    override fun forEach(fn: (node: Phase1Node) -> Unit) {
        fn(name)
        subParams?.forEach(fn)
        params?.forEach(fn)
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

        return builder.toString()
    }

    override fun resolve() = this

    override fun transform(transformer: (node: Phase1Node) -> Phase1Node) = transformer(AbstractionPart(
            name = name.transform(transformer) as Phase1Token,
            subParams = subParams?.map { it.transform(transformer) as Phase1Token },
            params = params?.map { it.transform(transformer) as Phase1Token }
    ))
}

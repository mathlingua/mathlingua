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

package mathlingua.common.textalk

enum class TexTalkNodeType {
    Token,
    Identifier,
    Operator,
    ParenGroup,
    SquareGroup,
    CurlyGroup,
    NamedGroup,
    Command,
    CommandPart,
    Expression,
    SubSup,
    Parameters,
    Comma,
    Is,
    ColonEquals
}

interface TexTalkNode {
    val type: TexTalkNodeType
    fun toCode(): String
    fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit)
    fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode): TexTalkNode
}

data class IsTexTalkNode(val lhs: ParametersTexTalkNode, val rhs: ParametersTexTalkNode) : TexTalkNode {
    override val type: TexTalkNodeType
        get() = TexTalkNodeType.Is

    override fun toCode(): String {
        val builder = StringBuilder()
        builder.append(lhs.toCode())
        builder.append(" is ")
        builder.append(rhs.toCode())
        return builder.toString()
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
        fn(lhs)
        fn(rhs)
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode): TexTalkNode {
        return transformer(IsTexTalkNode(
            lhs = lhs.transform(transformer) as ParametersTexTalkNode,
            rhs = rhs.transform(transformer) as ParametersTexTalkNode
        ))
    }
}

data class ColonEqualsTexTalkNode(val lhs: ParametersTexTalkNode, val rhs: ParametersTexTalkNode) : TexTalkNode {
    override val type: TexTalkNodeType
        get() = TexTalkNodeType.ColonEquals

    override fun toCode(): String {
        val builder = StringBuilder()
        builder.append(lhs.toCode())
        builder.append(" := ")
        builder.append(rhs.toCode())
        return builder.toString()
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
        fn(lhs)
        fn(rhs)
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode): TexTalkNode {
        return transformer(ColonEqualsTexTalkNode(
            lhs = lhs.transform(transformer) as ParametersTexTalkNode,
            rhs = rhs.transform(transformer) as ParametersTexTalkNode
        ))
    }
}

data class CommandPart(
    val name: TextTexTalkNode,
    val square: GroupTexTalkNode?,
    val subSup: SubSupTexTalkNode?,
    val groups: List<GroupTexTalkNode>,
    val namedGroups: List<NamedGroupTexTalkNode>
) : TexTalkNode {
    override val type: TexTalkNodeType
        get() = TexTalkNodeType.CommandPart

    override fun toCode(): String {
        val buffer = StringBuilder()

        buffer.append(name.toCode())

        if (square != null) {
            buffer.append(square.toCode())
        }

        if (subSup != null) {
            buffer.append(subSup.toCode())
        }

        for (grp in groups) {
            buffer.append(grp.toCode())
        }

        if (namedGroups.isNotEmpty()) {
            buffer.append(":")
        }
        for (namedGrp in namedGroups) {
            buffer.append(namedGrp.toCode())
        }
        return buffer.toString()
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
        fn(name)

        if (square != null) {
            fn(square)
        }

        if (subSup != null) {
            fn(subSup)
        }

        groups.forEach(fn)
        namedGroups.forEach(fn)
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode): TexTalkNode {
        return transformer(CommandPart(
            name = name.transform(transformer) as TextTexTalkNode,
            square = square?.transform(transformer) as GroupTexTalkNode?,
            subSup = subSup?.transform(transformer) as SubSupTexTalkNode?,
            groups = groups.map { it.transform(transformer) as GroupTexTalkNode },
            namedGroups = namedGroups.map { it.transform(transformer) as NamedGroupTexTalkNode }
        ))
    }
}

data class Command(val parts: List<CommandPart>) : TexTalkNode {
    override val type: TexTalkNodeType
        get() = TexTalkNodeType.Command

    override fun toCode(): String {
        val builder = StringBuilder("\\")
        for (i in 0 until parts.size) {
            if (i > 0) {
                builder.append('.')
            }
            builder.append(parts[i].toCode())
        }
        return builder.toString()
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
        parts.forEach(fn)
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode): TexTalkNode {
        return transformer(Command(
            parts = parts.map { it.transform(transformer) as CommandPart }
        ))
    }
}

data class ExpressionTexTalkNode(val children: List<TexTalkNode>) : TexTalkNode {

    override val type: TexTalkNodeType
        get() = TexTalkNodeType.Expression

    override fun toCode(): String {
        val builder = StringBuilder()
        val children = children
        for (i in children.indices) {
            val child = children[i]
            builder.append(child.toCode())
            if (i != children.size - 1) {
                builder.append(" ")
            }
        }
        return builder.toString()
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
        children.forEach(fn)
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode): TexTalkNode {
        return transformer(ExpressionTexTalkNode(
            children = children.map { it.transform(transformer) }
        ))
    }
}

data class ParametersTexTalkNode(val items: List<ExpressionTexTalkNode>) : TexTalkNode {
    override val type: TexTalkNodeType
        get() = TexTalkNodeType.Parameters

    override fun toCode(): String {
        val buffer = StringBuilder()

        if (items.isNotEmpty()) {
            buffer.append(items[0].toCode())
        }

        for (i in 1 until items.size) {
            buffer.append(", ")
            buffer.append(items[i].toCode())
        }

        return buffer.toString()
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
        items.forEach(fn)
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode): TexTalkNode {
        return transformer(ParametersTexTalkNode(
            items = items.map { it.transform(transformer) as ExpressionTexTalkNode }
        ))
    }
}

data class GroupTexTalkNode(override val type: TexTalkNodeType, val parameters: ParametersTexTalkNode) : TexTalkNode {

    override fun toCode(): String {
        val prefix: String
        val suffix: String

        when (type) {
            TexTalkNodeType.ParenGroup -> {
                prefix = "("
                suffix = ")"
            }
            TexTalkNodeType.SquareGroup -> {
                prefix = "["
                suffix = "]"
            }
            TexTalkNodeType.CurlyGroup -> {
                prefix = "{"
                suffix = "}"
            }
            else -> throw RuntimeException("Unrecognized group type $type")
        }

        val buffer = StringBuilder(prefix)
        buffer.append(parameters.toCode())
        buffer.append(suffix)
        return buffer.toString()
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
        fn(parameters)
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode): TexTalkNode {
        return transformer(GroupTexTalkNode(
            type = type,
            parameters = parameters.transform(transformer) as ParametersTexTalkNode
        ))
    }
}

data class NamedGroupTexTalkNode(
    val name: TextTexTalkNode,
    val group: GroupTexTalkNode
) : TexTalkNode {

    override val type: TexTalkNodeType
        get() = TexTalkNodeType.NamedGroup

    override fun toCode(): String {
        val buffer = StringBuilder()
        buffer.append(name.toCode())
        buffer.append(group.toCode())
        return buffer.toString()
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
        fn(name)
        fn(group)
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode): TexTalkNode {
        return transformer(NamedGroupTexTalkNode(
            name = name.transform(transformer) as TextTexTalkNode,
            group = group.transform(transformer) as GroupTexTalkNode
        ))
    }
}

data class SubSupTexTalkNode(
    val sub: GroupTexTalkNode?,
    val sup: GroupTexTalkNode?
) : TexTalkNode {

    override val type: TexTalkNodeType
        get() = TexTalkNodeType.SubSup

    override fun toCode(): String {
        val builder = StringBuilder()
        if (sub != null) {
            builder.append("_")
            builder.append(sub.toCode())
        }

        if (sup != null) {
            builder.append("^")
            builder.append(sup.toCode())
        }
        return builder.toString()
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
        if (sub != null) {
            fn(sub)
        }

        if (sup != null) {
            fn(sup)
        }
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode): TexTalkNode {
        return transformer(SubSupTexTalkNode(
            sub = sub?.transform(transformer) as GroupTexTalkNode?,
            sup = sup?.transform(transformer) as GroupTexTalkNode?
        ))
    }
}

data class TextTexTalkNode(override val type: TexTalkNodeType, val text: String) : TexTalkNode {
    override fun toCode(): String {
        return text
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode): TexTalkNode {
        return transformer(this)
    }
}

data class TexTalkToken(
    val text: String,
    val tokenType: TexTalkTokenType,
    val row: Int,
    val column: Int
) : TexTalkNode {
    override val type: TexTalkNodeType
        get() = TexTalkNodeType.Token

    override fun toCode(): String {
        return text
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode): TexTalkNode {
        return transformer(this)
    }
}

enum class TexTalkTokenType {
    Backslash,
    LParen,
    RParen,
    LSquare,
    RSquare,
    LCurly,
    RCurly,
    Operator,
    Identifier,
    Comma,
    Period,
    Colon,
    Underscore,
    Caret,
    ColonEquals,
    Is,
    Invalid
}

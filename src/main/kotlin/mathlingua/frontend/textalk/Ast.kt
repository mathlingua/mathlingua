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

package mathlingua.frontend.textalk

enum class TexTalkNodeType {
    Token,
    Identifier,
    Operator,
    SyntheticGroup,
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
    In,
    ColonColonEquals,
    ColonEquals,
    Mapping
}

interface TexTalkNode {
    val type: TexTalkNodeType
    fun toCode(interceptor: (node: TexTalkNode) -> String? = { null }): String
    fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit)
    fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode): TexTalkNode
}

data class IsTexTalkNode(val lhs: ParametersTexTalkNode, val rhs: ParametersTexTalkNode) :
    TexTalkNode {

    override val type: TexTalkNodeType
        get() = TexTalkNodeType.Is

    override fun toCode(interceptor: (node: TexTalkNode) -> String?): String {
        val res = interceptor(this)
        if (res != null) {
            return res
        }

        val builder = StringBuilder()
        builder.append(lhs.toCode(interceptor))
        builder.append(" is ")
        builder.append(rhs.toCode(interceptor))
        return builder.toString()
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
        fn(lhs)
        fn(rhs)
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode) =
        transformer(
            IsTexTalkNode(
                lhs = lhs.transform(transformer) as ParametersTexTalkNode,
                rhs = rhs.transform(transformer) as ParametersTexTalkNode))
}

data class InTexTalkNode(val lhs: ParametersTexTalkNode, val rhs: ParametersTexTalkNode) :
    TexTalkNode {

    override val type: TexTalkNodeType
        get() = TexTalkNodeType.In

    override fun toCode(interceptor: (node: TexTalkNode) -> String?): String {
        val res = interceptor(this)
        if (res != null) {
            return res
        }

        val builder = StringBuilder()
        builder.append(lhs.toCode(interceptor))
        builder.append(" in ")
        builder.append(rhs.toCode(interceptor))
        return builder.toString()
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
        fn(lhs)
        fn(rhs)
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode) =
        transformer(
            InTexTalkNode(
                lhs = lhs.transform(transformer) as ParametersTexTalkNode,
                rhs = rhs.transform(transformer) as ParametersTexTalkNode))
}

data class ColonEqualsTexTalkNode(val lhs: ParametersTexTalkNode, val rhs: ParametersTexTalkNode) :
    TexTalkNode {

    override val type: TexTalkNodeType
        get() = TexTalkNodeType.ColonEquals

    override fun toCode(interceptor: (node: TexTalkNode) -> String?): String {
        val res = interceptor(this)
        if (res != null) {
            return res
        }

        val builder = StringBuilder()
        builder.append(lhs.toCode(interceptor))
        builder.append(" := ")
        builder.append(rhs.toCode(interceptor))
        return builder.toString()
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
        fn(lhs)
        fn(rhs)
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode) =
        transformer(
            ColonEqualsTexTalkNode(
                lhs = lhs.transform(transformer) as ParametersTexTalkNode,
                rhs = rhs.transform(transformer) as ParametersTexTalkNode))
}

data class ColonColonEqualsTexTalkNode(
    val lhs: ParametersTexTalkNode, val rhs: ParametersTexTalkNode
) : TexTalkNode {

    override val type: TexTalkNodeType
        get() = TexTalkNodeType.ColonColonEquals

    override fun toCode(interceptor: (node: TexTalkNode) -> String?): String {
        val res = interceptor(this)
        if (res != null) {
            return res
        }

        val builder = StringBuilder()
        builder.append(lhs.toCode(interceptor))
        builder.append(" ::= ")
        builder.append(rhs.toCode(interceptor))
        return builder.toString()
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
        fn(lhs)
        fn(rhs)
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode) =
        transformer(
            ColonColonEqualsTexTalkNode(
                lhs = lhs.transform(transformer) as ParametersTexTalkNode,
                rhs = rhs.transform(transformer) as ParametersTexTalkNode))
}

data class CommandPart(
    val name: TextTexTalkNode,
    val square: GroupTexTalkNode?,
    val subSup: SubSupTexTalkNode?,
    val groups: List<GroupTexTalkNode>,
    val paren: GroupTexTalkNode?,
    val namedGroups: List<NamedGroupTexTalkNode>
) : TexTalkNode {

    override val type: TexTalkNodeType
        get() = TexTalkNodeType.CommandPart

    override fun toCode(interceptor: (node: TexTalkNode) -> String?): String {
        val res = interceptor(this)
        if (res != null) {
            return res
        }

        val buffer = StringBuilder()

        buffer.append(name.toCode(interceptor))

        if (square != null) {
            buffer.append(square.toCode(interceptor))
        }

        if (subSup != null) {
            buffer.append(subSup.toCode(interceptor))
        }

        for (grp in groups) {
            buffer.append(grp.toCode(interceptor))
        }

        if (paren != null) {
            buffer.append(paren.toCode(interceptor))
        }

        for (namedGrp in namedGroups) {
            buffer.append(namedGrp.toCode(interceptor))
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

        if (paren != null) {
            fn(paren)
        }

        namedGroups.forEach(fn)
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode) =
        transformer(
            CommandPart(
                name = name.transform(transformer) as TextTexTalkNode,
                square = square?.transform(transformer) as GroupTexTalkNode?,
                subSup = subSup?.transform(transformer) as SubSupTexTalkNode?,
                groups = groups.map { it.transform(transformer) as GroupTexTalkNode },
                paren = paren?.transform(transformer) as GroupTexTalkNode?,
                namedGroups =
                    namedGroups.map { it.transform(transformer) as NamedGroupTexTalkNode }))
}

data class Command(val parts: List<CommandPart>, val hasSuffix: Boolean) : TexTalkNode {

    override val type: TexTalkNodeType
        get() = TexTalkNodeType.Command

    override fun toCode(interceptor: (node: TexTalkNode) -> String?): String {
        val res = interceptor(this)
        if (res != null) {
            return res
        }

        val builder = StringBuilder("\\")
        for (i in 0 until parts.size) {
            if (i > 0) {
                builder.append('.')
            }
            builder.append(parts[i].toCode(interceptor))
        }
        if (hasSuffix) {
            builder.append("/")
        }
        return builder.toString()
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
        parts.forEach(fn)
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode) =
        transformer(
            Command(parts = parts.map { it.transform(transformer) as CommandPart }, hasSuffix))
}

data class ExpressionTexTalkNode(val children: List<TexTalkNode>) : TexTalkNode {

    override val type: TexTalkNodeType
        get() = TexTalkNodeType.Expression

    override fun toCode(interceptor: (node: TexTalkNode) -> String?): String {
        val res = interceptor(this)
        if (res != null) {
            return res
        }

        val builder = StringBuilder()
        for (i in children.indices) {
            val child = children[i]
            builder.append(child.toCode(interceptor))
            if (i != children.size - 1 && children[i + 1] is Command) {
                builder.append(" ")
            }
        }
        return builder.toString()
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
        children.forEach(fn)
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode) =
        transformer(ExpressionTexTalkNode(children = children.map { it.transform(transformer) }))
}

data class ParametersTexTalkNode(val items: List<ExpressionTexTalkNode>) : TexTalkNode {

    override val type: TexTalkNodeType
        get() = TexTalkNodeType.Parameters

    override fun toCode(interceptor: (node: TexTalkNode) -> String?): String {
        val res = interceptor(this)
        if (res != null) {
            return res
        }

        val buffer = StringBuilder()

        if (items.isNotEmpty()) {
            buffer.append(items[0].toCode(interceptor))
        }

        for (i in 1 until items.size) {
            buffer.append(", ")
            buffer.append(items[i].toCode(interceptor))
        }

        return buffer.toString()
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
        items.forEach(fn)
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode) =
        transformer(
            ParametersTexTalkNode(
                items = items.map { it.transform(transformer) as ExpressionTexTalkNode }))
}

data class MappingNode(
    val name: TextTexTalkNode, val subGroup: GroupTexTalkNode?, val parenGroup: GroupTexTalkNode?
) : TexTalkNode {
    override val type: TexTalkNodeType
        get() = TexTalkNodeType.Mapping

    override fun toCode(interceptor: (node: TexTalkNode) -> String?): String {
        val res = interceptor(this)
        if (res != null) {
            return res
        }

        val buffer = StringBuilder()

        buffer.append(name.toCode(interceptor))

        if (subGroup != null) {
            buffer.append("_")
            buffer.append(subGroup.toCode(interceptor))
        }

        if (parenGroup != null) {
            buffer.append(parenGroup.toCode(interceptor))
        }

        return buffer.toString()
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
        fn(name)

        if (subGroup != null) {
            fn(subGroup)
        }

        if (parenGroup != null) {
            fn(parenGroup)
        }
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode) =
        MappingNode(
            name = name.transform(transformer) as TextTexTalkNode,
            subGroup = subGroup?.transform(transformer) as GroupTexTalkNode?,
            parenGroup = parenGroup?.transform(transformer) as GroupTexTalkNode?)
}

data class GroupTexTalkNode(
    override val type: TexTalkNodeType, val parameters: ParametersTexTalkNode, val isVarArg: Boolean
) : TexTalkNode {

    override fun toCode(interceptor: (node: TexTalkNode) -> String?): String {
        val res = interceptor(this)
        if (res != null) {
            return res
        }

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
            TexTalkNodeType.SyntheticGroup -> {
                prefix = ""
                suffix = ""
            }
            else -> throw RuntimeException("Unrecognized group type $type")
        }

        val buffer = StringBuilder(prefix)
        buffer.append(parameters.toCode(interceptor))
        buffer.append(suffix)
        if (isVarArg) {
            buffer.append("...")
        }
        return buffer.toString()
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
        fn(parameters)
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode) =
        transformer(
            GroupTexTalkNode(
                type = type,
                parameters = parameters.transform(transformer) as ParametersTexTalkNode,
                isVarArg = isVarArg))
}

data class NamedGroupTexTalkNode(val name: TextTexTalkNode, val groups: List<GroupTexTalkNode>) :
    TexTalkNode {

    override val type: TexTalkNodeType
        get() = TexTalkNodeType.NamedGroup

    override fun toCode(interceptor: (node: TexTalkNode) -> String?): String {
        val res = interceptor(this)
        if (res != null) {
            return res
        }

        val buffer = StringBuilder()
        buffer.append(":")
        buffer.append(name.toCode(interceptor))
        for (grp in groups) {
            buffer.append(grp.toCode(interceptor))
        }
        return buffer.toString()
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
        fn(name)
        groups.forEach(fn)
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode) =
        transformer(
            NamedGroupTexTalkNode(
                name = name.transform(transformer) as TextTexTalkNode,
                groups = groups.map { it.transform(transformer) as GroupTexTalkNode }))
}

data class SubSupTexTalkNode(val sub: GroupTexTalkNode?, val sup: GroupTexTalkNode?) : TexTalkNode {

    override val type: TexTalkNodeType
        get() = TexTalkNodeType.SubSup

    override fun toCode(interceptor: (node: TexTalkNode) -> String?): String {
        val res = interceptor(this)
        if (res != null) {
            return res
        }

        val builder = StringBuilder()
        if (sub != null) {
            builder.append("_")
            if (sub.parameters.items.size == 1 &&
                sub.parameters.items[0].children.size == 1 &&
                sub.parameters.items[0].children[0].type == TexTalkNodeType.Identifier) {
                builder.append(sub.parameters.items[0].children[0].toCode(interceptor))
            } else {
                builder.append(sub.toCode(interceptor))
            }
        }

        if (sup != null) {
            builder.append("^")
            if (sup.parameters.items.size == 1 &&
                sup.parameters.items[0].children.size == 1 &&
                sup.parameters.items[0].children[0].type == TexTalkNodeType.Identifier) {
                builder.append(sup.parameters.items[0].children[0].toCode(interceptor))
            } else {
                builder.append(sup.toCode(interceptor))
            }
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

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode) =
        transformer(
            SubSupTexTalkNode(
                sub = sub?.transform(transformer) as GroupTexTalkNode?,
                sup = sup?.transform(transformer) as GroupTexTalkNode?))
}

data class TextTexTalkNode(
    override val type: TexTalkNodeType,
    val tokenType: TexTalkTokenType,
    val text: String,
    val isVarArg: Boolean
) : TexTalkNode {

    override fun toCode(interceptor: (node: TexTalkNode) -> String?): String {
        val res = interceptor(this)
        if (res != null) {
            return res
        }

        return text +
            if (isVarArg) {
                "..."
            } else {
                ""
            }
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {}

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode) =
        transformer(this)
}

data class OperatorTexTalkNode(
    val lhs: TexTalkNode?, val command: TexTalkNode, val rhs: TexTalkNode?
) : TexTalkNode {

    override val type: TexTalkNodeType
        get() = TexTalkNodeType.Operator

    override fun toCode(interceptor: (node: TexTalkNode) -> String?): String {
        val res = interceptor(this)
        if (res != null) {
            return res
        }

        val builder = StringBuilder()
        if (lhs != null) {
            builder.append(lhs.toCode(interceptor))
            if (rhs != null) {
                // it is a infix operator so put spaces between
                // the operator and operands
                builder.append(' ')
            }
        }

        builder.append(command.toCode(interceptor))

        if (rhs != null) {
            if (lhs != null) {
                // it is a infix operator so put spaces between
                // the operator and operands
                builder.append(' ')
            }
            builder.append(rhs.toCode(interceptor))
        }

        return builder.toString()
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {
        if (lhs != null) {
            fn(lhs)
        }
        fn(command)
        if (rhs != null) {
            fn(rhs)
        }
    }

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode) =
        transformer(
            OperatorTexTalkNode(
                lhs = lhs?.transform(transformer),
                command = command.transform(transformer),
                rhs = rhs?.transform(transformer)))
}

data class TexTalkToken(
    val text: String, val tokenType: TexTalkTokenType, val row: Int, val column: Int
) : TexTalkNode {

    override val type: TexTalkNodeType
        get() = TexTalkNodeType.Token

    override fun toCode(interceptor: (node: TexTalkNode) -> String?): String {
        val res = interceptor(this)
        if (res != null) {
            return res
        }

        return text
    }

    override fun forEach(fn: (texTalkNode: TexTalkNode) -> Unit) {}

    override fun transform(transformer: (texTalkNode: TexTalkNode) -> TexTalkNode) =
        transformer(this)
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
    ColonColonEquals,
    Is,
    In,
    DotDotDot,
    Invalid
}

fun getTexTalkAncestry(root: TexTalkNode, node: TexTalkNode): List<TexTalkNode> {
    val path = mutableListOf<TexTalkNode>()
    getTexTalkAncestryImpl(root, node, path)
    // 'node' itself shouldn't be in the ancestry
    if (path.isNotEmpty()) {
        path.removeAt(path.size - 1)
    }
    return path.reversed()
}

private fun getTexTalkAncestryImpl(
    root: TexTalkNode, node: TexTalkNode, path: MutableList<TexTalkNode>
) {
    if (root == node) {
        path.add(node)
        return
    }

    path.add(root)
    root.forEach {
        if (path.isEmpty() || path.last() != node) {
            getTexTalkAncestryImpl(it, node, path)
        }
    }
    if (path.isEmpty() || path.last() != node) {
        path.removeAt(path.size - 1)
    }
}

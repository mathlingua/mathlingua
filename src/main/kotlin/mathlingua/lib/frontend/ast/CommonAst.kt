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

package mathlingua.lib.frontend.ast

import java.lang.StringBuilder
import mathlingua.lib.frontend.MetaData

internal sealed interface CommonNode : ChalkTalkNode, TexTalkNode, NodeLexerToken

internal interface NameOrFunction : CommonNode

internal sealed interface NameOrVariadicName : CommonNode

// names
internal data class Name(val text: String, override val metadata: MetaData) :
    SquareTargetItem,
    NameOrNameAssignment,
    NameAssignmentItem,
    NameOrFunction,
    NameOrCommand,
    NameOrVariadicName,
    Expression,
    Target {
    override fun toCode() = text
}

// targets
internal sealed interface VariadicRhs : ChalkTalkNode, TexTalkNode

internal sealed interface VariadicIsRhs

internal sealed interface VariadicTarget : VariadicRhs

// <assignment> | <name> | <operator> | <tuple> | <sequence> | <function> | <set>
internal sealed interface Target : Argument

internal data class OperatorName(val text: String, override val metadata: MetaData) :
    Target, NameAssignmentItem, Operator {
    override fun toCode() = text
}

internal data class VariadicName(val name: Name, override val metadata: MetaData) :
    VariadicTarget, VariadicIsRhs, NameOrVariadicName {
    override fun toCode(): String = "$name..."
}

internal data class VariadicFunction(val function: FunctionCall, override val metadata: MetaData) :
    VariadicTarget {
    override fun toCode() = "${function.toCode()}..."
}

internal data class VariadicSequence(val sequence: Sequence, override val metadata: MetaData) :
    VariadicTarget {
    override fun toCode() = "${sequence.toCode()}..."
}

// functions
internal sealed interface FunctionCall : NameOrFunction, Expression

internal data class Function(
    val name: Name, val params: List<NameOrVariadicName>, override val metadata: MetaData
) : FunctionCall, NameOrFunction, Target, NameAssignmentItem, SquareTargetItem, Expression {
    override fun toCode(): String {
        val builder = StringBuilder()
        builder.append(name.toCode())
        builder.append("(")
        for (i in params.indices) {
            if (i > 0) {
                builder.append(", ")
            }
            builder.append(params[i].toCode())
        }
        builder.append(")")
        return builder.toString()
    }
}

internal data class SubParamCall(
    val name: Name, val subParams: List<NameOrVariadicName>, override val metadata: MetaData
) : FunctionCall {
    override fun toCode(): String {
        val builder = StringBuilder()
        builder.append(name.toCode())
        builder.append("_(")
        for (i in subParams.indices) {
            if (i > 0) {
                builder.append(", ")
            }
            builder.append(subParams[i].toCode())
        }
        builder.append(")")
        return builder.toString()
    }
}

internal data class SubAndRegularParamCall(
    val name: Name,
    val subParams: List<NameOrVariadicName>,
    val params: List<NameOrVariadicName>,
    override val metadata: MetaData
) : FunctionCall {
    override fun toCode(): String {
        val builder = StringBuilder()
        builder.append(name.toCode())
        builder.append("_(")
        for (i in subParams.indices) {
            if (i > 0) {
                builder.append(", ")
            }
            builder.append(subParams[i].toCode())
        }
        builder.append(")(")
        for (i in params.indices) {
            if (i > 0) {
                builder.append(", ")
            }
            builder.append(params[i].toCode())
        }
        builder.append(")")
        return builder.toString()
    }
}

// sequences
internal sealed interface Sequence : SquareTargetItem, NameAssignmentItem, Expression, Target

internal data class SubParamSequence(val func: SubParamCall, override val metadata: MetaData) :
    Sequence {
    override fun toCode(): String {
        val builder = StringBuilder()
        builder.append("{")
        builder.append(func.toCode())
        builder.append("}_(")
        for (i in func.subParams.indices) {
            if (i > 0) {
                builder.append(", ")
            }
            builder.append(func.subParams[i].toCode())
        }
        builder.append(")")
        return builder.toString()
    }
}

internal data class SubAndRegularParamSequence(
    val func: SubAndRegularParamCall, override val metadata: MetaData
) : Sequence {
    override fun toCode(): String {
        val builder = StringBuilder()
        builder.append("{")
        builder.append(func.toCode())
        builder.append("}_{")
        for (i in func.subParams.indices) {
            if (i > 0) {
                builder.append(", ")
            }
            builder.append(func.subParams[i].toCode())
        }
        builder.append("}")
        return builder.toString()
    }
}

// assignments
internal sealed interface Assignment : Target
// <name> | <operator> | <tuple> | <sequence> | <function> | <set>
internal sealed interface NameAssignmentItem : CommonNode

internal data class NameAssignment(
    val lhs: Name, val rhs: NameAssignmentItem, override val metadata: MetaData
) : Assignment, NameOrNameAssignment {
    override fun toCode(): String {
        val builder = StringBuilder()
        builder.append(lhs.toCode())
        builder.append(" := ")
        builder.append(rhs.toCode())
        return builder.toString()
    }
}

internal data class FunctionAssignment(
    val lhs: FunctionCall, val rhs: FunctionCall, override val metadata: MetaData
) : Assignment {
    override fun toCode(): String {
        val builder = StringBuilder()
        builder.append(lhs.toCode())
        builder.append(" := ")
        builder.append(rhs.toCode())
        return builder.toString()
    }
}

// <target> | <text> | <statement>
internal sealed interface Argument : CommonNode

// <name> | <tuple> | <sequence> | <function> | <set>
internal sealed interface SquareTargetItem : CommonNode

internal data class Tuple(val targets: List<Target>, override val metadata: MetaData) :
    SquareTargetItem, NameAssignmentItem, Target, Expression {
    override fun toCode(): String {
        val builder = StringBuilder()
        builder.append("(")
        for (i in targets.indices) {
            if (i > 0) {
                builder.append(", ")
            }
            builder.append(targets[i].toCode())
        }
        builder.append(")")
        return builder.toString()
    }
}

// <name> | <name assignment>
internal sealed interface NameOrNameAssignment : CommonNode

internal data class Set(val items: List<NameOrNameAssignment>, override val metadata: MetaData) :
    SquareTargetItem, NameAssignmentItem, Target, Expression {
    override fun toCode(): String {
        val builder = StringBuilder()
        builder.append("{")
        for (i in items.indices) {
            if (i > 0) {
                builder.append(", ")
            }
            builder.append(items[i].toCode())
        }
        builder.append("}")
        return builder.toString()
    }
}

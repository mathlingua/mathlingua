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

// names
internal data class Name(val text: String, override val metadata: MetaData) :
    Target, NameOrNameAssignment, NameAssignmentItem, NameOrFunction, NameOrCommand, Expression {
    override fun toCode() = text
}

internal data class OperatorName(val text: String, override val metadata: MetaData) :
    Target, NameAssignmentItem, NameOrFunction {
    override fun toCode() = text
}

internal data class NameParam(val name: Name, val isVarArgs: Boolean) {
    fun toCode(): String =
        if (isVarArgs) {
            "$name..."
        } else {
            name.text
        }
}

// functions
internal sealed interface Function : Target, NameAssignmentItem, NameOrFunction, Expression

internal data class RegularFunction(
    val name: Name, val params: List<NameParam>, override val metadata: MetaData
) : Function {
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

internal data class SubParamFunction(
    val name: Name, val subParams: List<NameParam>, override val metadata: MetaData
) : Function {
    override fun toCode(): String {
        val builder = StringBuilder()
        builder.append(name.toCode())
        builder.append("_{")
        for (i in subParams.indices) {
            if (i > 0) {
                builder.append(", ")
            }
            builder.append(subParams[i].toCode())
        }
        builder.append("}")
        return builder.toString()
    }
}

internal data class SubAndRegularParamFunction(
    val name: Name,
    val subParams: List<NameParam>,
    val params: List<NameParam>,
    override val metadata: MetaData
) : Function {
    override fun toCode(): String {
        val builder = StringBuilder()
        builder.append(name.toCode())
        builder.append("_{")
        for (i in subParams.indices) {
            if (i > 0) {
                builder.append(", ")
            }
            builder.append(subParams[i].toCode())
        }
        builder.append("}")
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

// sequences
internal sealed interface Sequence : Target, NameAssignmentItem, Expression

internal data class SubParamFunctionSequence(
    val func: SubParamFunction, override val metadata: MetaData
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

internal data class SubAndRegularParamFunctionSequence(
    val func: SubAndRegularParamFunction, override val metadata: MetaData
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
    val lhs: Function, val rhs: Function, override val metadata: MetaData
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

// targets
// <assignment> | <name> | <operator> | <tuple> | <sequence> | <function> | <set>
internal sealed interface Target : Argument

internal data class Tuple(val targets: List<Target>, override val metadata: MetaData) :
    Target, NameAssignmentItem, Expression {
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
    Target, NameAssignmentItem, Expression {
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

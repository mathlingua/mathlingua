package mathlingua.lib.frontend.ast

import mathlingua.lib.frontend.MetaData

internal sealed interface CommonNode : ChalkTalkNode, TexTalkNode

internal interface NameOrFunction : CommonNode

// names
internal data class Name(val text: String, override val metadata: MetaData) :
    Target, NameOrNameAssignment, NameAssignmentItem, NameOrFunction

internal data class OperatorName(val text: String, override val metadata: MetaData) :
    Target, NameAssignmentItem, NameOrFunction

internal data class NameParam(val name: Name, val isVarArgs: Boolean)

// functions
internal sealed interface Function : Target, NameAssignmentItem, NameOrFunction

internal data class RegularFunction(
    val name: Name, val params: List<NameParam>, override val metadata: MetaData
) : Function

internal data class SubParamFunction(
    val name: Name, val subParams: List<NameParam>, override val metadata: MetaData
) : Function

internal data class SubAndRegularParamFunction(
    val name: Name,
    val subParams: List<NameParam>,
    val params: List<NameParam>,
    override val metadata: MetaData
) : Function

// sequences
internal sealed interface Sequence : Target, NameAssignmentItem

internal data class SubParamFunctionSequence(
    val func: SubParamFunction, override val metadata: MetaData
) : Sequence

internal data class SubAndRegularParamFunctionSequence(
    val func: SubAndRegularParamFunction,
    override val metadata: MetaData
) : Sequence

// assignments
internal sealed interface Assignment : Target
// <name> | <operator> | <tuple> | <sequence> | <function> | <set>
internal sealed interface NameAssignmentItem : CommonNode

internal data class NameAssignment(
    val lhs: Name, val rhs: NameAssignmentItem, override val metadata: MetaData
) : Assignment, NameOrNameAssignment

internal data class FunctionAssignment(
    val lhs: Function, val rhs: Function, override val metadata: MetaData
) : Assignment

// <target> | <text> | <statement>
internal sealed interface Argument : CommonNode

// targets
// <assignment> | <name> | <operator> | <tuple> | <sequence> | <function> | <set>
internal sealed interface Target : Argument

internal data class Tuple(val targets: List<Target>, override val metadata: MetaData) :
    Target, NameAssignmentItem

// <name> | <name assignment>
internal sealed interface NameOrNameAssignment : CommonNode

internal data class Set(val items: List<NameOrNameAssignment>, override val metadata: MetaData) :
    Target, NameAssignmentItem


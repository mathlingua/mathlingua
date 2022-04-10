package mathlingua.lib.frontend.chalktalk

import mathlingua.lib.frontend.HasMetadata
import mathlingua.lib.frontend.Metadata

internal sealed interface ChalkTalkNode : HasMetadata

// names
internal data class Name(val text: String, override val metadata: Metadata) : Target, NameOrNameAssignment, NameAssignmentItem
internal data class NameParam(val name: Name, val isVarArgs: Boolean)

// functions
internal sealed interface Function : Target, NameAssignmentItem
internal data class RegularFunction(val name: Name, val params: List<NameParam>, override val metadata: Metadata) : Function
internal data class SubParamFunction(val name: Name, val subParams: List<NameParam>, override val metadata: Metadata) : Function
internal data class SubAndRegularParamFunction(val name: Name, val subParams: List<NameParam>, val params: List<NameParam>,
    override val metadata: Metadata) : Function

// sequences
internal sealed interface Sequence : Target, NameAssignmentItem
internal data class SubParamFunctionSequence(val func: SubParamFunction, val subParams: List<NameParam>,
    override val metadata: Metadata) : Sequence
internal data class SubAndRegularParamFunctionSequence(val func: SubAndRegularParamFunction, val subParams: List<NameParam>,
    override val metadata: Metadata) : Sequence

// assignments
internal sealed interface Assignment : Target
// <name> | <tuple> | <sequence> | <function> | <set>
internal sealed interface NameAssignmentItem : ChalkTalkNode
internal data class NameAssignment(val lhs: Name, val rhs: NameAssignmentItem, override val metadata: Metadata) : Assignment, NameOrNameAssignment
internal data class FunctionAssignment(val lhs: Function, val rhs: Function, override val metadata: Metadata) : Assignment

// targets
// <assignment> | <name> | <tuple> | <sequence> | <function> | <set>
internal sealed interface Target : ChalkTalkNode

internal data class Tuple(val targets: List<Target>, override val metadata: Metadata) : Target, NameAssignmentItem

// <name> | <name assignment>
internal sealed interface NameOrNameAssignment : ChalkTalkNode
internal data class Set(val items: List<NameOrNameAssignment>, override val metadata: Metadata) : Target, NameAssignmentItem

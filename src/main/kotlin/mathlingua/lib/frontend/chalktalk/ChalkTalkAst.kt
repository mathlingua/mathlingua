package mathlingua.lib.frontend.chalktalk

import mathlingua.lib.frontend.HasMetaData
import mathlingua.lib.frontend.MetaData

internal sealed interface ChalkTalkNode : HasMetaData

// names
internal data class Name(val text: String, override val metadata: MetaData) : Target, NameOrNameAssignment, NameAssignmentItem
internal data class NameParam(val name: Name, val isVarArgs: Boolean)

// functions
internal sealed interface Function : Target, NameAssignmentItem
internal data class RegularFunction(val name: Name, val params: List<NameParam>, override val metadata: MetaData) : Function
internal data class SubParamFunction(val name: Name, val subParams: List<NameParam>, override val metadata: MetaData) : Function
internal data class SubAndRegularParamFunction(val name: Name, val subParams: List<NameParam>, val params: List<NameParam>,
    override val metadata: MetaData) : Function

// sequences
internal sealed interface Sequence : Target, NameAssignmentItem
internal data class SubParamFunctionSequence(val func: SubParamFunction, val subParams: List<NameParam>,
    override val metadata: MetaData) : Sequence
internal data class SubAndRegularParamFunctionSequence(val func: SubAndRegularParamFunction, val subParams: List<NameParam>,
    override val metadata: MetaData) : Sequence

// assignments
internal sealed interface Assignment : Target
// <name> | <tuple> | <sequence> | <function> | <set>
internal sealed interface NameAssignmentItem : ChalkTalkNode
internal data class NameAssignment(val lhs: Name, val rhs: NameAssignmentItem, override val metadata: MetaData) : Assignment, NameOrNameAssignment
internal data class FunctionAssignment(val lhs: Function, val rhs: Function, override val metadata: MetaData) : Assignment

// targets
// <assignment> | <name> | <tuple> | <sequence> | <function> | <set>
internal sealed interface Target : ChalkTalkNode

internal data class Tuple(val targets: List<Target>, override val metadata: MetaData) : Target, NameAssignmentItem

// <name> | <name assignment>
internal sealed interface NameOrNameAssignment : ChalkTalkNode
internal data class Set(val items: List<NameOrNameAssignment>, override val metadata: MetaData) : Target, NameAssignmentItem

internal data class TextBlock(val text: String, override val metadata: MetaData) : ChalkTalkNode

internal data class Id(val text: String, override val metadata: MetaData) : ChalkTalkNode

internal data class Statement(val text: String, override val metadata: MetaData) : ChalkTalkNode

internal data class Text(val text: String, override val metadata: MetaData) : ChalkTalkNode

internal sealed class Section(val name: String) : ChalkTalkNode

internal object BeginGroup : ChalkTalkNode {
    override val metadata = MetaData(row = -1, column = -1, isInline = false)
    override fun toString() = javaClass.simpleName
}

internal object EndGroup : ChalkTalkNode {
    override val metadata = MetaData(row = -1, column = -1, isInline = false)
    override fun toString() = javaClass.simpleName
}

internal data class BeginSection(val name: String) : ChalkTalkNode {
    override val metadata = MetaData(row = -1, column = -1, isInline = false)
    override fun toString() = javaClass.simpleName
}

internal object EndSection : ChalkTalkNode {
    override val metadata = MetaData(row = -1, column = -1, isInline = false)
    override fun toString() = javaClass.simpleName
}

internal object BeginArgument : ChalkTalkNode {
    override val metadata = MetaData(row = -1, column = -1, isInline = false)
    override fun toString() = javaClass.simpleName
}

internal object EndArgument : ChalkTalkNode {
    override val metadata = MetaData(row = -1, column = -1, isInline = false)
    override fun toString() = javaClass.simpleName
}

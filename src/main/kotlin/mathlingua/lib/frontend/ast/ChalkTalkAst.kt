package mathlingua.lib.frontend.ast

import mathlingua.lib.frontend.HasMetaData
import mathlingua.lib.frontend.MetaData

internal sealed interface ChalkTalkNode : HasMetaData

internal data class TextBlock(val text: String, override val metadata: MetaData) : ChalkTalkNode

internal data class Id(val text: String, override val metadata: MetaData) : ChalkTalkNode

internal data class Statement(val text: String, override val metadata: MetaData) : Argument

internal data class Text(val text: String, override val metadata: MetaData) : Argument

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

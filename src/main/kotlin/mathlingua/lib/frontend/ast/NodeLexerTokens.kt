package mathlingua.lib.frontend.ast

import mathlingua.lib.frontend.HasMetaData
import mathlingua.lib.frontend.MetaData

internal sealed interface NodeLexerToken : HasMetaData

internal data class BeginGroup(override val metadata: MetaData) : NodeLexerToken

internal object EndGroup : NodeLexerToken {
    override val metadata = MetaData(row = -1, column = -1, isInline = true)
    override fun toString() = javaClass.simpleName
}

internal data class BeginSection(val name: String, override val metadata: MetaData) :
    NodeLexerToken

internal object EndSection : NodeLexerToken {
    override val metadata = MetaData(row = -1, column = -1, isInline = true)
    override fun toString() = javaClass.simpleName
}

internal data class BeginArgument(override val metadata: MetaData) : NodeLexerToken

internal object EndArgument : NodeLexerToken {
    override val metadata = MetaData(row = -1, column = -1, isInline = true)
    override fun toString() = javaClass.simpleName
}

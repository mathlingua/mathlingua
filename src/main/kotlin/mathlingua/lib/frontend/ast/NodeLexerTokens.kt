package mathlingua.lib.frontend.ast

import mathlingua.lib.frontend.HasMetaData
import mathlingua.lib.frontend.MetaData

internal sealed interface NodeLexerToken : HasMetaData

internal object BeginGroup : NodeLexerToken {
    override val metadata = MetaData(row = -1, column = -1, isInline = true)
    override fun toString() = javaClass.simpleName
}

internal object EndGroup : NodeLexerToken {
    override val metadata = MetaData(row = -1, column = -1, isInline = true)
    override fun toString() = javaClass.simpleName
}

internal data class BeginSection(val name: String) : NodeLexerToken {
    override val metadata = MetaData(row = -1, column = -1, isInline = true)
}

internal object EndSection : NodeLexerToken {
    override val metadata = MetaData(row = -1, column = -1, isInline = true)
    override fun toString() = javaClass.simpleName
}

internal object BeginArgument : NodeLexerToken {
    override val metadata = MetaData(row = -1, column = -1, isInline = true)
    override fun toString() = javaClass.simpleName
}

internal object EndArgument : NodeLexerToken {
    override val metadata = MetaData(row = -1, column = -1, isInline = true)
    override fun toString() = javaClass.simpleName
}

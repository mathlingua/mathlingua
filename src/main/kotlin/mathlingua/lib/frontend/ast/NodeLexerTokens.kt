package mathlingua.lib.frontend.ast

sealed interface NodeLexerToken

internal object BeginGroup : NodeLexerToken {
    override fun toString() = javaClass.simpleName
}

internal object EndGroup : NodeLexerToken {
    override fun toString() = javaClass.simpleName
}

internal data class BeginSection(val name: String) : NodeLexerToken

internal object EndSection : NodeLexerToken {
    override fun toString() = javaClass.simpleName
}

internal object BeginArgument : NodeLexerToken {
    override fun toString() = javaClass.simpleName
}

internal object EndArgument : NodeLexerToken {
    override fun toString() = javaClass.simpleName
}

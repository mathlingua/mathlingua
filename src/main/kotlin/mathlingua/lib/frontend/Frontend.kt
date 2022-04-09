package mathlingua.lib.frontend

internal data class ParseError(val message: String, val row: Int, val column: Int)

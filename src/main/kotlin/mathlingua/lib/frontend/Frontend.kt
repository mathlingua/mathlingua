package mathlingua.lib.frontend

internal data class ParseError(val message: String, val row: Int, val column: Int)

data class Metadata(var row: Int, var column: Int, var isInline: Boolean)

internal interface HasMetadata {
    val metadata: Metadata
}

package mathlingua.lib.frontend

internal data class ParseError(val message: String, val row: Int, val column: Int)

internal data class MetaData(var row: Int, var column: Int, var isInline: Boolean)

internal interface HasMetaData {
    val metadata: MetaData
}

package mathlingua.jvm

import mathlingua.common.MathLingua
import mathlingua.common.transform.moveInlineCommandsToIsNode

object SignatureTestBed {
    @JvmStatic
    fun main(args: Array<String>) {
        val text = """
            [\continuous.function]
            Defines: f
            means:
            . '\something'
            
            
            Result:
            . 'a = \continuous.function'
            . 'b := \continuous.function'
        """.trimIndent()
        val result = MathLingua().parse(text)
        for (err in result.errors) {
            println(err)
        }

        println(text)
        println("----------------------------------------")

        val res = result.document!!.results[0]
        println(moveInlineCommandsToIsNode(res, setOf("\\continuous.function"), { true }, { true }).toCode(false, 0))
    }
}

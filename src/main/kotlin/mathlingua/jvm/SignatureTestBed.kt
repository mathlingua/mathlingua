package mathlingua.jvm

import mathlingua.common.MathLingua
import mathlingua.common.textalk.Command
import mathlingua.common.transform.getCommandSignature
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
            . 'c = \function'
        """.trimIndent()
        val result = MathLingua().parse(text)
        for (err in result.errors) {
            println(err)
        }

        println(text)
        println("----------------------------------------")

//        val res = result.document!!.results[0]
//        println(moveInlineCommandsToIsNode(result.document!!.defines, res, { true }, {
//            it is Command && getCommandSignature(it).toCode() == "\\continuous.function"
//        }).toCode(false, 0))
    }
}

package mathlingua.jvm

import mathlingua.common.MathLingua
import mathlingua.common.ValidationFailure

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
        if (result is ValidationFailure) {
            for (err in result.errors) {
                println(err)
            }
        }

        println(text)
        println("----------------------------------------")

//        val res = result.document!!.results[0]
//        println(moveInlineCommandsToIsNode(result.document!!.defines, res, { true }, {
//            it is Command && getCommandSignature(it).toCode() == "\\continuous.function"
//        }).toCode(false, 0))
    }
}

package mathlingua.jvm

import mathlingua.common.MathLingua
import mathlingua.common.transform.glueCommands
import mathlingua.common.transform.separateIsStatements

object SignatureTestBed {
    @JvmStatic
    fun main(args: Array<String>) {
        val text = """
            [\a \b]
            Defines: f
            means:
            . 'x, y is \a \b, \c'
        """.trimIndent()
        val result = MathLingua().parse(text)
        for (err in result.errors) {
            println(err)
        }

        println(text)
        println("----------------------------------------")

        val def = result.document!!.defines[0]
        println(glueCommands(separateIsStatements(def)).toCode(false, 0))
    }
}

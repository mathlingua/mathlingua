package mathlingua.jvm

import mathlingua.common.MathLingua
import mathlingua.common.transform.glueCommands
import mathlingua.common.transform.separateIsStatements

object SignatureTestBed {
    @JvmStatic
    fun main(args: Array<String>) {
        val text = """
            [\a \b]
            Represents:
            that:
            . 'x, y is \a \b, \c'
        """.trimIndent()
        val result = MathLingua().parse(text)
        for (err in result.errors) {
            println(err)
        }

        println(text)
        println("----------------------------------------")

        val rep = result.document!!.represents[0]
        println(glueCommands(separateIsStatements(rep)).toCode(false, 0))
    }
}

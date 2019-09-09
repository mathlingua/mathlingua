package mathlingua.jvm

import mathlingua.common.MathLingua
import mathlingua.common.chalktalk.phase2.Statement
import mathlingua.common.transform.glueCommands
import mathlingua.common.transform.separateIsStatements

object SignatureTestBed {
    @JvmStatic
    fun main(args: Array<String>) {
        val text = """
            Result:
            . 'x, y is \a \b, \c'
        """.trimIndent()
        val result = MathLingua().parse(text)
        for (err in result.errors) {
            println(err)
        }

        println(text)
        println("----------------------------------------")

        val res = result.document!!.results[0]
        val stmt = res.resultSection.clauses.clauses[0] as Statement
        println(glueCommands(separateIsStatements(res)).toCode(false, 0))
    }
}

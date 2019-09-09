package mathlingua.jvm

import mathlingua.common.MathLingua
import mathlingua.common.chalktalk.phase2.Statement
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
        """.trimIndent()
        val result = MathLingua().parse(text)
        for (err in result.errors) {
            println(err)
        }

        println(text)
        println("----------------------------------------")

        val res = result.document!!.results[0]
        val stmt = res.resultSection.clauses.clauses[0] as Statement
        println(moveInlineCommandsToIsNode(stmt, mapOf("\\continuous.function" to "Q"), { true }, { true }).toCode(false, 0))
    }
}

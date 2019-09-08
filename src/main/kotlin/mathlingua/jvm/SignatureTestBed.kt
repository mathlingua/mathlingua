package mathlingua.jvm

import mathlingua.common.MathLingua
import mathlingua.common.chalktalk.phase2.ClauseListNode
import mathlingua.common.chalktalk.phase2.DefinesGroup
import mathlingua.common.chalktalk.phase2.ResultGroup
import mathlingua.common.chalktalk.phase2.ResultSection
import mathlingua.common.transform.moveInlineCommandsToIsNode

object SignatureTestBed {
    @JvmStatic
    fun main(args: Array<String>) {
        val text = """
            [\A{x}]
            Defines: a
            assuming: 'x is \X'
            means: 'a + x = 0'


            Result:
            . for: y, z
              then:
              . '\A{y} + 1 = 0'
              . '\A{y} + 2 = 1'
        """.trimIndent()
        val result = MathLingua().parse(text)
        for (err in result.errors) {
            println(err)
        }

        println(text)
        println("----------------------------------------")

        val defMap = mutableMapOf<String, DefinesGroup>()
        for (def in result.document!!.defines) {
            val sig = def.signature
            if (sig != null) {
                defMap[sig] = def
            }
        }

        val res = result.document!!.results[0]

        println(ResultGroup(
            resultSection = ResultSection(
                clauses = ClauseListNode(
                    clauses = res.resultSection.clauses.clauses.map {
                        moveInlineCommandsToIsNode(it, mapOf("\\A{?}" to "Q"), { true }, { true })
                    }
                )
            ),
            aliasSection = null,
            metaDataSection = null
        ).toCode(false, 0))
    }
}

package mathlingua.jvm

import mathlingua.common.MathLingua
import mathlingua.common.chalktalk.phase2.DefinesGroup
import mathlingua.common.transform.replaceIsNodes

object SignatureTestBed {
    @JvmStatic
    fun main(args: Array<String>) {
        val text = """
            [\continuous.function:on{X}]
            Defines: f
            assuming: 'f is \function'
            means:
            . for: x
              where: 'x \in X'
              then:
              . '\limit[t]_{x}{f(t)} = f(x)'


            Result:
            . for: g, A
              where:
              . 'A is \set'
              . 'g is \differentiable.function:on{A}'
              then:
              . 'g is \continuous.function:on{A}'
        """.trimIndent()
        val result = MathLingua().parse(text)
        for (err in result.errors) {
            println(err)
        }

        println(text)
        println("----------------------------------------")

        val defMap = mutableMapOf<String, DefinesGroup>()
        val defs = result.document!!.defines
        for (def in defs) {
            val sig = def.signature
            if (sig != null) {
                defMap[sig] = def
            }
        }

        val res = result.document!!.results[0]

        var count = 1
        fun nextVar(): String {
            return "#${count++}"
        }

        println(replaceIsNodes(res, defs) { true }.toCode(false, 0))
    }
}

package mathlingua.jvm

import mathlingua.common.MathLingua
import mathlingua.common.chalktalk.phase2.Statement
import mathlingua.common.textalk.Command
import mathlingua.common.textalk.CommandPart
import mathlingua.common.textalk.ExpressionTexTalkNode
import mathlingua.common.textalk.IsTexTalkNode
import mathlingua.common.textalk.TexTalkNode
import mathlingua.common.transform.glueCommands

object SignatureTestBed {
    @JvmStatic
    fun main(args: Array<String>) {
        val text = """
            Result:
            . 'x is \compact \set'
        """.trimIndent()
        val result = MathLingua().parse(text)
        for (err in result.errors) {
            println(err)
        }

        println(text)
        println("----------------------------------------")

        val res = result.document!!.results[0]
        val stmt = res.resultSection.clauses.clauses[0] as Statement
        val root = stmt.texTalkRoot.value!!
        val isNode = root.children[0] as IsTexTalkNode
        val rhsParameters = isNode.rhs
        val param = rhsParameters.items[0]
        println(glueCommands(param).toCode())
    }
}

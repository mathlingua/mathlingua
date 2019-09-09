package mathlingua.jvm

import mathlingua.common.MathLingua
import mathlingua.common.chalktalk.phase2.Statement
import mathlingua.common.textalk.Command
import mathlingua.common.textalk.TexTalkNode
import mathlingua.common.textalk.getAncestry
import mathlingua.common.transform.findCommands

object SignatureTestBed {
    @JvmStatic
    fun main(args: Array<String>) {
        val text = """
            [\a \b]
            Represents:
            that:
            . '\command:in{x + \some{\target}}'
        """.trimIndent()
        val result = MathLingua().parse(text)
        for (err in result.errors) {
            println(err)
        }

        println(text)
        println("----------------------------------------")

        val rep = result.document!!.represents[0]
        val stmt = rep.thatSection.clauses.clauses[0] as Statement
        val root = stmt.texTalkRoot.value!!
        val commands = findCommands(root)
        lateinit var target: TexTalkNode
        for (c in commands) {
            if (c.parts[0].name.text == "target") {
                target = c
            }
        }
        var i = 0
        for (item in getAncestry(root, target)) {
            println("${i++}: " + item)
        }
    }
}

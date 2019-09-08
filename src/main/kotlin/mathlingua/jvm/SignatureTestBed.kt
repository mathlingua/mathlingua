package mathlingua.jvm

/*
import mathlingua.common.MathLingua
import mathlingua.common.Validation
import mathlingua.common.chalktalk.phase2.DefinesGroup
import mathlingua.common.chalktalk.phase2.ForGroup
import mathlingua.common.chalktalk.phase2.ForSection
import mathlingua.common.chalktalk.phase2.Identifier
import mathlingua.common.chalktalk.phase2.Phase2Node
import mathlingua.common.chalktalk.phase2.Statement
import mathlingua.common.chalktalk.phase2.ThenSection
import mathlingua.common.chalktalk.phase2.WhereSection
import mathlingua.common.chalktalk.phase2.getCommandSignature
import mathlingua.common.textalk.Command
import mathlingua.common.textalk.ExpressionTexTalkNode
import mathlingua.common.textalk.TexTalkNode

var count = 1
fun nextVar(): String {
    return "\$${count++}"
}

object SignatureTestBed {
    @JvmStatic
    fun main(args: Array<String>) {
        val text = """
            [\A]
            Defines: a
            assuming: 'a is \someA'
            means:
            . 'a is \someAA'

            [\B]
            Defines: b
            assuming: 'b is \someB'
            means:
            . 'b is \someBB'

            Result:
            . for: x, y
              where:
              . 'x is \A'
              . 'y is \B'
              then:
              . '\A'
              . '\B'
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

        fun __chalkTransformer(node: Phase2Node): Phase2Node {
            if (node !is Statement) {
                return node
            }

            val validation = node.texTalkRoot
            if (!validation.isSuccessful) {
                return node
            }

            val root = validation.value!!
            val commands = findCommands(root)
            val signatures = commands.map {
                getCommandSignature(it).toCode()
            }.filter {
                defMap.containsKey(it)
            }

            val sigToReplacement = mutableMapOf<String, String>()
            for (sig in signatures) {
                sigToReplacement[sig] = nextVar()
            }

            val newRoot = replaceCommands(root, sigToReplacement)
            return ForGroup(
                forSection = ForSection(
                    targets = sigToReplacement.values.map { Identifier(name = it) }
                ),
                whereSection = WhereSection(
                    clauses = commands.filter {
                        sigToReplacement.containsKey(getCommandSignature(it).toCode())
                    }.map {
                        val sig = getCommandSignature(it).toCode()
                        val def = defMap[sig]!!
                        val directVars = getDefinesDirectVars(def)
                        val indirectVars = getDefinesIndirectVars(def)
                        val cmdVars = getVars(it)

                        val nameMap = mutableMapOf<String, String>()
                        for (v in directVars) {
                            nameMap[v] = nextVar()
                        }

                        assert(indirectVars.size == cmdVars.size)

                        for (i in indirectVars.indices) {
                            nameMap[indirectVars[i]] = cmdVars[i]
                        }

                        val newDef = renameVars(def, nameMap) as DefinesGroup
                        buildIfForDef(newDef)
                    }
                ),
                thenSection = ThenSection(
                    clauses = listOf(
                        Statement(
                            text = newRoot.toCode(),
                            texTalkRoot = Validation.success(newRoot as ExpressionTexTalkNode)
                        )
                    )
                )
            )
        }

        fun chalkTransformer(node: Phase2Node): Phase2Node {
            return if (node is Statement) {
                if (node.texTalkRoot.isSuccessful &&
                    node.texTalkRoot.value!!.children.size == 1 &&
                    node.texTalkRoot.value.children[0] is Command) {
                    val cmd = node.texTalkRoot.value.children[0]
                    val sig = getCommandSignature(cmd as Command).toCode()
                    if (defMap.containsKey(sig)) {
                        val def = defMap[sig]
                        val cmdVars = getVars(cmd)
                        val defIndirectVars = getDefinesIndirectVars(def!!)
                        val defDirectVars = getDefinesDirectVars(def)

                        val map = mutableMapOf<String, String>()
                        for (i in cmdVars.indices) {
                            map[defIndirectVars[i]] = cmdVars[i]
                        }

                        for (v in defDirectVars) {
                            map[v] = nextVar()
                        }

                        val ifGroup = buildIfForDef(def)
                        renameVars(ForGroup(
                            forSection = ForSection(
                                defDirectVars.map { Identifier(name = it) }
                            ),
                            whereSection = null,
                            thenSection = ThenSection(
                                listOf(ifGroup)
                            )
                        ), map)
                    } else {
                        node
                    }
                } else {
                    node
                }
            } else {
                node
            }
        }

        fun texTransformer(texTalkNode: TexTalkNode): TexTalkNode {
            return texTalkNode
        }

        val res = result.document.results[0]
        println(res.transform(::chalkTransformer, ::texTransformer).toCode(false, 0))
    }
}
*/

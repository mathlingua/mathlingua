package mathlingua.jvm

import mathlingua.common.MathLingua
import mathlingua.common.Validation
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase1.ast.Phase1Token
import mathlingua.common.chalktalk.phase2.AbstractionNode
import mathlingua.common.chalktalk.phase2.AggregateNode
import mathlingua.common.chalktalk.phase2.AssignmentNode
import mathlingua.common.chalktalk.phase2.DefinesGroup
import mathlingua.common.chalktalk.phase2.ForGroup
import mathlingua.common.chalktalk.phase2.ForSection
import mathlingua.common.chalktalk.phase2.Identifier
import mathlingua.common.chalktalk.phase2.IfGroup
import mathlingua.common.chalktalk.phase2.IfSection
import mathlingua.common.chalktalk.phase2.MeansSection
import mathlingua.common.chalktalk.phase2.Phase2Node
import mathlingua.common.chalktalk.phase2.Statement
import mathlingua.common.chalktalk.phase2.ThenSection
import mathlingua.common.chalktalk.phase2.TupleNode
import mathlingua.common.chalktalk.phase2.WhereSection
import mathlingua.common.chalktalk.phase2.getCommandSignature
import mathlingua.common.textalk.Command
import mathlingua.common.textalk.ExpressionTexTalkNode
import mathlingua.common.textalk.TexTalkNode
import mathlingua.common.textalk.TexTalkNodeType
import mathlingua.common.textalk.ParametersTexTalkNode
import mathlingua.common.textalk.TextTexTalkNode

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

fun replaceCommands(texTalkNode: TexTalkNode, sigToReplacement: Map<String, String>): TexTalkNode {
    return texTalkNode.transform {
        if (it !is Command) {
            it
        } else {
            val sig = getCommandSignature(it).toCode()
            if (!sigToReplacement.containsKey(sig)) {
                it
            } else {
                val name = sigToReplacement[sig]
                TextTexTalkNode(type = TexTalkNodeType.Identifier, text = name!!)
            }
        }
    }
}

fun findCommands(texTalkNode: TexTalkNode): List<Command> {
    val commands = mutableListOf<Command>()
    findCommandsImpl(texTalkNode, commands)
    return commands.distinct()
}

private fun findCommandsImpl(texTalkNode: TexTalkNode, commands: MutableList<Command>) {
    if (texTalkNode is Command) {
        commands.add(texTalkNode)
    }

    texTalkNode.forEach { findCommandsImpl(it, commands) }
}


fun replaceSignature(texTalkNode: TexTalkNode, signature: String, replacement: String): TexTalkNode {
    return texTalkNode.transform {
        if (it is Command && getCommandSignature(it).toCode() == signature) {
            TextTexTalkNode(type = TexTalkNodeType.Identifier, text = replacement)
        } else {
            texTalkNode
        }
    }
}

fun renameVars(root: Phase2Node, map: Map<String, String>): Phase2Node {
    fun chalkTransformer(node: Phase2Node): Phase2Node {
        return if (node is Identifier) {
            node.copy(name = map.getOrDefault(node.name, node.name))
        } else if (node is Statement) {
            if (node.texTalkRoot.isSuccessful) {
                val root = renameVars(node.texTalkRoot.value!!, map) as ExpressionTexTalkNode
                Statement(
                    text = root.toCode(),
                    texTalkRoot = Validation.success(root)
                )
            } else {
                node
            }
        } else {
            node
        }
    }

    fun texTransformer(texTalkNode: TexTalkNode): TexTalkNode {
        return if (texTalkNode is TextTexTalkNode) {
            texTalkNode.copy(text = map.getOrDefault(texTalkNode.text, texTalkNode.text))
        } else {
            texTalkNode
        }
    }

    return root.transform(::chalkTransformer, ::texTransformer)
}

fun renameVars(texTalkNode: TexTalkNode, map: Map<String, String>): TexTalkNode {
    return texTalkNode.transform {
        if (it is TextTexTalkNode) {
            it.copy(text = map.getOrDefault(it.text, it.text))
        } else {
            it
        }
    }
}

fun getDefinesDirectVars(def: DefinesGroup): List<String> {
    val vars = mutableListOf<String>()
    for (target in def.definesSection.targets) {
        getVarsImpl(target, vars)
    }
    return vars
}

fun getDefinesIndirectVars(def: DefinesGroup): List<String> {
    val vars = mutableListOf<String>()
    if (def.id.texTalkRoot.isSuccessful) {
        getDefinesIndirectVarsImpl(def.id.texTalkRoot.value!!, vars, false)
    }
    return vars
}

private fun getVars(texTalkNode: TexTalkNode): List<String> {
    val vars = mutableListOf<String>()
    getDefinesIndirectVarsImpl(texTalkNode, vars, false)
    return vars
}

private fun getDefinesIndirectVarsImpl(texTalkNode: TexTalkNode, vars: MutableList<String>, inParams: Boolean) {
    if (inParams && texTalkNode is TextTexTalkNode) {
        vars.add(texTalkNode.text)
    } else if (texTalkNode is ParametersTexTalkNode) {
        texTalkNode.forEach { getDefinesIndirectVarsImpl(it, vars, true) }
    } else {
        texTalkNode.forEach { getDefinesIndirectVarsImpl(it, vars, inParams) }
    }
}

private fun getVarsImpl(node: Phase2Node, vars: MutableList<String>) {
    if (node is Identifier) {
        vars.add(node.name)
    } else if (node is TupleNode) {
        getVarsImpl(node.tuple, vars)
    } else if (node is AggregateNode) {
        getVarsImpl(node.aggregate, vars)
    } else if (node is AbstractionNode) {
        getVarsImpl(node.abstraction, vars)
    } else if (node is AssignmentNode) {
        vars.add(node.assignment.lhs.text)
        getVarsImpl(node.assignment.rhs, vars)
    } else {
        node.forEach { getVarsImpl(it, vars) }
    }
}

private fun getVars(node: Phase1Node): List<String> {
    val vars = mutableListOf<String>()
    getVarsImpl(node, vars)
    return vars
}

private fun getVarsImpl(node: Phase1Node, vars: MutableList<String>) {
    if (node is Phase1Token) {
        vars.add(node.text)
    } else {
        node.forEach { getVarsImpl(it, vars) }
    }
}

fun buildIfForDef(def: DefinesGroup): IfGroup {
    return IfGroup(
        ifSection = IfSection(
            clauses = def.assumingSection?.clauses ?: emptyList()
        ),
        thenSection = ThenSection(
            clauses = def.meansSection.clauses
        )
    )
}

fun canonicalForm(def: DefinesGroup): DefinesGroup {
    return DefinesGroup(
        signature = def.signature,
        id = def.id,
        definesSection = def.definesSection,
        assumingSection = null,
        meansSection = MeansSection(
            clauses = listOf(buildIfForDef(def))
        ),
        aliasSection = def.aliasSection,
        metaDataSection = def.metaDataSection
    )
}

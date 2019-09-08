package mathlingua.jvm

import mathlingua.common.MathLingua
import mathlingua.common.Validation
import mathlingua.common.chalktalk.phase1.ast.ChalkTalkNode
import mathlingua.common.chalktalk.phase1.ast.ChalkTalkToken
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
import mathlingua.common.chalktalk.phase2.getCommandSignature
import mathlingua.common.textalk.Command
import mathlingua.common.textalk.ExpressionNode
import mathlingua.common.textalk.Node
import mathlingua.common.textalk.ParametersNode
import mathlingua.common.textalk.TextNode

var count = 1
fun nextVar(): String {
    return "\$${count++}"
}

object SignatureTestBed {
    @JvmStatic
    fun main(args: Array<String>) {
        val text = """
            [\function:on{A}]
            Defines: f
            assuming: 'A is \set'
            means:
            . 'f is \mapping:on{A}'

            Result:
            . for: g, B
              where: 'g is \function:on{B}'
              then:
              . '\function:on{B}'
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

        fun texTransformer(node: Node): Node {
            return node
        }

        val res = result.document.results[0]
        println(res.transform(::chalkTransformer, ::texTransformer).toCode(false, 0))
    }
}

fun renameVars(root: Phase2Node, map: Map<String, String>): Phase2Node {
    fun chalkTransformer(node: Phase2Node): Phase2Node {
        return if (node is Identifier) {
            node.copy(name = map.getOrDefault(node.name, node.name))
        } else if (node is Statement) {
            if (node.texTalkRoot.isSuccessful) {
                val root = renameVars(node.texTalkRoot.value!!, map) as ExpressionNode
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

    fun texTransformer(node: Node): Node {
        return if (node is TextNode) {
            node.copy(text = map.getOrDefault(node.text, node.text))
        } else {
            node
        }
    }

    return root.transform(::chalkTransformer, ::texTransformer)
}

fun renameVars(node: Node, map: Map<String, String>): Node {
    return node.transform {
        if (it is TextNode) {
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

private fun getVars(node: Node): List<String> {
    val vars = mutableListOf<String>()
    getDefinesIndirectVarsImpl(node, vars, false)
    return vars
}

private fun getDefinesIndirectVarsImpl(node: Node, vars: MutableList<String>, inParams: Boolean) {
    if (inParams && node is TextNode) {
        vars.add(node.text)
    } else if (node is ParametersNode) {
        node.forEach { getDefinesIndirectVarsImpl(it, vars, true) }
    } else {
        node.forEach { getDefinesIndirectVarsImpl(it, vars, inParams) }
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

private fun getVars(node: ChalkTalkNode): List<String> {
    val vars = mutableListOf<String>()
    getVarsImpl(node, vars)
    return vars
}

private fun getVarsImpl(node: ChalkTalkNode, vars: MutableList<String>) {
    if (node is ChalkTalkToken) {
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

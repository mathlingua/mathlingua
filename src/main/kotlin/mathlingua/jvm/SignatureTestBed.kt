package mathlingua.jvm

import mathlingua.common.MathLingua
import mathlingua.common.chalktalk.phase1.ast.ChalkTalkNode
import mathlingua.common.chalktalk.phase1.ast.ChalkTalkToken
import mathlingua.common.chalktalk.phase2.AbstractionNode
import mathlingua.common.chalktalk.phase2.AggregateNode
import mathlingua.common.chalktalk.phase2.AssignmentNode
import mathlingua.common.chalktalk.phase2.DefinesGroup
import mathlingua.common.chalktalk.phase2.Identifier
import mathlingua.common.chalktalk.phase2.IfGroup
import mathlingua.common.chalktalk.phase2.IfSection
import mathlingua.common.chalktalk.phase2.MeansSection
import mathlingua.common.chalktalk.phase2.Phase2Node
import mathlingua.common.chalktalk.phase2.ThenSection
import mathlingua.common.chalktalk.phase2.TupleNode
import mathlingua.common.textalk.Node
import mathlingua.common.textalk.ParametersNode
import mathlingua.common.textalk.TextNode

object SignatureTestBed {
    @JvmStatic
    fun main(args: Array<String>) {
        val text = """
            [\something{y}]
            Defines: x
            assuming:
            . 'y is \A'
            . 'y is \B'
            . '\somethingElse'
            means:
            . 'x is \B'
            . '\somethingMore'
            . 'x + y = 0'


            Result:
            . 'x is \something'
            . '\notSomething'
            . '\something'
            . 'x + \something'
        """.trimIndent()
        val result = MathLingua().parse(text)
        for (err in result.errors) {
            println(err)
        }

        val def = result.document!!.defines[0]
        println(canonicalForm(def).toCode(false, 0))

        println("Direct vars:")
        println(getDefinesDirectVars(def))

        println("Indirect vars:")
        println(getDefinesIndirectVars(def))

        println(def.renameVars(mapOf(
            "x" to "X",
            "y" to "Y"
        )).toCode(false, 0))
    }
}

var count = 0
fun nextVar(): String {
    return "var${count++}"
}

fun getDefinesDirectVars(def: DefinesGroup): Set<String> {
    val vars = mutableSetOf<String>()
    for (target in def.definesSection.targets) {
        getVarsImpl(target, vars)
    }
    return vars
}

fun getDefinesIndirectVars(def: DefinesGroup): Set<String> {
    val vars = mutableSetOf<String>()
    if (def.id.texTalkRoot.isSuccessful) {
        getDefinesIndirectVarsImpl(def.id.texTalkRoot.value!!, vars, false)
    }
    return vars
}

private fun getDefinesIndirectVarsImpl(node: Node, vars: MutableSet<String>, inParams: Boolean) {
    if (inParams && node is TextNode) {
        vars.add(node.text)
    } else if (node is ParametersNode) {
        node.forEach { getDefinesIndirectVarsImpl(it, vars, true) }
    } else {
        node.forEach { getDefinesIndirectVarsImpl(it, vars, inParams) }
    }
}

private fun getVarsImpl(node: Phase2Node, vars: MutableSet<String>) {
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

private fun getVarsImpl(node: ChalkTalkNode, vars: MutableSet<String>) {
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

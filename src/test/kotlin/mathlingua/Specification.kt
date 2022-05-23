package mathlingua

import mathlingua.lib.util.bold
import mathlingua.lib.util.red
import org.reflections.Reflections
import org.reflections.scanners.Scanners

private sealed interface Form

private enum class SequenceConstraint {
    ZeroOrMore,
    OneOrMore
}

private data class Literal(val of: String) : Form

private data class Regex(val of: String) : Form

private data class AnyOf(val of: List<String>) : Form

private data class Sequence(
    val of: String, val separator: String, val constraint: SequenceConstraint
) : Form

private data class ZeroOrMore(val of: Form) : Form

private data class OneOrMore(val of: Form) : Form

private data class Optionally(val of: Form) : Form

private data class Item(val of: List<Form>) : Form

private data class DefinitionOf(val name: String, val of: Form)

private data class Def(val name: String) : Form

private val COMMON_SPECIFICATION =
    listOf(
        DefinitionOf("Name", Regex("""[a-zA-Z0-9'"`]+("_"[a-zA-Z0-9'"`]+)?""")),
        DefinitionOf("OperatorName", Regex("""[~!@#${'$'}%^&*-+=|<>?'`"]+("_"[a-zA-Z0-9'"`]+)?""")),
        DefinitionOf(
            "NameAssignmentItem",
            anyOf("Name", "OperatorName", "Tuple", "Sequence", "Function", "Set")),
        DefinitionOf(
            "NameAssignment", items(Def("Name"), Literal(":="), Def("NameAssignmentItem"))),
        DefinitionOf("FunctionAssignment", items(Def("Function"), Literal(":="), Def("Function"))),
        DefinitionOf("Assignment", anyOf("NameAssignment", "FunctionAssignment")),
        DefinitionOf("VariadicName", items(Def("Name"), Optionally(Literal("...")))),
        DefinitionOf(
            "Function",
            items(
                Def("Name"),
                Literal("("),
                Sequence(
                    of = "VariadicName",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore),
                Literal(")"))),
        DefinitionOf(
            "SubParamCall",
            items(
                Def("Name"),
                Literal("_"),
                Literal("("),
                Sequence(
                    of = "VariadicName",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore),
                Literal(")"))),
        DefinitionOf(
            "SubAndRegularParamCall",
            items(
                Def("Name"),
                Literal("_"),
                Literal("("),
                Sequence(
                    of = "VariadicName",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore),
                Literal(")"),
                Literal("("),
                Sequence(
                    of = "VariadicName",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore),
                Literal(")"))),
        DefinitionOf("FunctionCall", anyOf("Function", "SubParamCall", "SubAndRegularParamCall")),
        DefinitionOf(
            "SubParamSequence",
            items(
                Literal("{"),
                Def("SubParamCall"),
                Literal("}"),
                Literal("_"),
                Literal("("),
                Sequence(
                    of = "VariadicName",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore),
                Literal(")"))),
        DefinitionOf(
            "SubAndRegularParamSequence",
            items(
                Literal("{"),
                Def("SubAndRegularParamCall"),
                Literal("}"),
                Literal("_"),
                Literal("("),
                Sequence(
                    of = "VariadicName",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore),
                Literal(")"))),
        DefinitionOf(
            "Sequence", anyOf("SubParamFunctionSequence", "SubAndRegularParamFunctionSequence")),
        DefinitionOf(
            "Tuple",
            items(
                Literal("("),
                Sequence(of = "Target", separator = ",", constraint = SequenceConstraint.OneOrMore),
                Literal(")"))),
        DefinitionOf("NameOrNameAssignment", anyOf("Name", "NameAssignment")),
        DefinitionOf(
            "Set",
            items(
                Literal("{"),
                Sequence(
                    of = "NameOrNameAssignment",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore),
                Literal("}"))),
        DefinitionOf(
            "Target",
            anyOf("Assignment", "Name", "OperatorName", "Tuple", "Sequence", "Function", "Set")))

private fun anyOf(vararg of: String) = AnyOf(of.toList())

private fun items(vararg of: Form) = Item(of.toList())

val AST_PACKAGE = "mathlingua.lib.frontend.ast"

private fun String.addAstPackagePrefix() =
    if (this.startsWith(AST_PACKAGE)) {
        this
    } else {
        "${AST_PACKAGE}.${this}"
    }

private fun String.removeAstPackagePrefix() =
    if (this.startsWith(AST_PACKAGE)) {
        // an additional character is removed to account
        // for the period after the package prefix
        this.substring(AST_PACKAGE.length + 1)
    } else {
        this
    }

fun verifySpec() {
    val reflections = Reflections(AST_PACKAGE, Scanners.values())
    val allTypesInCode =
        reflections.getAll(Scanners.SubTypes).filter {
            // only find mathlingua types
            it.contains(AST_PACKAGE) &&
                // ignore types associated with tests (tests make a class
                // for each test of the form <test-class>$<test-name>)
                !it.contains("$")
        }

    val allTypesInSpec = COMMON_SPECIFICATION.map { it.name.addAstPackagePrefix() }

    println(bold("Analyzing items declared in the spec but not in code:"))
    var notInCodeCount = 0
    for (t in allTypesInSpec) {
        if (t !in allTypesInCode) {
            notInCodeCount++
            println(
                "${red(bold("ERROR: "))} ${bold(t.removeAstPackagePrefix())} is declared in the spec but not defined in code")
        }
    }
    println("Found $notInCodeCount errors")
    println()

    /*
    println(bold("Analyzing items declared in the code but not in the spec:"))
    var notInSpecCount = 0
    for (t in allTypesInCode) {
        if (t !in allTypesInSpec) {
            notInSpecCount++
            println("${red(bold("ERROR: "))} ${bold(t.removeAstPackagePrefix())} is declared in code but not defined in the spec")
        }
    }
    println("Found $notInSpecCount errors")
    println()
     */

    val typeToCodeDirectImplementors = mutableMapOf<String, Set<String>>()
    val loader = ClassLoader.getSystemClassLoader()
    for (targetType in allTypesInCode) {
        val targetClass = loader.loadClass(targetType)
        if (targetClass.isInterface) {
            val result = mutableSetOf<String>()
            for (possibleImplementor in allTypesInCode) {
                val possibleImplementorClass = loader.loadClass(possibleImplementor)
                val interfaces = possibleImplementorClass.interfaces.map { it.name }.toSet()
                if (targetType in interfaces) {
                    result.add(possibleImplementor)
                }
            }
            typeToCodeDirectImplementors[targetType] = result
        }
    }

    val typeToSpecAnyOf = mutableMapOf<String, Set<String>>()
    for (def in COMMON_SPECIFICATION) {
        if (def.of is AnyOf) {
            typeToSpecAnyOf[def.name] = def.of.of.toSet()
        }
    }

    println(bold("Analyzing any-of items in spec that don't match the code:"))
    var anyOfErrorCount = 0
    for (item in typeToSpecAnyOf) {
        val subclasses = typeToCodeDirectImplementors[item.key.addAstPackagePrefix()] ?: emptySet()
        val expected =
            item.value.toList().map { it.removeAstPackagePrefix() }.sortedDescending().reversed()
        val found =
            subclasses.toList().map { it.removeAstPackagePrefix() }.sortedDescending().reversed()
        if (expected != found) {
            anyOfErrorCount++
            println(
                "${bold(red("ERROR: "))} The spec states that the interface ${bold(item.key.removeAstPackagePrefix())} should be implemented by [")
            expected.forEach { println("  ${it.removeAstPackagePrefix()}") }
            println("] but found [")
            found.forEach { println("  ${it.removeAstPackagePrefix()}") }
            println("]")
            println()
        }
    }
    println("Found $anyOfErrorCount errors")
    println()

    /*
    println(bold("Matching interfaces in the code that don't match any-of items in the spec"))
    var interfaceErrorCount = 0
    for (item in typeToCodeDirectImplementors) {
        val subTypes = typeToSpecAnyOf[item.key]
        if (subTypes != item.value) {
            interfaceErrorCount++
            println("${bold(red("ERROR: "))} Expected interface ${item.key.removeAstPackagePrefix()} to be implemented by [")
            item.value.forEach {
                println("  ${it.removeAstPackagePrefix()}")
            }
            println("] but found [")
            (subTypes ?: emptySet()).forEach {
                println("  ${it.removeAstPackagePrefix()}")
            }
            println("]")
            println()
        }
    }
    println("Found $interfaceErrorCount errors")
    println()
     */
}

fun main() {
    verifySpec()
}

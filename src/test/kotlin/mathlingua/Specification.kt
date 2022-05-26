package mathlingua

import mathlingua.lib.util.bold
import mathlingua.lib.util.red
import org.reflections.Reflections
import org.reflections.scanners.Scanners

private sealed interface Form {
    fun toCode(): String
}

private enum class SequenceConstraint {
    ZeroOrMore,
    OneOrMore
}

private enum class DefinitionType {
    Common,
    ChalkTalk,
    TexTalk
}

private data class Literal(val of: String) : Form {
    override fun toCode() = "\"$of\""
}

private data class Regex(val of: String) : Form {
    override fun toCode() = "Regex[$of]"
}

private data class AnyOf(val of: List<String>) : Form {
    override fun toCode(): String {
        val builder = StringBuilder()
        for (i in of.indices) {
            if (i > 0) {
                builder.append(" |\n")
            }
            builder.append(of[i])
        }
        return builder.toString()
    }
}

private data class Sequence(
    val of: String, val separator: String, val constraint: SequenceConstraint
) : Form {
    override fun toCode() = when (constraint) {
        SequenceConstraint.OneOrMore -> {
            "$of (\"$separator\" $of)*"
        }
        else -> {
            "(NONE | $of (\"$separator\" $of)*)"
        }
    }
}

private data class SuffixSequence(
    val of: String, val separator: String, val constraint: SequenceConstraint
) : Form {
    override fun toCode() = when (constraint) {
        SequenceConstraint.OneOrMore -> {
            "($of \".\") ($of \".\")*"
        }
        else -> {
            "($of \".\")*"
        }
    }
}

private data class ZeroOrMore(val of: Form) : Form {
    override fun toCode() = "(${of.toCode()})*"
}

private data class OneOrMore(val of: Form) : Form {
    override fun toCode() = "(${of.toCode()})+"
}

private data class Optionally(val of: Form) : Form {
    override fun toCode() = "(${of.toCode()})?"
}

private data class Either(val form1: Form, val form2: Form) : Form {
    override fun toCode() = "(${form1.toCode()} | ${form2.toCode()})"
}

private data class Keyword(val text: String) : Form {
    override fun toCode() = "'$text'"
}

private data class Item(val of: List<Form>) : Form {
    override fun toCode(): String {
        val builder = StringBuilder()
        for (i in of.indices) {
            if (i > 0) {
                builder.append(' ')
            }
            builder.append(of[i].toCode())
        }
        return builder.toString()
    }
}

private data class CharSequence(val prefix: String, val suffix: String, val regex: String, val escape: String?) : Form {
    override fun toCode() = "$prefix$regex$suffix [escape=$escape]"
}

private data class DefinitionOf(val name: String, val of: Form, val type: DefinitionType) {
    fun toCode() = when (of) {
        is AnyOf -> "$name ::= \n${of.toCode().split("\n").joinToString("\n") { "   $it" }}"
        else -> "$name ::= ${of.toCode()}"
    }
}

private data class Def(val name: String) : Form {
    override fun toCode() = name
}

private data class Section(val name: String, val arg: Form, val required: Boolean)

private val SPECIFICATION =
    listOf(
        DefinitionOf("Name", Regex("""[a-zA-Z0-9'"`]+("_"[a-zA-Z0-9'"`]+)?"""), DefinitionType.Common),
        DefinitionOf("OperatorName", Regex("""[~!@#${'$'}%^&*-+=|<>?'`"]+("_"[a-zA-Z0-9'"`]+)?"""), DefinitionType.Common),
        DefinitionOf(
            "NameAssignmentItem",
            anyOf("Name", "OperatorName", "Tuple", "Sequence", "Function", "Set"), DefinitionType.Common),
        DefinitionOf(
            "NameAssignment", items(Def("Name"), Literal(":="), Def("NameAssignmentItem")), DefinitionType.Common),
        DefinitionOf("FunctionAssignment", items(Def("Function"), Literal(":="), Def("Function")), DefinitionType.Common),
        DefinitionOf("Assignment", anyOf("NameAssignment", "FunctionAssignment"), DefinitionType.Common),
        DefinitionOf("VariadicName", items(Def("Name"), Optionally(Literal("..."))), DefinitionType.Common),
        DefinitionOf(
            "Function",
            items(
                Def("Name"),
                Literal("("),
                Sequence(
                    of = "VariadicName",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore),
                Literal(")")), DefinitionType.Common),
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
                Literal(")")), DefinitionType.Common),
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
                Literal(")")), DefinitionType.Common),
        DefinitionOf("FunctionCall", anyOf("Function", "SubParamCall", "SubAndRegularParamCall"), DefinitionType.Common),
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
                Literal(")")), DefinitionType.Common),
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
                Literal(")")), DefinitionType.Common),
        DefinitionOf(
            "Sequence", anyOf("SubParamFunctionSequence", "SubAndRegularParamFunctionSequence"), DefinitionType.Common),
        DefinitionOf(
            "Tuple",
            items(
                Literal("("),
                Sequence(of = "Target", separator = ",", constraint = SequenceConstraint.OneOrMore),
                Literal(")")), DefinitionType.Common),
        DefinitionOf("NameOrNameAssignment", anyOf("Name", "NameAssignment"), DefinitionType.Common),
        DefinitionOf(
            "Set",
            items(
                Literal("{"),
                Sequence(
                    of = "NameOrNameAssignment",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore),
                Literal("}")), DefinitionType.Common),
        DefinitionOf(
            "Target",
            anyOf("Assignment", "Name", "OperatorName", "Tuple", "Sequence", "Function", "Set"), DefinitionType.Common),
        DefinitionOf(
            "Argument",
            anyOf("Target", "Text", "Statement"),
            DefinitionType.ChalkTalk
        ),
        DefinitionOf(
            "Text",
            CharSequence(
                prefix = "\"",
                suffix = "\"",
                regex = ".*",
                escape = """\""""
            ),
            DefinitionType.ChalkTalk
        ),
        DefinitionOf(
            "TextBlock",
            CharSequence(
                prefix = "::",
                suffix = "::",
                regex = ".*",
                escape = "{::}",
            ),
            DefinitionType.ChalkTalk
        ),
        DefinitionOf(
            "Statement",
            Either(
                CharSequence(
                    prefix = "'",
                    suffix = "'",
                    regex = ".*",
                    escape = null
                ),
                CharSequence(
                    prefix = "`",
                    suffix = "`",
                    regex = ".*",
                    escape = null
                )
            ),
            DefinitionType.ChalkTalk
        ),
        DefinitionOf(
            "InfixCommandFormCall",
            items(
                Def("Name"),
                Def("InfixCommandForm"),
                Def("Name")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "IdPrefixOperatorCall",
            items(
                Def("OperatorName"),
                Def("Name")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "IdPostfixOperatorCall",
            items(
                Def("Name"),
                Def("OperatorName")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "IdInfixOperatorCall",
            items(
                Def("Name"),
                Def("OperatorName"),
                Def("Name")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "IdForm",
            anyOf(
                "CommandForm",
                "InfixCommandFormCall",
                "IdPrefixOperatorCall",
                "IdPostfixOperatorCall",
                "IdInfixOperatorCall"
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "Id",
            items(
                Literal("["),
                Def("IdForm"),
                Literal("]")
            ),
            DefinitionType.ChalkTalk
        ),
        DefinitionOf(
            "SquareTargetItem",
            anyOf(
                "Name",
                "Tuple",
                "Sequence",
                "Function",
                "Set"
            ),
            DefinitionType.ChalkTalk
        ),
        DefinitionOf(
            "CommandExpression",
            items(
                Literal("""\"""),
                Sequence(
                    of = "Name",
                    separator = ".",
                    constraint = SequenceConstraint.OneOrMore
                ),
                Optionally(
                    items(
                        Optionally(
                            items(
                                Literal("["),
                                Sequence(
                                    of = "SquareTargetItem",
                                    separator = ",",
                                    constraint = SequenceConstraint.OneOrMore
                                ),
                                Optionally(
                                    items(
                                        Literal("|"),
                                        Def("Name"),
                                        Literal("...")
                                    ),
                                ),
                                Literal("]"),
                            )
                        ),
                        Optionally(
                            items(
                                Literal("_"),
                                Literal("{"),
                                Sequence(
                                    of = "Expression",
                                    separator = ",",
                                    constraint = SequenceConstraint.OneOrMore
                                ),
                                Literal("}")
                            )
                        ),
                        Optionally(
                            items(
                                Literal("^"),
                                Literal("{"),
                                Sequence(
                                    of = "Expression",
                                    separator = ",",
                                    constraint = SequenceConstraint.OneOrMore
                                ),
                                Literal("}")
                            )
                        )
                    ),
                ),
                Optionally(
                    items(
                        Literal("{"),
                        Sequence(
                            of = "Expression",
                            separator = ",",
                            constraint = SequenceConstraint.OneOrMore
                        ),
                        Literal("}")
                    )
                ),
                ZeroOrMore(
                    items(
                        Literal(":"),
                        Literal("{"),
                        Sequence(
                            of = "Expression",
                            separator = ",",
                            constraint = SequenceConstraint.OneOrMore
                        ),
                        Literal("}")
                    )
                ),
                Optionally(
                    items(
                        Literal("("),
                        Sequence(
                            of = "Expression",
                            separator = ",",
                            constraint = SequenceConstraint.OneOrMore
                        ),
                        Literal(")")
                    )
                )
            ),
            DefinitionType.ChalkTalk
        ),
        DefinitionOf(
            "CommandForm",
            items(
                Literal("""\"""),
                Sequence(
                    of = "Name",
                    separator = ".",
                    constraint = SequenceConstraint.OneOrMore
                ),
                Optionally(
                    items(
                        Optionally(
                            items(
                                Literal("["),
                                Sequence(
                                    of = "SquareTargetItem",
                                    separator = ",",
                                    constraint = SequenceConstraint.OneOrMore
                                ),
                                Optionally(
                                    items(
                                        Literal("|"),
                                        Def("Name"),
                                        Literal("...")
                                    ),
                                ),
                                Literal("]"),
                            )
                        ),
                        Optionally(
                            items(
                                Literal("_"),
                                Literal("{"),
                                Sequence(
                                    of = "VariadicName",
                                    separator = ",",
                                    constraint = SequenceConstraint.OneOrMore
                                ),
                                Literal("}")
                            )
                        ),
                        Optionally(
                            items(
                                Literal("^"),
                                Literal("{"),
                                Sequence(
                                    of = "VariadicName",
                                    separator = ",",
                                    constraint = SequenceConstraint.OneOrMore
                                ),
                                Literal("}")
                            )
                        )
                    ),
                ),
                Optionally(
                    items(
                        Literal("{"),
                        Sequence(
                            of = "VariadicName",
                            separator = ",",
                            constraint = SequenceConstraint.OneOrMore
                        ),
                        Literal("}")
                    )
                ),
                ZeroOrMore(
                    items(
                        Literal(":"),
                        Literal("{"),
                        Sequence(
                            of = "VariadicName",
                            separator = ",",
                            constraint = SequenceConstraint.OneOrMore
                        ),
                        Literal("}")
                    )
                ),
                Optionally(
                    items(
                        Literal("("),
                        Sequence(
                            of = "VariadicName",
                            separator = ",",
                            constraint = SequenceConstraint.OneOrMore
                        ),
                        Literal(")")
                    )
                )
            ),
            DefinitionType.ChalkTalk
        ),
        DefinitionOf(
            "InfixCommandExpression",
            items(
                Def("CommandExpression"),
                Literal("/")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "InfixCommandForm",
            items(
                Def("CommandForm"),
                Literal("/")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "NameOrCommand",
            anyOf(
                "Name",
                "CommandExpression"
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "VariadicIsRhs",
            anyOf(
                "VariadicName",
                "CommandExpression"
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "VariadicIsExpression",
            items(
                Def("VariadicTarget"),
                Keyword("is"),
                Def("VariadicIsRhs")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "IsExpression",
            items(
                Sequence(
                    of = "Target",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore
                ),
                Keyword("is"),
                Sequence(
                    of = "NameOrCommand",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore
                )
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "MetaIsFormItem",
            anyOf(
                "statement",
                "assignment",
                "specification",
                "expression",
                "definition"
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "MetaIsForm",
            items(
                Literal("[:"),
                Sequence(
                    of = "MetaIsFormItem",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore
                ),
                Literal(":]")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "SignatureExpression",
            items(
                Literal("""\"""),
                Sequence(
                    of = "Name",
                    separator = ".",
                    constraint = SequenceConstraint.OneOrMore
                ),
                ZeroOrMore(
                    items(
                        Literal(":"),
                        Def("Name")
                    )
                )
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "AsExpression",
            items(
                Def("Expression"),
                Keyword("as"),
                Def("SignatureExpression")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "VariadicFunction",
            items(
                Def("Function"),
                Literal("...")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "VariadicSequence",
            items(
                Def("Sequence"),
                Literal("...")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "VariadicTarget",
            anyOf(
                "VariadicName",
                "VariadicFunction",
                "VariadicSequence"
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "VariadicRhs",
            anyOf(
                "VariadicTarget",
                "Expression"
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "VariadicInExpression",
            items(
                Def("VariadicTarget"),
                Literal("in"),
                Def("VariadicRhs")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "InExpression",
            items(
                Sequence(
                    of = "Target",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore
                ),
                Literal("in"),
                Def("Expression")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "VariadicNotInExpression",
            items(
                Def("VariadicTarget"),
                Literal("notin"),
                Def("VariadicRhs")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "NotInExpression",
            items(
                Sequence(
                    of = "Target",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore
                ),
                Literal("notin"),
                Def("Expression")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "VariadicColonEqualsExpression",
            items(
                Def("VariadicTarget"),
                Literal(":="),
                Def("VariadicRhs")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "ColonEqualsExpression",
            items(
                Def("Target"),
                Literal(":="),
                Def("Expression")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "EqualsExpression",
            items(
                Def("Expression"),
                Literal("="),
                Def("Expression")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "NotEqualsExpression",
            items(
                Def("Expression"),
                Literal("!="),
                Def("Expression")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "TypeScopedInfixOperatorName",
            items(
                Def("SignatureExpression"),
                Literal("::"),
                Def("OperatorName"),
                Literal("/")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "TypeScopedOperatorName",
            items(
                Def("SignatureExpression"),
                Literal("::"),
                Def("OperatorName")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "MemberScopedOperatorName",
            items(
                Literal("["),
                SuffixSequence(
                    of = "Name",
                    separator = ".",
                    constraint = SequenceConstraint.ZeroOrMore
                ),
                Def("OperatorName"),
                Literal("]")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "MemberScopedName",
            items(
                Sequence(
                    of = "Name",
                    separator = ".",
                    constraint = SequenceConstraint.OneOrMore
                )
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "Operator",
            anyOf(
                "OperatorName",
                "MemberScopedOperatorName",
                "TypeScopedOperatorName",
                "TypeScopedInfixOperatorName"
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "InfixCommandExpression",
            items(
                Def("Expression"),
                Def("InfixCommandExpression"),
                Def("Expression")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "PrefixOperatorExpression",
            items(
                Def("MemberScopedOperatorName"),
                Def("Expression")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "InfixOperatorExpression",
            items(
                Def("Expression"),
                Def("MemberScopedOperatorName"),
                Def("Expression")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "PostfixOperatorExpression",
            items(
                Def("Expression"),
                Def("MemberScopedOperatorName")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "FunctionCallExpression",
            items(
                Def("Name"),
                Literal("("),
                Sequence(
                    of = "Expression",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore
                ),
                Literal(")")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "SubParamCallExpression",
            items(
                Def("Name"),
                Literal("_"),
                Literal("("),
                Sequence(
                    of = "Expression",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore
                ),
                Literal(")")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "SubAndRegularParamCallExpression",
            items(
                Def("Name"),
                Literal("_"),
                Literal("("),
                Sequence(
                    of = "Expression",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore
                ),
                Literal(")"),
                Literal("("),
                Sequence(
                    of = "Expression",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore
                ),
                Literal(")")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "CallExpression",
            anyOf(
                "FunctionCallExpression",
                "SubParamCallExpression",
                "SubAndRegularParamCallExpression"
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "TupleExpression",
            items(
                Literal("("),
                Sequence(
                    of = "Expression",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore
                ),
                Literal(")")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "OperationExpression",
            anyOf(
                "PrefixOperatorExpression",
                "InfixOperatorExpression",
                "PostfixOperatorExpression",
                "InfixCommandExpression"
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "NameAssignmentExpression",
            items(
                Def("Name"),
                Literal(":="),
                Def("Expression")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "FunctionAssignmentExpression",
            items(
                Def("Function"),
                Literal(":="),
                Def("Expression")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "SetAssignmentExpression",
            items(
                Def("Set"),
                Literal(":="),
                Def("Expression")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "SequenceAssignmentExpression",
            items(
                Def("Sequence"),
                Literal(":="),
                Def("Expression")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "TupleAssignmentExpression",
            items(
                Def("Tuple"),
                Literal(":="),
                Def("Expression")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "NameAssignmentAssignmentExpression",
            items(
                Def("NameAssignment"),
                Literal(":="),
                Def("Expression")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "OperationAssignmentExpression",
            items(
                Def("OperationExpression"),
                Literal(":="),
                Def("Expression")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "AssignmentExpression",
            anyOf(
                "NameAssignmentExpression",
                "FunctionAssignmentExpression",
                "SetAssignmentExpression",
                "SequenceAssignmentExpression",
                "TupleAssignmentExpression",
                "NameAssignmentAssignmentExpression",
                "OperationAssignmentExpression"
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "ParenGroupingExpression",
            items(
                Literal("("),
                Def("Expression"),
                Literal(")")
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "CurlyGroupingExpression",
            items(
                Literal("{"),
                Def("Expression"),
                Literal("}"),
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "GroupingExpression",
            anyOf(
                "ParenGroupingExpression",
                "CurlyGroupingExpression"
            ),
            DefinitionType.TexTalk
        ),
        DefinitionOf(
            "Expression",
            anyOf(
                "Name",
                "MemberScopedName",
                "Tuple",
                "Sequence",
                "Function",
                "Set",
                "GroupingExpression",
                "OperationExpression",
                "CommandExpression",
                "AsExpression",
                "VariadicColonEqualsExpression",
                "ColonEqualsExpression",
                "EqualsExpression",
                "NotEqualsExpression",
                "CallExpression",
                "TupleExpression",
                "AssignmentExpression"
            ),
            DefinitionType.TexTalk
        )
)

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

    val allTypesInSpec = SPECIFICATION.map { it.name.addAstPackagePrefix() }

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
    for (def in SPECIFICATION) {
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

fun specToCode(): String {
    val builder = StringBuilder()
    for (c in SPECIFICATION) {
        builder.append(c.toCode())
        builder.append("\n\n")
    }
    return builder.toString()
}

fun main() {
    // verifySpec()
    println(specToCode())
}

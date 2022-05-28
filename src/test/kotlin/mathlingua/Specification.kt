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

private data class AnyOf(val of: List<Form>) : Form {
    override fun toCode(): String {
        val builder = StringBuilder()
        for (i in of.indices) {
            if (i > 0) {
                builder.append(" |\n")
            }
            builder.append(of[i].toCode())
        }
        return builder.toString()
    }
}

private data class Sequence(
    val of: String, val separator: String, val constraint: SequenceConstraint
) : Form {
    override fun toCode() =
        when (constraint) {
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
    override fun toCode() =
        when (constraint) {
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

private object None : Form {
    override fun toCode() = ""
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

private data class CharSequence(
    val prefix: String, val suffix: String, val regex: String, val escape: String?
) : Form {
    override fun toCode() = "$prefix$regex$suffix [escape=$escape]"
}

private data class DefinitionOf(val name: String, val of: Form, val type: DefinitionType) {
    fun toCode() =
        when (of) {
            is AnyOf -> "$name ::= \n${of.toCode().split("\n").joinToString("\n") { "   $it" }}"
            is Group -> of.toCode()
            else -> "$name ::= ${of.toCode()}"
        }
}

private data class Def(val name: String) : Form {
    override fun toCode() = name
}

private data class Section(val name: String, val arg: Form, val required: Boolean) {
    fun toCode() = "$name${if (required) {""} else {"?"}}: ${arg.toCode()}"
}

private data class Group(val id: String?, val of: List<Section>) : Form {
    override fun toCode(): String {
        val builder = StringBuilder()
        if (id != null) {
            builder.append("[$id]\n")
        }
        for (sec in of) {
            if (builder.isNotEmpty() && !builder.endsWith("\n")) {
                builder.append("\n")
            }
            builder.append(sec.toCode())
        }
        return builder.toString()
    }
}

private data class Statement(val of: List<String>) : Form {
    override fun toCode() = "Statement[${of.joinToString(" | ")}]"
}

private data class Text(val regex: String) : Form {
    override fun toCode() = "Text[${regex}]"
}

private val SPECIFICATION =
    listOf(
        DefinitionOf(
            "Name", Regex("""[a-zA-Z0-9'"`]+("_"[a-zA-Z0-9'"`]+)?"""), DefinitionType.Common),
        DefinitionOf(
            "OperatorName",
            Regex("""[~!@#${'$'}%^&*-+=|<>?'`"]+("_"[a-zA-Z0-9'"`]+)?"""),
            DefinitionType.Common),
        DefinitionOf(
            "NameAssignmentItem",
            anyOf(
                Def("Name"),
                Def("OperatorName"),
                Def("Tuple"),
                Def("Sequence"),
                Def("Function"),
                Def("Set")),
            DefinitionType.Common),
        DefinitionOf(
            "NameAssignment",
            items(Def("Name"), Literal(":="), Def("NameAssignmentItem")),
            DefinitionType.Common),
        DefinitionOf(
            "FunctionAssignment",
            items(Def("Function"), Literal(":="), Def("Function")),
            DefinitionType.Common),
        DefinitionOf(
            "Assignment",
            anyOf(Def("NameAssignment"), Def("FunctionAssignment")),
            DefinitionType.Common),
        DefinitionOf(
            "VariadicName", items(Def("Name"), Optionally(Literal("..."))), DefinitionType.Common),
        DefinitionOf(
            "Function",
            items(
                Def("Name"),
                Literal("("),
                Sequence(
                    of = "VariadicName",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore),
                Literal(")")),
            DefinitionType.Common),
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
                Literal(")")),
            DefinitionType.Common),
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
                Literal(")")),
            DefinitionType.Common),
        DefinitionOf(
            "FunctionCall",
            anyOf(Def("Function"), Def("SubParamCall"), Def("SubAndRegularParamCall")),
            DefinitionType.Common),
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
                Literal(")")),
            DefinitionType.Common),
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
                Literal(")")),
            DefinitionType.Common),
        DefinitionOf(
            "Sequence",
            anyOf(Def("SubParamFunctionSequence"), Def("SubAndRegularParamFunctionSequence")),
            DefinitionType.Common),
        DefinitionOf(
            "Tuple",
            items(
                Literal("("),
                Sequence(of = "Target", separator = ",", constraint = SequenceConstraint.OneOrMore),
                Literal(")")),
            DefinitionType.Common),
        DefinitionOf(
            "NameOrNameAssignment",
            anyOf(Def("Name"), Def("NameAssignment")),
            DefinitionType.Common),
        DefinitionOf(
            "Set",
            items(
                Literal("{"),
                Sequence(
                    of = "NameOrNameAssignment",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore),
                Literal("}")),
            DefinitionType.Common),
        DefinitionOf(
            "Target",
            anyOf(
                Def("Assignment"),
                Def("Name"),
                Def("OperatorName"),
                Def("Tuple"),
                Def("Sequence"),
                Def("Function"),
                Def("Set")),
            DefinitionType.Common),
        DefinitionOf(
            "Argument",
            anyOf(Def("Target"), Text(""), Statement(of = emptyList())),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "Text",
            CharSequence(prefix = "\"", suffix = "\"", regex = ".*", escape = """\""""),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "TextBlock",
            CharSequence(
                prefix = "::",
                suffix = "::",
                regex = ".*",
                escape = "{::}",
            ),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "Statement",
            Either(
                CharSequence(prefix = "'", suffix = "'", regex = ".*", escape = null),
                CharSequence(prefix = "`", suffix = "`", regex = ".*", escape = null)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "InfixCommandFormCall",
            items(Def("Name"), Def("InfixCommandForm"), Def("Name")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "IdPrefixOperatorCall",
            items(Def("OperatorName"), Def("Name")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "IdPostfixOperatorCall",
            items(Def("Name"), Def("OperatorName")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "IdInfixOperatorCall",
            items(Def("Name"), Def("OperatorName"), Def("Name")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "IdForm",
            anyOf(
                Def("CommandForm"),
                Def("InfixCommandFormCall"),
                Def("IdPrefixOperatorCall"),
                Def("IdPostfixOperatorCall"),
                Def("IdInfixOperatorCall")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "Id", items(Literal("["), Def("IdForm"), Literal("]")), DefinitionType.ChalkTalk),
        DefinitionOf(
            "SquareTargetItem",
            anyOf(Def("Name"), Def("Tuple"), Def("Sequence"), Def("Function"), Def("Set")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "CommandExpression",
            items(
                Literal("""\"""),
                Sequence(of = "Name", separator = ".", constraint = SequenceConstraint.OneOrMore),
                Optionally(
                    items(
                        Optionally(
                            items(
                                Literal("["),
                                Sequence(
                                    of = "SquareTargetItem",
                                    separator = ",",
                                    constraint = SequenceConstraint.OneOrMore),
                                Optionally(
                                    items(Literal("|"), Def("Name"), Literal("...")),
                                ),
                                Literal("]"),
                            )),
                        Optionally(
                            items(
                                Literal("_"),
                                Literal("{"),
                                Sequence(
                                    of = "Expression",
                                    separator = ",",
                                    constraint = SequenceConstraint.OneOrMore),
                                Literal("}"))),
                        Optionally(
                            items(
                                Literal("^"),
                                Literal("{"),
                                Sequence(
                                    of = "Expression",
                                    separator = ",",
                                    constraint = SequenceConstraint.OneOrMore),
                                Literal("}")))),
                ),
                Optionally(
                    items(
                        Literal("{"),
                        Sequence(
                            of = "Expression",
                            separator = ",",
                            constraint = SequenceConstraint.OneOrMore),
                        Literal("}"))),
                ZeroOrMore(
                    items(
                        Literal(":"),
                        Literal("{"),
                        Sequence(
                            of = "Expression",
                            separator = ",",
                            constraint = SequenceConstraint.OneOrMore),
                        Literal("}"))),
                Optionally(
                    items(
                        Literal("("),
                        Sequence(
                            of = "Expression",
                            separator = ",",
                            constraint = SequenceConstraint.OneOrMore),
                        Literal(")")))),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "CommandForm",
            items(
                Literal("""\"""),
                Sequence(of = "Name", separator = ".", constraint = SequenceConstraint.OneOrMore),
                Optionally(
                    items(
                        Optionally(
                            items(
                                Literal("["),
                                Sequence(
                                    of = "SquareTargetItem",
                                    separator = ",",
                                    constraint = SequenceConstraint.OneOrMore),
                                Optionally(
                                    items(Literal("|"), Def("Name"), Literal("...")),
                                ),
                                Literal("]"),
                            )),
                        Optionally(
                            items(
                                Literal("_"),
                                Literal("{"),
                                Sequence(
                                    of = "VariadicName",
                                    separator = ",",
                                    constraint = SequenceConstraint.OneOrMore),
                                Literal("}"))),
                        Optionally(
                            items(
                                Literal("^"),
                                Literal("{"),
                                Sequence(
                                    of = "VariadicName",
                                    separator = ",",
                                    constraint = SequenceConstraint.OneOrMore),
                                Literal("}")))),
                ),
                Optionally(
                    items(
                        Literal("{"),
                        Sequence(
                            of = "VariadicName",
                            separator = ",",
                            constraint = SequenceConstraint.OneOrMore),
                        Literal("}"))),
                ZeroOrMore(
                    items(
                        Literal(":"),
                        Literal("{"),
                        Sequence(
                            of = "VariadicName",
                            separator = ",",
                            constraint = SequenceConstraint.OneOrMore),
                        Literal("}"))),
                Optionally(
                    items(
                        Literal("("),
                        Sequence(
                            of = "VariadicName",
                            separator = ",",
                            constraint = SequenceConstraint.OneOrMore),
                        Literal(")")))),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "InfixCommandExpressionForm",
            items(Def("CommandExpression"), Literal("/")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "InfixCommandForm", items(Def("CommandForm"), Literal("/")), DefinitionType.TexTalk),
        DefinitionOf(
            "NameOrCommand", anyOf(Def("Name"), Def("CommandExpression")), DefinitionType.TexTalk),
        DefinitionOf(
            "VariadicIsRhs",
            anyOf(Def("VariadicName"), Def("CommandExpression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "VariadicIsExpression",
            items(Def("VariadicTarget"), Keyword("is"), Def("VariadicIsRhs")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "IsExpression",
            items(
                Sequence(of = "Target", separator = ",", constraint = SequenceConstraint.OneOrMore),
                Keyword("is"),
                Sequence(
                    of = "NameOrCommand",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore)),
            DefinitionType.TexTalk),
        DefinitionOf(
            "MetaIsFormItem",
            anyOf(
                Def("statement"),
                Def("assignment"),
                Def("specification"),
                Def("expression"),
                Def("definition")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "MetaIsForm",
            items(
                Literal("[:"),
                Sequence(
                    of = "MetaIsFormItem",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore),
                Literal(":]")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "SignatureExpression",
            items(
                Literal("""\"""),
                Sequence(of = "Name", separator = ".", constraint = SequenceConstraint.OneOrMore),
                ZeroOrMore(items(Literal(":"), Def("Name")))),
            DefinitionType.TexTalk),
        DefinitionOf(
            "AsExpression",
            items(Def("Expression"), Keyword("as"), Def("SignatureExpression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "VariadicFunction", items(Def("Function"), Literal("...")), DefinitionType.TexTalk),
        DefinitionOf(
            "VariadicSequence", items(Def("Sequence"), Literal("...")), DefinitionType.TexTalk),
        DefinitionOf(
            "VariadicTarget",
            anyOf(Def("VariadicName"), Def("VariadicFunction"), Def("VariadicSequence")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "VariadicRhs", anyOf(Def("VariadicTarget"), Def("Expression")), DefinitionType.TexTalk),
        DefinitionOf(
            "VariadicInExpression",
            items(Def("VariadicTarget"), Literal("in"), Def("VariadicRhs")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "InExpression",
            items(
                Sequence(of = "Target", separator = ",", constraint = SequenceConstraint.OneOrMore),
                Literal("in"),
                Def("Expression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "VariadicNotInExpression",
            items(Def("VariadicTarget"), Literal("notin"), Def("VariadicRhs")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "NotInExpression",
            items(
                Sequence(of = "Target", separator = ",", constraint = SequenceConstraint.OneOrMore),
                Literal("notin"),
                Def("Expression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "VariadicColonEqualsExpression",
            items(Def("VariadicTarget"), Literal(":="), Def("VariadicRhs")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "ColonEqualsExpression",
            items(Def("Target"), Literal(":="), Def("Expression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "EqualsExpression",
            items(Def("Expression"), Literal("="), Def("Expression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "NotEqualsExpression",
            items(Def("Expression"), Literal("!="), Def("Expression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "TypeScopedInfixOperatorName",
            items(Def("SignatureExpression"), Literal("::"), Def("OperatorName"), Literal("/")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "TypeScopedOperatorName",
            items(Def("SignatureExpression"), Literal("::"), Def("OperatorName")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "MemberScopedOperatorName",
            items(
                Literal("["),
                SuffixSequence(
                    of = "Name", separator = ".", constraint = SequenceConstraint.ZeroOrMore),
                Def("OperatorName"),
                Literal("]")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "MemberScopedName",
            items(
                Sequence(of = "Name", separator = ".", constraint = SequenceConstraint.OneOrMore)),
            DefinitionType.TexTalk),
        DefinitionOf(
            "Operator",
            anyOf(
                Def("OperatorName"),
                Def("MemberScopedOperatorName"),
                Def("TypeScopedOperatorName"),
                Def("TypeScopedInfixOperatorName")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "InfixCommandExpression",
            items(Def("Expression"), Def("InfixCommandExpressionForm"), Def("Expression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "PrefixOperatorExpression",
            items(Def("MemberScopedOperatorName"), Def("Expression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "InfixOperatorExpression",
            items(Def("Expression"), Def("MemberScopedOperatorName"), Def("Expression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "PostfixOperatorExpression",
            items(Def("Expression"), Def("MemberScopedOperatorName")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "FunctionCallExpression",
            items(
                Def("Name"),
                Literal("("),
                Sequence(
                    of = "Expression", separator = ",", constraint = SequenceConstraint.OneOrMore),
                Literal(")")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "SubParamCallExpression",
            items(
                Def("Name"),
                Literal("_"),
                Literal("("),
                Sequence(
                    of = "Expression", separator = ",", constraint = SequenceConstraint.OneOrMore),
                Literal(")")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "SubAndRegularParamCallExpression",
            items(
                Def("Name"),
                Literal("_"),
                Literal("("),
                Sequence(
                    of = "Expression", separator = ",", constraint = SequenceConstraint.OneOrMore),
                Literal(")"),
                Literal("("),
                Sequence(
                    of = "Expression", separator = ",", constraint = SequenceConstraint.OneOrMore),
                Literal(")")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "CallExpression",
            anyOf(
                Def("FunctionCallExpression"),
                Def("SubParamCallExpression"),
                Def("SubAndRegularParamCallExpression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "TupleExpression",
            items(
                Literal("("),
                Sequence(
                    of = "Expression", separator = ",", constraint = SequenceConstraint.OneOrMore),
                Literal(")")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "OperationExpression",
            anyOf(
                Def("PrefixOperatorExpression"),
                Def("InfixOperatorExpression"),
                Def("PostfixOperatorExpression"),
                Def("InfixCommandExpression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "NameAssignmentExpression",
            items(Def("Name"), Literal(":="), Def("Expression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "FunctionAssignmentExpression",
            items(Def("Function"), Literal(":="), Def("Expression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "SetAssignmentExpression",
            items(Def("Set"), Literal(":="), Def("Expression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "SequenceAssignmentExpression",
            items(Def("Sequence"), Literal(":="), Def("Expression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "TupleAssignmentExpression",
            items(Def("Tuple"), Literal(":="), Def("Expression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "NameAssignmentAssignmentExpression",
            items(Def("NameAssignment"), Literal(":="), Def("Expression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "OperationAssignmentExpression",
            items(Def("OperationExpression"), Literal(":="), Def("Expression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "AssignmentExpression",
            anyOf(
                Def("NameAssignmentExpression"),
                Def("FunctionAssignmentExpression"),
                Def("SetAssignmentExpression"),
                Def("SequenceAssignmentExpression"),
                Def("TupleAssignmentExpression"),
                Def("NameAssignmentAssignmentExpression"),
                Def("OperationAssignmentExpression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "ParenGroupingExpression",
            items(Literal("("), Def("Expression"), Literal(")")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "CurlyGroupingExpression",
            items(
                Literal("{"),
                Def("Expression"),
                Literal("}"),
            ),
            DefinitionType.TexTalk),
        DefinitionOf(
            "GroupingExpression",
            anyOf(Def("ParenGroupingExpression"), Def("CurlyGroupingExpression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "Expression",
            anyOf(
                Def("Name"),
                Def("MemberScopedName"),
                Def("Tuple"),
                Def("Sequence"),
                Def("Function"),
                Def("Set"),
                Def("GroupingExpression"),
                Def("OperationExpression"),
                Def("CommandExpression"),
                Def("AsExpression"),
                Def("VariadicColonEqualsExpression"),
                Def("ColonEqualsExpression"),
                Def("EqualsExpression"),
                Def("NotEqualsExpression"),
                Def("CallExpression"),
                Def("TupleExpression"),
                Def("AssignmentExpression")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "Clause",
            anyOf(
                Def("and:"),
                Def("not:"),
                Def("or:"),
                Def("exists:"),
                Def("existsUnique:"),
                Def("forAll:"),
                Def("if:"),
                Def("iff:"),
                Text(regex = ".*"),
                Statement(of = listOf("Expression"))),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "Spec",
            anyOf(
                Statement(of = listOf("IsExpression", "VariadicIsExpression")),
                Statement(of = listOf("InExpression", "VariadicInExpression"))),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "and:",
            group(null, Section(name = "and", arg = OneOrMore(Def("Clause")), required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "not:",
            group(null, Section(name = "not", arg = Def("Clause"), required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "or:",
            group(null, Section(name = "or", arg = OneOrMore(Def("Clause")), required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "exists:",
            group(
                null,
                Section(name = "exists", arg = OneOrMore(Def("Target")), required = true),
                Section(name = "where", arg = OneOrMore(Def("Spec")), required = false),
                Section(name = "suchThat", arg = OneOrMore(Def("Clause")), required = false)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "existsUnique:",
            group(
                null,
                Section(name = "existsUnique", arg = OneOrMore(Def("Target")), required = true),
                Section(name = "where", arg = OneOrMore(Def("Spec")), required = false),
                Section(name = "suchThat", arg = OneOrMore(Def("Clause")), required = false)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "forAll:",
            group(
                null,
                Section(name = "forAll", arg = OneOrMore(Def("Target")), required = true),
                Section(name = "where", arg = OneOrMore(Def("Spec")), required = false),
                Section(name = "suchThat", arg = OneOrMore(Def("Clause")), required = false),
                Section(name = "then", arg = OneOrMore(Def("Clause")), required = true),
            ),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "if:",
            group(
                null,
                Section(name = "if", arg = OneOrMore(Def("Clause")), required = true),
                Section(name = "then", arg = OneOrMore(Def("Clause")), required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "iff:",
            group(
                null,
                Section(name = "iff", arg = OneOrMore(Def("Clause")), required = true),
                Section(name = "then", arg = OneOrMore(Def("Clause")), required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "NameOrFunction", anyOf(Def("Name"), Def("Function")), DefinitionType.ChalkTalk),
        DefinitionOf(
            "generated:",
            group(
                null,
                Section(name = "generated", arg = None, required = true),
                Section(name = "from", arg = OneOrMore(Def("NameOrFunction")), required = true),
                Section(
                    name = "when",
                    arg = OneOrMore(Statement(of = listOf("ColonEqualsExpression"))),
                    required = false)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "piecewise:",
            group(
                null,
                Section(name = "piecewise", arg = None, required = true),
                Section(name = "when", arg = OneOrMore(Def("Clause")), required = false),
                Section(
                    name = "then",
                    arg = OneOrMore(Statement(of = listOf("ColonEqualsExpression"))),
                    required = false),
                Section(
                    name = "else",
                    arg = OneOrMore(Statement(of = listOf("ColonEqualsExpression"))),
                    required = false),
            ),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "matching:",
            group(
                null,
                Section(
                    name = "matching",
                    arg = OneOrMore(Statement(of = listOf("ColonEqualsExpression"))),
                    required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "ProvidedItem",
            anyOf(
                Statement(of = listOf("Expression InfixCommandExpression Expression")),
                Statement(of = listOf("OperationExpression"))),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "equality:",
            group(
                null,
                Section(name = "equality", arg = None, required = true),
                Section(
                    name = "between", arg = items(Def("Target"), Def("Target")), required = true),
                Section(name = "provided", arg = Def("ProvidedItem"), required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "membership:",
            group(
                null,
                Section(name = "membership", arg = None, required = true),
                Section(name = "through", arg = Statement(of = emptyList()), required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "view:",
            group(
                null,
                Section(name = "view", arg = None, required = true),
                Section(name = "as", arg = Text("SignatureExpression"), required = true),
                Section(name = "via", arg = Statement(of = listOf("Expression")), required = true),
                Section(name = "by", arg = Statement(of = listOf("CommandForm")), required = false),
            ),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "symbols:",
            group(
                null,
                Section(name = "symbols", arg = OneOrMore(Def("Name")), required = true),
                Section(
                    name = "where",
                    arg = OneOrMore(Statement(of = listOf("ColonEqualsExpression"))),
                    required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "memberSymbols:",
            group(
                null,
                Section(name = "memberSymbols", arg = OneOrMore(Def("Name")), required = true),
                Section(
                    name = "where",
                    arg = OneOrMore(Statement(of = listOf("ColonEqualsExpression"))),
                    required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "MetadataItem",
            anyOf(Def("note:"), Def("author:"), Def("tag:"), Def("reference:")),
            DefinitionType.ChalkTalk),
        DefinitionOf("AnyView", ZeroOrMore(Def("View")), DefinitionType.ChalkTalk),
        DefinitionOf(
            "ProvidingItem",
            anyOf(
                Def("AnyView"),
                Def("symbols:"),
                Def("memberSymbols:"),
                Def("equality:"),
                Def("membership:")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "SatisfyingItem",
            anyOf(
                Def("generated:"),
                Def("Clause"),
                Def("Spec"),
                Def("ColonEqualsExpression"),
                Def("VariadicColonEqualsExpression")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "ExpressingItem",
            anyOf(
                Def("piecewise:"),
                Def("match:"),
                Def("Clause"),
                Def("Spec"),
                Def("ColonEqualsExpression"),
                Def("VariadicColonEqualsExpression")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "Defines:",
            group(
                "Id",
                Section(name = "Defines", arg = Def("Target"), required = true),
                Section(name = "with", arg = OneOrMore(Def("Assignment")), required = false),
                Section(name = "given", arg = OneOrMore(Def("Target")), required = false),
                Section(name = "when", arg = OneOrMore(Def("Spec")), required = false),
                Section(name = "suchThat", arg = OneOrMore(Def("Clause")), required = false),
                Section(
                    name = "means",
                    arg = Statement(of = listOf("IsExpression", "VariadicIsExpression")),
                    required = false),
                Section(
                    name = "satisfying", arg = OneOrMore(Def("SatisfyingItem")), required = true),
                Section(
                    name = "expressing", arg = OneOrMore(Def("ExpressingItem")), required = true),
                Section(
                    name = "using",
                    arg = OneOrMore(Statement(of = listOf("ColonEqualsExpression"))),
                    required = false),
                Section(name = "writing", arg = OneOrMore(Text(".*")), required = false),
                Section(name = "written", arg = OneOrMore(Text(".*")), required = true),
                Section(name = "called", arg = OneOrMore(Text(".*")), required = false),
                Section(
                    name = "Providing", arg = OneOrMore(Def("ProvidingItem")), required = false),
                Section(name = "Metadata", arg = OneOrMore(Def("MetadataItem")), required = false)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "note:",
            group(null, Section(name = "note", arg = Text(".*"), required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "tag:",
            group(null, Section(name = "tag", arg = OneOrMore(Text(".*")), required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "reference:",
            group(
                null,
                Section(
                    name = "reference",
                    arg =
                        OneOrMore(
                            Def(
                                """
                        Text["@" (Name ".") Name (":page" "{" [0-9]+ "}")?
                           (":offset" "{" [0-9]+ "}")?
                           (":at" "{" Text[.*] "}"]
                    """.trimIndent())),
                    required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "ThatItem",
            anyOf(Def("Clause"), Def("Spec"), Def("ColonEqualsExpression")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "States:",
            group(
                null,
                Section(name = "States", arg = None, required = true),
                Section(name = "given", arg = OneOrMore(Def("Target")), required = false),
                Section(name = "when", arg = OneOrMore(Def("Spec")), required = false),
                Section(name = "suchThat", arg = OneOrMore(Def("Clause")), required = false),
                Section(name = "that", arg = OneOrMore(Def("ThatItem")), required = true),
                Section(
                    name = "using",
                    arg = Statement(of = listOf("ColonEqualsExpression")),
                    required = false),
                Section(name = "written", arg = OneOrMore(Text(".*")), required = true),
                Section(name = "called", arg = OneOrMore(Text(".*")), required = false),
                Section(name = "Metadata", arg = OneOrMore(Def("MetadataItem")), required = false)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "ResourceName",
            items(
                Literal("@"),
                Sequence(of = "Name", separator = ".", constraint = SequenceConstraint.OneOrMore)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "type:",
            group(null, Section(name = "type", arg = Text(".*"), required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "name:",
            group(null, Section(name = "name", arg = Text(".*"), required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "author:",
            group(null, Section(name = "author", arg = OneOrMore(Text(".*")), required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "homepage:",
            group(null, Section(name = "homepage", arg = Text(".*"), required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "url:",
            group(null, Section(name = "url", arg = Text(".*"), required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "offset:",
            group(null, Section(name = "offset", arg = Text(".*"), required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "ResourceItem",
            anyOf(
                Def("type"),
                Def("name"),
                Def("author"),
                Def("homepage"),
                Def("url"),
                Def("offset")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "Resource:",
            group(
                "ResourceName",
                Section(name = "Resource", arg = OneOrMore(Def("ResourceItem")), required = true),
                Section(name = "Metadata", arg = OneOrMore(Def("MetadataItem")), required = false)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "Axiom:",
            group(
                "Id?",
                Section(name = "Axiom", arg = ZeroOrMore(Text(".*")), required = true),
                Section(name = "given", arg = OneOrMore(Def("Target")), required = false),
                Section(name = "where", arg = OneOrMore(Def("Spec")), required = false),
                Section(name = "suchThat", arg = OneOrMore(Def("Clause")), required = false),
                Section(name = "then", arg = OneOrMore(Def("Clause")), required = true),
                Section(name = "iff", arg = OneOrMore(Def("Clause")), required = false),
                Section(
                    name = "using",
                    arg = OneOrMore(Statement(of = listOf("ColonEqualsExpression"))),
                    required = false),
                Section(name = "Metadata", arg = OneOrMore(Def("MetadataItem")), required = false)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "Conjecture:",
            group(
                "Id?",
                Section(name = "Conjecture", arg = ZeroOrMore(Text(".*")), required = true),
                Section(name = "given", arg = OneOrMore(Def("Target")), required = false),
                Section(name = "where", arg = OneOrMore(Def("Spec")), required = false),
                Section(name = "suchThat", arg = OneOrMore(Def("Clause")), required = false),
                Section(name = "then", arg = OneOrMore(Def("Clause")), required = true),
                Section(name = "iff", arg = OneOrMore(Def("Clause")), required = false),
                Section(
                    name = "using",
                    arg = OneOrMore(Statement(of = listOf("ColonEqualsExpression"))),
                    required = false),
                Section(name = "Metadata", arg = OneOrMore(Def("MetadataItem")), required = false)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "Theorem:",
            group(
                "Id?",
                Section(name = "Theorem", arg = ZeroOrMore(Text(".*")), required = true),
                Section(name = "given", arg = OneOrMore(Def("Target")), required = false),
                Section(name = "where", arg = OneOrMore(Def("Spec")), required = false),
                Section(name = "suchThat", arg = OneOrMore(Def("Clause")), required = false),
                Section(name = "then", arg = OneOrMore(Def("Clause")), required = true),
                Section(name = "iff", arg = OneOrMore(Def("Clause")), required = false),
                Section(
                    name = "using",
                    arg = OneOrMore(Statement(of = listOf("ColonEqualsExpression"))),
                    required = false),
                Section(name = "Proof", arg = OneOrMore(Text(".*")), required = false),
                Section(name = "Metadata", arg = OneOrMore(Def("MetadataItem")), required = false)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "TopicName",
            items(
                Literal("#"),
                Sequence(of = "Name", separator = ".", constraint = SequenceConstraint.OneOrMore)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "Topic:",
            group(
                null,
                Section(name = "Topic", arg = ZeroOrMore(Text(".*")), required = true),
                Section(name = "content", arg = Text(".*"), required = true),
                Section(name = "Metadata", arg = OneOrMore(Def("MetadataItem")), required = false)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "Note:",
            group(
                null,
                Section(name = "Note", arg = None, required = true),
                Section(name = "content", arg = Text(".*"), required = true),
                Section(name = "Metadata", arg = OneOrMore(Def("MetadataItem")), required = false)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "SpecifyItem",
            anyOf(
                Def("zero:"),
                Def("positiveInt:"),
                Def("negativeInt:"),
                Def("positiveFloat:"),
                Def("negativeFloat:")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "Specify:",
            group(null, Section(name = "Specify", arg = Def("SpecifyItem"), required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "zero:",
            group(
                null,
                Section(name = "zero", arg = None, required = true),
                Section(name = "is", arg = Text("SignatureExpression"), required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "positiveInt:",
            group(
                null,
                Section(name = "positiveInt", arg = None, required = true),
                Section(name = "is", arg = Text("SignatureExpression"), required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "negativeInt:",
            group(
                null,
                Section(name = "negativeInt", arg = None, required = true),
                Section(name = "is", arg = Text("SignatureExpression"), required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "positiveFloat:",
            group(
                null,
                Section(name = "positiveFloat", arg = None, required = true),
                Section(name = "is", arg = Text("SignatureExpression"), required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "negativeFloat:",
            group(
                null,
                Section(name = "negativeFloat", arg = None, required = true),
                Section(name = "is", arg = Text("SignatureExpression"), required = true)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "TopLevelGroup",
            anyOf(
                Def("Defines:"),
                Def("States:"),
                Def("Axiom:"),
                Def("Conjecture:"),
                Def("Theorem:"),
                Def("Topic:"),
                Def("Resource:"),
                Def("Specify:"),
                Def("Note:")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "TopLevelGroupOrTextBlock",
            anyOf(Def("TopLevelGroup"), Def("TextBlock")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "Document", ZeroOrMore(Def("TopLevelGroupOrTextBlock")), DefinitionType.ChalkTalk))

private fun anyOf(vararg of: Form) = AnyOf(of.toList())

private fun items(vararg of: Form) = Item(of.toList())

private fun group(id: String?, vararg of: Section) = Group(id = id, of = of.toList())

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

private class ErrorLogger {
    private var count = 0

    fun error(message: String) {
        count++
        println("${bold(red("ERROR:"))} $message")
    }

    fun log(message: String) = println(message)

    fun numErrors() = count
}

private fun checkForDuplicateDefinitionOfNames(logger: ErrorLogger) {
    val usedNames = mutableSetOf<String>()
    var count = 0
    for (spec in SPECIFICATION) {
        if (spec.name in usedNames) {
            count++
            logger.error("Duplicate defined name: ${bold(spec.name)}")
        }
        usedNames.add(spec.name)
    }
    println("Checking for duplicate definition names found $count errors")
}

private fun getAllTypesInCode(): List<String> {
    val reflections = Reflections(AST_PACKAGE, Scanners.values())
    return reflections.getAll(Scanners.SubTypes).filter {
        // only find mathlingua types
        it.contains(AST_PACKAGE) &&
            // ignore types associated with tests (tests make a class
            // for each test of the form <test-class>$<test-name>)
            !it.contains("$")
    }
}

private fun getAllTypesInSpec() = SPECIFICATION.map { it.name.addAstPackagePrefix() }

private fun checkAllItemsDeclaredInTheSpecButNotInCode(logger: ErrorLogger) {
    val allTypesInSpec = getAllTypesInSpec()
    val allTypesInCode = getAllTypesInCode()

    logger.log(bold("Analyzing items declared in the spec but not in code:"))
    var notInCodeCount = 0
    for (t in allTypesInSpec) {
        if (t !in allTypesInCode) {
            notInCodeCount++
            logger.log(
                "${red(bold("ERROR: "))} ${bold(t.removeAstPackagePrefix())} is declared in the spec but not defined in code")
        }
    }
    logger.log("Found $notInCodeCount errors")
}

/*
fun verifySpec() {
    println(bold("Analyzing items declared in the code but not in the spec:"))
    var notInSpecCount = 0
    for (t in allTypesInCode) {
        if (t !in allTypesInSpec) {
            notInSpecCount++
            println(
                "${red(bold("ERROR: "))} ${bold(t.removeAstPackagePrefix())} is declared in code but not defined in the spec")
        }
    }
    println("Found $notInSpecCount errors")
    println()

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
            typeToSpecAnyOf[def.name] = def.of.of.map { it.toCode() }.toSet()
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

    println(bold("Matching interfaces in the code that don't match any-of items in the spec"))
    var interfaceErrorCount = 0
    for (item in typeToCodeDirectImplementors) {
        val expected =
            item.value.toList().map { it.removeAstPackagePrefix() }.sortedDescending().reversed()
        val actual =
            typeToSpecAnyOf[item.key.removeAstPackagePrefix()]
                ?.map { it.removeAstPackagePrefix() }
                ?.sortedDescending()
                ?.reversed()
                ?: listOf("<not in spec>")
        if (actual != expected) {
            interfaceErrorCount++
            println(
                "${bold(red("ERROR: "))} Expected interface ${bold(item.key.removeAstPackagePrefix())} to be implemented by [")
            expected.forEach { println("  $it") }
            println("] but in the spec found [")
            actual.forEach { println("  $it") }
            println("]")
            println()
        }
    }
    println("Found $interfaceErrorCount errors")
    println()

    val totalCount = notInCodeCount + notInSpecCount + anyOfErrorCount + interfaceErrorCount
    println("Found $totalCount errors in total")
    println()
}
 */

fun specToCode(): String {
    val builder = StringBuilder()
    for (c in SPECIFICATION) {
        builder.append(c.toCode())
        builder.append("\n\n")
    }
    return builder.toString()
}

fun main() {
    val logger = ErrorLogger()

    logger.log(specToCode())
    logger.log("------------------------------------------------------")
    logger.log("")

    checkForDuplicateDefinitionOfNames(logger)
    checkAllItemsDeclaredInTheSpecButNotInCode(logger)

    logger.log("Total number of errors: ${logger.numErrors()}")
    // verifySpec()
}

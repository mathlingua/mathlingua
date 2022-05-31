/*
 * Copyright 2022 Dominic Kramer
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package mathlingua.spec

internal val MATHLINGUA_SPECIFICATION =
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
            "NameOrVariadicName", anyOf(Def("Name"), Def("VariadicName")), DefinitionType.Common),
        DefinitionOf(
            "VariadicName", items(Def("Name"), Optionally(Literal("..."))), DefinitionType.Common),
        DefinitionOf(
            "Function",
            items(
                Def("Name"),
                Literal("("),
                Sequence(
                    of = "NameOrVariadicName",
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
                    of = "NameOrVariadicName",
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
                    of = "NameOrVariadicName",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore),
                Literal(")"),
                Literal("("),
                Sequence(
                    of = "NameOrVariadicName",
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
            anyOf(Def("SubParamSequence"), Def("SubAndRegularParamSequence")),
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
            "SquareParams",
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
            DefinitionType.TexTalk),
        DefinitionOf(
            "NamedParameterExpression",
            items(
                Literal(":"),
                Literal("{"),
                Sequence(
                    of = "Expression", separator = ",", constraint = SequenceConstraint.OneOrMore),
                Literal("}")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "CommandExpression",
            items(
                Literal("""\"""),
                Sequence(of = "Name", separator = ".", constraint = SequenceConstraint.OneOrMore),
                Optionally(Def("SquareParams")),
                Optionally(
                    items(
                        Literal("{"),
                        Sequence(
                            of = "Expression",
                            separator = ",",
                            constraint = SequenceConstraint.OneOrMore),
                        Literal("}"))),
                ZeroOrMore(Def("NamedParameterExpression")),
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
            "NamedParameterForm",
            items(
                Literal(":"),
                Literal("{"),
                Sequence(
                    of = "VariadicName",
                    separator = ",",
                    constraint = SequenceConstraint.OneOrMore),
                Literal("}")),
            DefinitionType.TexTalk),
        DefinitionOf(
            "CommandForm",
            items(
                Literal("""\"""),
                Sequence(of = "Name", separator = ".", constraint = SequenceConstraint.OneOrMore),
                Optionally(Def("SquareParams")),
                Optionally(
                    items(
                        Literal("{"),
                        Sequence(
                            of = "VariadicName",
                            separator = ",",
                            constraint = SequenceConstraint.OneOrMore),
                        Literal("}"))),
                ZeroOrMore(Def("NamedParameterForm")),
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
        DefinitionOf("StatementIsFormItem", Literal("statement"), DefinitionType.TexTalk),
        DefinitionOf("AssignmentIsFormItem", Literal("assignment"), DefinitionType.TexTalk),
        DefinitionOf("SpecificationIsFormItem", Literal("specification"), DefinitionType.TexTalk),
        DefinitionOf("ExpressionIsFormItem", Literal("expression"), DefinitionType.TexTalk),
        DefinitionOf("DefinitionIsFormItem", Literal("definition"), DefinitionType.TexTalk),
        DefinitionOf(
            "MetaIsFormItem",
            anyOf(
                Def("StatementIsFormItem"),
                Def("AssignmentIsFormItem"),
                Def("SpecificationIsFormItem"),
                Def("ExpressionIsFormItem"),
                Def("DefinitionIsFormItem")),
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
                Def("FunctionCall"),
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
                Def("AssignmentExpression"),
                Def("InExpression"),
                Def("IsExpression"),
                Def("NotInExpression"),
                Def("VariadicInExpression"),
                Def("VariadicIsExpression"),
                Def("VariadicNotInExpression")),
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
                Def("Spec"),
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
            group(
                "AndGroup",
                null,
                Section(
                    name = "and",
                    arg = OneOrMore(Def("Clause")),
                    required = true,
                    classname = "AndSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "not:",
            group(
                "NotGroup",
                null,
                Section(
                    name = "not", arg = Def("Clause"), required = true, classname = "NotSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "or:",
            group(
                "OrGroup",
                null,
                Section(
                    name = "or",
                    arg = OneOrMore(Def("Clause")),
                    required = true,
                    classname = "OrSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "exists:",
            group(
                "ExistsGroup",
                null,
                Section(
                    name = "exists",
                    arg = OneOrMore(Def("Target")),
                    required = true,
                    classname = "ExistsSection"),
                Section(
                    name = "where",
                    arg = OneOrMore(Def("Spec")),
                    required = false,
                    classname = "WhereSection"),
                Section(
                    name = "suchThat",
                    arg = OneOrMore(Def("Clause")),
                    required = false,
                    classname = "SuchThatSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "existsUnique:",
            group(
                "ExistsUniqueGroup",
                null,
                Section(
                    name = "existsUnique",
                    arg = OneOrMore(Def("Target")),
                    required = true,
                    classname = "ExistsUniqueSection"),
                Section(
                    name = "where",
                    arg = OneOrMore(Def("Spec")),
                    required = false,
                    classname = "WhereSection"),
                Section(
                    name = "suchThat",
                    arg = OneOrMore(Def("Clause")),
                    required = false,
                    classname = "SuchThatSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "forAll:",
            group(
                "ForAllGroup",
                null,
                Section(
                    name = "forAll",
                    arg = OneOrMore(Def("Target")),
                    required = true,
                    classname = "ForAllSection"),
                Section(
                    name = "where",
                    arg = OneOrMore(Def("Spec")),
                    required = false,
                    classname = "WhereSection"),
                Section(
                    name = "suchThat",
                    arg = OneOrMore(Def("Clause")),
                    required = false,
                    classname = "SuchThatSection"),
                Section(
                    name = "then",
                    arg = OneOrMore(Def("Clause")),
                    required = true,
                    classname = "ThenSection"),
            ),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "if:",
            group(
                "IfGroup",
                null,
                Section(
                    name = "if",
                    arg = OneOrMore(Def("Clause")),
                    required = true,
                    classname = "IfSection"),
                Section(
                    name = "then",
                    arg = OneOrMore(Def("Clause")),
                    required = true,
                    classname = "ThenSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "iff:",
            group(
                "IffGroup",
                null,
                Section(
                    name = "iff",
                    arg = OneOrMore(Def("Clause")),
                    required = true,
                    classname = "IffSection"),
                Section(
                    name = "then",
                    arg = OneOrMore(Def("Clause")),
                    required = true,
                    classname = "ThenSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "NameOrFunction",
            anyOf(Def("Name"), Def("Function"), Def("FunctionCall")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "generated:",
            group(
                "GeneratedGroup",
                null,
                Section(
                    name = "generated",
                    arg = None,
                    required = true,
                    classname = "GeneratedSection"),
                Section(
                    name = "from",
                    arg = OneOrMore(Def("NameOrFunction")),
                    required = true,
                    classname = "FromSection"),
                Section(
                    name = "when",
                    arg = OneOrMore(Statement(of = listOf("ColonEqualsExpression"))),
                    required = false,
                    classname = "GeneratedWhenSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "piecewise:",
            group(
                "PiecewiseGroup",
                null,
                Section(
                    name = "piecewise",
                    arg = None,
                    required = true,
                    classname = "PiecewiseSection"),
                Section(
                    name = "when",
                    arg = OneOrMore(Def("Clause")),
                    required = false,
                    classname = "PiecewiseWhenSection"),
                Section(
                    name = "then",
                    arg = OneOrMore(Statement(of = listOf("ColonEqualsExpression"))),
                    required = false,
                    classname = "PiecewiseThenSection"),
                Section(
                    name = "else",
                    arg = OneOrMore(Statement(of = listOf("ColonEqualsExpression"))),
                    required = false,
                    classname = "PiecewiseElseSection"),
            ),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "matching:",
            group(
                "MatchingGroup",
                null,
                Section(
                    name = "matching",
                    arg = OneOrMore(Statement(of = listOf("ColonEqualsExpression"))),
                    required = true,
                    classname = "MatchingSection")),
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
                "EqualityGroup",
                null,
                Section(
                    name = "equality", arg = None, required = true, classname = "EqualitySection"),
                Section(
                    name = "between",
                    arg = items(Def("Target"), Def("Target")),
                    required = true,
                    classname = "BetweenSection"),
                Section(
                    name = "provided",
                    arg = Def("ProvidedItem"),
                    required = true,
                    classname = "ProvidedSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "membership:",
            group(
                "MembershipGroup",
                null,
                Section(
                    name = "membership",
                    arg = None,
                    required = true,
                    classname = "MembershipSection"),
                Section(
                    name = "through",
                    arg = Statement(of = emptyList()),
                    required = true,
                    classname = "ThroughSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "view:",
            group(
                "ViewGroup",
                null,
                Section(name = "view", arg = None, required = true, classname = "ViewSection"),
                Section(
                    name = "as",
                    arg = Text("SignatureExpression"),
                    required = true,
                    classname = "AsSection"),
                Section(
                    name = "via",
                    arg = Statement(of = listOf("Expression")),
                    required = true,
                    classname = "ViaSection"),
                Section(
                    name = "by",
                    arg = Statement(of = listOf("CommandForm")),
                    required = false,
                    classname = "BySection"),
            ),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "symbols:",
            group(
                "SymbolsGroup",
                null,
                Section(
                    name = "symbols",
                    arg = OneOrMore(Def("Name")),
                    required = true,
                    classname = "SymbolsSection"),
                Section(
                    name = "where",
                    arg = OneOrMore(Statement(of = listOf("ColonEqualsExpression"))),
                    required = true,
                    classname = "SymbolsWhereSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "memberSymbols:",
            group(
                "MemberSymbolsGroup",
                null,
                Section(
                    name = "memberSymbols",
                    arg = OneOrMore(Def("Name")),
                    required = true,
                    classname = "MemberSymbolsSection"),
                Section(
                    name = "where",
                    arg = OneOrMore(Statement(of = listOf("ColonEqualsExpression"))),
                    required = true,
                    classname = "MemberSymbolsWhereSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "MetadataItem",
            anyOf(Def("note:"), Def("author:"), Def("tag:"), Def("reference:")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "ProvidingItem",
            anyOf(
                Def("view:"),
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
                Def("matching:"),
                Def("Clause"),
                Def("Spec"),
                Def("ColonEqualsExpression"),
                Def("VariadicColonEqualsExpression")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "Defines:",
            group(
                "DefinesGroup",
                Def("Id"),
                Section(
                    name = "Defines",
                    arg = Def("Target"),
                    required = true,
                    classname = "DefinesSection"),
                Section(
                    name = "with",
                    arg = OneOrMore(Def("Assignment")),
                    required = false,
                    classname = "WithSection"),
                Section(
                    name = "given",
                    arg = OneOrMore(Def("Target")),
                    required = false,
                    classname = "GivenSection"),
                Section(
                    name = "when",
                    arg = OneOrMore(Def("Spec")),
                    required = false,
                    classname = "WhenSection"),
                Section(
                    name = "suchThat",
                    arg = OneOrMore(Def("Clause")),
                    required = false,
                    classname = "SuchThatSection"),
                Section(
                    name = "means",
                    arg = Statement(of = listOf("IsExpression", "VariadicIsExpression")),
                    required = false,
                    classname = "MeansSection"),
                Section(
                    name = "satisfying",
                    arg = OneOrMore(Def("SatisfyingItem")),
                    required = true,
                    classname = "SatisfyingSection"),
                Section(
                    name = "expressing",
                    arg = OneOrMore(Def("ExpressingItem")),
                    required = true,
                    classname = "ExpressingSection"),
                Section(
                    name = "using",
                    arg = OneOrMore(Statement(of = listOf("ColonEqualsExpression"))),
                    required = false,
                    classname = "UsingSection"),
                Section(
                    name = "writing",
                    arg = OneOrMore(Text(".*")),
                    required = false,
                    classname = "WritingSection"),
                Section(
                    name = "written",
                    arg = OneOrMore(Text(".*")),
                    required = true,
                    classname = "WrittenSection"),
                Section(
                    name = "called",
                    arg = OneOrMore(Text(".*")),
                    required = false,
                    classname = "CalledSection"),
                Section(
                    name = "Providing",
                    arg = OneOrMore(Def("ProvidingItem")),
                    required = false,
                    classname = "ProvidingSection"),
                Section(
                    name = "Metadata",
                    arg = OneOrMore(Def("MetadataItem")),
                    required = false,
                    classname = "MetadataSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "note:",
            group(
                "NoteGroup",
                null,
                Section(
                    name = "note", arg = Text(".*"), required = true, classname = "NoteSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "tag:",
            group(
                "TagGroup",
                null,
                Section(
                    name = "tag",
                    arg = OneOrMore(Text(".*")),
                    required = true,
                    classname = "TagSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "reference:",
            group(
                "ReferenceGroup",
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
                    required = true,
                    classname = "ReferenceSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "ThatItem",
            anyOf(Def("Clause"), Def("Spec"), Def("ColonEqualsExpression")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "States:",
            group(
                "StatesGroup",
                null,
                Section(name = "States", arg = None, required = true, classname = "StatesSection"),
                Section(
                    name = "given",
                    arg = OneOrMore(Def("Target")),
                    required = false,
                    classname = "GivenSection"),
                Section(
                    name = "when",
                    arg = OneOrMore(Def("Spec")),
                    required = false,
                    classname = "WhenSection"),
                Section(
                    name = "suchThat",
                    arg = OneOrMore(Def("Clause")),
                    required = false,
                    classname = "SuchThatSection"),
                Section(
                    name = "that",
                    arg = OneOrMore(Def("ThatItem")),
                    required = true,
                    classname = "ThatSection"),
                Section(
                    name = "using",
                    arg = Statement(of = listOf("ColonEqualsExpression")),
                    required = false,
                    classname = "UsingSection"),
                Section(
                    name = "written",
                    arg = OneOrMore(Text(".*")),
                    required = true,
                    classname = "WrittenSection"),
                Section(
                    name = "called",
                    arg = OneOrMore(Text(".*")),
                    required = false,
                    classname = "CalledSection"),
                Section(
                    name = "Metadata",
                    arg = OneOrMore(Def("MetadataItem")),
                    required = false,
                    classname = "MetadataSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "ResourceName",
            items(
                Literal("@"),
                Sequence(of = "Name", separator = ".", constraint = SequenceConstraint.OneOrMore)),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "type:",
            group(
                "TypeGroup",
                null,
                Section(
                    name = "type", arg = Text(".*"), required = true, classname = "TypeSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "name:",
            group(
                "NameGroup",
                null,
                Section(
                    name = "name", arg = Text(".*"), required = true, classname = "NameSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "author:",
            group(
                "AuthorGroup",
                null,
                Section(
                    name = "author",
                    arg = OneOrMore(Text(".*")),
                    required = true,
                    classname = "AuthorSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "homepage:",
            group(
                "HomepageGroup",
                null,
                Section(
                    name = "homepage",
                    arg = Text(".*"),
                    required = true,
                    classname = "HomepageSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "url:",
            group(
                "UrlGroup",
                null,
                Section(name = "url", arg = Text(".*"), required = true, classname = "UrlSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "offset:",
            group(
                "OffsetGroup",
                null,
                Section(
                    name = "offset",
                    arg = Text(".*"),
                    required = true,
                    classname = "OffsetSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "ResourceItem",
            anyOf(
                Def("type:"),
                Def("name:"),
                Def("author:"),
                Def("homepage:"),
                Def("url:"),
                Def("offset:")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "Resource:",
            group(
                "ResourceGroup",
                Def("ResourceName"),
                Section(
                    name = "Resource",
                    arg = OneOrMore(Def("ResourceItem")),
                    required = true,
                    classname = "ResourceSection"),
                Section(
                    name = "Metadata",
                    arg = OneOrMore(Def("MetadataItem")),
                    required = false,
                    classname = "MetadataSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "Axiom:",
            group(
                "AxiomGroup",
                Optionally(Def("Id")),
                Section(
                    name = "Axiom",
                    arg = ZeroOrMore(Text(".*")),
                    required = true,
                    classname = "AxiomSection"),
                Section(
                    name = "given",
                    arg = OneOrMore(Def("Target")),
                    required = false,
                    classname = "GivenSection"),
                Section(
                    name = "where",
                    arg = OneOrMore(Def("Spec")),
                    required = false,
                    classname = "WhereSection"),
                Section(
                    name = "suchThat",
                    arg = OneOrMore(Def("Clause")),
                    required = false,
                    classname = "SuchThatSection"),
                Section(
                    name = "then",
                    arg = OneOrMore(Def("Clause")),
                    required = true,
                    classname = "ThenSection"),
                Section(
                    name = "iff",
                    arg = OneOrMore(Def("Clause")),
                    required = false,
                    classname = "IffSection"),
                Section(
                    name = "using",
                    arg = OneOrMore(Statement(of = listOf("ColonEqualsExpression"))),
                    required = false,
                    classname = "UsingSection"),
                Section(
                    name = "Metadata",
                    arg = OneOrMore(Def("MetadataItem")),
                    required = false,
                    classname = "MetadataSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "Conjecture:",
            group(
                "ConjectureGroup",
                Optionally(Def("Id")),
                Section(
                    name = "Conjecture",
                    arg = ZeroOrMore(Text(".*")),
                    required = true,
                    classname = "ConjectureSection"),
                Section(
                    name = "given",
                    arg = OneOrMore(Def("Target")),
                    required = false,
                    classname = "GivenSection"),
                Section(
                    name = "where",
                    arg = OneOrMore(Def("Spec")),
                    required = false,
                    classname = "WhereSection"),
                Section(
                    name = "suchThat",
                    arg = OneOrMore(Def("Clause")),
                    required = false,
                    classname = "SuchThatSection"),
                Section(
                    name = "then",
                    arg = OneOrMore(Def("Clause")),
                    required = true,
                    classname = "ThenSection"),
                Section(
                    name = "iff",
                    arg = OneOrMore(Def("Clause")),
                    required = false,
                    classname = "IffSection"),
                Section(
                    name = "using",
                    arg = OneOrMore(Statement(of = listOf("ColonEqualsExpression"))),
                    required = false,
                    classname = "UsingSection"),
                Section(
                    name = "Metadata",
                    arg = OneOrMore(Def("MetadataItem")),
                    required = false,
                    classname = "MetadataSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "Theorem:",
            group(
                "TheoremGroup",
                Optionally(Def("Id")),
                Section(
                    name = "Theorem",
                    arg = ZeroOrMore(Text(".*")),
                    required = true,
                    classname = "TheoremSection"),
                Section(
                    name = "given",
                    arg = OneOrMore(Def("Target")),
                    required = false,
                    classname = "GivenSection"),
                Section(
                    name = "where",
                    arg = OneOrMore(Def("Spec")),
                    required = false,
                    classname = "WhereSection"),
                Section(
                    name = "suchThat",
                    arg = OneOrMore(Def("Clause")),
                    required = false,
                    classname = "SuchThatSection"),
                Section(
                    name = "then",
                    arg = OneOrMore(Def("Clause")),
                    required = true,
                    classname = "ThenSection"),
                Section(
                    name = "iff",
                    arg = OneOrMore(Def("Clause")),
                    required = false,
                    classname = "IffSection"),
                Section(
                    name = "using",
                    arg = OneOrMore(Statement(of = listOf("ColonEqualsExpression"))),
                    required = false,
                    classname = "UsingSection"),
                Section(
                    name = "Proof",
                    arg = OneOrMore(Text(".*")),
                    required = false,
                    classname = "ProofSection"),
                Section(
                    name = "Metadata",
                    arg = OneOrMore(Def("MetadataItem")),
                    required = false,
                    classname = "MetadataSection")),
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
                "TopicGroup",
                Def("TopicName"),
                Section(
                    name = "Topic",
                    arg = ZeroOrMore(Text(".*")),
                    required = true,
                    classname = "TopicSection"),
                Section(
                    name = "content",
                    arg = Text(".*"),
                    required = true,
                    classname = "ContentSection"),
                Section(
                    name = "Metadata",
                    arg = OneOrMore(Def("MetadataItem")),
                    required = false,
                    classname = "MetadataSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "Note:",
            group(
                "NoteTopLevelGroup",
                null,
                Section(
                    name = "Note", arg = None, required = true, classname = "NoteTopLevelSection"),
                Section(
                    name = "content",
                    arg = Text(".*"),
                    required = true,
                    classname = "ContentSection"),
                Section(
                    name = "Metadata",
                    arg = OneOrMore(Def("MetadataItem")),
                    required = false,
                    classname = "MetadataSection")),
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
            group(
                "SpecifyGroup",
                null,
                Section(
                    name = "Specify",
                    arg = Def("SpecifyItem"),
                    required = true,
                    classname = "SpecifySection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "zero:",
            group(
                "ZeroGroup",
                null,
                Section(name = "zero", arg = None, required = true, classname = "ZeroSection"),
                Section(
                    name = "is",
                    arg = Text("SignatureExpression"),
                    required = true,
                    classname = "IsSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "positiveInt:",
            group(
                "PositiveIntGroup",
                null,
                Section(
                    name = "positiveInt",
                    arg = None,
                    required = true,
                    classname = "PositiveIntSection"),
                Section(
                    name = "is",
                    arg = Text("SignatureExpression"),
                    required = true,
                    classname = "IsSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "negativeInt:",
            group(
                "NegativeIntGroup",
                null,
                Section(
                    name = "negativeInt",
                    arg = None,
                    required = true,
                    classname = "NegativeIntSection"),
                Section(
                    name = "is",
                    arg = Text("SignatureExpression"),
                    required = true,
                    classname = "IsSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "positiveFloat:",
            group(
                "PositiveFloatGroup",
                null,
                Section(
                    name = "positiveFloat",
                    arg = None,
                    required = true,
                    classname = "PositiveFloatSection"),
                Section(
                    name = "is",
                    arg = Text("SignatureExpression"),
                    required = true,
                    classname = "IsSection")),
            DefinitionType.ChalkTalk),
        DefinitionOf(
            "negativeFloat:",
            group(
                "NegativeFloatGroup",
                null,
                Section(
                    name = "negativeFloat",
                    arg = None,
                    required = true,
                    classname = "NegativeFloatSection"),
                Section(
                    name = "is",
                    arg = Text("SignatureExpression"),
                    required = true,
                    classname = "IsSection")),
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

internal sealed interface Form {
    fun toCode(): String
}

internal enum class SequenceConstraint {
    ZeroOrMore,
    OneOrMore
}

internal enum class DefinitionType {
    Common,
    ChalkTalk,
    TexTalk
}

internal data class Literal(val of: String) : Form {
    override fun toCode() = "\"$of\""
}

internal data class Regex(val of: String) : Form {
    override fun toCode() = "Regex[$of]"
}

internal data class AnyOf(val of: List<Form>) : Form {
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

internal data class Sequence(
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

internal data class SuffixSequence(
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

internal data class ZeroOrMore(val of: Form) : Form {
    override fun toCode() = "(${of.toCode()})*"
}

internal data class OneOrMore(val of: Form) : Form {
    override fun toCode() = "(${of.toCode()})+"
}

internal data class Optionally(val of: Form) : Form {
    override fun toCode() = "(${of.toCode()})?"
}

internal data class Either(val form1: Form, val form2: Form) : Form {
    override fun toCode() = "(${form1.toCode()} | ${form2.toCode()})"
}

internal object None : Form {
    override fun toCode() = ""
}

internal data class Keyword(val text: String) : Form {
    override fun toCode() = "'$text'"
}

internal data class Item(val of: List<Form>) : Form {
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

internal data class CharSequence(
    val prefix: String, val suffix: String, val regex: String, val escape: String?
) : Form {
    override fun toCode() = "$prefix$regex$suffix [escape=$escape]"
}

internal data class DefinitionOf(val name: String, val of: Form, val type: DefinitionType) {
    fun toCode() =
        when (of) {
            is AnyOf -> "$name ::= \n${of.toCode().split("\n").joinToString("\n") { "   $it" }}"
            is Group -> of.toCode()
            else -> "$name ::= ${of.toCode()}"
        }
}

internal data class Def(val name: String) : Form {
    override fun toCode() = name
}

internal data class Section(
    val name: String, val arg: Form, val required: Boolean, val classname: String
) {
    fun toCode() = "$name${if (required) {""} else {"?"}}: ${arg.toCode()}"
}

internal data class Group(val classname: String, val id: Form?, val of: List<Section>) : Form {
    override fun toCode(): String {
        val builder = StringBuilder()
        if (id != null) {
            builder.append("[${id.toCode()}]\n")
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

internal data class Statement(val of: List<String>) : Form {
    override fun toCode() = "Statement[${of.joinToString(" | ")}]"
}

internal data class Text(val regex: String) : Form {
    override fun toCode() = "Text[${regex}]"
}

internal const val AST_PACKAGE = "mathlingua.lib.frontend.ast"

/**
 * Adds the ast package prefix for a simple classname and does nothing for a fully qualified
 * classname
 */
internal fun String.addAstPackagePrefix() =
    if (this.startsWith(AST_PACKAGE)) {
        this
    } else {
        "${AST_PACKAGE}.${this}"
    }

internal fun DefinitionOf.getClassname() =
    if (this.of is Group) {
            this.of.classname
        } else {
            this.name
        }
        .addAstPackagePrefix()

internal fun getClassnameForDefName(name: String): String? {
    if (DEF_NAME_TO_CLASSNAME.isEmpty()) {
        DEF_NAME_TO_CLASSNAME["Text"] = "Text".addAstPackagePrefix()
        DEF_NAME_TO_CLASSNAME["Statement"] = "Statement".addAstPackagePrefix()
        for (item in MATHLINGUA_SPECIFICATION) {
            DEF_NAME_TO_CLASSNAME[item.name] = item.getClassname().addAstPackagePrefix()
            if (item.of is Group) {
                for (sec in item.of.of) {
                    DEF_NAME_TO_CLASSNAME[sec.name] = sec.classname.addAstPackagePrefix()
                }
            }
        }
    }
    return DEF_NAME_TO_CLASSNAME[name]
}

internal fun Form.getName() =
    when (this) {
        is Text -> "Text"
        is Statement -> "Statement"
        is Def -> this.name
        else -> null
    }

private val DEF_NAME_TO_CLASSNAME = mutableMapOf<String, String>()

private fun anyOf(vararg of: Form) = AnyOf(of.toList())

private fun items(vararg of: Form) = Item(of.toList())

private fun group(classname: String, id: Form?, vararg of: Section) =
    Group(classname = classname, id = id, of = of.toList())

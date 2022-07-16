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

package mathlingua.lib.frontend.ast

internal fun Node.toCode(separator: String = ""): String =
    when (this) {
        is Formulation ->
            if (this.text.contains("'")) {
                "`${this.text}`"
            } else {
                "'${this.text}'"
            }
        is FunctionForm -> {
            val builder = java.lang.StringBuilder()
            builder.append(name.toCode())
            builder.append("(")
            for (i in params.nodes.indices) {
                if (i > 0) {
                    builder.append(", ")
                }
                builder.append(params.nodes[i].toCode())
            }
            builder.append(")")
            builder.toString()
        }
        is Name -> text
        is NameAssignment -> {
            val builder = java.lang.StringBuilder()
            builder.append(lhs.toCode())
            builder.append(" := ")
            builder.append(rhs.toCode())
            builder.toString()
        }
        is OperatorName -> text
        is SubAndRegularParamSequenceForm -> {
            val builder = java.lang.StringBuilder()
            builder.append("{")
            builder.append(func.toCode())
            builder.append("}_{")
            for (i in func.subParams.nodes.indices) {
                if (i > 0) {
                    builder.append(", ")
                }
                builder.append(func.subParams.nodes[i].toCode())
            }
            builder.append("}")
            builder.toString()
        }
        is SubParamSequenceForm -> {
            val builder = java.lang.StringBuilder()
            builder.append("{")
            builder.append(func.toCode())
            builder.append("}_(")
            for (i in func.subParams.nodes.indices) {
                if (i > 0) {
                    builder.append(", ")
                }
                builder.append(func.subParams.nodes[i].toCode())
            }
            builder.append(")")
            builder.toString()
        }
        is SetForm -> {
            val builder = java.lang.StringBuilder()
            builder.append("{")
            for (i in items.nodes.indices) {
                if (i > 0) {
                    builder.append(", ")
                }
                builder.append(items.nodes[i].toCode())
            }
            builder.append("}")
            builder.toString()
        }
        is TupleForm -> {
            val builder = java.lang.StringBuilder()
            builder.append("(")
            for (i in targets.nodes.indices) {
                if (i > 0) {
                    builder.append(", ")
                }
                builder.append(targets.nodes[i].toCode())
            }
            builder.append(")")
            builder.toString()
        }
        is Text -> "\"$text\""
        is VariadicName -> "$name..."
        is Id -> text
        is TextBlock -> "::$text::"
        is SubAndRegularParamFormCall -> {
            val builder = java.lang.StringBuilder()
            builder.append(name.toCode())
            builder.append("_(")
            for (i in subParams.nodes.indices) {
                if (i > 0) {
                    builder.append(", ")
                }
                builder.append(subParams.nodes[i].toCode())
            }
            builder.append(")(")
            for (i in params.nodes.indices) {
                if (i > 0) {
                    builder.append(", ")
                }
                builder.append(params.nodes[i].toCode())
            }
            builder.append(")")
            builder.toString()
        }
        is SubParamFormCall -> {
            val builder = java.lang.StringBuilder()
            builder.append(name.toCode())
            builder.append("_(")
            for (i in subParams.nodes.indices) {
                if (i > 0) {
                    builder.append(", ")
                }
                builder.append(subParams.nodes[i].toCode())
            }
            builder.append(")")
            builder.toString()
        }
        is VariadicFunctionForm -> "${function.toCode()}..."
        is VariadicSequenceForm -> "${sequence.toCode()}..."
        is AndSection -> sectionToCode(this, *clauses.toTypedArray())
        is AndGroup -> groupToCode(null, andSection)
        is NotSection -> sectionToCode(this, clause)
        is NotGroup -> groupToCode(null, notSection)
        is OrSection -> sectionToCode(this, *clauses.toTypedArray())
        is OrGroup -> groupToCode(null, orSection)
        is ExistsSection -> sectionToCode(this, *targets.toTypedArray())
        is WhereSection -> sectionToCode(this, *specs.toTypedArray())
        is SuchThatSection -> sectionToCode(this, *clauses.toTypedArray())
        is ExistsGroup -> groupToCode(null, existsSection, whereSection, suchThatSection)
        is ExistsUniqueSection -> sectionToCode(this, *targets.toTypedArray())
        is ExistsUniqueGroup ->
            groupToCode(null, existsUniqueSection, whereSection, suchThatSection)
        is ForAllSection -> sectionToCode(this, *targets.toTypedArray())
        is ForAllGroup ->
            groupToCode(null, forAllSection, whereSection, suchThatSection, thenSection)
        is ThenSection -> sectionToCode(this, *clauses.toTypedArray())
        is IfSection -> sectionToCode(this, *clauses.toTypedArray())
        is IfGroup -> groupToCode(null, ifSection, thenSection)
        is IffSection -> sectionToCode(this, *clauses.toTypedArray())
        is IffGroup -> groupToCode(null, iffSection, thenSection)
        is GeneratedSection -> sectionToCode(this)
        is FromSection -> sectionToCode(this, *items.toTypedArray())
        is GeneratedWhenSection -> sectionToCode(this, *formulations.toTypedArray())
        is GeneratedGroup -> groupToCode(null, generatedSection, fromSection, generatedWhenSection)
        is PiecewiseSection -> sectionToCode(this)
        is PiecewiseWhenSection -> sectionToCode(this, *clauses.toTypedArray())
        is PiecewiseThenSection -> sectionToCode(this, *formulations.toTypedArray())
        is PiecewiseElseSection -> sectionToCode(this, *formulations.toTypedArray())
        is PiecewiseGroup ->
            groupToCode(
                null,
                piecewiseSection,
                piecewiseWhenSection,
                piecewiseThenSection,
                piecewiseElseSection)
        is MatchingSection -> sectionToCode(this, *formulations.toTypedArray())
        is MatchingGroup -> groupToCode(null, matchingSection)
        is EqualitySection -> sectionToCode(this)
        is BetweenSection -> sectionToCode(this, first, second)
        is ProvidedSection -> sectionToCode(this, statement)
        is EqualityGroup -> groupToCode(null, equalitySection, betweenSection, providedSection)
        is MembershipSection -> sectionToCode(this)
        is ThroughSection -> sectionToCode(this, through)
        is MembershipGroup -> groupToCode(null, membershipSection, throughSection)
        is ViewSection -> sectionToCode(this)
        is AsSection -> sectionToCode(this, asText)
        is ViaSection -> sectionToCode(this, via)
        is BySection -> sectionToCode(this, by)
        is ViewGroup -> groupToCode(null, viewSection, asSection, viaSection, bySection)
        is SymbolsSection -> sectionToCode(this, *names.toTypedArray())
        is SymbolsWhereSection -> sectionToCode(this, *formulations.toTypedArray())
        is SymbolsGroup -> groupToCode(null, symbolsSection, symbolsWhereSection)
        is SymbolsAsGroup -> groupToCode(null, symbolsSection, symbolsAsSection)
        is SymbolsAsSection -> sectionToCode(this, asSection)
        is MemberSymbolsSection -> sectionToCode(this, *names.toTypedArray())
        is MemberSymbolsWhereSection -> sectionToCode(this, *formulations.toTypedArray())
        is MemberSymbolsAsSection -> sectionToCode(this, asSection)
        is MemberSymbolsGroup -> groupToCode(null, memberSymbolsSection, memberSymbolsWhereSection)
        is MemberSymbolsAsGroup -> groupToCode(null, memberSymbolsSection, memberSymbolsAsSection)
        is ContributorSection -> sectionToCode(this, *items.toTypedArray())
        is ContributorGroup -> groupToCode(null, contributorSection)
        is AuthorSection -> sectionToCode(this, *items.toTypedArray())
        is AuthorGroup -> groupToCode(null, authorSection)
        is TagSection -> sectionToCode(this, *items.toTypedArray())
        is TagGroup -> groupToCode(null, tagSection)
        is ReferencesSection -> sectionToCode(this, *items.toTypedArray())
        is DefinesSection -> sectionToCode(this, target)
        is WithSection -> sectionToCode(this, *assignments.toTypedArray())
        is GivenSection -> sectionToCode(this, *targets.toTypedArray())
        is WhenSection -> sectionToCode(this, *specs.toTypedArray())
        is MeansSection -> sectionToCode(this, statement)
        is SatisfyingSection -> sectionToCode(this, *items.toTypedArray())
        is ExpressingSection -> sectionToCode(this, *items.toTypedArray())
        is UsingSection -> sectionToCode(this, *formulations.toTypedArray())
        is WritingGroup -> groupToCode(null, writingSection)
        is WritingSection -> sectionToCode(this, *items.toTypedArray())
        is WrittenGroup -> groupToCode(null, writtenSection)
        is WrittenSection -> sectionToCode(this, *items.toTypedArray())
        is CalledGroup -> groupToCode(null, calledSection)
        is CalledSection -> sectionToCode(this, *items.toTypedArray())
        is CodifiedSection -> sectionToCode(this, *items.toTypedArray())
        is ProvidingSection -> sectionToCode(this, *items.toTypedArray())
        is MetadataSection -> sectionToCode(this, *items.toTypedArray())
        is DefinesGroup ->
            groupToCode(
                id,
                definesSection,
                withSection,
                givenSection,
                whenSection,
                suchThatSection,
                meansSection,
                satisfyingSection,
                expressingSection,
                providingSection,
                usingSection,
                codifiedSection,
                documentedSection,
                referencesSection,
                metadataSection)
        is StatesSection -> sectionToCode(this)
        is ThatSection -> sectionToCode(this, *items.toTypedArray())
        is StatesGroup ->
            groupToCode(
                id,
                statesSection,
                givenSection,
                whenSection,
                suchThatSection,
                thatSection,
                usingSection,
                codifiedSection,
                documentedSection,
                referencesSection,
                metadataSection)
        is ResourceSection -> sectionToCode(this, *items.toTypedArray())
        is TypeSection -> sectionToCode(this, type)
        is TypeGroup -> groupToCode(null, typeSection)
        is NameSection -> sectionToCode(this, text)
        is NameGroup -> groupToCode(null, nameSection)
        is HomepageSection -> sectionToCode(this, homepage)
        is HomepageGroup -> groupToCode(null, homepageSection)
        is UrlSection -> sectionToCode(this, url)
        is UrlGroup -> groupToCode(null, urlSection)
        is OffsetSection -> sectionToCode(this, offset)
        is OffsetGroup -> groupToCode(null, offsetSection)
        is ResourceName -> "@$name"
        is ResourceGroup -> groupToCode(null, resourceSection)
        is AxiomSection -> sectionToCode(this)
        is AxiomGroup ->
            groupToCode(
                id,
                axiomSection,
                givenSection,
                whereSection,
                suchThatSection,
                thenSection,
                iffSection,
                usingSection,
                documentedSection,
                referencesSection,
                metadataSection)
        is ConjectureSection -> sectionToCode(this)
        is ConjectureGroup ->
            groupToCode(
                id,
                conjectureSection,
                givenSection,
                whereSection,
                suchThatSection,
                thenSection,
                iffSection,
                usingSection,
                documentedSection,
                referencesSection,
                metadataSection)
        is TheoremSection -> sectionToCode(this)
        is TheoremGroup ->
            groupToCode(
                id,
                theoremSection,
                givenSection,
                whereSection,
                suchThatSection,
                thenSection,
                iffSection,
                usingSection,
                proofSection,
                documentedSection,
                referencesSection,
                metadataSection)
        is ProofSection -> sectionToCode(this, *proofs.toTypedArray())
        is TopicSection -> sectionToCode(this)
        is ContentSection -> sectionToCode(this, content)
        is TopicName -> "#$name"
        is TopicGroup -> groupToCodeWithStringId(id, topicSection, contentSection, metadataSection)
        is NoteTopLevelSection -> sectionToCode(this)
        is NoteTopLevelGroup ->
            groupToCode(null, noteTopLevelSection, contentSection, metadataSection)
        is SpecifySection -> sectionToCode(this, *items.toTypedArray())
        is SpecifyGroup -> groupToCode(null, specifySection)
        is ZeroSection -> sectionToCode(this)
        is ZeroGroup -> groupToCode(null, zeroSection, isSection)
        is PositiveIntSection -> sectionToCode(this)
        is PositiveIntGroup -> groupToCode(null, positiveIntSection, isSection)
        is NegativeIntSection -> sectionToCode(this)
        is NegativeIntGroup -> groupToCode(null, negativeIntSection, isSection)
        is PositiveFloatSection -> sectionToCode(this)
        is PositiveFloatGroup -> groupToCode(null, positiveFloatSection, isSection)
        is NegativeFloatSection -> sectionToCode(this)
        is NegativeFloatGroup -> groupToCode(null, negativeFloatSection, isSection)
        is IsSection -> sectionToCode(this, form)
        is LooselySection -> sectionToCode(this, content)
        is LooselyGroup -> groupToCode(null, looselySection)
        is OverviewSection -> sectionToCode(this, content)
        is OverviewGroup -> groupToCode(null, overviewSection)
        is MotivationSection -> sectionToCode(this, content)
        is MotivationGroup -> groupToCode(null, motivationSection)
        is HistorySection -> sectionToCode(this, content)
        is HistoryGroup -> groupToCode(null, historySection)
        is ExamplesSection -> sectionToCode(this, *items.toTypedArray())
        is ExamplesGroup -> groupToCode(null, examplesSection)
        is RelatedSection -> sectionToCode(this, *items.toTypedArray())
        is RelatedGroup -> groupToCode(null, relatedSection)
        is DiscoveredSection -> sectionToCode(this, *items.toTypedArray())
        is DiscoveredGroup -> groupToCode(null, discoveredSection)
        is NotesSection -> sectionToCode(this, *items.toTypedArray())
        is NotesGroup -> groupToCode(null, notesSection)
        is DocumentedSection -> sectionToCode(this, *items.toTypedArray())
        is ColonEqualsExpression -> "${this.lhs.toCode()} := ${this.rhs.toCode()}"
        is VariadicColonEqualsExpression -> "${this.lhs.toCode()} := ${this.rhs.toCode()}"
        is CommandFormCall ->
            "\\${this.names.toCode(separator = ".")}${
            this.squareParams?.toCode() ?: ""}${if (this.subParams != null) {
            "_${this.subParams.toCode(separator = ", ")}"
        } else {""}}${
            if (this.supParams != null) {
                "^${this.supParams.toCode(separator = ", ")}"
            } else {
                ""
            }
        }${this.curlyParams?.toCode() ?: ""}${this.namedParams?.toCode() ?: ""}${this.parenParams?.toCode() ?: ""}"
        is InfixCommandFormCall ->
            "${this.lhs.toCode()} ${this.center.toCode()} ${this.rhs.toCode()}"
        is InfixOperatorFormCall ->
            "${this.lhs.toCode()} ${this.center.toCode()} ${this.rhs.toCode()}"
        is PostfixOperatorFormCall -> "${this.lhs.toCode()}${this.center.toCode()}"
        is PrefixOperatorFormCall -> "${this.center.toCode()}${this.rhs.toCode()}"
        is AsExpression -> "${this.lhs.toCode()} as ${this.toCode()}"
        is FunctionAssignmentExpression -> "${this.lhs.toCode()} := ${this.rhs.toCode()}"
        is NameAssignmentAssignmentExpression -> "${this.lhs.toCode()} := ${this.rhs.toCode()}"
        is NameAssignmentExpression -> "${this.lhs.toCode()} := ${this.rhs.toCode()}"
        is OperationAssignmentExpression -> "${this.lhs.toCode()} := ${this.rhs.toCode()}"
        is SequenceAssignmentExpression -> "${this.lhs.toCode()} := ${this.rhs.toCode()}"
        is SetAssignmentExpression -> "${this.lhs.toCode()} := ${this.rhs.toCode()}"
        is TupleAssignmentExpression -> "${this.lhs.toCode()} := ${this.rhs.toCode()}"
        is FunctionCallExpression -> "${this.name.toCode()}${this.args.toCode(separator = ", ")}"
        is SubAndRegularParamCallExpression ->
            "${this.name.toCode()}_${this.subArgs.toCode(separator = ", ")}${this.args.toCode(separator = ", ")}"
        is SubParamCallExpression ->
            "${this.name.toCode()}_${this.subArgs.toCode(separator = ", ")}"
        is SquareParams ->
            if (this.isNameParam()) {
                this.asNameParam().toCode()
            } else {
                this.asSquareTargetItems().toCode(separator = ", ")
            }
        is CommandExpressionCall ->
            "\\${this.names.toCode(separator = ".")}${
            this.squareParams?.toCode() ?: ""}${if (this.subParams != null) {
                "_${this.subParams.toCode(separator = ", ")}"
            } else {""}}${
                if (this.supParams != null) {
                    "^${this.supParams.toCode(separator = ", ")}"
                } else {
                    ""
                }
            }${this.curlyParams?.toCode() ?: ""}${this.namedParams?.toCode() ?: ""}${this.parenParams?.toCode() ?: ""}"
        is EqualsExpression -> "${this.lhs.toCode()} := ${this.rhs.toCode()}"
        is InExpression -> "${this.lhs.toCode()} in ${this.rhs.toCode()}"
        is IsExpression -> "${this.lhs.toCode()} is ${this.rhs.toCode()}"
        is MemberScopedName ->
            "${this.prefixes.toCode(separator = ".")}${if (this.prefixes.nodes.isNotEmpty()) {"."} else {""}}${this.name.toCode()}"
        is NotEqualsExpression -> "${this.lhs.toCode()} != ${this.rhs.toCode()}"
        is NotInExpression -> "${this.lhs.toCode()} notin ${this.rhs.toCode()}"
        is InfixCommandExpression ->
            "${this.lhs.toCode()} ${this.command.toCode()} ${this.rhs.toCode()}"
        is InfixOperatorExpression ->
            "${this.lhs.toCode()} ${this.operator.toCode()} ${this.rhs.toCode()}"
        is PostfixOperatorExpression -> "${this.value.toCode()}${this.operator.toCode()}"
        is PrefixOperatorExpression -> "${this.operator.toCode()}${this.value.toCode()}"
        is TupleExpression -> this.args.toCode(separator = ", ")
        is VariadicInExpression -> "${this.lhs.toCode()} in ${this.rhs.toCode()}"
        is VariadicIsExpression -> "${this.lhs.toCode()} is ${this.rhs.toCode()}"
        is VariadicNotInExpression -> "${this.lhs.toCode()} notin ${this.rhs.toCode()}"
        EmptyTexTalkNode -> ""
        is InfixCommandExpressionPart -> this.expression.toCode()
        is InfixCommandFormPart -> this.command.toCode()
        is MetaIsForm -> this.items.toCode(separator = ", ")
        is AssignmentIsFormItem -> "assignment"
        is DefinitionIsFormItem -> "definition"
        is ExpressionIsFormItem -> "expression"
        is SpecificationIsFormItem -> "specification"
        is StatementIsFormItem -> "statement"
        is NamedParameterExpression ->
            ":${this.name.toCode()}${this.params.toCode(separator = ", ")}"
        is NamedParameterForm -> ":${this.name.toCode()}${this.params.toCode(separator = ", ")}"
        is CurlyNodeList<*> -> "{${this.nodes.joinToString(separator) { it.toCode() }}}"
        is NonBracketNodeList<*> -> this.nodes.joinToString(separator) { it.toCode() }
        is ParenNodeList<*> -> "(${this.nodes.joinToString(separator) { it.toCode() }})"
        is SquareColonNodeList<*> -> "[:${this.nodes.joinToString(separator) { it.toCode() }}:]"
        is SquareNodeList<*> -> "[${this.nodes.joinToString(separator) { it.toCode() }}]"
        is MemberScopedOperatorName ->
            "${this.prefixes.toCode(separator = ".")}${if (this.prefixes.nodes.isNotEmpty()) {"."} else {""}}${this.name.toCode()}"
        is TypeScopedInfixOperatorName -> "${this.signature.toCode()}::${this.name.toCode()}/"
        is TypeScopedOperatorName -> "${this.signature.toCode()}::${this.name.toCode()}"
        is SignatureExpression ->
            "\\${this.names.toCode(separator = ".")}${
            if (this.colonNames.nodes.isNotEmpty()) {":"} else {""}
        }${this.colonNames.toCode(separator = ":")}"
        is TexTalkTokenNode -> this.token.text
        is CurlyGroupingExpression -> "{${this.expression.toCode()}}"
        is ParenGroupingExpression -> "(${this.expression.toCode()})"
    }

internal fun Document.toCode(): String {
    val builder = StringBuilder()
    for (i in items.indices) {
        if (i > 0) {
            builder.append("\n\n\n")
        }
        builder.append(items[i].toCode())
    }
    return builder.toString()
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

// adds a `. ` to the start of the code and indents the subsequent lines
private fun codeToDotSpaceArg(code: String): String {
    val builder = StringBuilder()
    val lines = code.split("\n")
    if (lines.isNotEmpty()) {
        builder.append(". ")
        builder.append(lines[0])
    }
    for (i in 1 until lines.size) {
        builder.append("\n  ")
        builder.append(lines[i])
    }
    return builder.toString()
}

private fun sectionToCode(section: Section, vararg args: ChalkTalkNode): String {
    val builder = StringBuilder()
    builder.append(section.name)
    builder.append(":")
    var i = 0
    while (i < args.size) {
        val arg = args[i++]
        if (arg.metadata.isInline == false) {
            builder.append("\n")
            builder.append(codeToDotSpaceArg(arg.toCode()))
            while (i < args.size && args[i].metadata.isInline == true) {
                builder.append(", ")
                builder.append(args[i++].toCode())
            }
        } else {
            builder.append(" ")
            builder.append(arg.toCode())
            while (i < args.size && args[i].metadata.isInline == true) {
                builder.append(", ")
                builder.append(args[i++].toCode())
            }
        }
    }
    return builder.toString()
}

private fun groupToCode(id: ChalkTalkNode?, vararg sections: Section?) =
    groupToCodeWithStringId(id, *sections)

private fun groupToCodeWithStringId(id: ChalkTalkNode?, vararg sections: Section?): String {
    val builder = StringBuilder()
    if (id != null) {
        builder.append("[")
        builder.append(id.toCode())
        builder.append("]\n")
    }
    var isFirstPrinted = true
    for (sect in sections) {
        if (sect != null) {
            if (!isFirstPrinted) {
                builder.append("\n")
            }
            builder.append(sect.toCode())
            isFirstPrinted = false
        }
    }
    return builder.toString()
}

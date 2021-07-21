/*
 * Copyright 2019 Google LLC
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

package mathlingua.backend

import mathlingua.backend.transform.Signature
import mathlingua.backend.transform.checkVars
import mathlingua.backend.transform.expandAsWritten
import mathlingua.backend.transform.getVars
import mathlingua.backend.transform.locateAllSignatures
import mathlingua.backend.transform.normalize
import mathlingua.backend.transform.signature
import mathlingua.cli.VirtualFile
import mathlingua.frontend.FrontEnd
import mathlingua.frontend.Parse
import mathlingua.frontend.chalktalk.phase1.ast.Abstraction
import mathlingua.frontend.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Token
import mathlingua.frontend.chalktalk.phase1.ast.Tuple
import mathlingua.frontend.chalktalk.phase1.newChalkTalkLexer
import mathlingua.frontend.chalktalk.phase1.newChalkTalkParser
import mathlingua.frontend.chalktalk.phase2.HtmlCodeWriter
import mathlingua.frontend.chalktalk.phase2.MathLinguaCodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.Document
import mathlingua.frontend.chalktalk.phase2.ast.clause.AbstractionNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.AssignmentNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.ClauseListNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Target
import mathlingua.frontend.chalktalk.phase2.ast.clause.TupleNode
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.ThenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.HasUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.CalledSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesMeansGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.MeansSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.FoundationGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.ConjectureGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremSection
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.LocationTracker
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.Validation
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.support.newLocationTracker
import mathlingua.frontend.support.validationSuccess
import mathlingua.frontend.textalk.ColonColonEqualsTexTalkNode
import mathlingua.frontend.textalk.ColonEqualsTexTalkNode
import mathlingua.frontend.textalk.Command
import mathlingua.frontend.textalk.ExpressionTexTalkNode
import mathlingua.frontend.textalk.GroupTexTalkNode
import mathlingua.frontend.textalk.OperatorTexTalkNode
import mathlingua.frontend.textalk.TexTalkNode
import mathlingua.frontend.textalk.TexTalkNodeType
import mathlingua.frontend.textalk.TexTalkTokenType
import mathlingua.frontend.textalk.TextTexTalkNode
import mathlingua.frontend.textalk.isOpChar
import mathlingua.frontend.textalk.newTexTalkLexer
import mathlingua.frontend.textalk.newTexTalkParser

data class SourceFile(
    val file: VirtualFile, val content: String, val validation: Validation<Parse>)

fun isMathLinguaFile(file: VirtualFile) =
    !file.isDirectory() && file.absolutePath().last().endsWith(".math")

private fun buildSourceFile(file: VirtualFile): SourceFile {
    val content = file.readText()
    return SourceFile(
        file = file, content = content, validation = FrontEnd.parseWithLocations(content))
}

data class ValueSourceTracker<T>(
    val value: T, val source: SourceFile, val tracker: MutableLocationTracker?)

interface SourceCollection {
    fun size(): Int
    fun getDefinedSignatures(): Set<ValueSourceTracker<Signature>>
    fun getDuplicateDefinedSignatures(): List<ValueSourceTracker<Signature>>
    fun getUndefinedSignatures(): Set<ValueSourceTracker<Signature>>
    fun findInvalidTypes(): List<ValueSourceTracker<ParseError>>
    fun getParseErrors(): List<ValueSourceTracker<ParseError>>
    fun getDuplicateContent(): List<ValueSourceTracker<TopLevelGroup>>
    fun getSymbolErrors(): List<ValueSourceTracker<ParseError>>
    fun prettyPrint(
        file: VirtualFile, html: Boolean, literal: Boolean, doExpand: Boolean
    ): Pair<List<Pair<String, Phase2Node?>>, List<ParseError>>
}

fun newSourceCollection(filesOrDirs: List<VirtualFile>): SourceCollection {
    val sources = mutableListOf<SourceFile>()
    for (file in filesOrDirs) {
        findVirtualFiles(file, sources)
    }
    return SourceCollectionImpl(sources)
}

private fun findVirtualFiles(file: VirtualFile, result: MutableList<SourceFile>) {
    if (isMathLinguaFile(file)) {
        result.add(buildSourceFile(file))
    }
    for (child in file.listFiles()) {
        findVirtualFiles(child, result)
    }
}

class SourceCollectionImpl(sources: List<SourceFile>) : SourceCollection {
    private val sourceFiles = mutableMapOf<String, SourceFile>()
    private val allGroups = mutableListOf<ValueSourceTracker<TopLevelGroup>>()
    private val definesGroups = mutableListOf<ValueSourceTracker<DefinesGroup>>()
    private val statesGroups = mutableListOf<ValueSourceTracker<StatesGroup>>()
    private val axiomGroups = mutableListOf<ValueSourceTracker<AxiomGroup>>()
    private val theoremGroups = mutableListOf<ValueSourceTracker<TheoremGroup>>()
    private val conjectureGroups = mutableListOf<ValueSourceTracker<ConjectureGroup>>()
    private val foundationGroups = mutableListOf<ValueSourceTracker<FoundationGroup>>()

    init {
        for (sf in sources) {
            sourceFiles[sf.file.absolutePath().joinToString("/")] = sf
        }

        for (sf in sources) {
            val validation = sf.validation
            if (validation is ValidationSuccess) {
                definesGroups.addAll(
                    validation.value.document.defines().map {
                        ValueSourceTracker(
                            source = sf,
                            tracker = validation.value.tracker,
                            value = normalize(it, validation.value.tracker) as DefinesGroup)
                    })

                statesGroups.addAll(
                    validation.value.document.states().map {
                        ValueSourceTracker(
                            source = sf,
                            tracker = validation.value.tracker,
                            value = normalize(it, validation.value.tracker) as StatesGroup)
                    })

                axiomGroups.addAll(
                    validation.value.document.axioms().map {
                        ValueSourceTracker(
                            source = sf,
                            tracker = validation.value.tracker,
                            value = normalize(it, validation.value.tracker) as AxiomGroup)
                    })

                theoremGroups.addAll(
                    validation.value.document.theorems().map {
                        ValueSourceTracker(
                            source = sf,
                            tracker = validation.value.tracker,
                            value = normalize(it, validation.value.tracker) as TheoremGroup)
                    })

                conjectureGroups.addAll(
                    validation.value.document.conjectures().map {
                        ValueSourceTracker(
                            source = sf,
                            tracker = validation.value.tracker,
                            value = normalize(it, validation.value.tracker) as ConjectureGroup)
                    })

                foundationGroups.addAll(
                    validation.value.document.foundations().map {
                        ValueSourceTracker(
                            source = sf,
                            tracker = validation.value.tracker,
                            value = normalize(it, validation.value.tracker) as FoundationGroup)
                    })

                allGroups.addAll(
                    validation.value.document.groups.map {
                        ValueSourceTracker(
                            source = sf,
                            tracker = validation.value.tracker,
                            value = normalize(it, validation.value.tracker) as TopLevelGroup)
                    })
            }
        }
    }

    private fun getOperatorIdentifiers(node: Phase2Node): Set<String> {
        val result = mutableSetOf<String>()
        getOperatorIdentifiersImpl(node, result)
        return result
    }

    private fun maybeAddAbstractionAsOperatorIdentifier(
        abs: Abstraction, result: MutableSet<String>
    ) {
        if (!abs.isEnclosed &&
            !abs.isVarArgs &&
            abs.subParams == null &&
            abs.parts.size == 1 &&
            abs.parts[0].params == null &&
            abs.parts[0].subParams == null &&
            abs.parts[0].tail == null &&
            abs.parts[0].name.type == ChalkTalkTokenType.Name &&
            isOperatorName(abs.parts[0].name.text)) {
            // it is of a 'simple' form like *, **, etc.
            result.add(abs.parts[0].name.text)
        }
    }

    private fun maybeAddTupleAsOperator(tuple: Tuple, result: MutableSet<String>) {
        for (item in tuple.items) {
            if (item is Abstraction) {
                maybeAddAbstractionAsOperatorIdentifier(item, result)
            }
        }
    }

    private fun getOperatorIdentifiersImpl(node: Phase2Node, result: MutableSet<String>) {
        if (node is AbstractionNode) {
            maybeAddAbstractionAsOperatorIdentifier(node.abstraction, result)
        } else if (node is TupleNode) {
            maybeAddTupleAsOperator(node.tuple, result)
        } else if (node is AssignmentNode) {
            val assign = node.assignment
            if (assign.rhs is Abstraction) {
                maybeAddAbstractionAsOperatorIdentifier(assign.rhs, result)
            } else if (assign.rhs is Tuple) {
                maybeAddTupleAsOperator(assign.rhs, result)
            }

            if (assign.lhs.type == ChalkTalkTokenType.Name && isOperatorName(assign.lhs.text)) {
                result.add(assign.lhs.text)
            }
        }
        node.forEach { getOperatorIdentifiersImpl(it, result) }
    }

    private fun getInnerDefinedSignatures(clauses: List<Clause>): Set<String> {
        val result = mutableSetOf<String>()
        for (clause in clauses) {
            if (clause is Statement) {
                when (val validation = clause.texTalkRoot
                ) {
                    is ValidationSuccess -> {
                        getInnerDefinedSignaturesImpl(validation.value, false, result)
                    }
                }
            }
        }
        return result
    }

    private fun getInnerDefinedSignaturesImpl(
        node: TexTalkNode, isInColonEquals: Boolean, result: MutableSet<String>
    ) {
        val isColonEquals = node is ColonEqualsTexTalkNode || node is ColonColonEqualsTexTalkNode
        if (node is TextTexTalkNode && isOperatorName(node.text)) {
            result.add(node.text)
        }
        node.forEach { getInnerDefinedSignaturesImpl(it, isInColonEquals || isColonEquals, result) }
    }

    private fun getUsingDefinedName(node: ExpressionTexTalkNode): String? {
        return if (node.children.size == 1 &&
            node.children[0] is ColonEqualsTexTalkNode &&
            (node.children[0] as ColonEqualsTexTalkNode).lhs.items.size == 1 &&
            (node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children.size == 1 &&
            (node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
                0] is GroupTexTalkNode &&
            ((node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
                    0] as GroupTexTalkNode)
                .parameters
                .items
                .size == 1 &&
            ((node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
                        0] as GroupTexTalkNode)
                    .parameters
                    .items[0]
                .children
                .size == 1 &&
            ((node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
                        0] as GroupTexTalkNode)
                    .parameters
                    .items[0]
                .children[0] is TextTexTalkNode) {
            // match f(x) := ...
            (((node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
                            0] as GroupTexTalkNode)
                        .parameters
                        .items[0]
                    .children[0] as TextTexTalkNode)
                .text
        } else if (node.children.size == 1 &&
            node.children[0] is ColonEqualsTexTalkNode &&
            (node.children[0] as ColonEqualsTexTalkNode).lhs.items.size == 1 &&
            (node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children.size == 1 &&
            (node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
                0] is TextTexTalkNode) {
            // match f := ...
            ((node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
                    0] as TextTexTalkNode)
                .text
        } else {
            null
        }
    }

    private fun getUsingDefinedSignature(node: ExpressionTexTalkNode): String? {
        return if (node.children.size == 1 &&
            node.children[0] is ColonEqualsTexTalkNode &&
            (node.children[0] as ColonEqualsTexTalkNode).lhs.items.size == 1 &&
            (node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children.size == 1 &&
            (node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
                0] is GroupTexTalkNode &&
            ((node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
                    0] as GroupTexTalkNode)
                .parameters
                .items
                .size == 1 &&
            ((node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
                        0] as GroupTexTalkNode)
                    .parameters
                    .items[0]
                .children
                .size == 1 &&
            ((node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
                        0] as GroupTexTalkNode)
                    .parameters
                    .items[0]
                .children[0] is OperatorTexTalkNode &&
            (((node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
                            0] as GroupTexTalkNode)
                        .parameters
                        .items[0]
                    .children[0] as OperatorTexTalkNode)
                .command is TextTexTalkNode &&
            ((((node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
                                0] as GroupTexTalkNode)
                            .parameters
                            .items[0]
                        .children[0] as OperatorTexTalkNode)
                    .command as TextTexTalkNode)
                .tokenType == TexTalkTokenType.Operator) {
            // match -f := ...
            ((((node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
                                0] as GroupTexTalkNode)
                            .parameters
                            .items[0]
                        .children[0] as OperatorTexTalkNode)
                    .command as TextTexTalkNode)
                .text
        } else if (node.children.isNotEmpty() &&
            node.children[0] is ColonEqualsTexTalkNode &&
            (node.children[0] as ColonEqualsTexTalkNode).lhs.items.isNotEmpty() &&
            (node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children.isNotEmpty() &&
            (node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
                0] is OperatorTexTalkNode &&
            ((node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
                    0] as OperatorTexTalkNode)
                .command is TextTexTalkNode) {
            // match `a + b := ...`
            (((node.children[0] as ColonEqualsTexTalkNode).lhs.items[0].children[
                        0] as OperatorTexTalkNode)
                    .command as TextTexTalkNode)
                .text
        } else if (node.children.size == 1 && node.children[0] is ColonEqualsTexTalkNode) {
            val colonEquals = node.children[0] as ColonEqualsTexTalkNode
            val lhs = colonEquals.lhs.items.firstOrNull()
            if (lhs != null) {
                IdStatement(text = lhs.toCode(), texTalkRoot = validationSuccess(lhs))
                    .signature(newLocationTracker())
                    ?.form
            } else {
                null
            }
        } else {
            null
        }
    }

    private fun getOperatorIdentifiersFromTargets(
        targets: List<Target>, tracker: LocationTracker?
    ): List<Signature> {
        val result = mutableListOf<Signature>()
        for (target in targets) {
            for (op in getOperatorIdentifiers(target)) {
                result.add(
                    Signature(
                        form = op,
                        location = tracker?.getLocationOf(target)
                                ?: Location(row = -1, column = -1)))
            }
        }
        return result
    }

    // an 'inner' signature is a signature that is only within scope of the given top level group
    private fun getInnerDefinedSignatures(
        group: TopLevelGroup, tracker: LocationTracker?
    ): Set<Signature> {
        val result = mutableSetOf<Signature>()

        val usingSection =
            when (group) {
                is DefinesGroup -> {
                    group.usingSection
                }
                is StatesGroup -> {
                    group.usingSection
                }
                is TheoremGroup -> {
                    group.usingSection
                }
                is ConjectureGroup -> {
                    group.usingSection
                }
                is AxiomGroup -> {
                    group.usingSection
                }
                else -> {
                    null
                }
            }

        if (usingSection != null) {
            for (clause in usingSection.clauses.clauses) {
                if (clause is Statement) {
                    val location = tracker?.getLocationOf(clause) ?: Location(-1, -1)
                    when (val validation = clause.texTalkRoot
                    ) {
                        is ValidationSuccess -> {
                            val sigForm = getUsingDefinedSignature(validation.value)
                            if (sigForm != null) {
                                result.add(Signature(form = sigForm, location = location))
                            }
                        }
                    }
                }
            }
        }

        if (group is DefinesGroup) {
            if (group.whenSection != null) {
                val location = tracker?.getLocationOf(group.whenSection!!) ?: Location(-1, -1)
                result.addAll(
                    getInnerDefinedSignatures(group.whenSection!!.clauses.clauses).map {
                        Signature(form = it, location = location)
                    })
            }

            result.addAll(getOperatorIdentifiersFromTargets(group.definesSection.targets, tracker))
        } else if (group is StatesGroup) {
            if (group.whenSection != null) {
                val location = tracker?.getLocationOf(group.whenSection!!) ?: Location(-1, -1)
                result.addAll(
                    getInnerDefinedSignatures(group.whenSection!!.clauses.clauses).map {
                        Signature(form = it, location = location)
                    })
            }
        } else if (group is TheoremGroup) {
            if (group.givenSection != null) {
                result.addAll(
                    getOperatorIdentifiersFromTargets(group.givenSection.targets, tracker))
            }
        } else if (group is AxiomGroup) {
            if (group.givenSection != null) {
                result.addAll(
                    getOperatorIdentifiersFromTargets(group.givenSection.targets, tracker))
            }
        } else if (group is ConjectureGroup) {
            if (group.givenSection != null) {
                result.addAll(
                    getOperatorIdentifiersFromTargets(group.givenSection.targets, tracker))
            }
        }
        return result
    }

    private fun getAllDefinedSignatures():
        List<Pair<ValueSourceTracker<Signature>, TopLevelGroup>> {
        val result = mutableListOf<Pair<ValueSourceTracker<Signature>, TopLevelGroup>>()

        fun processDefines(pair: ValueSourceTracker<DefinesGroup>) {
            val signature = pair.value.signature
            if (signature != null) {
                val location =
                    pair.tracker?.getLocationOf(pair.value) ?: Location(row = -1, column = -1)
                val vst =
                    ValueSourceTracker(
                        source = pair.source, tracker = pair.tracker, value = signature)
                result.add(Pair(vst, pair.value))
            }
        }

        fun processStates(pair: ValueSourceTracker<StatesGroup>) {
            val signature = pair.value.signature
            if (signature != null) {
                val vst =
                    ValueSourceTracker(
                        source = pair.source, tracker = pair.tracker, value = signature)
                result.add(Pair(vst, pair.value))
            }
        }

        fun processAxiom(pair: ValueSourceTracker<AxiomGroup>) {
            val signature = pair.value.signature
            if (signature != null) {
                val vst =
                    ValueSourceTracker(
                        source = pair.source, tracker = pair.tracker, value = signature)
                result.add(Pair(vst, pair.value))
            }
        }

        fun processTheorems(pair: ValueSourceTracker<TheoremGroup>) {
            val signature = pair.value.signature
            if (signature != null) {
                val vst =
                    ValueSourceTracker(
                        source = pair.source, tracker = pair.tracker, value = signature)
                result.add(Pair(vst, pair.value))
            }
        }

        fun processConjectures(pair: ValueSourceTracker<ConjectureGroup>) {
            val signature = pair.value.signature
            if (signature != null) {
                val vst =
                    ValueSourceTracker(
                        source = pair.source, tracker = pair.tracker, value = signature)
                result.add(Pair(vst, pair.value))
            }
        }

        for (pair in foundationGroups) {
            when (val item = pair.value.foundationSection.content
            ) {
                is DefinesGroup ->
                    processDefines(
                        ValueSourceTracker(
                            value = item, source = pair.source, tracker = pair.tracker))
                is StatesGroup ->
                    processStates(
                        ValueSourceTracker(
                            value = item, source = pair.source, tracker = pair.tracker))
            }
        }

        for (pair in definesGroups) {
            processDefines(pair)
        }

        for (pair in statesGroups) {
            processStates(pair)
        }

        for (pair in axiomGroups) {
            processAxiom(pair)
        }

        for (pair in theoremGroups) {
            processTheorems(pair)
        }

        for (pair in conjectureGroups) {
            processConjectures(pair)
        }

        return result
    }

    override fun size() = sourceFiles.size

    override fun getDefinedSignatures(): Set<ValueSourceTracker<Signature>> {
        val result = mutableSetOf<ValueSourceTracker<Signature>>()
        result.addAll(getAllDefinedSignatures().map { it.first })
        return result
    }

    override fun getDuplicateDefinedSignatures(): List<ValueSourceTracker<Signature>> {
        val duplicates = mutableListOf<ValueSourceTracker<Signature>>()
        val found = mutableSetOf<String>()
        for (sig in getAllDefinedSignatures().map { it.first }) {
            if (found.contains(sig.value.form)) {
                duplicates.add(sig)
            }
            found.add(sig.value.form)
        }
        return duplicates
    }

    override fun getUndefinedSignatures(): Set<ValueSourceTracker<Signature>> {
        val result = mutableSetOf<ValueSourceTracker<Signature>>()
        val globalDefinedSigs = getDefinedSignatures().map { it.value.form }.toSet()
        for (vst in allGroups) {
            val innerSigs =
                getInnerDefinedSignatures(vst.value, vst.tracker).map { it.form }.toSet()
            val usedSigs =
                locateAllSignatures(
                    vst.value, ignoreLhsEqual = true, vst.tracker ?: newLocationTracker())
            for (sig in usedSigs) {
                if (!globalDefinedSigs.contains(sig.form) && !innerSigs.contains(sig.form)) {
                    result.add(
                        ValueSourceTracker(value = sig, source = vst.source, tracker = vst.tracker))
                }
            }
        }
        return result
    }

    override fun findInvalidTypes(): List<ValueSourceTracker<ParseError>> {
        val result = mutableListOf<ValueSourceTracker<ParseError>>()
        val defs =
            getAllDefinedSignatures().filter { it.second is DefinesGroup }.map {
                Pair(it.first, it.second as DefinesGroup)
            }
        val analyzer = SymbolAnalyzer(defs)
        for (sf in sourceFiles) {
            val lexer = newChalkTalkLexer(sf.value.content)
            val parse = newChalkTalkParser().parse(lexer)
            result.addAll(
                parse.errors.map {
                    ValueSourceTracker(
                        source = sf.value,
                        tracker = null,
                        value =
                            ParseError(
                                message = it.message,
                                row = it.row.coerceAtLeast(0),
                                column = it.column.coerceAtLeast(0)))
                })

            val root = parse.root
            if (root != null) {
                for (stmtNode in findAllPhase1Statements(root)) {
                    val text = stmtNode.text.removeSurrounding("'", "'")
                    val texTalkLexer = newTexTalkLexer(text)
                    val texTalkResult = newTexTalkParser().parse(texTalkLexer)
                    result.addAll(
                        texTalkResult.errors.map {
                            ValueSourceTracker(
                                source = sf.value,
                                tracker = null,
                                value =
                                    ParseError(
                                        message = it.message,
                                        row =
                                            stmtNode.row.coerceAtLeast(0) + it.row.coerceAtLeast(0),
                                        column =
                                            stmtNode.column.coerceAtLeast(0) +
                                                it.column.coerceAtLeast(0)))
                        })
                }
            }

            val validation = sf.value.validation
            when (validation) {
                is ValidationSuccess -> {
                    val tracker = validation.value.tracker
                    val doc = validation.value.document
                    for (grp in doc.groups) {
                        val errors = analyzer.findInvalidTypes(grp, tracker)
                        result.addAll(
                            errors.map {
                                ValueSourceTracker(
                                    source = sf.value,
                                    tracker = tracker,
                                    value =
                                        ParseError(
                                            message = it.message,
                                            row = it.row.coerceAtLeast(0),
                                            column = it.column.coerceAtLeast(0)))
                            })
                    }

                    for (pair in findAllStatements(doc)) {
                        val stmt = pair.first
                        val aliasDefines = pair.second
                        val location = tracker.getLocationOf(stmt) ?: Location(row = 0, column = 0)
                        for (node in findAllTexTalkNodes(stmt)) {
                            val expansion =
                                expandAsWritten(
                                    node,
                                    getPatternsToWrittenAs(
                                        defines = definesGroups.map { it.value }.plus(aliasDefines),
                                        states = statesGroups.map { it.value },
                                        axioms = axiomGroups.map { it.value },
                                        foundations = foundationGroups.map { it.value }))
                            result.addAll(
                                expansion.errors.map {
                                    ValueSourceTracker(
                                        source = sf.value,
                                        tracker = tracker,
                                        value =
                                            ParseError(
                                                message = it,
                                                row = location.row.coerceAtLeast(0),
                                                column = location.column.coerceAtLeast(0)))
                                })
                        }
                    }
                }
            }
        }
        return result
    }

    private fun findAllPhase1Statements(node: Phase1Node): List<Phase1Token> {
        val result = mutableListOf<Phase1Token>()
        findAllPhase1StatementsImpl(node, result)
        return result
    }

    private fun findAllPhase1StatementsImpl(node: Phase1Node, result: MutableList<Phase1Token>) {
        if (node is Phase1Token && node.type == ChalkTalkTokenType.Statement) {
            result.add(node)
        }
        node.forEach { findAllPhase1StatementsImpl(it, result) }
    }

    private fun findAllStatements(node: Phase2Node): List<Pair<Statement, List<DefinesGroup>>> {
        val pairs = mutableListOf<Pair<Statement, HasUsingSection?>>()
        findAllStatementsImpl(
            node,
            if (node is HasUsingSection) {
                node
            } else {
                null
            },
            pairs)
        return pairs.map {
            val aliases = mutableListOf<DefinesGroup>()
            val usingSection = it.second?.usingSection
            if (usingSection != null) {
                for (clause in usingSection.clauses.clauses) {
                    if (clause is Statement &&
                        clause.texTalkRoot is ValidationSuccess &&
                        clause.texTalkRoot.value.children.firstOrNull() is ColonEqualsTexTalkNode) {

                        val colonEquals =
                            clause.texTalkRoot.value.children.first() as ColonEqualsTexTalkNode
                        val lhsItems = colonEquals.lhs.items
                        val rhsItems = colonEquals.rhs.items
                        if (lhsItems.size != rhsItems.size) {
                            throw RuntimeException(
                                "The left-hand-side and right-hand-side of a := must have the same number of " +
                                    "comma separated expressions")
                        }
                        for (i in lhsItems.indices) {
                            val lhs = lhsItems[i]
                            val rhs = rhsItems[i]
                            val id =
                                IdStatement(
                                    text = lhs.toCode(), texTalkRoot = validationSuccess(lhs))
                            val syntheticDefines =
                                DefinesMeansGroup(
                                    signature = id.signature(newLocationTracker()),
                                    id = id,
                                    definesSection = DefinesSection(targets = emptyList()),
                                    requiringSection = null,
                                    whenSection = null,
                                    meansSection =
                                        MeansSection(
                                            clauses = ClauseListNode(clauses = emptyList())),
                                    satisfyingSection = null,
                                    viewingSection = null,
                                    usingSection = null,
                                    writtenSection = WrittenSection(forms = listOf(rhs.toCode())),
                                    calledSection = CalledSection(forms = emptyList()),
                                    metaDataSection = null)
                            aliases.add(syntheticDefines)
                        }
                    }
                }
            }
            Pair(first = it.first, second = aliases)
        }
    }

    private fun findAllStatementsImpl(
        node: Phase2Node,
        hasUsingNode: HasUsingSection?,
        result: MutableList<Pair<Statement, HasUsingSection?>>
    ) {
        if (node is Statement) {
            result.add(Pair(node, hasUsingNode))
        }
        node.forEach {
            findAllStatementsImpl(
                it,
                if (hasUsingNode != null) {
                    hasUsingNode
                } else {
                    if (node is HasUsingSection) {
                        node
                    } else {
                        null
                    }
                },
                result)
        }
    }

    private fun findAllTexTalkNodes(node: Phase2Node): List<TexTalkNode> {
        val result = mutableListOf<TexTalkNode>()
        findAllTexTalkNodesImpl(node, result)
        return result
    }

    private fun findAllTexTalkNodesImpl(node: Phase2Node, result: MutableList<TexTalkNode>) {
        if (node is Statement) {
            when (val validation = node.texTalkRoot
            ) {
                is ValidationSuccess -> {
                    result.add(validation.value)
                }
            }
        }
        node.forEach { findAllTexTalkNodesImpl(it, result) }
    }

    override fun getParseErrors(): List<ValueSourceTracker<ParseError>> {
        val result = mutableListOf<ValueSourceTracker<ParseError>>()
        for (sf in sourceFiles) {
            val validation = sf.value.validation
            when (validation) {
                is ValidationFailure -> {
                    result.addAll(
                        validation.errors.map {
                            ValueSourceTracker(source = sf.value, tracker = null, value = it)
                        })
                }
                is ValidationSuccess -> {
                    val doc = validation.value.document
                    val foundations = doc.foundations()

                    val allDefines = mutableListOf<DefinesGroup>()
                    allDefines.addAll(doc.defines())
                    allDefines.addAll(
                        foundations
                            .map { it.foundationSection.content }
                            .filterIsInstance<DefinesGroup>())

                    val allStates = mutableListOf<StatesGroup>()
                    allStates.addAll(doc.states())
                    allStates.addAll(
                        foundations
                            .map { it.foundationSection.content }
                            .filterIsInstance<StatesGroup>())

                    val allSigRoot = mutableListOf<Validation<TexTalkNode>>()
                    allSigRoot.addAll(allDefines.map { it.id.texTalkRoot })
                    allSigRoot.addAll(allStates.map { it.id.texTalkRoot })

                    for (vald in allSigRoot) {
                        if (vald is ValidationFailure) {
                            result.addAll(
                                vald.errors.map {
                                    ValueSourceTracker(
                                        source = sf.value, tracker = null, value = it)
                                })
                        }
                    }
                }
            }
        }
        return result
    }

    override fun getDuplicateContent(): List<ValueSourceTracker<TopLevelGroup>> {
        val result = mutableListOf<ValueSourceTracker<TopLevelGroup>>()
        val allContent = mutableSetOf<String>()
        for (group in allGroups) {
            val content = group.value.toCode(false, 0).getCode().trim()
            if (allContent.contains(content)) {
                result.add(
                    ValueSourceTracker(
                        source = group.source, tracker = group.tracker, value = group.value))
            }
            allContent.add(content)
        }
        return result
    }

    override fun prettyPrint(
        file: VirtualFile, html: Boolean, literal: Boolean, doExpand: Boolean
    ): Pair<List<Pair<String, Phase2Node?>>, List<ParseError>> {
        val sourceFile = sourceFiles[file.absolutePath().joinToString("/")]
        return if (sourceFile != null) {
            prettyPrint(sourceFile.validation, html, literal, doExpand)
        } else {
            prettyPrint(file.readText(), html, literal, doExpand)
        }
    }

    private fun prettyPrint(
        input: String, html: Boolean, literal: Boolean, doExpand: Boolean
    ): Pair<List<Pair<String, Phase2Node?>>, List<ParseError>> {
        return prettyPrint(FrontEnd.parseWithLocations(input), html, literal, doExpand)
    }

    private fun prettyPrint(
        validation: Validation<Parse>, html: Boolean, literal: Boolean, doExpand: Boolean
    ): Pair<List<Pair<String, Phase2Node?>>, List<ParseError>> {
        val content: List<Pair<String, Phase2Node?>> =
            when (validation) {
                is ValidationFailure -> {
                    listOf(
                        Pair(
                            if (html) {
                                val builder = StringBuilder()
                                builder.append(
                                    "<html><head><style>.content { font-size: 1em; }" +
                                        "</style></head><body class='content'><ul>")
                                for (err in validation.errors) {
                                    builder.append(
                                        "<li><span style='color: #e61919;'>ERROR:</span> " +
                                            "${err.message} (${err.row + 1}, ${err.column + 1})</li>")
                                }
                                builder.append("</ul></body></html>")
                                builder.toString()
                            } else {
                                val builder = StringBuilder()
                                for (err in validation.errors) {
                                    builder.append(
                                        "ERROR: ${err.message} (${err.row + 1}, ${err.column + 1})\n")
                                }
                                builder.toString()
                            },
                            null))
                }
                is ValidationSuccess ->
                    prettyPrint(
                        doc = validation.value.document,
                        html = html,
                        literal = literal,
                        doExpand = doExpand)
            }

        return when (validation) {
            is ValidationFailure -> Pair(content, validation.errors)
            is ValidationSuccess -> Pair(content, emptyList())
        }
    }

    private fun prettyPrint(doc: Document, html: Boolean, literal: Boolean, doExpand: Boolean) =
        doc.groups.map { Pair(prettyPrint(it, html, literal, doExpand), it) }

    private fun getWriter(
        html: Boolean, literal: Boolean, doExpand: Boolean, aliasDefines: List<DefinesGroup>
    ) =
        if (html) {
            HtmlCodeWriter(
                defines = doExpand.thenUse { definesGroups.map { it.value }.plus(aliasDefines) },
                states = doExpand.thenUse { statesGroups.map { it.value } },
                axioms = doExpand.thenUse { axiomGroups.map { it.value } },
                foundations = doExpand.thenUse { foundationGroups.map { it.value } },
                literal = literal)
        } else {
            MathLinguaCodeWriter(
                defines = doExpand.thenUse { definesGroups.map { it.value }.plus(aliasDefines) },
                states = doExpand.thenUse { statesGroups.map { it.value } },
                axioms = doExpand.thenUse { axiomGroups.map { it.value } },
                foundations = doExpand.thenUse { foundationGroups.map { it.value } })
        }

    private fun prettyPrint(
        node: Phase2Node, html: Boolean, literal: Boolean, doExpand: Boolean
    ): String {
        val aliasDefines = mutableListOf<DefinesGroup>()
        // any alias such as `x \op/ y := ...` or `\f(x) := ...`
        // needs to be handled so that when being pretty-printed, the pretty printer
        // acts as if there is a signature `x \op/ y` (for example) with a written as
        // section that is the expanded version of the right-hand-side of the :=
        // This allows alias in `using:` sections to have their usages expanded correctly
        if (node is HasUsingSection) {
            val usingSection = node.usingSection
            if (usingSection != null) {
                for (clause in usingSection.clauses.clauses) {
                    if (clause is Statement &&
                        clause.texTalkRoot is ValidationSuccess &&
                        clause.texTalkRoot.value.children.firstOrNull() is ColonEqualsTexTalkNode) {

                        val colonEquals =
                            clause.texTalkRoot.value.children.first() as ColonEqualsTexTalkNode
                        val lhsItems = colonEquals.lhs.items
                        val rhsItems = colonEquals.rhs.items
                        if (lhsItems.size != 1 || rhsItems.size != 1) {
                            throw RuntimeException(
                                "The left-hand-side and right-hand-side of a := must have exactly one expression")
                        }

                        // Given the statment: '\f(x) := \g(x)'
                        // then lhs is `\f(x)`
                        // and lhsVars is the set containing only `x`
                        val lhs = lhsItems[0]
                        val lhsVars = getVars(texTalkNode = lhs, ignoreParens = false).toSet()

                        // convert the right hand side from `\g(x)` to `\g(x?)`
                        // to conform to the way "writtenAs" sections are written
                        val rhs =
                            rhsItems[0].transform {
                                if (it is TextTexTalkNode &&
                                    it.type == TexTalkNodeType.Identifier &&
                                    it.tokenType == TexTalkTokenType.Identifier &&
                                    lhsVars.contains(it.text)) {
                                    it.copy(
                                        // use toCode() so ... is printed if the identifier is
                                        // vararg
                                        text = "${it.toCode()}?")
                                } else {
                                    it
                                }
                            } as ExpressionTexTalkNode

                        val writer =
                            getWriter(
                                html = false,
                                literal = false,
                                doExpand = true,
                                aliasDefines = aliasDefines)
                        val tmpTheorem =
                            TheoremGroup(
                                signature = null,
                                id = null,
                                theoremSection = TheoremSection(names = emptyList()),
                                givenSection = null,
                                givenWhereSection = null,
                                ifOrIffSection = null,
                                thenSection =
                                    ThenSection(
                                        clauses =
                                            ClauseListNode(
                                                clauses =
                                                    listOf(
                                                        Statement(
                                                            text = rhs.toCode(),
                                                            texTalkRoot =
                                                                validationSuccess(rhs))))),
                                proofSection = null,
                                usingSection = null,
                                metaDataSection = null)
                        val expanded = tmpTheorem.toCode(false, 0, writer).getCode()
                        when (val validation = FrontEnd.parse(expanded)
                        ) {
                            is ValidationSuccess -> {
                                val doc = validation.value
                                val stmt =
                                    doc.theorems()[0].thenSection.clauses.clauses[0] as Statement
                                val id =
                                    IdStatement(
                                        text = lhs.toCode(), texTalkRoot = validationSuccess(lhs))
                                val syntheticDefines =
                                    DefinesMeansGroup(
                                        signature = id.signature(newLocationTracker()),
                                        id = id,
                                        definesSection = DefinesSection(targets = emptyList()),
                                        requiringSection = null,
                                        whenSection = null,
                                        meansSection =
                                            MeansSection(
                                                clauses = ClauseListNode(clauses = emptyList())),
                                        satisfyingSection = null,
                                        viewingSection = null,
                                        usingSection = null,
                                        writtenSection =
                                            WrittenSection(forms = listOf("\"${stmt.text}\"")),
                                        calledSection = CalledSection(forms = emptyList()),
                                        metaDataSection = null)
                                aliasDefines.add(syntheticDefines)
                            }
                        }
                    }
                }
            }
        }

        return getWriter(html, literal, doExpand, aliasDefines).generateCode(node)
    }

    override fun getSymbolErrors(): List<ValueSourceTracker<ParseError>> {
        val result = mutableListOf<ValueSourceTracker<ParseError>>()
        for (grp in allGroups) {
            val tracker = grp.tracker ?: newLocationTracker()
            val errs = checkVars(grp.value, tracker)
            result.addAll(
                errs.map { ValueSourceTracker(value = it, source = grp.source, tracker = tracker) })
        }
        return result
    }
}

fun getPatternsToWrittenAs(
    defines: List<DefinesGroup>,
    states: List<StatesGroup>,
    axioms: List<AxiomGroup>,
    foundations: List<FoundationGroup>
): Map<OperatorTexTalkNode, String> {
    val allDefines = mutableListOf<DefinesGroup>()
    allDefines.addAll(defines)

    val allStates = mutableListOf<StatesGroup>()
    allStates.addAll(states)

    for (f in foundations) {
        val content = f.foundationSection.content
        if (content is DefinesGroup) {
            allDefines.add(content)
        } else if (content is StatesGroup) {
            allStates.add(content)
        }
    }

    val result = mutableMapOf<OperatorTexTalkNode, String>()
    for (rep in allStates) {
        val writtenAs =
            rep.writtenSection?.forms?.getOrNull(0)?.removeSurrounding("\"", "\"") ?: continue

        val validation = rep.id.texTalkRoot
        if (validation is ValidationSuccess) {
            val exp = validation.value
            if (exp.children.size == 1 && exp.children[0] is OperatorTexTalkNode) {
                result[exp.children[0] as OperatorTexTalkNode] = writtenAs
            } else if (exp.children.size == 1 && exp.children[0] is Command) {
                result[
                    OperatorTexTalkNode(
                        lhs = null, command = exp.children[0] as Command, rhs = null)] = writtenAs
            }
        }
    }

    for (axiom in axioms) {
        val validation = axiom.id?.texTalkRoot
        if (validation is ValidationSuccess) {
            val exp = validation.value
            val name = axiom.axiomSection.names.getOrNull(0)
            val writtenAs =
                if (name != null) {
                    "\\textrm{${name.removeSurrounding("\"", "\"")}}"
                } else {
                    exp.toCode()
                }
            if (exp.children.size == 1 && exp.children[0] is OperatorTexTalkNode) {
                result[exp.children[0] as OperatorTexTalkNode] = writtenAs
            } else if (exp.children.size == 1 && exp.children[0] is Command) {
                val cmd = exp.children[0] as Command
                result[OperatorTexTalkNode(lhs = null, command = cmd, rhs = null)] = writtenAs
            }
        }
    }

    for (def in allDefines) {
        val writtenAs =
            def.writtenSection.forms.getOrNull(0)?.removeSurrounding("\"", "\"") ?: continue

        val validation = def.id.texTalkRoot
        if (validation is ValidationSuccess) {
            val exp = validation.value
            if (exp.children.size == 1 && exp.children[0] is OperatorTexTalkNode) {
                result[exp.children[0] as OperatorTexTalkNode] = writtenAs
            } else if (exp.children.size == 1 && exp.children[0] is Command) {
                val cmd = exp.children[0] as Command
                result[OperatorTexTalkNode(lhs = null, command = cmd, rhs = null)] = writtenAs
            }
        }
    }

    return result
}

private fun <T> Boolean.thenUse(value: () -> List<T>) =
    if (this) {
        value()
    } else {
        emptyList()
    }

fun isOperatorName(text: String): Boolean {
    var index = text.indexOf('_')
    if (index < 0) {
        index = text.length
    }
    for (i in 0 until index) {
        if (!isOpChar(text[i])) {
            return false
        }
    }
    return true
}

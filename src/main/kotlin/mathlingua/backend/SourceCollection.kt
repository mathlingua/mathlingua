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

import java.io.File
import mathlingua.backend.transform.Signature
import mathlingua.backend.transform.checkVars
import mathlingua.backend.transform.expandAsWritten
import mathlingua.backend.transform.locateAllSignatures
import mathlingua.backend.transform.normalize
import mathlingua.frontend.FrontEnd
import mathlingua.frontend.Parse
import mathlingua.frontend.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Token
import mathlingua.frontend.chalktalk.phase1.newChalkTalkLexer
import mathlingua.frontend.chalktalk.phase1.newChalkTalkParser
import mathlingua.frontend.chalktalk.phase2.HtmlCodeWriter
import mathlingua.frontend.chalktalk.phase2.MathLinguaCodeWriter
import mathlingua.frontend.chalktalk.phase2.ast.clause.AbstractionNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.inductively.ConstantGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.inductively.ConstructorGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGeneratedGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.FoundationGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.mutually.MutuallyGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.Validation
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.support.newLocationTracker
import mathlingua.frontend.textalk.Command
import mathlingua.frontend.textalk.OperatorTexTalkNode
import mathlingua.frontend.textalk.TexTalkNode
import mathlingua.frontend.textalk.newTexTalkLexer
import mathlingua.frontend.textalk.newTexTalkParser

data class SourceFile(val file: File?, val content: String, val validation: Validation<Parse>)

fun isMathLinguaFile(file: File) = file.isFile && file.extension == "math"

private fun buildSourceFile(file: File): SourceFile {
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
    fun getUsedSignatures(): Set<ValueSourceTracker<Signature>>
    fun getUndefinedSignatures(): Set<ValueSourceTracker<Signature>>
    fun findInvalidTypes(): List<ValueSourceTracker<ParseError>>
    fun getParseErrors(): List<ValueSourceTracker<ParseError>>
    fun getDuplicateContent(): List<ValueSourceTracker<TopLevelGroup>>
    fun getSymbolErrors(): List<ValueSourceTracker<ParseError>>
    fun prettyPrint(file: File, html: Boolean, doExpand: Boolean): Pair<String, List<ParseError>>
    fun prettyPrint(
        input: String, html: Boolean, doExpand: Boolean
    ): Pair<String, List<ParseError>>
    fun prettyPrint(node: Phase2Node, html: Boolean, doExpand: Boolean): String
}

fun newSourceCollectionFromFiles(filesOrDirs: List<File>): SourceCollection {
    val sources = mutableListOf<SourceFile>()
    for (f in filesOrDirs) {
        if (f.isDirectory) {
            sources.addAll(f.walk().filter { isMathLinguaFile(it) }.map { buildSourceFile(it) })
        } else if (isMathLinguaFile(f)) {
            sources.add(buildSourceFile(f))
        }
    }
    return SourceCollectionImpl(sources)
}

fun newSourceCollectionFromContent(sources: List<String>): SourceCollection {
    val sourceFiles =
        sources.map {
            SourceFile(file = null, content = it, validation = FrontEnd.parseWithLocations(it))
        }
    return SourceCollectionImpl(sourceFiles)
}

class SourceCollectionImpl(sources: List<SourceFile>) : SourceCollection {
    private val sourceFiles = mutableMapOf<String, SourceFile>()
    private val allGroups = mutableListOf<ValueSourceTracker<TopLevelGroup>>()
    private val definesGroups = mutableListOf<ValueSourceTracker<DefinesGroup>>()
    private val statesGroups = mutableListOf<ValueSourceTracker<StatesGroup>>()
    private val foundationGroups = mutableListOf<ValueSourceTracker<FoundationGroup>>()
    private val mutuallyGroups = mutableListOf<ValueSourceTracker<MutuallyGroup>>()
    private val patternsToWrittenAs: Map<OperatorTexTalkNode, String>

    init {
        for (sf in sources) {
            if (sf.file != null) {
                sourceFiles[sf.file.normalize().canonicalPath] = sf
            }
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

                foundationGroups.addAll(
                    validation.value.document.foundations().map {
                        ValueSourceTracker(
                            source = sf,
                            tracker = validation.value.tracker,
                            value = normalize(it, validation.value.tracker) as FoundationGroup)
                    })

                mutuallyGroups.addAll(
                    validation.value.document.mutually().map {
                        ValueSourceTracker(
                            source = sf,
                            tracker = validation.value.tracker,
                            value = normalize(it, validation.value.tracker) as MutuallyGroup)
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

        patternsToWrittenAs =
            getPatternsToWrittenAs(
                defines = definesGroups.map { it.value },
                states = statesGroups.map { it.value },
                foundations = foundationGroups.map { it.value },
                mutuallyGroups = mutuallyGroups.map { it.value })
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
                        source = pair.source,
                        tracker = pair.tracker,
                        value = Signature(form = signature, location = location))
                result.add(Pair(vst, pair.value))

                when (val defines = pair.value
                ) {
                    is DefinesGeneratedGroup -> {
                        val from = defines.generatedSection.inductivelyGroup.fromSection
                        for (clause in from.clauses.clauses) {
                            if (clause is ConstantGroup) {
                                for (nameClause in clause.constantSection.clauses.clauses) {
                                    val name = nameClause.toCode(false, 0).getCode()
                                    val nameVst =
                                        ValueSourceTracker(
                                            source = pair.source,
                                            tracker = pair.tracker,
                                            value =
                                                Signature(
                                                    form = "$signature.$name", location = location))
                                    result.add(Pair(nameVst, pair.value))
                                }
                            } else if (clause is ConstructorGroup) {
                                val targets = clause.constructorSection.targets
                                for (target in targets) {
                                    if (target is AbstractionNode) {
                                        val tail =
                                            target
                                                .abstraction
                                                .toCode()
                                                .replace(Regex("\\(.*\\)"), "()")
                                        val absVst =
                                            ValueSourceTracker(
                                                source = pair.source,
                                                tracker = pair.tracker,
                                                value =
                                                    Signature(
                                                        form = "$signature.$tail",
                                                        location = location))
                                        result.add(Pair(absVst, pair.value))
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        fun processStates(pair: ValueSourceTracker<StatesGroup>) {
            val signature = pair.value.signature
            if (signature != null) {
                val vst =
                    ValueSourceTracker(
                        source = pair.source,
                        tracker = pair.tracker,
                        value =
                            Signature(
                                form = signature,
                                location = pair.tracker?.getLocationOf(pair.value)
                                        ?: Location(row = -1, column = -1)))
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

    override fun getUsedSignatures(): Set<ValueSourceTracker<Signature>> {
        val result = mutableSetOf<ValueSourceTracker<Signature>>()
        for (sf in sourceFiles) {
            val validation = sf.value.validation
            when (validation) {
                is ValidationSuccess -> {
                    val signatures =
                        locateAllSignatures(validation.value.document, validation.value.tracker)
                    result.addAll(
                        signatures.map {
                            ValueSourceTracker(
                                source = sf.value, tracker = validation.value.tracker, value = it)
                        })
                }
            }
        }
        return result
    }

    override fun getUndefinedSignatures(): Set<ValueSourceTracker<Signature>> {
        val usedSigs = getUsedSignatures()
        val definedSigs = getDefinedSignatures().map { it.value.form }.toSet()
        val result = mutableSetOf<ValueSourceTracker<Signature>>()
        for (sig in usedSigs) {
            if (!definedSigs.contains(sig.value.form)) {
                result.add(sig)
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

                    for (stmt in findAllStatements(doc)) {
                        val location = tracker.getLocationOf(stmt) ?: Location(row = 0, column = 0)
                        for (node in findAllTexTalkNodes(stmt)) {
                            val expansion = expandAsWritten(node, patternsToWrittenAs)
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

    private fun findAllStatements(node: Phase2Node): List<Statement> {
        val result = mutableListOf<Statement>()
        findAllStatementsImpl(node, result)
        return result
    }

    private fun findAllStatementsImpl(node: Phase2Node, result: MutableList<Statement>) {
        if (node is Statement) {
            result.add(node)
        }
        node.forEach { findAllStatementsImpl(it, result) }
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
            val content = group.value.toCode(false, 0).getCode()
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
        file: File, html: Boolean, doExpand: Boolean
    ): Pair<String, List<ParseError>> {
        val sourceFile = sourceFiles[file.normalize().canonicalPath]
        return if (sourceFile != null) {
            prettyPrint(sourceFile.validation, html, doExpand)
        } else {
            prettyPrint(file.readText(), html, doExpand)
        }
    }

    override fun prettyPrint(
        input: String, html: Boolean, doExpand: Boolean
    ): Pair<String, List<ParseError>> {
        return prettyPrint(FrontEnd.parseWithLocations(input), html, doExpand)
    }

    private fun prettyPrint(
        validation: Validation<Parse>, html: Boolean, doExpand: Boolean
    ): Pair<String, List<ParseError>> {
        val content =
            when (validation) {
                is ValidationFailure -> {
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
                    }
                }
                is ValidationSuccess ->
                    prettyPrint(node = validation.value.document, html = html, doExpand = doExpand)
            }

        return when (validation) {
            is ValidationFailure -> Pair(content, validation.errors)
            is ValidationSuccess -> Pair(content, emptyList())
        }
    }

    override fun prettyPrint(node: Phase2Node, html: Boolean, doExpand: Boolean): String {
        val writer =
            if (html) {
                HtmlCodeWriter(
                    defines = doExpand.thenUse { definesGroups.map { it.value } },
                    states = doExpand.thenUse { statesGroups.map { it.value } },
                    foundations = doExpand.thenUse { foundationGroups.map { it.value } },
                    mutuallyGroups = doExpand.thenUse { mutuallyGroups.map { it.value } })
            } else {
                MathLinguaCodeWriter(
                    defines = doExpand.thenUse { definesGroups.map { it.value } },
                    states = doExpand.thenUse { statesGroups.map { it.value } },
                    foundations = doExpand.thenUse { foundationGroups.map { it.value } },
                    mutuallyGroups = doExpand.thenUse { mutuallyGroups.map { it.value } })
            }
        val code = node.toCode(false, 0, writer = writer).getCode()
        return if (html) {
            getHtml(code)
        } else {
            code
        }
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
    foundations: List<FoundationGroup>,
    mutuallyGroups: List<MutuallyGroup>
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

    for (m in mutuallyGroups) {
        for (item in m.mutuallySection.items) {
            if (item is DefinesGroup) {
                allDefines.add(item)
            } else if (item is StatesGroup) {
                allStates.add(item)
            }
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

        when (def) {
            is DefinesGeneratedGroup -> {
                val from = def.generatedSection.inductivelyGroup.fromSection
                for (clause in from.clauses.clauses) {
                    if (clause is ConstantGroup) {
                        for (nameClause in clause.constantSection.clauses.clauses) {
                            val name = nameClause.toCode(false, 0).getCode()
                            // an example of name is `0` while `def.signature` could be `\natural`
                            // Make the code `\natural.0` to be the command recognized
                            // and be written as `0`
                            val tail = nameClause.toCode(false, 0).getCode()
                            val code = def.signature + "." + tail
                            val lexer = newTexTalkLexer(code)
                            val parser = newTexTalkParser()
                            val parse = parser.parse(lexer)
                            if (parse.root.children.size == 1 &&
                                parse.root.children[0] is Command) {
                                result[
                                    OperatorTexTalkNode(
                                        lhs = null, command = parse.root.children[0], rhs = null)] =
                                    code
                            }
                        }
                    } else if (clause is ConstructorGroup) {
                        val targets = clause.constructorSection.targets
                        for (target in targets) {
                            if (target is AbstractionNode) {
                                // an example of the target could be `succ(x)` and
                                // the Defines signature as `\natural`.
                                // Make the code `\natural.succ(x)` and the written as
                                // succ(x?)
                                val tail = target.abstraction.toCode()
                                val code = def.signature + "." + tail
                                val lexer = newTexTalkLexer(code)
                                val parser = newTexTalkParser()
                                val parse = parser.parse(lexer)
                                if (parse.root.children.size == 1 &&
                                    parse.root.children[0] is Command) {
                                    result[
                                        OperatorTexTalkNode(
                                            lhs = null,
                                            command = parse.root.children[0],
                                            rhs = null)] =
                                        tail.replace(",", "?,").replace(")", "?)")
                                    // i.e. make f(x, y, z) into f(x?, y?, z?)
                                }
                            }
                        }
                    }
                }
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

private fun getHtml(body: String) =
    """
<!DOCTYPE html>
<html>
    <head>
        <link rel="stylesheet"
              href="https://cdn.jsdelivr.net/npm/katex@0.11.1/dist/katex.min.css"
              integrity="sha384-zB1R0rpPzHqg7Kpt0Aljp8JPLqbXI3bhnPWROx27a9N0Ll6ZP/+DiW/UqRcLbRjq"
              crossorigin="anonymous">
        <script defer
                src="https://cdn.jsdelivr.net/npm/katex@0.11.1/dist/katex.min.js"
                integrity="sha384-y23I5Q6l+B6vatafAwxRu/0oK/79VlbSz7Q9aiSZUvyWYIYsd+qj+o24G5ZU2zJz"
                crossorigin="anonymous">
        </script>
        <script>
            function buildMathFragment(rawText) {
                var text = rawText;
                if (text[0] === '"') {
                    text = text.substring(1);
                }
                if (text[text.length - 1] === '"') {
                    text = text.substring(0, text.length - 1);
                }
                text = text.replace(/([a-zA-Z0-9])\?\??/g, '${'$'}1');
                const fragment = document.createDocumentFragment();
                var buffer = '';
                var i = 0;
                while (i < text.length) {
                    if (text[i] === '\\' && text[i+1] === '[') {
                        i += 2; // skip over \ and [
                        fragment.appendChild(document.createTextNode(buffer));
                        buffer = '';

                        const span = document.createElement('span');
                        var math = '';
                        while (i < text.length &&
                            !(text[i] === '\\' && text[i+1] === ']')) {
                            math += text[i++];
                        }
                        if (text[i] === '\\') {
                            i++; // move past the \
                        }
                        if (text[i] === ']') {
                            i++; // move past the ]
                        }
                        try {
                            katex.render(math, span, {
                                throwOnError: true,
                                displayMode: true
                            });
                        } catch {
                            span.appendChild(document.createTextNode(math));
                        }
                        fragment.appendChild(span);
                    } else if (text[i] === '\\' && text[i+1] === '(') {
                        i += 2; // skip over \ and ()
                        fragment.appendChild(document.createTextNode(buffer));
                        buffer = '';

                        const span = document.createElement('span');
                        var math = '';
                        while (i < text.length &&
                            !(text[i] === '\\' && text[i+1] === ')')) {
                            math += text[i++];
                        }
                        if (text[i] === '\\') {
                            i++; // move past the \
                        }
                        if (text[i] === ')') {
                            i++; // move past the )
                        }
                        try {
                            katex.render(math, span, {
                                throwOnError: true,
                                displayMode: true
                            });
                        } catch {
                            span.appendChild(document.createTextNode(math));
                        }
                        fragment.appendChild(span);
                    } else if (text[i] === '${'$'}' && text[i+1] === '${'$'}') {
                        i += 2; // skip over ${'$'} and ${'$'}
                        fragment.appendChild(document.createTextNode(buffer));
                        buffer = '';

                        const span = document.createElement('span');
                        var math = '';
                        while (i < text.length &&
                            !(text[i] === '${'$'}' && text[i+1] === '${'$'}')) {
                            math += text[i++];
                        }
                        if (text[i] === '${'$'}') {
                            i++; // move past the ${'$'}
                        }
                        if (text[i] === '${'$'}') {
                            i++; // move past the ${'$'}
                        }
                        try {
                            katex.render(math, span, {
                                throwOnError: true,
                                displayMode: true
                            });
                        } catch {
                            span.appendChild(document.createTextNode(math));
                        }
                        fragment.appendChild(span);
                    } else if (text[i] === '${'$'}') {
                        i++; // skip over the ${'$'}
                        fragment.appendChild(document.createTextNode(buffer));
                        buffer = '';

                        const span = document.createElement('span');
                        var math = '';
                        while (i < text.length &&
                             text[i] !== '${'$'}') {
                            math += text[i++];
                        }
                        if (text[i] === '${'$'}') {
                            i++; // move past the ${'$'}
                        }
                        try {
                            katex.render(math, span, {
                                throwOnError: true,
                                displayMode: true
                            });
                        } catch {
                            span.appendChild(document.createTextNode(math));
                        }
                        fragment.appendChild(span);
                    } else {
                        buffer += text[i++];
                    }
                }

                if (buffer.length > 0) {
                    fragment.appendChild(document.createTextNode(buffer));
                }

                return fragment;
            }

            function render(node) {
                if (node.className && node.className.indexOf('no-render') >= 0) {
                    return;
                }

                let isInWritten = false;
                const parent = node.parentNode;
                if (node.className === 'mathlingua') {
                    for (let i=0; i<node.childNodes.length; i++) {
                        const n = node.childNodes[i];
                        if (n && n.className === 'mathlingua-header' &&
                            n.textContent === 'written:') {
                            isInWritten = true;
                            break;
                        }
                    }
                }

                for (let i = 0; i < node.childNodes.length; i++) {
                    const child = node.childNodes[i];

                    // node is an element node => nodeType === 1
                    // node is an attribute node => nodeType === 2
                    // node is a text node => nodeType === 3
                    // node is a comment node => nodeType === 8
                    if (child.nodeType === 3) {
                        let text = child.textContent;
                        if (text.trim()) {
                            if (isInWritten) {
                                // if the text is in a written: section
                                // turn "some text" to \[some text\]
                                // so the text is in math mode
                                if (text[0] === '"') {
                                    text = text.substring(1);
                                }
                                if (text[text.length - 1] === '"') {
                                    text = text.substring(0, text.length - 1);
                                }
                                text = '\\[' + text + '\\]';
                            }
                            const fragment = buildMathFragment(text);
                            i += fragment.childNodes.length - 1;
                            node.replaceChild(fragment, child);
                        }
                    } else if (child.nodeType === 1) {
                        render(child);
                    }
                }
            }
        </script>
        <style>
            .content {
                margin-top: 1em;
                margin-bottom: 1em;
                font-size: 1em;
            }

            .mathlingua {
                font-family: monospace;
            }

            .mathlingua-header {
                font-weight: bold;
                color: #0055bb;
            }

            .mathlingua-whitespace {
                padding: 0;
                margin: 0;
                margin-left: 1ex;
            }

            .mathlingua-id {
                font-weight: bold;
                color: #5500aa;
            }

            .mathlingua-text {
                color: #007700;
            }

            .mathlingua-text-no-render {
                color: #007700;
            }

            .mathlingua-statement-no-render {
                color: #007377;
            }

            .katex {
                font-size: 0.75em;
            }

            .katex-display {
                display: contents;
            }

            .katex-display > .katex {
                display: contents;
            }

            .katex-display > .katex > .katex-html {
                display: contents;
            }
        </style>
    </head>
    <body onload="render(document.body)">
        <div class="content">
            $body
        </div>
    </body>
</html>
"""

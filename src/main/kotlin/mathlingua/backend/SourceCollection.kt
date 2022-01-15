/*
 * Copyright 2019 The MathLingua Authors
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

import mathlingua.backend.transform.GroupScope
import mathlingua.backend.transform.Signature
import mathlingua.backend.transform.checkVarsPhase2Node
import mathlingua.backend.transform.expandAsWritten
import mathlingua.backend.transform.getVarsPhase2Node
import mathlingua.backend.transform.getVarsTexTalkNode
import mathlingua.backend.transform.locateAllSignatures
import mathlingua.backend.transform.normalize
import mathlingua.backend.transform.signature
import mathlingua.cli.EntityResult
import mathlingua.cli.ErrorResult
import mathlingua.cli.FileResult
import mathlingua.cli.VirtualFile
import mathlingua.cli.VirtualFileSystem
import mathlingua.cli.fixClassNameBug
import mathlingua.cli.getAllWords
import mathlingua.cli.newAutoComplete
import mathlingua.cli.newSearchIndex
import mathlingua.frontend.FrontEnd
import mathlingua.frontend.Parse
import mathlingua.frontend.chalktalk.phase1.ast.Abstraction
import mathlingua.frontend.chalktalk.phase1.ast.ChalkTalkTokenType
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.Phase1Token
import mathlingua.frontend.chalktalk.phase1.ast.Tuple
import mathlingua.frontend.chalktalk.phase1.newChalkTalkLexer
import mathlingua.frontend.chalktalk.phase1.newChalkTalkParser
import mathlingua.frontend.chalktalk.phase2.ast.Document
import mathlingua.frontend.chalktalk.phase2.ast.clause.AssignmentNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.ClauseListNode
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.clause.TupleNode
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.clause.If.ThenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.HasSignature
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.HasUsingSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.CalledSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.MeansSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.viewingas.ViewingAsSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.axiom.AxiomGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.conjecture.ConjectureGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resultlike.theorem.TheoremSection
import mathlingua.frontend.chalktalk.phase2.getCalledNames
import mathlingua.frontend.chalktalk.phase2.getInnerDefinedSignatures
import mathlingua.frontend.chalktalk.phase2.getPatternsToWrittenAs
import mathlingua.frontend.chalktalk.phase2.newHtmlCodeWriter
import mathlingua.frontend.chalktalk.phase2.newMathLinguaCodeWriter
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.LocationTracker
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.Validation
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.support.newLocationTracker
import mathlingua.frontend.support.validationSuccess
import mathlingua.frontend.textalk.ColonEqualsTexTalkNode
import mathlingua.frontend.textalk.Command
import mathlingua.frontend.textalk.ExpressionTexTalkNode
import mathlingua.frontend.textalk.InTexTalkNode
import mathlingua.frontend.textalk.IsTexTalkNode
import mathlingua.frontend.textalk.OperatorTexTalkNode
import mathlingua.frontend.textalk.TexTalkNode
import mathlingua.frontend.textalk.TexTalkNodeType
import mathlingua.frontend.textalk.TexTalkTokenType
import mathlingua.frontend.textalk.TextTexTalkNode
import mathlingua.frontend.textalk.isOpChar
import mathlingua.frontend.textalk.newTexTalkLexer
import mathlingua.frontend.textalk.newTexTalkParser
import mathlingua.md5Hash

internal data class SourceFile(
    val file: VirtualFile, val content: String, val validation: Validation<Parse>)

internal fun SourceFile.toFileResult(
    nextRelativePath: String?, previousRelativePath: String?, sourceCollection: SourceCollection
): FileResult {
    val relativePath = this.file.relativePath()
    return FileResult(
        relativePath = relativePath,
        nextRelativePath = nextRelativePath,
        previousRelativePath = previousRelativePath,
        content = this.content,
        entities =
            when (val validation = this.validation
            ) {
                is ValidationSuccess -> {
                    val doc = validation.value.document
                    doc.groups.map { it.toEntityResult(relativePath, sourceCollection) }
                }
                else -> {
                    emptyList()
                }
            },
        errors =
            when (val validation = this.validation
            ) {
                is ValidationFailure -> {
                    validation.errors.map {
                        ErrorResult(
                            relativePath = relativePath,
                            message = it.message,
                            row = it.row,
                            column = it.column)
                    }
                }
                else -> {
                    emptyList()
                }
            })
}

internal fun isMathLinguaFile(file: VirtualFile) =
    !file.isDirectory() && file.absolutePath().endsWith(".math")

internal fun buildSourceFile(file: VirtualFile): SourceFile {
    val content = file.readText()
    return SourceFile(
        file = file, content = content, validation = FrontEnd.parseWithLocations(content))
}

internal data class ValueSourceTracker<T>(
    val value: T, val source: SourceFile, val tracker: MutableLocationTracker?)

internal data class Page(val sourceFile: SourceFile, val fileResult: FileResult)

internal data class WrittenAsForm(val target: String?, val form: String)

internal interface SourceCollection {
    fun size(): Int
    fun getDefinedSignatures(): Set<ValueSourceTracker<Signature>>
    fun getDuplicateDefinedSignatures(): List<ValueSourceTracker<Signature>>
    fun getUndefinedSignatures(): Set<ValueSourceTracker<Signature>>
    fun findInvalidTypes(): List<ValueSourceTracker<ParseError>>
    fun getParseErrors(): List<ValueSourceTracker<ParseError>>
    fun getDuplicateContent(): List<ValueSourceTracker<TopLevelGroup>>
    fun getSymbolErrors(): List<ValueSourceTracker<ParseError>>
    fun getIsRhsErrors(): List<ValueSourceTracker<ParseError>>
    fun getColonEqualsRhsErrors(): List<ValueSourceTracker<ParseError>>
    fun getInputOutputSymbolErrors(): List<ValueSourceTracker<ParseError>>
    fun getNonExpressesUsedInNonIsNonInStatementsErrors(): List<ValueSourceTracker<ParseError>>
    fun prettyPrint(
        file: VirtualFile, html: Boolean, literal: Boolean, doExpand: Boolean
    ): Pair<List<Pair<String, Phase2Node?>>, List<ParseError>>
    fun prettyPrint(node: Phase2Node, html: Boolean, literal: Boolean, doExpand: Boolean): String
    fun getAllPaths(): List<String>
    fun getFirstPath(): String
    fun getPage(path: String): Page?
    fun getWithSignature(signature: String): EntityResult?
    fun addSource(sf: SourceFile)
    fun removeSource(path: String)
    fun findWordSuffixesFor(word: String): List<String>
    fun findSignaturesSuffixesFor(prefix: String): List<String>
    fun search(query: String): List<SourceFile>
}

internal fun newSourceCollection(
    fs: VirtualFileSystem, filesOrDirs: List<VirtualFile>
): SourceCollection {
    val sources = findMathLinguaFiles(filesOrDirs).map { buildSourceFile(it) }
    return SourceCollectionImpl(fs, sources)
}

internal fun newSourceCollectionFromCwd(fs: VirtualFileSystem) =
    newSourceCollection(fs, listOf(fs.cwd()))

internal fun findMathLinguaFiles(files: List<VirtualFile>): List<VirtualFile> {
    val result = mutableListOf<VirtualFile>()
    for (file in files) {
        findMathLinguaFilesImpl(file, result)
    }
    return result
}

// -----------------------------------------------------------------------------

private class StringPartsComparator : Comparator<List<String>> {
    override fun compare(list1: List<String>?, list2: List<String>?): Int {
        if (list1 == null && list2 != null) {
            return -1
        }

        if (list1 != null && list2 == null) {
            return 1
        }

        if (list1 == null && list2 == null) {
            return 0
        }

        return compare(list1!!, 0, list2!!, 0)
    }

    private fun getNumberPrefix(part: String): Double? {
        val index = part.indexOf('_')
        if (index < 0) {
            return null
        }
        return part.substring(0, index).toDoubleOrNull()
    }

    private fun compare(parts1: List<String>, index1: Int, parts2: List<String>, index2: Int): Int {
        if (index1 >= parts1.size && index2 >= parts2.size) {
            return 0
        }

        if (index1 >= parts1.size && index2 < parts2.size) {
            return -1
        }

        if (index1 < parts1.size && index2 >= parts2.size) {
            return 1
        }

        val p1 = parts1[index1]
        val p2 = parts2[index2]

        val num1 = getNumberPrefix(p1)
        val num2 = getNumberPrefix(p2)

        if (num1 != null && num2 == null) {
            return -1
        }

        if (num1 == null && num2 != null) {
            return 1
        }

        val comp =
            if (num1 == null && num2 == null) {
                p1.compareTo(p2)
            } else {
                num1!!.compareTo(num2!!)
            }

        return if (comp == 0) {
            compare(parts1, index1 + 1, parts2, index2 + 1)
        } else {
            comp
        }
    }
}

private val STRING_PARTS_COMPARATOR = StringPartsComparator()

private class SourcePathComparator : Comparator<VirtualFile> {
    override fun compare(file1: VirtualFile?, file2: VirtualFile?): Int {
        if (file1 == null && file2 != null) {
            return -1
        }

        if (file1 != null && file2 == null) {
            return 1
        }

        if (file1 == null && file2 == null) {
            return 0
        }

        return STRING_PARTS_COMPARATOR.compare(
            file1!!.absolutePathParts(), file2!!.absolutePathParts())
    }
}

private val SOURCE_PATH_COMPARATOR = SourcePathComparator()

private fun findMathLinguaFilesImpl(file: VirtualFile, result: MutableList<VirtualFile>) {
    if (isMathLinguaFile(file)) {
        result.add(file)
    }
    for (child in file.listFiles().sortedWith(SOURCE_PATH_COMPARATOR)) {
        findMathLinguaFilesImpl(child, result)
    }
}

private fun TopLevelGroup.toEntityResult(
    relativePath: String, sourceCollection: SourceCollection
): EntityResult {
    val renderedHtml =
        sourceCollection.prettyPrint(node = this, html = true, literal = false, doExpand = true)
    val rawHtml =
        sourceCollection.prettyPrint(node = this, html = true, literal = true, doExpand = false)
    return EntityResult(
        id = md5Hash(this.toCode(false, 0).getCode()),
        type = this.javaClass.simpleName,
        signature =
            if (this is HasSignature) {
                this.signature?.form
            } else {
                null
            },
        rawHtml = fixClassNameBug(rawHtml),
        renderedHtml = fixClassNameBug(renderedHtml),
        words = getAllWords(this).toList(),
        relativePath = relativePath,
        called = this.getCalledNames())
}

private data class Normalized<T>(val original: T, val normalized: T)

private class SourceCollectionImpl(val fs: VirtualFileSystem, val sources: List<SourceFile>) :
    SourceCollection {
    private val sourceFiles = mutableMapOf<String, SourceFile>()
    private val sourceFileToFileResult = mutableMapOf<SourceFile, FileResult>()

    private val wordAutoComplete = newAutoComplete(preserveCase = false)
    private val signatureAutoComplete = newAutoComplete(preserveCase = true)

    private val searchIndex = newSearchIndex(fs)

    private val signatureToTopLevelGroup = mutableMapOf<String, TopLevelGroup>()
    private val signatureToRelativePath = mutableMapOf<String, String>()

    private val allGroups = mutableListOf<ValueSourceTracker<Normalized<TopLevelGroup>>>()
    private val definesGroups = mutableListOf<ValueSourceTracker<Normalized<DefinesGroup>>>()
    private val statesGroups = mutableListOf<ValueSourceTracker<Normalized<StatesGroup>>>()
    private val axiomGroups = mutableListOf<ValueSourceTracker<Normalized<AxiomGroup>>>()
    private val theoremGroups = mutableListOf<ValueSourceTracker<Normalized<TheoremGroup>>>()
    private val conjectureGroups = mutableListOf<ValueSourceTracker<Normalized<ConjectureGroup>>>()

    init {
        // add all the sources
        for (sf in sources) {
            addSource(sf)
        }

        // pre-calculate the rendering of each page to speed up access later on
        for (path in getAllPaths()) {
            getPage(path)
        }
    }

    override fun search(query: String): List<SourceFile> {
        return searchIndex.search(query).mapNotNull {
            val path = it.joinToString("/")
            sourceFiles[path]
        }
    }

    override fun findWordSuffixesFor(word: String): List<String> {
        return wordAutoComplete.findSuffixes(word)
    }

    override fun findSignaturesSuffixesFor(prefix: String): List<String> {
        return signatureAutoComplete.findSuffixes(prefix).map { it }
    }

    override fun getAllPaths(): List<String> {
        return sourceFiles
            .keys
            .toList()
            .map { it.split("/") }
            .sortedWith(STRING_PARTS_COMPARATOR)
            .map { it.joinToString("/") }
    }

    override fun getFirstPath() = getAllPaths().first()

    override fun getPage(path: String): Page? {
        val sourceFile = sourceFiles[path] ?: return null
        val fileResult = sourceFileToFileResult[sourceFile]
        if (fileResult != null) {
            return Page(sourceFile = sourceFile, fileResult = fileResult)
        }

        var prev: SourceFile? = null
        var next: SourceFile? = null
        for (i in sources.indices) {
            if (sources[i].file.relativePath() == path) {
                prev = sources.getOrNull(i - 1)
                next = sources.getOrNull(i + 1)
                break
            }
        }

        val evalFileResult =
            sourceFile.toFileResult(
                previousRelativePath = prev?.file?.relativePath(),
                nextRelativePath = next?.file?.relativePath(),
                sourceCollection = this)
        sourceFileToFileResult[sourceFile] = evalFileResult
        return Page(sourceFile = sourceFile, fileResult = evalFileResult)
    }

    override fun getWithSignature(signature: String): EntityResult? {
        val relativePath = signatureToRelativePath[signature] ?: return null
        return signatureToTopLevelGroup[signature]?.toEntityResult(relativePath, this)
    }

    override fun removeSource(path: String) {
        sourceFileToFileResult.clear()
        val relPath = path.split("/")
        searchIndex.remove(relPath)
        val sf = sourceFiles[path] ?: return
        sourceFiles.remove(path)
        when (val validation = sf.validation
        ) {
            is ValidationSuccess -> {
                val doc = validation.value.document
                val docDefines = doc.defines().toSet()
                val docStates = doc.states().toSet()
                val docAxioms = doc.axioms().toSet()
                val docTheorems = doc.theorems().toSet()
                val docConjectures = doc.conjectures().toSet()
                val docAll = doc.groups.toSet()

                for (grp in docAll) {
                    if (grp is HasSignature && grp.signature != null) {
                        val key = grp.signature!!.form
                        signatureToTopLevelGroup.remove(key)
                        signatureToRelativePath.remove(key)
                        if (grp.id != null) {
                            signatureAutoComplete.remove(grp.id!!.text)
                        }
                    }
                }

                for (grp in validation.value.document.groups) {
                    for (word in getAllWords(grp)) {
                        wordAutoComplete.remove(word)
                    }
                }

                definesGroups.removeAll { docDefines.contains(it.value.original) }

                statesGroups.removeAll { docStates.contains(it.value.original) }

                axiomGroups.removeAll { docAxioms.contains(it.value.original) }

                theoremGroups.removeAll { docTheorems.contains(it.value.original) }

                conjectureGroups.removeAll { docConjectures.contains(it.value.original) }

                allGroups.removeAll { docAll.contains(it.value.original) }
            }
        }
    }

    override fun addSource(sf: SourceFile) {
        sourceFileToFileResult.clear()
        val relativePath = sf.file.relativePath()
        sourceFiles[relativePath] = sf
        searchIndex.add(sf)
        val validation = sf.validation
        if (validation is ValidationSuccess) {
            for (grp in validation.value.document.groups) {
                for (word in getAllWords(grp)) {
                    wordAutoComplete.add(word)
                }
            }

            for (grp in validation.value.document.groups) {
                if (grp is HasSignature && grp.signature != null) {
                    val key = grp.signature!!.form
                    signatureToTopLevelGroup[key] = grp
                    signatureToRelativePath[key] = relativePath
                    if (grp.id != null) {
                        signatureAutoComplete.add(grp.id!!.text)
                    }
                }
            }

            definesGroups.addAll(
                validation.value.document.defines().map {
                    ValueSourceTracker(
                        source = sf,
                        tracker = validation.value.tracker,
                        value =
                            Normalized(
                                original = it,
                                normalized =
                                    normalize(it, validation.value.tracker) as DefinesGroup))
                })

            statesGroups.addAll(
                validation.value.document.states().map {
                    ValueSourceTracker(
                        source = sf,
                        tracker = validation.value.tracker,
                        value =
                            Normalized(
                                original = it,
                                normalized =
                                    normalize(it, validation.value.tracker) as StatesGroup))
                })

            axiomGroups.addAll(
                validation.value.document.axioms().map {
                    ValueSourceTracker(
                        source = sf,
                        tracker = validation.value.tracker,
                        value =
                            Normalized(
                                original = it,
                                normalized = normalize(it, validation.value.tracker) as AxiomGroup))
                })

            theoremGroups.addAll(
                validation.value.document.theorems().map {
                    ValueSourceTracker(
                        source = sf,
                        tracker = validation.value.tracker,
                        value =
                            Normalized(
                                original = it,
                                normalized =
                                    normalize(it, validation.value.tracker) as TheoremGroup))
                })

            conjectureGroups.addAll(
                validation.value.document.conjectures().map {
                    ValueSourceTracker(
                        source = sf,
                        tracker = validation.value.tracker,
                        value =
                            Normalized(
                                original = it,
                                normalized =
                                    normalize(it, validation.value.tracker) as ConjectureGroup))
                })

            allGroups.addAll(
                validation.value.document.groups.map {
                    ValueSourceTracker(
                        source = sf,
                        tracker = validation.value.tracker,
                        value =
                            Normalized(
                                original = it,
                                normalized =
                                    normalize(it, validation.value.tracker) as TopLevelGroup))
                })
        }
    }

    override fun size() = sourceFiles.size

    override fun getIsRhsErrors(): List<ValueSourceTracker<ParseError>> {
        val result = mutableListOf<ValueSourceTracker<ParseError>>()
        val sigsWithExpresses = getSignaturesWithExpressesSection().map { it.value.form }.toSet()
        val theoremSigs = theoremGroups.mapNotNull { it.value.original.signature?.form }.toSet()
        val axiomSigs = axiomGroups.mapNotNull { it.value.original.signature?.form }.toSet()
        val conjectureSigs = axiomGroups.mapNotNull { it.value.original.signature?.form }.toSet()
        val statesSigs = statesGroups.mapNotNull { it.value.original.signature?.form }.toSet()
        for (svt in allGroups) {
            val rhsIsSigs =
                findIsRhsSignatures(svt.value.normalized, svt.tracker ?: newLocationTracker())
            for (sig in rhsIsSigs) {
                if (sigsWithExpresses.contains(sig.form)) {
                    result.add(
                        ValueSourceTracker(
                            value =
                                ParseError(
                                    message =
                                        "The right-hand-side of an `is` cannot reference a `Defines:` with an `expresses:` section but found '${sig.form}'",
                                    row = sig.location.row,
                                    column = sig.location.column),
                            source = svt.source,
                            tracker = svt.tracker))
                }

                if (theoremSigs.contains(sig.form)) {
                    result.add(
                        ValueSourceTracker(
                            value =
                                ParseError(
                                    message =
                                        "The right-hand-side of an `is` cannot reference a `Theorem:` but found '${sig.form}'",
                                    row = sig.location.row,
                                    column = sig.location.column),
                            source = svt.source,
                            tracker = svt.tracker))
                }

                if (axiomSigs.contains(sig.form)) {
                    result.add(
                        ValueSourceTracker(
                            value =
                                ParseError(
                                    message =
                                        "The right-hand-side of an `is` cannot reference a `Axiom:` but found '${sig.form}'",
                                    row = sig.location.row,
                                    column = sig.location.column),
                            source = svt.source,
                            tracker = svt.tracker))
                }

                if (conjectureSigs.contains(sig.form)) {
                    result.add(
                        ValueSourceTracker(
                            value =
                                ParseError(
                                    message =
                                        "The right-hand-side of an `is` cannot reference a `Conjecture:` but found '${sig.form}'",
                                    row = sig.location.row,
                                    column = sig.location.column),
                            source = svt.source,
                            tracker = svt.tracker))
                }

                if (statesSigs.contains(sig.form)) {
                    result.add(
                        ValueSourceTracker(
                            value =
                                ParseError(
                                    message =
                                        "The right-hand-side of an `is` cannot reference a `States:` but found '${sig.form}'",
                                    row = sig.location.row,
                                    column = sig.location.column),
                            source = svt.source,
                            tracker = svt.tracker))
                }
            }

            val sigsWithoutExpresses =
                getSignaturesWithoutExpressesSection().map { it.value.form }.toSet()
            val rhsInSigs =
                findInRhsSignatures(svt.value.normalized, svt.tracker ?: newLocationTracker())
            for (sig in rhsInSigs) {
                if (sigsWithoutExpresses.contains(sig.form)) {
                    result.add(
                        ValueSourceTracker(
                            value =
                                ParseError(
                                    message =
                                        "The right-hand-side of an `in` cannot reference a `Defines:` without an `expresses:` section but found '${sig.form}'",
                                    row = sig.location.row,
                                    column = sig.location.column),
                            source = svt.source,
                            tracker = svt.tracker))
                }
            }
        }
        return result
    }

    override fun getColonEqualsRhsErrors(): List<ValueSourceTracker<ParseError>> {
        val result = mutableListOf<ValueSourceTracker<ParseError>>()
        val sigsWithoutExpresses =
            getSignaturesWithoutExpressesSection().map { it.value.form }.toSet()
        for (svt in allGroups) {
            val rhsSigs =
                findColonEqualsRhsSignatures(
                    svt.value.normalized, svt.tracker ?: newLocationTracker())
            for (sig in rhsSigs) {
                if (sigsWithoutExpresses.contains(sig.form)) {
                    result.add(
                        ValueSourceTracker(
                            value =
                                ParseError(
                                    message =
                                        "The right-hand-side of an `:=` cannot reference a `Defines:` without an `expresses:` section but found '${sig.form}'",
                                    row = sig.location.row,
                                    column = sig.location.column),
                            source = svt.source,
                            tracker = svt.tracker))
                }
            }
        }
        return result
    }

    private fun getSignaturesWithStates(): List<ValueSourceTracker<Signature>> {
        val result = mutableListOf<ValueSourceTracker<Signature>>()
        for (svt in statesGroups) {
            val def = svt.value.original
            if (def.signature != null) {
                result.add(
                    ValueSourceTracker(
                        value = def.signature, source = svt.source, tracker = svt.tracker))
            }
        }
        return result
    }

    private fun getSignaturesWithoutExpressesSection(): List<ValueSourceTracker<Signature>> {
        val result = mutableListOf<ValueSourceTracker<Signature>>()
        for (svt in allGroups) {
            val def = svt.value.original
            if (def is HasSignature) {
                val sig = def.signature ?: continue
                if (def !is DefinesGroup || def.expressesSection == null) {
                    result.add(
                        ValueSourceTracker(value = sig, source = svt.source, tracker = svt.tracker))
                }
            }
        }
        return result
    }

    private fun getSignaturesWithExpressesSection(): List<ValueSourceTracker<Signature>> {
        val result = mutableListOf<ValueSourceTracker<Signature>>()
        for (svt in definesGroups) {
            val def = svt.value.original
            if (def.expressesSection != null && def.signature != null) {
                result.add(
                    ValueSourceTracker(
                        value = def.signature, source = svt.source, tracker = svt.tracker))
            }
        }
        return result
    }

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
                getInnerDefinedSignatures(vst.value.normalized, vst.tracker).map { it.form }.toSet()
            val usedSigs =
                locateAllSignatures(
                    vst.value.normalized,
                    ignoreLhsEqual = true,
                    vst.tracker ?: newLocationTracker())
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
        val analyzer = newSymbolAnalyzer(defs)
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

            when (val validation = sf.value.validation
            ) {
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
                                    target = null,
                                    node = node,
                                    operatorPatternToExpansion =
                                        getPatternsToWrittenAs(
                                            defines =
                                                definesGroups
                                                    .map { it.value.normalized }
                                                    .plus(aliasDefines),
                                            states = statesGroups.map { it.value.normalized },
                                            axioms = axiomGroups.map { it.value.normalized }))
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
                else -> {
                    // if parsing fails, then no further checking is needed
                }
            }
        }
        return result
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

                    val allDefines = mutableListOf<DefinesGroup>()
                    allDefines.addAll(doc.defines())

                    val allStates = mutableListOf<StatesGroup>()
                    allStates.addAll(doc.states())

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
            val content = group.value.normalized.toCode(false, 0).getCode().trim()
            if (allContent.contains(content)) {
                result.add(
                    ValueSourceTracker(
                        source = group.source,
                        tracker = group.tracker,
                        value = group.value.normalized))
            }
            allContent.add(content)
        }
        return result
    }

    override fun prettyPrint(
        file: VirtualFile, html: Boolean, literal: Boolean, doExpand: Boolean
    ): Pair<List<Pair<String, Phase2Node?>>, List<ParseError>> {
        val sourceFile = sourceFiles[file.absolutePathParts().joinToString("/")]
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
            newHtmlCodeWriter(
                defines =
                    doExpand.thenUse {
                        definesGroups.map { it.value.normalized }.plus(aliasDefines)
                    },
                states = doExpand.thenUse { statesGroups.map { it.value.normalized } },
                axioms = doExpand.thenUse { axiomGroups.map { it.value.normalized } },
                literal = literal)
        } else {
            newMathLinguaCodeWriter(
                defines =
                    doExpand.thenUse {
                        definesGroups.map { it.value.normalized }.plus(aliasDefines)
                    },
                states = doExpand.thenUse { statesGroups.map { it.value.normalized } },
                axioms = doExpand.thenUse { axiomGroups.map { it.value.normalized } })
        }

    private fun getExpandedText(
        exp: ExpressionTexTalkNode, aliasDefines: List<DefinesGroup>
    ): String? {
        val writer =
            getWriter(html = false, literal = false, doExpand = true, aliasDefines = aliasDefines)
        val tmpTheorem =
            TheoremGroup(
                signature = null,
                id = null,
                theoremSection = TheoremSection(names = emptyList()),
                givenSection = null,
                whenSection = null,
                thenSection =
                    ThenSection(
                        clauses =
                            ClauseListNode(
                                clauses =
                                    listOf(
                                        Statement(
                                            text = exp.toCode(),
                                            texTalkRoot = validationSuccess(exp))))),
                iffSection = null,
                proofSection = null,
                usingSection = null,
                metaDataSection = null)
        val expanded = tmpTheorem.toCode(false, 0, writer).getCode()
        return when (val validation = FrontEnd.parse(expanded)
        ) {
            is ValidationSuccess -> {
                val doc = validation.value
                val stmt = doc.theorems()[0].thenSection.clauses.clauses[0] as Statement
                stmt.text
            }
            else -> {
                null
            }
        }
    }

    private fun attemptAddAliasDefinesForColonEqualsTuple(
        colonEquals: ColonEqualsTexTalkNode, aliasDefines: MutableList<DefinesGroup>
    ) {
        val lhsItems = colonEquals.lhs.items
        val rhsItems = colonEquals.rhs.items
        if (lhsItems.size != 1 || rhsItems.size != 1) {
            println(
                "The left-hand-side and right-hand-side of a := must have exactly one expression")
            return
        }

        val lhsItem = lhsItems[0]
        if (lhsItem.children.size != 1 ||
            lhsItem.children[0] !is mathlingua.frontend.textalk.TupleNode) {
            return
        }

        val leftTuple = lhsItem.children[0] as mathlingua.frontend.textalk.TupleNode
        val lhsNames =
            leftTuple
                .params
                .items
                .filter { it.children.size == 1 && it.children[0] is TextTexTalkNode }
                .map { it.children[0] as TextTexTalkNode }
                .map { it.text }
        if (lhsNames.isEmpty()) {
            return
        }

        val rhsItem = rhsItems[0]
        if (rhsItem.children.size != 1 || rhsItem.children[0] !is Command) {
            return
        }
        val rhsCommand = rhsItem.children[0] as Command
        val rhsSignature = rhsCommand.signature()

        var rhsMatchedDefines: DefinesGroup? = null
        for (defVal in definesGroups) {
            val def = defVal.value.original
            if (def.signature?.form == rhsSignature) {
                rhsMatchedDefines = def
                break
            }
        }

        if (rhsMatchedDefines == null) {
            return
        }

        var rhsNames: List<String>? = null
        val rhsTargets = rhsMatchedDefines.definesSection.targets
        if (rhsTargets.size == 1) {
            val singleTarget = rhsTargets[0]
            var tupleTarget: Tuple? = null
            if (singleTarget is AssignmentNode && singleTarget.assignment.rhs is Tuple) {
                tupleTarget = singleTarget.assignment.rhs
            } else if (singleTarget is TupleNode) {
                tupleTarget = singleTarget.tuple
            }

            if (tupleTarget != null) {
                rhsNames =
                    tupleTarget.items
                        .filter {
                            it is Abstraction &&
                                !it.isEnclosed &&
                                !it.isVarArgs &&
                                it.parts.size == 1 &&
                                it.parts[0].params == null &&
                                it.parts[0].tail == null &&
                                it.parts[0].subParams == null
                        }
                        .map { (it as Abstraction).parts[0].name.text }
            }
        }

        if (rhsNames == null) {
            return
        }

        if (lhsNames.size != rhsNames.size) {
            return
        }

        val toFromNameMap = mutableMapOf<String, String>()
        for (i in lhsNames.indices) {
            toFromNameMap[rhsNames[i]] = lhsNames[i]
        }

        val fromNameToColonEqualsMap = mutableMapOf<String, ColonEqualsTexTalkNode>()

        fun maybeIdentifyFromNameToColonEquals(clause: Clause) {
            if (clause is Statement &&
                clause.texTalkRoot is ValidationSuccess &&
                clause.texTalkRoot.value.children.size == 1 &&
                clause.texTalkRoot.value.children[0] is ColonEqualsTexTalkNode) {
                val colonEqualsNode = clause.texTalkRoot.value.children[0] as ColonEqualsTexTalkNode
                if (colonEqualsNode.lhs.items.size == 1 &&
                    colonEqualsNode.lhs.items[0].children.size == 1 &&
                    colonEqualsNode.lhs.items[0].children[0] is OperatorTexTalkNode) {
                    val op = colonEqualsNode.lhs.items[0].children[0] as OperatorTexTalkNode
                    if (op.command is TextTexTalkNode) {
                        val opName = op.command.text
                        if (toFromNameMap.containsKey(opName)) {
                            val fromName = toFromNameMap[opName]!!
                            fromNameToColonEqualsMap[fromName] = colonEqualsNode
                        }
                    }
                }
            }
        }

        for (clause in rhsMatchedDefines.meansSection?.clauses?.clauses ?: emptyList()) {
            maybeIdentifyFromNameToColonEquals(clause)
        }

        for (clause in rhsMatchedDefines.expressesSection?.clauses?.clauses ?: emptyList()) {
            maybeIdentifyFromNameToColonEquals(clause)
        }

        for ((fromName, colonEqualsNode) in fromNameToColonEqualsMap.entries) {
            val rhs = colonEqualsNode.rhs.items[0]
            val rhsSig =
                if (rhs.children.size == 1 && rhs.children[0] is Command) {
                    (rhs.children[0] as Command).signature()
                } else if (rhs.children.size == 1 &&
                    rhs.children[0] is OperatorTexTalkNode &&
                    (rhs.children[0] as OperatorTexTalkNode).command is Command) {
                    ((rhs.children[0] as OperatorTexTalkNode).command as Command).signature()
                } else if (rhs.children.size == 1 &&
                    rhs.children[0] is OperatorTexTalkNode &&
                    (rhs.children[0] as OperatorTexTalkNode).command is TextTexTalkNode) {
                    ((rhs.children[0] as OperatorTexTalkNode).command as TextTexTalkNode).text
                } else {
                    null
                }

            if (rhsSig == null) {
                continue
            }

            var rhsDef: DefinesGroup? = null
            for (grpVal in definesGroups) {
                if (grpVal.value.original.signature?.form == rhsSig) {
                    rhsDef = grpVal.value.original
                    break
                }
            }

            if (rhsDef == null) {
                continue
            }

            val lhs =
                colonEqualsNode.lhs.items[0].transform {
                    if (it is TextTexTalkNode) {
                        val thisFromName = toFromNameMap[it.text]
                        if (fromName == thisFromName) {
                            TextTexTalkNode(
                                type = TexTalkNodeType.Identifier,
                                tokenType = TexTalkTokenType.Identifier,
                                text = fromName,
                                isVarArg = false)
                        } else {
                            it
                        }
                    } else {
                        it
                    }
                } as ExpressionTexTalkNode
            val id = IdStatement(text = lhs.toCode(), texTalkRoot = validationSuccess(lhs))
            val syntheticDefines = rhsDef.copy(id = id)
            aliasDefines.add(syntheticDefines)
        }
    }

    private fun attemptAddAliasDefinesForColonEqualsOperator(
        colonEquals: ColonEqualsTexTalkNode, aliasDefines: MutableList<DefinesGroup>
    ) {
        val lhsItems = colonEquals.lhs.items
        val rhsItems = colonEquals.rhs.items
        if (lhsItems.size != 1 || rhsItems.size != 1) {
            println(
                "The left-hand-side and right-hand-side of a := must have exactly one expression")
            return
        }

        // Given the statment: '\f(x) := \g(x)'
        // then lhs is `\f(x)`
        // and lhsVars is the set containing only `x`
        val lhs = lhsItems[0]
        val lhsVars =
            getVarsTexTalkNode(
                texTalkNode = lhs,
                isInLhsOfColonEqualsIsOrIn = true,
                groupScope = GroupScope.InNone,
                isInIdStatement = false,
                forceIsPlaceholder = false)
                .toSet()
                .map { it.name }

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

        val stmtText = getExpandedText(rhs, aliasDefines)
        if (stmtText != null) {
            val id = IdStatement(text = lhs.toCode(), texTalkRoot = validationSuccess(lhs))
            val syntheticDefines =
                DefinesGroup(
                    signature = id.signature(newLocationTracker()),
                    id = id,
                    definesSection = DefinesSection(targets = emptyList()),
                    givenSection = null,
                    whenSection = null,
                    meansSection = MeansSection(clauses = ClauseListNode(clauses = emptyList())),
                    expressesSection = null,
                    viewingSection = null,
                    usingSection = null,
                    writtenSection = WrittenSection(forms = listOf("\"${stmtText}\"")),
                    calledSection = CalledSection(forms = emptyList()),
                    metaDataSection = null)
            aliasDefines.add(syntheticDefines)
        }
    }

    override fun prettyPrint(
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
                        attemptAddAliasDefinesForColonEqualsOperator(colonEquals, aliasDefines)
                        attemptAddAliasDefinesForColonEqualsTuple(colonEquals, aliasDefines)
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
            val errs = checkVarsPhase2Node(grp.value.original, grp.value.normalized, tracker)
            result.addAll(
                errs.map { ValueSourceTracker(value = it, source = grp.source, tracker = tracker) })
        }
        return result
    }

    private fun getAllDefinedSignatures():
        List<Pair<ValueSourceTracker<Signature>, TopLevelGroup>> {
        val result =
            mutableListOf<Pair<ValueSourceTracker<Signature>, Normalized<out TopLevelGroup>>>()

        fun processDefines(pair: ValueSourceTracker<Normalized<DefinesGroup>>) {
            val signature = pair.value.normalized.signature
            if (signature != null) {
                val vst =
                    ValueSourceTracker(
                        source = pair.source, tracker = pair.tracker, value = signature)
                result.add(Pair(vst, pair.value))
            }
        }

        fun processStates(pair: ValueSourceTracker<Normalized<StatesGroup>>) {
            val signature = pair.value.normalized.signature
            if (signature != null) {
                val vst =
                    ValueSourceTracker(
                        source = pair.source, tracker = pair.tracker, value = signature)
                result.add(Pair(vst, pair.value))
            }
        }

        fun processAxiom(pair: ValueSourceTracker<Normalized<AxiomGroup>>) {
            val signature = pair.value.normalized.signature
            if (signature != null) {
                val vst =
                    ValueSourceTracker(
                        source = pair.source, tracker = pair.tracker, value = signature)
                result.add(Pair(vst, pair.value))
            }
        }

        fun processTheorems(pair: ValueSourceTracker<Normalized<TheoremGroup>>) {
            val signature = pair.value.normalized.signature
            if (signature != null) {
                val vst =
                    ValueSourceTracker(
                        source = pair.source, tracker = pair.tracker, value = signature)
                result.add(Pair(vst, pair.value))
            }
        }

        fun processConjectures(pair: ValueSourceTracker<Normalized<ConjectureGroup>>) {
            val signature = pair.value.normalized.signature
            if (signature != null) {
                val vst =
                    ValueSourceTracker(
                        source = pair.source, tracker = pair.tracker, value = signature)
                result.add(Pair(vst, pair.value))
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

        return result.map { Pair(first = it.first, second = it.second.normalized) }
    }

    override fun getInputOutputSymbolErrors(): List<ValueSourceTracker<ParseError>> {
        val errors = mutableListOf<ValueSourceTracker<ParseError>>()
        for (vst in allGroups) {
            val group = vst.value.normalized
            val tracker = vst.tracker ?: newLocationTracker()

            val inputs = getInputSymbols(group)
            val outputs = getOutputSymbols(group)

            val whenSection = getWhenSection(group)
            if (whenSection != null) {
                val usedSymbols =
                    findIsLhsSymbols(whenSection, tracker)
                        .toMutableList()
                        .plus(findColonEqualsLhsSymbols(whenSection, tracker))
                for (pair in usedSymbols) {
                    val sym = pair.first
                    val location = pair.second
                    if (outputs.contains(sym)) {
                        errors.add(
                            ValueSourceTracker(
                                value =
                                    ParseError(
                                        message =
                                            "A `when:` section cannot describe a symbol introduced in a `Defines:` section but found '${sym}'",
                                        row = location.row,
                                        column = location.column),
                                source = vst.source,
                                tracker = vst.tracker))
                    }
                }
            }

            for (meansOrEval in getMeansExpressesSections(group)) {
                val usedSymbols =
                    findIsLhsSymbols(meansOrEval, tracker)
                        .toMutableList()
                        .plus(findColonEqualsLhsSymbols(meansOrEval, tracker))
                for (pair in usedSymbols) {
                    val sym = pair.first
                    val location = pair.second
                    if (inputs.contains(sym) && !outputs.contains(sym)) {
                        errors.add(
                            ValueSourceTracker(
                                value =
                                    ParseError(
                                        message =
                                            "A `means:` or `expresses:` section cannot describe a symbol introduced in a [...] or `given:` section but found '${sym}'",
                                        row = location.row,
                                        column = location.column),
                                source = vst.source,
                                tracker = vst.tracker))
                    }
                }
            }
        }
        return errors
    }

    private fun isSignatureTopLevel(exp: TexTalkNode, signature: String) =
        exp is ExpressionTexTalkNode &&
            exp.children.size == 1 &&
            exp.children[0] is Command &&
            (exp.children[0] as Command).signature() == signature

    override fun getNonExpressesUsedInNonIsNonInStatementsErrors():
        List<ValueSourceTracker<ParseError>> {
        val errors = mutableListOf<ValueSourceTracker<ParseError>>()
        val signaturesWithoutExpresses =
            getSignaturesWithoutExpressesSection().map { it.value.form }.toSet()
        val statesSigs = statesGroups.mapNotNull { it.value.original.signature?.form }.toSet()
        val axiomSigs = axiomGroups.mapNotNull { it.value.original.signature?.form }.toSet()
        for (vst in allGroups) {
            val group = vst.value.normalized
            val tracker = vst.tracker ?: newLocationTracker()

            for (stmtPair in getNonIsNonInStatementsNonInAsSections(group)) {
                val stmt = stmtPair.first
                val exp = stmtPair.second
                for (sig in getSignaturesWithin(exp)) {
                    if (signaturesWithoutExpresses.contains(sig) &&
                        !statesSigs.contains(sig) &&
                        // a top level axiom signature is allowed
                        !(axiomSigs.contains(sig) && isSignatureTopLevel(exp, sig))) {
                        val location =
                            tracker.getLocationOf(stmt) ?: Location(row = -1, column = -1)
                        errors.add(
                            ValueSourceTracker(
                                value =
                                    ParseError(
                                        message =
                                            "Cannot use '$sig' in a non-`is` or non-`in` statement since its definition doesn't have an `expresses:` section",
                                        row = location.row,
                                        column = location.column),
                                source = vst.source,
                                tracker = vst.tracker))
                    }
                }
            }
        }
        return errors
    }
}

private fun getSignaturesWithin(node: TexTalkNode): List<String> {
    val result = mutableListOf<String>()
    getSignaturesImpl(node, result)
    return result
}

private fun getSignaturesImpl(node: TexTalkNode, result: MutableList<String>) {
    if (node is Command) {
        result.add(node.signature())
    } else {
        node.forEach { getSignaturesImpl(it, result) }
    }
}

private fun getNonIsNonInStatementsNonInAsSections(
    node: Phase2Node
): List<Pair<Statement, TexTalkNode>> {
    val result = mutableListOf<Pair<Statement, TexTalkNode>>()
    getNonIsNonInStatementsNonInAsSections(node, result)
    return result
}

private fun getNonIsNonInStatementsNonInAsSections(
    node: Phase2Node, result: MutableList<Pair<Statement, TexTalkNode>>
) {
    if (node is Statement) {
        when (val validation = node.texTalkRoot
        ) {
            is ValidationSuccess -> {
                val exp = validation.value
                if (exp.children.size != 1 ||
                    (exp.children[0] !is IsTexTalkNode && exp.children[0] !is InTexTalkNode)) {
                    result.add(Pair(node, exp))
                }
            }
            else -> {
                // invalid statements are not processed since it cannot be determined
                // if they are of the form `... is ...`
            }
        }
    } else if (node !is ViewingAsSection) {
        node.forEach { getNonIsNonInStatementsNonInAsSections(it, result) }
    }
}

private fun getWhenSection(topLevelGroup: TopLevelGroup) =
    when (topLevelGroup) {
        is DefinesGroup -> topLevelGroup.whenSection
        is StatesGroup -> topLevelGroup.whenSection
        is TheoremGroup -> topLevelGroup.whenSection
        is AxiomGroup -> topLevelGroup.whenSection
        is ConjectureGroup -> topLevelGroup.whenSection
        else -> null
    }

private fun getMeansExpressesSections(topLevelGroup: TopLevelGroup) =
    when (topLevelGroup) {
        is DefinesGroup -> {
            val result = mutableListOf<Phase2Node>()
            if (topLevelGroup.meansSection != null) {
                result.add(topLevelGroup.meansSection)
            }
            if (topLevelGroup.expressesSection != null) {
                result.add(topLevelGroup.expressesSection)
            }
            result
        }
        else -> emptyList()
    }

private fun getInputSymbols(topLevelGroup: TopLevelGroup): Set<String> {
    val result = mutableSetOf<String>()
    when (topLevelGroup) {
        is DefinesGroup -> {
            result.addAll(getVarsPhase2Node(topLevelGroup.id).map { it.name })
            if (topLevelGroup.givenSection != null) {
                result.addAll(getVarsPhase2Node(topLevelGroup.givenSection).map { it.name })
            }
        }
        is StatesGroup -> {
            result.addAll(getVarsPhase2Node(topLevelGroup.id).map { it.name })
            if (topLevelGroup.givenSection != null) {
                result.addAll(getVarsPhase2Node(topLevelGroup.givenSection).map { it.name })
            }
        }
        is TheoremGroup -> {
            if (topLevelGroup.id != null) {
                result.addAll(getVarsPhase2Node(topLevelGroup.id).map { it.name })
            }
            if (topLevelGroup.givenSection != null) {
                result.addAll(getVarsPhase2Node(topLevelGroup.givenSection).map { it.name })
            }
        }
        is AxiomGroup -> {
            if (topLevelGroup.id != null) {
                result.addAll(getVarsPhase2Node(topLevelGroup.id).map { it.name })
            }
            if (topLevelGroup.givenSection != null) {
                result.addAll(getVarsPhase2Node(topLevelGroup.givenSection).map { it.name })
            }
        }
        is ConjectureGroup -> {
            if (topLevelGroup.id != null) {
                result.addAll(getVarsPhase2Node(topLevelGroup.id).map { it.name })
            }
            if (topLevelGroup.givenSection != null) {
                result.addAll(getVarsPhase2Node(topLevelGroup.givenSection).map { it.name })
            }
        }
    }
    return result
}

private fun getOutputSymbols(topLevelGroup: TopLevelGroup): Set<String> {
    val result = mutableSetOf<String>()
    when (topLevelGroup) {
        is DefinesGroup -> {
            result.addAll(getVarsPhase2Node(topLevelGroup.definesSection).map { it.name })
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
                            IdStatement(text = lhs.toCode(), texTalkRoot = validationSuccess(lhs))
                        val syntheticDefines =
                            DefinesGroup(
                                signature = id.signature(newLocationTracker()),
                                id = id,
                                definesSection = DefinesSection(targets = emptyList()),
                                givenSection = null,
                                whenSection = null,
                                meansSection =
                                    MeansSection(clauses = ClauseListNode(clauses = emptyList())),
                                expressesSection = null,
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

private fun findIsRhsSignatures(
    node: Phase2Node, locationTracker: LocationTracker
): List<Signature> {
    val result = mutableListOf<Signature>()
    findIsRhsSignatures(node, locationTracker, result)
    return result
}

private fun findIsRhsSignatures(
    node: Phase2Node, locationTracker: LocationTracker, rhsIsSignatures: MutableList<Signature>
) {
    if (node is Statement) {
        when (node.texTalkRoot) {
            is ValidationSuccess -> {
                val location =
                    locationTracker.getLocationOf(node) ?: Location(row = -1, column = -1)
                findIsRhsSignatures(node.texTalkRoot.value, location, rhsIsSignatures)
            }
            else -> {
                // ignore statements that do not parse
            }
        }
    } else {
        node.forEach { findIsRhsSignatures(it, locationTracker, rhsIsSignatures) }
    }
}

private fun findIsRhsSignatures(
    node: TexTalkNode, location: Location, rhsIsSignatures: MutableList<Signature>
) {
    if (node is IsTexTalkNode) {
        node.rhs.items.forEach { item ->
            item.children.forEach { child ->
                if (child is Command) {
                    val signature = child.signature()
                    rhsIsSignatures.add(Signature(form = signature, location = location))
                }
            }
        }
    } else {
        node.forEach { findIsRhsSignatures(it, location, rhsIsSignatures) }
    }
}

private fun findInRhsSignatures(
    node: Phase2Node, locationTracker: LocationTracker
): List<Signature> {
    val result = mutableListOf<Signature>()
    findInRhsSignatures(node, locationTracker, result)
    return result
}

private fun findInRhsSignatures(
    node: Phase2Node, locationTracker: LocationTracker, rhsInSignatures: MutableList<Signature>
) {
    if (node is Statement) {
        when (node.texTalkRoot) {
            is ValidationSuccess -> {
                val location =
                    locationTracker.getLocationOf(node) ?: Location(row = -1, column = -1)
                findInRhsSignatures(node.texTalkRoot.value, location, rhsInSignatures)
            }
            else -> {
                // ignore statements that do not parse
            }
        }
    } else {
        node.forEach { findInRhsSignatures(it, locationTracker, rhsInSignatures) }
    }
}

private fun findInRhsSignatures(
    node: TexTalkNode, location: Location, rhsInSignatures: MutableList<Signature>
) {
    if (node is InTexTalkNode) {
        node.rhs.items.forEach { item ->
            item.children.forEach { child ->
                if (child is Command) {
                    val signature = child.signature()
                    rhsInSignatures.add(Signature(form = signature, location = location))
                }
            }
        }
    } else {
        node.forEach { findInRhsSignatures(it, location, rhsInSignatures) }
    }
}

private fun findColonEqualsRhsSignatures(
    node: Phase2Node, locationTracker: LocationTracker
): List<Signature> {
    val result = mutableListOf<Signature>()
    findColonEqualsRhsSignatures(node, locationTracker, result)
    return result
}

private fun findColonEqualsRhsSignatures(
    node: Phase2Node, locationTracker: LocationTracker, rhsIsSignatures: MutableList<Signature>
) {
    if (node is Statement) {
        when (node.texTalkRoot) {
            is ValidationSuccess -> {
                val location =
                    locationTracker.getLocationOf(node) ?: Location(row = -1, column = -1)
                findColonEqualsRhsSignatures(node.texTalkRoot.value, location, rhsIsSignatures)
            }
            else -> {
                // ignore statements that do not parse
            }
        }
    } else {
        node.forEach { findColonEqualsRhsSignatures(it, locationTracker, rhsIsSignatures) }
    }
}

private fun findColonEqualsRhsSignatures(
    node: TexTalkNode, location: Location, rhsIsSignatures: MutableList<Signature>
) {
    if (node is ColonEqualsTexTalkNode) {
        node.rhs.items.forEach { item ->
            item.children.forEach { child ->
                if (child is Command) {
                    val signature = child.signature()
                    rhsIsSignatures.add(Signature(form = signature, location = location))
                }
            }
        }
    } else {
        node.forEach { findColonEqualsRhsSignatures(it, location, rhsIsSignatures) }
    }
}

private fun findIsLhsSymbols(
    node: Phase2Node, locationTracker: LocationTracker
): List<Pair<String, Location>> {
    val result = mutableListOf<Pair<String, Location>>()
    findIsLhsSymbols(node, locationTracker, result)
    return result
}

private fun findIsLhsSymbols(
    node: Phase2Node,
    locationTracker: LocationTracker,
    lhsIsSymbols: MutableList<Pair<String, Location>>
) {
    if (node is Statement) {
        when (node.texTalkRoot) {
            is ValidationSuccess -> {
                val location =
                    locationTracker.getLocationOf(node) ?: Location(row = -1, column = -1)
                findIsLhsSymbols(node.texTalkRoot.value, location, lhsIsSymbols)
            }
            else -> {
                // ignore statements that do not parse
            }
        }
    } else {
        node.forEach { findIsLhsSymbols(it, locationTracker, lhsIsSymbols) }
    }
}

private fun findIsLhsSymbols(
    node: TexTalkNode, location: Location, lhsIsSymbols: MutableList<Pair<String, Location>>
) {
    if (node is IsTexTalkNode) {
        node.lhs.items.forEach {
            lhsIsSymbols.addAll(
                getVarsTexTalkNode(
                    it,
                    isInLhsOfColonEqualsIsOrIn = false,
                    groupScope = GroupScope.InNone,
                    isInIdStatement = false,
                    forceIsPlaceholder = false)
                    .map { symbol -> Pair(symbol.name, location) })
        }
    } else {
        node.forEach { findIsLhsSymbols(it, location, lhsIsSymbols) }
    }
}

private fun findColonEqualsLhsSymbols(
    node: Phase2Node, locationTracker: LocationTracker
): List<Pair<String, Location>> {
    val result = mutableListOf<Pair<String, Location>>()
    findColonEqualsLhsSymbols(node, locationTracker, result)
    return result
}

private fun findColonEqualsLhsSymbols(
    node: Phase2Node,
    locationTracker: LocationTracker,
    lhsColonEqualsSymbols: MutableList<Pair<String, Location>>
) {
    if (node is Statement) {
        when (node.texTalkRoot) {
            is ValidationSuccess -> {
                val location =
                    locationTracker.getLocationOf(node) ?: Location(row = -1, column = -1)
                findColonEqualsLhsSymbols(node.texTalkRoot.value, location, lhsColonEqualsSymbols)
            }
            else -> {
                // ignore statements that do not parse
            }
        }
    } else {
        node.forEach { findColonEqualsLhsSymbols(it, locationTracker, lhsColonEqualsSymbols) }
    }
}

private fun findColonEqualsLhsSymbols(
    node: TexTalkNode,
    location: Location,
    lhsColonEqualsSymbols: MutableList<Pair<String, Location>>
) {
    if (node is ColonEqualsTexTalkNode) {
        node.lhs.items.forEach {
            lhsColonEqualsSymbols.addAll(
                getVarsTexTalkNode(
                    it,
                    isInLhsOfColonEqualsIsOrIn = true,
                    groupScope = GroupScope.InNone,
                    isInIdStatement = false,
                    forceIsPlaceholder = false)
                    .map { symbol -> Pair(symbol.name, location) })
        }
    } else {
        node.forEach { findColonEqualsLhsSymbols(it, location, lhsColonEqualsSymbols) }
    }
}

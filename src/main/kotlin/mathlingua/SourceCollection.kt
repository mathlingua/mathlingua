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

package mathlingua

import java.io.File
import mathlingua.chalktalk.phase2.SymbolAnalyzer
import mathlingua.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.states.StatesGroup
import mathlingua.support.Location
import mathlingua.support.LocationTracker
import mathlingua.support.ParseError
import mathlingua.support.Validation
import mathlingua.support.ValidationFailure
import mathlingua.support.ValidationSuccess
import mathlingua.transform.locateAllSignatures

data class SourceFile(val file: File, val content: String, val validation: Validation<Parse>)

private fun isMathLinguaFile(file: File) = file.isFile && file.extension == "math"

private fun buildSourceFile(file: File): SourceFile {
    val content = file.readText()
    return SourceFile(
        file = file, content = content, validation = MathLingua.parseWithLocations(content))
}

data class ValueSourceTracker<T>(
    val value: T, val source: SourceFile, val tracker: LocationTracker?)

class SourceCollection(filesOrDirs: List<File>) {
    private val sourceFiles = mutableMapOf<File, SourceFile>()
    private val allGroups = mutableListOf<ValueSourceTracker<TopLevelGroup>>()
    private val definesGroups = mutableListOf<ValueSourceTracker<DefinesGroup>>()
    private val statesGroups = mutableListOf<ValueSourceTracker<StatesGroup>>()

    init {
        for (f in filesOrDirs) {
            if (f.isDirectory) {
                f.walk().filter { isMathLinguaFile(it) }.forEach {
                    sourceFiles[it] = buildSourceFile(it)
                }
            } else if (isMathLinguaFile(f)) {
                sourceFiles[f] = buildSourceFile(f)
            }
        }

        for (sf in sourceFiles.values) {
            val validation = sf.validation
            if (validation is ValidationSuccess) {
                definesGroups.addAll(
                    validation.value.document.defines().map {
                        ValueSourceTracker(
                            source = sf, tracker = validation.value.tracker, value = it)
                    })

                statesGroups.addAll(
                    validation.value.document.states().map {
                        ValueSourceTracker(
                            source = sf, tracker = validation.value.tracker, value = it)
                    })

                allGroups.addAll(
                    validation.value.document.groups.map {
                        ValueSourceTracker(
                            source = sf, tracker = validation.value.tracker, value = it)
                    })
            }
        }
    }

    private fun getAllDefinedSignatures(): List<ValueSourceTracker<Signature>> {
        val result = mutableListOf<ValueSourceTracker<Signature>>()
        for (pair in definesGroups) {
            val signature = pair.value.signature
            if (signature != null) {
                result.add(
                    ValueSourceTracker(
                        source = pair.source,
                        tracker = pair.tracker,
                        value =
                            Signature(
                                form = signature,
                                location = pair.tracker?.getLocationOf(pair.value)
                                        ?: Location(row = -1, column = -1))))
            }
        }
        for (pair in statesGroups) {
            val signature = pair.value.signature
            if (signature != null) {
                result.add(
                    ValueSourceTracker(
                        source = pair.source,
                        tracker = pair.tracker,
                        value =
                            Signature(
                                form = signature,
                                location = pair.tracker?.getLocationOf(pair.value)
                                        ?: Location(row = -1, column = -1))))
            }
        }
        return result
    }

    fun getDefinedSignatures(): Set<ValueSourceTracker<Signature>> {
        val result = mutableSetOf<ValueSourceTracker<Signature>>()
        result.addAll(getAllDefinedSignatures())
        return result
    }

    fun getDuplicateDefinedSignatures(): List<ValueSourceTracker<Signature>> {
        val duplicates = mutableListOf<ValueSourceTracker<Signature>>()
        val found = mutableSetOf<String>()
        for (sig in getAllDefinedSignatures()) {
            if (found.contains(sig.value.form)) {
                duplicates.add(sig)
            }
            found.add(sig.value.form)
        }
        return duplicates
    }

    fun getUsedSignatures(): Set<ValueSourceTracker<Signature>> {
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

    fun getUndefinedSignatures(): Set<ValueSourceTracker<Signature>> {
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

    fun findInvalidTypes(): List<ValueSourceTracker<ParseError>> {
        val result = mutableListOf<ValueSourceTracker<ParseError>>()
        val analyzer = SymbolAnalyzer(definesGroups.map { it.value })
        for (sf in sourceFiles) {
            val validation = sf.value.validation
            when (validation) {
                is ValidationSuccess -> {
                    val tracker = validation.value.tracker
                    val doc = validation.value.document
                    for (grp in doc.groups) {
                        val errors = analyzer.findInvalidTypes(grp, tracker)
                        result.addAll(
                            errors.map {
                                ValueSourceTracker(source = sf.value, tracker = tracker, value = it)
                            })
                    }
                }
            }
        }
        return result
    }

    fun getParseErrors(): List<ValueSourceTracker<ParseError>> {
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
            }
        }
        return result
    }

    fun getDuplicateContent(): List<ValueSourceTracker<TopLevelGroup>> {
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
}

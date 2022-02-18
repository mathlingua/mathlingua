/*
 * Copyright 2022 The MathLingua Authors
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

package mathlingua.cli

import mathlingua.MutableMultiSet
import mathlingua.backend.SourceFile
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.newMutableMultiSet

internal interface SearchIndex {
    fun add(sf: SourceFile)
    fun remove(path: List<String>)
    fun search(query: String): List<List<String>>
}

internal fun newSearchIndex(fs: VirtualFileSystem): SearchIndex {
    return SearchIndexImpl(fs)
}

// -----------------------------------------------------------------------------

private data class SearchIndexImpl(val fs: VirtualFileSystem) : SearchIndex {
    private val searchIndex = mutableMapOf<String, MutableMultiSet<List<String>>>()

    override fun add(sf: SourceFile) {
        when (val validation = sf.validation
        ) {
            is ValidationSuccess -> {
                val relPath = sf.file.relativePath()
                val doc = validation.value
                for (grp in doc.groups) {
                    for (word in getAllWords(grp)) {
                        val key = word.lowercase()
                        val pathSet = searchIndex[key] ?: newMutableMultiSet()
                        pathSet.add(relPath.split("/"))
                        searchIndex[key] = pathSet
                    }
                }
            }
        }
    }

    override fun remove(path: List<String>) {
        val keysToRemove = mutableListOf<String>()
        for (word in searchIndex.keys) {
            val key = word.lowercase()
            val pathSet = searchIndex[key]
            if (pathSet != null) {
                pathSet.remove(path)
                if (pathSet.isEmpty()) {
                    keysToRemove.add(key)
                }
            }
        }

        for (key in keysToRemove) {
            searchIndex.remove(key)
        }
    }

    override fun search(query: String): List<List<String>> {
        val terms =
            query.lowercase().split(" ").map { it.trim().lowercase() }.filter { it.isNotEmpty() }

        if (terms.isEmpty()) {
            return emptyList()
        }

        var result = searchIndex[terms[0]]?.toSet() ?: emptySet()
        if (result.isEmpty()) {
            return emptyList()
        }

        for (i in 1 until terms.size) {
            result = result.intersect(searchIndex[terms[i]]?.toSet() ?: emptySet())
        }

        return result.toList()
    }
}

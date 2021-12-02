package mathlingua.cli

import mathlingua.MutableMultiSet
import mathlingua.backend.SourceFile
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.newMutableMultiSet

data class SearchIndex(val fs: VirtualFileSystem) {
    private val searchIndex = mutableMapOf<String, MutableMultiSet<List<String>>>()

    fun add(sf: SourceFile) {
        when (val validation = sf.validation
        ) {
            is ValidationSuccess -> {
                val relPath = sf.file.relativePath()
                val doc = validation.value.document
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

    fun remove(path: List<String>) {
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

    fun search(query: String): List<List<String>> {
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

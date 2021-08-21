package mathlingua.cli

data class AutoComplete(private val preserveCase: Boolean) {
    private val root = TrieNode(count = 1, isWord = false, children = mutableMapOf())

    private fun preProcess(word: String) =
        if (preserveCase) {
            word
        } else {
            word.lowercase()
        }

    fun add(word: String) {
        val processedWord = preProcess(word)
        addImpl(root, processedWord, 0)
    }

    private fun addImpl(trieNode: TrieNode, word: String, index: Int) {
        if (index >= word.length) {
            return
        }

        val c = "" + word[index]
        if (!trieNode.children.containsKey(c)) {
            trieNode.children[c] =
                TrieNode(count = 1, isWord = index == word.length - 1, children = mutableMapOf())
        }

        val subNode = trieNode.children[c]!!
        subNode.count++

        if (index == word.length - 1) {
            subNode.isWord = true
        }

        addImpl(subNode, word, index + 1)
    }

    fun remove(word: String) {
        val processedWord = preProcess(word)
        removeImpl(root, processedWord, 0)
    }

    private fun removeImpl(trieNode: TrieNode, word: String, index: Int) {
        if (index >= word.length) {
            return
        }

        val c = "" + word[index]
        if (trieNode.children.containsKey(c)) {
            val subNode = trieNode.children[c]!!
            subNode.count--
            if (subNode.count == 0) {
                trieNode.children.remove(c)
            } else {
                removeImpl(subNode, word, index + 1)
            }
        }
    }

    fun findSuffixes(word: String): List<String> {
        val processedWord = preProcess(word)
        val node = findTrieLeaf(root, processedWord, 0) ?: return emptyList()
        val result = mutableSetOf<String>()
        getWordsUnder(StringBuilder(), node, result)
        return result.toList()
    }

    private fun findTrieLeaf(trieNode: TrieNode, word: String, index: Int): TrieNode? {
        if (index >= word.length) {
            return trieNode
        }

        val c = "" + word[index]
        if (trieNode.children.containsKey(c)) {
            return this.findTrieLeaf(trieNode.children[c]!!, word, index + 1)
        }

        return null
    }

    private fun getWordsUnder(
        builder: StringBuilder, trieNode: TrieNode, result: MutableSet<String>
    ) {
        if (trieNode.isWord) {
            result.add(builder.toString())
        }

        for (c in trieNode.children.keys) {
            builder.append(c)
            getWordsUnder(builder, trieNode.children[c]!!, result)
            builder.setLength(builder.length - 1)
        }
    }
}

private data class TrieNode(
    var count: Int, var isWord: Boolean, val children: MutableMap<String, TrieNode>)

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

package mathlingua.common.chalktalk.phase2

import kotlin.math.abs

fun indentedString(useDot: Boolean, indent: Int, line: String): String {
    val builder = StringBuilder()
    for (i in 0 until indent - 2) {
        builder.append(' ')
    }
    if (indent - 2 >= 0) {
        builder.append(if (useDot) '.' else ' ')
    }
    if (indent - 1 >= 0) {
        builder.append(' ')
    }
    builder.append(line)
    return builder.toString()
}

fun getChalkTalkAncestry(root: Phase2Node, node: Phase2Node): List<Phase2Node> {
    val path = mutableListOf<Phase2Node>()
    getChalkTalkAncestryImpl(root, node, path)
    // 'node' itself shouldn't be in the ancestry
    if (path.isNotEmpty()) {
        path.removeAt(path.size - 1)
    }
    return path.reversed()
}

private fun getChalkTalkAncestryImpl(root: Phase2Node, node: Phase2Node, path: MutableList<Phase2Node>) {
    if (root == node) {
        path.add(node)
        return
    }

    path.add(root)
    root.forEach {
        if (path.isEmpty() || path.last() != node) {
            getChalkTalkAncestryImpl(it, node, path)
        }
    }
    if (path.isEmpty() || path.last() != node) {
        path.removeAt(path.size - 1)
    }
}

fun findNearestChalkTalkAncestorWhere(root: Phase2Node, from: Phase2Node, predicate: (node: Phase2Node) -> Boolean): Phase2Node? {
    val ancestry = getChalkTalkAncestry(root, from)
    for (a in ancestry) {
        if (predicate(a)) {
            return a
        }
    }
    return null
}

fun findNode(node: Phase2Node, row: Int, col: Int): Phase2Node {
    val result = NearestNode(dist = Integer.MAX_VALUE, node = node)
    findNodeImpl(node, row, col, result)
    return result.node
}

private fun findNodeImpl(node: Phase2Node,
                         row: Int, col: Int,
                         result: NearestNode) {
    val d = dist(node, row, col)
    if (d <= result.dist) {
        result.dist = d
        result.node = node
    }

    node.forEach { findNodeImpl(it, row, col, result) }
}

private data class NearestNode(var dist: Int, var node: Phase2Node)

private fun dist(node: Phase2Node, row: Int, col: Int): Int {
    if (node.row != row) {
        return Integer.MAX_VALUE
    }

    if (node.column > col) {
        return Integer.MAX_VALUE
    }

    return col - node.column
}

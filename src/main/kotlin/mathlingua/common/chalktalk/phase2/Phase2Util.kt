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

import mathlingua.common.chalktalk.phase2.ast.Phase2Node

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

fun findNode(node: Phase2Node, row: Int, col: Int): Phase2Node {
    if (row < 0 || col < 0) {
        throw IllegalArgumentException("Row and column must be non-negative " +
                "but found row=$row, column=$col")
    }

    val nodesAtRow = mutableSetOf<Phase2Node>()
    findNodesAtRow(node, row, nodesAtRow)
    if (nodesAtRow.isEmpty()) {
        return if (row == 0) {
            node
        } else {
            findNode(node, row - 1, col)
        }
    }

    val sortedByCol = nodesAtRow.toList().sortedBy { it.column }
    var prev: Phase2Node = sortedByCol.first()
    for (n in sortedByCol) {
        if (n.column > col) {
            return prev
        }
        prev = n
    }
    return prev
}

fun findNodesAtRow(node: Phase2Node, row: Int, result: MutableSet<Phase2Node>) {
    if (node.row == row) {
        result.add(node)
    }
    node.forEach { findNodesAtRow(it, row, result) }
}

fun hasChild(node: Phase2Node, child: Phase2Node): Boolean {
    resetRowColumn(node)
    resetRowColumn(child)

    if (node == child) {
        return true
    }

    var found = false
    node.forEach {
        if (!found) {
            found = hasChild(it, child)
        }
    }

    return found
}

fun resetRowColumn(node: Phase2Node) {
    node.row = -1
    node.column = -1
    node.forEach { resetRowColumn(it) }
}

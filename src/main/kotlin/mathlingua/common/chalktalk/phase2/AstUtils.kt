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

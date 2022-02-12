/*
 * Copyright 2021 The MathLingua Authors
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

package mathlingua.frontend.textalk

internal fun isOpChar(c: Char) =
    (c == '!' ||
        c == '@' ||
        c == '%' ||
        c == '&' ||
        c == '*' ||
        c == '-' ||
        c == '+' ||
        c == '=' ||
        c == '|' ||
        c == '/' ||
        c == '<' ||
        c == '>')

internal fun getTexTalkAncestry(root: TexTalkNode, node: TexTalkNode): List<TexTalkNode> {
    val path = mutableListOf<TexTalkNode>()
    getTexTalkAncestryImpl(root, node, path)
    // 'node' itself shouldn't be in the ancestry
    if (path.isNotEmpty()) {
        path.removeAt(path.size - 1)
    }
    return path.reversed()
}

private fun getTexTalkAncestryImpl(
    root: TexTalkNode, node: TexTalkNode, path: MutableList<TexTalkNode>
) {
    if (root == node) {
        path.add(node)
        return
    }

    path.add(root)
    root.forEach {
        if (path.isEmpty() || path.last() != node) {
            getTexTalkAncestryImpl(it, node, path)
        }
    }
    if (path.isEmpty() || path.last() != node) {
        path.removeAt(path.size - 1)
    }
}

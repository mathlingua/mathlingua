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

import mathlingua.frontend.chalktalk.phase2.ast.clause.Identifier
import mathlingua.frontend.chalktalk.phase2.ast.clause.Statement
import mathlingua.frontend.chalktalk.phase2.ast.clause.Text
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.HasSignature
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelBlockComment
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.resource.ResourceGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item.StringItem
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.ContentItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.NameItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.OffsetItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.PageItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.SiteItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.StringSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.topic.TopicGroup
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.textalk.TexTalkNode
import mathlingua.frontend.textalk.TextTexTalkNode

internal fun getAllWords(node: Phase2Node): Set<String> {
    val result = mutableSetOf<String>()
    getAllWordsImpl(node, result)
    return result
}

// -----------------------------------------------------------------------------

private fun getAllWordsImpl(node: Phase2Node, words: MutableSet<String>) {
    if (node is HasSignature && node.signature != null) {
        words.add("[${node.signature!!.form}]")
    }

    when (node) {
        is ResourceGroup -> {
            // searching for a reference with or without @ in front
            // should find the associated reference group
            words.add(node.id)
            words.add(node.id.removePrefix("@"))
        }
        is DefinesGroup -> {
            if (node.signature != null) {
                words.add(node.signature.form)
            }
            when (val validation = node.id.texTalkRoot
            ) {
                is ValidationSuccess -> {
                    getAllWordsImpl(validation.value, words)
                }
            }
        }
        is StringItem -> {
            getAllWordsImpl(node.text, words)
        }
        is StringSection -> {
            getAllWordsImpl(node.name, words)
            for (value in node.values) {
                getAllWordsImpl(value, words)
            }
        }
        is ContentItemSection -> {
            getAllWordsImpl(node.content, words)
        }
        is NameItemSection -> {
            getAllWordsImpl(node.name, words)
        }
        is OffsetItemSection -> {
            getAllWordsImpl(node.offset, words)
        }
        is PageItemSection -> {
            getAllWordsImpl(node.page, words)
        }
        is SiteItemSection -> {
            getAllWordsImpl(node.url, words)
        }
        is Statement -> {
            val root = node.texTalkRoot
            if (root is ValidationSuccess) {
                getAllWordsImpl(root.value, words)
            }
        }
        is Identifier -> {
            words.add(node.name.lowercase())
        }
        is Text -> {
            getAllWordsImpl(node.text, words)
        }
        is TopLevelBlockComment -> {
            getAllWordsImpl(node.blockComment.text.removeSurrounding("::", "::"), words)
        }
        is TopicGroup -> {
            getAllWordsImpl(node.contentSection.text, words)
            if (node.id != null) {
                getAllWordsImpl(node.id, words)
            }
            for (name in node.topicSection.names) {
                getAllWordsImpl(name, words)
            }
        }
    }

    node.forEach { getAllWordsImpl(it, words) }
}

private fun getAllWordsImpl(node: TexTalkNode, words: MutableSet<String>) {
    when (node) {
        is TextTexTalkNode -> {
            getAllWordsImpl(node.text, words)
        }
    }

    node.forEach { getAllWordsImpl(it, words) }
}

private fun getAllWordsImpl(text: String, words: MutableSet<String>) {
    val builder = StringBuilder()
    for (c in text) {
        if (c.isLetterOrDigit()) {
            builder.append(c)
        } else {
            builder.append(' ')
        }
    }

    words.addAll(
        builder
            .toString()
            .replace("\r", " ")
            .replace("\n", " ")
            .split(" ")
            .map { it.trim() }
            .filter { it.isNotEmpty() }
            .map { sanitizeHtmlForJs(it.lowercase()) })
}

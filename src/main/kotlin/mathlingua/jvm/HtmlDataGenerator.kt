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

package mathlingua.jvm

import mathlingua.common.MathLingua
import mathlingua.common.chalktalk.phase2.MetaDataSection
import mathlingua.common.chalktalk.phase2.Phase2Node
import java.lang.NumberFormatException

object HtmlDataGenerator {
    @JvmStatic
    fun main(args: Array<String>) {
        val text = MATHLINGUA_SOURCE_FILE.readText()

        val resultForSources = MathLingua().parse(text)
        val srcIdToUrlMap = mutableMapOf<String, String>()
        val srcIdToOffsetMap = mutableMapOf<String, Int>()
        if (resultForSources.document != null) {
            for (src in resultForSources.document.sources) {
                var offset = 0
                var url: String? = null
                for (mapping in src.sourceSection.mappings) {
                    if (mapping.mapping.lhs.text == "offset") {
                        val rawText = mapping.mapping.rhs.text
                        // the rawText is of the form "..."
                        // so the " and " need to be removed
                        val numText = rawText.substring(1, rawText.length - 1)
                        try {
                            offset = Integer.parseInt(numText)
                        } catch (err: NumberFormatException) {
                            throw NumberFormatException("Invalid offset '$numText'.  Expected an integer.")
                        }
                    }

                    if (mapping.mapping.lhs.text == "url") {
                        val rawText = mapping.mapping.rhs.text
                        // the text is of the form "..."
                        // so the " and " need to be removed
                        url = rawText.substring(1, rawText.length - 1)
                    }
                }

                val key = "@${src.id}"
                srcIdToOffsetMap[key] = offset

                if (url != null) {
                    srcIdToUrlMap[key] = url
                }
            }
        }

        val parts = text.split("\n\n")
            .map { it.trim() }
            .filter { it.isNotBlank() }

        println(
            """
            window.MATHLINGUA_DATA = window.MATHLINGUA_DATA || [
        """.trimIndent()
        )

        for (part in parts) {
            val result = MathLingua().parse(part)
            val keywords = mutableSetOf<String>()
            var href: String? = null
            var mobileHref: String? = null
            if (result.document != null) {
                findKeywords(keywords, result.document)

                val metadata: MetaDataSection?
                if (result.document.defines.isNotEmpty()) {
                    metadata = result.document.defines.first().metaDataSection
                } else if (result.document.represents.isNotEmpty()) {
                    metadata = result.document.represents.first().metaDataSection
                } else if (result.document.results.isNotEmpty()) {
                    metadata = result.document.results.first().metaDataSection
                } else if (result.document.axioms.isNotEmpty()) {
                    metadata = result.document.axioms.first().metaDataSection
                } else if (result.document.conjectures.isNotEmpty()) {
                    metadata = result.document.conjectures.first().metaDataSection
                } else {
                    metadata = null
                }
                if (metadata != null) {
                    for (mapping in metadata.mappings) {
                        val lhs = mapping.mapping.lhs
                        if (lhs.text == "reference") {
                            val rhs = mapping.mapping.rhs
                            // the rhs is of the form "..."
                            // so remove the leading and trailing "
                            val rhsParts = rhs.text.substring(1, rhs.text.length - 1).split(";")
                            val map = mutableMapOf<String, String>()
                            for (rhsPart in rhsParts) {
                                val keyValue = rhsPart.split(":")
                                if (keyValue.size == 2) {
                                    val key = keyValue[0].trim()
                                    val value = keyValue[1].trim()
                                    map[key] = value
                                }
                            }

                            if (map.containsKey("source") && srcIdToUrlMap.containsKey(map["source"])) {
                                val srcKey = map["source"]
                                val ref = srcIdToUrlMap[srcKey]
                                href = ref
                                mobileHref = ref
                                if (map.containsKey("page")) {
                                    val offset = srcIdToOffsetMap[srcKey]!!
                                    val pageNum = Integer.parseInt(map["page"])
                                    // the page labeled 1 in the pdf is the 15th page of
                                    // the pdf document
                                    href += "#page=${pageNum + offset}"
                                    // mobile browsers expect no "=" to be present
                                    mobileHref += "#page${pageNum + offset}"
                                }
                            }
                        }
                    }
                }
            }
            val keywordList = keywords.toList()
            val builder = StringBuilder()
            builder.append("[")
            for (i in 0 until keywordList.size) {
                builder.append("\"${keywordList[i]}\"")
                if (i != keywordList.size - 1) {
                    builder.append(", ")
                }
            }
            builder.append("]")
            println(
                """
                {
                  "text": "${part.replace("\\", "\\\\")
                    .replace("\n", "\\n")
                    .replace("\"", "\\\"")}",
                  "keywords": $builder,
                  "href": "$href",
                  "mobileHref": "$mobileHref",
                },
            """.trimIndent()
            )
        }
        println("];")
    }

    fun findKeywords(keywords: MutableSet<String>, node: Phase2Node) {
        var hasChildren = false
        node.forEach {
            hasChildren = true
            findKeywords(keywords, it)
        }

        if (!hasChildren) {
            val code = node.toCode(false, 0)
                .trim()
                .toLowerCase()
                .replace("\"", " ")
                .replace("'", " ")
                .replace("$", " ")
                .replace("\\", " ")
                .replace(".", " ")
                .replace(",", " ")
                .replace(":", " ")
                .replace(";", " ")
                .replace("^", " ")
                .replace("_", " ")
                .replace("{", " ")
                .replace("}", " ")
                .replace("[", " ")
                .replace("]", " ")
                .replace("(", " ")
                .replace(")", " ")
                .replace("@", " ")
            for (word in code.split(" ")) {
                if (word.isNotBlank()) {
                    keywords.add(word)
                }
            }
        }
    }
}

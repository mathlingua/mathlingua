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

package mathlingua.ui

import mathlingua.chalktalk.phase2.MetaDataSection
import mathlingua.chalktalk.phase2.Phase2Node
import mathlingua.common.MathLingua

object HtmlDataGenerator {
    @JvmStatic
    fun main(args: Array<String>) {
        val text = SOURCE_FILE.readText()
        val parts = text.split("\n\n")
            .map { it.trim() }
            .filter { it.isNotBlank() }

        println(
            """
            define(() => {
              return {
                getData: () => {
                  return [
        """.trimIndent()
        )

        for (part in parts) {
            val result = MathLingua.parse(part)
            val keywords = mutableSetOf<String>()
            var href: String? = null
            var mobileHref: String? = null
            if (result.document != null) {
                findKeywords(keywords, result.document)
                val metadata: MetaDataSection?
                if (result.document.defines.isNotEmpty()) {
                    metadata = result.document.defines.first().metaDataSection
                } else if (result.document.refines.isNotEmpty()) {
                    metadata = result.document.refines.first().metaDataSection
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
                                val keyValue = part.split(":")
                                if (keyValue.size == 2) {
                                    val key = keyValue[0].trim().toLowerCase()
                                    val value = keyValue[1].trim().toLowerCase()
                                    map.put(key, value)
                                }
                            }

                            if (map.containsKey("source") && map.get("source") == "@aata") {
                                val ref = "http://abstract.ups.edu/download/aata-20190710-print.pdf"
                                href = ref
                                mobileHref = ref
                                if (map.containsKey("page")) {
                                    val pageNum = Integer.parseInt(map.get("page"))
                                    // the page labeled 1 in the pdf is the 15th page of
                                    // the pdf document
                                    href += "#page=${pageNum + 14}"
                                    // mobile browsers expect no "=" to be present
                                    mobileHref += "#page${pageNum + 14}"
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
                  "mobileHref": "$mobileHref"
                },
            """.trimIndent()
            )
        }

        println(
            """
                  ];
                }
              };
            });
        """.trimIndent()
        )
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

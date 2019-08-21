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

import mathlingua.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.MathLingua

object HtmlDataGenerator {
    @JvmStatic
    fun main(args: Array<String>) {
        val text = SOURCE_FILE.readText()
        val parts = text.split("\n\n")
            .map { it.trim() }
            .filter { it.isNotBlank() }

        println("""
            define(() => {
              return {
                getData: () => {
                  return [
        """.trimIndent())

        for (part in parts) {
            val result = MathLingua.parse(part)
            val keywords = mutableSetOf<String>()
            if (result.document != null) {
                findKeywords(keywords, result.document)
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
            println("""
                {
                  "text": "${part.replace("\\", "\\\\")
                                 .replace("\n", "\\n")
                                 .replace("\"", "\\\"")}",
                  "keywords": $builder,
                },
            """.trimIndent())
        }

        println("""
                  ];
                }
              };
            });
        """.trimIndent())
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

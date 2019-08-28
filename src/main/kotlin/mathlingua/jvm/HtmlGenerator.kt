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

object HtmlGenerator {
    @JvmStatic
    fun main(args: Array<String>) {
        val text = MATHLINGUA_SOURCE_FILE.readText()
        val parts = text.split("\n\n")
                .map { it.trim() }
                .filter { it.isNotBlank() }

        println(
                """
                <!doctype html>
                <html lang="en">
                <head>
                  <style type="text/css" media="screen">
                    body {
                      background: #eeeeee;
                    }

                    .centered {
                      background: #ffffff;
                      width: 40%;
                      margin-left: auto;
                      margin-right: auto;
                      box-shadow: 2px 2px 10px 1px #999999;
                    }

                    .padded {
                      padding-left: 10px;
                    }
                  </style>
                  <link rel="stylesheet"
                        href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/9.15.10/styles/atom-one-light.min.css">
                  <script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/9.15.10/highlight.min.js"></script>
                  <script
                    charset="UTF-8"
                    src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/9.15.9/languages/yaml.min.js"></script>
                  <script>hljs.initHighlightingOnLoad();</script>
                  <title>MathLingua</title>
                </head>
                <body>
        """.trimIndent()
        )

        for (part in parts) {
            println(
                    """
  <div class="centered">
    <pre class="padded">
        <code class="yaml">
$part
        </code>
    </pre>
  </div>
            """
            )
        }

        println(
                """
            </body>
            </html>
        """.trimIndent()
        )
    }
}

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
import mathlingua.common.ValidationSuccess

object HtmlCompletionsGenerator {
    @JvmStatic
    fun main(args: Array<String>) {
        val text = MATHLINGUA_SOURCE_FILE.readText()
        val parts = text.split("\n\n")
            .map { it.trim() }
            .filter { it.isNotBlank() }

        println(
            """
            window.MATHLINGUA_AUTOCOMPLETIONS = window.MATHLINGUA_AUTOCOMPLETIONS || [
        """.trimIndent()
        )

        val ml = MathLingua()
        val signatures = mutableSetOf<String>()
        for (part in parts) {
            val result = ml.parse(part)
            if (result is ValidationSuccess) {
                signatures.addAll(ml.findAllSignatures(result.value))
            }
        }

        var count = 0
        for (sig in signatures) {
            count++
            print("\"${sig.replace("\\", "\\\\")}\"")
            if (count != signatures.size) {
                println(",")
            }
        }

        println("];")
    }
}

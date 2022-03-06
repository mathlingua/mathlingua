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

package mathlingua.frontend.chalktalk.phase1.ast

import mathlingua.frontend.textalk.isOpChar

internal fun Phase1Node.deepForEachTopDown(fn: (node: Phase1Node) -> Boolean) {
    val doContinue = fn(this)
    if (doContinue) {
        this.forEach { it.deepForEachTopDown(fn) }
    }
}

fun isOperatorName(text: String): Boolean {
    var underscoreIndex = text.indexOf('_')
    if (underscoreIndex < 0) {
        underscoreIndex = text.length
    }
    var i = 0
    // treat G.* forms as an operator
    val dotIndex = text.indexOf(".")
    if (dotIndex in 0 until underscoreIndex) {
        i = dotIndex + 1
    }
    while (i < underscoreIndex) {
        if (!isOpChar(text[i++])) {
            return false
        }
    }
    return true
}

internal fun Phase1Node.findAllPhase1Statements(): List<Phase1Token> {
    val result = mutableListOf<Phase1Token>()
    this.deepForEachTopDown { n ->
        if (n is Phase1Token && n.type == ChalkTalkTokenType.Statement) {
            result.add(n)
        }
        true
    }
    return result
}

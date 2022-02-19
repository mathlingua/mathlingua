/*
 * Copyright 2019 The MathLingua Authors
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

package mathlingua.frontend.chalktalk.phase1

import kotlin.math.max

internal fun buildIndent(level: Int, isArg: Boolean): String {
    val buffer = StringBuilder()
    val numSpaces = if (isArg) 2 * max(level - 1, 0) else 2 * level
    for (i in 0 until numSpaces) {
        buffer.append(' ')
    }
    if (isArg) {
        buffer.append(". ")
    }
    return buffer.toString()
}

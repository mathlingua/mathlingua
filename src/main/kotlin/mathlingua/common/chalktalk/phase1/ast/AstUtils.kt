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

package mathlingua.common.chalktalk.phase1.ast

private fun max(val1: Int, val2: Int): Int {
    return if (val1 >= val2) val1 else val2
}

fun buildIndent(level: Int, isArg: Boolean): String {
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

fun getIndent(size: Int): String {
    val buffer = StringBuilder()
    for (i in 0 until size) {
        buffer.append(' ')
    }
    return buffer.toString()
}

fun getRow(node: ChalkTalkNode): Int {
    if (node is ChalkTalkToken) {
        return node.row
    }
    var rowResult = -1
    node.forEach {
        if (rowResult == -1) {
            val row = getRow(it)
            if (row >= 0) {
                rowResult = row
            }
        }
    }
    return rowResult
}

fun getColumn(node: ChalkTalkNode): Int {
    if (node is ChalkTalkToken) {
        return node.column
    }
    var colResult = -1
    node.forEach {
        if (colResult == -1) {
            val col = getColumn(it)
            if (col >= 0) {
                colResult = col
            }
        }
    }
    return colResult
}

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

package mathlingua.common

import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase1.ast.getColumn
import mathlingua.common.chalktalk.phase1.ast.getRow
import mathlingua.common.chalktalk.phase2.ast.Phase2Node

data class ParseError(
    override val message: String,
    val row: Int,
    val column: Int
) : RuntimeException(message)

data class Location(val row: Int, val column: Int)

interface LocationTracker {
    fun hasLocationOf(node: Phase2Node): Boolean
    fun getLocationOf(node: Phase2Node): Location?
}

interface MutableLocationTracker : LocationTracker {
    fun setLocationOf(node: Phase2Node, location: Location)
}

private class MutableLocationTrackerImpl : MutableLocationTracker {
    private val map: MutableMap<Phase2Node, Location> = mutableMapOf()

    override fun hasLocationOf(node: Phase2Node) = map.containsKey(node)

    override fun getLocationOf(node: Phase2Node) = map[node]

    override fun setLocationOf(node: Phase2Node, location: Location) {
        map[node] = location
    }
}

fun newLocationTracker(): MutableLocationTracker {
    return MutableLocationTrackerImpl()
}

sealed class Validation<out T>
abstract class ValidationSuccess<T>(val value: T) : Validation<T>()
abstract class ValidationFailure<T>(val errors: List<ParseError>) : Validation<T>()

data class ValidationSuccessImpl<T>(val v: T) : ValidationSuccess<T>(v)
data class ValidationFailureImpl<T>(val errs: List<ParseError>) : ValidationFailure<T>(errs)

fun <T> validationSuccess(value: T): ValidationSuccess<T> = ValidationSuccessImpl(value)

fun <T : Phase2Node> validationSuccess(tracker: MutableLocationTracker, phase1Node: Phase1Node, phase2Node: T): ValidationSuccess<T> {
    tracker.setLocationOf(phase2Node, Location(
            row = getRow(phase1Node),
            column = getColumn(phase1Node)
    ))
    return ValidationSuccessImpl(phase2Node)
}

fun <T> validationFailure(errors: List<ParseError>): ValidationFailure<T> =
    ValidationFailureImpl(errors)

private fun tokenize(text: String): List<String> {
    val tokens = mutableListOf<String>()
    var i = 0
    while (i < text.length) {
        if (text[i] == ' ') {
            val buffer = StringBuilder()
            while (i < text.length && text[i] == ' ') {
                buffer.append(text[i++])
            }
            tokens.add(buffer.toString())
        } else {
            val buffer = StringBuilder()
            while (i < text.length && text[i] != ' ') {
                buffer.append(text[i++])
            }
            tokens.add(buffer.toString())
        }
    }
    return tokens
}

internal fun justify(text: String, width: Int): List<String> {
    val tokens = tokenize(text)
    val lines = mutableListOf<String>()
    var i = 0
    while (i < tokens.size) {
        val curLine = StringBuilder()
        while (i < tokens.size && curLine.isEmpty() && tokens[i].isBlank()) {
            i++
        }
        while (i < tokens.size && curLine.length + tokens[i].length <= width) {
            curLine.append(tokens[i++])
        }
        if (i < tokens.size && curLine.isEmpty()) {
            curLine.append(tokens[i++])
        }
        lines.add(curLine.toString())
    }
    return lines
}

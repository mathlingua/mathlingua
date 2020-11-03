/*
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *    http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package mathlingua.support

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase1.ast.getColumn
import mathlingua.chalktalk.phase1.ast.getRow
import mathlingua.chalktalk.phase2.ast.common.Phase2Node

data class ParseError(
    override val message: String,
    val row: Int,
    val column: Int
) : RuntimeException(message)

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
    )
    )
    return ValidationSuccessImpl(phase2Node)
}

fun <T> validationFailure(errors: List<ParseError>): ValidationFailure<T> =
    ValidationFailureImpl(errors)

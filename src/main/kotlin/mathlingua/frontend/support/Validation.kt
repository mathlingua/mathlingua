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

package mathlingua.frontend.support

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase1.ast.getColumn
import mathlingua.frontend.chalktalk.phase1.ast.getRow
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node

internal data class ParseError(override val message: String, val row: Int, val column: Int) :
    RuntimeException(message)

internal sealed class Validation<out T>

internal abstract class ValidationSuccess<T>(val value: T) : Validation<T>()

internal abstract class ValidationFailure<T>(val errors: List<ParseError>) : Validation<T>()

internal data class ValidationSuccessImpl<T>(val v: T) : ValidationSuccess<T>(v)

internal data class ValidationFailureImpl<T>(val errs: List<ParseError>) :
    ValidationFailure<T>(errs)

internal fun <T> validationSuccess(value: T): ValidationSuccess<T> = ValidationSuccessImpl(value)

internal fun <T : Phase2Node> validationSuccess(
    tracker: MutableLocationTracker, phase1Node: Phase1Node, phase2Node: T
): ValidationSuccess<T> {
    tracker.setLocationOf(
        phase2Node, Location(row = getRow(phase1Node), column = getColumn(phase1Node)))
    return ValidationSuccessImpl(phase2Node)
}

internal fun <T> validationFailure(errors: List<ParseError>): ValidationFailure<T> =
    ValidationFailureImpl(errors)

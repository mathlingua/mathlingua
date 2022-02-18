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

internal data class ParseError(override val message: String, val row: Int, val column: Int) :
    RuntimeException(message)

internal sealed class Validation<out T>

internal data class ValidationSuccess<T>(val value: T) : Validation<T>()

internal data class ValidationFailure<T>(val errors: List<ParseError>) : Validation<T>()

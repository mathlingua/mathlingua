/*
 * Copyright 2020 The MathLingua Authors
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

package mathlingua.mathlingua

import assertk.assertThat
import assertk.assertions.isEqualTo
import assertk.assertions.isInstanceOf
import mathlingua.frontend.FrontEnd
import mathlingua.frontend.support.ValidationFailure
import mathlingua.loadMessageTestCases
import org.junit.jupiter.api.DynamicTest
import org.junit.jupiter.api.TestFactory

internal class MessageTest {
    @TestFactory
    fun `Expected Messages Tests`(): Collection<DynamicTest> {
        val testCases = loadMessageTestCases()

        return testCases.map {
            DynamicTest.dynamicTest("Reports errors: ${it.name}") {
                val validation = FrontEnd.parse(it.input)
                assertThat(validation).isInstanceOf(ValidationFailure::class.java)
                val failure = validation as ValidationFailure
                assertThat(failure.errors).isEqualTo(it.expectedErrors)
            }
        }
    }
}

/*
 * Copyright 2022 Dominic Kramer
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

package mathlingua.lib.frontend.ast

import kotlin.test.Test
import mathlingua.lib.frontend.MetaData
import strikt.api.expect
import strikt.assertions.isEqualTo

internal class CommonAstTest {
    @Test
    fun `correctly renders items to code`() {
        val metadata = MetaData(row = -1, column = -1, isInline = false)
        expect {
            that(Statement(text = "some' statement", metadata = metadata).toCode())
                .isEqualTo("`some' statement`")
            that(Statement(text = "some` statement", metadata = metadata).toCode())
                .isEqualTo("'some` statement'")
            that(Statement(text = "some statement", metadata = metadata).toCode())
                .isEqualTo("'some statement'")
        }
    }
}

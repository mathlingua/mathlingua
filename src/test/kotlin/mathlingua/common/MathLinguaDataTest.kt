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

import assertk.assertThat
import assertk.assertions.isEqualTo
import mathlingua.jvm.MATHLINGUA_SOURCE_FILE
import org.junit.jupiter.api.Test
import java.nio.file.Files

internal class MathLinguaDataTest {
    @Test
    fun `mathlingua text file is valid`() {
        val input = Files.readString(MATHLINGUA_SOURCE_FILE.toPath())
        val result = MathLingua().parse(input)
        val builder = StringBuilder()
        if (result is ValidationFailure) {
            for (err in result.errors) {
                builder.append(
                    "ERROR: (${err.row + 1}, ${err.column + 1}) ${err.message}\n"
                )
            }
        }
        // The test should fail if there any errors.
        // This assertThat() is used so that the parse
        // errors are printed to the console.
        assertThat(builder.toString()).isEqualTo("")
        assert(result is ValidationSuccess)
    }
}

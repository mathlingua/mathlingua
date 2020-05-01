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

package mathlingua.common.transform

import assertk.assertThat
import assertk.assertions.isEqualTo
import mathlingua.common.MathLingua
import mathlingua.common.ValidationSuccess
import org.junit.jupiter.api.Test

internal class SignatureUtilKtTest {
    @Test
    fun findAllStatementSignaturesNonGluedTest() {
        val validation = MathLingua.parse("[\\xyz{x}]\nDefines: y\nmeans: 'something'")
        val doc = (validation as ValidationSuccess).value
        assertThat(doc.defines.size).isEqualTo(1)
        val def = doc.defines[0]
        val stmt = def.id.toStatement()
        val signatures = findAllStatementSignatures(stmt)
        assertThat(signatures).isEqualTo(setOf("\\xyz{}"))
    }

    @Test
    fun findAllStatementSignaturesGluedTest() {
        val validation = MathLingua.parse("[\\abc \\xyz{x}]\nDefines: y\nmeans: 'something'")
        val doc = (validation as ValidationSuccess).value
        assertThat(doc.defines.size).isEqualTo(1)
        val def = doc.defines[0]
        val stmt = def.id.toStatement()
        val signatures = findAllStatementSignatures(stmt)
        assertThat(signatures).isEqualTo(setOf("\\abc.xyz{}"))
    }

    @Test
    fun findAllStatementSignaturesInfixTest() {
        val validation = MathLingua.parse("[x \\abc y]\nDefines: y\nmeans: 'something'")
        val doc = (validation as ValidationSuccess).value
        assertThat(doc.defines.size).isEqualTo(1)
        val def = doc.defines[0]
        val stmt = def.id.toStatement()
        val signatures = findAllStatementSignatures(stmt)
        assertThat(signatures).isEqualTo(setOf("\\abc"))
    }
}

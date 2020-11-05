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

package mathlingua.mathlingua.transform

import assertk.assertThat
import assertk.assertions.isEqualTo
import mathlingua.MathLingua
import mathlingua.Signature
import mathlingua.support.Location
import mathlingua.support.ParseError
import mathlingua.support.ValidationFailure
import mathlingua.support.ValidationSuccess
import mathlingua.support.newLocationTracker
import mathlingua.transform.findAllStatementSignatures
import org.junit.jupiter.api.Test

internal class SignatureUtilKtTest {
    @Test
    fun findAllStatementSignaturesNonGluedTest() {
        val validation = MathLingua.parse("[\\xyz{x}]\nDefines: y\nmeans: 'something'")
        val doc = (validation as ValidationSuccess).value
        val defines = doc.defines()
        assertThat(defines.size).isEqualTo(1)
        val def = defines[0]
        val stmt = def.id.toStatement()
        val signatures = findAllStatementSignatures(stmt, newLocationTracker())
        assertThat(signatures)
            .isEqualTo(setOf(Signature(form = "\\xyz", location = Location(row = -1, column = -1))))
    }

    @Test
    fun statementSignaturesNotAllowedToBeGluedTest() {
        val validation = MathLingua.parse("[\\abc \\xyz{x}]\nDefines: y\nmeans: 'something'")
        val doc = (validation as ValidationSuccess).value
        val defines = doc.defines()
        assertThat(defines.size).isEqualTo(1)
        val def = defines[0]
        val texTalkValidation = def.id.texTalkRoot
        assertThat(texTalkValidation is ValidationFailure)
        val failure = texTalkValidation as ValidationFailure
        assertThat(failure.errors)
            .isEqualTo(
                listOf(
                    ParseError(
                        message =
                            "Multiple infix operators cannot be side by side ('\\abc \\xyz{x}').  They " +
                                "can only be one of the forms: '\\x \\op \\y', '\\x \\op y', 'x \\op \\y', " +
                                "or 'x \\op y'",
                        row = -1,
                        column = -1)))
    }

    @Test
    fun findAllStatementSignaturesInfixTest() {
        val validation = MathLingua.parse("[x \\abc y]\nDefines: y\nmeans: 'something'")
        val doc = (validation as ValidationSuccess).value
        val defines = doc.defines()
        assertThat(defines.size).isEqualTo(1)
        val def = defines[0]
        val stmt = def.id.toStatement()
        val signatures = findAllStatementSignatures(stmt, newLocationTracker())
        assertThat(signatures)
            .isEqualTo(setOf(Signature(form = "\\abc", location = Location(row = -1, column = -1))))
    }
}

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
import mathlingua.backend.transform.Signature
import mathlingua.backend.transform.findAllStatementSignatures
import mathlingua.frontend.FrontEnd
import mathlingua.frontend.support.Location
import mathlingua.frontend.support.ParseError
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.support.newLocationTracker
import org.junit.jupiter.api.Test

internal class SignatureUtilKtTest {
    @Test
    fun findAllStatementSignaturesNonGluedTest() {
        val validation =
            FrontEnd.parse(
                "[\\xyz{x}]\nDefines: y\nwhere: 'something'\nmeans: 'something'\nwritten: \"something\"")
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
        val validation =
            FrontEnd.parse(
                "[\\abc \\xyz{x}]\nDefines: y\nwhere: 'something'\nmeans: 'something'\n" +
                    "written: \"something\"")
        assertThat(validation is ValidationFailure)
        assertThat((validation as ValidationFailure).errors)
            .isEqualTo(
                listOf(
                    ParseError(
                        message = "A command in an id cannot be of the form \\x \\y ...",
                        row = 0,
                        column = 0)))
    }

    @Test
    fun findAllStatementSignaturesInfixTest() {
        val validation =
            FrontEnd.parse(
                "[x \\abc/ y]\nDefines: y\nwhere: 'something'\nmeans: 'something'\n" +
                    "written: \"something\"")
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

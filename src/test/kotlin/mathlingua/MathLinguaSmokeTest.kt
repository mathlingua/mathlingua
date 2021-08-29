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

package mathlingua.mathlingua

import assertk.assertThat
import assertk.assertions.isEqualTo
import mathlingua.frontend.FrontEnd
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.textalk.newTexTalkLexer
import mathlingua.frontend.textalk.newTexTalkParser
import mathlingua.newDiskFileSystem
import org.junit.jupiter.api.Test

internal class MathLinguaDataTest {
    @Test
    fun `input MathLingua file is valid`() {
        val fs = newDiskFileSystem()
        val mathlinguaSourceFile = fs.getFile(listOf("src", "test", "resources", "mathlingua.math"))
        val input = mathlinguaSourceFile.readText()
        val result = FrontEnd.parse(input)
        val builder = StringBuilder()
        if (result is ValidationFailure) {
            for (err in result.errors) {
                builder.append("ERROR: (${err.row + 1}, ${err.column + 1}) ${err.message}\n")
            }
        }
        // The test should fail if there any errors.
        // This assertThat() is used so that the parse
        // errors are printed to the console.
        assertThat(builder.toString()).isEqualTo("")
        assert(result is ValidationSuccess)
    }

    @Test
    fun `input TexTalk file is valid`() {
        val fs = newDiskFileSystem()
        val sourceFile = fs.getFile(listOf("src", "test", "resources", "textalk.txt"))
        val lines = sourceFile.readText().lines()
        val builder = StringBuilder()
        val parser = newTexTalkParser()
        val token = ":OUTPUT:"
        for (lineIndex in lines.indices) {
            val line = lines[lineIndex]
            val index = line.indexOf(token)
            val prefix =
                if (index < 0) {
                        line
                    } else {
                        line.substring(0, index)
                    }
                    .trim()
            val expectedOutput =
                if (index < 0) {
                        line
                    } else {
                        line.substring(index + token.length).trim()
                    }
                    .trim()
            val lexer = newTexTalkLexer(prefix)
            val result = parser.parse(lexer)
            for (err in result.errors) {
                builder.append("ERROR: (${err.row + 1}, ${err.column + 1}) ${err.message}\n")
            }
            assertThat("Line ${lineIndex + 1}: ${result.root.toCode()}")
                .isEqualTo("Line ${lineIndex + 1}: $expectedOutput")
        }
        // The test should fail if there any errors.
        // This assertThat() is used so that the parse
        // errors are printed to the console.
        assertThat(builder.toString()).isEqualTo("")
    }
}

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

package mathlingua.lib

import java.io.File
import java.nio.file.Paths
import mathlingua.lib.backend.getDocsDir
import mathlingua.lib.backend.newWorkspace
import mathlingua.lib.frontend.DiagnosticType

private const val VERSION = "0.16"

private const val CNAME_FILE_NAME = "CNAME"

data class FileDiagnostic(
    val file: File, val type: DiagnosticType, val message: String, val row: Int, val column: Int)

interface Mlg {
    fun check(files: List<File>): List<FileDiagnostic>
    fun edit(noOpen: Boolean, port: Int)
    fun doc()
    fun clean()
    fun version(): String
}

fun newMlg(): Mlg = MlgImpl

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private object MlgImpl : Mlg {
    override fun check(files: List<File>): List<FileDiagnostic> = newWorkspace(cwd()).check()

    override fun edit(noOpen: Boolean, port: Int) {
        TODO("Not yet implemented")
        // starts a server that allows the user to interact with their
        // mathlingua code
    }

    override fun doc() = newWorkspace(cwd()).doc()

    override fun clean() {
        val cwd = cwd()

        val docsDir = getDocsDir(cwd)
        docsDir.deleteRecursively()
        docsDir.mkdirs()

        val cnameSource = File(cwd, CNAME_FILE_NAME)
        if (cnameSource.exists()) {
            val cnameDest = File(docsDir, CNAME_FILE_NAME)
            cnameDest.writeText(cnameSource.readText())
        }
    }

    override fun version(): String = VERSION
}

private fun cwd() = Paths.get(".").toAbsolutePath().normalize().toFile()

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

package mathlingua.lib.backend

import java.io.File
import java.nio.file.Paths
import mathlingua.lib.FileDiagnostic
import mathlingua.lib.frontend.Diagnostic
import mathlingua.lib.frontend.Frontend
import mathlingua.lib.frontend.ast.Document

internal interface Workspace {
    fun include(file: File)
    fun exclude(file: File)

    fun read(file: File): String
    fun write(file: File, content: String)
    fun delete(file: File)
    fun move(from: File, to: File)

    fun check(): List<FileDiagnostic>
    fun doc()
}

internal fun newWorkspace(baseDir: File): Workspace = WorkspaceImpl(baseDir)

internal fun getDocsDir(baseDir: File) = File(baseDir, "docs")

internal fun getContentDir(baseDir: File) = File(baseDir, "content")

internal fun cwd() = Paths.get(".").toAbsolutePath().normalize().toFile()

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private data class WorkspaceItem(
    val file: File, val doc: Document, val diagnostics: List<Diagnostic>)

private data class WorkspaceImpl(private val baseDir: File) : Workspace {
    private val items = mutableMapOf<File, WorkspaceItem>()

    override fun include(file: File) {
        val result = Frontend.parse(file.readText())
        items[file] = WorkspaceItem(file = file, doc = result.doc, diagnostics = result.diagnostics)
    }

    override fun exclude(file: File) {
        items.remove(file)
    }

    override fun read(file: File): String = file.readText()

    override fun write(file: File, content: String) {
        exclude(file)
        file.writeText(content)
        include(file)
    }

    override fun delete(file: File) {
        exclude(file)
        file.delete()
    }

    override fun move(from: File, to: File) {
        val text = from.readText()
        exclude(from)
        to.writeText(text)
        include(to)
    }

    override fun check(): List<FileDiagnostic> {
        val result = mutableListOf<FileDiagnostic>()
        for ((file, item) in items) {
            for (diag in item.diagnostics) {
                result.add(
                    FileDiagnostic(
                        file = file,
                        type = diag.type,
                        message = diag.message,
                        row = diag.row,
                        column = diag.column))
            }
        }
        return result
    }

    override fun doc() {
        println("Generating docs")
    }
}

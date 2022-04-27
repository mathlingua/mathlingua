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
import mathlingua.lib.FileDiagnostic

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

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private data class WorkspaceImpl(private val baseDir: File) : Workspace {
    override fun include(file: File) {
        TODO("Not yet implemented")
    }

    override fun exclude(file: File) {
        println("Excluding file: $file")
    }

    override fun read(file: File): String {
        println("Reading file: $file")
        return ""
    }

    override fun write(file: File, content: String) {
        println("Writing file: $file")
    }

    override fun delete(file: File) {
        println("Deleting file: $file")
    }

    override fun move(from: File, to: File) {
        println("Moving file $from to $to")
    }

    override fun check(): List<FileDiagnostic> {
        println("Checking")
        return emptyList()
    }

    override fun doc() {
        println("Generating docs")
    }
}

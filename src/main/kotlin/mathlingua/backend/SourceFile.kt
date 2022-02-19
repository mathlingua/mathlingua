/*
 * Copyright 2022 The MathLingua Authors
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

package mathlingua.backend

import mathlingua.cli.ErrorResult
import mathlingua.cli.FileResult
import mathlingua.cli.VirtualFile
import mathlingua.frontend.FrontEnd
import mathlingua.frontend.chalktalk.phase2.ast.Document
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.toEntityResult
import mathlingua.frontend.support.Validation
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess

internal data class SourceFile(
    val file: VirtualFile, val content: String, val validation: Validation<Document>)

internal fun SourceFile.toFileResult(
    nextRelativePath: String?, previousRelativePath: String?, sourceCollection: SourceCollection
): FileResult {
    val relativePath = this.file.relativePath()
    return FileResult(
        relativePath = relativePath,
        nextRelativePath = nextRelativePath,
        previousRelativePath = previousRelativePath,
        content = this.content,
        entities =
            when (val validation = this.validation
            ) {
                is ValidationSuccess -> {
                    val doc = validation.value
                    doc.groups.map { it.toEntityResult(relativePath, sourceCollection) }
                }
                else -> {
                    emptyList()
                }
            },
        errors =
            when (val validation = this.validation
            ) {
                is ValidationFailure -> {
                    validation.errors.map {
                        ErrorResult(
                            relativePath = relativePath,
                            message = it.message,
                            row = it.row,
                            column = it.column)
                    }
                }
                else -> {
                    emptyList()
                }
            })
}

internal fun VirtualFile.isMathLinguaFile() =
    !this.isDirectory() && this.absolutePath().endsWith(".math")

internal fun VirtualFile.buildSourceFile(): SourceFile {
    val content = this.readText()
    return SourceFile(file = this, content = content, validation = FrontEnd.parse(content))
}

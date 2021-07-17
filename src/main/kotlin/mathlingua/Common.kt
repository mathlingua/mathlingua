/*
 * Copyright 2021
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

package mathlingua

import com.soywiz.korio.file.baseName
import com.soywiz.korio.file.folderWithSlash
import com.soywiz.korio.file.fullPathNormalized
import com.soywiz.korio.file.parts
import com.soywiz.korio.file.std.localCurrentDirVfs
import com.soywiz.korio.file.std.localVfs
import io.ktor.application.call
import io.ktor.http.ContentType
import io.ktor.response.respondText
import io.ktor.routing.get
import io.ktor.routing.routing
import io.ktor.server.engine.embeddedServer
import io.ktor.server.netty.Netty
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.runBlocking
import mathlingua.cli.Logger
import mathlingua.cli.VirtualFile
import mathlingua.cli.VirtualFileImpl
import mathlingua.cli.VirtualFileSystem

var value: Long = 0

fun getProcessWiseUniqueID() = "PROCESSWISE_UNIQUE_ID_${value++}"

class Stack<T> {
    private val data = mutableListOf<T>()

    fun push(value: T) = data.add(value)

    fun peek(): T = data.last()

    fun pop(): T {
        val peek = peek()
        data.removeLast()
        return peek
    }

    fun isEmpty() = data.isEmpty()
}

class Queue<T> : Iterable<T> {
    private val data = mutableListOf<T>()

    fun offer(value: T) = data.add(value)

    fun peek(): T = data.first()

    fun poll(): T {
        val peek = peek()
        data.removeFirst()
        return peek
    }

    fun isEmpty() = data.isEmpty()

    override fun iterator(): Iterator<T> = data.iterator()
}

fun startServer(port: Int, logger: Logger, processor: () -> Pair<String, String>) {
    embeddedServer(Netty, port = port, host = "0.0.0.0") {
            routing {
                get("/") {
                    logger.log("Rendering...")
                    val start = System.currentTimeMillis()
                    val pair = processor()
                    val end = System.currentTimeMillis()
                    logger.log("Completed in ${end - start} ms")
                    call.respondText(text = pair.first, contentType = ContentType.Text.Html)
                    logger.log(pair.second)
                    logger.log("")
                }
            }
        }
        .start(wait = true)
}

fun newDiskFileSystem(): VirtualFileSystem {
    return DiskFileSystem()
}

private class DiskFileSystem : VirtualFileSystem {
    private val cwd = localCurrentDirVfs.fullPathNormalized.split(getFileSeparator())
    private val cwdFile = VirtualFileImpl(absolutePathParts = cwd, directory = true, this)

    private fun getAbsolutePath(relativePath: List<String>): List<String> {
        val absolutePath = mutableListOf<String>()
        absolutePath.addAll(cwd)
        absolutePath.addAll(relativePath)
        return absolutePath
    }

    override fun getFileSeparator() = localCurrentDirVfs.folderWithSlash.last().toString()

    override fun getFileOrDirectory(path: String): VirtualFile {
        val file = localVfs(cwd.plus(path).joinToString(getFileSeparator()))
        val cwdFile = localVfs(cwd.joinToString(getFileSeparator()))
        val relPath =
            relativePathTo(file.absolutePathInfo.parts(), cwdFile.absolutePathInfo.parts())
        val isDir = runBlocking { file.isDirectory() }
        return if (isDir) {
            getDirectory(relPath)
        } else {
            getFile(relPath)
        }
    }

    override fun getFile(relativePath: List<String>): VirtualFile {
        return VirtualFileImpl(
            absolutePathParts = getAbsolutePath(relativePath), directory = false, fs = this)
    }

    override fun getDirectory(relativePath: List<String>): VirtualFile {
        return VirtualFileImpl(
            absolutePathParts = getAbsolutePath(relativePath), directory = true, fs = this)
    }

    override fun cwd() = cwdFile

    private fun VirtualFile.toFile() = localVfs(absolutePath().joinToString(getFileSeparator()))

    override fun relativePathTo(vf: VirtualFile, dir: VirtualFile): List<String> {
        return relativePathTo(vf.absolutePath(), dir.absolutePath())
    }

    private fun relativePathTo(fileParts: List<String>, dirParts: List<String>): List<String> {
        val fileStack = Stack<String>()
        val dirStack = Stack<String>()

        for (item in fileParts.reversed()) {
            fileStack.push(item)
        }

        for (item in dirParts.reversed()) {
            dirStack.push(item)
        }

        while (!fileStack.isEmpty() && !dirStack.isEmpty()) {
            if (fileStack.peek() == dirStack.peek()) {
                fileStack.pop()
                dirStack.pop()
            } else {
                break
            }
        }

        val relativePath = mutableListOf<String>()
        while (!fileStack.isEmpty()) {
            relativePath.add(fileStack.pop())
        }

        return relativePath
    }

    override fun exists(vf: VirtualFile) = runBlocking { vf.toFile().exists() }

    override fun mkdirs(vf: VirtualFile) = runBlocking { vf.toFile().mkdir() }

    override fun readText(vf: VirtualFile) = runBlocking { vf.toFile().readString() }

    override fun writeText(vf: VirtualFile, content: String) =
        runBlocking { vf.toFile().writeString(content) }

    override fun listFiles(vf: VirtualFile): List<VirtualFile> {
        val children = runBlocking { vf.toFile().list().toList() }
        return children.sortedBy { it.baseName }.map {
            VirtualFileImpl(
                absolutePathParts = it.absolutePath.split(getFileSeparator()),
                directory = runBlocking { it.isDirectory() },
                this)
        }
    }

    override fun delete(vf: VirtualFile): Boolean {
        val file = vf.toFile()
        return runBlocking { file.delete() }
    }
}

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

import java.io.BufferedReader
import java.io.File
import java.io.InputStreamReader
import java.net.ServerSocket
import java.net.Socket
import java.net.SocketException
import java.nio.file.Paths
import java.security.MessageDigest
import java.util.LinkedList
import java.util.Stack
import java.util.UUID
import java.util.concurrent.Executors
import mathlingua.cli.Logger
import mathlingua.cli.VirtualFile
import mathlingua.cli.VirtualFileImpl
import mathlingua.cli.VirtualFileSystem

/**
 * Any code that use a class in a `java.*` package should be placed in this file. This is done to
 * consolidate JVM specific code into a single place.
 */

/** Used to get a random UUID value. */
fun getRandomUuid() = UUID.randomUUID().toString()

fun md5Hash(input: String) =
    MessageDigest.getInstance("MD5").digest(input.toByteArray()).joinToString("") {
        Integer.toHexString(it.toInt().and(0xFF)).uppercase()
    }

/**
 * The Stack<T> class, backed by a [java.util.Stack] hides the usage of the JVM specific Stack so
 * that the rest of the code can utilize a Stack without directly using a java.util.Stack.
 */
class Stack<T> {
    private val data = Stack<T>()

    fun push(value: T) {
        data.push(value)
    }

    fun peek(): T = data.peek()

    fun pop(): T = data.pop()

    fun isEmpty() = data.isEmpty()
}

/**
 * The Queue<T> class, backed by a [java.util.Queue] hides the usage of the JVM specific Queue so
 * that the rest of the code can utilize a Queue without directly using a java.util.Queue.
 */
class Queue<T> : Iterable<T> {
    private val data = LinkedList<T>()

    fun offer(value: T) {
        data.offer(value)
    }

    fun peek(): T = data.peek()

    fun poll(): T = data.poll()

    fun isEmpty() = data.isEmpty()

    override fun iterator(): Iterator<T> = data.iterator()
}

fun startServer(port: Int, logger: Logger, processor: () -> Pair<String, String>) {
    val serverSocket = ServerSocket(port)
    val service = Executors.newCachedThreadPool()
    Runtime.getRuntime().addShutdownHook(Thread { service.shutdown() })
    while (true) {
        val client = serverSocket.accept()
        service.submit { handleServeRequest(logger, client, processor) }
    }
}

// processor returns (HTML output, console output)
private fun handleServeRequest(
    logger: Logger, client: Socket, processor: () -> Pair<String, String>
) {
    try {
        val reader = BufferedReader(InputStreamReader(client.getInputStream()))
        // The first line of the request is of the form
        //   <method> <path> <http-version>
        // for example.
        //   GET / HTTP/1.1
        val firstLine = reader.readLine() ?: ""
        val parts = firstLine.split(" ")
        val method =
            if (parts.isEmpty()) {
                null
            } else {
                parts[0]
            }
        val urlWithoutParams =
            if (parts.size < 2) {
                null
            } else {
                val url = parts[1]
                val index = url.indexOf("?")
                if (index < 0) {
                    url
                } else {
                    url.substring(0, index)
                }
            }
        if (method != "GET" || urlWithoutParams != "/") {
            val output = client.getOutputStream()
            output.write("HTTP/1.1 404 Not Found\r\n".toByteArray())
            output.write("ContentType: text/html\r\n\r\n".toByteArray())
            output.write("Not Found".toByteArray())
            output.write("\r\n\r\n".toByteArray())
            output.flush()
            output.close()
        } else {
            val start = System.currentTimeMillis()
            val pair = processor()
            val end = System.currentTimeMillis()
            try {
                val output = client.getOutputStream()
                output.write("HTTP/1.1 200 OK\r\n".toByteArray())
                output.write("ContentType: text/html\r\n\r\n".toByteArray())
                output.write(pair.first.toByteArray())
                output.write("\r\n\r\n".toByteArray())
                output.flush()
                output.close()
            } finally {
                logger.log("Completed in ${end - start} ms")
                logger.log(pair.second)
            }
        }
    } catch (e: SocketException) {
        // It appears to be common for a SocketException with message
        // "Broken pipe (Write failed)" if a request comes in while another
        // is still being processed. Just absorb the exception and try to
        // reconnect.
    }
}

fun newDiskFileSystem(): VirtualFileSystem {
    return DiskFileSystem()
}

private class DiskFileSystem() : VirtualFileSystem {
    private val cwd =
        Paths.get(".").toAbsolutePath().normalize().toFile().absolutePath.split(getFileSeparator())
    private val cwdFile = VirtualFileImpl(absolutePathParts = cwd, directory = true, this)

    private fun getAbsolutePath(relativePath: List<String>): List<String> {
        val absolutePath = mutableListOf<String>()
        absolutePath.addAll(cwd)
        absolutePath.addAll(relativePath)
        return absolutePath
    }

    override fun getFileSeparator() = File.separator

    override fun getFileOrDirectory(path: String): VirtualFile {
        val file = File(path).normalize().absoluteFile
        val cwdFile = File(cwd.joinToString(getFileSeparator()))
        val relPath = file.toRelativeString(cwdFile).split(getFileSeparator())
        return if (file.isDirectory) {
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

    private fun VirtualFile.toFile() =
        File(this.absolutePath().joinToString(File.separator)).normalize()

    override fun relativePathTo(vf: VirtualFile, dir: VirtualFile) =
        vf.toFile().toRelativeString(dir.toFile()).split(File.separator)

    override fun exists(vf: VirtualFile) = vf.toFile().exists()

    override fun mkdirs(vf: VirtualFile) = vf.toFile().mkdirs()

    override fun readText(vf: VirtualFile) = vf.toFile().readText()

    override fun writeText(vf: VirtualFile, content: String) = vf.toFile().writeText(content)

    override fun listFiles(vf: VirtualFile): List<VirtualFile> {
        val children = vf.toFile().listFiles() ?: arrayOf()
        return children.sortedBy { it.name }.map {
            VirtualFileImpl(
                absolutePathParts = it.absolutePath.split(File.separator),
                directory = it.isDirectory,
                this)
        }
    }

    override fun delete(vf: VirtualFile): Boolean {
        val file = vf.toFile()
        return if (file.isDirectory) {
            file.deleteRecursively()
        } else {
            file.delete()
        }
    }
}

class MutableMultiSet<T> {
    private val data = mutableMapOf<T, Int>()

    fun add(value: T) {
        data[value] = 1 + data.getOrDefault(value, 0)
    }

    fun addAll(values: Collection<T>) {
        for (v in values) {
            add(v)
        }
    }

    fun remove(value: T) {
        if (data.containsKey(value)) {
            val newCount = data[value]!! - 1
            data[value] = newCount
            if (newCount <= 0) {
                data.remove(value)
            }
        }
    }

    fun contains(key: T) = data.getOrDefault(key, 0) > 0

    fun toList() = data.keys.toList()

    fun isEmpty() = data.isEmpty()

    fun toSet() = data.keys.toSet()

    companion object {
        fun <T> copy(set: MutableMultiSet<T>): MutableMultiSet<T> {
            val copy = MutableMultiSet<T>()
            for (entry in set.data.entries) {
                copy.data[entry.key] = entry.value
            }
            return copy
        }
    }

    override fun toString() = data.toString()
}

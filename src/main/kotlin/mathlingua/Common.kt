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
import java.util.UUID
import java.util.concurrent.Executors
import mathlingua.cli.Logger

fun getCwd() = Paths.get(".").toAbsolutePath().normalize().toFile()

fun getFileSeparator() = File.separator

fun getRandomUuid() = UUID.randomUUID().toString()

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
            logger.log("Rendering...")
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
                logger.log("")
            }
        }
    } catch (e: SocketException) {
        // It appears to be common for a SocketException with message
        // "Broken pipe (Write failed)" if a request comes in while another
        // is still being processed. Just absorb the exception and try to
        // reconnect.
    }
}

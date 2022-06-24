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

import io.ktor.application.call
import io.ktor.http.HttpStatusCode
import io.ktor.response.respond
import io.ktor.routing.get
import io.ktor.routing.routing
import io.ktor.server.cio.CIO
import io.ktor.server.engine.embeddedServer
import kotlinx.serialization.json.Json
import mathlingua.lib.backend.cwd
import mathlingua.lib.backend.findFilesWithExtension
import mathlingua.lib.backend.getContentDir
import mathlingua.lib.backend.newWorkspace

internal fun startServer(noOpen: Boolean, port: Int) {
    val server =
        embeddedServer(CIO, port = port, host = "0.0.0.0") {
            routing {
                get("/api/check") {
                    try {
                        val cwd = cwd()
                        val workspace = newWorkspace(cwd)
                        getContentDir(cwd).findFilesWithExtension(".math").forEach {
                            workspace.include(it)
                        }
                        val result = workspace.check()
                        call.respond(
                            Json.encodeToString(
                                CheckResponse.serializer(), CheckResponse(diagnostics = result)))
                    } catch (e: Exception) {
                        e.printStackTrace()
                        call.respond(HttpStatusCode.InternalServerError)
                    }
                }
            }
        }
    if (!noOpen) {
        openBrowser(port)
    }
    server.start(wait = true)
}

private fun openBrowser(port: Int) {
    val url = "http://localhost:${port}"
    try {
        val osName = System.getProperty("os.name").lowercase()
        if (osName.contains("win")) {
            ProcessBuilder("rundll32", "url.dll,FileProtocolHandler", url).start()
        } else if (osName.contains("mac")) {
            ProcessBuilder("open", url).start()
        } else {
            ProcessBuilder("xdg-open", url).start()
        }
    } catch (e: Exception) {
        println("Failed to automatically open $url")
        if (e.message != null) {
            println(e.message)
        }
    }
}

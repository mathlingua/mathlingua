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

package mathlingua.cli

import io.ktor.application.call
import io.ktor.http.HttpStatusCode
import io.ktor.http.content.files
import io.ktor.http.content.static
import io.ktor.request.receiveText
import io.ktor.response.respond
import io.ktor.response.respondFile
import io.ktor.routing.get
import io.ktor.routing.post
import io.ktor.routing.put
import io.ktor.routing.routing
import io.ktor.server.cio.CIO
import io.ktor.server.engine.embeddedServer
import java.io.File
import java.io.FileNotFoundException
import java.io.InputStream
import java.nio.file.Files
import java.nio.file.Paths
import java.util.zip.ZipFile
import kotlin.io.path.absolutePathString
import kotlin.io.path.writeBytes
import kotlin.system.exitProcess
import kotlinx.serialization.json.Json
import mathlingua.backend.BackEnd
import mathlingua.backend.SourceCollection
import mathlingua.backend.buildSourceFile
import mathlingua.backend.newSourceCollectionFromCwd

internal fun ensureDocsDirExists(fs: VirtualFileSystem, logger: Logger): File {
    val classLoader = ClassLoader.getSystemClassLoader()
    val checksumStream =
        classLoader.getResourceAsStream("checksum")
            ?: throw Exception("Failed to load static document assets checksum")

    val docDir = File(getDocsDirectory(fs).absolutePath())
    docDir.mkdirs()

    val docsChecksumFile = File(docDir, "checksum")
    val actualChecksum =
        try {
            docsChecksumFile.readText()
        } catch (e: FileNotFoundException) {
            ""
        }
    val expectedChecksum = String(checksumStream.readAllBytes())

    val cnameFile = File("CNAME")
    if (cnameFile.exists()) {
        val docsCnameFile = File(docDir, "CNAME")
        cnameFile.copyTo(target = docsCnameFile, overwrite = true)
    }

    if (actualChecksum != expectedChecksum) {
        logger.log("Initial run detected. Saving webapp files to speed up future runs.")

        // delete the docs directory so that no assets from any previous
        // times the docs directory was generated are left over
        if (docDir.exists()) {
            docDir.deleteRecursively()
        }

        val assetsZipStream =
            classLoader.getResourceAsStream("assets.zip")
                ?: throw Exception("Failed to load static document assets")

        val assetsZipBytes = assetsZipStream.readAllBytes()
        val tempAssetsZipFile = kotlin.io.path.createTempFile(suffix = ".zip")
        tempAssetsZipFile.writeBytes(assetsZipBytes)

        val zipPath = tempAssetsZipFile.toAbsolutePath().absolutePathString()

        val zip = ZipFile(zipPath)
        for (entry in zip.entries()) {
            if (!entry.toString().startsWith("assets/") || entry.toString() == "assets/") {
                continue
            }

            val outFile = File(docDir, entry.name.replace("assets/", ""))
            if (entry.isDirectory) {
                outFile.mkdirs()
            } else {
                val parent = outFile.parentFile
                parent?.mkdirs()
                outFile.writeBytes(readAllBytes(zip.getInputStream(entry)))
            }
        }
        zip.close()

        val indexFile = File(docDir, "index.html")
        val indexText =
            indexFile.readText().replace("<head>", "<head><script src=\"./data.js\"></script>")
        indexFile.writeText(indexText)
        logger.log("Wrote docs${File.separator}index.html")

        docsChecksumFile.writeText(expectedChecksum)
    }

    return docDir
}

private fun readAllBytes(stream: InputStream): ByteArray {
    val result = mutableListOf<Byte>()
    val tempArray = ByteArray(1024)
    while (true) {
        val numRead = stream.read(tempArray, 0, tempArray.size)
        if (numRead < 0) {
            break
        }
        for (i in 0 until numRead) {
            result.add(tempArray[i])
        }
    }
    return result.toByteArray()
}

internal fun startServer(fs: VirtualFileSystem, logger: Logger, port: Int, onStart: (() -> Unit)?) {
    logger.log("Opening http://localhost:$port for you to edit your MathLingua files.")
    logger.log("Every time you refresh the page, your MathLingua files will be re-analyzed.")

    var sourceCollection: SourceCollection? = null
    fun getSourceCollection(): SourceCollection {
        if (sourceCollection == null) {
            sourceCollection = newSourceCollectionFromCwd(fs = fs)
        }
        return sourceCollection!!
    }

    val contentDir = fs.getDirectory(listOf("content"))
    if (!fs.exists(contentDir)) {
        fs.mkdirs(contentDir)

        val welcomeFile = fs.getFile(listOf("content", "welcome.math"))
        if (!fs.exists(welcomeFile)) {
            fs.writeText(
                welcomeFile,
                """
                    ::
                    # Welcome to MathLingua
                    See [www.mathlingua.org](https://www.mathlingua.org) for more information and
                    help getting started.
                    ::
                """.trimIndent())
            // invalidate the collection so it is re-generated the next time it is requested
            sourceCollection = null
        }
    }

    val docsDir = ensureDocsDirExists(fs, logger)
    val server =
        embeddedServer(CIO, port = port, host = "0.0.0.0") {
            routing {
                static {
                    files(fs.cwd().absolutePath())
                    files(docsDir.absolutePath)
                }

                get("/") {
                    try {
                        logger.log("Re-analyzing the MathLingua code.")
                        // invalidate the source collection and regenerate it
                        sourceCollection = null
                        getSourceCollection()
                        call.respondFile(File(docsDir, "index.html"))
                    } catch (err: Exception) {
                        err.printStackTrace()
                        call.respond(HttpStatusCode.InternalServerError)
                    }
                }

                put("/api/writePage") {
                    try {
                        val pathAndContent =
                            Json.decodeFromString(WritePageRequest.serializer(), call.receiveText())
                        logger.log("Writing page ${pathAndContent.path}")

                        val file = fs.getFileOrDirectory(pathAndContent.path)
                        file.writeText(pathAndContent.content)
                        val newSource = file.buildSourceFile()
                        getSourceCollection().removeSource(pathAndContent.path)
                        getSourceCollection().addSource(newSource)
                        call.respond(HttpStatusCode.OK)
                    } catch (err: Exception) {
                        err.printStackTrace()
                        call.respond(HttpStatusCode.InternalServerError)
                    }
                }

                get("/api/readPage") {
                    try {
                        val path = call.request.queryParameters["path"]
                        logger.log("Reading page $path")
                        if (path == null) {
                            call.respond(HttpStatusCode.BadRequest)
                        } else {
                            val file = fs.getFileOrDirectory(path)
                            val content = file.readText()
                            call.respond(
                                Json.encodeToString(
                                    ReadPageResponse.serializer(),
                                    ReadPageResponse(content = content)))
                        }
                    } catch (err: Exception) {
                        err.printStackTrace()
                        call.respond(HttpStatusCode.InternalServerError)
                    }
                }

                get("/api/fileResult") {
                    try {
                        val path = call.request.queryParameters["path"]
                        logger.log("Getting file result for $path")
                        if (path == null) {
                            call.respond(HttpStatusCode.BadRequest)
                        } else {
                            val page = getSourceCollection().getPage(path)
                            if (page == null) {
                                call.respond(HttpStatusCode.NotFound)
                            } else {
                                call.respond(
                                    Json.encodeToString(FileResult.serializer(), page.fileResult))
                            }
                        }
                    } catch (err: Exception) {
                        err.printStackTrace()
                        call.respond(HttpStatusCode.InternalServerError)
                    }
                }

                post("/api/deleteDir") {
                    try {
                        val data =
                            Json.decodeFromString(DeleteDirRequest.serializer(), call.receiveText())
                        logger.log("Deleting directory ${data.path}")
                        Paths.get(data.path).toFile().deleteRecursively()
                        sourceCollection = null
                        getSourceCollection()
                        call.respond(HttpStatusCode.OK)
                    } catch (err: Exception) {
                        err.printStackTrace()
                        call.respond(HttpStatusCode.InternalServerError)
                    }
                }
                post("/api/deleteFile") {
                    try {
                        val data =
                            Json.decodeFromString(
                                DeleteFileRequest.serializer(), call.receiveText())
                        logger.log("Deleting file ${data.path}")
                        Paths.get(data.path).toFile().delete()
                        sourceCollection = null
                        getSourceCollection()
                        call.respond(HttpStatusCode.OK)
                    } catch (err: Exception) {
                        err.printStackTrace()
                        call.respond(HttpStatusCode.InternalServerError)
                    }
                }
                post("/api/renameDir") {
                    try {
                        val data =
                            Json.decodeFromString(RenameDirRequest.serializer(), call.receiveText())
                        logger.log("Renaming directory ${data.fromPath} to ${data.toPath}")
                        val from = Paths.get(data.fromPath)
                        val to = Paths.get(data.toPath)
                        Files.move(from, to)
                        sourceCollection = null
                        getSourceCollection()
                        call.respond(HttpStatusCode.OK)
                    } catch (err: Exception) {
                        err.printStackTrace()
                        call.respond(HttpStatusCode.InternalServerError)
                    }
                }
                post("/api/renameFile") {
                    try {
                        val data =
                            Json.decodeFromString(
                                RenameFileRequest.serializer(), call.receiveText())
                        logger.log("Renaming file ${data.fromPath} to ${data.toPath}")
                        val from = Paths.get(data.fromPath)
                        val to = Paths.get(data.toPath)
                        Files.move(from, to)
                        sourceCollection = null
                        getSourceCollection()
                        call.respond(HttpStatusCode.OK)
                    } catch (err: Exception) {
                        err.printStackTrace()
                        call.respond(HttpStatusCode.InternalServerError)
                    }
                }
                post("/api/newDir") {
                    try {
                        val data =
                            Json.decodeFromString(NewDirRequest.serializer(), call.receiveText())
                        logger.log("Creating new directory ${data.path}")
                        val dir: File = Paths.get(data.path).toFile()
                        dir.mkdirs()
                        val newFile = File(dir, "Untitled.math")
                        newFile.writeText("::\n::")
                        sourceCollection = null
                        getSourceCollection()
                        call.respond(HttpStatusCode.OK)
                    } catch (err: Exception) {
                        err.printStackTrace()
                        call.respond(HttpStatusCode.InternalServerError)
                    }
                }
                post("/api/newFile") {
                    try {
                        val data =
                            Json.decodeFromString(NewFileRequest.serializer(), call.receiveText())
                        logger.log("Creating new file ${data.path}")
                        Paths.get(data.path).toFile().writeText("::\n::")
                        sourceCollection = null
                        getSourceCollection()
                        call.respond(HttpStatusCode.OK)
                    } catch (err: Exception) {
                        err.printStackTrace()
                        call.respond(HttpStatusCode.InternalServerError)
                    }
                }
                get("/api/check") {
                    try {
                        logger.log("Checking")
                        val result =
                            CheckResponse(
                                errors =
                                    BackEnd.check(getSourceCollection()).map {
                                        CheckError(
                                            path = it.source.file.relativePath(),
                                            message = it.value.message,
                                            row = it.value.row,
                                            column = it.value.column)
                                    })
                        call.respond(Json.encodeToString(CheckResponse.serializer(), result))
                    } catch (err: Exception) {
                        err.printStackTrace()
                        call.respond(HttpStatusCode.InternalServerError)
                    }
                }
                get("/api/allPaths") {
                    try {
                        logger.log("Getting all paths")
                        val result = AllPathsResponse(paths = getSourceCollection().getAllPaths())
                        call.respond(Json.encodeToString(AllPathsResponse.serializer(), result))
                    } catch (err: Exception) {
                        err.printStackTrace()
                        call.respond(HttpStatusCode.InternalServerError)
                    }
                }
                get("/api/withSignature") {
                    try {
                        val signature = call.request.queryParameters["signature"]
                        logger.log("Getting entity with signature '${signature}'")
                        if (signature == null) {
                            call.respond(HttpStatusCode.BadRequest)
                        } else {
                            val entityResult = getSourceCollection().getWithSignature(signature)
                            if (entityResult == null) {
                                call.respond(HttpStatusCode.NotFound)
                            } else {
                                call.respond(
                                    Json.encodeToString(EntityResult.serializer(), entityResult))
                            }
                        }
                    } catch (err: Exception) {
                        err.printStackTrace()
                        call.respond(HttpStatusCode.InternalServerError)
                    }
                }
                get("/api/usedSignaturesAtRow") {
                    try {
                        val path = call.request.queryParameters["path"]
                        val row = call.request.queryParameters["row"]?.toIntOrNull()
                        logger.log("Getting used signatures for $path at row $row")
                        if (path == null || row == null) {
                            call.respond(HttpStatusCode.BadRequest)
                        } else {
                            val usedSignatures =
                                getSourceCollection().getUsedSignaturesAtRow(path, row)
                            val result =
                                UsedSignaturesAtRowResponse(
                                    signatures =
                                        usedSignatures
                                            .map {
                                                UsedSignature(
                                                    signature = it.value.form,
                                                    defPath = it.source.file.relativePath(),
                                                    defRow = it.value.location.row)
                                            }
                                            .sortedBy { it.signature })
                            call.respond(
                                Json.encodeToString(
                                    UsedSignaturesAtRowResponse.serializer(), result))
                        }
                    } catch (err: Exception) {
                        err.printStackTrace()
                        call.respond(HttpStatusCode.InternalServerError)
                    }
                }
                get("/api/search") {
                    try {
                        val query = call.request.queryParameters["query"] ?: ""
                        logger.log("Searching with query '$query'")
                        val result =
                            SearchResponse(
                                paths =
                                    getSourceCollection().search(query).map {
                                        it.file.relativePath()
                                    })
                        call.respond(Json.encodeToString(SearchResponse.serializer(), result))
                    } catch (err: Exception) {
                        err.printStackTrace()
                        call.respond(HttpStatusCode.InternalServerError)
                    }
                }
                get("/api/completeWord") {
                    try {
                        val word = call.request.queryParameters["word"] ?: ""
                        logger.log("Getting completions for word '$word'")
                        val suffixes = getSourceCollection().findWordSuffixesFor(word)
                        call.respond(
                            Json.encodeToString(
                                CompleteWordResponse.serializer(),
                                CompleteWordResponse(suffixes = suffixes)))
                    } catch (err: Exception) {
                        err.printStackTrace()
                        call.respond(HttpStatusCode.InternalServerError)
                    }
                }
                get("/api/completeSignature") {
                    try {
                        val prefix = call.request.queryParameters["prefix"] ?: ""
                        logger.log("Getting signature completions for prefix '$prefix'")
                        val suffixes = getSourceCollection().findSignaturesSuffixesFor(prefix)
                        call.respond(
                            Json.encodeToString(
                                CompleteSignatureResponse.serializer(),
                                CompleteSignatureResponse(suffixes = suffixes)))
                    } catch (err: Exception) {
                        err.printStackTrace()
                        call.respond(HttpStatusCode.InternalServerError)
                    }
                }
                get("/api/gitHubUrl") {
                    try {
                        logger.log("Getting the GitHub url")
                        call.respond(
                            Json.encodeToString(
                                GitHubUrlResponse.serializer(),
                                GitHubUrlResponse(url = getGitHubUrl())))
                    } catch (err: Exception) {
                        err.printStackTrace()
                        call.respond(HttpStatusCode.InternalServerError)
                    }
                }
                get("/api/firstPath") {
                    call.respond(
                        Json.encodeToString(
                            FirstPathResponse.serializer(),
                            FirstPathResponse(path = getSourceCollection().getFirstPath())))
                }
                get("/api/signatureIndex") {
                    call.respond(
                        Json.encodeToString(
                            SignatureIndex.serializer(),
                            buildSignatureIndex(getSourceCollection())))
                }
                get("/api/shutdown") {
                    // The /api/shutdown hook is used by the end-to-end tests to shutdown
                    // the server after the tests are done.  It is ok to expose this since
                    // the server is only designed to be run by a user on their local machine.
                    // Thus, they already have the ability to shutdown the server if they
                    // want to.
                    exitProcess(0)
                }
                get("/api/completions") {
                    call.respond(Json.encodeToString(Completions.serializer(), COMPLETIONS))
                }
                get("/api/configuration") {
                    call.respond(
                        Json.encodeToString(Configuration.serializer(), loadConfiguration()))
                }
                get("/api/{...}") { call.respond(HttpStatusCode.BadRequest) }
            }
        }

    if (onStart != null) {
        onStart()
    }
    server.start(wait = true)
}

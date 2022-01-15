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

import io.javalin.Javalin
import io.javalin.http.staticfiles.Location
import java.io.File
import java.nio.file.Files
import java.nio.file.Paths
import kotlin.system.exitProcess
import mathlingua.backend.BackEnd
import mathlingua.backend.SourceCollection
import mathlingua.backend.buildSourceFile
import mathlingua.backend.newSourceCollectionFromCwd

fun startServer(fs: VirtualFileSystem, logger: Logger, port: Int, onStart: (() -> Unit)?) {
    logger.log("Opening http://localhost:$port for you to edit your MathLingua files.")
    logger.log("Every time you refresh the page, your MathLingua files will be re-analyzed.")

    var sourceCollection: SourceCollection? = null
    fun getSourceCollection(): SourceCollection {
        if (sourceCollection == null) {
            sourceCollection = newSourceCollectionFromCwd(fs = fs)
        }
        return sourceCollection!!
    }

    if (getSourceCollection().getAllPaths().isEmpty()) {
        val contentDir = fs.getDirectory(listOf("content"))
        if (!fs.exists(contentDir)) {
            fs.mkdirs(contentDir)
        }

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

    val app = Javalin.create().start(port)
    app.config.addStaticFiles("/assets")
    app.config.addStaticFiles(fs.cwd().absolutePath(), Location.EXTERNAL)
    app.before("/") { ctx ->
        try {
            logger.log("Re-analyzing the MathLingua code.")
            // invalidate the source collection and regenerate it
            sourceCollection = null
            getSourceCollection()
        } catch (err: Exception) {
            err.printStackTrace()
            ctx.status(500)
        }
    }
    app
        .routes {}
        .put("/api/writePage") { ctx ->
            try {
                val pathAndContent = ctx.bodyAsClass(WritePageRequest::class.java)
                logger.log("Writing page ${pathAndContent.path}")

                val file = fs.getFileOrDirectory(pathAndContent.path)
                println("Writing to path ${pathAndContent.path} content:")
                println(pathAndContent.content)
                file.writeText(pathAndContent.content)
                println("Done writing to path ${pathAndContent.path}")
                val newSource = buildSourceFile(file)
                getSourceCollection().removeSource(pathAndContent.path)
                getSourceCollection().addSource(newSource)
                ctx.status(200)
            } catch (err: Exception) {
                err.printStackTrace()
                ctx.status(500)
            }
        }
        .get("/api/readPage") { ctx ->
            try {
                val path = ctx.queryParam("path", null)
                logger.log("Reading page $path")
                if (path == null) {
                    ctx.status(400)
                } else {
                    val file = fs.getFileOrDirectory(path)
                    val content = file.readText()
                    ctx.json(ReadPageResponse(content = content))
                }
            } catch (err: Exception) {
                err.printStackTrace()
                ctx.status(500)
            }
        }
        .get("/api/fileResult") { ctx ->
            try {
                val path = ctx.queryParam("path", null)
                logger.log("Getting file result for $path")
                if (path == null) {
                    ctx.status(400)
                } else {
                    val page = getSourceCollection().getPage(path)
                    if (page == null) {
                        ctx.status(404)
                    } else {
                        ctx.json(page.fileResult)
                    }
                }
            } catch (err: Exception) {
                err.printStackTrace()
                ctx.status(500)
            }
        }
        .post("/api/deleteDir") { ctx ->
            try {
                val data = ctx.bodyAsClass(DeleteDirRequest::class.java)
                logger.log("Deleting directory ${data.path}")
                Paths.get(data.path).toFile().deleteRecursively()
                sourceCollection = null
                getSourceCollection()
                ctx.status(200)
            } catch (err: Exception) {
                err.printStackTrace()
                ctx.status(500)
            }
        }
        .post("/api/deleteFile") { ctx ->
            try {
                val data = ctx.bodyAsClass(DeleteFileRequest::class.java)
                logger.log("Deleting file ${data.path}")
                Paths.get(data.path).toFile().delete()
                sourceCollection = null
                getSourceCollection()
                ctx.status(200)
            } catch (err: Exception) {
                err.printStackTrace()
                ctx.status(500)
            }
        }
        .post("/api/renameDir") { ctx ->
            try {
                val data = ctx.bodyAsClass(RenameDirRequest::class.java)
                logger.log("Renaming directory ${data.fromPath} to ${data.toPath}")
                val from = Paths.get(data.fromPath)
                val to = Paths.get(data.toPath)
                Files.move(from, to)
                sourceCollection = null
                getSourceCollection()
                ctx.status(200)
            } catch (err: Exception) {
                err.printStackTrace()
                ctx.status(500)
            }
        }
        .post("/api/renameFile") { ctx ->
            try {
                val data = ctx.bodyAsClass(RenameFileRequest::class.java)
                logger.log("Renaming file ${data.fromPath} to ${data.toPath}")
                val from = Paths.get(data.fromPath)
                val to = Paths.get(data.toPath)
                Files.move(from, to)
                sourceCollection = null
                getSourceCollection()
                ctx.status(200)
            } catch (err: Exception) {
                err.printStackTrace()
                ctx.status(500)
            }
        }
        .post("/api/newDir") { ctx ->
            try {
                val data = ctx.bodyAsClass(NewDirRequest::class.java)
                logger.log("Creating new directory ${data.path}")
                val dir = Paths.get(data.path).toFile()
                dir.mkdirs()
                val newFile = File(dir, "Untitled.math")
                newFile.writeText("::\n::")
                sourceCollection = null
                getSourceCollection()
                ctx.status(200)
            } catch (err: Exception) {
                err.printStackTrace()
                ctx.status(500)
            }
        }
        .post("/api/newFile") { ctx ->
            try {
                val data = ctx.bodyAsClass(NewFileRequest::class.java)
                logger.log("Creating new file ${data.path}")
                Paths.get(data.path).toFile().writeText("::\n::")
                sourceCollection = null
                getSourceCollection()
                ctx.status(200)
            } catch (err: Exception) {
                err.printStackTrace()
                ctx.status(500)
            }
        }
        .get("/api/check") { ctx ->
            try {
                logger.log("Checking")
                ctx.json(
                    CheckResponse(
                        errors =
                            BackEnd.check(getSourceCollection()).map {
                                CheckError(
                                    path = it.source.file.relativePath(),
                                    message = it.value.message,
                                    row = it.value.row,
                                    column = it.value.column)
                            }))
            } catch (err: Exception) {
                err.printStackTrace()
                ctx.status(500)
            }
        }
        .get("/api/allPaths") { ctx ->
            try {
                logger.log("Getting all paths")
                ctx.json(AllPathsResponse(paths = getSourceCollection().getAllPaths()))
            } catch (err: Exception) {
                err.printStackTrace()
                ctx.status(500)
            }
        }
        .get("/api/withSignature") { ctx ->
            try {
                val signature = ctx.queryParam("signature", null)
                logger.log("Getting entity with signature '${signature}'")
                if (signature == null) {
                    ctx.status(400)
                } else {
                    val entityResult = getSourceCollection().getWithSignature(signature)
                    if (entityResult == null) {
                        ctx.status(404)
                    } else {
                        ctx.json(entityResult)
                    }
                }
            } catch (err: Exception) {
                err.printStackTrace()
                ctx.status(500)
            }
        }
        .get("/api/search") { ctx ->
            try {
                val query = ctx.queryParam("query") ?: ""
                logger.log("Searching with query '$query'")
                ctx.json(
                    SearchResponse(
                        paths = getSourceCollection().search(query).map { it.file.relativePath() }))
            } catch (err: Exception) {
                err.printStackTrace()
                ctx.status(500)
            }
        }
        .get("/api/completeWord") { ctx ->
            try {
                val word = ctx.queryParam("word") ?: ""
                logger.log("Getting completions for word '$word'")
                val suffixes = getSourceCollection().findWordSuffixesFor(word)
                ctx.json(CompleteWordResponse(suffixes = suffixes))
            } catch (err: Exception) {
                err.printStackTrace()
                ctx.status(500)
            }
        }
        .get("/api/completeSignature") { ctx ->
            try {
                val prefix = ctx.queryParam("prefix") ?: ""
                logger.log("Getting signature completions for prefix '$prefix'")
                val suffixes = getSourceCollection().findSignaturesSuffixesFor(prefix)
                ctx.json(CompleteSignatureResponse(suffixes = suffixes))
            } catch (err: Exception) {
                err.printStackTrace()
                ctx.status(500)
            }
        }
        .get("/api/gitHubUrl") { ctx ->
            try {
                logger.log("Getting the GitHub url")
                ctx.json(GitHubUrlResponse(url = getGitHubUrl()))
            } catch (err: Exception) {
                err.printStackTrace()
                ctx.status(500)
            }
        }
        .get("/api/firstPath") { ctx ->
            ctx.json(FirstPathResponse(path = getSourceCollection().getFirstPath()))
        }
        .get("/api/signatureIndex") { ctx -> ctx.json(buildSignatureIndex(getSourceCollection())) }
        .get("/api/shutdown") {
            // The /api/shutdown hook is used by the end-to-end tests to shutdown
            // the server after the tests are done.  It is ok to expose this since
            // the server is only designed to be run by a user on their local machine.
            // Thus, they already have the ability to shutdown the server if they
            // want to.
            exitProcess(0)
        }
        .get("/api/completions") { ctx -> ctx.json(COMPLETIONS) }
        .get("/api/configuration") { ctx -> ctx.json(loadConfiguration()) }
        .get("/api/*") { ctx -> ctx.status(400) }

    if (onStart != null) {
        onStart()
    }
}

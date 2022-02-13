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

import kotlinx.serialization.Serializable

@Serializable data class FirstPathResponse(val path: String)

@Serializable data class DeleteDirRequest(val path: String)

@Serializable data class DeleteFileRequest(val path: String)

@Serializable data class RenameDirRequest(val fromPath: String, val toPath: String)

@Serializable data class RenameFileRequest(val fromPath: String, val toPath: String)

@Serializable data class NewDirRequest(val path: String)

@Serializable data class NewFileRequest(val path: String)

@Serializable data class ReadPageRequest(val path: String)

@Serializable data class ReadPageResponse(val content: String)

@Serializable data class WritePageRequest(val path: String, val content: String)

@Serializable data class CheckResponse(val errors: List<CheckError>)

@Serializable data class GitHubUrlResponse(val url: String?)

@Serializable
data class CheckError(val path: String, val message: String, val row: Int, val column: Int)

@Serializable data class AllPathsResponse(val paths: List<String>)

@Serializable data class HomeResponse(val homeHtml: String)

@Serializable data class SearchResponse(val paths: List<String>)

@Serializable data class CompleteWordResponse(val suffixes: List<String>)

@Serializable data class CompleteSignatureResponse(val suffixes: List<String>)

@Serializable data class SignatureIndex(val entries: List<SignatureIndexEntry>)

@Serializable
data class SignatureIndexEntry(
    val id: String, val relativePath: String, val signature: String?, val called: List<String>)

@Serializable
data class ErrorResult(
    val relativePath: String, val message: String, val row: Int, val column: Int)

@Serializable
data class CollectionResult(val fileResults: List<FileResult>, val errors: List<ErrorResult>)

@Serializable
data class DecompositionResult(
    val collectionResult: CollectionResult,
    val gitHubUrl: String?,
    val signatureIndex: SignatureIndex,
    val configuration: Configuration)

@Serializable data class CompletionItem(val name: String, val parts: List<String>)

@Serializable data class Completions(val items: List<CompletionItem>)

@Serializable
data class EntityResult(
    val id: String,
    val relativePath: String,
    val type: String,
    val signature: String?,
    val called: List<String>,
    val rawHtml: String,
    val renderedHtml: String,
    val words: List<String>)

@Serializable
data class FileResult(
    val relativePath: String,
    val nextRelativePath: String?,
    val previousRelativePath: String?,
    val content: String,
    val entities: List<EntityResult>,
    val errors: List<ErrorResult>)

@Serializable data class Configuration(val googleAnalyticsId: String?)

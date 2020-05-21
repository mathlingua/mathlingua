/*
 * Copyright 2019 Google LLC
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

package mathlingua.common

import mathlingua.common.chalktalk.phase1.newChalkTalkLexer
import mathlingua.common.chalktalk.phase1.newChalkTalkParser
import mathlingua.common.chalktalk.phase2.*
import mathlingua.common.chalktalk.phase2.ast.Document
import mathlingua.common.chalktalk.phase2.ast.Phase2Node
import mathlingua.common.chalktalk.phase2.ast.toplevel.DefinesGroup
import mathlingua.common.chalktalk.phase2.ast.toplevel.RepresentsGroup
import mathlingua.common.chalktalk.phase2.ast.toplevel.TopLevelGroup
import mathlingua.common.chalktalk.phase2.ast.validateDocument
import mathlingua.common.textalk.Command
import mathlingua.common.transform.*

data class Parse(val document: Document, val tracker: LocationTracker)

data class Signature(val form: String, val location: Location)

object MathLingua {
    fun parse(input: String): Validation<Document> =
        when (val validation = parseWithLocations(input)) {
            is ValidationSuccess -> validationSuccess(validation.value.document)
            is ValidationFailure -> validationFailure(validation.errors)
        }

    fun parseWithLocations(input: String): Validation<Parse> {
        val lexer = newChalkTalkLexer(input)

        val allErrors = mutableListOf<ParseError>()
        allErrors.addAll(lexer.errors())

        val parser = newChalkTalkParser()
        val (root, errors) = parser.parse(lexer)
        allErrors.addAll(errors)

        if (root == null || allErrors.isNotEmpty()) {
            return validationFailure(allErrors)
        }

        val tracker = newLocationTracker()
        return when (val documentValidation = validateDocument(root, tracker)) {
            is ValidationSuccess -> validationSuccess(Parse(
                    document = documentValidation.value,
                    tracker = tracker
            ))
            is ValidationFailure -> {
                allErrors.addAll(documentValidation.errors)
                validationFailure(allErrors)
            }
        }
    }

    fun justify(text: String, width: Int) = mathlingua.common.justify(text, width)

    fun prettyPrintIdentifier(text: String) = mathlingua.common.chalktalk.phase2.prettyPrintIdentifier(text)

    fun signatureOf(group: TopLevelGroup) = getSignature(group)

    fun signatureOf(command: Command) = getCommandSignature(command)

    fun findAllSignatures(node: Phase2Node, locationTracker: LocationTracker) = locateAllSignatures(node, locationTracker).toList()

    fun flattenSignature(signature: String) = mathlingua.common.transform.flattenSignature(signature)

    fun findAllCommands(node: Phase2Node) = locateAllCommands(node).toList()

    fun findUndefinedSignatures(input: String, supplemental: List<String>): List<Signature> {
        val definedSignatures = mutableSetOf<String>()
        definedSignatures.addAll(getAllDefinedSignatures(input))
        for (sup in supplemental) {
            definedSignatures.addAll(getAllDefinedSignatures(sup))
        }

        return when (val validation = parseWithLocations(input)) {
            is ValidationSuccess -> {
                val result = mutableListOf<Signature>()
                val signatures = findAllSignatures(validation.value.document, validation.value.tracker)
                for (sig in signatures) {
                    if (!definedSignatures.contains(sig.form)) {
                        result.add(sig)
                    }
                }
                result
            }
            is ValidationFailure -> emptyList()
        }
    }

    private fun getAllDefinedSignatures(input: String): List<String> {
        return when (val validation = parse(input)) {
            is ValidationSuccess -> {
                val result = mutableListOf<String>()
                val document = validation.value
                result.addAll(document.defines.mapNotNull { it.signature })
                result.addAll(document.represents.mapNotNull { it.signature })
                result
            }
            is ValidationFailure -> emptyList()
        }
    }

    fun expandAtPosition(
        text: String,
        row: Int,
        column: Int,
        defines: List<DefinesGroup>,
        represents: List<RepresentsGroup>
    ): Validation<Document> = when (val validation = parseWithLocations(text)) {
        is ValidationFailure -> validationFailure(validation.errors)
        is ValidationSuccess -> {
            val doc = validation.value.document
            val tracker = validation.value.tracker
            val target = findNode(tracker, doc, row, column)
            val newDoc = expandAtNode(doc, target, defines, represents) as Document
            validationSuccess(newDoc)
        }
    }

    fun expand(doc: Document) = fullExpandComplete(doc)
}

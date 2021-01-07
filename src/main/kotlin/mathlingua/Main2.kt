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

package mathlingua

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.subcommands
import com.github.ajalt.clikt.parameters.options.flag
import com.github.ajalt.clikt.parameters.options.option
import java.nio.file.Paths
import kotlinx.coroutines.runBlocking
import mathlingua.support.ParseError

private class Mlg2 : CliktCommand() {
    override fun run() = Unit
}

private class Check2 : CliktCommand(help = "Check input files for errors.") {
    private val json: Boolean by option(help = "Output the results in JSON format.").flag()

    override fun run(): Unit =
        runBlocking {
            val cwd = Paths.get(".").toAbsolutePath().normalize().toFile()
            val sourceCollection = SourceCollection(listOf(cwd))
            val errors = mutableListOf<ValueSourceTracker<ParseError>>()
            errors.addAll(sourceCollection.getParseErrors())
            errors.addAll(
                sourceCollection.getUndefinedSignatures().map {
                    ValueSourceTracker(
                        source = it.source,
                        tracker = it.tracker,
                        value =
                            ParseError(
                                message = "Undefined signature '${it.value.form}'",
                                row = it.value.location.row,
                                column = it.value.location.column))
                })
            errors.addAll(
                sourceCollection.getDuplicateDefinedSignatures().map {
                    ValueSourceTracker(
                        source = it.source,
                        tracker = it.tracker,
                        value =
                            ParseError(
                                message = "Duplicate defined signature '${it.value.form}'",
                                row = it.value.location.row,
                                column = it.value.location.column))
                })
            errors.addAll(sourceCollection.findInvalidTypes())

            val builder = StringBuilder()
            if (json) {
                builder.append("[")
            }
            for (i in errors.indices) {
                val err = errors[i]
                if (json) {
                    builder.append("{")
                    builder.append(
                        "  \"file\": \"${err.source.file.normalize().absolutePath.jsonSanitize()}\",")
                    builder.append("  \"type\": \"ERROR\",")
                    builder.append("  \"message\": \"${err.value.message.jsonSanitize()}\",")
                    builder.append("  \"failedLine\": \"\",")
                    builder.append("  \"row\": ${err.value.row},")
                    builder.append("  \"column\": ${err.value.column}")
                    builder.append("}")
                    if (i != errors.size - 1) {
                        builder.append(",")
                    }
                } else {
                    builder.append(bold(red("ERROR: ")))
                    builder.append(
                        bold(
                            "${err.source.file} (Line: ${err.value.row + 1}, Column: ${err.value.column + 1})\n"))
                    builder.append(err.value.message.trim())
                    if (i != errors.size - 1) {
                        builder.append("\n\n")
                    } else {
                        builder.append("\n")
                    }
                }
            }
            if (json) {
                builder.append("]")
            }

            println(builder.toString())
        }
}

fun main(args: Array<String>) = Mlg2().subcommands(Check2()).main(args)

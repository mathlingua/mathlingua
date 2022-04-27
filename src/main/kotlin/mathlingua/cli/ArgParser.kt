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

package mathlingua.cli

internal class CliException(message: String) : Exception(message)

internal data class ArgParse(val named: Map<String, List<String>>, val positional: List<String>)

internal fun parseArgs(
    args: Array<String>, expectedNamedArgs: Map<String, Int>, expectedPositionalArgs: Int?
): ArgParse {
    val named = mutableMapOf<String, List<String>>()
    val positional = mutableListOf<String>()
    var i = 0
    while (i < args.size) {
        val a = args[i++]
        if (a.startsWith("--")) {
            val name = a.substring(2)
            if (!expectedNamedArgs.containsKey(name)) {
                throw CliException("Unrecognized option $a")
            }
            val expectedCount = expectedNamedArgs[name]!!
            val values = mutableListOf<String>()
            while (i < args.size && values.size < expectedCount && !args[i].startsWith("--")) {
                values.add(args[i++])
            }
            if (values.size != expectedCount) {
                throw CliException(
                    "Expected $expectedCount arguments for option $a but found ${values.size}")
            }
            named[name] = values
        } else {
            positional.add(a)
        }
    }
    if (expectedPositionalArgs != null && positional.size != expectedPositionalArgs) {
        throw CliException(
            "Expected $expectedPositionalArgs positional arguments but found ${positional.size}")
    }
    return ArgParse(named, positional)
}

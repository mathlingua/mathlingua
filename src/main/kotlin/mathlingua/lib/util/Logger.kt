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

package mathlingua.lib.util

interface Logger {
    fun error(msg: String)
    fun log(msg: String)
}

fun newLogger(): Logger = LoggerImpl()

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private fun String.onWindowsReturn(text: String): String =
    if (System.getProperty("os.name").lowercase().contains("win")) {
        text
    } else {
        this
    }

internal fun bold(text: String) = "\u001B[1m$text\u001B[0m".onWindowsReturn(text)

internal fun red(text: String) = "\u001B[31m$text\u001B[0m".onWindowsReturn(text)

internal fun green(text: String) = "\u001B[32m$text\u001B[0m".onWindowsReturn(text)

private class LoggerImpl : Logger {
    override fun error(msg: String) {
        System.err.println(msg)
    }

    override fun log(msg: String) {
        println(msg)
    }
}
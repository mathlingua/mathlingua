/*
 * Copyright 2021 The MathLingua Authors
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

private fun String.onWindowsReturn(text: String): String =
    if (System.getProperty("os.name").lowercase().contains("win")) {
        text
    } else {
        this
    }

fun bold(text: String) = "\u001B[1m$text\u001B[0m".onWindowsReturn(text)

@Suppress("SAME_PARAMETER_VALUE")
fun green(text: String) = "\u001B[32m$text\u001B[0m".onWindowsReturn(text)

fun red(text: String) = "\u001B[31m$text\u001B[0m".onWindowsReturn(text)

@Suppress("UNUSED")
fun yellow(text: String) = "\u001B[33m$text\u001B[0m".onWindowsReturn(text)

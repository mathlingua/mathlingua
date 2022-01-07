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

import com.github.ajalt.clikt.output.TermUi

interface Logger {
    fun log(message: String)
    fun error(message: String)
}

interface AuditableLogger : Logger {
    fun getLogs(): List<String>
}

internal fun newTermUiLogger(termUi: TermUi): Logger {
    return TermUiLogger(termUi)
}

internal fun newMemoryLogger(): AuditableLogger {
    return MemoryLogger()
}

// -----------------------------------------------------------------------------

private class TermUiLogger(private val termUi: TermUi) : Logger {
    override fun log(message: String) = termUi.echo(message = message, err = false)

    override fun error(message: String) = termUi.echo(message = message, err = true)
}

private class MemoryLogger : AuditableLogger {
    private val logs = mutableListOf<String>()
    private val errors = mutableListOf<String>()

    override fun log(message: String) {
        logs.add(message)
    }

    override fun error(message: String) {
        errors.add(message)
    }

    override fun getLogs(): List<String> {
        return logs
    }

    fun getErrors(): List<String> {
        return errors
    }
}

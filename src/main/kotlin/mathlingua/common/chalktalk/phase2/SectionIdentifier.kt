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

package mathlingua.common.chalktalk.phase2

import mathlingua.common.ParseError
import mathlingua.common.chalktalk.phase1.ast.AstUtils
import mathlingua.common.chalktalk.phase1.ast.Section

object SectionIdentifier {

    fun identifySections(sections: List<Section>, vararg expected: String): Map<String, Section> {
        val patternBuilder = StringBuilder()
        for (name in expected) {
            patternBuilder.append(name)
            patternBuilder.append(":\n")
        }

        // the pattern is used for error messages
        val pattern = patternBuilder.toString()

        val sectionQueue = Queue<Section>()
        for (s in sections) {
            sectionQueue.offer(s)
        }

        val expectedQueue = Queue<String>()
        for (e in expected) {
            expectedQueue.offer(e)
        }

        val result = HashMap<String, Section>()

        while (!sectionQueue.isEmpty() && !expectedQueue.isEmpty()) {
            val nextSection = sectionQueue.peek()
            val maybeName = expectedQueue.peek()

            val isOptional = maybeName.endsWith("?")
            val trueName =
                    if (isOptional) maybeName.substring(0, maybeName.length - 1) else maybeName
            if (nextSection.name.text == trueName) {
                result[trueName] = nextSection
                // the expected name and Section have booth been used so move past them
                sectionQueue.poll()
                expectedQueue.poll()
            } else if (isOptional) {
                // The Section found doesn't match the expected name
                // but the expected name is optional.  So move past
                // the expected name but don't move past the Section
                // so it can be processed again in the next run of
                // the loop.
                expectedQueue.poll()
            } else {
                throw ParseError(
                        "For pattern:\n\n" + pattern +
                                "\nExpected '" + trueName + "' but found '" + nextSection.name.text + "'",
                        AstUtils.getRow(nextSection), AstUtils.getColumn(nextSection)
                )
            }
        }

        if (!sectionQueue.isEmpty()) {
            val peek = sectionQueue.peek()
            throw ParseError(
                    "For pattern:\n\n" + pattern +
                            "\nUnexpected Section '" + peek.name.text + "'",
                    AstUtils.getRow(peek), AstUtils.getColumn(peek)
            )
        }

        var nextExpected: String? = null
        for (exp in expectedQueue) {
            if (!exp.endsWith("?")) {
                // trim the ?
                nextExpected = exp
                break
            }
        }

        var startRow = -1
        var startColumn = -1
        if (!sections.isEmpty()) {
            val sect = sections[0]
            startRow = AstUtils.getRow(sect)
            startColumn = AstUtils.getColumn(sect)
        }

        if (nextExpected != null) {
            throw ParseError(
                    "For pattern:\n\n" + pattern +
                            "\nExpected a " + nextExpected, startRow, startColumn
            )
        }

        return result
    }
}

private class Queue<T> : Iterable<T> {
    private val data = ArrayList<T>()

    fun offer(item: T) {
        data.add(0, item)
    }

    fun poll(): T {
        return data.removeAt(0)
    }

    fun peek(): T {
        return data.elementAt(0)
    }

    fun isEmpty(): Boolean {
        return data.isEmpty()
    }

    override fun iterator(): Iterator<T> {
        return data.iterator()
    }
}

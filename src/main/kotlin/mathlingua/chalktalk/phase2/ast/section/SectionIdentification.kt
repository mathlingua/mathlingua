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

package mathlingua.chalktalk.phase2.ast.section

import java.util.LinkedList
import java.util.Queue
import mathlingua.chalktalk.phase1.ast.Section
import mathlingua.chalktalk.phase1.ast.getColumn
import mathlingua.chalktalk.phase1.ast.getRow
import mathlingua.support.ParseError

fun identifySections(sections: List<Section>, vararg expected: String): Map<String, Section> {
    val patternBuilder = StringBuilder()
    for (name in expected) {
        patternBuilder.append(name)
        patternBuilder.append(":\n")
    }

    // the pattern is used for error messages
    val pattern = patternBuilder.toString()

    val sectionQueue: Queue<Section> = LinkedList()
    for (s in sections) {
        sectionQueue.offer(s)
    }

    val expectedQueue: Queue<String> = LinkedList()
    for (e in expected) {
        expectedQueue.offer(e)
    }

    val usedSectionNames = mutableMapOf<String, Int>()
    val result = mutableMapOf<String, Section>()

    while (!sectionQueue.isEmpty() && !expectedQueue.isEmpty()) {
        val nextSection = sectionQueue.peek()
        val maybeName = expectedQueue.peek()

        val isOptional = maybeName.endsWith("?")
        val trueName = if (isOptional) maybeName.substring(0, maybeName.length - 1) else maybeName
        val key =
            if (usedSectionNames.containsKey(trueName)) {
                "$trueName${usedSectionNames[trueName]}"
            } else {
                trueName
            }
        usedSectionNames[trueName] = usedSectionNames.getOrDefault(trueName, 0) + 1
        if (nextSection.name.text == trueName) {
            result[key] = nextSection
            // the expected name and Section have both been used so move past them
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
                "For pattern:\n\n" +
                    pattern +
                    "\nExpected '" +
                    trueName +
                    "' but found '" +
                    nextSection.name.text +
                    "'",
                getRow(nextSection),
                getColumn(nextSection))
        }
    }

    if (!sectionQueue.isEmpty()) {
        val peek = sectionQueue.peek()
        throw ParseError(
            "For pattern:\n\n" + pattern + "\nUnexpected Section '" + peek.name.text + "'",
            getRow(peek),
            getColumn(peek))
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
    if (sections.isNotEmpty()) {
        val sect = sections[0]
        startRow = getRow(sect)
        startColumn = getColumn(sect)
    }

    if (nextExpected != null) {
        throw ParseError(
            "For pattern:\n\n" + pattern + "\nExpected a " + nextExpected, startRow, startColumn)
    }

    return result
}

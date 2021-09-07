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

package mathlingua.backend

import mathlingua.frontend.support.Location
import mathlingua.frontend.support.ParseError

object BackEnd {
    fun check(sourceCollection: SourceCollection): List<ValueSourceTracker<ParseError>> {
        val errors = mutableListOf<ValueSourceTracker<ParseError>>()
        errors.addAll(checkParseErrors(sourceCollection))
        errors.addAll(checkUndefinedSignatures(sourceCollection))
        errors.addAll(checkDuplicateDefinedSignatures(sourceCollection))
        errors.addAll(checkInvalidTypes(sourceCollection))
        errors.addAll(checkSymbolErrors(sourceCollection))
        return errors
    }

    fun checkUndefinedSignatures(sourceCollection: SourceCollection) =
        sourceCollection.getUndefinedSignatures().map {
            ValueSourceTracker(
                source = it.source,
                tracker = it.tracker,
                value =
                    ParseError(
                        message = "Undefined signature '${it.value.form}'",
                        row = it.value.location.row,
                        column = it.value.location.column))
        }

    fun checkDuplicateDefinedSignatures(sourceCollection: SourceCollection) =
        sourceCollection.getDuplicateDefinedSignatures().map {
            ValueSourceTracker(
                source = it.source,
                tracker = it.tracker,
                value =
                    ParseError(
                        message = "Duplicate defined signature '${it.value.form}'",
                        row = it.value.location.row,
                        column = it.value.location.column))
        }

    fun checkInvalidTypes(sourceCollection: SourceCollection) = sourceCollection.findInvalidTypes()

    fun checkParseErrors(sourceCollection: SourceCollection) = sourceCollection.getParseErrors()

    fun checkDuplicateContent(
        sourceCollection: SourceCollection
    ): List<ValueSourceTracker<ParseError>> {
        val errors = mutableListOf<ValueSourceTracker<ParseError>>()
        for (grp in sourceCollection.getDuplicateContent()) {
            val location = grp.tracker?.getLocationOf(grp.value) ?: Location(row = -1, column = -1)
            errors.add(
                ValueSourceTracker(
                    source = grp.source,
                    tracker = grp.tracker,
                    value =
                        ParseError(
                            message = "Duplicate content detected",
                            row = location.row,
                            column = location.column)))
        }
        return errors
    }

    fun checkSymbolErrors(
        sourceCollection: SourceCollection
    ): List<ValueSourceTracker<ParseError>> {
        return sourceCollection.getSymbolErrors()
    }
}

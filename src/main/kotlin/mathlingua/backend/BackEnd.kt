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
    internal fun check(sourceCollection: SourceCollection): List<ValueSourceTracker<ParseError>> {
        val errors = mutableListOf<ValueSourceTracker<ParseError>>()
        errors.addAll(checkParseErrors(sourceCollection))
        errors.addAll(checkUndefinedSignatures(sourceCollection))
        errors.addAll(checkDuplicateDefinedSignatures(sourceCollection))
        errors.addAll(checkInvalidTypes(sourceCollection))
        errors.addAll(checkSymbolErrors(sourceCollection))
        errors.addAll(checkIsRhs(sourceCollection))
        errors.addAll(checkColonEqualsRhs(sourceCollection))
        errors.addAll(checkInputOutputSymbolErrors(sourceCollection))
        errors.addAll(checkNonExpressesUsedInNonIsStatements(sourceCollection))
        return errors
    }

    private fun checkUndefinedSignatures(sourceCollection: SourceCollection) =
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

    private fun checkDuplicateDefinedSignatures(sourceCollection: SourceCollection) =
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

    private fun checkInvalidTypes(sourceCollection: SourceCollection) =
        sourceCollection.findInvalidTypes()

    private fun checkParseErrors(sourceCollection: SourceCollection) =
        sourceCollection.getParseErrors()

    private fun checkDuplicateContent(
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

    private fun checkSymbolErrors(
        sourceCollection: SourceCollection
    ): List<ValueSourceTracker<ParseError>> {
        return sourceCollection.getSymbolErrors()
    }

    private fun checkIsRhs(
        sourceCollection: SourceCollection
    ): List<ValueSourceTracker<ParseError>> {
        return sourceCollection.getIsRhsErrors()
    }

    private fun checkColonEqualsRhs(
        sourceCollection: SourceCollection
    ): List<ValueSourceTracker<ParseError>> {
        return sourceCollection.getColonEqualsRhsErrors()
    }

    private fun checkInputOutputSymbolErrors(
        sourceCollection: SourceCollection
    ): List<ValueSourceTracker<ParseError>> {
        return sourceCollection.getInputOutputSymbolErrors()
    }

    private fun checkNonExpressesUsedInNonIsStatements(
        sourceCollection: SourceCollection
    ): List<ValueSourceTracker<ParseError>> {
        return sourceCollection.getNonExpressesUsedInNonIsNonInStatementsErrors()
    }
}

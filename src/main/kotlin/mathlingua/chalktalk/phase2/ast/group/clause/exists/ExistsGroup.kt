/*
 * Copyright 2020 Google LLC
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

package mathlingua.chalktalk.phase2.ast.group.clause.exists

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.ast.DEFAULT_EXISTS_GROUP
import mathlingua.chalktalk.phase2.ast.DEFAULT_SUCH_THAT_SECTION
import mathlingua.chalktalk.phase2.ast.clause.Clause
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.clause.validateMidOptionalTripleSectionGroup
import mathlingua.chalktalk.phase2.ast.common.ThreePartNode
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.WhereSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.neoValidateWhereSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.validateWhereSection
import mathlingua.chalktalk.phase2.ast.neoValidateGroup
import mathlingua.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.chalktalk.phase2.ast.section.neoIdentifySections
import mathlingua.chalktalk.phase2.ast.section.neoIfNonNull
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError

data class ExistsGroup(
    val existsSection: ExistsSection,
    val whereSection: WhereSection?,
    val suchThatSection: SuchThatSection
) :
    ThreePartNode<ExistsSection, WhereSection?, SuchThatSection>(
        existsSection, whereSection, suchThatSection, ::ExistsGroup),
    Clause

fun isExistsGroup(node: Phase1Node) = firstSectionMatchesName(node, "exists")

fun validateExistsGroup(rawNode: Phase1Node, tracker: MutableLocationTracker) =
    validateMidOptionalTripleSectionGroup(
        tracker,
        rawNode,
        "exists",
        ::validateExistsSection,
        "where?",
        ::validateWhereSection,
        "suchThat",
        ::validateSuchThatSection,
        ::ExistsGroup)

fun neoValidateExistsGroup(node: Phase1Node, errors: MutableList<ParseError>) =
    neoValidateGroup(node, errors, "exists", DEFAULT_EXISTS_GROUP) { group ->
        neoIdentifySections(group, errors, DEFAULT_EXISTS_GROUP) { sections ->
            ExistsGroup(
                existsSection = neoValidateExistsSection(),
                whereSection = neoIfNonNull(sections["where"]) {
                    neoValidateWhereSection(it, errors)
                },
                suchThatSection = ensureNonNull(sections["suchThat"], DEFAULT_SUCH_THAT_SECTION) {
                    neoValidateSuchThatSection(it, errors)
                }
            )
        }
    }

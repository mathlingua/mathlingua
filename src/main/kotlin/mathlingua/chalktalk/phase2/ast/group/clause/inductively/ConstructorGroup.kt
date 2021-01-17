/*
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *    http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package mathlingua.chalktalk.phase2.ast.group.clause.inductively

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.ast.DEFAULT_CONSTRUCTOR_GROUP
import mathlingua.chalktalk.phase2.ast.DEFAULT_CONSTRUCTOR_SECTION
import mathlingua.chalktalk.phase2.ast.DEFAULT_FROM_SECTION
import mathlingua.chalktalk.phase2.ast.clause.Clause
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.common.TwoPartNode
import mathlingua.chalktalk.phase2.ast.group.clause.mapping.FromSection
import mathlingua.chalktalk.phase2.ast.group.clause.mapping.validateFromSection
import mathlingua.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.chalktalk.phase2.ast.section.identifySections
import mathlingua.chalktalk.phase2.ast.track
import mathlingua.chalktalk.phase2.ast.validateGroup
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError

data class ConstructorGroup(
    val constructorSection: ConstructorSection, val fromSection: FromSection
) :
    TwoPartNode<ConstructorSection, FromSection>(
        constructorSection, fromSection, ::ConstructorGroup),
    Clause

fun isConstructorGroup(node: Phase1Node) = firstSectionMatchesName(node, "constructor")

fun validateConstructorGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "constructor", DEFAULT_CONSTRUCTOR_GROUP) { group ->
            identifySections(
                group, errors, DEFAULT_CONSTRUCTOR_GROUP, listOf("constructor", "from")) {
            sections ->
                ConstructorGroup(
                    constructorSection =
                        ensureNonNull(sections["constructor"], DEFAULT_CONSTRUCTOR_SECTION) {
                            validateConstructorSection(it, errors, tracker)
                        },
                    fromSection =
                        ensureNonNull(sections["from"], DEFAULT_FROM_SECTION) {
                            validateFromSection(it, errors, tracker)
                        })
            }
        }
    }

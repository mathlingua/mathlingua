/*
 * Copyright 2021
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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.membership

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_MEMBERSHIP_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_MEMBERSHIP_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_THROUGH_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.Clause
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.TwoPartNode
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.viewingas.ThroughSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.viewing.viewingas.validateThroughSection
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

data class MembershipGroup(
    val membershipSection: MembershipSection, val throughSection: ThroughSection
) :
    TwoPartNode<MembershipSection, ThroughSection>(
        membershipSection, throughSection, ::MembershipGroup),
    Clause

fun isMembershipGroup(node: Phase1Node) = firstSectionMatchesName(node, "membership")

fun validateMembershipGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "membership", DEFAULT_MEMBERSHIP_GROUP) { group ->
            identifySections(
                group, errors, DEFAULT_MEMBERSHIP_GROUP, listOf("membership", "through")) {
            sections ->
                MembershipGroup(
                    membershipSection =
                        ensureNonNull(sections["membership"], DEFAULT_MEMBERSHIP_SECTION) {
                            validateMembershipSection(it, errors, tracker)
                        },
                    throughSection =
                        ensureNonNull(sections["through"], DEFAULT_THROUGH_SECTION) {
                            validateThroughSection(it, errors, tracker)
                        })
            }
        }
    }

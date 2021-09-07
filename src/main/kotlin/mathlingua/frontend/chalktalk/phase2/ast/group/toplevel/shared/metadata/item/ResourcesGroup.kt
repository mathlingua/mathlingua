/*
 * Copyright 2020 The MathLingua Authors
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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_RESOURCES_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_RESOURCES_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.OnePartNode
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.ResourcesSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateResourcesSection
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

data class ResourcesGroup(val resourcesSection: ResourcesSection) :
    OnePartNode<ResourcesSection>(resourcesSection, ::ResourcesGroup), MetaDataItem

fun isResourcesGroup(node: Phase1Node) = firstSectionMatchesName(node, "resources")

fun validateResourcesGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node, errors, "resources", DEFAULT_RESOURCES_GROUP) { group ->
            identifySections(group, errors, DEFAULT_RESOURCES_GROUP, listOf("resources")) {
            sections ->
                ResourcesGroup(
                    resourcesSection =
                        ensureNonNull(sections["resources"], DEFAULT_RESOURCES_SECTION) {
                            validateResourcesSection(it, errors, tracker)
                        })
            }
        }
    }

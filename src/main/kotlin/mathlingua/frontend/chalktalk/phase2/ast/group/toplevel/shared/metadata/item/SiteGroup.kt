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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_SITE_GROUP
import mathlingua.frontend.chalktalk.phase2.ast.DEFAULT_SITE_ITEM_SECTION
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.common.TwoPartNode
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.NameItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.SiteItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateNameItemSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.validateSiteItemSection
import mathlingua.frontend.chalktalk.phase2.ast.section.ensureNonNull
import mathlingua.frontend.chalktalk.phase2.ast.section.identifySections
import mathlingua.frontend.chalktalk.phase2.ast.section.ifNonNull
import mathlingua.frontend.chalktalk.phase2.ast.track
import mathlingua.frontend.chalktalk.phase2.ast.validateGroup
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

internal data class SiteGroup(
    val siteItemSection: SiteItemSection, val nameItemSection: NameItemSection?
) :
    TwoPartNode<SiteItemSection, NameItemSection?>(siteItemSection, nameItemSection, ::SiteGroup),
    MetaDataItem,
    ResourceItem

internal fun isSiteGroup(node: Phase1Node) = firstSectionMatchesName(node, "site")

internal fun validateSiteGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
) =
    track(node, tracker) {
        validateGroup(node.resolve(), errors, "site", DEFAULT_SITE_GROUP) { group ->
            identifySections(group, errors, DEFAULT_SITE_GROUP, listOf("site", "name?")) {
            sections ->
                SiteGroup(
                    siteItemSection =
                        ensureNonNull(sections["site"], DEFAULT_SITE_ITEM_SECTION) {
                            validateSiteItemSection(it, errors, tracker)
                        },
                    nameItemSection =
                        ifNonNull(sections["name"]) {
                            validateNameItemSection(it, errors, tracker)
                        })
            }
        }
    }

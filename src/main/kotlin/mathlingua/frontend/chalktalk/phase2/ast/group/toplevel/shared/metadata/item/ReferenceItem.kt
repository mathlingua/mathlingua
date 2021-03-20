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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.item

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

interface ReferenceItem : Phase2Node

fun validateReferenceItem(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
): ReferenceItem =
    when {
        isSiteGroup(node.resolve()) -> {
            validateSiteGroup(node, errors, tracker)
        }
        isSourceItemGroup(node.resolve()) -> {
            validateSourceItemGroup(node, errors, tracker)
        }
        else -> {
            throw RuntimeException("Unknown ReferenceItem: $node")
        }
    }

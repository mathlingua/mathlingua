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

package mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines

import mathlingua.frontend.chalktalk.phase1.ast.Phase1Node
import mathlingua.frontend.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.frontend.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.DefinesStatesOrViews
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.frontend.support.MutableLocationTracker
import mathlingua.frontend.support.ParseError

abstract class DefinesGroup(override val metaDataSection: MetaDataSection?) :
    TopLevelGroup(metaDataSection), DefinesStatesOrViews {
    abstract val signature: String?
    abstract val definesSection: DefinesSection
    abstract val id: IdStatement
    abstract val writtenSection: WrittenSection
}

fun isDefinesGroup(node: Phase1Node) = firstSectionMatchesName(node, "Defines")

fun validateDefinesGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
): DefinesGroup =
    when {
        isDefinesCollectsGroup(node) -> {
            validateDefinesCollectsGroup(node, errors, tracker)
        }
        isDefinesGeneratedGroup(node) -> {
            validateDefinesGeneratedGroup(node, errors, tracker)
        }
        isDefinesInstantiatedGroup(node) -> {
            validateDefinesInstantiatedGroup(node, errors, tracker)
        }
        isDefinesMapsGroup(node) -> {
            validateDefinesMapsGroup(node, errors, tracker)
        }
        isDefinesEvaluatedGroup(node) -> {
            validateDefinesEvaluatedGroup(node, errors, tracker)
        }
        else -> {
            validateDefinesMeansGroup(node, errors, tracker)
        }
    }

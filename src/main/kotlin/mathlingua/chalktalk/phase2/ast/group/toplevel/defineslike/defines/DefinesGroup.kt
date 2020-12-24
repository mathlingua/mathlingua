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

package mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.defines

import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase2.ast.clause.IdStatement
import mathlingua.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.chalktalk.phase2.ast.group.toplevel.TopLevelGroup
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.WrittenSection
import mathlingua.chalktalk.phase2.ast.group.toplevel.defineslike.foundation.DefinesStatesOrViews
import mathlingua.chalktalk.phase2.ast.group.toplevel.shared.metadata.section.MetaDataSection
import mathlingua.support.MutableLocationTracker
import mathlingua.support.ParseError

abstract class DefinesGroup(override val metaDataSection: MetaDataSection?) :
    TopLevelGroup(metaDataSection), DefinesStatesOrViews {
    abstract val signature: String?
    abstract val id: IdStatement
    abstract val writtenSection: WrittenSection
}

fun isDefinesGroup(node: Phase1Node) = firstSectionMatchesName(node, "Defines")

fun neoValidateDefinesGroup(
    node: Phase1Node, errors: MutableList<ParseError>, tracker: MutableLocationTracker
): DefinesGroup =
    when {
        isDefinesCollectsGroup(node) -> {
            neoValidateDefinesCollectsGroup(node, errors, tracker)
        }
        isDefinesGeneratedGroup(node) -> {
            neoValidateDefinesGeneratedGroup(node, errors, tracker)
        }
        isDefinesInstantiatedGroup(node) -> {
            neoValidateDefinesInstantiatedGroup(node, errors, tracker)
        }
        isDefinesMapsGroup(node) -> {
            neoValidateDefinesMapsGroup(node, errors, tracker)
        }
        isDefinesEvaluatedGroup(node) -> {
            neoValidateDefinesEvaluatedGroup(node, errors, tracker)
        }
        else -> {
            neoValidateDefinesMeansGroup(node, errors, tracker)
        }
    }

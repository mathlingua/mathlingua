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

package mathlingua.common.chalktalk.phase2.ast.toplevel

import mathlingua.common.MutableLocationTracker
import mathlingua.common.chalktalk.phase1.ast.Group
import mathlingua.common.chalktalk.phase2.ast.clause.firstSectionMatchesName
import mathlingua.common.chalktalk.phase2.ast.metadata.section.MetaDataSection
import mathlingua.common.chalktalk.phase2.ast.section.TextSection

data class DefinitionGroup(
    override val textSection: TextSection,
    override val metaDataSection: MetaDataSection?
) : ProtoGroup(textSection, metaDataSection)

fun isDefinitionGroup(groupNode: Group) = firstSectionMatchesName(groupNode, "Definition")

fun validateDefinitionGroup(groupNode: Group, tracker: MutableLocationTracker) =
    validateProtoGroup(groupNode, "Definition", tracker, ::DefinitionGroup)

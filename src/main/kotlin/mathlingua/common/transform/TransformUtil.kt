/*
 * Copyright 2019 Google LLC
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

package mathlingua.common.transform

import mathlingua.common.chalktalk.phase2.DefinesGroup
import mathlingua.common.chalktalk.phase2.IfGroup
import mathlingua.common.chalktalk.phase2.IfSection
import mathlingua.common.chalktalk.phase2.MeansSection
import mathlingua.common.chalktalk.phase2.ThenSection

fun toCanonicalForm(def: DefinesGroup): DefinesGroup {
    return DefinesGroup(
        signature = def.signature,
        id = def.id,
        definesSection = def.definesSection,
        assumingSection = null,
        meansSection = MeansSection(
            clauses = listOf(buildIfThen(def))
        ),
        aliasSection = def.aliasSection,
        metaDataSection = def.metaDataSection
    )
}

fun buildIfThen(def: DefinesGroup): IfGroup {
    return IfGroup(
        ifSection = IfSection(
            clauses = def.assumingSection?.clauses ?: emptyList()
        ),
        thenSection = ThenSection(
            clauses = def.meansSection.clauses
        )
    )
}

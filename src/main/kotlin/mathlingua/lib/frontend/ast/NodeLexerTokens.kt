/*
 * Copyright 2022 Dominic Kramer
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

package mathlingua.lib.frontend.ast

import mathlingua.lib.frontend.HasMetaData
import mathlingua.lib.frontend.MetaData

internal sealed interface NodeLexerToken : HasMetaData

internal data class BeginGroup(val name: String?, override val metadata: MetaData) : NodeLexerToken

internal data class EndGroup(override val metadata: MetaData) : NodeLexerToken

internal data class BeginSection(val name: String, override val metadata: MetaData) :
    NodeLexerToken

internal data class EndSection(override val metadata: MetaData) : NodeLexerToken

internal data class BeginArgument(override val metadata: MetaData) : NodeLexerToken

internal data class EndArgument(override val metadata: MetaData) : NodeLexerToken

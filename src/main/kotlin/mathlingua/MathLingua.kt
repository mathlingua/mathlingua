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

package mathlingua

object MathLingua {
    /*

        fun expandWrittenAs(
            phase2Node: Phase2Node, patternToExpansion: Map<OperatorTexTalkNode, String>
        ): Phase2Node {
            return phase2Node.transform {
                when (it) {
                    is Statement ->
                        when (val validation = it.texTalkRoot
                        ) {
                            is ValidationFailure -> it
                            is ValidationSuccess -> {
                                val texTalkNode = validation.value
                                val expansion = expandAsWritten(texTalkNode, patternToExpansion)
                                Statement(
                                    text = expansion.text ?: it.toCode(false, 0).getCode(),
                                    texTalkRoot = validation)
                            }
                        }
                    else -> it
                }
            }
        }


    */
}

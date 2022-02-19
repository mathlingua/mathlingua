/*
 * Copyright 2022 The MathLingua Authors
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

package mathlingua.frontend.textalk

import mathlingua.backend.transform.GroupScope
import mathlingua.backend.transform.Signature
import mathlingua.backend.transform.getVarsTexTalkNode
import mathlingua.backend.transform.signature
import mathlingua.frontend.support.Location

internal fun TexTalkNode.deepForEachTopDown(fn: (node: TexTalkNode) -> Boolean) {
    val doContinue = fn(this)
    if (doContinue) {
        this.forEach { it.deepForEachTopDown(fn) }
    }
}

internal fun TexTalkNode.getSignaturesWithin(): List<String> {
    val result = mutableListOf<String>()
    this.deepForEachTopDown {
        if (it is Command) {
            result.add(it.signature())
        }
        true
    }
    return result
}

internal fun TexTalkNode.findIsRhsSignatures(
    location: Location, rhsIsSignatures: MutableList<Signature>
) {
    if (this is IsTexTalkNode) {
        this.rhs.items.forEach { item ->
            item.children.forEach { child ->
                if (child is Command) {
                    val signature = child.signature()
                    rhsIsSignatures.add(Signature(form = signature, location = location))
                }
            }
        }
    } else {
        this.forEach { it.findIsRhsSignatures(location, rhsIsSignatures) }
    }
}

internal fun TexTalkNode.findInRhsSignatures(
    location: Location, rhsInSignatures: MutableList<Signature>
) {
    this.deepForEachTopDown { n ->
        if (n is InTexTalkNode) {
            n.rhs.items.forEach { item ->
                item.children.forEach { child ->
                    if (child is Command) {
                        val signature = child.signature()
                        rhsInSignatures.add(Signature(form = signature, location = location))
                    }
                }
            }
        }
        true
    }
}

internal fun TexTalkNode.findColonEqualsRhsSignatures(
    location: Location, rhsIsSignatures: MutableList<Signature>
) {
    this.deepForEachTopDown { n ->
        if (n is ColonEqualsTexTalkNode) {
            n.rhs.items.forEach { item ->
                item.children.forEach { child ->
                    if (child is Command) {
                        val signature = child.signature()
                        rhsIsSignatures.add(Signature(form = signature, location = location))
                    }
                }
            }
        }
        true
    }
}

internal fun TexTalkNode.findIsLhsSymbols(
    location: Location, lhsIsSymbols: MutableList<Pair<String, Location>>
) {
    this.deepForEachTopDown { n ->
        if (n is IsTexTalkNode) {
            n.lhs.items.forEach {
                lhsIsSymbols.addAll(
                    it.getVarsTexTalkNode(
                            isInLhsOfColonEqualsIsOrIn = false,
                            groupScope = GroupScope.InNone,
                            isInIdStatement = false,
                            forceIsPlaceholder = false)
                        .map { symbol -> Pair(symbol.name, location) })
            }
        }
        true
    }
}

internal fun TexTalkNode.findColonEqualsLhsSymbols(
    location: Location, lhsColonEqualsSymbols: MutableList<Pair<String, Location>>
) {
    this.deepForEachTopDown { n ->
        if (n is ColonEqualsTexTalkNode) {
            n.lhs.items.forEach {
                lhsColonEqualsSymbols.addAll(
                    it.getVarsTexTalkNode(
                            isInLhsOfColonEqualsIsOrIn = true,
                            groupScope = GroupScope.InNone,
                            isInIdStatement = false,
                            forceIsPlaceholder = false)
                        .map { symbol -> Pair(symbol.name, location) })
            }
        }
        true
    }
}

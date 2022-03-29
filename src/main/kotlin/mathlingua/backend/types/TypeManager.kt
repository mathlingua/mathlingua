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

package mathlingua.backend.types

import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.defines.DefinesGroup
import mathlingua.frontend.chalktalk.phase2.ast.group.toplevel.defineslike.providing.viewing.ViewGroup
import mathlingua.frontend.support.ValidationSuccess
import mathlingua.frontend.textalk.IsTexTalkNode
import mathlingua.frontend.textalk.getSignaturesWithin
import mathlingua.newQueue
import mathlingua.newStack

internal interface TypeManager {
    fun add(defines: DefinesGroup)
    fun remove(defines: DefinesGroup)
    fun isSigDescendantOf(sig: String, targetSig: String): Boolean
    fun isSigViewableAs(sig: String, targetSig: String): Boolean
    fun isSigIs(sig: String, targetSig: String): Boolean
    fun getLineage(sig: String): List<String>
    fun getLeastCommonAncestor(sigs: List<String>): String?
    fun doTypesMatch(actual: Set<String>, expected: Set<String>): Boolean
}

internal fun newTypeManager(): TypeManager {
    return TypeManagerImpl()
}

// ----------------------------------------------------------

private class TypeManagerImpl : TypeManager {
    private val sigToDefines = mutableMapOf<String, DefinesGroup>()
    private val sigToParentSig = mutableMapOf<String, String>()
    private val sigToViewableSigs = mutableMapOf<String, MutableSet<String>>()

    override fun add(defines: DefinesGroup) {
        val sig = defines.signature?.form ?: return
        sigToDefines[sig] = defines
        val meansStmt = defines.meansSection?.statements?.get(0)
        when (val root = meansStmt?.texTalkRoot
        ) {
            is ValidationSuccess -> {
                val child = root.value.children[0]
                if (child is IsTexTalkNode) {
                    val parentSig = child.rhs.items[0].getSignaturesWithin().first()
                    sigToParentSig[sig] = parentSig
                }
            }
            else -> {
                // ignore Defines: that have an invalid means: section
            }
        }

        val clauses = defines.providingSection?.clauses?.clauses
        if (clauses != null && clauses.isNotEmpty()) {
            clauses.forEach { clause ->
                if (clause is ViewGroup) {
                    when (val root = clause.viewAsSection.statement.texTalkRoot
                    ) {
                        is ValidationSuccess -> {
                            val sigsWithin = root.value.getSignaturesWithin()
                            if (sigsWithin.isNotEmpty()) {
                                val viewSig = sigsWithin.first()
                                if (sig !in sigToViewableSigs) {
                                    sigToViewableSigs[sig] = mutableSetOf()
                                }
                                sigToViewableSigs[sig]?.add(viewSig)
                            }
                        }
                        else -> {
                            // ignore as: sections that are invalid
                        }
                    }
                }
            }
        }
    }

    override fun remove(defines: DefinesGroup) {
        val sig = defines.signature?.form ?: return
        sigToDefines.remove(sig)
        sigToParentSig.remove(sig)
        sigToViewableSigs.remove(sig)
    }

    override fun isSigDescendantOf(sig: String, targetSig: String): Boolean {
        var cur: String? = sig
        while (cur != null) {
            if (cur == targetSig) {
                return true
            }
            cur = sigToParentSig[cur]
        }
        return false
    }

    override fun isSigViewableAs(sig: String, targetSig: String): Boolean {
        val visited = mutableSetOf<String>()
        val queue = newQueue<String>()
        queue.offer(sig)
        while (!queue.isEmpty()) {
            val size = queue.size()
            for (i in 0 until size) {
                val top = queue.poll()
                visited.add(top)
                if (top == targetSig || isSigDescendantOf(top, targetSig)) {
                    return true
                }
                sigToViewableSigs[top]?.forEach { viewSig ->
                    if (viewSig !in visited) {
                        queue.offer(viewSig)
                    }
                }
            }
        }
        return false
    }

    override fun isSigIs(sig: String, targetSig: String) =
        isSigDescendantOf(sig, targetSig) || isSigViewableAs(sig, targetSig)

    override fun getLineage(sig: String): List<String> {
        val result = mutableListOf<String>()
        var cur: String? = sig
        while (cur != null) {
            result.add(cur)
            cur = sigToParentSig[cur]
        }
        return result
    }

    override fun getLeastCommonAncestor(sigs: List<String>): String? {
        if (sigs.isEmpty()) {
            return null
        }

        val lineages =
            sigs.map {
                val stack = newStack<String>()
                getLineage(it).forEach { item -> stack.push(item) }
                stack
            }

        var latest: String? = null
        while (!lineages[0].isEmpty()) {
            val candidate = lineages[0].pop()
            var allMatch = true
            for (i in 1 until lineages.size) {
                if (lineages[i].isEmpty() || lineages[i].peek() != candidate) {
                    allMatch = false
                }

                if (!lineages[i].isEmpty()) {
                    lineages[i].pop()
                }
            }

            // If they all don't match then break and the latest ancestor they all
            // had in common will be returned or null if they have no such ancestor
            // in common.
            if (!allMatch) {
                break
            }

            latest = candidate
        }

        return latest
    }

    override fun doTypesMatch(actual: Set<String>, expected: Set<String>): Boolean {
        for (exp in expected) {
            if (!doTypesContain(actual, exp)) {
                return false
            }
        }
        return true
    }

    private fun doTypesContain(types: Set<String>, target: String): Boolean {
        for (t in types) {
            if (isSigIs(t, target)) {
                return true
            }
        }
        return false
    }
}

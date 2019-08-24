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

package mathlingua.common.chalktalk.phase1.ast

interface ChalkTalkNode {
    fun forEach(fn: (node: ChalkTalkNode) -> Unit)
    fun toCode(): String
    fun resolve(): ChalkTalkNode
}

data class Root(val groups: List<Group>) : ChalkTalkNode {

    override fun forEach(fn: (node: ChalkTalkNode) -> Unit) {
        groups.forEach(fn)
    }

    fun print(buffer: StringBuilder) {
        for (grp in groups) {
            grp.print(buffer, 0, false)
        }
    }

    override fun toCode(): String {
        val buffer = StringBuilder()
        print(buffer)
        return buffer.toString()
    }

    override fun resolve(): ChalkTalkNode {
        return this
    }
}

data class Argument(val chalkTalkTarget: ChalkTalkTarget) : ChalkTalkNode {

    override fun forEach(fn: (node: ChalkTalkNode) -> Unit) {
        fn(chalkTalkTarget)
    }

    fun print(buffer: StringBuilder, level: Int) {
        when (chalkTalkTarget) {
            is Group -> chalkTalkTarget.print(buffer, level, true)
            is ChalkTalkToken -> {
                buffer.append(AstUtils.buildIndent(level, true))
                buffer.append(chalkTalkTarget.text)
                buffer.append("\n")
            }
            is Abstraction -> {
                buffer.append(AstUtils.buildIndent(level, true))
                buffer.append(chalkTalkTarget.toCode())
                buffer.append("\n")
            }
            is Aggregate -> {
                buffer.append(AstUtils.buildIndent(level, true))
                buffer.append(chalkTalkTarget.toCode())
                buffer.append("\n")
            }
            is Assignment -> {
                buffer.append(AstUtils.buildIndent(level, true))
                buffer.append(chalkTalkTarget.toCode())
                buffer.append("\n")
            }
            is Mapping -> {
                buffer.append(AstUtils.buildIndent(level, true))
                buffer.append(chalkTalkTarget.toCode())
                buffer.append("\n")
            }
            is Tuple -> {
                buffer.append(AstUtils.buildIndent(level, true))
                buffer.append(chalkTalkTarget.toCode())
                buffer.append("\n")
            }
        }
    }

    override fun toCode(): String {
        val buffer = StringBuilder()
        this.print(buffer, 0)
        return buffer.toString()
    }

    override fun resolve(): ChalkTalkNode {
        return chalkTalkTarget.resolve()
    }
}

data class Section(val name: ChalkTalkToken, val args: List<Argument>) : ChalkTalkNode {

    override fun forEach(fn: (node: ChalkTalkNode) -> Unit) {
        fn(name)
        args.forEach(fn)
    }

    fun print(buffer: StringBuilder, level: Int, fromArg: Boolean) {
        buffer.append(AstUtils.buildIndent(level, fromArg))
        buffer.append(name.text)
        buffer.append(":\n")
        for (arg in args) {
            arg.print(buffer, level + 1)
        }
    }

    override fun toCode(): String {
        val buffer = StringBuilder()
        print(buffer, 0, false)
        return buffer.toString()
    }

    override fun resolve(): ChalkTalkNode {
        return this
    }
}


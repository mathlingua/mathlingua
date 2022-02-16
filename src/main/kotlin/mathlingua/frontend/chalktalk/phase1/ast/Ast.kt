/*
 * Copyright 2019 The MathLingua Authors
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

package mathlingua.frontend.chalktalk.phase1.ast

internal interface Phase1Node {
    fun forEach(fn: (node: Phase1Node) -> Unit)
    fun toCode(): String
    fun resolve(): Phase1Node
    fun transform(transformer: (node: Phase1Node) -> Phase1Node): Phase1Node
}

internal data class Root(val groups: List<GroupOrBlockComment>) : Phase1Node {

    override fun forEach(fn: (node: Phase1Node) -> Unit) = groups.forEach(fn)

    private fun print(buffer: StringBuilder) {
        for (grp in groups) {
            if (grp is Group) {
                grp.print(buffer, 0, false)
            } else if (grp is BlockComment) {
                grp.print(buffer)
            }
        }
    }

    override fun toCode(): String {
        val buffer = StringBuilder()
        print(buffer)
        return buffer.toString()
    }

    override fun resolve() = this

    override fun transform(transformer: (node: Phase1Node) -> Phase1Node) =
        transformer(Root(groups = groups.map { it.transform(transformer) as Group }))
}

internal data class Argument(val chalkTalkTarget: Phase1Target, val onNewLine: Boolean) :
    Phase1Node {

    override fun forEach(fn: (node: Phase1Node) -> Unit) = fn(chalkTalkTarget)

    fun print(buffer: StringBuilder, level: Int) {
        when (chalkTalkTarget) {
            is Group -> chalkTalkTarget.print(buffer, level, onNewLine)
            is Phase1Token -> {
                buffer.append(buildIndent(level, onNewLine))
                buffer.append(chalkTalkTarget.text)
            }
            is Abstraction -> {
                buffer.append(buildIndent(level, onNewLine))
                buffer.append(chalkTalkTarget.toCode())
            }
            is Assignment -> {
                buffer.append(buildIndent(level, onNewLine))
                buffer.append(chalkTalkTarget.toCode())
            }
            is Tuple -> {
                buffer.append(buildIndent(level, onNewLine))
                buffer.append(chalkTalkTarget.toCode())
            }
            is AbstractionPart -> {
                throw RuntimeException("Argument.print: Unexpected AbstractionPart encountered")
            }
        }
    }

    override fun toCode(): String {
        val buffer = StringBuilder()
        this.print(buffer, 0)
        return buffer.toString()
    }

    override fun resolve() = chalkTalkTarget.resolve()

    override fun transform(transformer: (node: Phase1Node) -> Phase1Node) =
        transformer(
            Argument(
                chalkTalkTarget = chalkTalkTarget.transform(transformer) as Phase1Target,
                onNewLine = onNewLine))
}

internal data class Section(val name: Phase1Token, val args: List<Argument>) : Phase1Node {

    override fun forEach(fn: (node: Phase1Node) -> Unit) {
        fn(name)
        args.forEach(fn)
    }

    fun print(buffer: StringBuilder, level: Int, fromArg: Boolean) {
        buffer.append(buildIndent(level, fromArg))
        buffer.append(name.text)
        buffer.append(":")
        var first = true
        for (arg in args) {
            if (arg.onNewLine) {
                buffer.append("\n")
            } else {
                if (first) {
                    buffer.append(" ")
                } else {
                    buffer.append(", ")
                }
            }
            arg.print(
                buffer,
                if (arg.onNewLine) {
                    level + 1
                } else {
                    0
                })
            first = false
        }
        if (args.isNotEmpty()) {
            buffer.append("\n")
        }
    }

    override fun toCode(): String {
        val buffer = StringBuilder()
        print(buffer, 0, false)
        return buffer.toString()
    }

    override fun resolve() = this

    override fun transform(transformer: (node: Phase1Node) -> Phase1Node) =
        transformer(
            Section(
                name = name.transform(transformer) as Phase1Token,
                args = args.map { it.transform(transformer) as Argument }))
}

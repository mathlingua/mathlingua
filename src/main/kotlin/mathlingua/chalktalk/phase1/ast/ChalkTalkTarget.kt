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

package mathlingua.chalktalk.phase1.ast

enum class ChalkTalkTokenType {
  DotSpace,
  Name,
  Colon,
  String,
  Statement,
  Id,
  Comma,
  Begin,
  End,
  Linebreak,
  Invalid,
  Equals,
  ColonEquals,
  LParen,
  RParen,
  LCurly,
  RCurly
}

sealed class ChalkTalkTarget : ChalkTalkNode
sealed class TupleItem : ChalkTalkTarget()
sealed class AssignmentRhs : TupleItem()

data class ChalkTalkToken(val text: String, val type: ChalkTalkTokenType, val row: Int, val column: Int) :
  AssignmentRhs() {

  override fun forEach(fn: (node: ChalkTalkNode) -> Unit) {
  }

  override fun toCode(): String {
    return text
  }

  override fun resolve(): ChalkTalkNode {
    return this
  }
}

data class Mapping(val lhs: ChalkTalkToken, val rhs: ChalkTalkToken) : ChalkTalkTarget() {

  override fun forEach(fn: (node: ChalkTalkNode) -> Unit) {
    fn(lhs)
    fn(rhs)
  }

  override fun toCode(): String {
    return lhs.toCode() + " = " + rhs.toCode()
  }

  override fun resolve(): ChalkTalkNode {
    return this
  }
}

data class Group(val sections: List<Section>, val id: ChalkTalkToken?) :
  ChalkTalkTarget() {

  override fun forEach(fn: (node: ChalkTalkNode) -> Unit) {
    if (id != null) {
      fn(id)
    }
    sections.forEach(fn)
  }

  fun print(buffer: StringBuilder, level: Int, fromArg: Boolean) {
    if (id != null) {
      buffer.append(id.text)
      buffer.append("\n")
    }
    var first = true
    for (sect in sections) {
      if (first) {
        sect.print(buffer, level, fromArg && id == null)
      } else {
        sect.print(buffer, level, false)
      }
      first = false
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

data class Assignment(val lhs: ChalkTalkToken, val rhs: AssignmentRhs) : TupleItem() {

  override fun forEach(fn: (node: ChalkTalkNode) -> Unit) {
    fn(lhs)
    fn(rhs)
  }

  override fun toCode(): String {
    return lhs.toCode() + " := " + rhs.toCode()
  }

  override fun resolve(): ChalkTalkNode {
    return this
  }
}

data class Tuple(val items: List<TupleItem>) : AssignmentRhs() {

  override fun forEach(fn: (node: ChalkTalkNode) -> Unit) {
    items.forEach(fn)
  }

  override fun toCode(): String {
    var builder = StringBuilder()
    builder.append('(')
    for (i in 0 until items.size) {
      builder.append(items[i].toCode())
      if (i != items.size - 1) {
        builder.append(", ")
      }
    }
    builder.append(')')
    return builder.toString()
  }

  override fun resolve(): ChalkTalkNode {
    return this
  }
}

data class Abstraction(val name: ChalkTalkToken, val params: List<ChalkTalkToken>) : TupleItem() {

  override fun forEach(fn: (node: ChalkTalkNode) -> Unit) {
    fn(name)
    params.forEach(fn)
  }

  override fun toCode(): String {
    val builder = StringBuilder()
    builder.append(name.toCode())
    builder.append('(')
    for (i in 0 until params.size) {
      builder.append(params[i].toCode())
      if (i != params.size - 1) {
        builder.append(", ")
      }
    }
    builder.append(')')
    return builder.toString()
  }

  override fun resolve(): ChalkTalkNode {
    return this
  }
}

data class Aggregate(val params: List<ChalkTalkToken>) : AssignmentRhs() {

  override fun forEach(fn: (node: ChalkTalkNode) -> Unit) {
    params.forEach(fn)
  }

  override fun toCode(): String {
    val builder = StringBuilder()
    builder.append('{')
    for (i in 0 until params.size) {
      builder.append(params[i].toCode())
      if (i != params.size - 1) {
        builder.append(", ")
      }
    }
    builder.append('}')
    return builder.toString()
  }

  override fun resolve(): ChalkTalkNode {
    return this
  }
}

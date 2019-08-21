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

package mathlingua.chalktalk.phase2.ast

import mathlingua.chalktalk.phase1.ast.AstUtils
import mathlingua.chalktalk.phase1.ast.ChalkTalkNode
import mathlingua.chalktalk.phase1.ast.Section
import mathlingua.common.ParseError
import mathlingua.common.Validation
import java.util.*

data class ClauseListSection(val name: String, val clauses: List<Clause>)

object ClauseListValidator {

  fun <T> validate(
    rawNode: ChalkTalkNode, expectedName: String,
    builder: (clauses: List<Clause>) -> T
  ): Validation<T> {
    val node = rawNode.resolve()

    val validation =
      validate(node, expectedName)
    if (!validation.isSuccessful) {
      return Validation.failure(validation.errors)
    }

    val clauses = validation.value!!.clauses
    return Validation.success(builder(clauses))
  }

  private fun validate(node: ChalkTalkNode, expectedName: String): Validation<ClauseListSection> {
    val errors = ArrayList<ParseError>()
    if (node !is Section) {
      errors.add(
        ParseError(
          "Expected a Section but found a " + node.javaClass.simpleName,
          AstUtils.getRow(node), AstUtils.getColumn(node)
        )
      )
    }

    val (name, args) = node as Section
    if (name.text != expectedName) {
      errors.add(
        ParseError(
          "Expected a Section with name " +
            expectedName + " but found " + name.text,
          AstUtils.getRow(node), AstUtils.getColumn(node)
        )
      )
    }

    if (args.isEmpty()) {
      errors.add(
        ParseError(
          "Section '" + name.text + "' requires at least one argument.",
          AstUtils.getRow(node), AstUtils.getColumn(node)
        )
      )
    }

    val clauses = ArrayList<Clause>()
    for (arg in args) {
      val validation = Clause.validate(arg)
      if (validation.isSuccessful) {
        clauses.add(validation.value!!)
      } else {
        errors.addAll(validation.errors)
      }
    }

    return if (errors.isNotEmpty()) {
      Validation.failure(errors)
    } else Validation.success(
      ClauseListSection(
        name.text,
        clauses
      )
    )

  }
}

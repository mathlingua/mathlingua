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

data class TargetListSection(val targets: List<Target>)

object TargetListValidator {

  fun <T> validate(
    node: ChalkTalkNode,
    expectedName: String,
    builder: (targets: List<Target>) -> T
  ): Validation<T> {
    var node = node
    node = node.resolve()

    val validation =
      validate(node, expectedName)
    if (!validation.isSuccessful) {
      return Validation.failure(validation.errors)
    }

    val targets = validation.value!!.targets
    return Validation.success(builder(targets))
  }

  private fun validate(
    node: ChalkTalkNode,
    expectedName: String
  ): Validation<TargetListSection> {
    val errors = ArrayList<ParseError>()
    if (node !is Section) {
      errors.add(
        ParseError(
          "Expected a Section but found a " + node.javaClass.simpleName,
          AstUtils.getRow(node), AstUtils.getColumn(node)
        )
      )
    }

    val (name1, args) = node as Section
    val name = name1.text
    if (name != expectedName) {
      errors.add(
        ParseError(
          "Expected a Section with name " +
            expectedName + " but found " + name,
          AstUtils.getRow(node),
          AstUtils.getColumn(node)
        )
      )
    }

    val targets = ArrayList<Target>()

    if (args.isEmpty()) {
      errors.add(
        ParseError(
          "Section '" + name1.text +
            "' requires at least one argument.",
          AstUtils.getRow(node),
          AstUtils.getColumn(node)
        )
      )
    }

    for (arg in args) {
      val clauseValidation = Clause.validate(arg)
      if (clauseValidation.isSuccessful) {
        val clause = clauseValidation.value
        if (clause is Target) {
          targets.add(clause)
          continue
        }
      } else {
        errors.addAll(clauseValidation.errors)
      }

      errors.add(
        ParseError(
          "Expected an Target but found " +
            arg.javaClass.simpleName, AstUtils.getRow(arg), AstUtils.getColumn(arg)
        )
      )
    }

    return if (errors.isNotEmpty()) {
      Validation.failure(errors)
    } else Validation.success(TargetListSection(targets))

  }
}

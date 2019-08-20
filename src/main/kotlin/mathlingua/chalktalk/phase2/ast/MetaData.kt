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
import mathlingua.chalktalk.phase1.ast.Section
import mathlingua.common.ParseError
import mathlingua.common.Validation

data class MetaDataSection(val mappings: List<MappingNode>) : Phase2Node {
  override fun forEach(fn: (node: Phase2Node) -> Unit) {
    mappings.forEach(fn)
  }

  override fun toCode(isArg: Boolean, indent: Int): String {
    val builder = StringBuilder()
    builder.append(indentedString(isArg, indent, "Metadata:"))
    builder.append('\n')
    for (i in 0 until mappings.size) {
      builder.append(mappings[i].toCode(true, indent + 2))
      if (i != mappings.size - 1) {
        builder.append('\n')
      }
    }
    return builder.toString()
  }

  companion object {
    fun validate(section: Section): Validation<MetaDataSection> {
      if (section.name.text != "Metadata") {
        return Validation.failure(
          listOf(
            ParseError(
              "Expected a 'Metadata' but found '${section.name.text}'",
              AstUtils.getRow(section), AstUtils.getColumn(section)
            )
          )
        )
      }

      val errors = mutableListOf<ParseError>()
      val mappings = mutableListOf<MappingNode>()
      for (arg in section.args) {
        val validation = MappingNode.validate(arg)
        if (validation.isSuccessful) {
          mappings.add(validation.value!!)
        } else {
          errors.addAll(validation.errors)
        }
      }

      return if (errors.isNotEmpty()) {
        Validation.failure(errors)
      } else {
        Validation.success(MetaDataSection(mappings))
      }
    }
  }
}

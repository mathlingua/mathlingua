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

import mathlingua.chalktalk.phase1.ast.*
import mathlingua.common.ParseError
import mathlingua.common.Validation
import mathlingua.textalk.ExpressionNode
import mathlingua.textalk.newTexTalkLexer
import mathlingua.textalk.newTexTalkParser

private data class ValidationPair<T>(
  val matches: (node: ChalkTalkNode) -> Boolean,
  val validate: (node: ChalkTalkNode) -> Validation<T>
)

private val CLAUSE_VALIDATORS = listOf(
  ValidationPair(
    AbstractionNode.Companion::isAbstraction,
    AbstractionNode.Companion::validate
  ),
  ValidationPair(
    AggregateNode.Companion::isAggregate,
    AggregateNode.Companion::validate
  ),
  ValidationPair(
    TupleNode.Companion::isTuple,
    TupleNode.Companion::validate
  ),
  ValidationPair(
    AssignmentNode.Companion::isAssignment,
    AssignmentNode.Companion::validate
  ),
  ValidationPair(
    Identifier.Companion::isIdentifier,
    Identifier.Companion::validate
  ),
  ValidationPair(
    Statement.Companion::isStatement,
    Statement.Companion::validate
  ),
  ValidationPair(
    Text.Companion::isText,
    Text.Companion::validate
  ),
  ValidationPair(
    ForGroup.Companion::isForGroup,
    ForGroup.Companion::validate
  ),
  ValidationPair(
    ExistsGroup.Companion::isExistsGroup,
    ExistsGroup.Companion::validate
  ),
  ValidationPair(
    NotGroup.Companion::isNotGroup,
    NotGroup.Companion::validate
  ),
  ValidationPair(
    OrGroup.Companion::isOrGroup,
    OrGroup.Companion::validate
  ),
  ValidationPair(
    IfGroup.Companion::isIfGroup,
    IfGroup.Companion::validate
  ),
  ValidationPair(
    IffGroup.Companion::isIffGroup,
    IffGroup.Companion::validate
  )
)

sealed class Clause : Phase2Node {
  companion object {
    fun validate(rawNode: ChalkTalkNode): Validation<Clause> {
      val node = rawNode.resolve()

      for (pair in CLAUSE_VALIDATORS) {
        if (pair.matches(node)) {
          val validation = pair.validate(node)
          return if (validation.isSuccessful) {
            return Validation.success(validation.value!!)
          } else {
            Validation.failure(validation.errors)
          }
        }
      }

      return Validation.failure(
        listOf(
          ParseError(
            "Expected a Target but found " + node.javaClass.simpleName,
            AstUtils.getRow(node), AstUtils.getColumn(node)
          )
        )
      )
    }
  }
}

sealed class Target : Clause()

data class AbstractionNode(val abstraction: Abstraction) : Target() {
  override fun forEach(fn: (node: Phase2Node) -> Unit) {
  }

  override fun toCode(isArg: Boolean, indent: Int): String {
    return toCode(isArg, indent, abstraction)
  }

  companion object {
    fun isAbstraction(node: ChalkTalkNode): Boolean {
      return node is Abstraction
    }

    fun validate(node: ChalkTalkNode): Validation<AbstractionNode> {
      return validateWrappedNode(node,
        "AbstractionNode",
        { it as? Abstraction },
        { AbstractionNode(it) })
    }
  }
}

data class AggregateNode(val aggregate: Aggregate) : Target() {
  override fun forEach(fn: (node: Phase2Node) -> Unit) {
  }

  override fun toCode(isArg: Boolean, indent: Int): String {
    return toCode(isArg, indent, aggregate)
  }

  companion object {
    fun isAggregate(node: ChalkTalkNode): Boolean {
      return node is Aggregate
    }

    fun validate(node: ChalkTalkNode): Validation<AggregateNode> {
      return validateWrappedNode(node,
        "AggregateNode",
        { it as? Aggregate },
        { AggregateNode(it) })
    }
  }
}

data class TupleNode(val tuple: Tuple) : Target() {
  override fun forEach(fn: (node: Phase2Node) -> Unit) {
  }

  override fun toCode(isArg: Boolean, indent: Int): String {
    return toCode(isArg, indent, tuple)
  }

  companion object {
    fun isTuple(node: ChalkTalkNode): Boolean {
      return node is Tuple
    }

    fun validate(node: ChalkTalkNode): Validation<TupleNode> {
      return validateWrappedNode(node,
        "TupleNode",
        { it as? Tuple },
        { TupleNode(it) })
    }
  }
}

data class AssignmentNode(val assignment: Assignment) : Target() {
  override fun forEach(fn: (node: Phase2Node) -> Unit) {
  }

  override fun toCode(isArg: Boolean, indent: Int): String {
    return toCode(isArg, indent, assignment)
  }

  companion object {
    fun isAssignment(node: ChalkTalkNode): Boolean {
      return node is Assignment
    }

    fun validate(node: ChalkTalkNode): Validation<AssignmentNode> {
      return validateWrappedNode(
        node,
        "AssignmentNode",
        { it as? Assignment },
        ::AssignmentNode
      )
    }
  }
}

data class MappingNode(val mapping: Mapping) : Phase2Node {
  override fun forEach(fn: (node: Phase2Node) -> Unit) {
  }

  override fun toCode(isArg: Boolean, indent: Int): String {
    return toCode(isArg, indent, mapping)
  }

  companion object {
    fun isMapping(node: ChalkTalkNode): Boolean {
      return node is Mapping
    }

    fun validate(node: ChalkTalkNode): Validation<MappingNode> {
      return validateWrappedNode(
        node,
        "MappingNode",
        { it as? Mapping },
        ::MappingNode
      )
    }
  }
}

data class Identifier(val name: String) : Target() {
  override fun forEach(fn: (node: Phase2Node) -> Unit) {
  }

  override fun toCode(isArg: Boolean, indent: Int): String {
    return indentedString(isArg, indent, name)
  }

  companion object {

    fun isIdentifier(node: ChalkTalkNode): Boolean {
      return node is ChalkTalkToken && node.type === ChalkTalkTokenType.Name
    }

    fun validate(rawNode: ChalkTalkNode): Validation<Identifier> {
      val node = rawNode.resolve()

      val errors = ArrayList<ParseError>()
      if (node !is ChalkTalkToken) {
        errors.add(
          ParseError(
            "Cannot convert a " +
              node.javaClass.simpleName + " to a ChalkTalkToken",
            AstUtils.getRow(node), AstUtils.getColumn(node)
          )
        )
        return Validation.failure(errors)
      }

      val (text, type, row, column) = node
      if (type !== ChalkTalkTokenType.Name) {
        errors.add(
          ParseError(
            "A token of type $type is not an identifier",
            row, column
          )
        )
        return Validation.failure(errors)
      }

      return Validation.success(Identifier(text))
    }
  }
}

data class Statement(
  val text: String,
  val texTalkRoot: Validation<ExpressionNode>
) : Clause() {
  override fun forEach(fn: (node: Phase2Node) -> Unit) {
  }

  override fun toCode(isArg: Boolean, indent: Int): String {
    return indentedString(isArg, indent, "'$text'")
  }

  companion object {

    fun isStatement(node: ChalkTalkNode): Boolean {
      return node is ChalkTalkToken && node.type === ChalkTalkTokenType.Statement
    }

    fun validate(node: ChalkTalkNode): Validation<Statement> {
      var node = node
      node = node.resolve()

      val errors = ArrayList<ParseError>()
      if (node !is ChalkTalkToken) {
        errors.add(
          ParseError(
            "Cannot convert a " +
              node.javaClass.simpleName + " to a ChalkTalkToken",
            AstUtils.getRow(node), AstUtils.getColumn(node)
          )
        )
      }

      val (rawText, type, row, column) = node as ChalkTalkToken
      if (type !== ChalkTalkTokenType.Statement) {
        errors.add(
          ParseError(
            "Cannot convert a " + node.toCode() + " to a Statement",
            row, column
          )
        )
        return Validation.failure(errors)
      }

      // the text is of the form '...'
      // so the open and closing ' need to be trimmed
      val text = rawText.substring(1, rawText.length - 1)

      val texTalkErrors = ArrayList<ParseError>()

      val lexer = newTexTalkLexer(text)
      texTalkErrors.addAll(lexer.errors)

      val parser = newTexTalkParser()
      val result = parser.parse(lexer)
      texTalkErrors.addAll(result.errors)

      val validation = if (texTalkErrors.isEmpty())
        Validation.success(result.root)
      else
        Validation.failure(texTalkErrors)

      return Validation.success(Statement(text, validation))
    }
  }
}

data class Text(val text: String) : Clause() {
  override fun forEach(fn: (node: Phase2Node) -> Unit) {
  }

  override fun toCode(isArg: Boolean, indent: Int): String {
    return indentedString(isArg, indent, text)
  }

  companion object {

    fun isText(node: ChalkTalkNode): Boolean {
      return node is ChalkTalkToken && node.type === ChalkTalkTokenType.String
    }

    fun validate(node: ChalkTalkNode): Validation<Text> {
      var node = node
      node = node.resolve()

      val errors = ArrayList<ParseError>()
      if (node !is ChalkTalkToken) {
        errors.add(
          ParseError(
            "Cannot convert a " +
              node.javaClass.simpleName + " to a ChalkTalkToken",
            AstUtils.getRow(node), AstUtils.getColumn(node)
          )
        )
      }

      val (text, type, row, column) = node as ChalkTalkToken
      if (type !== ChalkTalkTokenType.String) {
        errors.add(
          ParseError(
            "Cannot convert a " + node.toCode() + " to Text",
            row, column
          )
        )
        return Validation.failure(errors)
      }

      return Validation.success(Text(text))
    }
  }
}

data class ExistsGroup(
  val existsSection: ExistsSection,
  val suchThatSection: SuchThatSection
) : Clause() {
  override fun forEach(fn: (node: Phase2Node) -> Unit) {
    fn(existsSection)
    fn(suchThatSection)
  }

  override fun toCode(isArg: Boolean, indent: Int): String {
    return toCode(isArg, indent, existsSection, suchThatSection)
  }

  companion object {

    fun isExistsGroup(node: ChalkTalkNode): Boolean {
      return firstSectionMatchesName(node, "exists")
    }

    fun validate(node: ChalkTalkNode): Validation<ExistsGroup> {
      return validateDoubleSectionGroup(
        node,
        "exists",
        ExistsSection.Companion::validate,
        "suchThat",
        SuchThatSection.Companion::validate,
        ::ExistsGroup
      )
    }
  }
}

data class IfGroup(
  val ifSection: IfSection,
  val thenSection: ThenSection
) : Clause() {
  override fun forEach(fn: (node: Phase2Node) -> Unit) {
    fn(ifSection)
    fn(thenSection)
  }

  override fun toCode(isArg: Boolean, indent: Int): String {
    return toCode(isArg, indent, ifSection, thenSection)
  }

  companion object {

    fun isIfGroup(node: ChalkTalkNode): Boolean {
      return firstSectionMatchesName(node, "if")
    }

    fun validate(node: ChalkTalkNode): Validation<IfGroup> {
      return validateDoubleSectionGroup(
        node,
        "if",
        IfSection.Companion::validate,
        "then",
        ThenSection.Companion::validate,
        ::IfGroup
      )
    }
  }
}

data class IffGroup(
  val iffSection: IffSection,
  val thenSection: ThenSection
) : Clause() {
  override fun forEach(fn: (node: Phase2Node) -> Unit) {
    fn(iffSection)
    fn(thenSection)
  }

  override fun toCode(isArg: Boolean, indent: Int): String {
    return toCode(isArg, indent, iffSection, thenSection)
  }

  companion object {

    fun isIffGroup(node: ChalkTalkNode): Boolean {
      return firstSectionMatchesName(node, "iff")
    }

    fun validate(node: ChalkTalkNode): Validation<IffGroup> {
      return validateDoubleSectionGroup(
        node,
        "iff",
        IffSection.Companion::validate,
        "then",
        ThenSection.Companion::validate,
        ::IffGroup
      )
    }
  }
}


data class ForGroup(
  val forSection: ForSection,
  val whereSection: WhereSection?,
  val thenSection: ThenSection
) : Clause() {
  override fun forEach(fn: (node: Phase2Node) -> Unit) {
    fn(forSection)
    if (whereSection != null) {
      fn(whereSection)
    }
    fn(thenSection)
  }

  override fun toCode(isArg: Boolean, indent: Int): String {
    return toCode(isArg, indent, forSection, whereSection, thenSection)
  }

  companion object {

    fun isForGroup(node: ChalkTalkNode): Boolean {
      return firstSectionMatchesName(node, "for")
    }

    fun validate(rawNode: ChalkTalkNode): Validation<ForGroup> {
      val node = rawNode.resolve()

      val errors = ArrayList<ParseError>()
      if (node !is Group) {
        errors.add(
          ParseError(
            "Expected a Group but found " + node.javaClass.simpleName,
            AstUtils.getRow(node), AstUtils.getColumn(node)
          )
        )
        return Validation.failure(errors)
      }

      val (sections) = node

      val sectionMap: Map<String, Section>
      try {
        sectionMap = SectionIdentifier.identifySections(
          sections,
          "for", "where?", "then"
        )
      } catch (e: ParseError) {
        errors.add(ParseError(e.message!!, e.row, e.column))
        return Validation.failure(errors)
      }

      var forSection: ForSection? = null
      val forNode = sectionMap["for"]
      val forEvaluation = ForSection.validate(forNode!!)
      if (forEvaluation.isSuccessful) {
        forSection = forEvaluation.value
      } else {
        errors.addAll(forEvaluation.errors)
      }

      var whereSection: WhereSection? = null
      if (sectionMap.containsKey("where")) {
        val where = sectionMap["where"]
        val whereValidation = WhereSection.validate(where!!)
        if (whereValidation.isSuccessful) {
          whereSection = whereValidation.value!!
        } else {
          errors.addAll(whereValidation.errors)
        }
      }

      var thenSection: ThenSection? = null
      val then = sectionMap["then"]
      val thenValidation = ThenSection.validate(then!!)
      if (thenValidation.isSuccessful) {
        thenSection = thenValidation.value
      } else {
        errors.addAll(thenValidation.errors)
      }

      return if (!errors.isEmpty()) {
        Validation.failure(errors)
      } else Validation.success(ForGroup(forSection!!, whereSection, thenSection!!))

    }
  }
}

data class NotGroup(val notSection: NotSection) : Clause() {
  override fun forEach(fn: (node: Phase2Node) -> Unit) {
    fn(notSection)
  }

  override fun toCode(isArg: Boolean, indent: Int): String {
    return notSection.toCode(isArg, indent)
  }

  companion object {
    fun isNotGroup(node: ChalkTalkNode): Boolean {
      return firstSectionMatchesName(node, "not")
    }

    fun validate(node: ChalkTalkNode): Validation<NotGroup> {
      return validateSingleSectionGroup(
        node, "not", ::NotGroup,
        NotSection.Companion::validate
      )
    }
  }
}

data class OrGroup(val orSection: OrSection) : Clause() {
  override fun forEach(fn: (node: Phase2Node) -> Unit) {
    fn(orSection)
  }

  override fun toCode(isArg: Boolean, indent: Int): String {
    return orSection.toCode(isArg, indent)
  }

  companion object {
    fun isOrGroup(node: ChalkTalkNode): Boolean {
      return firstSectionMatchesName(node, "or")
    }

    fun validate(node: ChalkTalkNode): Validation<OrGroup> {
      return validateSingleSectionGroup(
        node, "or", ::OrGroup,
        OrSection.Companion::validate
      )
    }
  }
}

fun firstSectionMatchesName(node: ChalkTalkNode, name: String): Boolean {
  if (node !is Group) {
    return false
  }

  val (sections) = node
  return if (sections.isEmpty()) {
    false
  } else sections[0].name.text == name
}

fun <G, S> validateSingleSectionGroup(
  node: ChalkTalkNode,
  sectionName: String,
  buildGroup: (sect: S) -> G,
  validateSection: (section: Section) -> Validation<S>
): Validation<G> {
  var node = node
  node = node.resolve()

  val errors = ArrayList<ParseError>()
  if (node !is mathlingua.chalktalk.phase1.ast.Group) {
    errors.add(
      ParseError(
        "Expected a Group but found " + node.javaClass.simpleName,
        AstUtils.getRow(node), AstUtils.getColumn(node)
      )
    )
    return Validation.failure(errors)
  }

  val (sections) = node
  val sectionMap: Map<String, Section>
  try {
    sectionMap = SectionIdentifier.identifySections(
      sections,
      sectionName
    )
  } catch (e: ParseError) {
    errors.add(ParseError(e.message!!, e.row, e.column))
    return Validation.failure(errors)
  }

  var section: S? = null
  val sect = sectionMap[sectionName]
  val validation = validateSection(sect!!)
  if (validation.isSuccessful) {
    section = validation.value
  } else {
    errors.addAll(validation.errors)
  }

  return if (errors.isNotEmpty()) {
    Validation.failure(errors)
  } else Validation.success(buildGroup(section!!))
}

private fun <G, S1, S2> validateDoubleSectionGroup(
  rawNode: ChalkTalkNode,
  section1Name: String,
  validateSection1: (section: Section) -> Validation<S1>,
  section2Name: String,
  validateSection2: (section: Section) -> Validation<S2>,
  buildGroup: (sect1: S1, sect2: S2) -> G
): Validation<G> {
  val node = rawNode.resolve()

  val errors = ArrayList<ParseError>()
  if (node !is Group) {
    errors.add(
      ParseError(
        "Expected a Group but found a " + node.javaClass.simpleName,
        AstUtils.getRow(node), AstUtils.getColumn(node)
      )
    )
    return Validation.failure(errors)
  }

  val (sections) = node

  val sectionMap: Map<String, Section>
  try {
    sectionMap = SectionIdentifier.identifySections(
      sections, section1Name, section2Name
    )
  } catch (e: ParseError) {
    errors.add(ParseError(e.message!!, e.row, e.column))
    return Validation.failure(errors)
  }

  var section1: S1? = null
  val sect1 = sectionMap[section1Name]
  val section1Validation = validateSection1(sect1!!)
  if (section1Validation.isSuccessful) {
    section1 = section1Validation.value
  } else {
    errors.addAll(section1Validation.errors)
  }

  var section2: S2? = null
  val sect2 = sectionMap[section2Name]
  val section2Validation = validateSection2(sect2!!)
  if (section2Validation.isSuccessful) {
    section2 = section2Validation.value
  } else {
    errors.addAll(section2Validation.errors)
  }

  return if (!errors.isEmpty()) {
    Validation.failure(errors)
  } else Validation.success(buildGroup(section1!!, section2!!))
}

private fun <Wrapped, Base> validateWrappedNode(
  rawNode: ChalkTalkNode,
  expectedType: String,
  checkType: (node: ChalkTalkNode) -> Base?,
  build: (base: Base) -> Wrapped
): Validation<Wrapped> {
  val node = rawNode.resolve()

  val base = checkType(node)
  if (base == null) {
    return Validation.failure(
      listOf(
        ParseError(
          "Cannot convert a " +
            node.javaClass.simpleName + " to an $expectedType",
          AstUtils.getRow(node), AstUtils.getColumn(node)
        )
      )
    )
  }

  return Validation.success(build(base))
}

fun toCode(isArg: Boolean, indent: Int, chalkTalkNode: ChalkTalkNode): String {
  val builder = StringBuilder()
  builder.append(indentedString(isArg, indent, ""))
  builder.append(chalkTalkNode.toCode())
  return builder.toString()
}

fun toCode(isArg: Boolean, indent: Int, vararg sections: Phase2Node?): String {
  val builder = StringBuilder()
  for (i in 0 until sections.size) {
    val sect = sections[i]
    if (sect != null) {
      builder.append(sect.toCode(isArg, indent))
      if (i != sections.size - 1) {
        builder.append('\n')
      }
    }
  }
  return builder.toString()
}

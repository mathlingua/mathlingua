/*
 * Copyright 2022 Dominic Kramer
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

package mathlingua.spec

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue
import kotlin.text.Regex
import org.reflections.Reflections
import org.reflections.scanners.Scanners

class SpecificationTest {
    @Test
    fun `verify spec has no duplicate definitions`() {
        val usedNames = mutableSetOf<String>()
        for (spec in MATHLINGUA_SPECIFICATION) {
            assertFalse(
                actual = spec.name in usedNames,
                message =
                    "Expected ${spec.name} to not be in the already defined names: $usedNames")
            usedNames.add(spec.name)
        }
    }

    @Test
    fun `verify all definitions in spec are in code`() {
        val typesInCode = getAllTypesInCode().sorted().joinToString("\n")
        val typesInSpec = getAllTypesInSpec().sorted().joinToString("\n")
        assertEquals(expected = typesInSpec, actual = typesInCode)
    }

    @Test
    fun `verify all any-of types in spec align with interfaces in code`() {
        val anyOfTypesInSpec = MATHLINGUA_SPECIFICATION.filter { it.of is AnyOf }
        for (i in anyOfTypesInSpec.indices) {
            val type = anyOfTypesInSpec[i]
            val classname = type.getClassname()
            // verify that the classname of the type in the spec is an interface in the code
            assertTrue(
                actual = isInterface(classname), message = "Expected $classname to be an interface")
            // get all the direct implements of the interface in the code
            val implementors = getDirectAstImplementorsOf(classname)
            val anyOf = type.of as AnyOf
            // assert the items specified in the union in the spec are exactly the
            // implementors of the interface in the code
            val nameToClassname = mutableMapOf<String, String>()
            for (item in MATHLINGUA_SPECIFICATION) {
                nameToClassname[item.name] = item.getClassname()
            }
            assertEquals(
                // Statement[...] and Text[...] forms should be replaced with Statement and Text
                expected =
                    anyOf
                        .of
                        .asSequence()
                        .mapNotNull {
                            when (it) {
                                is Text -> {
                                    "Text".addAstPackagePrefix()
                                }
                                is Statement -> {
                                    "Statement".addAstPackagePrefix()
                                }
                                else -> {
                                    nameToClassname[it.toCode()]
                                        ?.replace(Regex("\\[.*\\]"), "")
                                        ?.addAstPackagePrefix()
                                }
                            }
                        }
                        .toSet()
                        .toList()
                        .sorted()
                        .joinToString("\n"),
                actual = implementors.toSet().toList().sorted().joinToString("\n"),
                message = "implementors of $classname")
        }
    }

    @Test
    fun `verify all interfaces in code align with any-of types in spec`() {
        val allInterfacesInCode = getAllTypesInCode().filter { isInterface(it) }.sorted()
        val anyOfTypesInSpec =
            MATHLINGUA_SPECIFICATION.filter { it.of is AnyOf }.map { it.getClassname() }.sorted()
        assertEquals(expected = anyOfTypesInSpec, actual = allInterfacesInCode)
    }

    @Test
    fun `verify all groups match the spec`() {
        // TODO: implement this test that verifies that all groups have the correct sections
    }
}

private const val AST_PACKAGE = "mathlingua.lib.frontend.ast"

/**
 * Adds the ast package prefix for a simple classname and does nothing for a fully qualified
 * classname
 */
private fun String.addAstPackagePrefix() =
    if (this.startsWith(AST_PACKAGE)) {
        this
    } else {
        "${AST_PACKAGE}.${this}"
    }

/**
 * Removes the ast package prefix from the given fully qualified classname and does nothing for a
 * simple classname.
 */
private fun String.removeAstPackagePrefix() =
    if (this.startsWith(AST_PACKAGE)) {
        // an additional character is removed to account
        // for the period after the package prefix
        this.substring(AST_PACKAGE.length + 1)
    } else {
        this
    }

/** Returns the fully qualified name of all the classes and interfaces in the ast package. */
private fun getAllTypesInCode(): List<String> {
    val reflections = Reflections(AST_PACKAGE, Scanners.SubTypes.filterResultsBy { true })
    return reflections.getAll(Scanners.SubTypes).filter {
        // only find mathlingua types
        it.contains(AST_PACKAGE) &&
            // ignore types associated with tests (tests make a class
            // for each test of the form <test-class>$<test-name>)
            !it.contains("$") &&
            !it.endsWith("Test") &&
            !it.endsWith("Kt") &&
            it != "mathlingua.lib.frontend.ast.ChalkTalkNode" &&
            it != "mathlingua.lib.frontend.ast.TexTalkNode" &&
            it != "mathlingua.lib.frontend.ast.CommonNode" &&
            it != "mathlingua.lib.frontend.ast.NodeLexerToken" &&
            it != "mathlingua.lib.frontend.ast.HasMetaData" &&
            it != "mathlingua.lib.frontend.ast.BeginGroup" &&
            it != "mathlingua.lib.frontend.ast.EndGroup" &&
            it != "mathlingua.lib.frontend.ast.BeginSection" &&
            it != "mathlingua.lib.frontend.ast.EndSection" &&
            it != "mathlingua.lib.frontend.ast.BeginArgument" &&
            it != "mathlingua.lib.frontend.ast.EndArgument" &&
            it != "mathlingua.lib.frontend.ast.Group" &&
            it != "mathlingua.lib.frontend.ast.TexTalkToken" &&
            it != "mathlingua.lib.frontend.ast.TexTalkTokenType" &&
            it != "mathlingua.lib.frontend.ast.ToCode" &&
            it != "mathlingua.lib.frontend.ast.TexTalkNodeOrToken" &&
            it != "mathlingua.lib.frontend.ast.Section"
    }
}

private fun DefinitionOf.getClassname() =
    if (this.of is Group) {
        this.of.classname
    } else if (this.of is Text) {
        "Text"
    } else
        if (this.of is Statement) {
                "Statement"
            } else {
                this.name
            }
            .addAstPackagePrefix()

private fun getAllTypesInSpec(): List<String> {
    val result = mutableSetOf<String>()
    for (def in MATHLINGUA_SPECIFICATION) {
        result.add(def.getClassname().addAstPackagePrefix())
        if (def.of is Group) {
            result.addAll(def.of.of.map { it.classname.addAstPackagePrefix() })
        }
    }
    return result.toList().sorted()
}

private fun isInterface(classname: String) =
    try {
        ClassLoader.getSystemClassLoader().loadClass(classname).isInterface
    } catch (e: ClassNotFoundException) {
        false
    }

private fun getDirectAstImplementorsOf(classname: String): Set<String> {
    val allTypes = getAllTypesInCode()
    val loader = ClassLoader.getSystemClassLoader()
    return allTypes
        .filter {
            loader.loadClass(it).interfaces.map { intf -> intf.name }.toSet().contains(classname)
        }
        .toSet()
}

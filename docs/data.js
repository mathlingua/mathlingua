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

window.MATHLINGUA_DATA = window.MATHLINGUA_DATA || [
{
  "text": "[\\set]\nDefines: X\nmeans:\n. \"A well defined collection of objects\"\nMetadata:\n. reference = \"source: @AATA; page: 3\"",
  "keywords": ["set", "x", "a", "well", "defined", "collection", "of", "objects", "reference", "=", "source", "aata", "page", "3"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\set"
},
{
  "text": "[a \\in X]\nRepresents:\nassuming: 'X is \\set'\nthat:\n. \"$a$ is an element of $X$\"\nMetadata:\n. reference = \"source: @AATA; page: 4\"",
  "keywords": ["a", "in", "x", "represents", "is", "set", "an", "element", "of", "reference", "=", "source", "aata", "page", "4"],
  "href": "null",
  "mobileHref": "null",
  "signature": null
},
{
  "text": "[A \\subset B]\nRepresents:\nassuming: 'A, B is \\set'\nthat:\n. for: a\n  where: 'a \\in A'\n  then: 'a \\in B'\nMetadata:\n. reference = \"source: @AATA; page: 4\"",
  "keywords": ["a", "subset", "b", "represents", "is", "set", "in", "reference", "=", "source", "aata", "page", "4"],
  "href": "null",
  "mobileHref": "null",
  "signature": null
},
{
  "text": "[x \\neq y]\nRepresents:\nthat: 'x != y'",
  "keywords": ["x", "neq", "y", "represents", "!="],
  "href": "null",
  "mobileHref": "null",
  "signature": null
},
{
  "text": "[A \\proper.subset B]\nRepresents:\nassuming: 'A, B is \\set'\nthat:\n. 'A \\subset B'\n. 'A \\neq B'\nMetadata:\n. reference = \"source: @AATA; page: 4\"",
  "keywords": ["a", "proper", "subset", "b", "represents", "is", "set", "neq", "reference", "=", "source", "aata", "page", "4"],
  "href": "null",
  "mobileHref": "null",
  "signature": null
},
{
  "text": "[\\empty.set]\nRefines: E\nmeans:\n. not:\n  . exists: x\n    suchThat: 'x \\in E'\nMetadata:\n. reference = \"source: @AATA; page: 4\"",
  "keywords": ["empty", "set", "e", "x", "in", "reference", "=", "source", "aata", "page", "4"],
  "href": "null",
  "mobileHref": "null",
  "signature": null
},
{
  "text": "[A \\union B]\nDefines: C := {c}\nassuming: 'A, B is \\set'\nmeans:\n. 'C is \\set'\n. or:\n  . 'c \\in A'\n  . 'c \\in B'\nMetadata:\n. reference = \"source: @AATA; page: 4\"",
  "keywords": ["a", "union", "b", "c", "=", "is", "set", "in", "reference", "source", "aata", "page", "4"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\union"
},
{
  "text": "[A \\intersect B]\nDefines: C := {c}\nassuming: 'A, B is \\set'\nmeans:\n. 'C is \\set'\n. 'c \\in A'\n. 'c \\in B'\nMetadata:\n. reference = \"source: @AATA; page: 4\"",
  "keywords": ["a", "intersect", "b", "c", "=", "is", "set", "in", "reference", "source", "aata", "page", "4"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\intersect"
},
{
  "text": "[\\disjoint.set]\nDefines: (A, B)\nassuming: 'A, B is \\set'\nmeans: '(A \\intersect B) = \\empty.set'\nMetadata:\n. reference = \"source: @AATA; page: 5\"",
  "keywords": ["disjoint", "set", "a", "b", "is", "intersect", "=", "empty", "reference", "source", "aata", "page", "5"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\disjoint.set"
},
{
  "text": "[x \\notin X]\nRepresents:\nassuming: 'X is \\set'\nthat:\n. not:\n  . 'x \\in X'",
  "keywords": ["x", "notin", "represents", "is", "set", "in"],
  "href": "null",
  "mobileHref": "null",
  "signature": null
},
{
  "text": "[\\complement:of{A}in{U}]\nDefines: X := {x}\nassuming: 'A, U is \\set'\nmeans:\n. 'x \\in U'\n. 'x \\notin A'\nMetadata:\n. reference = \"source: @AATA; page: 5\"",
  "keywords": ["complement", "of", "a", "in", "u", "x", "=", "is", "set", "notin", "reference", "source", "aata", "page", "5"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\complement:of{?}in{?}"
},
{
  "text": "[A \\set.difference B]\nDefines: C := {c}\nassuming: 'A, B is \\set'\nmeans:\n. 'c \\in A'\n. 'c \\notin B'\nMetadata:\n. reference = \"source: @AATA; page: 5\"",
  "keywords": ["a", "set", "difference", "b", "c", "=", "is", "in", "notin", "reference", "source", "aata", "page", "5"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\set.difference"
},
{
  "text": "[A \\set.cartesian.product B]\nDefines: C := {x,y}\nassuming: 'A, B is \\set'\nmeans:\n. 'x \\in A'\n. 'y \\in B'\nMetadata:\n. reference = \"source: @AATA; page: 6\"",
  "keywords": ["a", "set", "cartesian", "product", "b", "c", "=", "x", "y", "is", "in", "reference", "source", "aata", "page", "6"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\set.cartesian.product"
},
{
  "text": "[\\function.domain:of{f}]\nDefines: X\nassuming:\n. for: A, B\n  where: 'A, B is \\set'\n  then: 'f is \\function:on{A}to{B}'\nmeans:\n. 'X := A'\nMetadata:\n. reference = \"source: @AATA; page: 7\"",
  "keywords": ["function", "domain", "of", "f", "x", "a", "b", "is", "set", "on", "to", "=", "reference", "source", "aata", "page", "7"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\function.domain:of{?}"
},
{
  "text": "[\\function.range:of{f}]\nDefines: X\nassuming: 'f is \\function'\nmeans:\n. 'X = \\set[x]{f(x)}{f \\in \\domain{f}}'\nAlias:\n. domain = \"function.domain:of\"\nMetadata:\n. reference = \"source: @AATA; page: 7\"",
  "keywords": ["function", "range", "of", "f", "x", "is", "=", "set", "in", "domain", "reference", "source", "aata", "page", "7"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\function.range:of{?}"
},
{
  "text": "[\\surjective \\function:on{A}to{B}]\nRefines: f\nmeans:\n. for: b\n  where: 'b \\in B'\n  then:\n  . exists: a\n    suchThat:\n    . 'f(a) = b'\nMetadata:\n. reference = \"source: @AATA; page: 8\"",
  "keywords": ["surjective", "function", "on", "a", "to", "b", "f", "in", "=", "reference", "source", "aata", "page", "8"],
  "href": "null",
  "mobileHref": "null",
  "signature": null
},
{
  "text": "[\\injective \\function:on{A}to{B}]\nRefines: f\nmeans:\n. for: a1, a2\n  where: 'a1, a2 \\in A'\n  then:\n  . if: 'a1 \\neq a2'\n    then: 'f(a1) \\neq f(a2)'\nMetadata:\n. reference = \"source: @AATA; page: 8\"",
  "keywords": ["injective", "function", "on", "a", "to", "b", "f", "a1", "a2", "in", "neq", "reference", "=", "source", "aata", "page", "8"],
  "href": "null",
  "mobileHref": "null",
  "signature": null
},
{
  "text": "[\\bijective \\function]\nRefines: f\nmeans:\n. 'f \\is \\injective \\surjective \\function'\nMetadata:\n. reference = \"source: @AATA; page: 8\"",
  "keywords": ["bijective", "function", "f", "is", "injective", "surjective", "reference", "=", "source", "aata", "page", "8"],
  "href": "null",
  "mobileHref": "null",
  "signature": null
},
{
  "text": "[g \\of f]\nDefines: h(x)\nassuming:\n. for: A, B, C\n  where: 'A, B, C is \\set'\n  then:\n  . 'f is \\function:on{A}to{B}'\n  . 'g is \\function:on{B}to{C}'\nmeans:\n. 'h is \\function:on{A}to{C}'\n. 'h(x) = g(f(x))'\nMetadata:\n. reference = \"source: @AATA; page: 8\"",
  "keywords": ["g", "of", "f", "h", "x", "a", "b", "c", "is", "set", "function", "on", "to", "=", "reference", "source", "aata", "page", "8"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\of"
},
{
  "text": "[\\permutation:of{S}]\nDefines: f\nassuming: 'S is \\set'\nmeans: 'f is \\bijective \\function:on{S}to{S}'\nMetadata:\n. reference = \"source: @AATA; page: 9\"",
  "keywords": ["permutation", "of", "s", "f", "is", "set", "bijective", "function", "on", "to", "reference", "=", "source", "aata", "page", "9"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\permutation:of{?}"
},
{
  "text": "[\\identity.function:on{A}]\nDefines: f(x)\nassuming: 'A \\in \\set'\nmeans:\n. 'f is \\function:on{A}to{A}'\n. for: x\n  then: 'f(x) = x'\nMetadata:\n. reference = \"source: @AATA; page: 10\"",
  "keywords": ["identity", "function", "on", "a", "f", "x", "in", "set", "is", "to", "=", "reference", "source", "aata", "page", "10"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\identity.function:on{?}"
},
{
  "text": "[\\inverse.function:of{f}]\nDefines: g(x)\nassuming:\n. for: A, B\n  where: 'A, B is \\set'\n  then: 'f is \\function:on{A}to{B}'\nmeans:\n. 'g is \\function:on{B}to{A}'\n. '(g \\of f) = \\identity.function:on{A}'\nMetadata:\n. reference = \"source: @AATA; page: 10\"",
  "keywords": ["inverse", "function", "of", "f", "g", "x", "a", "b", "is", "set", "on", "to", "=", "identity", "reference", "source", "aata", "page", "10"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\inverse.function:of{?}"
},
{
  "text": "[\\equivalence.relation:on{X}]\nDefines: R := {a, b}\nassuming: 'X is \\set'\nmeans:\n. 'R \\subset (X \\cross X)'\n. for: x\n  where: 'x \\in X'\n  then: '(x, x) \\in R'\n. for: x, y\n  where: 'x, y \\in X'\n  then:\n  . if: '(x, y) \\in R'\n    then: '(y, x) \\in R'\n. for: x, y, z\n  where: 'x, y, z \\in X'\n  then:\n  . if:\n    . '(x, y) \\in R'\n    . '(y, z) \\in R'\n    then:\n    . '(x, z) \\in R'\nAlias:\n. cross = \"set.cartesian.product\"\nMetadata:\n. reference = \"source: @AATA; page: 11\"",
  "keywords": ["equivalence", "relation", "on", "x", "r", "=", "a", "b", "is", "set", "subset", "cross", "in", "y", "z", "reference", "source", "aata", "page", "11"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\equivalence.relation:on{?}"
},
{
  "text": "[\\integer]\nDefines: n\nmeans: \"$n$ is a whole number\"",
  "keywords": ["integer", "n", "is", "a", "whole", "number"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\integer"
},
{
  "text": "[\\positive.integer]\nDefines: n\nmeans:\n. 'n is \\integer'\n. 'n > 0'",
  "keywords": ["positive", "integer", "n", "is", ">", "0"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\positive.integer"
},
{
  "text": "[\\nonzero]\nDefines: x\nmeans: 'x \\neq 0'",
  "keywords": ["nonzero", "x", "neq", "0"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\nonzero"
},
{
  "text": "[x \\binary.operation:on{G} y]\nDefines: *\nassuming: 'G is \\set'\nmeans:\n. '* is \\function:on{G \\cross G}to{G}'\nAlias:\n. cross = \"set.cartesian.product\"\nMetadata:\n. reference = \"source: @AATA; page: 33\"",
  "keywords": ["x", "binary", "operation", "on", "g", "y", "*", "is", "set", "function", "cross", "to", "reference", "=", "source", "aata", "page", "33"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\binary.operation:on{?}"
},
{
  "text": "[\\group]\nDefines: G := (X, *)\nassuming:\n. 'X is \\set'\n. '* is \\binary.operator:on{X}'\nmeans:\n. for: a, b, c\n  where: 'a, b, c \\in X'\n  then: '(a * b) * c = a * (b * c)'\n. exists: e\n  suchThat:\n  . 'e \\in X'\n  . for: a\n    where: 'x \\in X'\n    then: 'a * e = e * a = a'\n. for: a\n  where: 'a \\in X'\n  then:\n  . exists: b\n    suchThat:\n    . 'b \\in X'\n    . 'a * b = b * a = e'\nMetadata:\n. reference = \"source: @AATA; page: 34\"",
  "keywords": ["group", "g", "=", "x", "*", "is", "set", "binary", "operator", "on", "a", "b", "c", "in", "e", "reference", "source", "aata", "page", "34"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\group"
},
{
  "text": "[\\abelian \\group]\nRefines: G := (X, *)\nmeans:\n. for: a, b\n  where: 'a, b \\in X'\n  then:\n  . 'a * b = b * a'\nMetadata:\n. reference = \"source: @AATA; page: 34\"",
  "keywords": ["abelian", "group", "g", "=", "x", "*", "a", "b", "in", "reference", "source", "aata", "page", "34"],
  "href": "null",
  "mobileHref": "null",
  "signature": null
},
{
  "text": "[\\matrix{m, n}:over{F}]\nDefines: M\nassuming:\n. 'F is \\set'\n. 'm, n is \\positive \\integer'\nmeans:\n. \"an $m$ by $n$ grid of elements\"",
  "keywords": ["matrix", "m", "n", "over", "f", "is", "set", "positive", "integer", "an", "by", "grid", "of", "elements"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\matrix{?, ?}:over{?}"
},
{
  "text": "[\\identity \\matrix{n}]\nRefines: M(i, j)\nassuming: 'i, j is \\positive \\integer'\nmeans:\n. if: 'i \\neq j'\n  then: 'M(i, j) = 0'\n. 'M(i, i) = 1'\nMetadata:\n. reference = \"source: @AATA; page: 35\"",
  "keywords": ["identity", "matrix", "n", "m", "i", "j", "is", "positive", "integer", "neq", "=", "0", "1", "reference", "source", "aata", "page", "35"],
  "href": "null",
  "mobileHref": "null",
  "signature": null
},
{
  "text": "[A * B]\nDefines: *\nmeans:\n. if: 'A, B is \\matrix'\n  then: 'A * B is \\matrix'",
  "keywords": ["a", "*", "b", "is", "matrix"],
  "href": "null",
  "mobileHref": "null",
  "signature": null
},
{
  "text": "[\\invertible \\matrix]\nRefines: M\nmeans:\n. exists: A\n  suchThat: 'M * A = \\identity.matrix'\nMetadata:\n. reference = \"source: @AATA; page: 35\"",
  "keywords": ["invertible", "matrix", "m", "a", "*", "=", "identity", "reference", "source", "aata", "page", "35"],
  "href": "null",
  "mobileHref": "null",
  "signature": null
},
{
  "text": "[\\general.linear.group{n}:over{F}]\nDefines: X\nassuming:\n. or:\n  . 'F = \\reals'\n  . 'F = \\complexes'\n. 'n is \\integer'\nmeans:\n. 'X = \\set[M]{M}{M is \\invertible.matrix}'\nMetadata:\n. reference = \"source: @AATA; page: 35\"",
  "keywords": ["general", "linear", "group", "n", "over", "f", "x", "=", "reals", "complexes", "is", "integer", "set", "m", "invertible", "matrix", "reference", "source", "aata", "page", "35"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\general.linear.group{?}:over{?}"
},
{
  "text": "[\\finite \\set]\nRefines: X\nmeans:\n. \"$X$ has a finite number of elements\"\nMetadata:\n. reference = \"source: @AATA; page: 36\"",
  "keywords": ["finite", "set", "x", "has", "a", "number", "of", "elements", "reference", "=", "source", "aata", "page", "36"],
  "href": "null",
  "mobileHref": "null",
  "signature": null
},
{
  "text": "[\\infinite \\set]\nRefines: X\nmeans:\n. not:\n  . 'X is \\finite \\set'\nMetadata:\n. reference = \"source: @AATA; page: 36\"",
  "keywords": ["infinite", "set", "x", "is", "finite", "reference", "=", "source", "aata", "page", "36"],
  "href": "null",
  "mobileHref": "null",
  "signature": null
},
{
  "text": "[\\set.cardinality:of{X}]\nDefines: n\nassuming: 'X is \\set'\nmeans: \"$n$ is the number of elements in $X$\"",
  "keywords": ["set", "cardinality", "of", "x", "n", "is", "the", "number", "elements", "in"],
  "href": "null",
  "mobileHref": "null",
  "signature": "\\set.cardinality:of{?}"
},
{
  "text": "[\\finite \\group]\nRefines: G := (X, *)\nmeans: 'X \\is \\finite \\set'\nMetadata:\n. reference = \"source: @AATA; page: 36\"",
  "keywords": ["finite", "group", "g", "=", "x", "*", "is", "set", "reference", "source", "aata", "page", "36"],
  "href": "null",
  "mobileHref": "null",
  "signature": null
},
{
  "text": "[\\infinite \\group]\nRefines: G := (X, *)\nmeans: 'X \\is \\infinite \\set'\nMetadata:\n. reference = \"source: @AATA; page: 36\"",
  "keywords": ["infinite", "group", "g", "=", "x", "*", "is", "set", "reference", "source", "aata", "page", "36"],
  "href": "null",
  "mobileHref": "null",
  "signature": null
},
{
  "text": "[\\group.order:of{G := (X, *)}]\nDefines: n\nassuming: 'G is \\group'\nmeans: 'n = \\set.cardinality:of{X}'\nMetadata:\n. reference = \"source: @AATA; page: 36\"",
  "keywords": ["group", "order", "of", "g", "=", "x", "*", "n", "is", "set", "cardinality", "reference", "source", "aata", "page", "36"],
  "href": "null",
  "mobileHref": "null",
  "signature": null
},
];

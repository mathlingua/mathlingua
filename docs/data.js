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
  "text": "[AATA]\nSource:\n. name = \"Abstract Algebra Theory and Applications\"\n. author = \"Thomas W. Judson\"\n. date = \"July 10, 2019\"\n. url = \"http://abstract.ups.edu/download/aata-20190710-print.pdf\"\n. offset = \"14\"",
  "keywords": ["name", "=", "abstract", "algebra", "theory", "and", "applications", "author", "thomas", "w", "judson", "date", "july", "10", "2019", "url", "http", "//abstract", "ups", "edu/download/aata-20190710-print", "pdf", "offset", "14"],
  "href": "null",
  "mobileHref": "null",
},
{
  "text": "[TrenchRealAnalysis]\nSource:\n. name = \"Introduction to Real Analysis\"\n. author = \"William F. Trench\"\n. date = \"December 2013\"\n. url = \"https://digitalcommons.trinity.edu/cgi/viewcontent.cgi?article=1006&context=mono\"\n. offset = \"9\"",
  "keywords": ["name", "=", "introduction", "to", "real", "analysis", "author", "william", "f", "trench", "date", "december", "2013", "url", "https", "//digitalcommons", "trinity", "edu/cgi/viewcontent", "cgi?article=1006&context=mono", "offset", "9"],
  "href": "null",
  "mobileHref": "null",
},
{
  "text": "[\\set]\nDefines: X\nmeans:\n. \"A well defined collection of objects\"\nMetadata:\n. reference = \"source: @AATA; page: 3\"",
  "keywords": ["set", "x", "a", "well", "defined", "collection", "of", "objects", "reference", "=", "source", "aata", "page", "3"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=17",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page17",
},
{
  "text": "[a \\in X]\nRepresents:\nassuming: 'X is \\set'\nthat:\n. \"$a$ is an element of $X$\"\nMetadata:\n. reference = \"source: @AATA; page: 4\"",
  "keywords": ["a", "in", "x", "represents", "is", "set", "an", "element", "of", "reference", "=", "source", "aata", "page", "4"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=18",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page18",
},
{
  "text": "[A \\subset B]\nRepresents:\nassuming: 'A, B is \\set'\nthat:\n. for: a\n  where: 'a \\in A'\n  then: 'a \\in B'\nMetadata:\n. reference = \"source: @AATA; page: 4\"",
  "keywords": ["a", "subset", "b", "represents", "is", "set", "in", "reference", "=", "source", "aata", "page", "4"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=18",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page18",
},
{
  "text": "[x \\neq y]\nRepresents:\nthat: 'x != y'",
  "keywords": ["x", "neq", "y", "represents", "!="],
  "href": "null",
  "mobileHref": "null",
},
{
  "text": "[A \\proper.subset B]\nRepresents:\nassuming: 'A, B is \\set'\nthat:\n. 'A \\subset B'\n. 'A \\neq B'\nMetadata:\n. reference = \"source: @AATA; page: 4\"",
  "keywords": ["a", "proper", "subset", "b", "represents", "is", "set", "neq", "reference", "=", "source", "aata", "page", "4"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=18",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page18",
},
{
  "text": "[\\empty.set]\nDefines: E\nmeans:\n. 'E is \\set'\n. not:\n  . exists: x\n    suchThat: 'x \\in E'\nMetadata:\n. reference = \"source: @AATA; page: 4\"",
  "keywords": ["empty", "set", "e", "is", "x", "in", "reference", "=", "source", "aata", "page", "4"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=18",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page18",
},
{
  "text": "[A \\union B]\nDefines: C := {c}\nassuming: 'A, B is \\set'\nmeans:\n. 'C is \\set'\n. or:\n  . 'c \\in A'\n  . 'c \\in B'\nMetadata:\n. reference = \"source: @AATA; page: 4\"",
  "keywords": ["a", "union", "b", "c", "=", "is", "set", "in", "reference", "source", "aata", "page", "4"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=18",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page18",
},
{
  "text": "[A \\intersect B]\nDefines: C := {c}\nassuming: 'A, B is \\set'\nmeans:\n. 'C is \\set'\n. 'c \\in A'\n. 'c \\in B'\nMetadata:\n. reference = \"source: @AATA; page: 4\"",
  "keywords": ["a", "intersect", "b", "c", "=", "is", "set", "in", "reference", "source", "aata", "page", "4"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=18",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page18",
},
{
  "text": "[\\disjoint.set]\nDefines: (A, B)\nassuming: 'A, B is \\set'\nmeans: '(A \\intersect B) = \\empty.set'\nMetadata:\n. reference = \"source: @AATA; page: 5\"",
  "keywords": ["disjoint", "set", "a", "b", "is", "intersect", "=", "empty", "reference", "source", "aata", "page", "5"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=19",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page19",
},
{
  "text": "[x \\notin X]\nRepresents:\nassuming: 'X is \\set'\nthat:\n. not:\n  . 'x \\in X'",
  "keywords": ["x", "notin", "represents", "is", "set", "in"],
  "href": "null",
  "mobileHref": "null",
},
{
  "text": "[\\complement:of{A}in{U}]\nDefines: X := {x}\nassuming: 'A, U is \\set'\nmeans:\n. 'x \\in U'\n. 'x \\notin A'\nMetadata:\n. reference = \"source: @AATA; page: 5\"",
  "keywords": ["complement", "of", "a", "in", "u", "x", "=", "is", "set", "notin", "reference", "source", "aata", "page", "5"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=19",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page19",
},
{
  "text": "[A \\set.difference B]\nDefines: C := {c}\nassuming: 'A, B is \\set'\nmeans:\n. 'c \\in A'\n. 'c \\notin B'\nMetadata:\n. reference = \"source: @AATA; page: 5\"",
  "keywords": ["a", "set", "difference", "b", "c", "=", "is", "in", "notin", "reference", "source", "aata", "page", "5"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=19",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page19",
},
{
  "text": "[A \\set.cartesian.product B]\nDefines: C := {x,y}\nassuming: 'A, B is \\set'\nmeans:\n. 'x \\in A'\n. 'y \\in B'\nMetadata:\n. reference = \"source: @AATA; page: 6\"",
  "keywords": ["a", "set", "cartesian", "product", "b", "c", "=", "x", "y", "is", "in", "reference", "source", "aata", "page", "6"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=20",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page20",
},
{
  "text": "[\\function.domain:of{f}]\nDefines: X\nassuming:\n. for: A, B\n  where: 'A, B is \\set'\n  then: 'f is \\function:on{A}to{B}'\nmeans:\n. 'X := A'\nMetadata:\n. reference = \"source: @AATA; page: 7\"",
  "keywords": ["function", "domain", "of", "f", "x", "a", "b", "is", "set", "on", "to", "=", "reference", "source", "aata", "page", "7"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=21",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page21",
},
{
  "text": "[\\function.range:of{f}]\nDefines: X\nassuming: 'f is \\function'\nmeans:\n. 'X = \\set[x]{f(x)}{f \\in \\domain{f}}'\nAlias:\n. domain = \"function.domain:of\"\nMetadata:\n. reference = \"source: @AATA; page: 7\"",
  "keywords": ["function", "range", "of", "f", "x", "is", "=", "set", "in", "domain", "reference", "source", "aata", "page", "7"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=21",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page21",
},
{
  "text": "[\\surjective \\function:on{A}to{B}]\nDefines: f\nmeans:\n. 'f is \\function:on{A}to{B}'\n. for: b\n  where: 'b \\in B'\n  then:\n  . exists: a\n    suchThat:\n    . 'f(a) = b'\nMetadata:\n. reference = \"source: @AATA; page: 8\"",
  "keywords": ["surjective", "function", "on", "a", "to", "b", "f", "is", "in", "=", "reference", "source", "aata", "page", "8"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=22",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page22",
},
{
  "text": "[\\injective \\function:on{A}to{B}]\nDefines: f\nmeans:\n. 'f is \\function:on{A}to{B}'\n. for: a1, a2\n  where: 'a1, a2 \\in A'\n  then:\n  . if: 'a1 \\neq a2'\n    then: 'f(a1) \\neq f(a2)'\nMetadata:\n. reference = \"source: @AATA; page: 8\"",
  "keywords": ["injective", "function", "on", "a", "to", "b", "f", "is", "a1", "a2", "in", "neq", "reference", "=", "source", "aata", "page", "8"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=22",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page22",
},
{
  "text": "[\\bijective \\function]\nDefines: f\nmeans:\n. 'f \\is \\injective \\surjective \\function'\nMetadata:\n. reference = \"source: @AATA; page: 8\"",
  "keywords": ["bijective", "function", "f", "is", "injective", "surjective", "reference", "=", "source", "aata", "page", "8"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=22",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page22",
},
{
  "text": "[g \\of f]\nDefines: h(x)\nassuming:\n. for: A, B, C\n  where: 'A, B, C is \\set'\n  then:\n  . 'f is \\function:on{A}to{B}'\n  . 'g is \\function:on{B}to{C}'\nmeans:\n. 'h is \\function:on{A}to{C}'\n. 'h(x) = g(f(x))'\nMetadata:\n. reference = \"source: @AATA; page: 8\"",
  "keywords": ["g", "of", "f", "h", "x", "a", "b", "c", "is", "set", "function", "on", "to", "=", "reference", "source", "aata", "page", "8"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=22",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page22",
},
{
  "text": "[\\permutation:of{S}]\nDefines: f\nassuming: 'S is \\set'\nmeans: 'f is \\bijective \\function:on{S}to{S}'\nMetadata:\n. reference = \"source: @AATA; page: 9\"",
  "keywords": ["permutation", "of", "s", "f", "is", "set", "bijective", "function", "on", "to", "reference", "=", "source", "aata", "page", "9"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=23",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page23",
},
{
  "text": "[\\identity.function:on{A}]\nDefines: f(x)\nassuming: 'A \\in \\set'\nmeans:\n. 'f is \\function:on{A}to{A}'\n. for: x\n  then: 'f(x) = x'\nMetadata:\n. reference = \"source: @AATA; page: 10\"",
  "keywords": ["identity", "function", "on", "a", "f", "x", "in", "set", "is", "to", "=", "reference", "source", "aata", "page", "10"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=24",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page24",
},
{
  "text": "[\\inverse.function:of{f}]\nDefines: g(x)\nassuming:\n. for: A, B\n  where: 'A, B is \\set'\n  then: 'f is \\function:on{A}to{B}'\nmeans:\n. 'g is \\function:on{B}to{A}'\n. '(g \\of f) = \\identity.function:on{A}'\nMetadata:\n. reference = \"source: @AATA; page: 10\"",
  "keywords": ["inverse", "function", "of", "f", "g", "x", "a", "b", "is", "set", "on", "to", "=", "identity", "reference", "source", "aata", "page", "10"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=24",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page24",
},
{
  "text": "[\\equivalence.relation:on{X}]\nDefines: R := {a, b}\nassuming: 'X is \\set'\nmeans:\n. 'R \\subset (X \\cross X)'\n. for: x\n  where: 'x \\in X'\n  then: '(x, x) \\in R'\n. for: x, y\n  where: 'x, y \\in X'\n  then:\n  . if: '(x, y) \\in R'\n    then: '(y, x) \\in R'\n. for: x, y, z\n  where: 'x, y, z \\in X'\n  then:\n  . if:\n    . '(x, y) \\in R'\n    . '(y, z) \\in R'\n    then:\n    . '(x, z) \\in R'\nAlias:\n. cross = \"set.cartesian.product\"\nMetadata:\n. reference = \"source: @AATA; page: 11\"",
  "keywords": ["equivalence", "relation", "on", "x", "r", "=", "a", "b", "is", "set", "subset", "cross", "in", "y", "z", "reference", "source", "aata", "page", "11"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=25",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page25",
},
{
  "text": "[\\integer]\nDefines: n\nmeans: \"$n$ is a whole number\"",
  "keywords": ["integer", "n", "is", "a", "whole", "number"],
  "href": "null",
  "mobileHref": "null",
},
{
  "text": "[\\positive.integer]\nDefines: n\nmeans:\n. 'n is \\integer'\n. 'n > 0'",
  "keywords": ["positive", "integer", "n", "is", ">", "0"],
  "href": "null",
  "mobileHref": "null",
},
{
  "text": "[\\nonzero]\nDefines: x\nmeans: 'x \\neq 0'",
  "keywords": ["nonzero", "x", "neq", "0"],
  "href": "null",
  "mobileHref": "null",
},
{
  "text": "[x \\binary.operation:on{G} y]\nDefines: *\nassuming: 'G is \\set'\nmeans:\n. '* is \\function:on{G \\cross G}to{G}'\nAlias:\n. cross = \"set.cartesian.product\"\nMetadata:\n. reference = \"source: @AATA; page: 33\"",
  "keywords": ["x", "binary", "operation", "on", "g", "y", "*", "is", "set", "function", "cross", "to", "reference", "=", "source", "aata", "page", "33"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=47",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page47",
},
{
  "text": "[\\group]\nDefines: G := (X, *, e)\nassuming:\n. 'X is \\set'\n. '* is \\binary.operator:on{X}'\nmeans:\n. for: a, b, c\n  where: 'a, b, c \\in X'\n  then: '(a * b) * c = a * (b * c)'\n. 'e \\in X'\n. for: a\n  where: 'x \\in X'\n  then: 'a * e = e * a = a'\n. for: a\n  where: 'a \\in X'\n  then:\n  . exists: b\n    suchThat:\n    . 'b \\in X'\n    . 'a * b = b * a = e'\nMetadata:\n. reference = \"source: @AATA; page: 34\"",
  "keywords": ["group", "g", "=", "x", "*", "e", "is", "set", "binary", "operator", "on", "a", "b", "c", "in", "reference", "source", "aata", "page", "34"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=48",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page48",
},
{
  "text": "[\\abelian \\group]\nDefines: G := (X, *, e)\nmeans:\n. 'G is \\group'\n. for: a, b\n  where: 'a, b \\in X'\n  then:\n  . 'a * b = b * a'\nMetadata:\n. reference = \"source: @AATA; page: 34\"",
  "keywords": ["abelian", "group", "g", "=", "x", "*", "e", "is", "a", "b", "in", "reference", "source", "aata", "page", "34"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=48",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page48",
},
{
  "text": "[\\matrix{m, n}:over{F}]\nDefines: M\nassuming:\n. 'F is \\set'\n. 'm, n is \\positive \\integer'\nmeans:\n. \"an $m$ by $n$ grid of elements\"",
  "keywords": ["matrix", "m", "n", "over", "f", "is", "set", "positive", "integer", "an", "by", "grid", "of", "elements"],
  "href": "null",
  "mobileHref": "null",
},
{
  "text": "[\\identity \\matrix{n}]\nDefines: M(i, j)\nassuming: 'i, j is \\positive \\integer'\nmeans:\n. 'M is \\matrix'\n. if: 'i \\neq j'\n  then: 'M(i, j) = 0'\n. 'M(i, i) = 1'\nMetadata:\n. reference = \"source: @AATA; page: 35\"",
  "keywords": ["identity", "matrix", "n", "m", "i", "j", "is", "positive", "integer", "neq", "=", "0", "1", "reference", "source", "aata", "page", "35"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=49",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page49",
},
{
  "text": "[A * B]\nDefines: *\nmeans:\n. if: 'A, B is \\matrix'\n  then: 'A * B is \\matrix'",
  "keywords": ["a", "*", "b", "is", "matrix"],
  "href": "null",
  "mobileHref": "null",
},
{
  "text": "[\\invertible \\matrix]\nDefines: M\nmeans:\n. 'M is \\matrix'\n. exists: A\n  suchThat: 'M * A = \\identity.matrix'\nMetadata:\n. reference = \"source: @AATA; page: 35\"",
  "keywords": ["invertible", "matrix", "m", "is", "a", "*", "=", "identity", "reference", "source", "aata", "page", "35"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=49",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page49",
},
{
  "text": "[\\general.linear.group{n}:over{F}]\nDefines: X\nassuming:\n. or:\n  . 'F = \\reals'\n  . 'F = \\complexes'\n. 'n is \\integer'\nmeans:\n. 'X = \\set[M]{M}{M is \\invertible.matrix}'\nMetadata:\n. reference = \"source: @AATA; page: 35\"",
  "keywords": ["general", "linear", "group", "n", "over", "f", "x", "=", "reals", "complexes", "is", "integer", "set", "m", "invertible", "matrix", "reference", "source", "aata", "page", "35"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=49",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page49",
},
{
  "text": "[\\finite \\set]\nDefines: X\nmeans:\n. 'X is \\set'\n. \"$X$ has a finite number of elements\"\nMetadata:\n. reference = \"source: @AATA; page: 36\"",
  "keywords": ["finite", "set", "x", "is", "has", "a", "number", "of", "elements", "reference", "=", "source", "aata", "page", "36"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=50",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page50",
},
{
  "text": "[\\infinite \\set]\nDefines: X\nmeans:\n. 'X is \\set'\n. not:\n  . 'X is \\finite \\set'\nMetadata:\n. reference = \"source: @AATA; page: 36\"",
  "keywords": ["infinite", "set", "x", "is", "finite", "reference", "=", "source", "aata", "page", "36"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=50",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page50",
},
{
  "text": "[\\set.cardinality:of{X}]\nDefines: n\nassuming: 'X is \\set'\nmeans: \"$n$ is the number of elements in $X$\"",
  "keywords": ["set", "cardinality", "of", "x", "n", "is", "the", "number", "elements", "in"],
  "href": "null",
  "mobileHref": "null",
},
{
  "text": "[\\finite \\group]\nDefines: G := (X, *, e)\nmeans:\n. 'G is \\group'\n. 'X \\is \\finite \\set'\nMetadata:\n. reference = \"source: @AATA; page: 36\"",
  "keywords": ["finite", "group", "g", "=", "x", "*", "e", "is", "set", "reference", "source", "aata", "page", "36"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=50",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page50",
},
{
  "text": "[\\infinite \\group]\nDefines: G := (X, *, e)\nmeans:\n. 'G is \\group'\n. 'X \\is \\infinite \\set'\nMetadata:\n. reference = \"source: @AATA; page: 36\"",
  "keywords": ["infinite", "group", "g", "=", "x", "*", "e", "is", "set", "reference", "source", "aata", "page", "36"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=50",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page50",
},
{
  "text": "[\\group.order:of{G := (X, *)}]\nDefines: n\nassuming: 'G is \\group'\nmeans: 'n = \\set.cardinality:of{X}'\nMetadata:\n. reference = \"source: @AATA; page: 36\"",
  "keywords": ["group", "order", "of", "g", "=", "x", "*", "n", "is", "set", "cardinality", "reference", "source", "aata", "page", "36"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=50",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page50",
},
{
  "text": "[\\nonempty \\set]\nDefines: X\nmeans:\n. 'X is \\set'\n. exists: x\n  suchThat: 'x \\in X'",
  "keywords": ["nonempty", "set", "x", "is", "in"],
  "href": "null",
  "mobileHref": "null",
},
{
  "text": "[\\ring]\nDefines: R := (X, +, *, 0)\nassuming:\n. 'X is \\nonempty \\set'\n. '+, * is \\binary.operation:on{X}'\nmeans:\n. '(X, +, 0) is \\abelian.group'\n. for: a, b, c\n  where: 'a, b, c \\in X'\n  then: '(a*b)*c = a*(b*c)'\n. for: a, b, c\n  where: 'a, b, c \\in X'\n  then:\n  . 'a*(b + c) = a*b + a*c'\n  . '(a + b)*c = a*c + b*c'\nMetadata:\n. reference = \"source: @AATA; page: 199\"",
  "keywords": ["ring", "r", "=", "x", "+", "*", "0", "is", "nonempty", "set", "binary", "operation", "on", "abelian", "group", "a", "b", "c", "in", "a*b", "*c", "a*", "b*c", "a*c", "reference", "source", "aata", "page", "199"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=213",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page213",
},
{
  "text": "[\\with.unity \\ring]\nDefines: R := (X, +, *, 0, 1)\nmeans:\n. 'R is \\ring'\n. for: x\n  where: 'x \\in X'\n  then: '1*x = x*1 = x'\nMetadata:\n. reference = \"source: @AATA; page: 199\"",
  "keywords": ["with", "unity", "ring", "r", "=", "x", "+", "*", "0", "1", "is", "in", "1*x", "x*1", "reference", "source", "aata", "page", "199"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=213",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page213",
},
{
  "text": "[\\commutative \\ring]\nDefines: R := (X, +, *, 0)\nmeans:\n. 'R is \\ring'\n. for: x, y\n  where: 'x, y \\in X'\n  then: 'x*y = y*x'\nMetadata:\n. reference = \"source: @AATA; page: 199\"",
  "keywords": ["commutative", "ring", "r", "=", "x", "+", "*", "0", "is", "y", "in", "x*y", "y*x", "reference", "source", "aata", "page", "199"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=213",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page213",
},
{
  "text": "[\\integral.domain]\nDefines: R := (X, +, *, 0, 1)\nmeans:\n. '(X, +, *, 0) is \\commutative \\ring'\n. '(X, +, *, 0, 1) is \\with.unity \\ring'\n. for: x, y\n  where: 'x, y \\in X'\n  then:\n  . if: 'x*y = 0'\n    then:\n    . or:\n      . 'x = 0'\n      . 'y = 0'\nMetadata:\n. reference = \"source: @AATA; page: 200\"",
  "keywords": ["integral", "domain", "r", "=", "x", "+", "*", "0", "1", "is", "commutative", "ring", "with", "unity", "y", "in", "x*y", "reference", "source", "aata", "page", "200"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=214",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page214",
},
{
  "text": "[\\ring.unit:in{R := (X, +, *, 0, 1)}]\nDefines: u\nassuming: 'R is \\with.identity.ring'\nmeans:\n. 'u \\neq 0'\n. exists: b\n  suchThat:\n  . 'b is \\unique'\n  . 'u*b = b*u = 1'\nMetadata:\n. reference = \"source: @AATA; page: 200\"",
  "keywords": ["ring", "unit", "in", "r", "=", "x", "+", "*", "0", "1", "u", "is", "with", "identity", "neq", "b", "unique", "u*b", "b*u", "reference", "source", "aata", "page", "200"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=214",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page214",
},
{
  "text": "[\\division \\with.identity.ring]\nDefines: R := (X, +, *, 0, 1)\nmeans:\n. 'R is \\ring'\n. for: x\n  where:\n  . 'x \\in X'\n  . 'x \\neq 0'\n  then:\n  . 'x is \\ring.unit:in{R}'\nMetadata:\n. reference = \"source: @AATA; page: 200\"",
  "keywords": ["division", "with", "identity", "ring", "r", "=", "x", "+", "*", "0", "1", "is", "in", "neq", "unit", "reference", "source", "aata", "page", "200"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=214",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page214",
},
{
  "text": "[\\field]\nDefines: F := (X, +, *, 0, 1)\nmeans:\n. '(X, +, *, 0) is \\commutative \\ring'\n. '(X, +, *, 0, 1) is \\division \\with.identity.ring'\nMetadata:\n. reference = \"source: @AATA; page: 200\"",
  "keywords": ["field", "f", "=", "x", "+", "*", "0", "1", "is", "commutative", "ring", "division", "with", "identity", "reference", "source", "aata", "page", "200"],
  "href": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page=214",
  "mobileHref": "http://abstract.ups.edu/download/aata-20190710-print.pdf#page214",
},
];

# MathLingua Language Design Decisions

**Decision:**

Variadic args can only be used in `is`, `in`, `:=` as
```
  (a...) := (b...)
  (f(x)...) := (g(x)...)
  (f(x...)...) := (g(x...)...)
  a... is b...
  f(x)... is \a, \b
  f(x...)... is \a, \b
  a... in A, B
  f(x)... in A, B
  f(x...)... in A, B
  a... notin A, B
  f(x)... notin A, B
  f(x...)... notin A, B
```

**Rationale:**

The `()` is used with `:=` because `a, b := B, c` is parsed as
`a` and `b := B` and `c`.  Thus parentheses are needed so that
`(a, b) := (B, c)`.  In addition, the number of elements in the
tuple on the left must equal the number on the right.

The use of `()` is also used since in the form `(a, b) := \f`
where `\f` defines a tuple, the tuple is in parentheses.

With `is`, `in`, and `notin`, however, the number of elements on
the left and right can differ, and `is`, `in`, and `notin` binds
stronger than `,`.  That is,
```
  a, b is \A, \B, \C
```
is interpreted as
```
  a is \A
  a is \B
  a is \C
  b is \A
  b is \B
  b is \C
```

**Decision:**

Sequences use `(...)` for arguments.  That is:
```
  A := {a_(i)}_(i)
```
instead of `A := {a_{i}}_{i}`.

**Rational:**

The use of parentheses match the use of parentheses in
function definitions and calls `f(x)`.

It also allows signatures to contain `_(i)`.  That is,
the signatures
```
  \sin(x)
```
and
```
  \some.sequence_(i)
```
are allowed.  In addition, if the signature has underscores
there isn't any ambiguities since underscore arguments in
signatures use curly braces.  For example, see:
```
  \f[x]_{a}^{b}_(i)
```

**Decision:**

Curly brace arguments can use commas to separate arguments or use
multi-curly braces.  Both mean the same thing.  For example:
```
  \f{a, b, c}
  \f{a}{b}{c}
  \f{a, b}{c}
```
all mean the same thing.

**Rationale:**

This is needed since `a, b is \B, c is \C` parses as
`a, b is (\B, c is \C)` which is invalid.  Thus,
```
  \f{a}{b is \B}{c is \C}
```
can be used to make the parsing clear.  This is also
useful in cases such as `\frac{a}{b}` that can be more
clear than `\frac{a, b}`.

# Examples that use the meta capabilities of MathLingua

```yaml
[\if{a...}:then{b...}]
States:
when: 'a..., b... is [:spec, statement, assignment:]'
that:
. if: a...
  then: b...
written: ""


[\iff{a...}:then{b...}]
States:
when: 'a..., b... is [:spec, statement, assignment:]'
that:
. iff: a...
  then: b...
written: ""


[\exists[x...]:where{a(x)...}:suchThat{b(x)...}:then{c(x)...}]
States:
when:
. 'a(x)... is [:spec:]'
. 'b(x)... is [:spec, :statement, assignment:]'
. 'c(x)... is [:spec, statement, assignment:]'
that:
. exists: x...
  where: a(x)...
  suchThat: b(x)...
  then: c(x)...
written: ""


[\exists[x...]:where{a(x)...}:then{b(x)...}]
States:
when:
. 'a(x)... is [:spec:]'
. 'b(x)... is [:spec, statement, assignment:]'
that:
. exists: x...
  where: a(x)...
  then: b(x)...
written: ""


[\existsUnique[x...]:where{a(x)...}:suchThat{b(x)...}:then{c(x)...}]
States:
when:
. 'a(x)... is [:spec:]'
. 'b(x)... is [:spec, :statement, assignment:]'
. 'c(x)... is [:spec, statement, assignment:]'
that:
. existsUnique: x...
  where: a(x)...
  suchThat: b(x)...
  then: c(x)...
written: ""


[\existsUnique[x...]:where{a(x)...}:then{b(x)...}]
States:
when:
. 'a(x)... is [:spec:]'
. 'b(x)... is [:spec, statement, assignment:]'
that:
. existsUnique: x...
  where: a(x)...
  then: b(x)...
written: ""


[\forAll[x...]:where{a(x)...}:suchThat{b(x)...}:then{c(x)...}]
States:
when:
. 'a(x)... is [:spec:]'
. 'b(x)... is [:spec, :statement, assignment:]'
. 'c(x)... is [:spec, statement, assignment:]'
that:
. forAll: x...
  where: a(x)...
  suchThat: b(x)...
  then: c(x)...
written: ""


[\forAll[x...]:where{a(x)...}:then{b(x)...}]
States:
when:
. 'a(x)... is [:spec:]'
. 'b(x)... is [:spec, statement, assignment:]'
that:
. forAll: x...
  where: a(x)...
  then: b(x)...
written: ""


[a \and/ b]
States:
when: 'a, b is [:spec, statement:]'
then:
. and: a, b
written: ""


[a \or/ b]
States:
when: 'a, b is [:spec, statement:]'
then:
. or: a, b
written: ""


[\not{a}]
States:
when: 'a is [:spec, statement:]'
then:
. not: a
written: ""
```

# Examples of how sets can be defined

```yaml
[\set]
Defines: {x}
written: ""


[\set[v...]:of{a(v)...}:where{b(v)...}:suchThat{c(v)...}]
Defines: X := {(x...)}
when:
. 'a(v) is [:assignment, expression:]'
. 'b(v) is [:spec:]'
. 'c(v) is [:statement:]'
means: 'X is \set'
satisfying:
. '(x...) := (a(v)...)'
. 'b(v)...'
. 'c(v)...'
written: ""
```

**Decision:**

Set targets are of the form `{(x, y)}` and not `{x, y}`.

**Rationale:**

The syntax is more clear and more clearly aligns with the use of
tuples for definitions that define tuples.

```yaml
[\set[v...]:of{a(v)...}:where{b(v)...}]
Defines: X := {(x...)}
when:
. 'a(v) is [:assignment, expression:]'
. 'b(v) is [:spec:]'
means: 'X is \set'
satisfying:
. '(x...) := (a(v)...)'
. 'b(v)...'
written: ""
```

The following is an example of how the \set:of:where and \set:of:where:suchThat
signature can be used.

> **Note:** The \real, >, and \subset/ signatures have not been previously
            defined but their definitions are what would be expected.

```yaml
Theorem:
given: X, Y
suchThat:
. 'X := \set[x]:of{x}:where{x is \real}'
. 'Y := \set[y]:of{y}:where{y is \real}:suchThat{x > 0}{x < 10}'
then:
. 'Y \subset/ X'
```

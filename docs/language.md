# MathLingua Language Guide

This is the human-facing guide to the MathLingua language implemented in this
repository. It explains how authors write `.mlg` files, how the structural and
formulation layers fit together, and what semantic checks the current
implementation performs.

For exact parser-level details, keep these references nearby:

- [Structural syntax](structural_syntax.md) describes line structure, groups,
  sections, and clause groups.
- [Formulation syntax](formulation_syntax.md) describes expressions, forms,
  declarations, command headers, statement helpers, and aliases.

Those files are the precise syntax references. This file is the readable map of
the same territory.

## The Two Layers

A MathLingua source file has two syntax layers.

1. The structural layer is line-oriented. It recognizes groups such as
   `Describes`, `Theorem`, `Documented`, and `forAll`.
2. The formulation layer is expression-oriented. It recognizes mathematical
   forms such as `f(x_)`, `x "in" A`, `\function:on{A}:to{B}`, and
   `G is \set via X`.

Most source lines are first parsed structurally. Whenever a section expects a
formula, command, alias, or header, the structural parser delegates that text to
one of the formulation parsers.

## File Shape

An `.mlg` document is a sequence of groups. A group may have a heading line in
square brackets, followed by one or more sections.

```text
[\function:on{A}:to{B}]
Describes: f(x__)
when:
. A, B is \set
Documented:
. called: "function from $A?$ to $B?$"
```

Headings identify local labels, commands, people, or resources depending on the
group kind. The first section label, not the heading, determines the structural
group kind.

Section labels are case-sensitive and order-sensitive. Optional sections may be
omitted, but if present they must appear in the order defined for that group.
For example, `using:` must come before `when:`, and `Documented:` must come
after `Provides:` and `Justified:` in definition groups.

Blank lines and lines whose trimmed text starts with `--` are comments and are
ignored in normal parse positions.

## Lines, Sections, and Arguments

A section has one of these shapes:

```text
label:
label: inline argument
```

Additional arguments are written on following lines indented under the section.
The dot form:

```text
. x is \set
```

is treated as an argument line at an indent two spaces deeper than the current
line. It is the preferred style for repeated arguments.

Text arguments must be quoted:

```text
Title: "Algebra"
```

The parser strips the outer quotes and does not interpret escape sequences.

Any non-text argument line containing `:` is treated as a nested structural
group, not as a formulation. This is important when writing formulas that
contain command tails. Put such formulas inline where the section parser expects
formulation text, or use a multiline formulation block when needed.

Multiline formulations are started only by a line whose entire text is one of
`(`, `[`, `{`, or `(.`, and ended by the matching `)`, `]`, `}`, or `.)` at the
same indent.

Single-quoted formulations are not accepted.

## Names and Placeholders

Normal names are either identifier-like names or backtick names.

```text
x
x_1
123
`x + y`
```

Identifier-like names may contain internal underscores, but must start and end
with an ASCII letter or digit. Backtick names may contain any non-backtick text.

Placeholders end in `_`, and magnetic placeholders end in `__`.

```text
x_
value_
x__
```

Placeholders are used in forms and declarations. Magnetic placeholders are used
for function-like forms that bind one placeholder with special rendering
behavior, such as `f(x__)`.

The exact spellings `is`, `is?`, `is_not?`, and `via` are reserved by the
lexer-driven formulation parser.

## Forms and Declarations

Forms describe the syntactic shape of mathematical objects. Declarations may
name those forms.

```text
x
f(x_)
g := f(x_, y_)
(x_, y_)
Pair := (x_, y_)
{x_}
Set := {x_}
x_ |plus| y_
neg| x_
x_ |prime
```

Function forms support either one magnetic placeholder or one or more ordinary
placeholders. Mixed magnetic and ordinary placeholders are not accepted.

Tuple forms require at least two elements. One-element tuples are not currently
supported.

Set forms contain a placeholder form, such as `{x_}` or `{x_(i_)}`.

When a declaration is used in a defining context, the declared names become
available to later checks. For example, `G := (X, *, e)` declares `G`, `X`, and
`e`; operators in tuple elements are not declared as ordinary symbols.

## Expressions

Expressions cover ordinary mathematical formulas.

```text
x + y
f(x, y)
map[| key := x, value := y |]
(x, y)
{x "in" A : x_ | x = y}
F[A]
\function:on{A}:to{B}(x)
x is? \set
```

The expression precedence, from lowest to highest, is:

1. spec and predicate forms
2. equality and special binary operators
3. addition and subtraction
4. multiplication and division
5. powers
6. named operators and infix commands
7. unary `+` and `-`
8. atoms

Powers associate to the right. The arithmetic binary levels associate to the
left. Named-operator and infix-command expressions are single-step; an
ungrouped chain like `a |f| b |g| c` is not accepted.

Subset expressions are intentionally narrow. The supported forms are:

```text
F[A]
F[A, B]
F[A[B]]
```

The names inside subset brackets must be names, not arbitrary expressions.

Expression labels are written after grouped expressions:

```text
(x + y)[:sum:]
(. x + y .)[:normalized:]
```

## Commands and Signatures

Commands begin with `\` and are the main way the language names mathematical
concepts.

```text
\set
\function:on{A}:to{B}
\relation:from{A}:to{B}(x, y)
```

A command signature is the command shape with concrete arguments removed. Both
`\function:on{A}:to{B}` and `\function:on{X}:to{Y}` have the signature
`\function:on:to`.

Curly argument groups are required where the command definition expects them.
Trailing parenthesized groups are invocation groups. If a definition includes
only trailing parenthesized groups beyond the actual use, the use may omit those
groups.

For example, a definition heading:

```text
[\some.function{A}(x, y)]
```

may be referenced as either:

```text
\some.function{A}
\some.function{A}(x, y)
```

Every command tail such as `:to` or `:from` must include at least one curly
argument group.

Infix commands use `\:` and `:/`:

```text
x \:divides:/ y
[\:divides:/]
```

Refined commands use `::` and a parenthesized refinement list:

```text
\(continuous, bounded)::function:on{A}:to{B}
[\(continuous)::function:on{A}:to{B}]
```

The expression parser does not accept refined command expressions as ordinary
expressions. Refined command expressions are accepted in statement contexts that
use the refined statement parser, such as theorem `given:` sections.

## Statement Forms

Several section types use statement-like formulation parsers rather than the
general expression parser.

An `is` statement has this shape:

```text
x is \set
f(x_) is \function:on{A}:to{B}
x_, y_ is \set
```

The helper parser for `is` statements requires spaces around ` is ` and requires
the right-hand side to be a command type expression. It is stricter than the
general expression parser's `x is y` expression form.

A spec statement uses a quoted operator:

```text
x "in" A
x "less than" y
```

In expression parsing, quoted operators must be identifier-like quoted names,
such as `"in"`. In statement helper parsing, quoted operator text is scanned
raw at top level and may contain spaces or punctuation.

An `is via` statement records a subtype or extension view:

```text
G is \set via X
X, Y is \set via (X, Y)
```

The left side must be an `is` statement. The right side after `via` may be any
form or declaration.

## Top-Level Groups

These groups may appear at the top level of a document.

| Group | Heading | Required purpose |
| --- | --- | --- |
| `Title` | none | document title text |
| `Section` | none | first-level prose heading |
| `Subsection` | none | second-level prose heading |
| `Subsubsection` | none | third-level prose heading |
| `Describes` | command | introduces a command for a mathematical form |
| `Defines` | command | defines a statement, specification, or type fact |
| `Refines` | command | defines a refined command in terms of another command |
| `States` | command | defines a named statement with a `that:` body |
| `Axiom` | optional command | theorem-like assertion |
| `Theorem` | optional command | theorem-like assertion |
| `Corollary` | optional command | theorem-like assertion with `of:` text |
| `Lemma` | optional command | theorem-like assertion |
| `Conjecture` | optional command | theorem-like assertion |
| `Person` | author | person metadata |
| `Resource` | resource | bibliography or web metadata |
| `Specify` | none | numeric-domain specification metadata |

Definition-like groups with command headings introduce command signatures.
Duplicate signatures are rejected across all definition kinds, including named
theorem-like groups.

`Describes`, `Defines`, and `Refines` must include a `called:` item somewhere in
their `Documented:` section. `States` and theorem-like groups may have
documentation, but the current semantic check does not require a `called:` item
for them.

## Definition Groups

`Describes` introduces the form associated with a command.

```text
[\set]
Describes: X
Documented:
. called: "set"
```

Optional sections, in order:

```text
using:
when:
extends:
specifies:
satisfies:
Provides:
Justified:
Documented:
Aliases:
References:
Metadata:
```

`Defines` introduces a command by an `is` or spec statement.

```text
[\foo{s}]
Defines: x is \bar{s}
Documented:
. called: "foo"
```

It accepts `using:`, `when:`, `expresses:`, and the same support sections as
`Describes`.

`Refines` introduces a refined command.

```text
[\(continuous)::function:on{A}:to{B}]
Refines: f(x__) is \function:on{A}:to{B}
Documented:
. called: "continuous"
```

`States` defines a command-backed statement body:

```text
[\commutative{S}]
States:
when: S is \set
that:
. forAll: x, y is \element.of{S}
  then:
  . x * y = y * x
```

## Theorem-Like Groups

`Axiom`, `Theorem`, `Corollary`, `Lemma`, and `Conjecture` share the same proof
shape. `then:` is required. `given:`, `where:`, `iff:`, and support sections are
optional. `Corollary` also requires an `of:` section.

```text
Theorem:
given:
. X, Y is \set
. f is \function:on{X}:to{Y}
then:
. f is? \function:on{X}:to{Y}
```

Items in `given:` introduce available type/spec facts and declared symbols for
the theorem body. Items in `where:` are local assumptions or bindings available
while checking `then:` and `iff:`.

If a theorem-like group has a command heading, that heading introduces a
signature and participates in duplicate-signature and reference checks.

## Clause Groups

Clause sections accept either inline expressions or nested clause groups.

| Clause | Meaning in the checker |
| --- | --- |
| `not` | checks the nested clause in the current context |
| `allOf` | checks all children; when assumed, gathers facts from children |
| `anyOf` | checks all children |
| `oneOf` | checks all children |
| `exists` | creates a child context from its binding/spec and assumes `suchThat:` |
| `existsUnique` | same as `exists`, with unique-existence intent |
| `forAll` | creates a child context, assumes `where:`, checks `then:` |
| `if` | assumes `if:`, checks `then:` |
| `iff` | assumes `iff:`, checks `then:` |
| `piecewise` | assumes `if:`, checks `then:`; `else:` is checked in the outer context |
| `given` | assumes one refined-capable given statement, then checks `then:` |

Bindings are written with `:=` and create local syntactic substitutions.

```text
where:
. A := B
then:
. \foo{B}
```

If the context knows `A is \real`, then `\foo{B}` may satisfy a requirement for
`\real` because `A := B` makes the two keys normalize together.

Quantifier bindings are local to the clause group that introduces them.

## Support Sections

`Provides:` accepts:

- `symbol:` groups, which usually define aliases or specification operators
- `connection:` groups, which contain prose fields such as `to:`, `means:`,
  `signifies:`, `viewable:`, and `through:`

`Documented:` accepts:

- `written:`
- `called:`
- `writing:`
- `overview:`
- `related:`
- `discoverer:`

`Justified:` accepts `label:` and `by:` groups with `comment:` text.

`Aliases:` accepts `alias:` groups. `Metadata:` accepts `id:` and `version:`.
`References:` contains resource headers such as `$book.chapter`.

`Person` groups use author headings such as `[@ada.lovelace]` and require
`name:` plus `biography:`. `Resource` groups use resource headings such as
`[$principia]` and contain resource item groups like `title:`, `author:`,
`url:`, and `year:`.

## Aliases

Expression aliases use `:=>`.

```text
alias: f(x_) :=> x + x
```

The alias left-hand side may be a form/declaration, a simple command header, or
an infix command header. Refined command headers are not accepted on the
left-hand side.

Spec-operator aliases use `:->`.

```text
symbol: x_ "in" R :-> x is \real
```

When a described command provides a spec-operator alias, the type checker can
reduce matching spec facts. If the context knows `R is \reals` and `r "in" R`,
the alias above lets the checker prove `r is \real`.

Writing aliases use `:~>`.

```text
writing: f(x_) :~> f(x)
as:
. "f(x)"
```

The body after `:~>` is raw non-empty text, not parsed formulation syntax.

## Semantic Checking

The semantic checker runs after parsing. It currently performs three broad
checks.

First, it collects all command definitions. It rejects duplicate command
signatures and checks that `Describes`, `Defines`, and `Refines` have
`Documented:` metadata containing at least one `called:` item.

Second, it walks every command-like reference and checks that the referenced
signature exists and that the argument shape matches. Refined command references
may fall back to checking their base command and individual refinement pieces
when the composed refined signature is not defined directly.

Third, it checks symbol usage and type requirements. Unknown symbols are
reported as `Unrecognized symbol`. Type requirements are reported as
`Could not prove requirement ...` when the current context does not imply the
required fact.

## Symbol Scope

The checker is intentionally conservative about undeclared variables.

Symbols are introduced by:

- command header forms in definition and named theorem-like groups
- the main `Describes:`, `Defines:`, and `Refines:` subjects
- assumptions in `using:`, `when:`, theorem `given:`, and local clause groups
- local bindings such as `A := B`
- subject forms in assumed `is` or spec facts
- names inside forms, tuples, set declarations, function declarations, and
  placeholder forms

Numeric literal names made only of ASCII digits are accepted without prior
declaration.

Symbols used only in a conclusion must already be known. For example:

```text
Theorem:
given:
. X, Y is \set
. f is \function:on{X}:to{Y}
then:
. f is? \function:on{X}:to{Z}
```

reports `Z` as unrecognized because `Z` was not introduced by the theorem
heading, `given:`, or `where:`.

Declaration forms introduce their nested names. In:

```text
[\group]
Describes: G := (X, *, e)
extends: G is \set via X
```

the `Describes:` form introduces `G`, `X`, and `e`, so `via X` is a recognized
symbol.

## Type Facts and Requirements

A command definition may declare requirements through `using:` and `when:`.
Those requirements must be provable whenever the command is used.

```text
[\function:on{A}:to{B}]
Describes: f(x__)
when: A, B is \set
Documented:
. called: "function"
```

A later reference to `\function:on{G}:to{G}` requires the checker to prove
`G is \set`.

The checker understands two fact kinds:

- type facts, such as `G is \set`
- spec facts, such as `x "in" G`

Facts can be introduced by `given:`, `where:`, `when:`, `using:`, assumed clause
groups, and expression facts such as `x is \set` or `x "in" X`.

When command arguments are substituted into requirements, local bindings are
normalized. If `A := B` is in scope, facts about `A` can satisfy requirements
about `B`, and vice versa.

## Subtyping With `extends:`

`extends:` introduces subtype and extension implications for `Describes`
definitions.

```text
[\group]
Describes: G := (X, *, e)
extends: G is \set via X
Documented:
. called: "group"
```

This means that if the checker knows `G is \group`, it can prove `G is \set`.
The implication is recursive, so subtype chains are followed.

The `via` form documents and validates the view used to regard the subtype as
the supertype. In the current checker, the target fact on the left side of
`via` is what participates in proof search; the `via` form itself is checked for
declared symbols but is not otherwise used for term rewriting.

An `extends:` section may also use a spec statement:

```text
extends: x "in" X
```

That records the corresponding spec fact as an implication for values of the
owning type.

## Specification Operators

Provided spec operators connect notation such as membership to type facts or
other spec facts.

```text
[\reals]
Describes: R
Provides:
. symbol: x_ "in" R :-> x is \real
Documented:
. called: "reals"
```

If the context contains `R is \reals` and `r "in" R`, the checker can reduce the
spec fact through the provided symbol and prove `r is \real`.

Direct spec requirements are also supported. If a command requires `x "in" G`,
then an exact matching spec fact in the context satisfies that requirement even
without reducing it to a type fact.

## Rendering Metadata

`called:` entries provide plain-text rendering names. They may contain math
substitution markers such as `$A?$`.

```text
Documented:
. called: "function on $A?$ to $B?$"
```

`written:` entries provide math-mode rendering templates.

```text
Documented:
. written: "f? \: : \: A? \rightarrow B?"
```

The renderer uses these entries to display commands, forms, and definitions.
The semantic checker only enforces that `Describes`, `Defines`, and `Refines`
include at least one `called:` item.

## Current Footguns

These behaviors are intentionally documented because authors will run into
them.

- Section order is strict, and capitalization is exact.
- Group kind is chosen by the first section label, not by the heading.
- Some singular sections keep only the first valid parsed value and ignore extra
  valid values.
- Text parsing strips only the outer quotes and does not process escapes.
- Any non-text argument line containing `:` becomes a nested group.
- Clause formulation arguments use `parse_expression`, not `parse_is_or_spec`.
- Empty documents are accepted.
- Heading-only groups are not valid structural groups.
- One-element tuples are not supported.
- Subset syntax only supports the three name-only shapes listed above.
- Refined command expressions are not ordinary expressions.
- Quoted operators are stricter in lexer-driven expressions than in raw
  statement helpers.
- Command tail parts require `{...}` arguments.

When in doubt about exact syntax, use [structural_syntax.md](structural_syntax.md)
and [formulation_syntax.md](formulation_syntax.md) as the parser-level source of
truth.

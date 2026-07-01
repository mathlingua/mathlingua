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
   forms such as `f(x_)`, `x "in" A`, `\function:on{A}:to{B}`,
   `(_ "in" A) => (_ "in" B)`, and `G is \set via X`.

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
after `Enables:` and `Justified:` in definition groups.

Lines whose trimmed text starts with `--` are comments. At the top level, blank
lines and comments are skipped before the next group; inside a group or section,
comments are skipped but blank lines terminate the current block.

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

A non-text argument line starts a nested structural group when it is a heading
or when its first colon follows a section-label-shaped prefix. Formulation
delimiters such as `:=`, `:->`, `:=>`, and `:~>` are excluded from that
structural-colon rule, and command tails such as `\function:on{X}:to{Y}` remain
formulations because the prefix before the colon is not a section label.

Multiline formulations are started only by a line whose entire text is one of
`(`, `[`, `{`, or `(.`, and ended by the matching `)`, `]`, `}`, or `.)` at the
same indent.

Single-quoted formulations are not accepted.

## Names and Placeholders

Normal names are either identifier-like names or stropped symbolic names.

```text
x
x_1
123
`*`
`*+`
```

Identifier-like names may contain internal underscores, but must start and end
with an ASCII letter or digit. Stropped symbolic names are wrapped in backticks
and may contain only operator characters from `-~!#%^&*\+=|<>/`.

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
g ::= f(x_, y_)
(x_, y_)
Pair ::= (x_, y_)
{x_ : ...}
Set ::= {x_ : ...}
x_ |plus| y_
neg| x_
x_ |prime
```

Function forms support either one magnetic placeholder or one or more ordinary
placeholders. Mixed magnetic and ordinary placeholders are not accepted.

Tuple forms require at least two elements. One-element tuples are not currently
supported.

Set forms contain a placeholder form, such as `{x_ : ...}` or `{x_(i_) : ...}`.

When a declaration is used in a defining context, the declared names become
available to later checks. For example, `G ::= (X, *, e)` declares `G`, `X`, and
`e`; operators in tuple elements are not declared as ordinary symbols.

## Expressions

Expressions cover ordinary mathematical formulas.

```text
x + y
f(x, y)
map[| key := x, value := y |]
(x, y)
{x_ : x_ "in" A | x_ = y}
F[A]
\function:on{A}:to{B}(x)
x is? \set
\set is? \\type
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
left. Named-operator and infix-command expressions also associate to the left,
so `a |f| b |g| c` is accepted as a left-associated chain.

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

Argument-group counts are tracked as the command's required shape, but they do
not disambiguate definitions. Two definitions with the same signature and
different argument counts are still duplicate command signatures.

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

In command declaration headings, a tail may be written with `:?` to make that
tail optional at reference sites:

```text
[\function:on{A}:?to{B}]
```

This declares both `\function:on` and `\function:on:to`. Multiple optional
tails may be independently omitted as long as the remaining tails keep the order
from the heading, so `[\foo:?baz{A}:?bar{B}]` accepts `\foo`,
`\foo:baz`, `\foo:bar`, and `\foo:baz:bar`, but not `\foo:bar:baz`.
The `:?` spelling is only for command declaration headings; expressions use
plain `:baz` or `:bar` for the optional parts they include.

Infix commands use `\.` and `./`:

```text
x \.divides./ y
[X \.set.=./ Y]
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

The helper parser for `is` statements requires spaces around ` is `. The
right-hand side can be an ordinary command type expression or a supported
built-in type expression. Expression-level `is?` and `is_not?` accept command,
refined-command, and supported built-in type predicates; the helper parser
differs mainly in its subject syntax and in the refined-command variant used by
theorem-style `given:` sections.

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
| `SectionTitle` | none | first-level prose heading |
| `SubsectionTitle` | none | second-level prose heading |
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

Groups with command headings introduce command signatures: `Describes`,
`Defines`, `Refines`, `States`, and theorem-like groups that have an optional
heading. Duplicate signatures are rejected across all of these definition
kinds.

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
Enables:
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

`specifies:` sections on `Describes` and `Refines` are parsed and command
references inside them are validated, but the current type checker does not use
them as assumptions, requirements, or proof facts.

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
the theorem body. Theorem-like `given:` sections accept refined command type
expressions. Items in `where:` are local assumptions or declarations available
while checking `then:` and `iff:`.

If a theorem-like group has a command heading, that heading introduces a
signature and participates in duplicate-signature and reference checks.

## Clause Groups

Clause sections accept inline formulations or nested clause groups. Inline
clause formulations are tried as declaration statements first, then ordinary
expressions.

| Clause | Meaning in the checker |
| --- | --- |
| `not` | checks the nested clause in the current context |
| `allOf` | checks all children; when assumed, gathers facts from children |
| `anyOf` | checks all children |
| `oneOf` | checks all children |
| `exists` | creates a child context from its declaration and assumes optional `suchThat:` clauses |
| `existsUnique` | same as `exists`, with unique-existence intent |
| `forAll` | creates a child context, assumes `where:`, checks `then:` |
| `if` | assumes `if:`, checks `then:` |
| `iff` | assumes `iff:`, checks `then:` |
| `piecewise` | assumes `if:`, checks `then:`; `else:` is checked in the outer context |
| `given` | assumes one refined-capable given statement, then checks `then:` |

Declarations can combine `::=` with `:=` to introduce symbols and create local
syntactic substitutions.

```text
where:
. A ::= B := B
then:
. \foo{B}
```

If the context knows `A is \real`, then `\foo{B}` may satisfy a requirement for
`\real` because `A ::= B := B` makes the two keys normalize together.

Quantifier declarations are local to the clause group that introduces them.

## Support Sections

`Requires:` accepts:

- `capability:` groups, which define notation that is part of the construct's
  definition
- `definition:` groups of the form `\command is <spec>`, which require the
  referenced command to be a top-level `Defines:` entry whose definition
  establishes the requested fact

`Enables:` accepts:

- `capability:` groups, which define additional notation made available by the
  construct
- `from:` plus `capability:`, which defines notation made available by a cast
  source
- `from:` plus `as:`, which defines how facts from a cast source are viewed as
  facts about the described form
- `viewable:` groups, which declare that the described type can be viewed as
  another type when satisfying already-resolved command requirements
- `connection:` groups, which contain prose fields such as `to:`, `means:`,
  `signifies:`, `viewable:`, and `through:`

For type checking, capabilities from `Requires:` and `Enables:` are combined.
The separate sections are for communication: `Requires:` describes what the
construct has by definition, while `Enables:` describes further operations that
the construct supports.

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

Open-text fields are retained as prose. Command-looking text inside prose,
metadata, references, and rendering templates is not parsed as formulation
syntax for semantic reference checking.

## Aliases

Expression aliases use `:=>`.

```text
alias: f(x_) :=> x + x
```

The alias left-hand side may be a form/declaration, a simple command header, or
an infix command header. Refined command headers are not accepted on the
left-hand side. The right-hand expression is parsed, but the current semantic
reference walker does not validate command references inside that expression.

Spec-operator aliases use `:->`.

```text
capability: x_ "in" R :-> x is \real
capability: x_ "in" X :-> \\abstract
```

When a described command enables a spec-operator alias, the type checker can
reduce matching spec facts. If the context knows `R is \reals` and `r "in" R`,
the alias above lets the checker establish `r is \real`.

The target of a spec-operator alias may also be a built-in keyword written with
two leading backslashes, such as `\\abstract`. Spec-operator aliases are
currently treated as declarations by the reference walker, so command references
inside their target are not validated there. Built-in targets are accepted by
the parser, but the current type-reduction code ignores them.

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
`Could not establish requirement ...` when the current context does not establish
the required fact.

The checker is not a proof checker for theorem conclusions. It checks that
conclusions are syntactically valid, use declared symbols, reference defined
commands with the right argument shapes, and satisfy any requirements of those
commands.

## Symbol Scope

The checker is intentionally conservative about undeclared variables.

Symbols are introduced by:

- command header forms in definition and named theorem-like groups
- the main `Describes:`, `Defines:`, and `Refines:` subjects
- assumptions in `using:`, `when:`, theorem `given:`, and local clause groups
- local declarations such as `A ::= B := B`
- subject forms in assumed `is` or spec facts
- names inside forms, tuples, set declarations, function declarations, and
  placeholder forms

Numeric literal names made only of ASCII digits are accepted without prior
declaration.

Assumptions are processed in order. In a declaration statement, the subject and
optional `::=` expansion introduce names before any `:=` right-hand expression
is checked. Command arguments, spec targets, and names used only on the right
side of `:=` must already be known from earlier context.

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
Describes: G ::= (X, *, e)
extends: G is \set via X
```

the `Describes:` form introduces `G`, `X`, and `e`, so `via X` is a recognized
symbol.

## Type Facts and Requirements

A command definition may declare requirements through `using:` and `when:`.
Those requirements must be provable whenever the command is used as an
expression or predicate, and whenever a parameterized command type expression is
used in an `is` statement.

```text
[\function:on{A}:to{B}]
Describes: f(x__)
when: A, B is \set
Documented:
. called: "function"
```

A later reference to `\function:on{G}:to{G}` in either an expression or a type
assertion requires the checker to prove `G is \set`. Type assertions for
no-argument `Describes` commands are nominal: `G is \group` records that fact
without expanding the internal `\group` requirements at the assertion site.

The checker understands these fact kinds:

- type facts, such as `G is \set`
- spec facts, such as `x "in" G`

It also has built-in types for meta-level checks. In particular, `\\type`
holds for command references whose top-level entry is a `Describes:` item.
Thus `\set is? \\type` succeeds when `\set` is described, while
`\sqrt is? \\type` fails when `\sqrt` is a `Defines:` item.

Facts can be introduced by `given:`, `where:`, `when:`, `using:`, assumed clause
groups, and expression facts such as `x is \set` or `x "in" X`.

When command arguments are substituted into requirements, local definitions are
normalized. If `A ::= B := B` is in scope, facts about `A` can satisfy
requirements about `B`, and vice versa.

Refined command type expressions are accepted in refined-capable statement
positions and are reference-checked, but the current proof context records type
facts only for ordinary command type expressions. A fact such as
`f is \(continuous)::function:on{A}:to{B}` does not currently become a usable
type fact for proving later requirements.

Refined command fallback shapes are also used for reference validation. If a
composed refined command is not defined directly, the checker can validate the
base command and refinement pieces for existence and arity. Requirement proving
for command use still looks up the exact command signature being used.

## Subtyping With `extends:`

`extends:` introduces subtype and extension implications for `Describes`
definitions.

```text
[\group]
Describes: G ::= (X, *, e)
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

Function-like types can describe their call behavior with a function type on
the right-hand side of an `is` statement:

```text
[\function:on{A}:to{B}]
Describes: f(x__)
when: A, B is \set
extends: f is (_ "in" A) => (_ "in" B)
Documented:
. called: "function"
```

The input side contains one or more specs and the output side contains exactly
one spec. Both sides must be parenthesized, and each spec parameter must be
written as `_`. If the checker knows `f is \function:on{A}:to{B}` and
`y "in" A`, it can validate `f(y)` and prove `f(y) "in" B`.

## Specification Operators

Required and enabled specification capabilities connect notation such as
membership to type facts or other spec facts.

```text
[\reals]
Describes: R
Requires:
. capability: x_ "in" R :-> x is \real
Documented:
. called: "reals"
```

If the context contains `R is \reals` and `r "in" R`, the checker can reduce the
spec fact through the capability and establish `r is \real`.

The alias target must satisfy the requirements of any command type it uses in
the owning context. For example, if `\element.of:group{G}` requires
`G is \set`, then a capability on `\group` may alias membership to
`\element.of:group{G}` only when `G is \set` is available, commonly through an
`extends: G is \set` subtype declaration on `\group`.

Direct spec requirements are also supported once the target type requires or
enables the operator. If `\group` has `x_ "in" G` as a capability and a command
requires `x "in" G`, then an exact matching spec fact in the context satisfies
that requirement even without reducing it to a type fact. A raw fact such as
`x "in" G` is invalid when the checker knows `G` has a type that does not enable
`"in"`.

### Cast-Backed Capabilities

`Enables:` may use a `from:` group to describe capabilities supplied by a cast
literal rather than by the opaque type itself.

```text
[\set]
Describes: X
Requires:
. capability: x_ "in" X :-> \\abstract
Enables:
. from: Y ::= {y__ : ...}
  capability: x_ "in" X :-> x_ member_of Y
Documented:
. called: "set"
```

If a value is introduced as `A := \set@{x_ : x_ is \real}`, the checker records
the literal for `A`. When it later reduces `a "in" A`, the `from:` capability
substitutes the source subject `Y` with `A`, producing `a member_of A`. The
existing `member_of` reducer then reads the cast literal and can establish
`a is \real`.

An ordinary non-`from:` capability on an opaque target does not read a cast
literal through `member_of`. For example, `Describes: X` with
`capability: x_ "in" X :-> x_ member_of X` does not make `\set@{...}` expose
the literal's element facts. Use a structural target such as
`Describes: X ::= {x__ : ...}` or an explicit `from:` capability for that.

A `from:` group may also use `as:` with an expression binding, for example:

```text
Enables:
. from: P ::= {(p_, q_) : ...}
  as: f(p_) := q_
```

This records and validates the cast view from the source structure to the
described form. If `F := \function@{(p_, q_) : q_ is \set}`, the binding lets
the checker use facts about `q_` from the source literal as facts about
`F(p_)`; for example it can establish `F(a) is \set` when the source literal
supports that substitution.

### Viewable Casts

`Enables:` may contain `viewable:` groups:

```text
[\integer]
Describes: n
Enables:
. viewable:
  as: r := \as.rational{n} is \rational
  states: n \.embedded.to./ r
```

The `as:` declaration states the target type using `is`. The `:= ...`
construction is optional; without it, the cast is accepted but the converted
value is opaque.
The optional `states:` clause records a statement relating the original value
and the viewed value.

Viewable casts are used when checking whether an already-resolved command's
arguments satisfy its requirements. For example, if `\integer` is viewable as
`\rational`, then a command requiring `x is \rational` may accept an integer
argument. Viewable casts are not used for operator resolution: `+` on integers
will not resolve to `+` on rationals merely because integers are viewable as
rationals.

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

Both `called:` and `written:` templates support conditional fragments:

```text
@[U]{_{U?}}:{_X}
@[x, y]{x? + y?}
```

The first branch is rendered only when every variable listed in `[...]` has a
substitution value. The optional `:{...}` branch is rendered otherwise. If the
fallback branch is omitted, the conditional renders nothing when the variables
are not all present. Conditional fragments may be nested.

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
- Section-shaped colons in non-text argument lines start nested groups.
- Clause formulation arguments are parsed in fallback order: declaration
  statement, then expression.
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

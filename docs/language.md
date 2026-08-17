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
   `Defines`, `Theorem`, `Documented`, and `forAll`.
2. The formulation layer is expression-oriented. It recognizes mathematical
   forms such as `f(x_)`, `x "in" A`, `\function:on{A}:to{B}`,
   `(_ "in" A) -> (_ "in" B)`, and `G is \set via X`.

Most source lines are first parsed structurally. Whenever a section expects a
formula, command, alias, or header, the structural parser delegates that text to
one of the formulation parsers.

## File Shape

An `.mlg` document is a sequence of groups. A group may have a heading line in
square brackets, followed by one or more sections.

```text
[\function:on{A}:to{B}]
Defines: f(x__)
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
For example, `using:` must come before `when:`, and `Justification:` must come
after `Documented:` in definition groups.

Lines whose trimmed text starts with `--` are comments. At the top level, blank
lines and comments are skipped before the next group; inside a group or section,
comments are skipped but blank lines terminate the current block.

## Lines, Sections, and Arguments

A section has one of these shapes:

```text
name:
name: inline argument
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
X'
x'_a'
123
`*`
`*+`
`*'`
```

Identifier-like names must start with an ASCII letter or digit and may contain
internal underscores. They may also carry **prime marks** — one or more trailing
`'` after an alphanumeric — so a name may end in a prime (`X'`, `X''`, `e'`),
including on a subscript (`x'_a'`). Symbolic operator names are a run of operator
characters from `-~!#%^&*\+=|<>/`, optionally carrying trailing primes (`*'`,
`*''`) and/or a `_`-prefixed subscript (`*_1`, `+_i`, `<=_max`, `*'_a`) so
operators can be primed and indexed. Prime marks render as LaTeX primes (`X'` as
$X'$, `x'_a'` as $x'_{a'}$). Stropped symbolic names wrap an operator name in
backticks: `` `*` `` is the operator `*` referred
to by name, so where `*` is bound (e.g. a magma's operation) `` `*` `` resolves as
that operator's value — passable as an argument and invocable in function form as
`` `*`(a, b) ``. It renders as the operator itself, without the backticks
(`` `*` `` renders as $*$, `` `*'` `` as $*'$).

Placeholders end in `_`, and magnetic placeholders end in `__`.

```text
x_
value_
x__
```

Placeholders are used in forms and declarations. Magnetic placeholders are used
for function-like forms that bind one placeholder with special rendering
behavior, such as `f(x__)`.

The exact spellings `is`, `is?`, and `via` are reserved by the
lexer-driven formulation parser.

## Forms and Declarations

Forms describe the syntactic shape of mathematical objects. Declarations may
name those forms.

```text
x
f(x_)
g ::= f(x_, y_)
G ::= g(x_) ::= y_
(x_, y_)
pt ::= (x_, y_)
{x_ : ...}
set ::= {x_ : ...}
x_ |plus| y_
neg| x_
x_ |prime
```

Function forms support either one magnetic placeholder or one or more ordinary
placeholders. They may instead declare a ranged variadic input:

```text
f(x_[i_ := 1...n])
f(x__[i_ := 1...n])
```

`x_` spreads the mapping inputs, while `x__` records that the variadic inputs
are treated as a tuple. The ranged input must be the mapping's only parameter,
must bind an index and a length name, and must start at 0 or 1. Mixed magnetic
and ordinary placeholders are not accepted.

Tuple forms require at least two elements. One-element tuples are not currently
supported.

Set forms contain a placeholder form, such as `{x_ : ...}` or `{x_(i_) : ...}`.

When a declaration is used in a defining context, the declared names become
available to later checks. For example, `G ::= (X, *, e)` declares `G`, `X`, `*`,
and `e`. A function alias such as `G ::= g(x_)` declares both `G` and `g`;
`G ::= g(x_) ::= y_` additionally declares the output placeholder `y_`. The
alias and mapping name denote the same mapping. An operator form declares its
operator symbol as well as its
placeholders — `x_ * y_` declares `*`, `x_`, and `y_` — so a later use like
`a * b` resolves as the application `*(a, b)`.

## Expressions

Expressions cover ordinary mathematical formulas.

```text
x + y
f(x, y)
f[| key := x, value := y |]
f[|x1?_ = 1, x2?_ = 2, ... = 0|]
(x, y)
{x_ : x_ "in" A | x_ = y}
F[A]
\function:on{A}:to{B}(x)
x is? \set
\set is? \\type
```

Builtin types and targets are written with two leading backslashes: `\\type`
(the type of `Defines` types), `\\statement`, `\\expression`,
`\\specification` (the types of statements, expressions, and specification
literals), `\\opaque` (an unstructured value), and `\\abstract` (the abstract
`:->` capability target).

A named mapping call accepts both the legacy `:=` assignment spelling and `=`.
The catch-all slot `... = value` supplies the value for parameters not named by
an earlier element. A call may also name a ranged subset, for example
`f[|x_[i_[1...m]] = 1, ... = 0|]`. Omitting the catch-all represents a partial
application, such as `f[|x1?_ = 1|]`.

The expression precedence, from lowest to highest, is:

1. function literal `=>` (right-associative)
2. spec and predicate forms (`is`, `is?`, quoted `"op"` specs, infix specs
   `\:...:/`, `member_of`, `satisfies`, spec literals)
3. infix commands `\.name./`
4. equality (`=`, `!=`) and special binary operators
5. addition and subtraction
6. multiplication and division
7. powers
8. named operators (`|op|`, member-path `|M.*|`)
9. unary prefix (`+`, `-`, prefix named operators)
10. postfix named operators (`x |f`)
11. atoms

Note that infix commands (level 3) bind **looser** than arithmetic, while named
operators (level 8) bind **tighter** — they are not the same level. Powers
associate to the right; the arithmetic and named-operator levels associate to
the left, so `a |f| b |g| c` is a left-associated chain.

Subset expressions are intentionally narrow. The supported forms are:

```text
x[i]
x[i, j]
x[i[j]]
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
different argument counts are still duplicate command signatures. The one
exception is a mapping-parameter header, whose mapping arity and selected
parameter positions form a specialized signature as described below.

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

Callable headings may use placeholder spelling for their trailing parameters,
matching function declarations. Thus `[\natural.succ(n_)]` declares the same
one-argument command shape as a trailing `(n)` group while making the parameter
role explicit in the source.

Infix command headings accept the same placeholder spelling for their operands.
For example, `[n_ \.natural.+./ m_]` binds `n` and `m` as the two parameters of
the infix command, just as `[\sin(x_)]` binds `x` for a callable command.

Every command tail such as `:to` or `:from` must include at least one curly
argument group.

### Numeric literal and index types

A collection configures the fallback types of numeric spellings with one
top-level `Specify:` group:

```text
Specify:
. decimal:
  is: \real
. zeroOrPositiveInt:
  is: \whole
. positiveInt:
  is: \natural
. int:
  is: \integer
```

The four `is:` values are ordinary type expressions and must name nominal or
built-in types. They classify decimal literals such as `1.2`, zero, positive
integer literals, and negative integer literals respectively. Numeric text is
still resolved through the current lexical scopes first. An explicit local
declaration or fact for a numeric spelling therefore takes precedence; the
corresponding `Specify:` type is consulted only after scope and derived-fact
lookup fail.

The same configuration determines variadic index types. In
`x[i_ := 1...n]`, both `i` and `n` have the `positiveInt` type. In
`x[i_ := 0...n]`, they have the `zeroOrPositiveInt` type. Plain `x...` and
`x...n` are one-based and use `positiveInt`. Consequently an index can be any
ordinary expression whose result has the configured type, not only a bare
name: for example `x[i * j]` is valid when the multiplication result has the
configured `positiveInt` type. Two-dimensional variadics apply the same rule to
both axes and their bounds.

### Variadic command arguments

Only curly argument groups can be variadic. A variadic group occupies the whole
`{...}` group and accepts one or more expressions:

```text
x...
x...n
x[i_ := 0...]
x[i_ := 1...]
x[i_ := 0...n]
x[i_ := 1...n]
```

The plain `x...` form does not name the length. `x...n` names the length without
binding an index. The bracketed forms bind an index and explicitly choose zero-
or one-based indexing; no other starting index is accepted. This is separate
from a magnetic `x__` parameter in parentheses: magnetic parameters collect
arguments into one tuple, while curly variadic parameters spread one or more
arguments through the command.

Reusing a length name constrains groups to the same number of arguments:

```text
[\foo{x...n}:bar{y...n}]  # equal lengths
[\foo{x...n}:bar{y...m}]  # independent lengths
```

Elements are referenced with indexed syntax such as `x[0]`, `x[1]`, `x[i_]`,
or `x[i * j]`.
The supported slice forms are:

```text
x...
x[0...n]
x[1...n]
x[0...i_...n]
x[1...i_...n]
```

A slice broadcasts when it is the left operand of `:=`, `=`, `!=`, `is`, `is?`,
or a quoted specification operator (including its predicate form). A slice may
also appear on the right of `:=`, `=`, or `!=`; paired slices must spell exactly
the same start, binder, and end. In a clause list, the broadcast produces one
clause per element, so `. x[1...n] = 0` inside `anyOf:` means one alternative
for every indexed `x`.

The variadic mapping builtin maps aligned slices pointwise:

```text
\\map{x[1...i_...n]}:to{x[i_] + 1}
\\map{x[1...i_...n], y[1...i_...n]}:to{x[i_] + y[i_] + 1}
```

All input slices must have exactly the same range spelling; multiple slices are
zipped rather than combined as a Cartesian product. The result expression may
use the bound index.

The reduction builtins accept a binary operator and one slice:

```text
\\leftReduce{`+`}:on{x[1...n]}
\\rightReduce{`+`}:on{x[1...i_...n]}
```

They select left and right association respectively and both render with an
ellipsis, such as `x_1 + ... + x_n`. A bound slice also displays its middle term,
such as `x_1 + ... + x_i + ... + x_n`.

Variadic parameters and slices use the corresponding symbolic rendering. In
particular, `x...` renders as `x_1, ..., x_.`, while
`x[1...i_...n]` renders as `x_1, ..., x_i, ..., x_n` (and a zero-based range
starts with `x_0`).

#### Two-dimensional variadic curly arguments

A curly argument group may instead declare a rectangular two-dimensional
variadic parameter. No other argument delimiter supports the 2D form. Both
indices use the same zero- or one-based origin, and the row and column bounds
may be named or omitted:

```text
x[(i_, j_) := (1,1)...(m,n)]
x[(i_, j_) := (0,0)...(m,n)]
x[(i_, j_) := (1,1)...]
x[(i_, j_) := (0,0)...]
```

For example, `\foo{x[(i_, j_) := (1,1)...(m,n)]}` accepts a
semicolon-separated rectangular argument:

```text
\foo{a, b, c;
     x, y, z;
     q, r, s}
```

Commas separate columns and semicolons separate rows. Every row must have the
same nonzero number of entries, and a flat comma-separated argument does not
match a 2D parameter. When present, `m` and `n` receive the row and column
counts. An element is referenced as `x[i,j]`.

A 2D selection gives each axis independently as a single index, a range, or
`...` for the whole axis:

```text
x[a...i_...b, c...j_...d]
x[a...b, c...d]
x[a...b, ...]       x[..., c...d]
x[..., ...]
x[a, ...]           x[..., c]
x[a, c...d]         x[a...b, c]
```

At least one axis must be a range or `...`; `x[a,c]` is the ordinary single
element. A 2D slice broadcasts under the same assignment, equality, inequality,
type, predicate, and quoted-specification relations as a 1D slice. Thus
`x[a...b, c...d] is \real` and `x[a...b, c...d] = 1` apply to every
selected `x[i,j]`. Paired 2D slices must use exactly the same two axis
selections.

In a command's `when:` section, `x is T` constrains the variadic value as a
whole. To constrain its cells directly, write `x[..., ...] is T`; the checker
instantiates that requirement once for every supplied matrix element. The
indexed form `x[i,j] is T` provides the same per-cell constraint while making
the bound row and column indices available to the definition.

### Mapping parameters and specialized signatures

A command heading can select parameters from a mapping declared in another
curly argument group. An exact selector uses the mapping's declared parameter
name:

```text
[\integral{f(x_, y_)}:d{f.x_}]
[\integral{f(x_, y_)}:d{f.y_}]
```

These headings distinguish integration over the first and second parameters.
An arbitrary selector has `?` before its trailing `_` and introduces a fresh
local selector name:

```text
[\integral{f(x_, y_)}:d{f.u?_}]
[\integral{f(x_, y_)}:d{f.u?_, f.v?_}]
```

Here `u` and `v` can match any parameters of `f`. An arbitrary selector name
must not duplicate a declared parameter name: `f.x?_` is invalid when the
mapping is `f(x_, y_)`; use exact `f.x_` or choose a fresh name.

A ranged variadic mapping supports either a fixed number of arbitrary selected
parameters or a variadic subset:

```text
[\integral{f(x_[i_:=1...n])}:d{f.x1?_, f.x2?_}]
[\integral{f(x_[i_:=1...n])}:d{f.x_[i_[j_:=1...m]]}]
```

The second selector denotes the subset
`x_[i_[1]], x_[i_[2]], ..., x_[i_[m]]`. Its outer name and index must match the
associated ranged mapping parameter.

At a use site the mapping parameters must be written explicitly. Command
mapping-literal sugar binds them for the selector group:

```text
\integral[x_, y_ is \real]{x_^2 + y_^2}:d{x_}
\integral[x_, y_, z_ is \real]{x_^2 + y_^2 + z_^2}:d{x_, y_}
```

Every selected value must be one of those bound parameters, so replacing `x_`
with an unrelated `c` is an error. A header using mapping-parameter selectors
must obey all of these rules:

- selectors occur in exactly one curly argument group;
- every value in that group is a mapping-parameter selector and uses an owner
  such as `f.`;
- all selectors in the group have the same owner;
- exactly one other curly group declares that owner with explicit parameters;
- mapping-parameter selectors are currently supported only by ordinary command
  headings, not infix or refined headings.

A selector is already bound through its associated mapping form, so it does not
require a separate `when:` entry. The mapping owner may still require one (for
example, `when: f is \function`), and a selector may appear in `when:` when an
additional refinement is useful, but omission of that selector entry is valid.

In a `Documented:written:` template for one of these headings, the mapping
owner placeholder denotes the mapping body rather than the entire anonymous
mapping, and selector placeholders denote the parameters chosen at the use
site. For example, `written: "\frac{d}{d x_?} f?"` renders
`\d[x_, y_ is \natural]{x_ + y_}:d{x_}` as `\frac{d}{d x} x + y`.

Such a heading has both a specialized and a general signature. Exact positions
are encoded as `#1`, `#2`, and so on, arbitrary positions as `#?`, and a
variadic subset as `#*`:

```text
Heading                                      Specialized signature
\integral{f(x_, y_)}:d{f.x_}                \integral{_(2)}:d{#1}
\integral{f(x_, y_)}:d{f.y_}                \integral{_(2)}:d{#2}
\integral{f(x_, y_)}:d{f.u?_, f.v?_}        \integral{_(2)}:d{#?, #?}
\integral{f(x_[i_:=1...n])}:d{f.x1?_, f.x2?_}
                                             \integral{_(*)}:d{#?, #?}
\integral{f(x_[i_:=1...n])}:d{f.x_[i_[j_:=1...m]]}
                                             \integral{_(*)}:d{#*}
```

All five have the general signature `\integral:d`. Specialized signatures must
be unique, while several definitions may share a general signature. Resolution
first prefers a matching fixed mapping arity over `_(*)`, then exact selected
positions in their written order over `#?`, and finally `#*`. If two candidates
are equally specific, the invocation is ambiguous and is rejected.

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
built-in type expression. Expression-level `is?` accepts command, refined-command,
and supported built-in type predicates; the helper parser differs mainly in its
subject syntax and in the refined-command variant used by theorem-style `given:`
sections.

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

### Other Statement And Expression Forms

- **Infix specification commands** are written `<a> \:name:/ <b>` (with the
  predicate form `\:name?:/`). They are the specification-level analogue of the
  infix command `\.name./` and are declared by an infix-spec `Defines:` header
  `[A \:subset:/ B]`.
- **Function literals** are anonymous functions written with `=>`:
  `(x_ is \real) => x_ + 1`.
- **Spec literals** are `\\specification` values written with an implicit `?`
  subject — `? is \set`, `? "in" X` — and are instantiated by a `satisfies:`
  clause.
- **Inferred parameters** are command arguments written `X?`: the first
  occurrence introduces `X` with the type its position requires; later uses are
  the plain name.
- **`member_of`** (`x member_of X`) is the builtin collection-membership form,
  and **`satisfies`** applies a spec literal to a subject.
- A command may carry a **context suffix** `#using{...}` or `#given{...}`
  supplying that command's `using:`/`given:` values inline.

## Top-Level Groups

These groups may appear at the top level of a document.

Each entry below gives the group, the kind of heading it takes (in
parentheses), and its purpose.

- **`Title`** (no heading) — document title text.
- **`SectionTitle`** (no heading) — first-level prose heading.
- **`SubsectionTitle`** (no heading) — second-level prose heading.
- **`Text`** (no heading) — a prose block (Markdown with embedded LaTeX).
- **`Writing`** (no heading) — collection-wide writing aliases (`:~>`); at most
  one per collection.
- **`Disambiguates`** (operator/function form heading) — global resolution of an
  ambiguous operator or function into typed branches.
- **`Defines`** (command heading) — introduces a command for a mathematical
  form or typed/value-bearing declaration.
- **`Declares`** (command heading) — declares a statement, specification, or type
  fact; marked `abstractly:` it leaves parts of its value for a `Realizes` to
  supply.
- **`Realizes`** (command heading) — a concrete realization of a `Declares`
  marked `abstractly:`.
- **`Refines`** (refined command or refined spec-infix heading) — defines a
  refined command in terms of another command.
- **`States`** (command heading) — defines a named statement with a `that:`
  body.
- **`Axiom`** (optional command heading) — theorem-like assertion.
- **`Theorem`** (optional command heading) — theorem-like assertion.
- **`Conjecture`** (optional command heading) — theorem-shaped assertion that
  is not claimed to have a proof.
- **`Person`** (author heading) — person metadata.
- **`Resource`** (resource heading) — bibliography or web metadata.
- **`Specify`** (no heading) — numeric-domain specification metadata.
- **`Relation`** (no heading) — bidirectional relationship between two concepts,
  topics, or definitions (`between:`/`and:`, with quoted
  `"#topic"`/`"\signature"` references).
- **`Equivalent`** (command heading) — interchangeable commands under a shared
  name (`to:`).
- **`Topic`** (topic heading) — names a documentation topic (`#some.name`);
  optional `within:` parent and `Related:` links (quoted
  `"#topic"`/`"\signature"` references).
- **`TextTheorem`** / **`TextAxiom`** / **`TextConjecture`** / **`TextDefinition`**
  (no heading) — opaque prose placeholders (see below).

### Text placeholders

`TextTheorem:`, `TextAxiom:`, `TextConjecture:`, and `TextDefinition:` each hold a
Markdown-with-LaTeX body standing in for a structured
theorem/axiom/conjecture/definition that will be written later. They are **opaque
to the type-checker** — the body is never parsed as MathLingua, so it may freely
mention commands that do not exist yet.

```text
TextTheorem: "For every group $G$, the identity element is **unique**."
Documented:
. called: "Uniqueness of identity"
. written: "\text{Uniqueness of the identity}"
. description: "A placeholder for the uniqueness theorem."
. notes: "Turn this into a structured Theorem once \group exists."
References:
. $book.algebra
Id: "…"
```

The optional `Documented:` accepts only `called:`, `written:`, `description:`,
and `notes:`; `References:` records the citations to carry into the structured
form; `Id:` is required. `notes:` are prose reminders for the later conversion.
Writing the prose, references, and layout first — then filling in structured
forms — is easier than going from nothing straight to structured groups. A
placeholder renders as a card titled by its `called:`/`written:` (or the kind
word when untitled), with the body as rendered Markdown and `Documented:` behind
the supporting-sections toggle. Its `References:` and `Id:` sections are hidden
behind that toggle as well.

Groups with command headings introduce command signatures: `Defines`,
`Declares`, `Refines`, `States`, `Equivalent`, and theorem-like groups that have
an optional heading. Duplicate signatures are rejected across all of these
definition kinds.

`Defines`, `Declares`, and `States` must include a `called:` **or** `written:`
item somewhere in their `Documented:` section. `Refines` must instead include an
`adjective:` item (and its `Documented:` rejects `called:`). Theorem-like groups
(`Axiom`, `Theorem`, `Conjecture`) may have documentation but require no such
item.

## Definition Groups

`Defines` introduces the form or declaration associated with a command. Its
target may include an `is` type relation and an optional `:=` value definition.

```text
[\set]
Defines: X
Documented:
. called: "set"
```

Optional sections, in order:

```text
using:
when:
extends:
means:
satisfies:
Requires:
Enables:
Documented:
Justification:
Aliases:
References:
Metadata:
```

(`Refines` uses `extends:`/`satisfies:` rather than `means:`. `Defines`,
`Declares` and `Realizes` each have a `means:` section, with different rules for
what its items may say — see below.)

`Declares` introduces a command by an `is` or spec statement.

```text
[\foo{s}]
Declares: x is \bar{s}
Documented:
. called: "foo"
```

It accepts `abstractly:`, `using:`, `when:`, `means:`, `expresses:`, and the same
support sections as `Defines`.

A declaration whose target destructures a tuple says what each part is in
`means:`:

```text
[\foo]
Declares: X ::= (A, B, C)
means:
. A := ...
. B := ...
. C is \real.function
expresses:
. piecewise: ...
```

Every item must supply a **value**, either directly with `:=` or indirectly:
`C is \real.function` states the type and a `piecewise:` in `expresses:` says
what `C` is. An item that states a specification and never defines it is an
error (``\`C\` states a specification but no value; define it with \`:=\`, define
it in \`expresses:\`, or mark this \`Declares:\` \`abstractly:\``).

### Abstract declarations and `Realizes`

`abstractly:` — a zero-argument marker placed immediately after `Declares:` —
lets a declaration state a concrete thing with abstract parts. Its `is`/`"op"`
`means:` items are then *left open* for a realization to supply:

```text
[\naturals]
Declares: Nb ::= (N, 0, succ(n_))
abstractly:
means:
. N is \set
. 0 "in" N
. succ is \function:on{N}:to{N}
```

A `Realizes` supplies them:

```text
[\von.neumann.naturals]
Realizes: Nb := \naturals
means:
. N := ...
. 0 := ...
. succ(n_) := ...
```

The three definition kinds line up with a familiar hierarchy: a `Defines` is an
interface, a `Declares` marked `abstractly:` is an abstract base class, and a
`Declares` without it is a concrete one.

- The `Realizes:` target names the declaration it realizes with `:=`, not `is`.
  That is the general rule: a `Defines` names a **type**, so its values are
  written `X is \foo`; a `Declares` names a **value**, so it is written
  `Y := \bar`.
- The named command must be a `Declares:` marked `abstractly:`
  (``\`Realizes:\` must name a \`Declares:\` marked \`abstractly:\``).
- The `means:` must supply **every** symbol the declaration left abstract;
  omitting one is an error (``Missing realization for abstract symbol
  \`{symbol}\``). As in a concrete `Declares`, a `means:` item may state a type
  and leave the value to `expresses:`.
- A realization's components keep the types its declaration gave them, so
  destructuring a realized value types them exactly as destructuring the
  abstract declaration does.

`Realizes` accepts the same support sections as `Declares` (`using:`, `when:`,
`means:`, `expresses:`, `Requires:`, `Enables:`, `Documented:`, and the rest).

`Refines` introduces a refined command.

```text
[\(continuous)::function:on{A}:to{B}]
Refines: f(x__)
Documented:
. adjective: "continuous"
```

For ordinary refined command headings, the rendered `Refines:` value makes the
inferred base type explicit even though the source need not repeat it. For
example, `[\(finite)::group]` with `Refines: G ::= (X, *, e)` is displayed as
`G ::= (X, *, e) is \group`. If the source already states an `is` relation, the
renderer does not add another one. Parameterized base types are preserved too:
under `[\(commutative)::binary.operation:on{X}]`, `Refines: x_ * y_` is
displayed as `x * y is \binary.operation:on{X}`.

A specification operator declared with a `\:...:/` heading can be refined with
the same parenthesized refinement syntax. The base operator is inferred from
the heading, just as it is for an ordinary refined command:

```text
[A \:(nonempty)::subset:/ B]
Refines: A
when: B is \set
Documented:
. adjective: "nonempty"
. written: "A? \subset_{+} B?"
```

The refined operator can then be used in declarations and expressions:

```text
. X' \:(nonempty)::subset:/ X
```

An explicit refined spec-infix definition is not required when the refinement
is inherited through the type the base operator extends. If `\:subset:/`
extends its left operand to `\set` and `\(nonempty)::set` is defined, then
`X' \:(nonempty)::subset:/ X` resolves implicitly. It establishes both the base
`X' \:subset:/ X` fact and `X' is \(nonempty)::set`.

Ordinary refined types keep their adjectives before the rendered type. A single
refinement such as `\(a)::type` renders as `<a> <type>`, while
`\(a, b, c)::type` renders as `(<a>, <b>, and <c>) <type>`. Refined spec-infix
forms use a different order: the rendered base relation comes first, followed by
the refinements in parentheses. Thus `X \:(a, b, c)::relation:/ Y` renders as
`<X relation Y> (<a>, <b>, and <c>)`. An explicit `written:` template remains an
override for custom notation.

#### `Refines` Refinement Markers (`implicitly:`/`explicitly:`)

A refinement of a base type is automatically available on that type's subtypes:
if `\(finite)::magma` refines `\magma` and `\group` extends `\magma`, then
`\(finite)::group` is already implied (a finite group is just a finite magma
whose carrier is a group). Authors may still want to write the
`\(finite)::group` refinement out explicitly — for documentation, or to give it
extra properties. Two optional, mutually exclusive, zero-argument marker sections
placed immediately after `Refines:` state the author's intent, and the checker
verifies the body is consistent with the marker:

```text
[\(finite)::group]
Refines: G(x__)
implicitly:
extends: G is \(finite)::magma
Documented:
. adjective: "finite"
```

- `implicitly:` — the group merely restates the inherited definition. The body
  must contain **nothing beyond** the inherited `extends:` clause (the scaffolding
  `using:`/`when:` sections are allowed). Adding `satisfies:`, `Requires:`,
  `Enables:`, or `Justification:` is an error — mark it `explicitly:` instead.
  Furthermore, the `extends:` clause must **literally name the parent type's
  refinement**: the same adjective(s) applied to a supertype of the refined base
  type (above, `\(finite)::magma`, because `\group` extends `\magma`). An
  `extends:` clause that names anything else is an error.
- `explicitly:` — the group overrides the inherited definition with stronger
  meaning, so it must add **at least one** property beyond the inherited
  `extends:` clause (for example a `satisfies:` section). If the body is only the
  inherited `extends:`, that is the trivial case and must be marked `implicitly:`.

Both markers are only meaningful when the refined base type is itself a subtype
of another type (its own definition names a type it extends). Using either
marker on a base that is not a subtype of anything is an error. When the base is
not a subtype, no marker is written at all.

A `means:` section (only `Defines` has one) types the described form's
parts — including the components of a destructuring target — and those facts are
assumed when checking the definition body and stored so a value of the type
carries them (see [Subtyping: The Type A `Defines` Extends](#subtyping-the-type-a-defines-extends)).

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

`Axiom`, `Theorem`, and `Conjecture` share the same assertion shape. `then:` is
required. `given:`, `where:`, `iff:`, and support sections are optional. A
`Conjecture:` records an unproved statement. Its optional `Justification:` can
capture supporting rationale or evidence, but does not claim a proof.

The head section (`Axiom:`/`Theorem:`/`Conjecture:`) takes no argument. A
result's name is given in `Documented:` `called:`, exactly as for the definition
items, and renders as the card's title (whether or not the item has a command
heading).

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

- **`not`** — checks the nested clause in the current context.
- **`allOf`** — checks all children; when assumed, gathers facts from children.
- **`anyOf`** — checks all children.
- **`oneOf`** — checks all children.
- **`exists`** — creates a child context from its declaration and assumes
  optional `suchThat:` clauses.
- **`existsUnique`** — same as `exists`, with unique-existence intent.
- **`forAll`** — creates a child context, assumes `where:`, checks `then:`.
- **`let`** — introduces one or more local bindings in a child context, assumes
  an optional `where:` guard, and checks `then:` with those bindings and
  assumptions available.
- **`if`** — assumes `if:`, checks `then:`.
- **`have`** — assumes `iff:`, checks `have:`.
- **`equivalently`** — a chain of biconditionals — sugar for pairwise `iff`.
- **`piecewise`** — assumes `if:`, checks `then:`; the optional `else:` is checked
  in the outer context.
- **`given`** — assumes one refined-capable given statement (optional `where:`),
  then checks `then:`.

For example, a `let:` clause can introduce a member for use in its body:

```text
. let: n "in" X
  where: n != x
  then: n = n
```

Each of these clauses also has a builtin *command* form used inline in a
statement position: `\\not{...}`, `\\allOf{...}`, `\\anyOf{...}`, `\\oneOf{...}`,
`\\forAll{...}:then{...}`, `\\have{...}:iff{...}`, `\\given{...}:then{...}`, and
`\\piecewise{...}:then{...}:else{...}`. They are checked with the same scoping
rules as the corresponding groups, and a malformed one (wrong arity, missing
required tail, or unknown builtin clause command) is reported.

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

Quantifier and `let:` declarations are local to the clause group that introduces
them.

## Support Sections

`Requires:` accepts:

- `capability:` groups, which define notation that is part of the construct's
  definition
- `definition:` groups of the form `\command is <spec>`, which require the
  referenced command to be a top-level `Declares:` entry whose definition
  establishes the requested fact

`Enables:` accepts:

- `capability:` groups, which define additional notation made available by the
  construct
- `from:` plus `capability:`, which defines notation made available by a cast
  source
- `from:` plus `as:`, which defines how facts from a cast source are viewed as
  facts about the described form
- `relation:` groups, which record relationships to another declaration and
  may opt into type-system cast behavior with `represents: \\coercion` or
  `represents: \\encoding`

A `capability:` left-hand side may be a spec (`x_ "in" X :-> …`), an operator
form (`x_ * y_ :=> …`), a command, or a **member access** — `x.inv` — or
**member call** — `x.f(a_) :=> …`. For a member form the owner (`x`) must be
exactly the described subject; a use like `p.inv` or `p.f(v)` on a value of the
construct's type then reduces to the capability's target.

For type checking, capabilities from `Requires:` and `Enables:` are combined.
The separate sections are for communication: `Requires:` describes what the
construct has by definition, while `Enables:` describes further operations that
the construct supports.

`Documented:` accepts:

- `written:`
- `called:`
- `adjective:` (required by `Refines`)
- `description:`
- `writing:`
- `overview:`
- `related:`
- `discoverer:`

`Justification:` (placed after `Documented:`) accepts only `have:`/`asserting:`
groups, each with a required `[label]` heading. Any grouped expression, statement,
or specification elsewhere in the group may carry a `[:label:]`, including inside
a nested clause or expression. For example, both a `means:` item
`(.x is \foo.)[:1:]` and a `satisfies:` clause
``(.*' := \restriction:of{`*`}:on{X' \.set.cross./ X'}.)[:2:]`` may reference
entries. A matching entry establishes the labeled formulation using its
`asserting:` items (exactly as an inline `have:`/`asserting:` group would); its
`have:` must restate that formulation, an unmatched label is checked inline as
usual, and every entry must be referenced by some labeled formulation.

`Aliases:` accepts `alias:` groups. `Metadata:` accepts `id:` and `version:`.
`References:` contains resource references such as `$book.chapter` (the quoted
form `"$book.chapter"` is accepted too). A resolved reference renders as the
resource title followed by its authors in parentheses. If the resource has a
URL, the citation links to it. PDF resources also accept `:page{n}`; the link's
physical PDF page is `n + offset - 1`, where `offset` defaults to `1`.
In the viewer, `References:` is a supporting section and is collapsed by
default alongside `Documented:` and `Id:`.

### Mapping writing templates

A documented `writing:`/`as:` group controls how a mapping introduced by
`Defines:` is rendered. It is valid only when the `Defines:` target is a
mapping. The `writing:` value must be either the exact mapping form or that same
form with each placeholder replaced by its ordinary name:

```text
[\real.sequence]
Defines: X ::= x(i_)
Documented:
. writing: x(i)
  as: "x?_{i?}"
. writing: x(i_)
  as: "\left\{x?\right\}_{i_?=1}^{\infty}"
```

The ordinary form (`x(i)`) renders invocations such as `x(j)`, substituting
`x?` and `i?` with the invoked mapping and argument. The placeholder form
(`x(i_)`) renders the mapping itself, such as in
`x(i_) is \real.sequence`; its template may use the placeholder key `i_?`.
The `as:` value is raw LaTeX rendering text.

When the target has a declaration alias, as in `X ::= x(i_)`, the two
`writing:` forms still use the mapping name `x`, not the alias `X`. The same
rule applies when an output placeholder is present:
`X ::= x(i_) ::= y_` still uses `x(i)` and `x(i_)`.

`Person` groups use author headings such as `[@ada.lovelace]` and require
name text on `Person:` with optional `biography:`. `Resource` groups use resource headings such as
`[$principia]` and contain resource item groups like `title:`, `author:`,
`url:`, and `year:`. For PDFs, `offset:` is the physical page on which the
document's numbered page 1 appears.

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

Collection-wide `Writing:` aliases use `:~>` and apply to plain names. They are
separate from documented mapping writing templates.

```text
Writing:
. "alpha :~> \alpha"
```

Each alias is a double-quoted string. Quotes are required because the body after
`:~>` is raw non-empty LaTeX, not parsed formulation syntax, and could otherwise
be misread as a formulation.

Every top-level non-prose item (`Defines:`, `Declares:`, `Refines:`, `States:`,
`Axiom:`, `Theorem:`, `Conjecture:`, `Relation:`, and `Disambiguates:`) may carry
its own optional `Writing:` section immediately after `Aliases:`. It uses the same
double-quoted `"name :~> body"` form and overrides, for that item only, how the
named identifiers render — layered on top of the collection-wide `Writing:` group.

```text
[\natural]
Defines: n
Documented:
. called: "natural"
Writing:
. "pi :~> \varpi"
```

## Checks Performed by `mlg check`

`mlg check` runs a fixed pipeline over the collection and reports every problem
it finds as a `Level::Error` diagnostic (the checker itself emits no warnings).
It finishes with either `Checked N file(s)` or `Found N issue(s).`. Before
checking, unless `mlg.json` sets `"formatOnCheck": false`, the collection is
formatted first (`Formatted N file(s)`), and any top-level item missing an `Id:`
has one generated and written back.

The stages below run in order; each is enumerated so this section is the
authoritative catalog of what the tool validates. Error text is shown as a
template with `{placeholders}`.

### 1. Configuration (`mlg.json`)

Every field is required and typed. Checks: file is readable
(`Failed to read mlg.json: {error}`), valid JSON (`Invalid JSON in mlg.json`),
a JSON object (`mlg.json must be a JSON object`); each required field present
(`mlg.json is missing required field "{field}"` for `name`, `version`,
`margin`, `formatOnCheck`, `outputDir`); `name`/`version` are strings, `margin`
is a positive integer, `formatOnCheck` is a boolean, and `outputDir` is a
non-empty relative path inside the collection. The removed key `print_margin`
reports `... was renamed to "margin"`.

### 2. Collection, directory, and `toc` structure

An explicit target must be a `.mlg` file (`Not a .mlg file`), a real filesystem
entry (`Unsupported filesystem entry`), and part of the collection
(`Path is not part of the source collection`). Every directory's `toc` file is
validated against its actual children: no `Duplicate toc entry \`{name}\``,
every listed name resolves (`toc entry \`{name}\` does not match an existing
.mlg file or directory`), and every real child is listed (`Directory toc is
missing entry \`{name}\``). `toc` lines must be well formed
(`Missing toc file name`, `toc entries must be direct .mlg file or directory
names`, `toc entry title cannot be empty`).

### 3. Parsing

Parsing has three layers, each with its own diagnostics:

- **Proto (line/indentation)** — line shape, section indentation, `:` on
  section lines, argument bullets. Single-quoted formulations are rejected
  (`Single-quoted formulations are not allowed`).
- **Structural (groups/sections)** — each group must match its section
  pattern: a required (non-`?`) section that is missing reports
  `Expected section \`{name}\``, an out-of-order section reports `Expected
  \`{a}\` but found \`{b}\``, and an extra section reports `Unexpected section
  \`{label}\``. Unknown top-level heads report `Unexpected top-level group
  \`{other}\``. Clauses, formulations, headings, and text bodies each have
  "expected …" and "invalid …" diagnostics; `Refines` documentation rejects
  `called:` (`use adjective:`), `Topic` documentation accepts only `called:`,
  and `represents:` entries must be `\coercion` or `\encoding`. `Disambiguates`
  headings and branches have their own structural rules.
- **Formulation (expressions/forms)** — token errors (`invalid token`,
  `unexpected {token}; expected {expected}`) and construct-specific errors such
  as `command headers must start with \`\\``, `expected top-level \` is \``,
  `expected top-level \` is! \``, `expected top-level \`:=\``, `... \` via \``,
  `... \`:=>\``, `... \`:->\``, `Invalid clause expression in \`{label}\`:
  {error}`, and set/placeholder/operator shape errors.

### 4. `mlg` code fences in prose

A ```` ```mlg ```` fenced block inside a `Text:` value (or documentation prose)
is parsed as real source; a failure reports `Syntax error in \`mlg\` code block:
{message}`. ```` ```mlg-fragment ```` and non-`mlg` fences are skipped.

### 5. Identifiers, uniqueness, and single-instance items

Every top-level item must have `Id:` (`Top-level item must include an \`Id:\`
section`) holding a quoted UUID (`\`Id:\` section must contain a quoted UUID`,
`\`Id:\` value \`{value}\` must be a UUID`); Ids must be unique
(`Duplicate Id \`{value}\`; first used at {loc}`). At most one top-level
`Writing:` item is allowed. Command signatures must be unique across definition
kinds (`Duplicate command signature \`{sig}\` ...`); mapping-parameter headers
use their specialized signature for this check, so distinct specialized
signatures may share one general signature. Each operator/function key may have
at most one `Disambiguates` (`Duplicate disambiguation for
\`{key}\``). Spec-infix headings (`\:...:/`) are allowed only on `Defines`, and
refined headings (`::`) only on `Refines`.

### 6. Documentation requirements

`Defines`, `Declares`, and `States` must include a `called:` **or** `written:`
item in `Documented:` (`{kind} entries must include either a \`called:\` or a
\`written:\` item in \`Documented:\``). `Refines` must include an `adjective:`
item instead (`Refines entries must include an \`adjective:\` item ...`).

### 7. Command references and argument shapes

Every command-like reference must resolve to a defined signature
(`Undefined command signature \`{signature}\``) with a matching argument shape
(`Command signature \`{sig}\` expects argument shape \`{expected}\` but found
\`{actual}\``); refined references may fall back to their base command and
refinement pieces. Mapping-parameter references resolve among the specialized
definitions for their general signature and report an ambiguity when no unique
most-specific candidate exists. Command `when:`/context requirements are
checked at use
sites (`Could not establish requirement \`{fact}\` for command \`{signature}\``,
plus `Command ... does not accept ...`, `Unknown ... parameter ...`,
`Missing ... value for parameter ...`). This includes the reduction target of an
`Enables:` `capability:` — both the `:->` form (`x_ "in" G :-> x_ is
\group.element:of{G}`) and the `:=>` form — so a capability that reduces to an
undefined command is reported.

### 8. Target-symbol specification (`Defines`/`Declares`)

Every parameter and target symbol a definition introduces must be given a type.
A header parameter needs a `when:` fact (`Missing \`when:\` requirement for
parameter \`{parameter}\``) — except the symbol the definition describes. A
spec-infix heading such as `[A \:subset:/ B]` is sugar for a command whose left
operand is the symbol being defined, so `A` is stated by the `Defines:` target
and may not appear in `when:` at all (`\`when:\` requirement for \`A\` is not
allowed because \`A\` is what this definition describes; state its type on the
definition's target instead`). `when:` says only what a *use* of the command
requires of the remaining operands. A `Defines` target symbol must be typed directly
or through the type the definition extends (`Missing specification for
target symbol \`{symbol}\`; specify it directly or through the type the
\`Defines:\` target extends`). A `Refines` target symbol may also be inherited: a
symbol the refined base type already declares (through the type that base
extends, its `means:`, or its described components) counts as specified, so
`\(associative)::binary.operation:on{X}` need not respecify the `*` that
`\binary.operation:on{X}` already types. Specification is **exactly once**: a
symbol the subtype clauses already type — including the components a `via`
reaches — may not be typed again in `means:` (`Duplicate specification for target
symbol \`{symbol}\`; it is already specified by ...`). All of a group's subtype
clauses count as one source, so two `extends:` clauses may name the same subject
or reach the same component through different views. A `Declares` target symbol must be
assigned (`Missing definition for target symbol \`{symbol}\`; assign it with
\`:=\` ... or top-level \`expresses:\``) at most once (`Duplicate definition for
target symbol ...`), and a `Declares` value **must state its type** — either
`... is <type>` or a top-level build `\<type>@<value>`:

```text
` `Declares:` target `X` must state its type: use `... is <type>` or a
  top-level `\...@...` build (e.g. `\set@{...}`) `
```

A bare `X := {…}` is rejected even when the type is inferable.

### 9. `when:` clauses

`when:` may only constrain the definition's own parameters (`... is not allowed
because \`{subject}\` is not a parameter of this definition`) and only with
`<subject> is <type>` or `<subject> "op" <target>` forms.

### 10. Specifications vs. statements

An `is` specification or infix specification (`\:...:/`) *introduces* a symbol,
so it is only allowed in binding positions (`exists:`, `given:`, `forAll:`,
`let:`, `where:`, `when:`, `suchThat:`). In a statement position (`then:`, `iff:`,
`that:`, `if:`, `not:`, `allOf:`, …) it is rejected in favor of the predicate
form `is?` / `\:...?:/` (`An \`is\` specification introduces a symbol and is only
allowed in \`exists:\`, \`given:\`, \`forAll:\`, or \`let:\`; use the statement form
\`is?\` here`).

### 11. Symbol scope

Every referenced symbol must be introduced before use, or
`Unrecognized symbol \`{name}\`` is reported. Introduction sites are listed in
[Symbol Scope](#symbol-scope) below. Note `:=` does **not** introduce symbols —
its right-hand side must already be in scope — whereas `::=` does.

### 12. Spec facts and operators

A spec fact `x "op" T` is valid only when `T`'s type provides that operator
(`Could not validate spec fact \`{fact}\`: no provided spec operator \`"{op}"\`
is available for \`{target}\``); infix spec signatures must be defined by
`Defines`. Operators resolve in order: an in-scope value applied as a call; a
colon-qualified provided-symbol capability owned by a single operand type; a
`Disambiguates` entry; then a provided-symbol capability owned by the operands'
common type (with spec-known operands reduced to their `is`-facts). If none
apply, `Could not resolve {label}: no matching \`Disambiguates\` entry was found`
(or `Could not resolve operator \`{symbol}\` from {source}`). Member access
reports `Could not resolve member \`{name}\` for \`{owner}\``.

### 13. Capabilities, requirements, and casts/builds

`Requires:`/`Enables:` capability aliases are validated: a provided spec
operator's target must be the described item; a `Required definition` must
reference a `Declares:` entry and establish its stated fact. Build expressions
`\<type>@<value>` (coercion) and `\<type>@!<value>` (coercion + encoding) are
checked (`Could not build \`{expression}\``). Type predicates, function-type
specs, and `is` type arguments are checked
(`Could not establish predicate ...`, `Could not establish requirement ... for
function ...`, `\`{name}\` is not a known type`).

### 14. Group-specific validations

- **`Equivalent:`** — every `to:` item must be a command that uses the header's
  parameters directly (not expressions); all items must be the same definition
  kind; and their target shape, defined type, `when:` requirements, extended
  type, and capabilities must agree across members.
- **`Defines:` extended type** — a `Defines:` target that states an `is`/spec
  relation may not also have an `extends:` section; the two spellings are
  equivalent, so exactly one of them carries the clauses.
- **`Refines:` `extends:`** — the `extends:` subject must match the `Refines:`
  subject, a `[[...]]` in it must name that subject, and a `Refines:` must have
  the form `Refines: <form>`.
- **`Refines:` refinement markers** — the optional `implicitly:`/`explicitly:`
  marker sections must take no arguments and are mutually exclusive. Either
  marker requires the refined base type to be a subtype of another type;
  `implicitly:` additionally requires the body to contain only the inherited
  `extends:` clause (no `satisfies:`/`Requires:`/`Enables:`/`Justification:`) and
  that clause to literally name the parent type's refinement (the same
  adjective(s) applied to a direct supertype of the refined base type), while
  `explicitly:` requires at least one such property beyond the inherited
  `extends:` clause.
- **Function literals** — a function-literal parameter must be a name with a spec
  (`(x_ is ...)`), or a bare name whose type is already known from an `is`;
  otherwise it is rejected.
- **Function types** — function-type spec parameters must be written `_`
  (`Function type parameters must be \`_\``), and a call must match the function's
  arity (`Could not match function \`{name}\` with {n} argument(s)`).
- **`satisfies`** — its right-hand side must be a specification
  (`\`satisfies\` requires a specification on the right-hand side`).
- **Spec-operator targets** — the target of a `:->` spec operator must be a
  value, not a type (`the target of a spec operator must be a value, not the type
  \`{signature}\``).
- **Inferred parameters** — an `X?` argument may introduce `X` only once
  (`Inferred parameter \`{name}\` is already introduced`).

### What the checker does *not* do

It is not a proof checker: it never verifies that a theorem's conclusion is
true. It checks that every formulation parses, references defined commands with
the right shapes, uses only introduced symbols, and satisfies the stated
requirements of the commands and operators it mentions.

## Symbol Scope

The checker is intentionally conservative about undeclared variables.

Symbols are introduced by:

- command header forms in definition and named theorem-like groups
- the main `Defines:`, `Declares:`, and `Refines:` subjects
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
Defines: G ::= (X, *, e) is \set via X
means:
. * is \function:on{X}:to{X}
. e "in" G
```

the `Defines:` form introduces `G`, `X`, `*`, and `e`. `via X` types `X`, so
only `*` and `e` remain for `means:`. The structural symbols are
specified outside of `when:`, and `via X` is a recognized structural view.

## Type Facts and Requirements

A command definition may declare requirements through `using:` and `when:`.
Those requirements must be provable whenever the command is used as an
expression or predicate, and whenever a parameterized command type expression is
used in an `is` statement.

```text
[\function:on{A}:to{B}]
Defines: f(x__)
when: A, B is \set
Documented:
. called: "function"
```

A later reference to `\function:on{G}:to{G}` in either an expression or a type
assertion requires the checker to prove `G is \set`. Type assertions for
no-argument `Defines` commands are nominal: `G is \group` records that fact
without expanding the internal `\group` requirements at the assertion site.

Literal command arguments are compatible with a required nominal type when
their declared structure matches. For a mapping literal, the checker compares
the number and types of its inputs and the type of its output with the mapping
shape described by the required type. Tuple literals are compared by component
count and positional component types. Set literals are compared with the
required type's described element pattern and element types. This structural
check applies only to literals; named values continue to require ordinary type
facts.

When a mapping input or output is specified by membership in a literal-backed
collection, the collection's element specification participates in this check.
Thus a parameter known to be `\natural` satisfies an expected `"in"
\naturals.set` spec when `\naturals.set` describes exactly those elements.
The mapping body is checked after resolving type-directed operators, so an
overloaded expression such as `x + y` contributes the output type of the
selected addition operator. This also works when the operator is declared as a
callable type: an addition declared as `\binary.operation:on{\naturals.set}`
inherits its application result from the codomain of the function type it
extends.

The checker understands these fact kinds:

- type facts, such as `G is \set`
- spec facts, such as `x "in" G`

It also has built-in types for meta-level checks. In particular, `\\type`
holds for command references whose top-level entry is a `Defines:` item.
Thus `\set is? \\type` succeeds when `\set` is described, while
`\sqrt is? \\type` fails when `\sqrt` is a `Declares:` item.

The built-in type `\\opaque` is satisfied by any declared value. It is useful
when a definition only needs an argument to exist but should not learn anything
about that argument. A fact such as `A is \\opaque` does not imply `A is \set`,
does not enable set capabilities, and does not otherwise contribute concrete
type information.

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

## Subtyping: The Type A `Defines` Extends

A `Defines` group states the type it extends in one of two equivalent ways: on
the `Defines:` target itself, or in an `extends:` section. Both introduce the
same subtype and extension implications, and **a group may use one or the other
but not both**.

The target spelling is the short one, and is what most definitions want, because
the subject is written once:

```text
[\group]
Defines: G ::= (X, *, e) is \set via X
means:
. * is \function:on{X}:to{X}
. e "in" G
Documented:
. called: "group"
```

This means that if the checker knows `G is \group`, it can prove `G is \set`.
The implication is recursive, so subtype chains are followed.

`Defines: X is \foo` and

```text
Defines: X
extends: X is \foo
```

mean exactly the same thing. The `extends:` section exists for what a target
cannot express: **extending several types at once**, typically through different
views of the same tuple.

```text
[\foo]
Defines: X ::= (A, B, C)
extends:
. X is \bar via (A, B)
. X is \baz via (B, C)
```

The type may name a refined command, as in
`Defines: S ::= (X, *) is \(finite)::magma via (X, *)`.

An `is` relation may be followed by `via`, which both documents the view used to
regard the subtype as the supertype **and sets the types of the `via` symbols**,
so they need not be repeated in `means:`:

- `M ::= (X, *) is \set via X` — a single `via` symbol becomes an instance of
  the extended type, i.e. it records `X is \set`.
- `S ::= (X, *) is \magma via (X, *)` — a `via` tuple maps positionally onto the
  extended type's own components, so `X` and `*` inherit the types `\magma`
  gives its components (`X is \set`, `* is \binary.operation:on{S}`).

Because `via` supplies those types, the `means:` section only needs to type
components no `via` covers (for `\group` above, `*` and `e`) — and it may not
retype the ones a `via` does cover, since a symbol is specified exactly once. A
`via` without an `is` relation to accompany it is an error.

An operator-form target states its type about the operator itself, so
`Defines: x_ * y_ is \function:on{X \.cross./ X}:to{X}` records the fact about
`*`.

### Destructuring targets

A `Defines`/`Declares` target, a command parameter, or a `given:`/`using:`
binding may destructure a tuple with `::=`:

```text
Defines: M ::= (X, *)
[\magma.element:of{M ::= (X, *)}]
given: M ::= (X, *) is \magma
```

The component names (including operator components like `*`) are introduced, and
their types are inferred: from the extended type's `is`/`via` and then
`means:` for a `Defines` target; from the parameter's `when:` type for a command parameter;
and from the right-hand type for a `given:`/`Declares:` binding. Components typed
this way do not each need a separate `when:` entry, and member access reaches
them (`M.X`, `M.*`). Only `::=` introduces these symbols — `:=` requires its
right-hand side to already be in scope.

A subtype clause may also state a spec relation instead of an `is` relation, in
either spelling:

```text
[\element:of{X}]
Defines: x "in" X
when: X is \set
```

That records the corresponding spec fact as an implication for values of the
owning type.

Function-like types can describe their call behavior with a function type on
the right-hand side of an `is` statement:

```text
[\function:on{A}:to{B}]
Defines: f(x__) is (_ "in" A) -> (_ "in" B)
when: A, B is \set
Documented:
. called: "function"
```

The input side contains one or more specs and the output side contains exactly
one spec. Both sides must be parenthesized, and each spec parameter must be
written as `_`. If the checker knows `f is \function:on{A}:to{B}` and
`y "in" A`, it can validate `f(y)` and prove `f(y) "in" B`.

Structural type literals use complete spec literals at every leaf. This keeps
higher-order specification operators intact instead of treating every
component as an ordinary nominal type:

```text
(? is \natural, ? "in" \reals)
{? is \natural : ...}
{(? is \natural, ? "in" \reals) : ...}
(? is \natural) -> (? "in" \naturals)
(? is \natural, ? "in" \reals) -> (? is \real)
```

These types match `(x, y)`, `{x : ...}`, `{(x, y) : ...}`, `x_ => ...`, and
`(x_, y_) => ...`, respectively. Each `?` is instantiated with the matching
term component. A raw nominal tuple such as `(\natural, \real)` is not a type
literal; write `(? is \natural, ? is \real)` instead.

The `->` form accepts one or more input spec literals and exactly one
output spec literal, so users can state unary, binary, and n-ary function types
directly. The checker retains that input arity rather than requiring users to
write nested unary function types. Function literals use `=>`, keeping type and
value syntax distinct while mirroring the `:->` and `:=>` operator forms.

For a function definition, component declarations and a whole-function
declaration are equivalent alternatives:

```text
Defines: f(x_) ::= y_
means:
. x_ is \real
. y_ is \real
```

```text
Defines: f(x_) ::= y_
means:
. f is (? is \real) -> (? is \real)
```

## Specification Operators

Required and enabled specification capabilities connect notation such as
membership to type facts or other spec facts.

```text
[\reals]
Defines: R
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
`\element.of:group{G}` only when `G is \set` is available, commonly because
`\group` is defined as `Defines: G ::= (X, *, e) is \set via X`.

Direct spec requirements are also supported once the target type requires or
enables the operator. If `\group` has `x_ "in" G` as a capability and a command
requires `x "in" G`, then an exact matching spec fact in the context satisfies
that requirement even without reducing it to a type fact. A raw fact such as
`x "in" G` is invalid when the checker knows `G` has a type that does not enable
`"in"`.

### Build-Backed Capabilities

`Enables:` may use a `from:` group to describe capabilities supplied by a built
literal rather than by the opaque type itself.

```text
[\set]
Defines: X
Requires:
. capability: x_ "in" X :-> \\abstract
Enables:
. from: Y ::= {y__ : ...}
  capability: x_ "in" X :-> x_ member_of Y
Documented:
. called: "set"
```

If a value is introduced as `A := \set@{x_ : x_ is \real}`, the checker
records the literal for `A`. When it later reduces `a "in" A`, the `from:`
capability substitutes the source subject `Y` with `A`, producing
`a member_of A`. The existing `member_of` reducer then reads the built literal
and can establish `a is \real`.

An ordinary non-`from:` capability on an opaque target does not read a built
literal through `member_of`. For example, `Defines: X` with
`capability: x_ "in" X :-> x_ member_of X` does not make
`\set@{...}` expose the literal's element facts. Use a structural target such as
`Defines: X ::= {x__ : ...}` or an explicit `from:` capability for that.

A `from:` group may also use `as:` with an expression binding, for example:

```text
Enables:
. from: P ::= {(p_, q_) : ...}
  as: f(p_) := q_
```

This records and validates the view from the source structure to the
described form. If `F := \function@{(p_, q_) : q_ is \set}`, the binding
lets the checker use facts about `q_` from the source literal as facts about
`F(p_)`; for example it can establish `F(a) is \set` when the source literal
supports that substitution.

### Build Expressions (`\type@value` and `\type@!value`)

An expression may build a value at a stated type using a command type followed
by `@` (soft) or `@!` (hard) and the value:

- `\type@value` — a **soft build**. It succeeds when `value` already has that
  type, has a parent type that extends to it, or the value's type (or a parent)
  has an `Enables:` `relation:` to `\type` marked `represents: \\coercion`.
- `\type@!value` — a **hard build**. It performs the same checks and
  additionally allows `relation:` groups marked `represents: \\encoding`. Use
  `@!` when the value is being viewed at a lower abstraction level.

```text
X := \set@{x_ : x_ is \real}
n := \rational@k
s := \set@!m
```

The old `value as \type` / `value as! \type` cast syntax has been removed;
`\type@value` and `\type@!value` replace them. These are also the only way to
state a `Declares:` value's type without `is` — a top-level build such as
`X := \set@{...}` is sugar for `... is \set` (see the target-symbol check
above). A build whose value cannot be viewed at the requested type reports
`Could not build \`{expression}\``.

The related symbol-introduction forms are `is` and `is!`: `x is \type`
introduces `x` with a soft view (coercion) of the type, and `x is! \type`
introduces it with a hard view (coercion + encoding). They are the named
counterparts of `@` and `@!`.

`Enables:` may contain `relation:` groups that back builds:

```text
[\integer]
Defines: n
Enables:
. relation:
  to: r := \rational@n is \rational
  when: n is \integer
  means: n \.embedded.to./ r
  represents: \\coercion
```

The `to:` declaration states the target type using `is`. The `:= ...`
construction is optional; without it, the relation is accepted but the converted
value is opaque. The optional `when:` section can contain ordinary declarations
and hard-view declarations such as `a0 := a is! \set`. The optional `means:`
clause records a statement relating the original value and the viewed value.

Relations marked `\\coercion` are used when checking whether an already-resolved
command's arguments satisfy its requirements. For example, if `\integer` has a
relation to `\rational` marked `\\coercion`, then a command requiring
`x is \rational` may accept an integer argument, and `\rational@k` succeeds for
an integer `k`. These relationships are **not** used for operator resolution:
`+` on integers will not resolve to `+` on rationals merely because integers can
be viewed as rationals.

Relations marked `\\encoding` are used only by hard builds (`@!`) and `is!`.
They describe a lower-level representation an object may be pushed down to, such
as a natural number treated as an underlying set.

Unmarked `relation:` groups are still valid. They record user-defined
relationships for readers and for future semantic extensions without affecting
builds.

## Operators as Application

A named operator and a symbolic operator both desugar to application when they
name something callable:

- **`x |op| y` means `op(x, y)`**, `f| x` and `x |f` mean `f(x)`. The `|op|`
  content may be a dotted **member path** such as `|M.*|` or `|x.y.z|`, which
  tracks down through a value's fields: `x |M.*| y` is `M.*(x, y)`, reaching the
  `*` component of a destructured `M ::= (X, *)`.
- **A symbolic operator `x * y` desugars to `*(x, y)` when `*` names a bound
  value** in scope (for example the operation component of a destructured
  magma). Otherwise `*`, `+`, … keep their built-in arithmetic resolution.

A binary operator resolves in order: (1) the application desugar above, when the
symbol is bound; (2) a provided-symbol capability whose operator is
colon-qualified so a single operand type owns it (`:op`, `op:`, `:op:`), matched
against the owning operand's type; (3) a `Disambiguates` entry; and (4) a
provided-symbol capability owned by the operands' *common* type, where a value
known only through a spec (`y "in" M`) is first reduced to its `is`-facts to make
the match. So if `y "in" M` makes `y` a `\magma.element` and `\magma.element`
`Enables:` `capability: x_ * y_ :=> ...`, then `y * y` resolves through that
capability (step 4). If none apply, the operator is reported unresolved. Prefix
and postfix operators, and member access, follow the analogous provided-symbol
and disambiguation paths.

A capability may also declare a **bracketed placeholder operator**
`x_ [*] y_`, where `[*]` names a symbol drawn from the definition's
inputs/`Defines:` (here the `*` component of `M ::= (X, *)`) rather than a
fixed character. The provided operator's name is then the operand's concrete
operation symbol.

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

### Placeholder parentheses

A placeholder may carry a `+` or `-` before its `?` to control the parentheses
around the value substituted into it. The modifier is not part of the name, so
`A?`, `A+?`, and `A-?` all substitute the same value `A`.

- `A?` — substitutes the value exactly as rendered.
- `A+?` — wraps the value in exactly one pair of parentheses, unless it is a
  single atom.
- `A-?` — removes every pair of parentheses wrapping the value.

With `A` bound to each of the following, the three forms render as (shown
`A?` / `A+?` / `A-?`):

- `A = 1+2` → `1 + 2` / `(1 + 2)` / `1 + 2`
- `A = (1+2)` → `(1 + 2)` / `(1 + 2)` / `1 + 2`
- `A = (((1+2)))` → `(((1 + 2)))` / `(1 + 2)` / `1 + 2`
- `A = a` → `a` / `a` / `a`

`A+?` never doubles parentheses: the value is first reduced to its bare form and
then wrapped once. Only parentheses that enclose the *whole* value are removed, so
`(1+2)+(3+4)` is left intact by `A-?` — its leading `(` closes before the end and
therefore does not wrap the expression.

A value counts as a single atom, and so is left unwrapped by `A+?`, when it
contains no space or comma outside of a bracket. That covers names (`a`, `x_1`),
single commands (`\emptyset`, `\mathsf{Field}_{V}`), and function calls (`f(x)`),
but not compound expressions (`1 + 2`) or comma-separated tuples (`X, Y`).

A modifier only applies when the `+` or `-` sits between the name and the `?`, so
`A?-B?` is still a placeholder, a literal `-`, and another placeholder. Where a
template is rendered without values — a card title, for instance — a modifier has
nothing to act on and shows the same bare name that `A?` does.

### Variadic placeholders

A variadic header parameter may be used in `called:` and `written:` just like an
ordinary placeholder. With `x...` in the header, `x?`, `x+?`, and `x-?` insert
the whole variadic value and apply the usual parenthesis rule. At an invocation,
the concrete arguments are used; in a definition title without concrete values,
the symbolic rendering is used.

Three suffix forms control how the individual values are combined:

- `x?{...A}` appends `A` to every value.
- `x?{B...}` prepends `B` to every value.
- `x?{...C...}` inserts `C` between adjacent values.

The `A`, `B`, or `C` body may be arbitrary template text, including LaTeX. For
example, `x?{...\text{ and }...}` renders three arguments as
`x_1 \text{ and } x_2 \text{ and } x_3`.

A 2D variadic placeholder nests the element notation inside a row notation:

```text
x?{R...{...C}}
x?{{...C}...R}
x?{L...{...C}...R}
```

The inner braces use the same prefix (`{C...}`), postfix (`{...C}`), or infix
(`{...C...}`) rules across the elements of each row. Text before the inner
braces is applied at the start of every row and text after it at the end of
every row. For example,
`\left[ x?{{...,...}\\} \right]` joins columns with commas and appends a
LaTeX row break to every row, rendering the supplied argument as a matrix in
square brackets.

The same expansion is used for a definition card's symbolic title. A header
bounded by `(m,n)` therefore renders symbolic first, indexed, and last rows and
columns (with ellipsis rows between them) through the nested template instead
of displaying the raw `{...}` notation.

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
The semantic checker requires `Defines`, `Declares`, and `States` to include at
least one `called:` or `written:` item. A `Refines:` group instead requires an
`adjective:` item and rejects `called:`.

### Card titles

When a `Documented:` section supplies both a `called:` and a `written:` form, the
item's card is titled with both: the human name, a wide space, then the written
notation, which the viewer renders muted so it reads as "and here is its symbol"
rather than as a second, competing title.

```text
[\empty.set]
Defines: X
Documented:
. called: "empty set"
. written: "\emptyset"
```

That card is titled "Empty set  $\emptyset$" (with `$\emptyset$` muted). An item
documented with only one of the two forms is titled with that form alone.

The title always shows the name first and the notation second, whichever order
the two are listed in. Listing order still decides which single form names the
item *inline* — `called:` first makes `X is \empty.set` read "X is empty set",
`written:` first makes it read "X is $\emptyset$" — but it does not reorder the
card title.

A `written:` nested inside a `called:` group documents that called form, so it
pairs with it in the title just as a top-level `written:` would.

`Refines:` items are named by an `adjective:` rather than a `called:`, so their
titles are unaffected.

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

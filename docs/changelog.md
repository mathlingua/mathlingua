# Implementation Changelog

This document records the recent MathLingua language, checker, renderer, view,
and CLI behavior implemented in this repository. It is intentionally rule-focused:
each section captures not only the feature, but also the conditions under which
the feature is valid.

## Subtype Views In `specifies:`

A `Declares:` group may now use several `is ... via ...` items in `specifies:`
to map different slices of its tuple onto different supertypes. Refined types
are accepted before `via`. Every view must consist only of component symbols
from the `Declares:` tuple, written verbatim, and its left subject must be the
thing the group declares; computed forms, foreign symbols, and repeated symbols
are rejected. Each supertype supplies the mapped
components' types positionally. Views may overlap only when they assign the
overlapping symbol exactly the same type; otherwise the checker reports a
conflicting subtype specification.

## Directed Relations

Top-level `Relation:` items now have two distinct endpoint shapes. The existing
`between:`/`and:` form represents an undirected relationship, while the new
`from:`/`to:` form represents a directed relationship whose source and
destination remain explicit in the structural AST. Both forms support the same
`using:`, `when:`, `specifies:`, documentation, justification, alias, writing,
reference, and metadata sections. Endpoint spellings from the two forms cannot
be mixed.

## Checked MathLingua In Prose

MathLingua fragments embedded in any quoted prose value are now semantically
checked as well as rendered. `<<name>>`/`<</name>>` scope markers must be
balanced and properly nested; declarations using `is`, quoted specification
operators, or `:=` introduce symbols until the current prose scope closes.
Prose inherits the enclosing item's symbols, so theorem proof text can use
heading and `given:`/`where:` names but not bindings local to its conclusion.
Directory prefaces receive the same checks as navigable source files.

Nested text arguments in cards now pass through the Markdown-and-KaTeX renderer,
so ordinary `$...$`/`$$...$$` and generated `{. ... .}`/`{{. ... .}}` math render
instead of appearing as literal dollar-delimited source.

## Proofs Follow Theorem Cards

A theorem's optional `Proof:` is rendered as document prose immediately after
the theorem card instead of as a labeled section inside it. The proof has its
own heading, retains MathLingua-aware Markdown and LaTeX rendering, and ends
with a proof box. A source toggle shows the `Proof:` section alone; commands in
that source are interactive definition references. `Proof:` now belongs after
`Documented:` and before `Aliases:`/`Writing:`/`References:`, whose aliases,
rendering overrides, and citations remain available to the proof.

## Nested `given:` Clause Removal

The redundant nested `given:` / `where?:` / `then:` clause group has been
removed. Use `let:` / `where?:` / `then:` for local bindings and assumptions.
The corresponding inline builtin is now `\\let{...}:where{...}:then{...}`;
`\\given` is no longer supported. The `given:` section on theorem-like top-level
groups remains supported.

## Duplicate Symbol Introductions Are Rejected

The semantic checker now distinguishes known names from symbols that have been
introduced as definitions. A scope cannot introduce the same defined symbol
twice: this catches duplicate command-header parameters, repeated `:=`
bindings, nested quantifiers or `let:` groups that shadow an enclosing binding,
`using:` declarations that redeclare header or target symbols, and colliding
`view: as:` bindings. These cases report that the symbol has already been
defined.

Function-form placeholders remain local to their forms and may repeat in nested
quantifiers. The callable name introduced by the form may not be rebound.

## Direct Components In Specifications And Rendering

Direct component expressions such as `\naturals..N` are now accepted as
specification targets, including facts such as `n "in" \naturals..N`. The
source continues to use `..` to distinguish direct access on a concrete
`Defines:`/`Realizes:` command from ordinary member access on a bound value.
Rendered mathematics uses a single dot for both forms once that distinction has
been resolved.

## Assignment-Backed `specifies:` Facts

The `specifies:` sections of both `Declares:` and `Defines:` accept top-level
`:=` assignments. Type and capability inference follows those assignments: if a
structured definition's capability is stated in terms of a local component,
assigning that component in a realization lets the capability apply through the
assigned command value.

## Repeated `elseIf:` Branches In `piecewise:`

A structural `piecewise:` clause now accepts zero or more `elseIf:`/`then:`
pairs between its initial `if:`/`then:` pair and optional `else:`. Each
condition is assumed while its corresponding result branch is checked.

## Placeholder Normalization In Rendering Templates

`called:` and `written:` templates now match a function-form parameter whether
the template includes its source underscore or uses the normalized name. For a
heading parameter `n_`, `n_?` and `n?` therefore refer to the same substitution,
and `S(n_?)` renders without exposing the source placeholder suffix.

## Refined Types In Clause Bindings

Clause formulations, including `have:` and theorem-like binding positions, now
use the refined declaration parser. Bindings such as
`A is \(inductive)::set` are accepted consistently in quantifiers,
justifications, and other clause positions.

## Optional `asserting:` in `have:` Groups

The `asserting:` section of a `have:`/`asserting:`/`because?:`/`by?:` group is
now optional. When it is omitted, the checker validates the `have:` clauses
against the facts already in scope, while still reference-validating any
`because:` clauses and `by:` references. A `have:` group with `iff:` continues
to select the separate iff shorthand.

## Refined Capabilities on Defined Values

The target of a `:->` or `:<->:` spec capability may now assert a refined type,
for example `x_ "in" omega :-> x_ is \(inductive)::set`. A capability owned by
a `Defines:` or `Realizes:` entry applies to that defined command value (with
its parameters instantiated), while a capability owned by `Declares:` continues
to apply to every value of the declared type. Facts derived at a refined type
also establish that refinement's base type.

Function-form definitions now receive their declared function input facts while
their bodies are checked. Thus `S(n_) is \function:on{\omega}:to{\omega}` puts
`n "in" \omega` in scope; a capability can refine that fact and supply a base
requirement such as `n is \set`.

## `\\anything` Builtin Type

The universal unstructured-value builtin is now named `\\anything` instead of
`\\opaque`. Its semantics are unchanged: every declared value satisfies it,
and an `\\anything` fact establishes no more specific type information.

## Anonymous Spec-Literal Placeholder

Spec literals now use `_` exclusively as their anonymous subject. The former
`? is Type` and `? "op" Target` spellings are rejected; write `_ is Type` and
`_ "op" Target` instead. Tuple, set, and function type literals accept one or
more `_` specifications as appropriate, and the viewer renders each anonymous
placeholder as `\cdot`. Other uses of `?`, including inferred arguments and
predicate operators, are unchanged.

## LCD Viewer Themes

The viewer theme menu now includes LCD and LCD Dark. The pair uses a grayscale
monochrome palette, monospaced display type, square one-pixel geometry, hard
shadows, and a subtle screen-cell matrix that recalls the dark squares of early
liquid-crystal displays. Both variants participate in the existing theme
persistence and pre-render color-scheme setup.

## Directed And Equivalent Spec Capabilities

Spec capabilities now distinguish implication from equivalence. `a :-> b`
adds only the directed inference from `a` to `b`; `a :<->: b` adds both
directions. Either form accepts multiple conclusions separated by `;`, and the
reverse direction of `:<->:` requires all of them.

Set-builder membership now follows the same distinction. For
`{x_ : specs}`, membership and the element specifications are equivalent. For
`{x_ : specs | condition}`, membership implies the specifications and the
condition, while reverse inference succeeds only after both are established.
When that reverse check fails, requirement diagnostics report the missing set
condition.

## `Enables: view:` Replaces Cast Relations

The nested `Enables: relation:/to:/when:/represents:` group has been removed.
So have the `\\coercion` and `\\encoding` markers, hard builds (`@!`), and
hard type introductions (`is!`). Ordinary `@` builds and `is` declarations
remain.

An `Enables:` `view:` group now states a type through which an object may be
used. It has a zero-argument `view:` marker, a required `as:` declaration of
the exact form `<symbol> := <expression> is <type>`, and an optional
`signifies:` statement checked in that declaration's scope:

```text
[\naturals]
Defines: Nb ::= (N, 0, S(n_))
abstractly:
specifies:
. N is \set
. 0 "in" N
. S is \function:on{N}:to{N}
Enables:
. view:
  as: X := N is \set
```

Here `Nb` remains the structured value; the view makes `\naturals`
viewable as a `\set` through `N`. Consequently a requirement such as
`A is \set` accepts `\naturals`, including the domain and codomain requirements
of `\function:on{\naturals}:to{\naturals}`. The view does not add
`Nb is \set` to the definition's output facts.

If the `:=` right-hand side is a name or tuple of names, the checker follows
facts through the corresponding qualified component(s), such as
`\naturals..N`. Other expressions advertise only the introduced symbol's type.

## Viewer Type Annotations

The web viewer now includes an interactive type inspection toggle in the viewer
chrome. When "Types" is enabled, cards render per-line inferred type annotations
for sub-expressions directly alongside the source expressions.

The type information is computed during the semantic check pass, mapped to line
numbers and byte spans (`TypeAnnotation`), and serialized into the view model by
the view builder pass. Both `mlg view` and `mlg export` package this metadata so
readers can inspect resolved expression types in the browser without running the
compiler locally.

## Direct Component Access With `..`

A concrete object command can expose a component from its destructuring target
without first being bound and destructured. If `\reals` defines
`Rb ::= (R, 0, 1, +, *, <=)`, then `\reals..0` denotes its `0` component.
Callable components use the same operator, for example
`\naturals..succ(n)`. Component types and command parameters are instantiated
from the owning definition. The owner must be a `Defines:` or `Realizes:`
command; a bound/destructured value continues to use ordinary `Rb.0` access.

## Refines Cards Use The Primary Adjective And Resolve `[[subject]]`

When a `Refines:` `adjective:` item lists several texts, the first is now the
primary display adjective used in the card title and in rendered refined types.
Later texts remain documented alternatives rather than being concatenated into
the displayed name.

A dynamic refined tail in that group's `specifies:` section now renders through
the base type encoded by the enclosing heading. For example, under
`[\(bijective)::function:?on{A}:?to{B}]`,
`\(injective, surjective)::[[f]]` renders as an injective and surjective function
on `A` to `B`, rather than as an injective and surjective `f`.

## `Refines:` Targets May Repeat The Full Base Target

A `Refines:` target may now repeat the complete target of the base `Declares:`
item. For a base declared as `f(x__) ::= y_`, all of the following refinement
targets are valid:

```text
Refines: f(x__) ::= y_
Refines: f(x__)
Refines: f
```

The refinement may omit trailing structure it does not need, while every shape
it includes must agree with the base target. Tuple destructuring keeps its
existing positional component-shape checks.

## The `Refines:` `extends:` Section Is Now `specifies:`

The inheritance section inside a `Refines:` group is now spelled `specifies:`.
Its position, contents, and semantics are unchanged; only its label and the
corresponding structural AST names change. `RefinesExtendsSection` is now
`RefinesSpecifiesSection`, and `RefinesGroup::extends` is now
`RefinesGroup::specifies`. The `extends:` section of a `Declares:` group is
unaffected. Existing `Refines:` source must use the new spelling.

## The `means:` Section Is Now `specifies:`

Every section previously spelled `means:` is now spelled `specifies:`. This is a
terminology change only; no section moves, changes contents, or changes
semantics. It affects all four sections that carried the old name:

- the `specifies:` of a `Declares:` group, which types the described form's
  parts
- the `specifies:` of a `Defines:`/`Realizes:` group, which supplies a value
  (or, under `abstractly:`, a specification) for each part
- the `specifies:` of a top-level `Relation:`, either a statement or prose
- the `specifies:` of an `Enables:` `relation:` group and of a `Topic:`
  `Related:` entry

```text
[\monoid]
Declares: M ::= (X, *, e) is \magma via (X, *)
specifies: e "in" X
```

The structural AST renames the corresponding types and fields —
`DeclaresMeansSection`, `DefinesMeansSection`, `RelationMeansSection`,
`RelationshipMeansSection`, and `TopicRelatedMeansSection` become
`DeclaresSpecifiesSection`, `DefinesSpecifiesSection`,
`RelationSpecifiesSection`, `RelationshipSpecifiesSection`, and
`TopicRelatedSpecifiesSection`, the `RelationMeans` enum becomes
`RelationSpecifies`, and each group's `means` field becomes `specifies`.
Diagnostics, completions, examples, and the current language documentation use
the new label. Existing source collections must rename the section when
migrating.

## `Defines:` And `Declares:` Swap Names

The two top-level definition labels are swapped so their names better match
their roles. This is a terminology change only; their semantics and section
layouts do not move independently of them:

- `Declares:` now names the type/interface-style construct formerly spelled
  `Defines:`. It supports `extends:`, `satisfies:`, and type-specifying `means:`
  items.
- `Defines:` now names the value/declaration-style construct formerly spelled
  `Declares:`. It supports `abstractly:`, value-supplying `means:`, and
  `expresses:`; `Realizes:` continues to realize an abstract instance of this
  construct.

The structural AST, semantic definition kinds, diagnostics, completion labels,
rendered item kinds, examples, and current language documentation use the new
names. Existing source collections must swap these two labels when migrating.

## `goldens/examples/`: A Collection That Exercises Everything

`docs/examples.txt` is replaced by `goldens/examples/`, a complete MathLingua
collection that illustrates every language feature **and checks cleanly**. The
old file was only guaranteed to parse; its examples referenced commands that were
never defined, so nothing verified that they meant anything. The collection has
no such escape: `mlg check` accepts it with no diagnostics, so every example is
a working one.

It is organized as ten content files, read in order:

```text
01_outline     Title/SectionTitle/SubsectionTitle/Text, Writing:, Specify:
02_core        the handful of definitions everything else builds on
03_numbers     number types, Defines values, Disambiguates
04_algebra     what a Declares extends: is/via, means:, the extends: section
05_defines     Defines values and parts, abstractly:, Realizes
06_refines     Refines, implicitly:/explicitly:, refined spec-infix
07_headings    every command heading shape
08_statements  States, Axiom/Theorem/Conjecture, every clause group
09_relating    Equivalent, Relation, Topic, Person, Resource, text-only items
10_support     each support section in detail
```

A test copies the collection to a temporary directory (a check rewrites its
input, and a test should not modify the source tree), runs the real checker over
it, asserts there are no diagnostics, and asserts every top-level item kind
appears — so neither an example nor a newly added item kind can drift from the
implementation.

Two gaps in `Id:` auto-insertion surfaced while building it and are fixed: a
top-level `Writing:` group and the four `Text*` items were excluded from
`is_top_level_item_label`, so `mlg check` demanded an `Id:` for them but never
wrote one. `Text*` items had to carry a hand-written `Id:` because their grammar
requires the section; now, like every other top-level item, they get one.

## `States:` And `Equivalent:` Are Marker Sections

Neither head section takes an argument any more. Prose describing the item goes
in `Documented:`, where the rest of an item's human-facing text already lives:

```text
[\associative]
States:
that: \true
Documented:
. called: "associative"
. overview: "Associativity"
. description: "Grouping does not matter."
```

```text
[\continuous:at{p}]
Equivalent:
to:
. \epsilon.delta.continuous:at{p}
. \neighborhood.continuous:at{p}
Documented:
. called: "continuous at $p?$"
. description: "The epsilon-delta and neighborhood definitions agree."
```

Content in either section is now reported as `Section `{label}` does not accept
content`, the same diagnostic the other marker sections use. `StatesSection` and
`EquivalentSection` are gone from the structural AST, along with the `states` and
`equivalent` fields of their groups.

## Abstract Declarations And `Realizes`

`Declares` gains a `means:` section, immediately above `expresses:`, so a
declaration whose target destructures a tuple can say what each part is:

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

Every item must supply a **value** — directly with `:=`, or indirectly by
stating a type in `means:` and defining it in `expresses:` (a `piecewise:` that
says what `C` is). An item that states a specification and never defines it is
an error unless the declaration is marked abstract.

`abstractly:` is a new zero-argument marker placed immediately after `Declares:`.
It lets a declaration describe a concrete thing with abstract parts: its
`is`/`"op"` `means:` items are left open, for a realization to supply.

```text
[\naturals]
Declares: Nb ::= (N, 0, succ(n_))
abstractly:
means:
. N is \set
. 0 "in" N
. succ is \function:on{N}:to{N}
```

`Realizes` is a new top-level item that supplies them:

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
`Declares` without it is a concrete one. The `:=` in the `Realizes:` target
follows the general rule that a `Defines` names a type (`X is \foo`) while a
`Declares` names a value (`Y := \bar`).

Rules:

- A `Realizes:` target must name a `Declares:` marked `abstractly:`
  (``Realizes:`` must name a `Declares:` marked `abstractly:`; `\set` is a
  `Defines:``).
- Its `means:` must supply **every** symbol the declaration left abstract;
  omitting one reports `Missing realization for abstract symbol `{symbol}``.
- A realization's `means:` items must themselves be concrete, on the same rule
  as a non-abstract `Declares:`.
- A realization's components keep the types its declaration gave them, so
  `Nb ::= (N, 0, succ) := \von.neumann.naturals` types `N`, `0` and `succ`
  exactly as destructuring `\naturals` does. Destructuring now follows a `:=`
  value's command as well as an `is` type, which is how a declaration's value is
  written.
- A destructuring `Declares:` subject counts as defined once `means:` supplies
  each of its components, and an abstract declaration is exempt from the rule
  that a `Declares:` value must state its type — `means:` states it instead.

`Realizes` participates everywhere `Declares` does: it registers a command
signature, requires an `Id:` (auto-inserted like any other item), requires a
`called:` or `written:` item in `Documented:`, renders as a card, is a definition
kind for `mlg release`, and is offered by LSP completion.

Separately, the LSP completion table had drifted from the section orders it
mirrors: `Writing:` was missing from every definition and theorem-like group, and
`implicitly:`/`explicitly:` from `Refines`. Both are now offered.

## A Spec-Infix Heading's Left Operand Is Not A `when:` Parameter

A spec-infix heading such as `[A \:subset:/ B]` is sugar for a command whose left
operand is the symbol being defined. That symbol is stated by the `Defines:`
target, so it may no longer appear in `when:` — previously it was merely not
*required* there:

```text
[A \:subset:/ B]
Defines: A is \set
when: A, B is \set                 <- `when:` requirement for `A` is not allowed
```

The correction is `when: B is \set`. `when:` says only what a *use* of the
command requires of its remaining operands. The message names the cause:
```
`when:` requirement for `A` is not allowed because `A` is what this definition
describes; state its type on the definition's target instead
```

The same holds for a refined spec-infix `Refines:` whose target is the heading's
left operand.

## A Symbol Is Specified Exactly Once

A `Defines` target symbol must now be given its type exactly once, not merely at
least once. The subtype clauses already type the components their `via` reaches,
so repeating one in `means:` is an error:

```text
[\group]
Defines: G ::= (X, *, e) is \set via X
means:
. X is \set                        <- Duplicate specification for `X`
. * is \function:on{X}:to{X}
. e "in" G
```

`via X` states `X is \set`, so `means:` carries only the components the clauses
do not reach — `*` and `e`, which still *must* be specified there. The exception
is an additive refinement of the already specified base type: an inherited
`* is \binary.operation:on{X}` may be strengthened with
`* is \(associative)::binary.operation:on{X}`. A different base type remains a
duplicate. The message is `Duplicate specification for target symbol
`{symbol}`; it is already specified by ...`, naming the earlier source (`the
`Defines:` target`, `an `extends:` clause`, or ``means:``).

All of a group's subtype clauses count as a **single** source. `extends:` exists
precisely so one definition can extend several types, so two clauses may name the
same subject, or reach the same component through different views, without that
counting as two specifications:

```text
[\triple]
Defines: X ::= (A, B, C)
extends:
. X is \left.pair via (A, B)
. X is \right.pair via (B, C)
```

`when:` and `using:` are not specification sources for this rule: `when:` states
what a *use* of the command requires and `using:` introduces auxiliary symbols,
neither of which is the definition stating the type of its own target.

## The `Defines` `declares:` Section Is Now `means:`

The section of a `Defines` group that types the described form's parts is now
spelled `means:`:

```text
[\group]
Defines: G ::= (X, *, e) is \set via X
means:
. X is \set
. * is \function:on{X}:to{X}
. e "in" G
```

Only the label changes; the section's contents, position (after `extends:`,
before `satisfies:`), and semantics are the same. The structural AST exposes it
as `DefinesMeansSection` through `DefinesGroup::means`, and completions suggest
the new label.

`means:` is not ambiguous with the three other sections of that name — the
`means:` of an `Enables:` `relation:` group, of a top-level `Relation:`, and of a
`Documented:`/`Topic:` `related:` entry — because sections are identified against
their own group's pattern, and each of those groups is dispatched by its own
first label (`relation`, `to`), never by `means`.

Note that `means:` was, several releases ago, the spelling of what is now
`extends:`. A `Defines` `means:` section now types the target's parts and does
**not** state what the definition extends; `Refines` still rejects `means:`
outright.

## A `Defines` Target May State The Type It Extends

The type a `Defines` group extends can now be written on the `Defines:` target
itself, with the same `is`/specification statement — including its `via` view —
that an `extends:` section takes:

```text
[\group]
Defines: G ::= (X, *, e) is \monoid via (X, *)
```

```text
[A \:subset:/ B]
Defines: A is \set
when: B is \set
```

`Defines: X is \foo` and `Defines: X` with `extends: X is \foo` are equivalent,
and a group may use one spelling or the other but **not both** (`A \`Defines:\`
target that names the type it extends cannot also have an \`extends:\` section`).
The target spelling is the short one and fits the common single-clause case; the
`extends:` section remains for what a target cannot express — extending several
types at once, each through a different view of the same tuple:

```text
[\foo]
Defines: X ::= (A, B, C)
extends:
. X is \bar via (A, B)
. X is \baz via (B, C)
```

Consequences:

- `extends:` now takes **one or more** clauses rather than exactly one, and each
  is an `ExtendsItem` — a `DeclarationStatement` with an optional `via` — rather
  than an `IsOrViaItem`. `DefinesSection` carries the target's own
  `via: Option<FormOrDeclaration>` alongside its argument, and
  `extends_clauses` in `structural::ast` normalizes both spellings into a single
  borrowed clause list that the checker, the walk, and the registry all work
  from.
- A `via` with no `is` clause to accompany it is a parse error.
- The `Defines:` target accepts refined commands in its `is` relation, as
  `extends:` does (`Defines: S ::= (X, *) is \(finite)::magma via (X, *)`).
- The target and the `extends:` clauses are checked after `using:`/`when:` and
  the `via` facts are assumed, since the type a definition extends may name
  symbols only those bring into scope — as in `Defines: x "in" X` under
  `when: M is \magma`.
- When the subject is an operator form, its facts are recorded about the
  operator: `Defines: x_ * y_ is \function:on{X \.cross./ X}:to{X}` states its
  fact about `*`, matching how the operator is named in `means:`, in a
  `Refines:` of the same form, and in member access. This is what lets an
  `extends:` clause whose subject was a component of the target — `extends: * is
  \function:…` — move onto the target's own form.

`Refines` is unaffected: its target already carries an `is` relation naming the
base type, so its refinement target cannot merge into the same statement and its
`extends:` section stays a single clause.

## Per-Line Type Information Over The Language Server

`mlg lsp` now answers `textDocument/hover` with the type of every expression,
sub-expression, and statement on the hovered line.

The information is a by-product of the ordinary type check rather than a second
analysis: `check_documents_collecting_type_info` installs a recorder on the
signature registry for one named file, and the walk files each line's entire
expression tree against the type context in force at that line. A value reports
the facts its result carries, a name that carries none of its own reports what
the enclosing scope declared about it, and a statement reports the fact it
asserts.

Because a formulation AST carries no source positions, lines are identified by
parsing each row's formulation on its own and matching it against the walk's
expressions by structural equality, scoped to the enclosing top-level item.

The server resolves types only when a document is opened or saved — never on
edit, since a check walks the whole collection — and serves the last resolved
answer until the next save. A hover on a document edited since that check says
so.

## Configurable Numeric Literals And Variadic Indices

Top-level `Specify:` now has four typed categories: `decimal`,
`zeroOrPositiveInt`, `positiveInt`, and `int`, each with an `is:` type
expression. The checker uses these as global fallbacks for decimal, zero,
positive-integer, and negative-integer spellings only after explicit lexical
scope facts fail, so a locally defined numeric spelling takes precedence.

Variadic headers use the same configuration for their index domains. One-based
and plain `x...` parameters use `positiveInt`; zero-based parameters use
`zeroOrPositiveInt`; 2D parameters apply the configured type to both axes.
Indexed elements now accept computed expressions such as `x[i * j]` and
`x[\next{i}]`. The checker infers the expression's result type and rejects the
index when it cannot establish the category required by the header.

## Two-Dimensional Variadic Curly Arguments

Curly command arguments now support rectangular 2D variadic parameters such as
`x[(i_, j_) := (1,1)...(m,n)]` and the zero-based or unbounded variants.
Invocations separate columns with commas and rows with semicolons. The checker
binds row and column counts independently, substitutes `x[i,j]` per element,
and rejects ragged matrices, empty dimensions, flat arguments, and 2D syntax in
non-curly argument groups.

Two-axis selections support indexed rows or columns, bounded ranges with
optional index binders, and `...` for either whole axis. They broadcast under
the same relations as 1D slices. Written templates can nest the ordinary
variadic element notation inside row prefix/postfix notation; this supports
matrix renderings such as `\left[ x?{{...,...}\\} \right]`.

In `when:`, a whole-axis requirement such as `x[..., ...] is \natural` is
instantiated once per supplied cell instead of being applied to the flattened
argument list. Symbolic definition-card titles also retain their 2D row shape,
including one-cell ellipsis rows, so nested templates such as
`\begin{bmatrix} x?{{...&...}...\\} \end{bmatrix}` render through the same
matrix path as concrete invocations.

## Structural Literal Command Arguments

Tuple, set, and mapping literals can now satisfy a command's nominal type
requirements from their structure. A mapping literal is checked by its input
arity and input/output specs; a tuple literal is checked by its component count
and positional component specs; and a set literal is checked against the
described element pattern and element specs. The checker therefore accepts a
typed mapping literal where a command requires `f is \function` without first
requiring an artificial nominal fact about the literal. A mismatched arity,
shape, input type, element type, or output type still produces the normal
`Could not establish requirement` diagnostic.

Mapping input and output specs may be collection membership requirements. For
example, if `\naturals.set` is a literal-backed collection whose elements are
natural numbers, `x is \natural` satisfies `x "in" \naturals.set` while checking
a mapping literal. Operator expressions in the mapping body are resolved before
the output spec is checked. If an operator is declared through a callable type
such as `\binary.operation:on{X}`, the checker follows that type's `extends:`
chain to its function codomain and uses the codomain as the application's output
spec.

## Variadic Mapping Parameters And Parameter Selection

Function forms now accept ranged variadic inputs such as
`f(x_[i_:=1...n])` and tuple-valued `f(x__[i_:=1...n])`. Ordinary command
headers can select exact mapping inputs (`f.x_`), arbitrary inputs under fresh
names (`f.u?_`), or a variadic subset (`f.x_[i_[j_:=1...m]]`). The checker
enforces the single-selector-group and associated-explicit-mapping rules and
verifies that use-site selector names are parameters bound by the mapping
literal. Because a selector is bound by that associated mapping, it is allowed
but not required as a separate `when:` subject.

These headers register specialized signatures such as `_(2)`, `#1`, `#?`, and
`#*` beneath a shared general signature. Invocation resolution prefers fixed
arity over variadic arity, exact ordered positions over arbitrary positions,
and a variadic subset last; equally specific matches are rejected as ambiguous.
The renderer uses the same ranking, and mapping-literal parameter names remain
in scope for sibling selector groups. Written templates receive the mapping
body for the owner placeholder and the concrete selected parameter for each
selector placeholder, so authoring markers such as the trailing `_` do not leak
into the rendered notation.

Named mapping calls now also accept `=` assignments, catch-all `... = value`,
and ranged selector text, while preserving the existing `:=` spelling.

## Mapping Writing Templates

A mapping-shaped `Defines:` item may now declare documented `writing:`/`as:`
templates for both its ordinary invocation form and the mapping form itself.
The `writing:` target must exactly match the defined mapping with either all
placeholders retained or all placeholders replaced by their ordinary names.
Invocation templates substitute call arguments, while mapping templates render
references to the mapping as a value, such as `x(i_) is \real.sequence`.
For `X ::= x(i_)` and `X ::= x(i_) ::= y_`, `X` is an additional name for the
mapping, while documented writing forms continue to use `x(i_)` or `x(i)`.

## Structured Conjecture Groups

`Conjecture:` is now a supported top-level group with the same assertion shape
as `Theorem:`: optional command heading, `given:`, `where:`, `iff:`, and support
sections around a required `then:`. It represents a statement that is not
claimed to have a proof. An optional `Justification:` records rationale or
evidence without changing that status.

## Corollary Group Removal

`Corollary:` is no longer a supported top-level group. Use `Theorem:` for a
structured consequence, and describe its relationship to an earlier result in
the theorem's documentation. The structural AST, parser, semantic checker,
completions, release metadata, renderer, and current language documentation no
longer include a distinct corollary kind.

## Local `let:` Clauses

Clause-valued sections now accept a `let:` group, an optional `where:` guard,
and a required `then:` section. Each `let:` argument uses the same
refined-capable binding syntax as quantifier bindings and introduces its symbols
in order. The bindings are available to `where:`, and its assumptions are
available while checking `then:`; both remain local to the clause group.

## Self-Contained Viewer And Export

`mlg view` now serves the production web viewer and collection JSON directly
from the Rust process. `mlg export` copies the same embedded assets into the
static site. Neither command installs packages, invokes npm, or requires Node.js
on the user's machine; the generated Vite assets are packaged into the `mlg`
executable.

## Atomic Viewer Themes

The viewer theme menu now includes Atomic and Atomic Dark alongside the
existing Classic, Mono, Flat Gray, Sepia, and Retro light/dark pairs. Atomic's
cards, navigation, typography, source view, supporting-section controls, and
syntax colors share a coordinated high-contrast treatment. Theme selection is
persisted in browser local storage like the other viewer themes.

## Structural Spec-Literal Types

Type expressions now support structural tuple, set, and function types whose
leaves are complete specification literals:

```text
(_ is \natural, _ "in" \reals)
{_ is \natural : ...}
{(_ is \natural, _ "in" \reals) : ...}
(_ is \natural) -> (_ "in" \naturals)
(_ is \natural, _ "in" \reals) -> (_ is \real)
```

The checker matches those shapes to tuple, set, and mapping terms and
instantiates every `_` with the corresponding term component. Quoted
specification operators are retained as spec facts; they are not reduced to
ordinary `is` facts. A function definition may therefore declare its argument
and result components individually or declare the function itself with one of
the new function types. Raw nominal shapes such as `(\natural, \real)` remain
invalid. The `->` notation accepts one or more inputs and preserves the
declared arity, allowing unary, binary, and n-ary function types without a
curried spelling. Anonymous function literals use `=>`, so bare type and value
arrows mirror the existing `:->` spec-operator and `:=>` expression-alias
forms. The former `|->` spelling is no longer accepted.

## Definition And Declaration Names

The former `Defines:` group is now named `Declares:`, and the former
`Describes:` group is now named `Defines:`. Within the new `Defines:` group,
the former `specifies:` section is now named `declares:`. The parser, structural
AST, semantic diagnostics, completions, release metadata, examples, and golden
fixtures all use the new meanings. `Describes:` and `specifies:` are no longer
accepted; `Defines:` now has the former `Describes:` semantics.

## Callable Command Heading Parameters

Trailing parenthesized command-heading parameters and infix command operands now
accept standalone placeholder spelling. For example, `[\natural.succ(n_)]`
defines a callable command with parameter `n`, while
`[n_ \.natural.+./ m_]` defines an infix command with parameters `n` and `m`.
Each placeholder is normalized to the existing named-parameter shape used for
signature checking, substitution, and rendering. Existing bare-name and
structured-form heading parameters remain unchanged.

## Soft Build Rendering

The `@` in a soft build such as `\set@{...}` remains part of the source syntax,
but the mathematical view now renders it as an explicit thin space instead of
showing the `@` glyph. Hard builds written with `@!` keep their visible marker.
Multiple set-literal specifications retain their semicolon separation in the
mathematical view rather than being displayed with commas, with a thin space
after each semicolon for readability.

## Implied Refines Type Rendering

The mathematical view now makes a `Refines:` entry's heading-implied base type
explicit. For example, `[\(finite)::group]` with
`Refines: G ::= (X, *, e)` renders the section value as
`G ::= (X, *, e) is \group`. This is display-only: source syntax and semantic
checking are unchanged. Parameterized base types such as
`\binary.operation:on{X}` are included for operator targets as well. An
explicitly written relation is not duplicated, and refined infix-spec headings
keep their existing relation rendering.

The definition-inheritance section is now spelled `extends:` in both `Defines`
and `Refines` groups. The structural AST exposes this as `ExtendsSection` /
`RefinesExtendsSection` through each group's `extends` field, completions suggest
the new label, and the former `means:` spelling is rejected.

Member capabilities now render through their `written:` templates, so a rule
such as `x.inv :=> ...` with `written: "x+?^{-1}"` renders `x.inv` as
`x^{-1}`. Subscripted operator symbols also preserve their base notation and
subscript: for example, `*_1` renders as `\ast_1` in tuple declarations and
expressions.

## Refined Spec-Infix Commands

`Refines:` now accepts a refined specification-operator heading such as
`[A \:(nonempty)::subset:/ B]`, in addition to prefix command refinements such
as `[\(bounded)::function:on{A}:to{B}]`. The corresponding
`A \:(nonempty)::subset:/ B` syntax is accepted at declaration and expression
use sites, participates in signature lookup and rendering, and falls back to
the base `A \:subset:/ B` fact inside the refinement body.

An undeclared refined spec-infix signature can also resolve implicitly through
the base operator's `extends:` type. Thus, when `\:subset:/` extends `\set` and
`\(nonempty)::set` is a declared refinement, `\:(nonempty)::subset:/` is valid and
reduces to both the base subset fact and the refined nonempty-set fact. The base
operator's and refined type's `when:` requirements are checked at the use site.

Implicit refined spec-infix relations now render the base relation first and
place their adjectives in a parenthesized English list afterward. For example,
`X \:(a, b, c)::relation:/ Y` renders as `<X relation Y> (<a>, <b>, and <c>)`.
Ordinary refined types retain their adjective-first rendering: `\(a)::type`
renders as `<a> <type>`, and `\(a, b, c)::type` renders as
`(<a>, <b>, and <c>) <type>`. An explicit `written:` template still overrides
the composed rendering.

## Prime Marks In Names And Operators

Names and symbolic operators may now carry **prime marks** — one or more trailing
`'`. Value names accept primes after an alphanumeric, so a name may end in a
prime and a subscript may too: `X'`, `X''`, `e'`, `x'_a'`. Symbolic operators
accept trailing primes alongside the existing subscripts: `*'`, `*''`, `*'_a`. A
bare `*` is unchanged. Prime marks are ordinary name characters (they never start
a name), and stropped operators may be primed (`` `*'` ``).

The identifier atom `[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?` becomes
`[A-Za-z0-9](?:[A-Za-z0-9_']*[A-Za-z0-9'])?` (equivalent for prime-free input),
and operator lexing gains a prime alternative in its continuation. Primes render
as LaTeX primes — `X'` as $X'$, `x'_a'` as $x'_{a'}$, `*'` as a primed operator —
so a primed name reads the same in `mlg view` as it does in source.
Backtick-stropped operators also now render as the operator itself (`` `*` `` as
$*$, `` `*'` `` as $*'$) instead of showing the backticks.

## `Text*` Prose Placeholders

Four new top-level groups — `TextTheorem:`, `TextAxiom:`, `TextConjecture:`, and
`TextDefinition:` — hold a Markdown-with-LaTeX body standing in for a structured
theorem/axiom/conjecture/definition to be written later. Each accepts an optional
`Documented:` (restricted to `called:`, `written:`, `description:`, and the new
`notes:` item), an optional `References:`, and a required `Id:`.

They are **opaque to the type-checker**: the body is never parsed as MathLingua,
so it may mention commands that do not exist yet, and the walker validates no
references inside them. Their purpose is to let an author lay out a collection's
prose, citations, and structure first — then fill in structured forms, which is
easier than going from nothing to structured groups. `notes:` records reminders
for that later conversion.

In the rendered view a placeholder is a card titled by its `called:`/`written:`
(or the kind word — "Theorem", "Definition", … — when untitled), with the body
as rendered Markdown and `References:`/`Documented:`/`Id:` behind the
supporting-sections toggle. The `notes:` documentation item is also accepted in
any `Documented:` section.

## `Justification:` Section And Labeled Specifications

The former `Justified:` section (which held `label:`/`by:` prose notes) is
**replaced** by a `Justification:` section that accepts **only**
`have:`/`asserting?:`/`because?:`/`by?:` groups, each with a required `[label]`
heading. The section now appears **after** `Documented:` (it previously came
before it).

A grouped expression, statement, or specification anywhere else in the same
group may carry a `[:label:]`, including inside nested clauses and expressions.
For example, a `declares:` item may be written `(.x is \foo.)[:1:]`, and a
`satisfies:` clause may be written ``(.*' := `*`.)[:2:]``. When its label matches a
`Justification:` entry's `[label]`:

- the labeled specification is established using that entry's `have:` group and
  any optional `asserting:` items — exactly as an inline `have:` item would be —
  so an assertion the checker takes as true can discharge a requirement the
  labeled specification could not reach on its own;
- the entry's `have:` must **restate the labeled specification exactly** (a
  mismatch is reported); and
- **every** entry must be referenced by some labeled specification (an
  unreferenced entry is reported).

A `[:label:]` with no matching entry is checked inline as an ordinary
specification. The `because:`/`by:` of an entry are reference-validated but never
proven, as with any `have:` group.

## Symbol Introduction, Builds, And Operators

### `as`/`as!` Casts Removed; `\type@value` / `\type@!value` Builds

The `value as \type` and `value as! \type` cast expressions are **removed** (the
`Cast` AST node is gone). A value is now built at a stated type with a command
type followed by `@` or `@!`:

- `\type@value` — a **soft build**: succeeds when the value already has the
  type, extends to it, or the value's type has an `Enables:` `relation:` to it
  marked `represents: \\coercion`.
- `\type@!value` — a **hard build**: also allows relations marked
  `represents: \\encoding`.

A build whose value cannot be viewed at the requested type reports
`Could not build \`{expression}\``. The named counterparts are `x is \type`
(soft) and `x is! \type` (hard), which introduce a symbol with the same view
semantics. Set builders after `@` accept `;`-separated specs, e.g.
`\set@{(a_, b_) : a_ "in" A; b_ "in" B}`.

### `Declares:` Must State Its Type

A `Declares:` target must state its type: either `X := value is <type>` or a
top-level build `X := \<type>@<value>` (which is sugar for `... is <type>`). A
bare `X := {…}` is rejected — `` `Declares:` target `X` must state its type: use
`... is <type>` or a top-level `\...@...` build (e.g. `\set@{...}`) `` — even
when the type is inferable.

### Operators As Application

`x |op| y` is syntactic sugar for `op(x, y)`; `f| x` and `x |f` for `f(x)`. The
`|op|` content may be a dotted **member path** (`|M.*|`, `|x.y.z|`) that tracks
down through a value's fields, so `x |M.*| y` is the member call `M.*(x, y)`. A
plain **symbolic** operator `x * y` also desugars to `*(x, y)` when `*` names a
bound value in scope (otherwise `*`, `+`, … keep their built-in arithmetic
resolution).

### Inferred Parameters From Argument Types

A `?`-suffixed name in a `when:` requirement — `A`/`B` in
`g is \function:on{A?}:to{B?}` — is an inferred parameter, solved at a use site by
unifying that requirement against a fact already known about its subject. So
`\restriction:of{g}:on{X}` used with a `g` of type `\function:on{P}:to{Q}` binds
`A := P`, `B := Q`, making a later requirement like `X \:subset:/ A` resolve to
`X \:subset:/ P`. The subject's type is followed through extension rules, so an
argument whose type merely *extends* a function type (e.g. a `\binary.operation`)
is matched too.

### `have:`/`asserting:`/`because?:`/`by?:` Assertions

A new clause group lets an author supply an explicit assertion where the
(deliberately simple) type system needs help:

```text
have: *_1 is \restriction:of{`*`}:on{X1 \.set.cross./ X1}
asserting: (.X1 \.set.cross./ X1.) \:subset?:/ (.X \.set.cross./ X.)
because: X1 \:subset?:/ X
by: \cross.of.subset.is.subset.of.cross#given{X := X; Y := X1}
```

- `have:` — the specification/statement/expression that would otherwise appear in
  this position.
- `asserting?:` — optional items **taken as true**; when present, the checker
  verifies that under them the `have:` item holds (an infix-spec or spec
  *question* such as `A \:subset?:/ B` is assumed as its `A \:subset:/ B` fact).
  When omitted, the `have:` item is checked using only the surrounding facts.
- `because?:` — justification clauses, and `by?:` theorem references
  (`\thm#given{…}`): these are **not** proven as logical consequences; their
  command/theorem references are still reference-validated.

It is accepted wherever a clause or specification goes — `declares:`,
`satisfies:`, `then:`, `suchThat:`, and so on. `have:`/`iff:` remains the
shorthand iff clause; the presence of `iff:` selects that group, and other
`have:` groups use the assertion/justification form.

### Stropped Operators As Values

A backtick-stropped operator `` `*` `` now resolves to the operator `*` it names,
so where `*` is bound (a magma's operation, a tuple component `M ::= (X, *)`, …)
`` `*` `` is that operator as a first-class value — it carries `*`'s type,
can be passed as an argument (`\restriction:of{`*`}:…`), and can be invoked in
function form as `` `*`(a, b) `` (equivalent to `a * b`). Stropping is stripped
for symbol lookup, keying, and type resolution, so `` `*` `` and `*` denote the
same symbol.

### Subscripted Operator Names; Destructuring Spec-Infix Headings

A symbolic operator name may carry a `_`-prefixed subscript, so `*_1`, `+_i`, and
`<=_max` are valid operators (a run of operator characters followed by a name
subscript), mirroring subscripted value names. This lets a tuple carry indexed
operations, e.g. `H ::= (X1, *_1, e1)`.

A spec-infix `Defines` heading whose left operand destructures now matches its
`Defines:` argument correctly: `[H ::= (X1, *_1, e1) \:sub:/ G ::= (X, *, e)]`
with `Defines: H ::= (X1, *_1, e1)` compares on the subject name (`H`) rather
than the full destructuring key, and the described subject is no longer wrongly
required to appear in `when:`.

### Operator Forms Bind Their Operator; Refines Inherits Base Specs

An operator form — `x_ * y_` (infix), `neg| x_` (prefix), `x_ !` (postfix) —
now introduces the operator symbol itself as a named value, alongside its
placeholders. So a body that uses the operator, such as a refinement's
`satisfies: (a * b) * c = a * (b * c)`, resolves `a * b` as the application
`*(a, b)` instead of reporting `*` as an unresolved built-in operator.

A `Refines:` also inherits the symbol specifications of the base type it refines.
`\(associative)::binary.operation:on{X}` refines `\binary.operation:on{X}`, whose
`extends: * is \function:…` already declares `*`; the refinement therefore need
not respecify `*`, and its uses of `*` are typed from the base. A refinement
target symbol is treated as specified when the base type declares it (through the
base's own `extends:`/`declares:` or described components).

### Refined Commands In `extends:`/`declares:`

The `is` relation of a `extends:` or `declares:` item may now name a refined
command as the type, e.g. `declares: * is \(associative)::binary.operation:on{X}`
or `extends: S is \(finite)::magma`. Previously only the `Refines:` `extends:`
accepted refined command references.

### Operand-Type Operator Capabilities

A plain operator resolves in order: the application desugar (when the symbol is
bound), a `Disambiguates` entry, then a provided-symbol capability owned by the
operands' common type. So a `\magma.element` that `Enables:`
`capability: x_ * y_ :=> …` makes `y * y` resolve for two magma elements. Values
known only through a spec (`y "in" M`) are reduced to their `is`-facts for this
owner-type match.

### Spec Capability Equivalence Is Explicit

A spec-operator capability
`x_ "in" G :<->: x_ is \group.element:of{G}` defines an equivalence:
`x "in" G` reduces to `x is \group.element:of{G}`, and the membership is also
established when the element type holds. The one-way spelling `:->` performs
only the first inference. Requirement proving tries each equivalent providing
capability disjunctively and requires all of a capability's target facts, with
a cycle guard.

### Member-Access Capabilities (`x.y`, `x.f(a_)`)

An `Enables:` `capability:` may now use a member-access left-hand side: `x.inv`
(member access) or `x.f(a_)` (member call). The owner must be exactly the
definition's `Defines:`/`Declares:`/`Refines:` subject (otherwise `Member
capability owner \`z\` must be the described item \`x\``). It is collected as a
provided member keyed by the member name and argument arity and owned by the
definition's type, so a use `p.inv` / `p.f(v)` on a value of that type resolves
to the reduction target. Previously a member-access LHS failed to parse and was
misreported as `expected top-level \`:->\``.

Member resolution now also recognizes owner types implied by specification
capabilities and instantiates the owner's type parameters in the member target.
For example, if `p "in" H` reduces to `p is \group.element:of{H}`, then `p.inv`
resolves through the group-element capability with its formal group parameter
bound to `H`.

Member access also accepts a grouped expression as its owner and carries the
owner expression's inferred output facts into capability lookup. Thus both
`(x * y).inv` and `(x.inv).inv` resolve when multiplication and `inv` produce
group elements. A bracketed placeholder-operator capability such as
`x_ [*] y_ :=> ...` is registered under the referenced `*` symbol rather than
the literal bracketed spelling.

### Bracketed Placeholder Operators `[*]`

A capability LHS may write `x_ [*] y_`, where `[*]` names a symbol drawn from the
definition's inputs/`Defines:` (e.g. the `*` component of `M ::= (X, *)`)
rather than a literal character. It parses as an infix-operator form whose
operator text retains the brackets.

### Destructuring Component Binding And Typing

A destructuring target `Name ::= (c1, …, cn)` introduces its components
(including operator components like `*`) and infers their types: from
`extends:`/`extends: … via …` and then `declares:` for a `Defines` target;
from the parameter's `when:` type for a command parameter `{M ::= (X, *)}`; and
from the right-hand type for a `given:`/`Declares:` binding `M ::= (X, *) is \T`.
Such components need no separate `when:` entry, and member access reaches them
(`M.X`, `M.*`). Only `::=` introduces components; `:=` requires its right-hand
side already in scope.

Definitions may assign a destructured infix operator pointwise in `expresses:`,
for example `(a1_, a2_) *_3 (b1_, b2_) := (a1_ *_1 b1_, a2_ *_2 b2_)`.
The two operand patterns become the operator mapping's parameters, while the
operator retains the type inferred from its enclosing destructured `Declares:`
target. Declarations in an `expresses:` block are processed in order, so an
earlier assignment such as `X3 := {...}` supplies facts needed by later
assignments. Destructured components of typed heading operands such as `G1 ::=
(X1, *_1, e1)` are likewise available to those assignments.

### `extends: … via …` Sets Component Types

`extends: M is \set via X` records `X is \set`; `extends: S is \magma via (X, *)`
maps the `via` tuple positionally onto `\magma`'s components, giving `X is \set`
and `* is \binary.operation:on{S}`. `declares:` then only needs to type
components the `via` does not cover.

### `States:` Requires `called:`/`written:`

Like `Defines`/`Declares`, a `States:` group must include a `called:` or
`written:` item in `Documented:`.

### Placeholder-Spec Capability Targets

A `:->` capability target may be a spec on the bound placeholder, e.g.
`capability: x_ "in" A :-> x_ "in" B` (meaning `x "in" A` implies `x "in" B`).

## Structural Language

### `Refines:` Refinement Markers (`implicitly:`/`explicitly:`)

A refinement may destructure the refined value, for example `Refines: G ::=
(X, *, e)`. The component names are local aliases whose positional shapes must
match the base type's `Defines:` target; an operator component must remain an
operator component, and the tuple arity must agree. The base component types and
specifications are inherited onto those local names for use in `satisfies:`,
`extends:`, `Requires:`, and `Enables:`.

A `Refines:` group may now carry an optional, zero-argument marker section —
`implicitly:` or `explicitly:` — placed immediately after `Refines:`. The two are
mutually exclusive and are stored as an `Option<RefinementKind>` on
`RefinesGroup`. They document (and let the checker verify) whether an explicitly
written refinement of a subtype merely restates the definition inherited from a
supertype's refinement, or overrides it with extra properties.

- A refinement of a base type is implied on that type's subtypes (a
  `\(finite)::group` is available because `\group` extends `\magma` and
  `\(finite)::magma` exists). The markers let authors write such a refinement out
  and signal their intent to readers.
- `implicitly:` — the body must contain **only** the inherited `extends:` clause
  (plus scaffolding `using:`/`when:`). Adding `satisfies:`, `Requires:`,
  `Enables:`, or `Justification:` is an error. The `extends:` clause must also
  literally name the parent type's refinement — the same adjective(s) applied to
  a direct supertype of the refined base type (`\(finite)::group` extends
  `\(finite)::magma`, since `\group` extends `\magma`) — and naming anything else
  is an error.
- `explicitly:` — the body must add **at least one** property beyond the
  inherited `extends:` clause; a body that is only the inherited `extends:` must
  be marked `implicitly:` instead.
- Either marker requires the refined base type to itself be a subtype of another
  type (to have a `extends:` clause of its own); using a marker on a
  non-subtype base is an error, and non-subtype refinements take no marker.
- Both markers reject any arguments ("`implicitly:` is a marker section and takes
  no arguments"), and specifying both reports "A `Refines:` may specify at most
  one of `implicitly:` or `explicitly:`".

### `Lemma:` and `Conjecture:` Items Removed

The top-level `Lemma:` and `Conjecture:` items are **removed**. Each had exactly the
same `given:` / `where:` / `then:` / `iff:` shape as `Theorem:` and differed only in
its label, so both are redundant — a `Theorem:` states the same result, and a
`related:` documentation link records how one result relates to another.

- Migrate a former `Lemma:` or `Conjecture:` by changing its head to `Theorem:`;
  every other section (including the name in `Documented:` `called:`) is unchanged.
- `Lemma:` and `Conjecture:` are no longer accepted top-level group heads and now
  produce an "Unexpected top-level group" error.

### `Enables:` `relation:` uses `represents:`; `connection:` removed

The directional `relation:` group nested inside `Enables:` — which relates the
described concept *to* another and registers a cast rule for the type checker —
had its cast section renamed and its keywords changed:

- The section `as?:` is renamed to `represents?:`.
- The keyword `\\viewable_as` is renamed to `\\coercion` and `\\encoded_by` to
  `\\encoding`. Any other value is an error pointing at the two allowed keywords.

The semantics are unchanged by the rename:

- `represents: \\coercion` records that a value of the described type **may be used
  where the related type is expected** — an ordinary coercion. If `\integer` has a
  `relation:` `to:` `\rational` marked `\\coercion`, then `x is \integer` lets you
  write the soft build `\rational@x`, and a `\baz{a}` requiring `a is \rational`
  accepts a `\integer` argument (the integer coerces to a rational).
- `represents: \\encoding` records an **abstraction boundary**: the described type
  does *not* coerce to the related type, and a soft build (`@`) will not cross
  it. Only the hard build `\bar@!x` follows an encoding. This expresses e.g. that
  an `\integer` is *encoded as* a `\set` without an integer *being* a set:
  `\set@x` will not work, but `\set@!x` will.

The separate `connection:` group under `Enables:` is **removed**. It is superseded
by the `relation:` `to:` group, which covers the same directional relationships and
additionally drives cast checking.

### Theorem-Like Result Names Move to `Documented:` `called:`

The head sections of the theorem-like items — `Axiom:`, `Theorem:`, and
`Corollary:` — no longer take an argument. Previously they accepted
an optional quoted name of the result; a name given there is now an error that
points at `Documented:` `called:`.

- A result's name is instead written in `Documented:` `called:`, exactly as
  definition items name themselves, so naming is uniform across the language.
- The name renders as the card's **title**. A command-headed theorem-like resolves
  its title through the command-signature registry (as before); a **heading-less**
  theorem-like now takes its title from `Documented:` `called:` too — previously a
  heading-less item's name did not render as a title.
- `Corollary:` still takes its `of:` section (what it follows from); only its head
  name moved.

### Top-Level `Topic:` Item

A new top-level item names a documentation topic and organizes topics into a
hierarchy.

```
[#some.name]
Topic: <text>*
within?: "#<parent.topic>"
Related?:
. to: "<#topic | \signature>"+
  means: <text>
Documented?: called: <text>
Id?: <text>
```

- The heading `[#some.name]` uses a `#` sigil followed by any number of dotted
  names (like a label or command path, but for topics). It renders as a human
  title by title-casing the dotted path — `#real.analysis` renders as "Real
  Analysis" — unless `Documented:called:` supplies an explicit rendering.
- `Topic:` carries optional descriptive prose; `within?:` names a parent topic
  (making this a sub-topic).
- `Related?:` lists relationships to other topics or definitions. Each entry's
  `to:` points at one or more targets and `means:` explains the relationship.
- **References are quoted text.** `within:` and each `to:` target are written in
  double quotes so a reference is never confused with a usage: `to: "#topic"` is a
  topic reference and `to: "\sin"` is the `\sin` *definition* (a **signature** — a
  `\command` with its arguments removed, so `\function:on{A}:to{B}` is written
  `\function:on:to`). A signature names the `Defines`/`Declares`/`Refines`/`States`/
  theorem-like item itself, not a use of it. `called:` (already quoted) rounds out
  the four quoted-text fields (`within:`, `to:`, `means:`, `called:`).
- `Documented?:` is restricted to a single `called:` field, which only controls
  how the topic title renders; other documentation fields are rejected.
- It is stated, not checked: topic and signature references are recorded but not
  required to resolve, and the item registers no command signatures or type facts.
  `mlg check` auto-inserts an `Id:` as for any top-level item.
- A top-level `Relation:` may relate two topics or definitions (see below),
  letting a document record relationships between topics and definitions as well
  as between concepts.

### Top-Level `Relation:` Item

A new top-level item states a bidirectional relationship between two concepts,
topics, or definitions.

```
Relation:
using?: <declaration>+
between: <declaration | "#topic" | "\signature">
and: <declaration | "#topic" | "\signature">
when?: <spec>+
means?: <statement | text>
Documented?: ...  Justification?: ...  Aliases?: ...  References?: ...  Metadata?: ...  Id?: ...
```

- `between:` and `and:` each name one side of the relationship: an unquoted
  declaration (such as `a is \real`) for a concept, or a **quoted-text reference**
  for a topic (`"#real.analysis"`) or a definition **signature** (`"\sin"`, or
  `"\function:on:to"` for `\function:on{A}:to{B}`), in any combination. Quoting a
  reference keeps a `\signature` distinct from a usage, matching `Topic:`'s
  `within:`/`to:` convention.
- `means?:` is either an unquoted **statement** (a clause) of what the
  relationship means, or a **quoted-text** prose description of it.
- `using?:` brings auxiliary declarations into scope (as on `Defines:`/`States:`)
  and `when?:` gives spec preconditions.
- It is heading-less (no `[...]`) and takes the same trailing sections as the
  theorem-like items. `mlg check` auto-inserts an `Id:` as for any top-level item.
- Whereas the directional `relation:` group inside `Enables:` relates the
  described concept *to* another (and with `represents: \\coercion`/`\\encoding`
  registers a cast rule the type checker uses), the top-level `Relation:` is
  standalone and bidirectional — e.g. for stating that two concepts are equivalent.
- It is checked like a theorem: any `between:`/`and:` *declarations* introduce
  their subjects and a *statement* `means:` is validated for declared symbols and
  valid command references. Quoted-text references and a prose `means:` are
  recorded, not checked — the relationship is stated, not proven, and it registers
  no type facts.

### Top-Level `Equivalent:` Item

A new top-level item declares that several commands are interchangeable under a
shared name.

```
[\foo:of{A}:with{B}]
Equivalent:
using?: <declaration>+
when?: <spec>+
to:
. \foo2{A, B}
. \foo3:with{B}:and{A}
Documented?: ...  Justification?: ...  References?: ...  Id?: ...
```

- The `[...]` heading names the equivalence class and registers a command
  signature (referenceable and duplicate-checked, like `Defines:`/`States:`).
  `mlg check` auto-inserts an `Id:`.
- Each `to:` command must use the header parameters directly, as bare names — no
  compound expressions, nested commands, or `using:` symbols.
- Local validation: every `to:` member must be defined and be one of
  `Defines`/`Declares`/`States`/`Refines`, and all members must be the same
  kind; they must share the same target shape and — by kind — the same `is` type
  (`Declares`), `extends:` target (`Defines`), or base type (`Refines`); they
  must provide the same capabilities (by name and arity); and the item's own
  `when:` must guarantee each member's requirements.
- Interchangeability: the members are mutually substitutable to the type checker.
  A value known to be one member (or the class-naming header) satisfies a
  requirement that it be any other member, provided the target member's header
  parameters are all pinned — to matching actuals — by the known member (so a
  member instantiated at *different* actuals is correctly not accepted). A
  capability or spec operator the class provides also resolves on a value typed
  as any member or the header, since every member provides the same capabilities.

### `equivalently:` Clause

A new clause `equivalently: <clause>+` asserts that its sub-clauses are all
mutually equivalent (sugar for a chain of `iff`s). It is checked like `allOf:`
(each sub-clause is validated in turn) and carries no additional type meaning. It
is not valid inside a `when:` section.

### Markdown MathLingua Fences

Any quoted text value (`Text:` Markdown, `description:`, `means:` prose, and so
on) may contain fenced blocks tagged `mlg` or `mlg-fragment`. Both render as
MathLingua source using the same syntax-highlighted presentation used by cards;
other fenced code blocks retain ordinary Markdown code rendering. Consecutive
MathLingua code blocks are now separated by a gap so they no longer run together.

- **`mlg`** blocks are whole items. `mlg check` validates their **syntax**: the
  fenced code is structurally parsed (after undoing the enclosing text's `\"`
  escaping and the fence's Markdown indentation), and any parse errors are reported
  against the containing file at the fenced line. Only syntax is checked — the code
  is never type-checked, so an example may freely reference commands, topics, or
  symbols that are not defined in the collection.
- **`mlg-fragment`** blocks may hold any snippet — structural, formulation, or
  header code — that need not be a complete item. They are highlighted the same
  way but are **never checked**, so a bare formulation or header renders without
  producing a syntax error.

### Page Content Blocks

Top-level page content blocks are supported and render directly on the page
rather than inside cards.

- `Title: "..."` renders as the page title.
- `SectionTitle: "..."` renders as a section heading.
- `SubsectionTitle: "..."` renders as a subsection heading.
- `Text: "..."` renders prose directly on the page.
- `Section:` was replaced by `SectionTitle:`.
- `Subsection:` was replaced by `SubsectionTitle:`.
- `Subsubsection:` was removed.
- Text literals may span multiple source lines.
- `Text:` content supports Markdown and LaTeX.
- Page headings use the same blue color family as card section labels.

### Top-Level IDs

Every top-level item must contain an `Id:` section.

- `mlg check` generates a missing `Id:` before checking the item.
- Generated IDs use a real UUID v4 implementation.
- The generated form is a normal random UUID, not a fixed or mostly-zero value.
- It is an error for two top-level items to have the same ID.
- `mlg check` adds only the missing `Id:` section. It does not add a separator
  line before the ID.
- The view shows the item ID in the hidden card details area.
- Internal item identity should use the `Id:` value rather than deriving identity
  from source position or rendered text.

### Table Of Contents Files

Directories may contain a `toc` file that controls the order and visibility of
entries in the left outline.

- If a directory has no `toc`, files are listed alphabetically.
- Fallback display names preserve capitalization semantics already used by the
  view and replace underscores with spaces.
- If a directory has a `toc`, every `.mlg` file in that directory must be listed.
- If a directory has a `toc`, every subdirectory in that directory must be listed.
- A listed file or directory that does not exist is an error.
- A `.mlg` file or directory omitted from the `toc` is an error.
- Each listed entry may be written as just the path, for example:
  `some_file.mlg`.
- Each listed entry may also use `-> HIDDEN` to hide it from the rendered view.
- Each listed entry may use `-> Some Title` to provide a custom display title.
- The same `toc` rules apply independently in each subdirectory.

## Documentation Text And Rendering Names

### `called:` And `written:`

`called:` and `written:` have distinct meanings.

- `called:` is non-math text.
- `written:` is math-mode text.
- For `Defines:` and `Declares:`, at least one of `called:` or `written:` is
  required.
- If both are provided, the renderer uses the appropriate one for the context.
- If only `called:` is provided, the missing `written:` text is generated from it.
- If only `written:` is provided, the missing `called:` text is generated from it
  by using the written text in math mode.
- The `called:` text is used for `Defines:` and `Declares:` labels and for the
  right-hand side of rendered `is` statements.
- The `written:` text is used when the item appears as an expression.
- Card titles remove placeholder markers such as `?` from rendered title text.
- Generated titles preserve normal lowercase words such as `is`; for example,
  `A? is a subset of B?` renders as `A is a subset of B`, not
  `A Is a subset of B`.

### `Refines:` Documentation

`Refines:` uses adjective-based documentation.

- `Refines:` entries require an `adjective:` item in `Documented:`.
- `Refines:` entries may contain `written:`.
- `Refines:` entries may not use `called:`.
- When a refined type is rendered, adjectives are printed before the base
  described type.
- Multiple refinements render as comma-separated adjectives, for example
  `injective, surjective function`.

### Conditional Documentation Fragments

`called:` and `written:` text support conditional fragments.

```text
@[<vars>]{<text-if-present>}:{<text-if-missing>}
```

Rules:

- The `:{<text-if-missing>}` part is optional.
- `<vars>` is a comma-separated list of variable names.
- `@[U]{_{U?}}` outputs its body only if `U` is specified.
- `@[U]{_{U?}}:{_X}` outputs `_X` if `U` is not specified.
- `@[x, y]{...}` requires both `x` and `y` to be specified.
- Conditional fragments may nest.
- Nested fragments are evaluated in the same variable environment as the outer
  text.

Example:

```text
@[x]{x + @[y]{y}:{*}}
```

If `x` and `y` are specified, this renders `x + y`. If `x` is specified and `y`
is not, it renders `x + *`.

## Formulation Syntax

### Transparent Grouping

Grouped expressions of the form:

```text
(. x + y .)
```

are supported as source-only grouping.

- The grouping disambiguates the source.
- The grouping parentheses are not rendered.
- `(. x + y .)` renders as `x + y`.

### Optional Command Tails

Optional command tails of the form `:?name{value}` are supported in command
headers and expressions.

- `:?within{U}` declares an optional tail.
- Optional tail variables are allowed in `when:` requirements.
- Optional tail variables are not required in `when:` requirements.
- In expressions, an optional tail is applied only when its value is defined.
- Rendering may use conditional documentation fragments to include or omit text
  based on optional tails.

### Names With Numeric Suffixes

Identifier-like names ending with digits render the digits as subscripts.

- `x1` renders as `x_1`.
- `abc123` renders as `abc_123`.
- This is a rendering rule for names, not a source-level rewrite.

### Builtin Kinds

Builtin kinds render as plain text.

- `\\statement` renders as `statement`.
- `\\expression` renders as `expression`.
- `\\specification` renders as `specification`.
- `\\anything` is satisfied by any value but does not establish any more specific
  type information.
- The renderer no longer treats these as a newline plus italic text.

### Refined Command Syntax

Refined command headers use the form:

```text
[\(adjective)::base.command:?tail{X}]
```

Rules:

- Only `Refines:` entries may use a refined command header.
- The `Refines:` section contains only the refined subject form.
- The `Refines:` section no longer repeats `is <base>`.
- The base command after `::` comes from the header.
- A refined expression may include multiple refinements, such as
  `\(injective, surjective)::function`.
- A refinement may extend other refinements with:
  `extends: f is \(injective, surjective)::[[f]]`.
- `[[f]]` is valid only in a refined expression of the form
  `\(... )::[[f]]`.
- `[[f]]` means "the current type of `f`", allowing the extension to apply to
  a more specific base type such as `bounded.function`.

### Function And Collection Shapes

`Defines:` and `Declares:` support richer target shapes.

Function targets:

- `Defines: f(x_) ::= y_` describes a function-like target with one input.
- `Defines: f(x__) ::= y_` describes a function-like target that accepts any
  number of inputs, treated as a single tuple.
- `Defines: f(x_, y_, z_) ::= w_` describes a function-like target that
  accepts exactly three separate arguments.
- `declares:` on such a `Defines:` target states the input and output
  requirements.
- `Declares: h(x__) := f(g(x__)) is \function:on{A}:to{C}` is accepted.

Collection targets:

- `Defines: X ::= {x__ : ...}` describes a collection shape whose elements
  may have any arity and are treated as a tuple.
- `Defines: X ::= {x_ : ...}` describes a collection shape accepting a single
  value, where that value may itself have any expression shape.
- `member_of` is a keyword used by enabled membership capabilities.
- `x member_of X` is valid only when `X` is a collection literal or has a
  collection literal attached by an explicit build.
- `\set@{x_ : x_ is \real}` builds a collection literal at the described type.
- `A := {x_ : x_ is \real | x_ > 2} is \set` binds the literal to `A` as a set.
- If `A := \set@{x_ : x_ is \real}` and `x "in" A`, the checker can
  establish `x is \real`.
- If `A is \set` without a collection literal, membership establishes
  `x is \\anything`.

### Set Builder Definitions

Set builder definitions allow general element forms before the colon.

- A set builder may use a name, tuple, function form, or other valid form in
  the binder position.
- For example, `{(a_, b_) : a_ "in" A, b_ "in" B}` is accepted.
- This applies in declarations and definitions such as:
  `Declares: C := {(a_, b_) : ...} is \set`.
- Specifications after the colon may be separated by `,` **or** `;`. The `;`
  form is also accepted after a build (`\set@{(a_, b_) : a_ "in" A; b_ "in" B}`).

### Function Literals

An anonymous mapping is written with `=>`:

- `(x_ is \real) => x_ + 1` maps an input to an output. A function-literal
  parameter must be a name with a spec (`(x_ is ...)`), or a bare name when the
  type is already known from an `is`. A bare parameter without a known type, or
  an undeclared symbol in the body, is rejected.

### Inferred Parameters

A command argument written `X?` introduces `X` inline the first time it appears:

- The first occurrence of `X?` at a command-argument position declares `X` into
  scope with the type its argument position requires; later uses are the plain
  name `X`. Reintroducing an already-introduced inferred parameter is an error.

### Spec Literals And `is` Indirection

- A `\\specification` value may be written with an anonymous `_` subject, e.g.
  `_ is \x` or `_ "in" X`, and instantiated by a `satisfies:` clause.
- `is` accepts type indirection: a type parameter can stand for a type
  (`x is T` where `T is \\type`), and `T is \\type` records that `T` is itself a
  type.

### Collection-Argument Sugar

A command argument that is a bare collection literal is sugared, so
`\foo{x_ : x_ is \real}` is treated as `\foo{{x_ : x_ is \real}}`.

## Semantic Checks

### Specifications Are Not Allowed Where Statements Are Expected

An `is` specification or infix specification introduces a symbol, so it is only
valid in binding positions (`exists:`, `given:`, `forAll:`, `let:`, `where:`,
`when:`, `suchThat:`). In a statement position (`then:`, `iff:`, `that:`, `if:`,
`not:`, `allOf:`, `anyOf:`, `oneOf:`, `equivalently:`) the predicate form must be used
instead: `x is? \set` rather than `x is \set`, and `A \:subset?:/ B` rather than
`A \:subset:/ B`.

### Editor Language Server

`mlg lsp` communicates over standard input/output and provides collection
diagnostics on open and save; context-aware completion for group heads, section
items, and commands; jump to definition for command uses; and workspace rename
for top-level command signatures and their uses. Command completion is triggered
after `\\` and uses snippets when the client supports them.

### `when:` Requirements

Definition-like entries validate `when:` against the parameters introduced by
their headers.

Rules:

- Required non-optional header parameters must have a corresponding `when:`
  requirement.
- Optional tail parameters are allowed in `when:` but are not required unless a
  `Defines:` entry references them in semantic constraints such as
  `declares:`, `extends:`, or `satisfies:`.
- Target symbols introduced by a declaration target such as `G ::= (X, *, e)`
  are not `when:` parameters unless they also occur in the command header.
- Target symbols introduced by `Defines:`, `Declares:`, and `Refines:` targets
  must have specifications directly, such as through `declares:`, `using:`, or
  an `is` relation, or transitively through `extends: ... via ...`.
- `A, B is \set` counts as both `A is \set` and `B is \set`.
- `P, Q is \\statement` counts as both `P is \\statement` and
  `Q is \\statement`.
- `when:` clauses only support:
  - `<subject> is <type>`
  - `<subject> "op" <target>`
- Assignments, definitions, and arbitrary expressions are not valid `when:`
  clauses.

### Symbol Scoping

The checker reports any ordinary symbol use that has not been introduced.

- Binding and assumption sections such as `given:`, `exists:`, `existsUnique:`,
  `forAll:`, and `let:` introduce their declared subjects.
- Clause-group `exists:`, `existsUnique:`, `forAll:`, and `let:`
  sections may contain multiple block arguments; each argument is introduced in
  order before the guard, predicate, or body is checked.
- Declaration definitions make declaration-side symbols available to the right
  hand side, so `f(x_) := x_` is valid.
- Declaration relations are checked too, so `Declares: f(x_) := x_ is
  \function:on{A}:to{B}` requires `B` to have been introduced.
- Membership assumptions bind the member side, but the collection side must
  already be declared.
- Explicit optional command tail arguments are checked for undeclared symbols
  even when the tail is inactive for requirement matching.

### Existential Clauses

Existential clause groups support optional predicates.

- `exists: x is \real` is valid without a `suchThat:` section.
- `existsUnique: x is \real` is valid without a `suchThat:` section.
- If `suchThat:` is present, it must contain one or more clauses.
- A present `suchThat:` section is checked the same way as before.
- If `suchThat:` is omitted, the clause still introduces the existential
  binding inside the existential's child context, but has no predicate clauses
  to assume.

### Type Facts And Extensions

The checker uses simple type facts and extension facts.

- `extends: X is \set` lets an item described by the refined type be used where
  a set is required.
- `extends: G is \set via X` records the extension through the given structural
  component.
- Facts introduced by `given:`, `when:`, `extends:`, `declares:`, and enabled
  membership capabilities are available while checking dependent statements.
- This is type establishment, not theorem proving.

### `Declares:` And `Defines:` Usage

The checker distinguishes values, definitions, and described types.

- `X is \foo` is used when `\foo` is a `Defines:` entry.
- `X := \foo` is used when `\foo` is a `Declares:` entry.
- `X := \set@{...}` is valid where a definition-style binding is expected.
- A `Declares:` entry may include an expression and result type, such as
  `Declares: C := A is \set`.

## Operators, Symbols, And Disambiguation

### Command Headers For Operators And Functions

Command headers may define special operators, named operators, and functions.

Examples:

```text
[x_ + y_]
[x_ |op| y_]
[f(x_)]
[f| x_]
[x_ |f]
```

Rules:

- These headers do not use colon-directed forms such as `:|op|`.
- Special operators are not limited to `+`, `-`, `*`, `/`, `=`, or `^`.
- Any sequence of special operator characters may be an operator.
- Operators may include named suffixes such as `*_1` or `*_free`.
- Prefix and postfix named operators are treated as one-argument functions.
- Infix named operators are treated as two-argument functions.
- `f| x` is equivalent to `f(x)`.
- `x |f` is equivalent to `f(x)`.
- `x |op| y` is equivalent to `op(x, y)`.
- `x * y` is equivalent to `` `*`(x, y) ``.

### Type-Directed Operator Resolution

Type-directed operator forms are supported.

- `x :- y` resolves `-` from the type of `x`.
- `x -: y` resolves `-` from the type of `y`.
- `x :-: y` resolves `-` from the least common ancestor type of `x` and `y`.
- If both operands have the same type, `x :-: y` resolves from that type.
- The same rule applies to named operators, for example `x :|op| y`,
  `x |op|: y`, and `x :|op|: y`.
- Resolution searches the `Requires:` and `Enables:` capabilities on the
  selected type and its parent types.
- It is an error if the operator is not enabled by the selected type hierarchy.
- It is an error if the resolved operator's requirements do not match the
  operands.

### Plain Operator Resolution

Plain operators such as `x - y` use scope and disambiguation.

- The checker first searches local scope, moving outward.
- If no local definition is found, it searches global command definitions.
- If a matching `Disambiguates:` entry exists, branches are considered in order.
- The first `when:` branch whose requirements match the operands is used.
- If no branch matches and an `else:` branch exists, the `else:` branch is used.
- A `Disambiguates:` entry may contain only an `else:` branch.
- If no definition or applicable disambiguation is found, the operator is an
  error.
- Plain `=` and `!=` are exceptions: they may be written for any operand types
  without a definition. If the common operand type enables `=` or `!=`, that
  capability is still used.
- Fallback `=` and `!=` expressions are treated as statements, so they can be
  passed to commands that require `\\statement`.

### `Requires:` And `Enables:`

Types can now separate definitional requirements from additional capabilities.

- `Requires:` is accepted on command-backed top-level entries that support
  `Enables:`.
- `Requires:` must appear before `Enables:` when both are present.
- `Requires:` accepts `capability:` groups.
- `Requires:` accepts `definition:` groups of the form `\command is <spec>`.
- Capabilities from `Requires:` and `Enables:` are unioned for type checking.
- `Requires:` is intended for operations that are part of the definition of a
  construct.
- `Enables:` is intended for additional supported operations that come from
  other definitions.
- A `Requires.definition:` item succeeds only when the referenced command is a
  top-level `Declares:` item and that definition's output facts establish the
  requested `is <spec>` fact.
- A `Requires.definition:` item fails if the referenced command is undefined,
  is not a `Declares:` entry, or does not establish the requested fact.
- `Enables:` accepts cast-backed `from:` groups.
- A `from:` group must contain exactly one of `capability:` or `as:`.
- `from: ... capability:` capabilities are used only when the actual target has
  a recorded cast or set literal.
- When reducing a `from: ... capability:` rule, the source subject from the
  `from:` declaration is substituted with the actual target value.
- Ordinary non-`from:` capabilities on opaque targets no longer read cast
  literals through `member_of`; literal-backed membership requires a structural
  target or an explicit `from:` capability.
- `from: ... as:` records and validates an expression binding that describes
  how to view the cast source as the described form.
- A `from: ... as:` binding can reduce facts about a casted function call by
  matching the binding's left side against the call and substituting the right
  side into facts from the cast literal.
- `Enables:` accepts `relation:` groups with required `to:` declarations and
  optional `when:`, `means:`, `represents:`, and `by:` sections.
- The `:= ...` construction in a `relation:` `to:` declaration is optional.
- `relation:` entries marked with `represents: \\coercion` provide ordinary cast
  relationships.
- `relation:` entries marked with `represents: \\encoding` provide hard-cast
  abstraction relationships for `as!`.
- View relationships may satisfy requirements after a command or operator has
  already resolved.
- View relationships are not used to resolve operators or capabilities.

### Capability Rules

`Requires:` and `Enables:` capabilities define type-specific notation.

Rules for `:=>`:

- In an infix capability such as `x_ - y_ :=> ...`, both operands are treated as
  values of the type currently being described.
- In prefix or postfix capabilities, the single operand is treated as a value
  of the type currently being described.
- The right-hand side may use the described subject and the capability operands.

Rules for `:->`:

- The right-hand side must be the item being defined by the capability.
- The left-hand side does not receive an implicit type from the described item.
- This is used for specification operators such as membership.
- The checker infers only from the left-hand specification to the right-hand
  target or targets.

Rules for `:<->:` are the same, except that the checker may also infer the
left-hand specification after establishing every right-hand target.

Function and value capabilities:

- A function capability such as `f(x_) :=> \foo{X, x_}` is used as `X.f(a)`.
- Arguments of such function capabilities do not receive an implicit type.
- A bare capability such as `a :=> \some.value{X}` is used as `X.a`.
- A callable-owner capability may use the described subject as the function
  name, for example `R(a_, b_) :-> (a_, b_) "in" R`.
- If `R is \relation:from{A}:to{B}`, then `R(a, b)` resolves through that
  capability, reducing to the capability target with `R`, `a_`, and `b_`
  substituted.
- A callable-owner capability's `written:` text is used by `mlg view` to render
  matching calls, for example `written: "a_? \: R \: b_?"` renders `R(a, b)` as
  `a \: R \: b`.
- All capabilities have access to the subject of the item being
  described.

Set expression literals now also accept the unconstrained ellipsis form, such as
`{(p_, q_) : ...}`. This is useful in `from:` declarations that describe the
shape of an accepted cast source without adding element constraints.

### Built-In `\\type`

The checker supports the built-in type predicate `\\type`.

- `\foo is? \\type` succeeds when `\foo` is a top-level `Defines:` entry.
- `\foo is? \\type` fails when `\foo` is a `Declares:` entry.
- `\foo is_not? \\type` succeeds when `\foo` is not a described type.
- Ordinary built-in type facts share the same fact-checking path as
  `\\statement`, `\\expression`, and `\\specification`.

## Rendering And View

### Viewer Serving And Navigation

- The development viewer listens on `0.0.0.0` while reporting a localhost URL.
- Route links are warmed in the background so page navigation starts loading
  before selection.
- Loading routes display skeleton content.
- Directory display names use the directory name rather than an internal page
  title, while `toc` display overrides continue to take precedence.

### Math Rendering

Rendering behavior was tightened in several places.

- `(. ... .)` source groups do not render visible parentheses.
- Defined command references render according to their documented `written:`
  or derived written form.
- `called:` content is rendered in text mode.
- `written:` content is rendered in math mode.
- If an axiom, theorem, or similar entry has no explicit name, its label is
  converted to a display name by replacing dots with spaces and capitalizing
  words.
- For example, `\axiom.of.unordered.pair` renders as
  `Axiom Of Unordered Pair`.
- LaTeX package support includes the packages needed for common math such as
  `\emptyset`.
- There is no arbitrary fallback that special-cases commands such as
  `\empty.set`.

### Card View

The card view includes source and detail controls.

- Each top-level card has a subtle source icon in the top-right corner.
- Clicking the icon flips the card to show syntax-colored MathLingua source.
- The source back side has a white background and no gray border, so the source
  appears directly on the card.
- Source view margins and padding are compact.
- `Documented:` is hidden behind the card expander.
- `Enables:` is also hidden behind the card expander.
- The item `Id:` appears in the hidden details area.

### Definition Drilldown

Clickable definitions in cards open related definition cards below the current
card.

- The definition area is rendered as an inset, etched region.
- The inset region has its own close button that closes all cards in that
  region.
- Individual close buttons remove only the selected definition card.
- Clicking another definition appends that card to the top of the definition
  list.
- Opening a new definition does not replace or remove previously opened
  definition cards.
- Definition cards can themselves contain clickable definitions, allowing a
  vertical chain of exploration.

### Navigation And Outline

The viewer has responsive navigation behavior.

- Pages have subtle previous and next navigation buttons.
- The buttons show only the destination section names.
- The first page does not show a previous button.
- The last page does not show a next button.
- On desktop-width clients, the left outline is open by default.
- On narrow screens and mobile, the left outline is closed by default.
- On narrow screens and mobile, selecting an entry closes the outline.
- On desktop, selecting an entry keeps the outline open.
- Initial render state is aligned between server and client to avoid hydration
  mismatches.
- Loading a specific route no longer flashes the first table-of-contents page
  before switching to the requested page.
- Directory names in the outline use the same font sizing as other outline
  entries.

### Refresh Behavior

`mlg view` refresh behavior was changed.

- Refreshing the browser reparses the MathLingua source.
- The user does not need to stop and restart `mlg view` to see source changes.
- If the updated source has errors, the view keeps the last valid rendered data.
- In that error case, standard output shows the errors that prevented the view
  from updating.

## CLI

### `mlg.json` Must Spell Out Every Field

`mlg.json` no longer has implicit defaults: a valid config must contain every
field, so the whole configuration is visible and editable in one place rather
than split between the file and defaults applied behind the scenes.

- The required fields are `name`, `version`, `margin`, `formatOnCheck`, and
  `outputDir`. Keys are camelCase. `mlg check` reports each missing one as
  `mlg.json is missing required field "<field>"`. Each is typed: `name`/`version`
  are strings, `margin` is a positive integer, `formatOnCheck` is a boolean, and
  `outputDir` is a non-empty relative path inside the collection (a bad value
  reports `... must be a non-empty relative path within the collection`).
- `mlg init` writes a fresh `mlg.json` with every field at its default, so the
  author sees all of them:

  ```json
  {
    "name": "",
    "version": "0",
    "margin": 80,
    "formatOnCheck": true,
    "outputDir": "docs"
  }
  ```

- When `mlg init` finds an existing `mlg.json` that is missing fields, it asks
  (on an interactive terminal) whether to fill them in with their defaults. It
  preserves existing values and any extra fields, and appends the filled-in ones.
  Without a terminal it reports the gaps and leaves the file unchanged.
- The runtime accessors still fall back to the defaults for a partial config so
  other commands keep working, but the collection will not pass `mlg check`
  until the fields are present.
- A config carrying the old `print_margin` key is treated as missing `margin`;
  the existing rename error stands in for the missing-field error rather than
  reporting both.

### `mlg check` Formats The Collection First

`mlg check` runs the same formatting pass as `mlg format` over the collection
before checking it, so a checked collection is also a formatted one.

- The `mlg.json` field `formatOnCheck` controls this. It must be a boolean; a
  collection is formatted unless it opts out with `"formatOnCheck": false`.
  (It was originally optional and defaulting to `true`; it is now a required
  field — see *`mlg.json` Must Spell Out Every Field* above.)
- Formatting is whole-collection even when the check is narrowed to explicit
  paths: a check of a few files already reads the whole collection to resolve
  them, and formatting only the named files would leave the rest in whatever
  state the last check happened to touch.
- It runs *before* the source is parsed. Checking the pre-format source would
  report line and column positions that no longer exist by the time the author
  reads them.
- Files that were rewritten are reported as `Formatted N files`, ahead of the
  check summary. A run that changed nothing says nothing about formatting.
- Outside a collection — no `mlg.json` in the current directory or any ancestor
  — there is no root to format and no config to read, so `mlg check` on loose
  files formats nothing.

### `Text:` Blocks, Code Fences, And Escapes

`Text:` and other quoted prose is Markdown with embedded LaTeX, and both the
checker and the formatter treat it carefully:

- A ```` ```mlg ```` fenced code block inside quoted prose is parsed as real
  source and reports `Syntax error in \`mlg\` code block: ...` when it fails to
  parse; ```` ```mlg-fragment ```` and non-`mlg` fences are skipped. A closing
  ```` ``` ```` glued directly to the text terminator (```` ```" ````) is
  handled correctly.
- `mlg check` reads escaped quotes (`\"`) inside `Text:` values without
  corrupting the surrounding text, and no longer over-collapses backslash
  escapes (so embedded `\\command` builtins survive round-tripping).
- `mlg format` reflows prose in `Text:` blocks to the configured margin but does
  **not** reformat fenced code blocks or Markdown list items (`* `, numbered
  lists), preserving their structure; text content is de-indented before
  rendering.

### `mlg view` Prefaces And Navigation

A `_preface_.mlg` file in a directory is excluded from the page list and instead
rendered as that directory's preface (its overview), and the left-panel
collection navigation orders and titles entries from the directory `toc` files.

### `mlg help` Descriptions

`mlg help` and each subcommand's `--help` print concise descriptions of every
command and its options.

### `mlg.json` `print_margin` Renamed To `margin`

The optional `mlg.json` field controlling the target line width for `mlg format`
is renamed from `print_margin` to `margin`.

- `margin` must be a positive integer. (It was originally optional, defaulting
  to a width of 80; it is now a required field — see *`mlg.json` Must Spell Out
  Every Field* above.)
- `print_margin` is **no longer read**. Because unknown fields in `mlg.json` are
  otherwise ignored for forward compatibility, a collection still carrying the
  old key would silently fall back to the default width, so the old key is
  rejected with an error naming the new one:
  `mlg.json field "print_margin" was renamed to "margin"; rename it to keep the
  configured width`.
- Migrate by renaming the key; the value is unchanged.

### Default Format Margin Changed From 100 To 80

The default target line width for `mlg format` — used when `mlg.json` has no
`margin` field — is changed from 100 to 80.

- A collection that sets `margin` explicitly is unaffected.
- A collection that relies on the default will have its inline `"..."` text
  values reflowed to the narrower width the next time `mlg format` runs. Set
  `"margin": 100` in `mlg.json` to keep the previous width.

### `mlg export`

`mlg export` checks and renders the current collection, then builds a static
copy of the viewer. The output directory is the `mlg.json` `outputDir` field
(default `docs`), written at the collection root (the conventional GitHub Pages
source folder), so there is no `--output` option. `--force` replaces a nonempty
output directory, and `--cname` writes a GitHub Pages `CNAME` file. The export
also writes `.nojekyll` and the route data required by the static viewer.

The base path for subpath hosting is inferred automatically: for a GitHub
Pages **project** site the base path is derived from the git remote (the
repository name), so a project site is linked correctly without configuration;
`--base-path` overrides the inferred value.

### `mlg clean`

`mlg clean` removes the generated output directory (`outputDir`, default `docs`)
from the collection (the
inverse of `mlg export`). It must be run inside a Mathlingua collection (a
directory tree containing `mlg.json`) and is a no-op when `docs/` is absent.

### `mlg release`

`mlg release --summary "<text>"` records an immutable, content-addressed snapshot
of the collection into a `metadata/` directory next to `content/`.

- `--summary` is required and describes the release.
- The command aborts unless the collection is inside a Git repository with no
  uncommitted, unstaged, or untracked changes, and `mlg check` reports no errors.
- The new repo version is the integer `version` in `mlg.json` plus one, written
  back to `mlg.json` (other fields are preserved).
- `metadata/collection.json` is an append-only list; each release appends
  `{version, version_control_sha256, summary}`, where `version_control_sha256` is
  the current `HEAD` commit.
- `metadata/items/<id>.json` is an append-only version history per top-level item,
  each entry `{version, sha256, repo_version}` where `sha256` is the SHA-256 of
  the item's source and `version` is a per-item counter.
- An item gains a new entry when its content hash differs from its latest recorded
  entry.
- When a definition (`Declares:`, `Defines:`, `States:`, `Refines:`,
  `Disambiguates:`) is (re)versioned, every definition it uses is re-versioned as
  well, transitively across the dependency graph.
- The dependency graph is computed as a DAG before anything is written, so a
  definition reached by several changed items is updated once per release, not
  once per use. Items reached only by propagation record their current (unchanged)
  hash under a new version number.
- Non-definition items (page content, people, resources, theorem-like items) are
  versioned on their own content but do not propagate to anything.
- On success it prints the new repo version, the commit sha, a count of updated
  items, the summary, and the updated items grouped by top-level kind. Each item
  is shown with its `previous → new` version (or `new → v1` for a first release),
  and the version column is aligned across the whole report. Items with a bracket
  heading are shown by that heading, page content (`Title:`/`Text:` and the like)
  by a truncated preview of its text, and anything else by its id.
- When a release both changes some items and propagates version bumps to others,
  each item is tagged `changed` (its own contents changed) or `propagated` (it was
  re-versioned only because a definition that uses it changed), and the `Updated`
  line shows the breakdown, e.g. `4 of 127 items (1 changed, 3 propagated)`. The
  tags are omitted when there is nothing to distinguish (for example a first
  release, where every item is a content change).
- `--dry-run` computes and prints exactly what the release would record but writes
  nothing: no metadata files and no `mlg.json` version bump. It still requires a
  clean Git repository and a passing `mlg check`, so the preview matches what a
  real release would do.
- `--diff` additionally prints a line-level diff of each item whose contents
  changed since the previous release, comparing the item against its source at the
  previous release's commit. Items updated only by propagation (unchanged content)
  appear in the summary but are omitted from the diffs. The flag works with or
  without `--dry-run`, so the intended flow is `mlg release --dry-run --diff` to
  review, then `mlg release` to record.
- A real (non-dry-run) release then regenerates the published site: it removes
  `docs/` and runs `mlg export`, so `docs/` reflects the version just recorded.
  The regenerated `docs/` (along with the new `metadata/` and the `mlg.json`
  version bump) is left uncommitted for the author to commit. A dry run does not
  touch `docs/`.

### `mlg debug`

A hidden `mlg debug` command was added for parser exploration.

- It opens a textual user interface.
- The user can select formulation, structural, or command-header mode.
- The left panel is text input.
- The right panel shows parse errors and the parse tree.
- Parse trees are pretty-printed with real newlines and nested structure rather
  than a single escaped string.
- The command is hidden from normal help output where supported by the CLI
  framework.

### `mlg extract`

A hidden `mlg extract <id>` command was added for building reproducible test
cases out of a working collection.

- It takes the `Id:` of one top-level item and prints that item together with
  every definition it depends on, transitively.
- Dependencies come from the same graph `mlg release` uses: a command occurrence
  is resolved through the semantic signature registry, so the edges match what
  go-to-definition sees.
- Items are printed in dependencies-first order — each item appears after
  everything it uses, with the requested item last — separated by two blank
  lines, the same gap `mlg format` normalizes to, so the output is already in
  canonical form. An item reachable by several paths is printed once.
- The output is exactly the items' source slices, so pasting it into a fresh
  collection reproduces the original behavior: it passes `mlg check` whenever
  the collection it came from did.
- An id that no top-level item carries is an error naming that id, and running
  outside a collection is an error.
- A collection that does *not* check cleanly is not a failure — reproducing a
  case the checker mishandles is the point. The check pass still runs (it is
  what resolves dependency edges and fills in missing `Id:` sections), but its
  diagnostics are collected separately: only a count is reported, on stderr,
  pointing at `mlg check` for detail. Extracted source therefore always leaves
  stdout clean enough to redirect straight into a file.
- The command is hidden from normal help output where supported by the CLI
  framework.

### `mlg report`

A hidden `mlg report <id>...` command was added for filing parser issues against
`mathlingua/mathlingua`.

- It takes one or more `Id:` values and extracts them exactly as `mlg extract`
  does, so a report always quotes the same self-contained collection.
- It opens `$VISUAL` (else `$EDITOR`, else `vi` — `notepad` on Windows) on a
  Markdown template: a title line, prompts for what happened and what was
  expected, and the extracted MathLingua already fenced under `Reproduction`.
  Either variable may carry arguments, as `EDITOR="code --wait"` does.
- The fence around the extracted code is one backtick longer than the longest
  backtick run inside it, so an item whose quoted text embeds its own ` ```mlg `
  fence cannot close the block early.
- On exit the finished issue is printed and the user chooses `[r]eport`,
  `[e]dit`, or `[c]ancel`. Editing reopens the editor on the current text and
  asks again. An unrecognized answer re-asks; a blank answer is not a choice, so
  pressing Enter never posts. Cancelling reports nothing and is not a failure.
- The issue title is the first non-empty line with any leading `#` stripped; the
  body is everything after it. An issue with no title is an error and posts
  nothing.
- Reporting posts through `gh issue create --repo mathlingua/mathlingua` when
  the `gh` CLI is available, which carries the whole body and needs no browser.
  When `gh` is absent it falls back to opening a prefilled issue form in the
  browser; a body too large to survive a URL is replaced with a pointer to
  `mlg extract` rather than silently truncated. A `gh` that runs but refuses
  (typically unauthenticated) is reported rather than falling back, so its
  complaint is not discarded.
- The command requires an interactive terminal and errors otherwise, directing
  the user to `mlg extract` for the scriptable path.
- The command is hidden from normal help output where supported by the CLI
  framework.

### `mlg whte_rbt.obj`

A hidden easter-egg command was added.

- The command is `mlg whte_rbt.obj`.
- It prints the Jurassic Park style security-interface transcript.
- It types commands with small delays.
- It ends by repeatedly printing `YOU DIDN'T SAY THE MAGIC WORD!`.
- The command is hidden from normal help output where supported by the CLI
  framework.

## Error Reporting

Error messages were made more user-facing.

- Requirement failures now say `Could not establish requirement` rather than
  `Could not prove requirement`.
- The wording avoids implying theorem proving.
- AST debug representations are avoided in user-facing messages.
- Expressions in diagnostics are rendered in source-like form, such as `A - B`.
- Paths under the configured content directory are shortened. For example,
  `content/sets/set.mlg` is reported as `sets/set.mlg`.
- Line and column information is reported where the checker can locate the
  relevant source span.
- Parser errors surfaced through `mlg debug` are formatted for readability.

# Implementation Changelog

This document records the recent MathLingua language, checker, renderer, view,
and CLI behavior implemented in this repository. It is intentionally rule-focused:
each section captures not only the feature, but also the conditions under which
the feature is valid.

## Structural Language

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
- For `Describes:` and `Defines:`, at least one of `called:` or `written:` is
  required.
- If both are provided, the renderer uses the appropriate one for the context.
- If only `called:` is provided, the missing `written:` text is generated from it.
- If only `written:` is provided, the missing `called:` text is generated from it
  by using the written text in math mode.
- The `called:` text is used for `Describes:` and `Defines:` labels and for the
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
- `\\opaque` is satisfied by any value but does not establish any more specific
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

`Describes:` and `Defines:` support richer target shapes.

Function targets:

- `Describes: f(x_) ::= y_` describes a function-like target with one input.
- `Describes: f(x__) ::= y_` describes a function-like target that accepts any
  number of inputs, treated as a single tuple.
- `Describes: f(x_, y_, z_) ::= w_` describes a function-like target that
  accepts exactly three separate arguments.
- `specifies:` on such a `Describes:` target states the input and output
  requirements.
- `Defines: h(x__) := f(g(x__)) is \function:on{A}:to{C}` is accepted.

Collection targets:

- `Describes: X ::= {x__ : ...}` describes a collection shape whose elements
  may have any arity and are treated as a tuple.
- `Describes: X ::= {x_ : ...}` describes a collection shape accepting a single
  value, where that value may itself have any expression shape.
- `member_of` is a keyword used by enabled membership capabilities.
- `x member_of X` is valid only when `X` is a collection literal or has a
  collection literal attached by an explicit cast.
- `{x_ : x_ is \real} as \set` casts a collection literal to the described type.
- `A := {x_ : x_ is \real | x_ > 2} is \set` binds the literal to `A` as a set.
- If `A := {x_ : x_ is \real} as \set` and `x "in" A`, the checker can
  establish `x is \real`.
- If `A is \set` without a collection literal, membership establishes
  `x is \\opaque`.

### Set Builder Definitions

Set builder definitions allow general element forms before the colon.

- A set builder may use a name, tuple, function form, or other valid form in
  the binder position.
- For example, `{(a_, b_) : a_ "in" A, b_ "in" B}` is accepted.
- This applies in declarations and definitions such as:
  `Defines: C := {(a_, b_) : ...} is \set`.

## Semantic Checks

### `when:` Requirements

Definition-like entries validate `when:` against the parameters introduced by
their headers.

Rules:

- Required non-optional header parameters must have a corresponding `when:`
  requirement.
- Optional tail parameters are allowed in `when:` but are not required unless a
  `Describes:` entry references them in semantic constraints such as
  `specifies:`, `extends:`, or `satisfies:`.
- Target symbols introduced by a declaration target such as `G ::= (X, *, e)`
  are not `when:` parameters unless they also occur in the command header.
- Target symbols introduced by `Describes:`, `Defines:`, and `Refines:` targets
  must have specifications directly, such as through `specifies:`, `using:`, or
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
  and `forAll:` introduce their declared subjects.
- Clause-group `given:`, `exists:`, `existsUnique:`, and `forAll:` sections may
  contain multiple block arguments; each argument is introduced in order before
  the guard, predicate, or body is checked.
- Declaration definitions make declaration-side symbols available to the right
  hand side, so `f(x_) := x_` is valid.
- Declaration relations are checked too, so `Defines: f(x_) := x_ is
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
- Facts introduced by `given:`, `when:`, `extends:`, `specifies:`, and enabled
  membership capabilities are available while checking dependent statements.
- This is type establishment, not theorem proving.

### `Defines:` And `Describes:` Usage

The checker distinguishes values, definitions, and described types.

- `X is \foo` is used when `\foo` is a `Describes:` entry.
- `X := \foo` is used when `\foo` is a `Defines:` entry.
- `X := {...} as \set` is valid where a definition-style binding is expected.
- A `Defines:` entry may include an expression and result type, such as
  `Defines: C := A is \set`.

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
  top-level `Defines:` item and that definition's output facts establish the
  requested `is <spec>` fact.
- A `Requires.definition:` item fails if the referenced command is undefined,
  is not a `Defines:` entry, or does not establish the requested fact.
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
- `Enables:` accepts `view:` groups with required `as:` declarations and
  optional `means:` clauses.
- The `:= ...` construction in a `view:` `as:` declaration is optional.
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

- `\foo is? \\type` succeeds when `\foo` is a top-level `Describes:` entry.
- `\foo is? \\type` fails when `\foo` is a `Defines:` entry.
- `\foo is_not? \\type` succeeds when `\foo` is not a described type.
- Ordinary built-in type facts share the same fact-checking path as
  `\\statement`, `\\expression`, and `\\specification`.

## Rendering And View

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

## Testbed Content

The testbed content was expanded to exercise the implemented behavior.

- Existing set content was updated with real `Title:`, `SectionTitle:`,
  `SubsectionTitle:`, and `Text:` blocks.
- The set page now includes starter definitions such as singleton and unordered
  pair examples.
- New starter pages were added for natural numbers, integers, rationals, reals,
  algebra, and analysis.
- The number-system pages include simple carrier sets, element types, initial
  constructors, and a small tower of extensions:
  `natural number -> integer -> rational -> real`.
- Algebra includes starter structures such as semigroup, monoid, ring, and
  field.
- Analysis includes starter entries for real sequences, convergent sequences,
  limits, and continuous functions.
- The testbed `toc` was updated to include the new pages in a stable order.

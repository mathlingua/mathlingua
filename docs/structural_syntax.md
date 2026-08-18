# Structural Syntax

This file describes the structural language exactly as it is currently implemented in the Rust code.
For a more readable overview of how structural syntax fits into the whole
language, start with [language.md](language.md); for the same territory shown as
worked examples, see [examples.txt](examples.txt).

Intended workflow:

1. Treat this file as the editable syntax spec for the structural language.
2. When the structural language should change, update this file first.
3. Then update the code in `src/frontend/proto/` and `src/frontend/structural/` to match.

At the time this file was written, it matches these implementation files:

- `src/frontend/proto/lexer.rs`
- `src/frontend/proto/ast.rs`
- `src/frontend/proto/parser.rs`
- `src/frontend/structural/ast.rs`
- `src/frontend/structural/parser.rs`

## Overview

The structural language is implemented in two layers:

1. A line-oriented proto syntax parser in `src/frontend/proto/`.
2. A structural AST builder in `src/frontend/structural/` that interprets proto groups by section labels and formulation subparsers.

The structural AST is intentionally section-oriented:

- every syntactic node is a `*Group`
- each group contains `*Section` fields
- optional sections are `Option<...Section>`
- repeated contents are carried inside the section:
  - `OneOrMore<T>`
  - `ZeroOrMore<T>`

This means optionality and multiplicity are separated:

- `when: Option<WhenSection>` means the `when:` section is optional
- `WhenSection { arguments: OneOrMore<Clause> }` means if `when:` is present, it must contain one or more clauses

## Notation

This document uses the following notation:

- `X` means exactly one value in the section.
- `X+` means one or more values in the section.
- `X*` means zero or more values in the section.
- `section?: X+` means the section itself is optional.
- `heading = command` means the group heading must parse as a formulation command header.
- `heading = label` means the group heading must parse as a structural label header.
- `heading = none` means the group must not have a heading.

All section labels are case-sensitive and must appear in the exact order shown below.

## Layer 1: Proto Surface Syntax

Before the structural AST is built, the input is parsed into proto groups.

### Lines

The proto lexer works line-by-line. For each input line:

- leading spaces become indentation
- if the trimmed line starts with `. `, that prefix is removed from the stored text and the logical indent is increased by `2`
- the original `. ` marker is preserved only through metadata and display rendering

Example:

```text
  . x
```

becomes:

- indent `4`
- text `x`
- `has_dot = true`

### Comments and blank lines

- blank lines are lines whose text is empty after removing leading spaces
- comment lines are lines whose trimmed text starts with `--`
- at the top level, blank lines and comments are skipped before looking for the next group
- inside groups and sections, comments are skipped but blank lines terminate the current block

### Headings

A proto heading line is any non-text line whose stored text:

- starts with `[`
- ends with `]`

The inside text is used as the raw heading string.

Example:

```text
[\function:on{A}]
```

The proto parser does not know what kind of heading this is. The structural parser decides later.

### Text literals

A proto text literal is any line whose stored text:

- starts with `"`
- ends with `"`

No escaping is interpreted at this layer.

### Sections

A proto section line is a line at the current group indent that contains a
structural section colon.

Surface shape:

```text
label:
label: inline argument
```

The first structural section colon splits the label from the optional inline
argument. The label prefix must be non-empty and contain only ASCII letters,
digits, and `_`.

### Nested arguments

Arguments belonging to a section are expected at indent `section_indent + 2`.

Each argument line is classified in this order:

1. text literal if the whole line starts and ends with `"`
2. nested group if the line is a heading line or has a structural section colon
3. formulation otherwise

Important implementation consequence:

- a non-text argument line starts a nested group only when the first colon has a section-label-shaped prefix made from ASCII letters, digits, and `_`
- colons in formulation delimiters `::=`, `:=`, `:?`, `:->`, `:=>`, and `:~>` do not start nested groups
- command tails such as `\function:on{X}:to{Y}` are formulations because the text before the first colon is not a section-label-shaped prefix

### Multiline formulations

An inline argument or formulation line becomes a multiline formulation block only if its entire text is exactly one of:

- `(`
- `[`
- `{`
- `(.`

The parser then consumes following lines until it finds a line at the same indent whose text is exactly the matching close delimiter:

- `)`
- `]`
- `}`
- `.)`

The opening delimiter line is stored as its normalized text. Following consumed
lines are rendered back with their indentation and `. ` markers in the stored
formulation text.

### Single-quoted formulations

Single-quoted formulations are explicitly rejected by the proto parser:

```text
'x'
```

This produces a diagnostic:

```text
Single-quoted formulations are not allowed
```

### Proto grammar summary

```text
Document ::= Group*

Group ::= HeadingLine? Section*

HeadingLine ::= "[" RawHeadingText "]"

Section ::= Label ":" InlineArgument? Argument*

Argument ::= TextLiteral | Group | Formulation
```

This is a behavioral summary, not a lexer grammar. In particular, `Argument ::= Group` is selected by the implementation rule "heading line or structural section colon".
The proto parser can produce a heading-only group, but the structural parser
cannot dispatch it because group kind is chosen by the first section label.

## Layer 2: Structural AST Construction

The structural parser identifies a group by its first section label, not by its heading.

Examples:

- a top-level group whose first section is `Defines:` is parsed as `DefinesGroup`
- a nested group inside `Enables:` whose first section is `capability:` is parsed as `CapabilityGroup`
- a nested group inside `Enables:` whose first section is `from:` is parsed as a cast-backed enables group
- a clause group whose first section is `if:` is parsed as `IfGroup`

The heading is then validated according to that group kind.

Important implementation rules:

- section order is strict
- optional sections may be skipped
- present sections may not be reordered
- unexpected sections are rejected with diagnostics
- several singular section parsers keep only the first matching value and ignore extra valid values

## Structural AST Conventions

### Repeated-value wrappers

The structural AST uses:

- `OneOrMore<T>` for non-empty repeated contents
- `ZeroOrMore<T>` for possibly-empty repeated contents

`OneOrMore<T>` is a real non-empty wrapper in the current Rust code.

### Text wrappers

Quoted text sections are converted to typed wrappers:

- `OpenText`
- `WrittenText`
- `CalledText`
- `WritingText`

All of them use the same surface syntax:

- the source must be quoted with `"..."`
- the structural parser strips the outer quotes
- no escape processing is performed

### Clause values

A `Clause` can be either:

- a declaration statement parsed by `parse_ordinary_declaration_statement`, stored as `Clause::Declaration`
- a formulation expression parsed by `parse_expression`, stored as `Clause::Expression`
- or a nested clause group such as `exists`, `if`, `piecewise`, and so on

Formulation clause entries are tried in that order: declaration statement first, then expression.

## Top-Level Groups

The document AST is:

```text
Document ::= TopLevelItem*
```

An empty document is supported by the current implementation because `Document.items` is `ZeroOrMore<TopLevelItem>`.

### Top-level group inventory

- **`Title`** — `TitleGroup`, heading: none. Sections: `Title: OpenText`
- **`SectionTitle`** — `SectionTitleGroup`, heading: none. Sections: `SectionTitle: OpenText`
- **`SubsectionTitle`** — `SubsectionTitleGroup`, heading: none. Sections: `SubsectionTitle: OpenText`
- **`Text`** — `TextGroup`, heading: none. Sections: `Text: OpenText`
- **`Writing`** — `TopLevelWritingGroup`, heading: none. Sections: `Writing: WritingAlias+` (each alias is a double-quoted string of the form `"name :~> body"`; the LHS must be a `Name` and the body is raw LaTeX)
- **`Disambiguates`** — `DisambiguatesGroup`, heading: operator/function form. Sections: `Disambiguates:`, zero or more ordered `when: Clause+`/`to: Expression` branches, `else?: Expression`, `Documented?`, `Justification?`, `Aliases?`, `Writing?: "WritingAlias"+`, `References?`, `Metadata?`
- **`Defines`** — `DefinesGroup`, heading: command. Sections: `Defines: DefinesTarget [via FormOrDeclaration]`, `using?: DeclarationStatement+`, `when?: Clause+`, `extends?: ExtendsItem+`, `means?: IsOrViaItem+`, `satisfies?: Clause+`, `Requires?: RequiresItem+`, `Enables?: EnablesItem+`, `Documented?: DocumentedItem+`, `Justification?: HaveGroup+`, `Aliases?: AliasItem+`, `Writing?: "WritingAlias"+`, `References?: ResourceHeader+`, `Metadata?: MetadataItem+`. `DefinesTarget` is tried as a `FormOrDeclaration` first, then as a `DeclarationStatement` whose `is` relation may name a refined command, allowing typed or value-bearing targets such as `X := value is \set`. That relation is what the definition extends (see `language.md`), and an `is` relation may be followed by `via <FormOrDeclaration>`, stored on the section rather than the target. An `ExtendsItem` is the same `DeclarationStatement [via FormOrDeclaration]` pair; the `extends?:` section spells out the same clauses and exists to allow more than one, so a target that states a relation and an `extends?:` section are mutually exclusive. `extends_clauses` in `structural::ast` normalizes the two spellings into one borrowed list that every consumer works from
- **`Declares`** — `DeclaresGroup`, heading: command. Sections: `Declares: DeclarationStatement`, `abstractly?` (marker, no arguments), `using?: DeclarationStatement+`, `when?: Clause+`, `means?: IsOrViaItem+`, `expresses?: Clause`, `Requires?: RequiresItem+`, `Enables?: EnablesItem+`, `Documented?: DocumentedItem+`, `Justification?: HaveGroup+`, `Aliases?: AliasItem+`, `Writing?: "WritingAlias"+`, `References?: ResourceHeader+`, `Metadata?: MetadataItem+`. `abstractly:` is stored as `DeclaresGroup::abstractly`; it marks the `means:` items that state a specification without a value as abstract, for a `RealizesGroup` to supply
- **`Realizes`** — `RealizesGroup`, heading: command. Sections: `Realizes: DeclarationStatement`, `using?: DeclarationStatement+`, `when?: Clause+`, `means?: IsOrViaItem+`, `expresses?: Clause`, and the same support sections as `Declares`. The target names the realized declaration with `:=` (`Realizes: Nb := \naturals`), which must be a `Declares` marked `abstractly:`; `means:` must supply every symbol that declaration left abstract
- **`Refines`** — `RefinesGroup`, heading: refined command or refined spec-infix command (for example, `[A \:(nonempty)::subset:/ B]`). Sections: `Refines: RefinedDeclarationStatement`, `implicitly?` (marker, no arguments), `explicitly?` (marker, no arguments), `using?: DeclarationStatement+`, `when?: Clause+`, `extends?: RefinedDeclarationStatement`, `satisfies?: Clause+`, `Requires?: RequiresItem+`, `Enables?: EnablesItem+`, `Documented?: DocumentedItem+`, `Justification?: HaveGroup+`, `Aliases?: AliasItem+`, `Writing?: "WritingAlias"+`, `References?: ResourceHeader+`, `Metadata?: MetadataItem+`. `implicitly:`/`explicitly:` are optional, mutually exclusive, zero-argument marker sections stored as an `Option<RefinementKind>` (`Implicit`/`Explicit`); see the validation rules under `Refines` refinement markers in `language.md`
- **`States`** — `StatesGroup`, heading: command. Sections: `States:` (marker, no arguments), `using?: DeclarationStatement+`, `when?: Clause+`, `that: Clause+`, `Requires?: RequiresItem+`, `Enables?: EnablesItem+`, `Documented?: DocumentedItem+`, `Justification?: HaveGroup+`, `Aliases?: AliasItem+`, `Writing?: "WritingAlias"+`, `References?: ResourceHeader+`, `Metadata?: MetadataItem+`
- **`Axiom`** — `AxiomGroup`, heading: command?. Sections: `Axiom:`, `given?: RefinedDeclarationStatement+`, `where?: Clause+`, `then: Clause+`, `iff?: Clause+`, `Documented?: DocumentedItem+`, `Justification?: HaveGroup+`, `Aliases?: AliasItem+`, `Writing?: "WritingAlias"+`, `References?: ResourceHeader+`, `Metadata?: MetadataItem+`
- **`Theorem`** — `TheoremGroup`, heading: command?. Sections: `Theorem:`, `given?: RefinedDeclarationStatement+`, `where?: Clause+`, `then: Clause+`, `iff?: Clause+`, `Documented?: DocumentedItem+`, `Justification?: HaveGroup+`, `Aliases?: AliasItem+`, `Writing?: "WritingAlias"+`, `References?: ResourceHeader+`, `Metadata?: MetadataItem+`
- **`Conjecture`** — `ConjectureGroup`, heading: command?. Sections: `Conjecture:`, `given?: RefinedDeclarationStatement+`, `where?: Clause+`, `then: Clause+`, `iff?: Clause+`, `Documented?: DocumentedItem+`, `Justification?: HaveGroup+`, `Aliases?: AliasItem+`, `Writing?: "WritingAlias"+`, `References?: ResourceHeader+`, `Metadata?: MetadataItem+`. It records a theorem-shaped statement that is not claimed to have a proof
- **`Person`** — `PersonGroup`, heading: author. Sections: `Person: OpenText+`, `biography?: OpenText`
- **`Resource`** — `ResourceGroup`, heading: resource. Sections: `Resource: ResourceItem+`
- **`Specify`** — `SpecifyGroup`, heading: none. Sections: `Specify: SpecifyItem+`
- **`Relation`** — `RelationGroup`, heading: none. Sections: `Relation: OpenText*`, `using?: DeclarationStatement+`, `between: RelationSubject`, `and: RelationSubject`, `when?: Clause+`, `means?: RelationMeans`, `Documented?: DocumentedItem+`, `Justification?: HaveGroup+`, `Aliases?: AliasItem+`, `Writing?: "WritingAlias"+`, `References?: ResourceHeader+`, `Metadata?: MetadataItem+`
- **`Equivalent`** — `EquivalentGroup`, heading: command. Sections: `Equivalent:` (marker, no arguments), `using?: DeclarationStatement+`, `when?: Clause+`, `to: Expression+`, `Documented?: DocumentedItem+`, `Justification?: HaveGroup+`, `References?: ResourceHeader+`
- **`Topic`** — `TopicGroup`, heading: topic. Sections: `Topic: OpenText*`, `within?: OpenText`, `Related?: TopicRelatedItem+`, `Documented?: CalledDocumentedItem+`
- **`TextTheorem`** / **`TextAxiom`** / **`TextConjecture`** / **`TextDefinition`** — `TextItemGroup` (one `TextItemKind` per leading label), heading: none. Sections: `<label>: OpenText` (a markdown-with-LaTeX body), `Documented?: TextDocumentedItem+`, `References?: ResourceHeader+`, `Id: OpenText`. These are opaque prose placeholders for a structured theorem/axiom/conjecture/definition to be written later; the type-checker never inspects them. `TextDocumentedItem` is restricted to `called:`, `written:`, `description:`, and `notes:`.

Notes:

- `command?` means the heading is optional, but if present it must parse as a formulation command header
- `OpenText*` means the section itself is required but may contain zero text entries
- **Every** top-level group additionally accepts a final optional `Id?: OpenText`
  section (a quoted UUID). It is validated for placement and uniqueness and then
  discarded — no AST node stores it — so it is omitted from the per-group rows
  above rather than repeated on each. `Refines:` documentation is special-cased:
  its `Documented:` rejects `called:` and instead requires `adjective:`.

## Nested Group Categories

The following groups are used inside top-level sections.

### Alias items

Used inside `Aliases:`.

- **`alias`** — `AliasGroup`, heading: label?. Sections: `alias: AliasKind`, `written?: WrittenText+`

`AliasKind` is one of:

- `ExpressionAlias`
- `SpecOperatorAlias`

### Requires items

Used inside `Requires:`.

- **`capability`** — `CapabilityGroup`, heading: label?. Sections: `capability: AliasKind`, `written?: WrittenText+`
- **`definition`** — `DefinitionGroup`, heading: label?. Sections: `definition: DefinitionRequirement`

`DefinitionRequirement` is parsed from the formulation shape
`\command is <type-or-spec>`.

### Enables items

Used inside `Enables:`.

- **`capability`** — `CapabilityGroup`, heading: label?. Sections: `capability: AliasKind`, `written?: WrittenText+`
- **`from`** — `FromCapabilityGroup`, heading: label?. Sections: `from: DeclarationStatement`, `capability: AliasKind`, `written?: WrittenText+`
- **`from`** — `FromAsGroup`, heading: label?. Sections: `from: DeclarationStatement`, `as: ExpressionBinding`
- **`relation`** — `RelationGroup`, heading: label?. Sections: `relation: OpenText*`, `to: RelationshipDeclaration`, `when?: RelationWhenItem+`, `means?: Clause`, `represents?: RelationKind+`, `by?: OpenText+`

`from:` groups must contain exactly one of `capability:` or `as:`.
`RelationWhenItem` is either a declaration statement or a hard-cast statement.
`RelationKind` is one of `\\coercion` or `\\encoding`.

### Documented items

Used inside `Documented:`.

- **`written`** — `WrittenGroup`, heading: label?. Sections: `written: WrittenText+`
- **`called`** — `CalledGroup`, heading: label?. Sections: `called: CalledText+`, `written?: WrittenText+`
- **`adjective`** — `AdjectiveGroup`, heading: label?. Sections: `adjective: AdjectiveText+` (required by `Refines:`; `Refines:` `Documented:` rejects `called:`)
- **`description`** — `DescriptionGroup`, heading: label?. Sections: `description: OpenText`
- **`writing`** — `WritingGroup`, heading: label?. Sections: `writing: MappingWritingTarget`, `as: WritingText+`. It is valid only in a mapping-shaped `Defines:` item; the target must be either that exact placeholder mapping form or the same invocation with placeholders replaced by their ordinary names. For an aliased form such as `X ::= x(i_)`, these targets use the mapping name `x`
- **`overview`** — `OverviewGroup`, heading: label?. Sections: `overview: OpenText`
- **`related`** — `RelatedGroup`, heading: label?. Sections: `related: OpenText+`
- **`discoverer`** — `DiscovererGroup`, heading: label?. Sections: `discoverer: OpenText*`
- **`notes`** — `NotesGroup`, heading: label?. Sections: `notes: OpenText+` (prose reminders; used by the opaque `Text*` placeholders to record how to fill in a structured form later)

### Justification items

Used inside `Justification:` (which appears after `Documented:`). Each item is a
`HaveGroup` — a `have:`/`asserting:`/`because?:`/`by?:` group (see the `have`
clause group above) with a required `[label]` heading. Any grouped expression,
statement, or specification elsewhere in the group—including a nested
formulation or a `satisfies:` clause—may carry the matching `[:label:]`. It is
established using that entry's `have:`/`asserting:`; the entry's `have:` must
restate the labeled formulation, and every entry must be referenced by some
labeled formulation.

### Metadata items

Used inside `Metadata:`.

- **`id`** — `IdGroup`, heading: none. Sections: `id: OpenText`
- **`version`** — `VersionGroup`, heading: none. Sections: `version: OpenText`

### Specify items

Used inside top-level `Specify:`.

- **`decimal`** — `NumericSpecificationGroup`. Sections: `decimal:` (empty), `is: TypeExpression`
- **`zeroOrPositiveInt`** — `NumericSpecificationGroup`. Sections: `zeroOrPositiveInt:` (empty), `is: TypeExpression`
- **`positiveInt`** — `NumericSpecificationGroup`. Sections: `positiveInt:` (empty), `is: TypeExpression`
- **`int`** — `NumericSpecificationGroup`. Sections: `int:` (empty), `is: TypeExpression`

All four entries are optional, but a category may occur at most once across the
collection. Unlike the earlier prose-shaped specification groups, `is:` is
parsed as formulation type syntax and participates in semantic checking.

### Resource items

Used inside top-level `Resource:`.

`References:` entries name these resources with `$` headers. Both
`$book.chapter` and the quoted `"$book.chapter"` form are accepted. A PDF
resource reference may append `:page{n}`. Its `offset:` is the physical PDF page
corresponding to logical page 1, so the linked physical page is
`n + offset - 1`. The viewer displays a resolved reference as the resource's
title followed by its author entries in parentheses. A resource URL becomes the
link target; for `:page{n}` on a PDF URL, the calculated physical page is added
as the PDF fragment. `References:` is treated as supporting content and is
collapsed by default with `Documented:` and `Id:`.

- **`title`** — `ResourceTitleGroup`, heading: none. Sections: `title: OpenText`
- **`author`** — `ResourceAuthorGroup`, heading: none. Sections: `author: OpenText+`
- **`offset`** — `ResourceOffsetGroup`, heading: none. Sections: `offset: OpenText`
- **`url`** — `ResourceUrlGroup`, heading: none. Sections: `url: OpenText`
- **`homepage`** — `ResourceHomepageGroup`, heading: none. Sections: `homepage: OpenText`
- **`type`** — `ResourceTypeGroup`, heading: none. Sections: `type: OpenText`
- **`edition`** — `ResourceEditionGroup`, heading: none. Sections: `edition: OpenText`
- **`editor`** — `ResourceEditorGroup`, heading: none. Sections: `editor: OpenText`
- **`institution`** — `ResourceInstitutionGroup`, heading: none. Sections: `institution: OpenText`
- **`journal`** — `ResourceJournalGroup`, heading: none. Sections: `journal: OpenText`
- **`publisher`** — `ResourcePublisherGroup`, heading: none. Sections: `publisher: OpenText`
- **`volume`** — `ResourceVolumeGroup`, heading: none. Sections: `volume: OpenText`
- **`month`** — `ResourceMonthGroup`, heading: none. Sections: `month: OpenText`
- **`year`** — `ResourceYearGroup`, heading: none. Sections: `year: OpenText`
- **`description`** — `ResourceDescriptionGroup`, heading: none. Sections: `description: OpenText`

## Clause Groups

Clause groups are used anywhere a section expects `Clause` values.

If a clause section contains:

- a formulation argument, it is first tried as `parse_ordinary_declaration_statement`, then `parse_expression`
- a nested group, it is dispatched by its first section label

### Clause inventory

- **`not`** — `NotGroup`, heading: label?. Sections: `not: Clause`
- **`allOf`** — `AllOfGroup`, heading: label?. Sections: `allOf: Clause+`
- **`anyOf`** — `AnyOfGroup`, heading: label?. Sections: `anyOf: Clause+`
- **`oneOf`** — `OneOfGroup`, heading: label?. Sections: `oneOf: Clause+`
- **`exists`** — `ExistsGroup`, heading: label?. Sections: `exists: BindingOrSpec`, `suchThat?: Clause+`
- **`existsUnique`** — `ExistsUniqueGroup`, heading: label?. Sections: `existsUnique: BindingOrSpec`, `suchThat?: Clause+`
- **`forAll`** — `ForAllGroup`, heading: label?. Sections: `forAll: BindingOrSpec`, `where?: Clause+`, `then: Clause+`
- **`let`** — `LetGroup`, heading: label?. Sections: `let: BindingOrSpec`, `where?: Clause+`, `then: Clause+`
- **`if`** — `IfGroup`, heading: label?. Sections: `if: Clause+`, `then: Clause+`
- **`have`** — `IffGroup`, heading: label?. Sections: `have: Clause+`, `iff: Clause+`. A `have:` group whose second section is `asserting:` (rather than `iff:`) is instead a `HaveGroup` (`Clause::Have`): `have: Clause+`, `asserting: Clause+`, `because?: Clause+`, `by?: Expression+` — an escape hatch that asserts the `have:` item holds given the `asserting:` items (also accepted as a `means:` item). `because:`/`by:` are justification the checker only reference-validates, never proves.
- **`piecewise`** — `PiecewiseGroup`, heading: label?. Sections: `piecewise: OpenText*`, `if: Clause+`, `then: Clause+`, `else?: Clause+`
- **`given`** — `GivenGroup`, heading: label?. Sections: `given: RefinedDeclarationStatement`, `where?: Clause+`, `then: Clause+`
- **`equivalently`** — `EquivalentlyGroup`, heading: label?. Sections: `equivalently: Clause+`

`equivalently:` asserts that its sub-clauses are all mutually equivalent (sugar
for a chain of `iff`s); it is checked like `allOf:` and carries no additional
type meaning.

## Heading Kinds

Structural groups validate their raw proto headings with formulation helper parsers.

### Command headings

Required on:

- `Defines`
- `Declares`
- `Refines`
- `States`
- `Equivalent`

Optional on:

- `Axiom`
- `Theorem`
- `Conjecture`

These headings must parse with `parse_command_header`.

### Label headings

Optional on:

- alias items
- enables items
- documented items
- specify items
- clause groups

Required on `HaveGroup` items nested directly under `Justification:`; the same
group uses an optional label in clause or `means:` positions.

These headings must parse with `parse_label_header`.

### Author headings

Required on:

- `Person`

These headings must parse with `parse_author_header`.

### Resource headings

Required on:

- `Resource`

These headings must parse with `parse_resource_header`.

## Formulation Parsers Used by Structural Sections

The structural parser delegates section content to formulation parsers as follows:

- `FormOrDeclaration` — `parse_form_or_declaration`
- `DefinesTarget` — try `parse_form_or_declaration`, then
  `parse_ordinary_declaration_statement`
- `DeclarationStatement` — `parse_ordinary_declaration_statement`
- `RefinedDeclarationStatement` — `parse_refined_declaration_statement`
- `IsOrViaItem` — try `parse_is_via_statement`, then `parse_ordinary_declaration_statement`
- `BindingOrSpec` — `parse_refined_declaration_statement`
- `AliasKind` — try `parse_expression_alias`, then `parse_spec_operator_alias`
- `MappingWritingTarget` — accept a function declaration form or function-call expression; semantic checking restricts it to the enclosing `Defines:` mapping
- `WritingAlias` — `parse_writing_alias`, applied to the contents of each double-quoted `Writing:` argument (used by the collection-wide `Writing:` group)
- `ResourceHeader` — `parse_resource_header`
- `CommandHeader` — `parse_command_header`
- `AuthorHeader` — `parse_author_header`
- `LabelHeader` — `parse_label_header`

Clause formulation arguments are tried as `parse_ordinary_declaration_statement`, then `parse_expression`. This means helper-only forms such as comma-separated `is` subjects or quoted operators with spaces are represented as declaration statements, while expression-compatible facts can still be parsed as declaration statements when they appear in clause position.

Declaration statements and `parse_is_via_statement` accept comma-separated form lists on the left of `is`, including placeholder forms, for example `f(x_), y_ is \set`. `parse_is_via_statement` accepts any form/declaration after `via`, such as `X` or `(X, Y)`.

## Compact AST Schema

This section is intentionally dense. It is the parser-oriented reference for
the current Rust structural AST and naming.

Conventions used below:

- `[CommandHeader]` means the heading is required and must parse as a formulation command header.
- `[CommandHeader]?` means the heading is optional, but if present must parse as a formulation command header.
- `[LabelHeader]?` means an optional structural label heading. `HaveGroup`
  headings are optional in ordinary clause/`means:` positions but required
  when the group is an item of `Justification:`.
- `[AuthorHeader]` means a required author heading.
- `[ResourceHeader]` means a required resource heading.
- If no heading line is shown, the group must not have a heading.
- `Text<T>` means a quoted text literal that the structural parser strips into wrapper type `T`.

### Unions and wrappers

```union
IsOrViaItemUnion ::=
    | IsViaStatement
    | DeclarationStatement
    | HaveGroup
    | LabeledIsOrViaItem

LabeledIsOrViaItem ::= Grouped(IsOrViaItemUnion) Label
```

```union
BindingOrSpecUnion ::=
    | DeclarationStatement
```

```union
AliasKindUnion ::=
    | ExpressionAlias
    | SpecOperatorAlias
```

```union
DefinesTargetUnion ::=
    | FormOrDeclaration
    | DeclarationStatement
```

```union
AliasItemUnion ::=
    | AliasGroup
```

```union
RequiresItemUnion ::=
    | CapabilityGroup
    | DefinitionGroup
```

```union
EnablesItemUnion ::=
    | CapabilityGroup
    | FromCapabilityGroup
    | FromAsGroup
    | RelationGroup
```

```union
DocumentedItemUnion ::=
    | WrittenGroup
    | CalledGroup
    | AdjectiveGroup
    | WritingGroup
    | OverviewGroup
    | DescriptionGroup
    | RelatedGroup
    | DiscovererGroup
    | NotesGroup
```


```union
MetadataItemUnion ::=
    | IdGroup
    | VersionGroup
```

```union
SpecifyItemUnion ::=
    | NumericSpecificationGroup  # decimal, zeroOrPositiveInt, positiveInt, or int
```

```union
ResourceItemUnion ::=
    | ResourceTitleGroup
    | ResourceAuthorGroup
    | ResourceOffsetGroup
    | ResourceUrlGroup
    | ResourceHomepageGroup
    | ResourceTypeGroup
    | ResourceEditionGroup
    | ResourceEditorGroup
    | ResourceInstitutionGroup
    | ResourceJournalGroup
    | ResourcePublisherGroup
    | ResourceVolumeGroup
    | ResourceMonthGroup
    | ResourceYearGroup
    | ResourceDescriptionGroup
```

```union
ClauseUnion ::=
    | NotGroup
    | AllOfGroup
    | AnyOfGroup
    | OneOfGroup
    | ExistsGroup
    | ExistsUniqueGroup
    | ForAllGroup
    | LetGroup
    | IfGroup
    | IffGroup
    | PiecewiseGroup
    | GivenGroup
    | EquivalentlyGroup
    | HaveGroup
    | DeclarationStatement
    | Expression
```

```union
TopLevelItemUnion ::=
    | TitleGroup
    | SectionTitleGroup
    | SubsectionTitleGroup
    | TextGroup
    | TopLevelWritingGroup
    | DisambiguatesGroup
    | DefinesGroup
    | DeclaresGroup
    | RealizesGroup
    | RefinesGroup
    | StatesGroup
    | AxiomGroup
    | TheoremGroup
    | ConjectureGroup
    | PersonGroup
    | ResourceGroup
    | SpecifyGroup
    | RelationGroup
    | EquivalentGroup
    | TopicGroup
    | TextItemGroup
```

```union
OpenText ::= Text<OpenText>
```

```union
WrittenText ::= Text<WrittenText>
```

```union
CalledText ::= Text<CalledText>
```

```union
AdjectiveText ::= Text<AdjectiveText>
```

```union
WritingText ::= Text<WritingText>
```

```union
Root ::= TopLevelItemUnion*
```

### Top-level groups

```group
Title: <OpenText>
```

```group
SectionTitle: <OpenText>
```

```group
SubsectionTitle: <OpenText>
```

```group
Text: <OpenText>
```

```group
Writing: "<WritingAlias>"+
```

```group
[FunctionOrOperatorForm]
Disambiguates:
(when: <ClauseUnion>+
 to: <Expression>)*
else?: <Expression>
Documented?: <DocumentedItemUnion>+
Justification?: <HaveGroup>+
Aliases?: <AliasItemUnion>+
Writing?: "<WritingAlias>"+
References?: <ResourceHeader>+
Metadata?: <MetadataItemUnion>+
```

`Disambiguates:` requires at least one `when:`/`to:` branch or an `else:`.
Each `when:` belongs to the immediately following `to:`. Use `else:` for the
unconditional fallback.

```group
[CommandHeader]
Defines: <DefinesTargetUnion> [via <FormOrDeclaration>]
using?: <DeclarationStatement>+
when?: <ClauseUnion>+
extends?: <DeclarationStatement [via <FormOrDeclaration>]>+
means?: <IsOrViaItemUnion>+
satisfies?: <ClauseUnion>+
Requires?: <RequiresItemUnion>+
Enables?: <EnablesItemUnion>+
Documented?: <DocumentedItemUnion>+
Justification?: <HaveGroup>+
Aliases?: <AliasItemUnion>+
Writing?: "<WritingAlias>"+
References?: <ResourceHeader>+
Metadata?: <MetadataItemUnion>+
```

```group
[CommandHeader]
Declares: <DeclarationStatement>
abstractly?:
using?: <DeclarationStatement>+
when?: <ClauseUnion>+
means?: <IsOrViaItemUnion>+
expresses?: <ClauseUnion>
Requires?: <RequiresItemUnion>+
Enables?: <EnablesItemUnion>+
Documented?: <DocumentedItemUnion>+
Justification?: <HaveGroup>+
Aliases?: <AliasItemUnion>+
Writing?: "<WritingAlias>"+
References?: <ResourceHeader>+
Metadata?: <MetadataItemUnion>+
```

```group
[CommandHeader]
Realizes: <DeclarationStatement>
using?: <DeclarationStatement>+
when?: <ClauseUnion>+
means?: <IsOrViaItemUnion>+
expresses?: <ClauseUnion>
Requires?: <RequiresItemUnion>+
Enables?: <EnablesItemUnion>+
Documented?: <DocumentedItemUnion>+
Justification?: <HaveGroup>+
Aliases?: <AliasItemUnion>+
Writing?: "<WritingAlias>"+
References?: <ResourceHeader>+
Metadata?: <MetadataItemUnion>+
```

```group
[CommandHeader]
Refines: <RefinedDeclarationStatement>
implicitly?:
explicitly?:
using?: <DeclarationStatement>+
when?: <ClauseUnion>+
extends?: <RefinedDeclarationStatement>
satisfies?: <ClauseUnion>+
Requires?: <RequiresItemUnion>+
Enables?: <EnablesItemUnion>+
Documented?: <DocumentedItemUnion>+
Justification?: <HaveGroup>+
Aliases?: <AliasItemUnion>+
Writing?: "<WritingAlias>"+
References?: <ResourceHeader>+
Metadata?: <MetadataItemUnion>+
```

```group
[CommandHeader]
States:
using?: <DeclarationStatement>+
when?: <ClauseUnion>+
that: <ClauseUnion>+
Requires?: <RequiresItemUnion>+
Enables?: <EnablesItemUnion>+
Documented?: <DocumentedItemUnion>+
Justification?: <HaveGroup>+
Aliases?: <AliasItemUnion>+
Writing?: "<WritingAlias>"+
References?: <ResourceHeader>+
Metadata?: <MetadataItemUnion>+
```

```group
[CommandHeader]?
Axiom:
given?: <RefinedDeclarationStatement>+
where?: <ClauseUnion>+
then: <ClauseUnion>+
iff?: <ClauseUnion>+
Documented?: <DocumentedItemUnion>+
Justification?: <HaveGroup>+
Aliases?: <AliasItemUnion>+
Writing?: "<WritingAlias>"+
References?: <ResourceHeader>+
Metadata?: <MetadataItemUnion>+
```

```group
[CommandHeader]?
Theorem:
given?: <RefinedDeclarationStatement>+
where?: <ClauseUnion>+
then: <ClauseUnion>+
iff?: <ClauseUnion>+
Documented?: <DocumentedItemUnion>+
Justification?: <HaveGroup>+
Aliases?: <AliasItemUnion>+
Writing?: "<WritingAlias>"+
References?: <ResourceHeader>+
Metadata?: <MetadataItemUnion>+
```

```group
[CommandHeader]?
Conjecture:
given?: <RefinedDeclarationStatement>+
where?: <ClauseUnion>+
then: <ClauseUnion>+
iff?: <ClauseUnion>+
Documented?: <DocumentedItemUnion>+
Justification?: <HaveGroup>+
Aliases?: <AliasItemUnion>+
Writing?: "<WritingAlias>"+
References?: <ResourceHeader>+
Metadata?: <MetadataItemUnion>+
```

The theorem-like head sections (`Axiom:`/`Theorem:`/`Conjecture:`)
take no argument. A result's name is given in `Documented:` `called:`, exactly as
for the definition items, and renders as the card's title.

```group
[AuthorHeader]
Person: <OpenText>+
biography?: <OpenText>
```

```group
[ResourceHeader]
Resource: <ResourceItemUnion>+
```

```group
Specify:
  decimal?:
    is: <TypeExpression>
  zeroOrPositiveInt?:
    is: <TypeExpression>
  positiveInt?:
    is: <TypeExpression>
  int?:
    is: <TypeExpression>
```

```group
Relation: <OpenText>*
using?: <DeclarationStatement>+
between: <RelationSubject>
and: <RelationSubject>
when?: <ClauseUnion>+
means?: <RelationMeans>
Documented?: <DocumentedItemUnion>+
Justification?: <HaveGroup>+
Aliases?: <AliasItemUnion>+
Writing?: "<WritingAlias>"+
References?: <ResourceHeader>+
Metadata?: <MetadataItemUnion>+
```

A top-level `Relation:` states a bidirectional relationship between the two
subjects named in `between:` and `and:` (for example that they are equivalent).
Each `RelationSubject` is either an unquoted declaration (such as `a is \real`)
for a concept, or a quoted-text reference — a `"#topic"` or a `"\signature"` (a
`\command` with its arguments removed, like `\function:on:to`) — so a `Relation:`
may relate concepts, topics, and definitions in any combination. Its `RelationMeans`
is likewise either an unquoted clause (a statement of what the relationship means)
or a quoted-text prose description. Contrast the directional `relation:` group
nested inside `Enables:` ([`EnablesRelationGroup`], which relates the described
concept *to* another and, with `represents: \\coercion`/`\\encoding`, registers a
cast rule): the top-level item is heading-less, standalone, and registers no cast — it
is checked like a theorem (any declaration subjects and a statement `means:` are
validated for declared symbols and valid command references; quoted-text
references and a prose `means:` are recorded, not proven).

```group
[CommandHeader]
Equivalent:
using?: <DeclarationStatement>+
when?: <ClauseUnion>+
to: <Expression>+
Documented?: <DocumentedItemUnion>+
Justification?: <HaveGroup>+
References?: <ResourceHeader>+
```

A top-level `Equivalent:` declares that the `\command`s listed under `to:` are
interchangeable under the shared name given by its `[...]` heading. Each `to:`
command must use the header parameters directly, as bare names (no compound
expressions, no `using:` symbols). The item registers its heading as a command
signature and is validated locally:

- every `to:` member must be defined and be a `Defines`, `Declares`, `States`,
  or `Refines` — and all members must be the same one of those kinds;
- the members must declare the same target shape, and (by kind) the same `is`
  type (`Declares`), the same extended types (`Defines`), or the same base type
  (`Refines`);
- the members must provide the same set of capabilities (by name and arity); and
- this item's own `when:` must guarantee each member's requirements.

The members are then mutually substitutable to the type checker: a value known to
be one member (or the class-naming header) satisfies a requirement that it be any
other member, as long as the target member's header parameters are all pinned to
matching actuals by the known member. A capability or spec operator the class
provides likewise resolves on a value typed as any member or the header.

```group
[TopicHeader]
Topic: <OpenText>*
within?: <OpenText>
Related?: <TopicRelatedItem>+
Documented?: <CalledDocumentedItem>+
```

```group
to: <OpenText>+
means: <OpenText>
```

A top-level `Topic:` names a documentation topic. Its required heading is a
`TopicHeader` — a `#` sigil followed by a dotted name path (for example
`[#real.analysis]`) — which renders as a human title by title-casing the path
("Real Analysis") unless the `Documented:called:` text overrides it. `Topic:`
carries optional descriptive prose and `within?:` names a parent topic to make
this a sub-topic. `Related?:` records how the topic relates to others: each
`TopicRelatedItem` pairs one or more `to:` targets with a `means:` description.

References (`within:` and each `to:`) are **quoted text** so a reference is never
mistaken for a usage. A `"#topic"` value is a topic reference; a `"\signature"`
value is a **signature** — a `\command` with its arguments removed, e.g.
`\function:on{A}:to{B}` written as `\function:on:to` — that names a
`Defines`/`Declares`/`Refines`/`States`/theorem-like definition itself, not a use
of it. With `called:` (already quoted), the four quoted-text fields are `within:`,
`to:`, `means:`, and `called:`.

`Documented?:` accepts only `called:` (a `CalledDocumentedItem`); other
documentation fields are rejected. The item is stated, not checked: topic and
signature references need not resolve, and it registers no command signatures or
type facts.

```group
TextTheorem | TextAxiom | TextConjecture | TextDefinition: <OpenText>
Documented?: <TextDocumentedItem>+
References?: <ResourceHeader>+
Id: <OpenText>
```

`TextDocumentedItem` is restricted to `called:`, `written:`, `description:`,
and `notes:`.

### Nested item groups

```group
[LabelHeader]?
alias: <AliasKindUnion>
written?: <WrittenText>+
```

```group
[LabelHeader]?
capability: <AliasKindUnion>
written?: <WrittenText>+
```

```group
[LabelHeader]?
definition: <DefinitionRequirement>
```

```group
[LabelHeader]?
from: <DeclarationStatement>
capability: <AliasKindUnion>
written?: <WrittenText>+
```

```group
[LabelHeader]?
from: <DeclarationStatement>
as: <ExpressionBinding>
```

```group
[LabelHeader]?
relation: <OpenText>*
to: <RelationshipDeclaration>
when?: <RelationWhenItem>+
means?: <ClauseUnion>
represents?: <RelationKind>+
by?: <OpenText>+
```

```group
[LabelHeader]?
written: <WrittenText>+
```

```group
[LabelHeader]?
called: <CalledText>+
written?: <WrittenText>+
```

```group
[LabelHeader]?
writing: <MappingWritingTarget>
as: <WritingText>+
```

`MappingWritingTarget` is restricted to the exact mapping form from the
enclosing `Defines:` item (for example `x(i_)`) or that form with each
placeholder replaced by its ordinary name (for example `x(i)`). A declaration
alias (`X` in `X ::= x(i_)`) does not replace the mapping name in these forms.

```group
[LabelHeader]?
overview: <OpenText>
```

```group
[LabelHeader]?
adjective: <AdjectiveText>+
```

```group
[LabelHeader]?
description: <OpenText>
```

```group
[LabelHeader]?
related: <OpenText>+
```

```group
[LabelHeader]?
discoverer: <OpenText>*
```

```group
[LabelHeader]?
notes: <OpenText>+
```

```group
[LabelHeader]?
have: <ClauseUnion>+
asserting: <ClauseUnion>+
because?: <ClauseUnion>+
by?: <Expression>+
```

```group
id: <OpenText>
```

```group
version: <OpenText>
```

```group
[LabelHeader]?
positive: <OpenText>*
int: <OpenText>*
is: <OpenText>*
```

```group
[LabelHeader]?
negative: <OpenText>*
int: <OpenText>*
is: <OpenText>*
```

```group
[LabelHeader]?
zero: <OpenText>*
is: <OpenText>*
```

```group
[LabelHeader]?
positive: <OpenText>*
decimal: <OpenText>*
is: <OpenText>*
```

```group
[LabelHeader]?
negative: <OpenText>*
decimal: <OpenText>*
is: <OpenText>*
```

### Resource item groups

```group
title: <OpenText>
```

```group
author: <OpenText>+
```

```group
offset: <OpenText>
```

```group
url: <OpenText>
```

```group
homepage: <OpenText>
```

```group
type: <OpenText>
```

```group
edition: <OpenText>
```

```group
editor: <OpenText>
```

```group
institution: <OpenText>
```

```group
journal: <OpenText>
```

```group
publisher: <OpenText>
```

```group
volume: <OpenText>
```

```group
month: <OpenText>
```

```group
year: <OpenText>
```

```group
description: <OpenText>
```

### Clause groups

```group
[LabelHeader]?
not: <ClauseUnion>
```

```group
[LabelHeader]?
allOf: <ClauseUnion>+
```

```group
[LabelHeader]?
anyOf: <ClauseUnion>+
```

```group
[LabelHeader]?
oneOf: <ClauseUnion>+
```

```group
[LabelHeader]?
exists: <BindingOrSpecUnion>
suchThat?: <ClauseUnion>+
```

```group
[LabelHeader]?
existsUnique: <BindingOrSpecUnion>
suchThat?: <ClauseUnion>+
```

```group
[LabelHeader]?
forAll: <BindingOrSpecUnion>
where?: <ClauseUnion>+
then: <ClauseUnion>+
```

```group
[LabelHeader]?
let: <BindingOrSpecUnion>
where?: <ClauseUnion>+
then: <ClauseUnion>+
```

```group
[LabelHeader]?
if: <ClauseUnion>+
then: <ClauseUnion>+
```

```group
[LabelHeader]?
have: <ClauseUnion>+
iff: <ClauseUnion>+
```

```group
[LabelHeader]?
have: <ClauseUnion>+
asserting: <ClauseUnion>+
because?: <ClauseUnion>+
by?: <Expression>+
```

```group
[LabelHeader]?
piecewise: <OpenText>*
if: <ClauseUnion>+
then: <ClauseUnion>+
else?: <ClauseUnion>+
```

```group
[LabelHeader]?
given: <RefinedDeclarationStatement>
where?: <ClauseUnion>+
then: <ClauseUnion>+
```

```group
[LabelHeader]?
equivalently: <ClauseUnion>+
```

## Current Implementation Notes and Footguns

### Top-level and nested group kind is chosen by first section label

The heading does not determine the group type.
Groups without sections cannot be recognized by the structural parser.

### Section order is strict

For each group kind, sections must appear in the declared order shown in this document. Optional sections may be omitted, but present sections may not move.

### Exact capitalization matters

Examples:

- `using:` is lowercase
- `Requires:` is capitalized
- `Enables:` is capitalized
- `Metadata:` is capitalized
- `that:` is lowercase

### Singular structural sections keep only the first parsed value

The helper functions:

- `parse_required_formulation`
- `parse_required_clause`
- `parse_required_open_text`

all collect matching entries and return only the first one.

That means extra valid entries in a singular section are currently ignored rather than diagnosed.

Examples of affected sections:

- `Defines:`
- `Declares:`
- `means:`
- `expresses:`
- `overview:`
- all singular resource item sections

### Text parsing is very literal

- only fully quoted text is accepted for text sections
- the outermost quotes are simply stripped
- no escape sequences are interpreted

So the stored `OpenText` for `"abc"` is `abc`, but `\"` is not specially handled.

### Section-shaped colons start nested groups

This behavior comes from the proto parser. A non-text argument line starts a
nested group if it is a heading or if its first colon follows a
section-label-shaped prefix. Formulation delimiters `::=`, `:=`, `:?`, `:->`,
`:=>`, and `:~>` are excluded from this structural-colon rule.

### Clause formulation parsing has a fallback order

A clause line like:

```text
. x is \type{A}
```

is parsed as a declaration statement and wrapped as `Clause::Declaration`,
because declaration statements accept `is` facts, comma-separated subjects, and
quoted operators. If a line is not a declaration statement, it is then tried as
a formulation expression.

### Empty-but-required sections are real

Sections like:

- `States:`
- `Axiom:`
- `piecewise:`

are required as sections even though their contents are `OpenText*` and may therefore be empty.

### Reserved-word field names gain trailing underscores in Rust

The AST surface labels remain:

- `where`
- `if`
- `else`
- `as`
- `type`
- `is`

But the Rust struct fields use names like:

- `where_`
- `if_`
- `else_`
- `as_`
- `type_`
- `is_`

### Empty documents are supported

`parse_document` can return an empty structural document and constructs `Document.items` as `ZeroOrMore<TopLevelItem>`.

### Heading-only groups are not a supported structural form

The proto parser can produce a group that has a heading but no sections.

The structural parser does not have a valid top-level dispatch path for such a group, so malformed inputs of that shape are skipped during structural dispatch and can therefore contribute to an empty document.

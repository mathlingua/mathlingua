# Structural Syntax

This file describes the structural language exactly as it is currently implemented in the Rust code.
For a more readable overview of how structural syntax fits into the whole
language, start with [language.md](language.md).

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

- a top-level group whose first section is `Describes:` is parsed as `DescribesGroup`
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

| First section label | AST node | Heading | Ordered sections |
| --- | --- | --- | --- |
| `Title` | `TitleGroup` | none | `Title: OpenText` |
| `SectionTitle` | `SectionTitleGroup` | none | `SectionTitle: OpenText` |
| `SubsectionTitle` | `SubsectionTitleGroup` | none | `SubsectionTitle: OpenText` |
| `Describes` | `DescribesGroup` | command | `Describes: FormOrDeclaration`, `using?: DeclarationStatement+`, `when?: Clause+`, `extends?: IsOrViaItem`, `specifies?: IsOrViaItem+`, `satisfies?: Clause+`, `Requires?: RequiresItem+`, `Enables?: EnablesItem+`, `Justified?: JustifiedItem+`, `Documented?: DocumentedItem+`, `Aliases?: AliasItem+`, `References?: ResourceHeader+`, `Metadata?: MetadataItem+` |
| `Defines` | `DefinesGroup` | command | `Defines: DeclarationStatement`, `using?: DeclarationStatement+`, `when?: Clause+`, `expresses?: Clause`, `Requires?: RequiresItem+`, `Enables?: EnablesItem+`, `Justified?: JustifiedItem+`, `Documented?: DocumentedItem+`, `Aliases?: AliasItem+`, `References?: ResourceHeader+`, `Metadata?: MetadataItem+` |
| `Refines` | `RefinesGroup` | command | `Refines: DeclarationStatement`, `using?: DeclarationStatement+`, `when?: Clause+`, `specifies?: DeclarationStatement`, `satisfies?: Clause+`, `Requires?: RequiresItem+`, `Enables?: EnablesItem+`, `Justified?: JustifiedItem+`, `Documented?: DocumentedItem+`, `Aliases?: AliasItem+`, `References?: ResourceHeader+`, `Metadata?: MetadataItem+` |
| `States` | `StatesGroup` | command | `States: OpenText*`, `using?: DeclarationStatement+`, `when?: Clause+`, `that: Clause+`, `Requires?: RequiresItem+`, `Enables?: EnablesItem+`, `Justified?: JustifiedItem+`, `Documented?: DocumentedItem+`, `Aliases?: AliasItem+`, `References?: ResourceHeader+`, `Metadata?: MetadataItem+` |
| `Axiom` | `AxiomGroup` | command? | `Axiom: OpenText*`, `given?: RefinedDeclarationStatement+`, `where?: Clause+`, `then: Clause+`, `iff?: Clause+`, `Justified?: JustifiedItem+`, `Documented?: DocumentedItem+`, `Aliases?: AliasItem+`, `References?: ResourceHeader+`, `Metadata?: MetadataItem+` |
| `Theorem` | `TheoremGroup` | command? | `Theorem: OpenText*`, `given?: RefinedDeclarationStatement+`, `where?: Clause+`, `then: Clause+`, `iff?: Clause+`, `Justified?: JustifiedItem+`, `Documented?: DocumentedItem+`, `Aliases?: AliasItem+`, `References?: ResourceHeader+`, `Metadata?: MetadataItem+` |
| `Corollary` | `CorollaryGroup` | command? | `Corollary: OpenText*`, `of: OpenText*`, `given?: RefinedDeclarationStatement+`, `where?: Clause+`, `then: Clause+`, `iff?: Clause+`, `Justified?: JustifiedItem+`, `Documented?: DocumentedItem+`, `Aliases?: AliasItem+`, `References?: ResourceHeader+`, `Metadata?: MetadataItem+` |
| `Lemma` | `LemmaGroup` | command? | `Lemma: OpenText*`, `given?: RefinedDeclarationStatement+`, `where?: Clause+`, `then: Clause+`, `iff?: Clause+`, `Justified?: JustifiedItem+`, `Documented?: DocumentedItem+`, `Aliases?: AliasItem+`, `References?: ResourceHeader+`, `Metadata?: MetadataItem+` |
| `Conjecture` | `ConjectureGroup` | command? | `Conjecture: OpenText*`, `given?: RefinedDeclarationStatement+`, `where?: Clause+`, `then: Clause+`, `iff?: Clause+`, `Justified?: JustifiedItem+`, `Documented?: DocumentedItem+`, `Aliases?: AliasItem+`, `References?: ResourceHeader+`, `Metadata?: MetadataItem+` |
| `Person` | `PersonGroup` | author | `Person: OpenText+`, `biography?: OpenText` |
| `Resource` | `ResourceGroup` | resource | `Resource: ResourceItem+` |
| `Specify` | `SpecifyGroup` | none | `Specify: SpecifyItem+` |

Notes:

- `command?` means the heading is optional, but if present it must parse as a formulation command header
- `OpenText*` means the section itself is required but may contain zero text entries

## Nested Group Categories

The following groups are used inside top-level sections.

### Alias items

Used inside `Aliases:`.

| First section label | AST node | Heading | Ordered sections |
| --- | --- | --- | --- |
| `alias` | `AliasGroup` | label? | `alias: AliasKind`, `written?: WrittenText+` |

`AliasKind` is one of:

- `ExpressionAlias`
- `SpecOperatorAlias`

### Requires items

Used inside `Requires:`.

| First section label | AST node | Heading | Ordered sections |
| --- | --- | --- | --- |
| `capability` | `CapabilityGroup` | label? | `capability: AliasKind`, `written?: WrittenText+` |
| `definition` | `DefinitionGroup` | label? | `definition: DefinitionRequirement` |

`DefinitionRequirement` is parsed from the formulation shape
`\command is <type-or-spec>`.

### Enables items

Used inside `Enables:`.

| First section label | AST node | Heading | Ordered sections |
| --- | --- | --- | --- |
| `capability` | `CapabilityGroup` | label? | `capability: AliasKind`, `written?: WrittenText+` |
| `from` | `FromCapabilityGroup` | label? | `from: DeclarationStatement`, `capability: AliasKind`, `written?: WrittenText+` |
| `from` | `FromAsGroup` | label? | `from: DeclarationStatement`, `as: ExpressionBinding` |
| `relation` | `RelationGroup` | label? | `relation: OpenText*`, `to: RelationshipDeclaration`, `when?: RelationWhenItem+`, `means?: Clause`, `as?: RelationKind+`, `by?: OpenText+` |
| `connection` | `ConnectionGroup` | label? | `connection: OpenText*`, `to: OpenText*`, `using?: DeclarationStatement+`, `means: OpenText*`, `signifies?: OpenText*`, `viewable?: OpenText*`, `through?: OpenText*` |

`from:` groups must contain exactly one of `capability:` or `as:`.
`RelationWhenItem` is either a declaration statement or a hard-cast statement.
`RelationKind` is one of `\\view` or `\\abstraction`.

### Documented items

Used inside `Documented:`.

| First section label | AST node | Heading | Ordered sections |
| --- | --- | --- | --- |
| `written` | `WrittenGroup` | label? | `written: WrittenText+` |
| `called` | `CalledGroup` | label? | `called: CalledText+`, `written?: WrittenText+` |
| `writing` | `WritingGroup` | label? | `writing: WritingAlias`, `as: WritingText+` |
| `overview` | `OverviewGroup` | label? | `overview: OpenText` |
| `related` | `RelatedGroup` | label? | `related: OpenText+` |
| `discoverer` | `DiscovererGroup` | label? | `discoverer: OpenText*` |

### Justified items

Used inside `Justified:`.

| First section label | AST node | Heading | Ordered sections |
| --- | --- | --- | --- |
| `label` | `LabelGroup` | label? | `label: OpenText*`, `by: OpenText*`, `comment: OpenText` |
| `by` | `ByGroup` | label? | `by: OpenText*`, `comment: OpenText` |

### Metadata items

Used inside `Metadata:`.

| First section label | AST node | Heading | Ordered sections |
| --- | --- | --- | --- |
| `id` | `IdGroup` | none | `id: OpenText` |
| `version` | `VersionGroup` | none | `version: OpenText` |

### Specify items

Used inside top-level `Specify:`.

| First section label | AST node | Heading | Ordered sections |
| --- | --- | --- | --- |
| `positive` with an `int` section | `PositiveIntGroup` | label? | `positive: OpenText*`, `int: OpenText*`, `is: OpenText*` |
| `negative` with an `int` section | `NegativeIntGroup` | label? | `negative: OpenText*`, `int: OpenText*`, `is: OpenText*` |
| `zero` | `ZeroGroup` | label? | `zero: OpenText*`, `is: OpenText*` |
| `positive` without an `int` section | `PositiveDecimalGroup` | label? | `positive: OpenText*`, `decimal: OpenText*`, `is: OpenText*` |
| `negative` without an `int` section | `NegativeDecimalGroup` | label? | `negative: OpenText*`, `decimal: OpenText*`, `is: OpenText*` |

Important implementation detail:

- `positive` and `negative` groups are distinguished as `int` vs `decimal` by checking whether any section label is literally `int`

### Resource items

Used inside top-level `Resource:`.

| First section label | AST node | Heading | Ordered sections |
| --- | --- | --- | --- |
| `title` | `ResourceTitleGroup` | none | `title: OpenText` |
| `author` | `ResourceAuthorGroup` | none | `author: OpenText+` |
| `offset` | `ResourceOffsetGroup` | none | `offset: OpenText` |
| `url` | `ResourceUrlGroup` | none | `url: OpenText` |
| `homepage` | `ResourceHomepageGroup` | none | `homepage: OpenText` |
| `type` | `ResourceTypeGroup` | none | `type: OpenText` |
| `edition` | `ResourceEditionGroup` | none | `edition: OpenText` |
| `editor` | `ResourceEditorGroup` | none | `editor: OpenText` |
| `institution` | `ResourceInstitutionGroup` | none | `institution: OpenText` |
| `journal` | `ResourceJournalGroup` | none | `journal: OpenText` |
| `publisher` | `ResourcePublisherGroup` | none | `publisher: OpenText` |
| `volume` | `ResourceVolumeGroup` | none | `volume: OpenText` |
| `month` | `ResourceMonthGroup` | none | `month: OpenText` |
| `year` | `ResourceYearGroup` | none | `year: OpenText` |
| `description` | `ResourceDescriptionGroup` | none | `description: OpenText` |

## Clause Groups

Clause groups are used anywhere a section expects `Clause` values.

If a clause section contains:

- a formulation argument, it is first tried as `parse_ordinary_declaration_statement`, then `parse_expression`
- a nested group, it is dispatched by its first section label

### Clause inventory

| First section label | AST node | Heading | Ordered sections |
| --- | --- | --- | --- |
| `not` | `NotGroup` | label? | `not: Clause` |
| `allOf` | `AllOfGroup` | label? | `allOf: Clause+` |
| `anyOf` | `AnyOfGroup` | label? | `anyOf: Clause+` |
| `oneOf` | `OneOfGroup` | label? | `oneOf: Clause+` |
| `exists` | `ExistsGroup` | label? | `exists: BindingOrSpec`, `suchThat?: Clause+` |
| `existsUnique` | `ExistsUniqueGroup` | label? | `existsUnique: BindingOrSpec`, `suchThat?: Clause+` |
| `forAll` | `ForAllGroup` | label? | `forAll: BindingOrSpec`, `where?: Clause+`, `then: Clause+` |
| `if` | `IfGroup` | label? | `if: Clause+`, `then: Clause+` |
| `have` | `IffGroup` | label? | `have: Clause+`, `iff: Clause+` |
| `piecewise` | `PiecewiseGroup` | label? | `piecewise: OpenText*`, `if: Clause+`, `then: Clause+`, `else?: Clause+` |
| `given` | `GivenGroup` | label? | `given: RefinedDeclarationStatement`, `where?: Clause+`, `then: Clause+` |

## Heading Kinds

Structural groups validate their raw proto headings with formulation helper parsers.

### Command headings

Required on:

- `Describes`
- `Defines`
- `Refines`
- `States`

Optional on:

- `Axiom`
- `Theorem`
- `Corollary`
- `Lemma`
- `Conjecture`

These headings must parse with `parse_command_header`.

### Label headings

Optional on:

- alias items
- enables items
- documented items
- justified items
- specify items
- clause groups

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

| Structural content kind | Parser used |
| --- | --- |
| `FormOrDeclaration` | `parse_form_or_declaration` |
| `DeclarationStatement` | `parse_ordinary_declaration_statement` |
| `RefinedDeclarationStatement` | `parse_refined_declaration_statement` |
| `IsOrViaItem` | try `parse_is_via_statement`, then `parse_ordinary_declaration_statement` |
| `BindingOrSpec` | `parse_ordinary_declaration_statement` |
| `AliasKind` | try `parse_expression_alias`, then `parse_spec_operator_alias` |
| `WritingAlias` | `parse_writing_alias` |
| `ResourceHeader` | `parse_resource_header` |
| `CommandHeader` | `parse_command_header` |
| `AuthorHeader` | `parse_author_header` |
| `LabelHeader` | `parse_label_header` |

Clause formulation arguments are tried as `parse_ordinary_declaration_statement`, then `parse_expression`. This means helper-only forms such as comma-separated `is` subjects or quoted operators with spaces are represented as declaration statements, while expression-compatible facts can still be parsed as declaration statements when they appear in clause position.

Declaration statements and `parse_is_via_statement` accept comma-separated form lists on the left of `is`, including placeholder forms, for example `f(x_), y_ is \set`. `parse_is_via_statement` accepts any form/declaration after `via`, such as `X` or `(X, Y)`.

## Compact AST Schema

This section is intentionally dense. It is the closest thing in this file to the old `old_docs/structural_ast.md`, but updated to reflect the current Rust implementation and naming.

Conventions used below:

- `[CommandHeader]` means the heading is required and must parse as a formulation command header.
- `[CommandHeader]?` means the heading is optional, but if present must parse as a formulation command header.
- `[LabelHeader]?` means an optional structural label heading.
- `[AuthorHeader]` means a required author heading.
- `[ResourceHeader]` means a required resource heading.
- If no heading line is shown, the group must not have a heading.
- `Text<T>` means a quoted text literal that the structural parser strips into wrapper type `T`.

### Unions and wrappers

```union
IsOrViaItemUnion ::=
    | IsViaStatement
    | DeclarationStatement
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
    | ViewableGroup
    | ConnectionGroup
```

```union
DocumentedItemUnion ::=
    | WrittenGroup
    | CalledGroup
    | WritingGroup
    | OverviewGroup
    | RelatedGroup
    | DiscovererGroup
```

```union
JustifiedItemUnion ::=
    | LabelGroup
    | ByGroup
```

```union
MetadataItemUnion ::=
    | IdGroup
    | VersionGroup
```

```union
SpecifyItemUnion ::=
    | PositiveIntGroup
    | NegativeIntGroup
    | ZeroGroup
    | PositiveDecimalGroup
    | NegativeDecimalGroup
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
    | IfGroup
    | IffGroup
    | PiecewiseGroup
    | GivenGroup
    | DeclarationStatement
    | Expression
```

```union
TopLevelItemUnion ::=
    | TitleGroup
    | SectionTitleGroup
    | SubsectionTitleGroup
    | DescribesGroup
    | DefinesGroup
    | RefinesGroup
    | StatesGroup
    | AxiomGroup
    | TheoremGroup
    | CorollaryGroup
    | LemmaGroup
    | ConjectureGroup
    | PersonGroup
    | ResourceGroup
    | SpecifyGroup
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
[CommandHeader]
Describes: <FormOrDeclaration>
using?: <DeclarationStatement>+
when?: <ClauseUnion>+
extends?: <IsOrViaItemUnion>
specifies?: <IsOrViaItemUnion>+
satisfies?: <ClauseUnion>+
Requires?: <RequiresItemUnion>+
Enables?: <EnablesItemUnion>+
Justified?: <JustifiedItemUnion>+
Documented?: <DocumentedItemUnion>+
Aliases?: <AliasItemUnion>+
References?: <ResourceHeader>+
Metadata?: <MetadataItemUnion>+
```

```group
[CommandHeader]
Defines: <DeclarationStatement>
using?: <DeclarationStatement>+
when?: <ClauseUnion>+
expresses?: <ClauseUnion>
Requires?: <RequiresItemUnion>+
Enables?: <EnablesItemUnion>+
Justified?: <JustifiedItemUnion>+
Documented?: <DocumentedItemUnion>+
Aliases?: <AliasItemUnion>+
References?: <ResourceHeader>+
Metadata?: <MetadataItemUnion>+
```

```group
[CommandHeader]
Refines: <RefinedDeclarationStatement>
using?: <DeclarationStatement>+
when?: <ClauseUnion>+
specifies?: <RefinedDeclarationStatement>
satisfies?: <ClauseUnion>+
Requires?: <RequiresItemUnion>+
Enables?: <EnablesItemUnion>+
Justified?: <JustifiedItemUnion>+
Documented?: <DocumentedItemUnion>+
Aliases?: <AliasItemUnion>+
References?: <ResourceHeader>+
Metadata?: <MetadataItemUnion>+
```

```group
[CommandHeader]
States: <OpenText>*
using?: <DeclarationStatement>+
when?: <ClauseUnion>+
that: <ClauseUnion>+
Requires?: <RequiresItemUnion>+
Enables?: <EnablesItemUnion>+
Justified?: <JustifiedItemUnion>+
Documented?: <DocumentedItemUnion>+
Aliases?: <AliasItemUnion>+
References?: <ResourceHeader>+
Metadata?: <MetadataItemUnion>+
```

```group
[CommandHeader]?
Axiom: <OpenText>*
given?: <RefinedDeclarationStatement>+
where?: <ClauseUnion>+
then: <ClauseUnion>+
iff?: <ClauseUnion>+
Justified?: <JustifiedItemUnion>+
Documented?: <DocumentedItemUnion>+
Aliases?: <AliasItemUnion>+
References?: <ResourceHeader>+
Metadata?: <MetadataItemUnion>+
```

```group
[CommandHeader]?
Theorem: <OpenText>*
given?: <RefinedDeclarationStatement>+
where?: <ClauseUnion>+
then: <ClauseUnion>+
iff?: <ClauseUnion>+
Justified?: <JustifiedItemUnion>+
Documented?: <DocumentedItemUnion>+
Aliases?: <AliasItemUnion>+
References?: <ResourceHeader>+
Metadata?: <MetadataItemUnion>+
```

```group
[CommandHeader]?
Corollary: <OpenText>*
of: <OpenText>*
given?: <RefinedDeclarationStatement>+
where?: <ClauseUnion>+
then: <ClauseUnion>+
iff?: <ClauseUnion>+
Justified?: <JustifiedItemUnion>+
Documented?: <DocumentedItemUnion>+
Aliases?: <AliasItemUnion>+
References?: <ResourceHeader>+
Metadata?: <MetadataItemUnion>+
```

```group
[CommandHeader]?
Lemma: <OpenText>*
given?: <RefinedDeclarationStatement>+
where?: <ClauseUnion>+
then: <ClauseUnion>+
iff?: <ClauseUnion>+
Justified?: <JustifiedItemUnion>+
Documented?: <DocumentedItemUnion>+
Aliases?: <AliasItemUnion>+
References?: <ResourceHeader>+
Metadata?: <MetadataItemUnion>+
```

```group
[CommandHeader]?
Conjecture: <OpenText>*
given?: <RefinedDeclarationStatement>+
where?: <ClauseUnion>+
then: <ClauseUnion>+
iff?: <ClauseUnion>+
Justified?: <JustifiedItemUnion>+
Documented?: <DocumentedItemUnion>+
Aliases?: <AliasItemUnion>+
References?: <ResourceHeader>+
Metadata?: <MetadataItemUnion>+
```

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
Specify: <SpecifyItemUnion>+
```

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
means?: <Clause>
as?: <RelationKind>+
by?: <OpenText>+
```

```group
[LabelHeader]?
connection: <OpenText>*
to: <OpenText>*
using?: <DeclarationStatement>+
means: <OpenText>*
signifies?: <OpenText>*
viewable?: <OpenText>*
through?: <OpenText>*
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
writing: <WritingAlias>
as: <WritingText>+
```

```group
[LabelHeader]?
overview: <OpenText>
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
label: <OpenText>*
by: <OpenText>*
comment: <OpenText>
```

```group
[LabelHeader]?
by: <OpenText>*
comment: <OpenText>
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

- `Describes:`
- `Defines:`
- `extends:`
- `expresses:`
- `overview:`
- `comment:`
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

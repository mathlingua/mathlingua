# Project Architecture

This document describes the high-level architecture and repository layout of the
current MathLingua implementation. It is intended for contributors who need to
understand where a behavior belongs before changing the parser, checker,
renderer, CLI, or web viewer.

For language syntax details, use these companion documents:

- [language.md](language.md) is the human-facing language guide.
- [structural_syntax.md](structural_syntax.md) is the parser-level structural
  syntax reference.
- [formulation_syntax.md](formulation_syntax.md) is the parser-level
  formulation syntax reference.

## System Overview

The repository contains one Rust crate named `mlg` plus an embedded Next.js
viewer under `web/`.

At a high level, the system does four jobs:

1. Parse `.mlg` files into typed Rust ASTs.
2. Check parsed files for semantic, reference, symbol, and requirement errors.
3. Render parsed collections into a JSON view model.
4. Serve that view model in a local web UI.

The main runtime paths are:

```text
mlg check
  CLI
  -> SourceCollection::load
  -> SourceCollection check passes
      -> structural parsing
      -> semantic checking
  -> event output

mlg view
  CLI
  -> SourceCollection::load
  -> SourceCollection check passes
      -> structural parsing
      -> semantic checking
  -> view-model generation pass
      -> render registry
      -> proto parser for display layout
  -> temporary collection.json
  -> Next.js viewer
```

The Rust code owns parsing, validation, semantic checking, and rendering
decisions. The web app receives a presentation-oriented JSON payload and avoids
depending on Rust AST internals.

## Repository Layout

```text
.
├── Cargo.toml
├── build.rs
├── docs/
├── goldens/
├── src/
├── testbed/
└── web/
```

Important roots:

- `Cargo.toml` defines the Rust crate, dependencies, and binary/library package.
- `build.rs` runs LALRPOP before compilation so the generated formulation parser
  exists when Rust code is compiled.
- `docs/` contains human and parser-level documentation.
- `goldens/` contains expected parser outputs used by parser tests.
- `src/` contains the Rust CLI, parsers, semantic checker, renderer, and event
  system.
- `testbed/` is a small example MathLingua collection.
- `web/` contains the embedded Next.js viewer launched by `mlg view`.

Generated/build artifacts such as `target/`, `web/.next/`, and
`web/node_modules/` are not architectural source. They may exist locally after
running builds or the viewer.

## Rust Crate Shape

The public library surface is declared in `src/lib.rs`.

```text
src/
├── main.rs
├── lib.rs
├── cli.rs
├── constants.rs
├── environment.rs
├── events/
├── frontend/
├── backend/
└── mlg/
```

Responsibilities:

- `src/main.rs` is the binary entrypoint. It parses CLI arguments, attaches a
  console event listener, runs the requested command, and maps errors to process
  exit codes.
- `src/lib.rs` exposes command helpers and internal modules for tests and
  embedding.
- `src/cli.rs` defines the `clap` command-line interface.
- `src/environment.rs` contains process environment helpers.
- `src/constants.rs` defines shared constants such as collection filenames.
- `src/events/` is the shared diagnostic and logging system.
- `src/frontend/` contains lexing and parsing.
- `src/backend/` contains collection loading, semantic checking, and viewer
  model generation.
- `src/mlg/` contains command orchestration for `check`, `init`, `version`, and
  `view`.

## Command Layer

The command layer is split between:

- `src/main.rs`, which handles binary concerns.
- `src/cli.rs`, which defines command-line syntax.
- `src/mlg/`, which implements command behavior.

The implemented top-level commands are:

- `mlg check [PATH...]`
- `mlg init`
- `mlg version`
- `mlg view [--port PORT]`

`src/mlg/mod.rs` re-exports the command entrypoints used by `src/main.rs` and
`src/lib.rs`.

### Collection Resolution

Collection and source-file resolution lives in `src/backend/collection.rs`.

Rules:

- A MathLingua collection root is the nearest ancestor containing `mlg.json`.
- If no `mlg.json` is found, the current working directory is treated as an
  ad-hoc collection root.
- `backend::collection::SourceCollection` owns collection-root discovery,
  config validation, source-file collection, and the shared check-pass sequence
  used by both `mlg check` and `mlg view`.
- `SourceCollection::load` accepts only a root path and collects every `.mlg`
  file under the resolved collection root in deterministic path order.
- Explicit `mlg check` paths are resolved relative to the current working
  directory and become a diagnostic filter. The full collection is still parsed
  and checked; only diagnostics located in selected files are shown.
- File targets must have the `.mlg` extension.
- Directory targets are traversed recursively.

### Config Handling

Collection config handling lives in `src/backend/config.rs`.

The current config file is `mlg.json`. The config model is intentionally small:

```json
{
  "name": "",
  "version": "0"
}
```

Validation requires `name` and `version` to exist and be strings. Extra fields
are accepted for forward compatibility.

`src/mlg/init.rs` creates missing `mlg.json` and `content/` entries, and leaves
existing entries untouched.

## Event and Diagnostic Architecture

All major layers report through `src/events/`.

The central type is `EventLog`:

- It is append-only.
- It stores events for later inspection by tests and command logic.
- It can notify listeners as events are emitted.
- The CLI attaches `EventConsoleWriter` as a listener.

Events carry:

- audience: user-facing or system-facing
- level: log, warning, error, or debug
- origin: the subsystem that emitted the event
- optional file/path/row/span location

This design keeps parsers, checkers, and command orchestration independent of
stdout/stderr. Code emits structured events; the command layer decides how those
events are displayed.

Markers are used for bounded event ranges. For example, `mlg check` records a
begin/end marker around one check run so callers can inspect the events emitted
during that run.

## Frontend Architecture

The frontend has three layers:

```text
raw source text
  -> proto parser
  -> structural parser
  -> typed structural Document

formulation snippets
  -> formulation lexer/parser/helper parsers
  -> formulation AST nodes
```

The module root is `src/frontend/mod.rs`.

The frontend root is the API used by the rest of the crate. It exposes
`parse_document` for in-memory source text, `parse_source_file` for filesystem
source files, `ParsedSourceFile`, and the structural/formulation AST types.
Backend code should import from `frontend::...` rather than reaching into
`frontend::structural` or `frontend::formulation` internals.

### Proto Layer

Location: `src/frontend/proto/`

The proto layer is indentation-sensitive and intentionally shallow. It parses
source text into broad groups, sections, text literals, nested groups, and raw
formulation strings without interpreting mathematical syntax.

Main files:

- `lexer.rs` normalizes source lines, indentation, `. ` argument markers, blank
  lines, and comments.
- `parser.rs` builds proto groups and sections from normalized lines.
- `ast.rs` defines proto `Group`, `Section`, `Argument`, `Formulation`, and
  `TextLiteral`.

The proto parser is responsible for source shape and recovery. It does not know
whether a group is a `Theorem`, `Describes`, `Resource`, or clause group. That
classification happens in the structural layer.

### Formulation Layer

Location: `src/frontend/formulation/`

The formulation layer parses mathematical syntax: names, forms, expressions,
commands, aliases, statement helpers, and command headers.

Main files:

- `token.rs` defines Logos tokens.
- `lexer.rs` wraps the Logos token stream for LALRPOP.
- `grammar.lalrpop` defines the generated expression/form parser.
- `parser.rs` re-exports hand-written parser entrypoints and helpers.
- `parser/entrypoints.rs` contains public parser functions.
- `parser/commands.rs`, `parser/statements.rs`, `parser/lists.rs`, and
  `parser/scan.rs` implement scanner-based helper parsers.
- `ast.rs` and `ast/` define formulation AST nodes.
- `span.rs` stores byte spans for parsed formulation nodes.

There is no single formulation root grammar. Different structural sections call
different parser entrypoints. Examples include:

- `parse_expression`
- `parse_declaration_statement`
- `parse_form_or_declaration`
- `parse_refined_declaration_statement`
- `parse_is_via_statement`
- `parse_command_header`
- `parse_expression_alias`
- `parse_spec_operator_alias`
- `parse_writing_alias`

The LALRPOP grammar handles lexer-driven expressions and forms. Several
statement-like and command-header forms are intentionally hand-written because
they need top-level delimiter scanning rather than ordinary token grammar.

### Structural Layer

Location: `src/frontend/structural/`

The structural layer turns proto groups into the typed MathLingua document AST.

Main files:

- `parser.rs` composes proto parsing with structural recognition.
- `parser/top_level/` parses top-level groups such as `Describes`, `Defines`,
  `Theorem`, `Resource`, and `Specify`.
- `parser/nested/` parses nested support groups such as documentation,
  metadata, resource items, and specification items.
- `parser/clauses.rs` parses logical clause groups.
- `parser/helpers/` contains shared section, text, heading, formulation, group,
  and clause helpers.
- `ast.rs` and `ast/` define the typed structural AST.

The key structural rule is that group kind is chosen by the first section label,
not by the bracket heading. The heading is then validated according to the group
kind.

Structural parsing delegates mathematical content to formulation parser
entrypoints. For example, a `Describes:` argument uses
`parse_form_or_declaration`, theorem `given:` uses
`parse_is_or_refined_statement_spec`, and clause formulations use the clause
fallback order documented in [structural_syntax.md](structural_syntax.md).

## Backend Architecture

The backend is split into collection pass orchestration, semantic checking, and
viewer model generation.

```text
src/backend/
├── collection.rs
├── config.rs
├── semantic/
│   ├── mod.rs
│   └── ...
└── view/
```

`collection.rs` defines `SourceCollection`, the shared checked-collection
entity that owns root resolution, source discovery, structural parsing, and
semantic checking. `mlg check` can add a `SourceFileFilter` so path-specific
runs still check the whole collection while reporting only diagnostics from the
requested files. The viewer can then run the collection's optional view-model
generation pass.

`config.rs` owns `mlg.json` constants, default contents, and validation used by
collection loading and initialization.

### Semantic Checker

Location: `src/backend/semantic/`

The semantic checker runs after structural parsing. It does not mutate the AST;
it walks parsed documents and emits events.

The public entrypoint is:

```rust
check_documents(files, event_log)
```

The checker has three broad passes:

1. Collect command definitions into a global signature registry.
2. Validate command-like references against that registry.
3. Validate symbol usage and command type requirements.

Important files:

- `check.rs` orchestrates the semantic passes.
- `types.rs` defines checker data structures such as `SignatureRegistry`,
  `DefinitionEntry`, `TypeFact`, and extension/spec rules.
- `shapes.rs` computes canonical command signatures and argument shapes.
- `validation.rs` validates references for existence and argument shape.
- `typecheck.rs` implements symbol scope, facts, substitutions, requirements,
  subtyping through `extends:`, and spec-operator reduction.
- `locator.rs` maps semantic diagnostics back to source locations.
- `walk/` traverses top-level groups, clauses, statements, expressions, forms,
  and support sections for reference validation.

The signature registry is global across all checked files. Duplicate command
signatures are rejected across `Describes`, `Defines`, `Refines`, `States`, and
named theorem-like groups.

The type checker is intentionally conservative. It checks command references,
argument shapes, declared symbols, known type/spec facts, command requirements,
and subtype/spec implications. It is not a proof checker for theorem
conclusions.

### View Builder and Renderer

Location: `src/backend/view/`

The view backend builds a serialized presentation model consumed by the web
viewer.

Main files:

- `model.rs` defines `CollectionView`, `FileView`, `GroupView`, `SectionView`,
  and `ArgumentView`.
- `builder.rs` receives checked `ParsedSourceFile` values, builds a render
  registry, reruns the proto parser for source layout, and creates the JSON
  view model.
- `render.rs` wires together rendering internals.
- `render/registry.rs` builds rendering lookup tables from parsed files.
- `render/signatures.rs` computes render signatures.
- `render/templates.rs` applies documented/written rendering templates.
- `render/commands.rs`, `render/expressions.rs`, `render/statements.rs`,
  `render/names.rs`, and `render/fallbacks.rs` render AST fragments to LaTeX.
- `render/escaping.rs` handles LaTeX escaping.

The view builder deliberately emits a presentation-oriented JSON model instead
of exposing frontend AST internals to TypeScript. This keeps the web viewer
stable when Rust AST internals change.

Structural parsing and semantic checking happen before the view builder through
the shared `SourceCollection` passes in `src/backend/collection.rs`. The
builder is therefore a rendering pass, not a second private checker.

## Web Viewer Architecture

Location: `web/`

The web viewer is a Next.js application launched by `mlg view`.

Source layout:

```text
web/
├── app/
├── components/
├── lib/
├── package.json
├── next.config.ts
└── tsconfig.json
```

Key files:

- `web/app/page.tsx` and `web/app/[...path]/page.tsx` route all viewer paths to
  the same viewer page.
- `web/app/viewer-page.tsx` loads the collection view and renders the shell.
- `web/lib/data.ts` reads the JSON payload from `MLG_VIEW_DATA_PATH`.
- `web/lib/types.ts` mirrors the Rust serialized view model.
- `web/lib/presenter.ts` contains route, label, and file-browser presentation
  helpers.
- `web/components/` renders the viewer shell, file list, group cards, section
  content, argument lists, and LaTeX.

`mlg view` writes a temporary `collection.json`, sets `MLG_VIEW_DATA_PATH`, and
starts `npm run dev` in `web/`. The web app reads only that JSON path. It does
not parse `.mlg` files and does not run semantic checks.

## Check Command Data Flow

`mlg check` is implemented by `src/mlg/check.rs`.

```text
main.rs
  -> Cli::parse
  -> mlg::check_in
  -> SourceCollection::load
      -> find collection root
      -> validate mlg.json when a collection root exists
      -> collect collection .mlg files
  -> SourceCollection::diagnostic_filter for explicit PATH arguments
  -> SourceCollection::run_check_passes_filtered
      -> frontend::parse_source_file for each .mlg file
      -> backend::semantic::check_documents
      -> replay diagnostics accepted by the filter
  -> EventLog summary
```

Parsing diagnostics are first collected in a file-local `EventLog`. They are
then copied into the command event log with the source file path attached. This
lets frontend parsers stay file-agnostic while CLI output still points to the
right file.

The command exits with a non-zero process code if any error-level event remains
in the command event log.

## View Command Data Flow

`mlg view` is implemented by `src/mlg/view.rs`.

```text
main.rs
  -> Cli::parse
  -> mlg::view_in
  -> SourceCollection::load
      -> find collection root
      -> validate mlg.json when a collection root exists
      -> collect collection .mlg files
  -> SourceCollection::run_check_passes
      -> frontend::parse_source_file for each .mlg file
      -> backend::semantic::check_documents
  -> SourceCollection::build_view
      -> backend::view::build_collection_view
          -> build render registry
          -> rerun proto parser for display layout
          -> create CollectionView
  -> write temporary collection.json
  -> ensure web dependencies
  -> npm run dev with MLG_VIEW_DATA_PATH
```

The viewer command treats parser and semantic errors as blocking, because the
rendered output would otherwise be misleading.

The temporary view data directory is removed after the Next.js process exits.

## Parser Generation

The expression/form grammar is generated by LALRPOP.

Source grammar:

```text
src/frontend/formulation/grammar.lalrpop
```

Build hook:

```text
build.rs
```

`build.rs` calls `lalrpop::process_root()`, so Cargo regenerates the parser
before compiling the crate.

Only part of the formulation language lives in the LALRPOP grammar. Command
headers, refined commands, aliases, and statement helper forms are implemented
by hand-written parsers in `src/frontend/formulation/parser/`.

## Testing Layout

Tests live close to the code they exercise.

Examples:

- `src/frontend/formulation/parser/tests.rs` and nested parser tests cover
  formulation parser behavior.
- `src/frontend/structural/parser/tests.rs` and nested tests cover structural
  parsing.
- `src/mlg/check/tests/` covers command-level checking behavior.
- `src/backend/view/render/tests.rs` covers rendering behavior.
- `src/backend/semantic/` behavior is covered through semantic and command
  tests.

Golden parser outputs are stored in:

```text
goldens/formulation/
goldens/structural/
```

The `testbed/` directory contains a small collection useful for manual command
testing and viewer smoke checks.

## Design Boundaries

These boundaries are intentional and should be preserved unless there is a
specific design change:

- The proto parser should remain shallow. It should classify source layout, not
  understand mathematical syntax.
- The structural parser should own group/section recognition and should call
  formulation parser entrypoints for mathematical content.
- The formulation parser should not depend on structural group types.
- The semantic checker should emit events rather than printing directly.
- The web viewer should consume `CollectionView` JSON and should not parse or
  semantically validate `.mlg` files.
- Rendering should use documented metadata and render registries, not ad hoc
  TypeScript parsing of MathLingua syntax.
- Command orchestration in `src/mlg/` should compose lower-level modules rather
  than duplicate parser or checker logic.

## Extension Points

Common changes generally belong in these places:

- Add or change language syntax:
  update `docs/language.md`, `docs/structural_syntax.md`, or
  `docs/formulation_syntax.md`; then update the relevant frontend parser and
  AST.
- Add a new structural group:
  update `src/frontend/structural/ast/`, structural parser dispatch, parser
  helpers/tests, semantic walkers if it can contain command references, renderer
  support if it should be visible, and docs.
- Add a new formulation construct:
  update `src/frontend/formulation/ast/`, `grammar.lalrpop` or hand-written
  parser helpers, semantic walkers/checkers, renderer support, tests, and docs.
- Add a new semantic rule:
  update `src/backend/semantic/`, especially the relevant walker, shape,
  validation, or typecheck code; then add command-level or semantic tests.
- Add new rendered output:
  update `src/backend/view/model.rs` and `builder.rs`; then mirror the JSON type
  in `web/lib/types.ts` and update web components.
- Add a CLI command:
  update `src/cli.rs`, `src/main.rs`, and add an orchestration module under
  `src/mlg/`.

## Current Architectural Constraints

The current implementation has several important constraints:

- Semantic checking is multi-pass and registry-based. New command-like syntax
  must be added to both reference walkers and type checking where appropriate.
- Source location reporting is best-effort and relies on the original source
  text plus parsed shapes.
- The viewer reruns the proto parser to preserve display layout after semantic
  checks have already used the structural AST.
- The TypeScript view model mirrors Rust serialization manually; schema changes
  must be kept in sync.
- The web viewer is served as a development Next.js server, not as a prebuilt
  static app.
- The language has syntax forms parsed by generated grammar and syntax forms
  parsed by scanner helpers. Changes must account for both paths.

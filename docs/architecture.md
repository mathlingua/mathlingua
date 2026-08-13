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

The repository contains one Rust crate named `mlg` plus a React viewer under
`web/`. The viewer's prebuilt static assets are embedded in the Rust binary.

At a high level, the system does five jobs:

1. Parse `.mlg` files into typed Rust ASTs.
2. Check parsed files for semantic, reference, symbol, and requirement errors.
3. Render parsed collections into a JSON view model.
4. Serve that view model in a local web UI or export it as a static site.
5. Provide editor diagnostics, context-aware completion, definition lookup, and
   rename over LSP.

The main runtime paths are:

```text
mlg check
  CLI
  -> format the collection (unless formatOnCheck is false)
  -> SourceCollection::load  (also generates any missing Id: sections)
  -> SourceCollection check passes
      -> structural parsing (proto -> structural -> formulation)
      -> mlg code-fence syntax check
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
  -> embedded HTTP server and static viewer assets

mlg export
  -> the same collection checks and view-model generation as `mlg view`
  -> embedded static viewer assets and route data

mlg lsp
  -> collection diagnostics and context-aware completion
  -> command definition lookup and workspace rename
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
├── testbed/     # a sample MathLingua collection ("Mathlore") used for testing
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
- `web/` contains the React/Vite viewer sources and the generated `web/dist/`
  assets embedded into `mlg`.

Generated/build artifacts such as `target/` and `web/node_modules/` are not
architectural source. `web/dist/` is an exception: it is checked in and shipped
in the crate so installed `mlg` binaries do not require Node.js or npm.

## Rust Crate Shape

The public library surface is declared in `src/lib.rs`.

```text
src/
├── main.rs
├── lib.rs
├── cli.rs
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
- `src/events/` is the shared diagnostic and logging system.
- `src/frontend/` contains lexing and parsing.
- `src/backend/` contains collection loading, semantic checking, and viewer
  model generation.
- `src/mlg/` contains command orchestration — `check`, `format`, `clean`,
  `export`, `init`, `lsp`, `release`, `version`, `view`, and the hidden
  diagnostic commands `debug`, `extract`, `report`, and `whte_rbt.obj` — plus
  completion and command-specific utilities (`util.rs`).

## Command Layer

The command layer is split between:

- `src/main.rs`, which handles binary concerns.
- `src/cli.rs`, which defines command-line syntax.
- `src/mlg/`, which implements command behavior.

The implemented top-level commands are:

- `mlg check [PATH...]`
- `mlg format`
- `mlg clean`
- `mlg export [--base-path PATH] [--cname DOMAIN] [--force]`
- `mlg init`
- `mlg release --summary TEXT [--dry-run] [--diff]`
- `mlg version`
- `mlg view [--port PORT]`

The following commands are hidden (`#[command(hide = true)]`) but implemented:

- `mlg lsp` — the editor language server over stdio.
- `mlg debug` — internal diagnostics.
- `mlg extract <ID>` — print the top-level item with the given `Id:`.
- `mlg report <ID...>` — report on items by `Id:`.
- `mlg whte_rbt.obj` — an easter egg.

The CLI also has global diagnostic filtering flags: `--event-audience`
(`--event-scope` is an alias), `--event-level`, and `--event-markers`.
`mlg check` supports machine-readable `--json` diagnostics and
`--diagnostic-schema` output.

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
- Sources are collected from `<root>/content/` when that directory exists,
  otherwise from the root itself.
- `SourceCollection::load` also **writes to disk**: it generates a UUID `Id:`
  section into any top-level item that lacks one (`ensure_source_file_ids`).
- Source order is a display-name sort (filename with `_` treated as a space and
  case ignored), not raw path order, and a directory's `toc` file overrides that
  order and can hide entries (a `HIDDEN` directive) or re-title them.
- A `_preface_.mlg` file is special: it is excluded from the page list and
  rendered as its directory's preface/overview.
- Explicit `mlg check` paths are resolved relative to the current working
  directory and become a diagnostic filter. The full collection is still parsed
  and checked; only diagnostics located in selected files are shown.
- File targets must have the `.mlg` extension; directory targets are traversed
  recursively.

### Config Handling

Collection config handling lives in `src/backend/config.rs`.

The current config file is `mlg.json`. The config model is intentionally small,
and every field is required so the whole configuration is visible in one place:

```json
{
  "name": "",
  "version": "0",
  "margin": 80,
  "formatOnCheck": true,
  "outputDir": "docs"
}
```

Keys are camelCase. `CONFIG_FIELDS` lists these five fields in the order
`mlg init` writes them. Validation requires every one to be present:

- `name` and `version` must be strings.
- `margin` — the target line width for `mlg format` — must be a positive
  integer.
- `formatOnCheck` — whether `mlg check` formats the collection before checking
  it — must be a boolean.
- `outputDir` — the directory `mlg export` builds into and `mlg clean` removes —
  must be a non-empty relative path that stays within the collection root (not
  absolute and no `..`), so those destructive commands cannot escape the
  collection.

There are no implicit defaults in a valid config: a missing field is a `mlg check`
error rather than a silent fallback. The accessors `Config::margin`,
`Config::format_on_check`, and `Config::output_dir` still fall back to
`DEFAULT_MARGIN` (80), `DEFAULT_FORMAT_ON_CHECK` (`true`), and `DEFAULT_OUTPUT_DIR`
(`docs`), but only so that other commands keep running on an already-flagged
partial config; `mlg check` is what enforces presence.
Extra fields are accepted for forward compatibility, with one exception:
`margin` was formerly named `print_margin`, and a config still carrying the old
key is rejected with a message naming the new one (which also stands in for the
"missing `margin`" error, so the two are not both reported).

`src/mlg/init.rs` creates a missing `mlg.json` with every field at its default,
and creates a missing `content/`. When `mlg.json` already exists but omits some
required fields, `mlg init` asks (on an interactive terminal) whether to fill
them in with their defaults, preserving existing values and any extra fields;
without a terminal it reports the gaps and leaves the file untouched. The
`config_object`, `missing_config_fields`, and `merge_default_fields` helpers in
`config.rs` back this, so init and validation agree on the field set.

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
whether a group is a `Theorem`, `Defines`, `Resource`, or clause group. That
classification happens in the structural layer.

### Formulation Layer

Location: `src/frontend/formulation/`

The formulation layer parses mathematical syntax: names, forms, expressions,
commands, aliases, statement helpers, and command headers.

Main files:

- `token.rs` defines the Logos tokens.
- `lexer.rs` wraps the Logos token stream for LALRPOP.
- `grammar.lalrpop` defines the generated expression/form parser.
- `parser.rs` contains the public formulation parser functions and the
  scanner-based hand-written helpers.
- `ast.rs` defines the formulation AST nodes (including byte spans).
- `mod.rs` re-exports the public API.

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

- `parser.rs` — a single module that composes proto parsing with structural
  recognition. It parses every top-level group (`Defines`, `Declares`,
  `Refines`, `States`, `Theorem`, `Axiom`, `Conjecture`, `Disambiguates`,
  `Relation`, `Equivalent`, `Topic`, `Resource`, `Person`, `Specify`, the prose
  and clause groups, etc.), their nested support groups (documentation,
  metadata, `Requires:`/`Enables:` items, resource items, justification items),
  clause groups, and the shared section/text/heading/formulation helpers.
- `ast.rs` defines the typed structural AST.
- `mod.rs` re-exports the public API.

There is no `parser/` or `ast/` subdirectory; the structural layer is the two
files `parser.rs` and `ast.rs`.

The key structural rule is that group kind is chosen by the first section label,
not by the bracket heading. The heading is then validated according to the group
kind.

Structural parsing delegates mathematical content to formulation parser
entrypoints. For example, a `Defines:` argument uses
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
├── release.rs
├── definition.rs   # go-to-definition (LSP)
├── rename.rs       # workspace rename (LSP)
├── extract.rs      # backs `mlg extract`/`mlg report`
├── text_fence.rs   # syntax-checks ```mlg fences in prose
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

`release.rs` builds the release dependency graph consumed by `mlg release`. For
each top-level item it records the item id, kind, exact source slice, and the ids
of the definitions that item uses. Item identity, kind, and source slicing reuse
the proto parser (the same layer the view uses); command-use resolution reuses the
semantic signature registry through `semantic::collect_definition_locations` and
`semantic::command_occurrences`, so a "use" edge is resolved exactly as
go-to-definition resolves a command occurrence.

### Semantic Checker

Location: `src/backend/semantic/`

The semantic checker runs after structural parsing. It does not mutate the AST;
it walks parsed documents and emits events.

The public entrypoint is:

```rust
check_documents(files, event_log)
```

`check_documents` first validates top-level item `Id:` sections (present, a
UUID, unique) and that at most one `Writing:` item exists, then runs three broad
registry passes:

1. Collect command definitions into a global signature registry (and reject
   duplicate signatures, missing `Documented:` requirements, etc.).
2. Validate command-like references against that registry.
3. Validate symbol usage and command type requirements (the type checker).

Important files:

- `check.rs` orchestrates the semantic passes (Id/Writing/signature/
  disambiguation/documented checks).
- `types.rs` defines checker data structures such as `SignatureRegistry`,
  `DefinitionTypeInfo`, `TypeFact`, and extension/spec/provided-symbol rules.
- `shapes.rs` computes canonical command signatures and argument shapes,
  including specialized mapping-parameter signatures (`_(n)`, `#n`, `#?`,
  and `#*`) and their general-signature fallback. It also preserves 2D curly
  argument row lengths so rectangular and variadic matrix shapes remain
  distinct from flat variadic groups.
- `validation.rs` validates references for existence and argument shape and
  ranks mapping-parameter overloads by arity and selector specificity. For 2D
  variadics it rejects empty, ragged, and flat actual groups and binds row and
  column length names independently.
- `typecheck.rs` implements symbol scope, facts, substitutions, requirements,
  subtyping through `extends:`, destructuring, operator/member resolution, and
  spec-operator reduction.
- `locator.rs` maps semantic diagnostics back to source locations.
- `definition.rs` and `rename.rs` back LSP go-to-definition and rename; `uses.rs`
  finds command occurrences (also used by `mlg release`).
- `walk.rs`/`walk/` traverses top-level groups, clauses, statements,
  expressions, forms, and support sections for reference validation.

The signature registry is global across all checked files. Duplicate command
signatures are rejected across `Defines`, `Declares`, `Refines`, `States`, and
named theorem-like groups. Mapping-parameter definitions additionally populate
a general-signature-to-specialized-signatures index. Several specialized
definitions may share one general signature, but each specialized signature is
still globally unique; an invocation must resolve to one uniquely most-specific
candidate.

The type checker is intentionally conservative. It checks command references,
argument shapes, declared symbols, known type/spec facts, command requirements,
and subtype/spec implications. It is not a proof checker for theorem
conclusions.

### View Builder and Renderer

Location: `src/backend/view/`

The view backend builds a serialized presentation model consumed by the web
viewer.

Main files:

- `model.rs` defines `CollectionView`, `DirectoryView`, `PageView`, `FileView`,
  `GroupView`, `SectionView`, and `ArgumentView`.
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

Variadic template substitutions retain shape metadata alongside their rendered
values. Concrete 2D command arguments carry their row lengths, while symbolic
2D header parameters carry indexed rows plus explicit ellipsis rows. This lets
`render/templates.rs` apply nested row/column notation consistently to both an
invocation and its definition-card title.

The view builder deliberately emits a presentation-oriented JSON model instead
of exposing frontend AST internals to TypeScript. This keeps the web viewer
stable when Rust AST internals change.

Structural parsing and semantic checking happen before the view builder through
the shared `SourceCollection` passes in `src/backend/collection.rs`. The
builder is therefore a rendering pass, not a second private checker.

## Web Viewer Architecture

Location: `web/`

The web viewer is a static React application built with Vite. Its generated
assets are compiled into the Rust executable with `rust-embed`.

Source layout:

```text
web/
├── app/
├── components/
├── dist/
├── lib/
├── public/
├── index.html
├── package.json
├── vite.config.ts
└── tsconfig.json
```

Key files:

- `web/app/main.tsx` reads the runtime configuration, loads the collection view,
  and renders the shell.
- `web/index.html` contains placeholders that Rust replaces with the live or
  exported viewer configuration.
- `web/vite.config.ts` provides the frontend development server and its local
  collection-data endpoint.
- `web/dist/` contains the committed production build embedded by
  `src/mlg/view_assets.rs`.
- `web/lib/types.ts` mirrors the Rust serialized view model.
- `web/lib/presenter.ts` contains route, label, and file-browser presentation
  helpers.
- `web/components/` renders the viewer shell, file list, group cards, section
  content, argument lists, and LaTeX.

Group cards divide their sections into primary content and supporting content.
`Documented:`, `References:`, item-level `Writing:`, `Enables:`, `Provides:`,
`Justification:`, and `Id:` are collapsed behind the supporting-sections toggle
by default.

Viewer themes are declared centrally in `web/components/viewer-theme.ts`. The
current choices are Classic, Mono, Flat Gray, Sepia, Retro, and Atomic, each
with light and dark variants. Classic is the default; the selected theme is
stored under `mlg-view-theme` in browser local storage and applied through the
root `data-theme` attribute and matching `color-scheme`.

`mlg view` writes a temporary `collection.json` and serves it with the embedded
assets from a small Rust HTTP server. The web app reads only that JSON endpoint.
It does not parse `.mlg` files and does not run semantic checks. Node.js and npm
are frontend development dependencies, not runtime dependencies of `mlg view`
or `mlg export`.

After changing frontend code, contributors run `npm install` and `npm run build`
inside `web/`, then commit the updated `web/dist/` files. This keeps Cargo
builds, packaged crates, and downloaded binaries self-contained.

## Check Command Data Flow

`mlg check` is implemented by `src/mlg/check.rs`.

```text
main.rs
  -> Cli::parse
  -> mlg::check_in
  -> format_before_checking (unless formatOnCheck is false; before load, so
       reported positions match the source the author reads)
  -> SourceCollection::load
      -> find collection root
      -> validate mlg.json when a collection root exists
      -> collect collection .mlg files (from content/ if present)
      -> generate + write missing Id: sections
  -> SourceCollection::diagnostic_filter for explicit PATH arguments
  -> SourceCollection::run_check_passes_filtered
      -> frontend::parse_source_file for each .mlg file (proto -> structural
           -> formulation)
      -> backend::text_fence::check_text_fence_syntax (```mlg fences in prose)
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
  -> bind the requested port once
  -> serve collection.json and the embedded viewer assets from Rust
```

The viewer command treats parser and semantic errors as blocking, because the
rendered output would otherwise be misleading.

The viewer watches source files and refreshes the temporary JSON after valid
changes. The HTTP server continues to serve the last valid view if a refresh
contains parser or semantic errors.

## Release Command Data Flow

`mlg release` is implemented by `src/mlg/release.rs`.

```text
main.rs
  -> Cli::parse
  -> mlg::release
  -> find collection root (mlg.json)
  -> require a clean Git work tree and capture HEAD sha
  -> SourceCollection::load + run_check_passes  (must be error-free)
  -> backend::release::build_release_items
      -> proto parser for item id/kind/source
      -> semantic command-use resolution for dependency edges
  -> read existing metadata/ (collection.json, items/<id>.json)
  -> compute the update set as a transitive closure over `uses` edges
  -> write metadata/, then bump mlg.json version
```

The command snapshots the current committed state of the collection into a
`metadata/` directory next to `content/`:

- `metadata/collection.json` is an append-only list of releases, each recording
  the new repo `version`, the `version_control_sha256` of the commit the release
  corresponds to, and the `--summary` text.
- `metadata/items/<id>.json` is an append-only version history for each top-level
  item, keyed by the SHA-256 of the item's source.

An item is (re)versioned when its content hash changed since its last recorded
entry. In addition, when a *definition* (`Declares`, `Defines`, `States`,
`Refines`, `Disambiguates`) is (re)versioned, every definition it uses is
re-versioned transitively, deduplicated so each item gains at most one new entry
per release. The whole update set is computed in memory before anything is
written, and `mlg.json` is bumped last.

A real (non-dry-run) release finishes by regenerating the published site: the CLI
(`src/main.rs`) runs `mlg clean` then `mlg export`, replacing `docs/` so it
matches the release just recorded. This orchestration lives at the binary level
because it composes whole commands, each with its own console listener; the
`release` library entry point itself only writes the release metadata.

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
by the hand-written functions in `src/frontend/formulation/parser.rs`.

## Testing Layout

Tests live close to the code they exercise.

Examples:

- tests in `src/frontend/formulation/parser.rs` cover formulation parsing.
- tests in `src/frontend/structural/parser.rs` cover structural parsing.
- tests in `src/mlg/check.rs` cover command-level checking behavior.
- `src/backend/view/render/tests.rs` covers rendering behavior.
- `src/backend/semantic/` behavior is covered through semantic and command
  tests.

Golden parser outputs are stored in:

```text
goldens/formulation/
goldens/structural/
```

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
  update `src/frontend/structural/ast.rs`, structural parser dispatch, parser
  helpers/tests, semantic walkers if it can contain command references, renderer
  support if it should be visible, and docs.
- Add a new formulation construct:
  update `src/frontend/formulation/ast.rs`, `grammar.lalrpop` or hand-written
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
- The TypeScript viewer build is committed in `web/dist/` and embedded in the
  executable. Frontend source changes must include a refreshed production build.
- The language has syntax forms parsed by generated grammar and syntax forms
  parsed by scanner helpers. Changes must account for both paths.

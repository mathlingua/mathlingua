# Mathlingua Cheat Sheet

A concise high-level reference for the Mathlingua language. For in-depth specifications and guides, see the [Language Guide](language.md), [Structural Syntax](structural_syntax.md), [Formulation Syntax](formulation_syntax.md), and [Type and Operator Resolution](type_and_operator_resolution.md).

**Jump to:**
- [Top-Level Structural Groups](#top-level-structural-groups)
- [Structural Clauses](#structural-clauses)
- [Supporting Sections](#supporting-sections)
- [Headings and Headers](#headings-and-headers)
- [Formulations, Declarations, and Operators](#formulations-declarations-and-operators)
- [Builtin Types and Commands](#builtin-types-and-commands)
- [Prose Delimiters and Citations](#prose-delimiters-and-citations)
- [Command-Line Interface (CLI)](#command-line-interface-cli)

| Syntax Name | Syntax Example | Description |
| :--- | :--- | :--- |
| <a id="top-level-structural-groups"></a>**Top-Level Structural Groups** | | |
| `Title` | `Title: "..."` | Top-level document title |
| `SectionTitle` | `SectionTitle: "..."` | Major section heading within a document |
| `SubsectionTitle` | `SubsectionTitle: "..."` | Subsection heading within a section |
| `Text` | `Text: "..."` | Standalone narrative prose block |
| `Writing` | `Writing: "name :~> body"` | Custom LaTeX rendering alias for a name or symbol |
| `Declares` | `Declares: f(x__)` | Declares a mathematical symbol, function shape, or operator |
| `Defines` | `Defines: X := \set@{...}` | Defines a mathematical concept or object with explicit construction |
| `Realizes` | `Realizes: Nb := \naturals` | Supplies concrete implementation for an abstract definition |
| `Refines` | `Refines: x is \real` | Specializes a declaration with additional constraints or properties |
| `States` | `States:` | Declares a named mathematical property, condition, or theorem-like claim |
| `Axiom` | `Axiom:` | Assumed foundational mathematical truth |
| `Theorem` | `Theorem:` | Proven mathematical statement (optionally with proof) |
| `Conjecture` | `Conjecture:` | Unproven mathematical hypothesis |
| `Example` | `Example:` | Concrete instance or demonstration of a concept |
| `Equivalent` | `Equivalent:` | States mutual equivalence between multiple mathematical claims |
| `Relation` (undirected) | `Relation: ... between: ... and: ...` | Symmetric or undirected relationship between entities |
| `Relation` (directed) | `Relation: ... from: ... to: ...` | Directed relationship from a source to a target entity |
| `Disambiguates` | `Disambiguates:` | Dispatches overloaded operators or functions by argument types |
| `Specify` | `Specify:` | Collection-wide numeric literal type mapping (decimal, int, etc.) |
| `Person` | `Person: "Name"` | Author, contributor, or mathematician metadata |
| `Resource` | `Resource:` | Bibliographic source (book, paper, website) cited by entries |
| `Topic` | `Topic: "Name"` | Subject tag or domain classification |
| <a id="structural-clauses"></a>**Structural Clauses** | | |
| `forAll` | `forAll: x_` | Universal quantifier over variables or specifications |
| `exists` | `exists: x_` | Existential quantifier asserting at least one satisfying witness |
| `existsUnique` | `existsUnique: x_` | Uniqueness quantifier asserting exactly one satisfying witness |
| `if` | `if: P` | Conditional premise implying following clauses |
| `have` (iff) | `have: P ... iff: Q` | Biconditional assertion stating equivalence between facts |
| `have` (asserting) | `have: P ... asserting: Q` | Deductive step establishing claim with local assertions and justifications |
| `piecewise` | `piecewise:` | Branching conditional definition or statement (`if`/`then`/`elseIf`/`else`) |
| `let` | `let: x_` | Introduces locally bound variables or hypotheses |
| `not` | `not: P` | Logical negation of a nested clause |
| `allOf` | `allOf:` | Conjunction requiring all nested clauses to hold |
| `anyOf` | `anyOf:` | Disjunction requiring at least one nested clause to hold |
| `oneOf` | `oneOf:` | Exclusive disjunction requiring exactly one nested clause to hold |
| `equivalently` | `equivalently:` | Asserts that all nested clauses are mutually equivalent |
| <a id="supporting-sections"></a>**Supporting Sections** | | |
| `using:` | `using: x is \set` | Fixed parameters or contextual variables in scope |
| `when:` | `when: x is \real` | Parameter typing or spec preconditions for a definition, relation, or disambiguation branch |
| `assuming:` | `assuming: A \:subset?:/ U` | Optional clauses after `when:` on definitions and relations implicitly asserted for type checking |
| `where:` | `where: x "in" X` | Conditions constraining quantified variables, `let:` bindings, or proposition givens |
| `suchThat:` | `suchThat: A \:subset?:/ U` | Optional clauses on propositions (`Axiom`/`Theorem`/`Conjecture`), `forAll:`, `exists:`, and `existsUnique:` implicitly asserted for type checking |
| `given:` | `given: x is \real` | Arbitrary given objects or hypotheses in a theorem, axiom, or conjecture |
| `then:` | `then: P` | Conclusion or consequence of premises |
| `iff:` | `iff: P` | Necessary and sufficient condition |
| `that:` | `that: P` | Body clauses of a `States:` definition |
| `to:` | `to: \cmd` | Target expression(s) of `Equivalent:`, `Disambiguates:`, directed `Relation:`, or `Topic:` `Related:` |
| `specifies:` | `specifies: e "in" X` | Component types/values on `Declares`/`Defines`/`Realizes`, inherited refinement on `Refines`, or relationship meaning on `Relation`/`Topic` |
| `satisfies:` | `satisfies: x > 0` | Invariant or property guaranteed by a declaration or refinement |
| `extends:` | `extends: X is \group` | Declares inheritance or specialization of base types or structures |
| `abstractly:` | `abstractly:` | Marks a `Defines:` specification as abstract, requiring realization via `Realizes:` |
| `implicitly:` / `explicitly:` | `implicitly:` | Marks a `Refines:` entry as inheriting vs. overriding a supertype's refinement |
| `expresses:` | `expresses: P` | One or more clauses that define a `Defines:` or `Realizes:` value indirectly |
| `by:` / `because:` | `by: \thm` | Optional theorem reference(s) and prose/clause rationale on `have:` and `Appendix:` `. justification:` |
| `Requires:` | `Requires:` | Prerequisites (capabilities or definitions) needed to use an entry |
| `Enables:` | `Enables:` | Capabilities, views, or conversions provided by an entry |
| `Proof:` | `Proof: "..."` | Informal or formal proof body for a theorem (placed before `Appendix:`) |
| `Appendix:` | `Appendix:` | Container for auxiliary sections (`called:`, `written:`, `adjective:`, `description:`, `overview:`, `related:`, `discoverer:`, `writing:`, `justification:`, `aliases:`, `references:`, `metadata:`) |
| `. called:` / `. written:` / `. description:` | `. called: "set"` | Human-readable documentation entries directly inside `Appendix:` |
| `. justification:` | `. justification:` | Deductive steps inside `Appendix:` discharging formulation labels (`[:label:]`) |
| `. aliases:` | `. aliases:` | Alternative notation aliases inside `Appendix:` (`:=>`, `:->`, `:<->:`) |
| `. writing:` | `. writing:` | Item-scoped LaTeX rendering overrides (`"name :~> body"`) or mapping writing (`writing: ... as: "..."`) inside `Appendix:` |
| `. references:` | `. references:` | Citations to external resources inside `Appendix:` (`$resource` or `for:` / `entry:`) |
| `. metadata:` | `. metadata:` | Versioning metadata inside `Appendix:` (`version:`) |
| `Id:` | `Id: "uuid"` | Unique UUID identifier as the final section of any top-level item |
| <a id="headings-and-headers"></a>**Headings and Headers** | | |
| Command Heading | `[\function:on{A}:to{B}]` | Identifies group by full command pattern with inputs |
| Operator Heading | `[a \:(nonempty)::subset:/ b]` | Identifies group by infix operator or spec-infix pattern |
| Label Heading | `[label]` | Local label naming a clause, step, or justification |
| Author Header | `[@author.id]` | Identifies a person entry or cites an author |
| Resource Header | `[$resource.id]` | Identifies a bibliographic resource or cites a reference |
| Topic Header | `[#topic.id]` | Identifies a topic classification entry |
| <a id="formulations-declarations-and-operators"></a>**Formulations, Declarations, and Operators** | | |
| Structural Assignment | `X ::= {x_ : ...}` | Defines a structural pattern or alias matching literal forms |
| Value Assignment | `x := 0` | Binds a symbol or variable to a specific value or construction |
| Equality | `x = y` | Asserts mathematical equality between two expressions |
| Inequality | `x != y` | Asserts mathematical inequality between two expressions |
| Type Assertion | `x is \real` | Asserts that an expression belongs to a type or satisfies a specification |
| Type Query | `x is? \real` | Boolean predicate testing whether an expression satisfies a type |
| Refined Type / View | `x is \group via X` | Type assertion through an explicit view or structural adapter |
| Spec Literal | `_ is \real` | Anonymous specification value (`\\specification`) |
| Quoted Spec | `x "in" X` | Binary relation or membership specification |
| Quoted Spec Query | `x "in"? X` | Boolean predicate testing a quoted binary relation |
| Infix Command | `a \.plus./ b` | Infix invocation of a defined command |
| Infix Spec | `a \:subset:/ b` | Infix specification statement |
| Infix Spec Query | `a \:subset?:/ b` | Boolean predicate testing an infix specification |
| Adjective Refinement | `\(nonempty)::set` | Modifies a command type with a refined adjective constraint |
| Coercion | `\set@{...}` | Coerces a raw value to a command type |
| Function Literal | `(x_ is \real) => x_ + 1` | Anonymous lambda/mapping literal from typed inputs to an expression |
| Function Type | `(_ is \real) -> (_ is \real)` | Type signature for functions mapping domain to codomain |
| Named Function Call | `f[| x := 1, ... := 0 |]` | Calls a function with explicit `param := value` bindings |
| Membership / Satisfies | `x memberOf X` / `x satisfies S` | Built-in collection membership and specification-satisfaction predicates |
| Command Context | `\cmd{A}#using{X := S}` | Supplies `#using{...}` or `#given{...}` context bindings at a command use site |
| Expression Alias | `f(x_) :=> x + x` | Expression or operator alias mapping a form to an expression reduction |
| Directed Spec Alias | `x_ "in" R :-> x is \real` | Directed specification capability asserting that the LHS implies the RHS |
| Equivalence Spec Alias | `x_ "in" S :<->: x is \integer` | Bidirectional specification capability asserting LHS and RHS equivalence |
| Named Operators | `a \|op\| b`, `neg\| x`, `x \|prime` | Infix (`\|op\|`, `:\|op\|`, `\|op\|:`, `:\|op\|:`), prefix (`name\|`), and postfix (`\|name`) named operators |
| Member Access / Call | `M.*` / `M.*(x, y)` / `\reals..0` | Accesses or invokes a component of a structured object (`.` on variables, `..` on commands) |
| Set Literal / Builder | `{x_ : x_ "in" X}` | Set comprehension or collection literal |
| Tuple | `(a, b)` | Ordered sequence of two or more elements |
| Variadic / Slice | `x...` / `x[1...n]` | Represents an arbitrary sequence of arguments or array slice |
| Inferred Symbol | `x?` / `x?_` | Parameter whose value is inferred from context (or inferred mapping selector in named calls) |
| Inline Label | `(x + y)[:label:]` | Labels a grouped formulation (referenced via `(:label:)` in `Appendix:` `justification:`, `references:`, or prose) |
| Label Reference | `(:label:)` / `(:l1, l2:)` | Refers to a defined label in `Appendix:` `justification:` `. for:`, `references:` `. for:`, or prose |
| Line Label | `. [label]: x + y` | Labels a single-line formulation or text argument in a section |
| <a id="builtin-types-and-commands"></a>**Builtin Types and Commands** | | |
| Builtin Types | `\\type`, `\\statement`, `\\expression`, `\\specification`, `\\anything` | Language primitive types for classification and constraints |
| Builtin Capability Target | `\\abstract` | Placeholder target for abstract specification capabilities in `:->` or `:<->:` |
| Builtin Connectives | `\\not{P}`, `\\and{...}`, `\\or{...}`, `\\oneOf{...}` | Formulation-level logical connectives (also `\\allOf`, `\\anyOf`) |
| Builtin Quantifiers | `\\forAll{...}:then{...}`, `\\exists{...}`, `\\let{...}` | Formulation-level quantifiers (`\\forAll`, `\\exists`, `\\existsUnique`, `\\let`) |
| Builtin Conditionals | `\\if{...}:then{...}`, `\\have{...}:iff{...}`, `\\piecewise:if{...}:then{...}:else{...}` | Formulation-level conditional statements (`\\if`, `\\have`, `\\piecewise`) |
| Builtin Variadic Commands | `\\map{...}:to{...}`, `\\leftReduce{...}:on{...}` | Pointwise slice mapping (`\\map`) and reductions (`\\leftReduce`, `\\rightReduce`) |
| <a id="prose-delimiters-and-citations"></a>**Prose Delimiters and Citations** | | |
| Inline Formulation | `{. x + y .}` | Embeds an inline math expression inside prose text |
| Display Formulation | `{{. x + y .}}` | Embeds a centered display math expression inside prose text |
| Theorem Reference | `{: \thm :}` | Links to a theorem, axiom, or conjecture within prose |
| Labeled Prose Quote | `''quote''[:ref:]` | Labels a quoted phrase in prose (can be cited via `Appendix:` `. references:` `. for: (:ref:)`) |
| Scoped Context | `<<scope>> ... <</scope>>` | Constrains mathematical context and notations within prose |
| <a id="command-line-interface-cli"></a>**Command-Line Interface (CLI)** | | |
| Check | `mlg check [--watch] [--json] [--diagnostic-schema] [--read-only] [PATH...]` | Type-checks, validates LaTeX, and checks all `.mlg` files in the collection |
| Search | `mlg search [--json] <query>` | Searches collection for matching symbols, commands, or text |
| Structure | `mlg structure [--json]` | Inspects collection document hierarchy, TOC, and items |
| Format | `mlg format [--ids]` | Formats `.mlg` files to the configured margin (`--ids` also inserts missing `Id:` sections) |
| View | `mlg view [--port PORT] [--watch] [--read-only]` | Launches local web viewer server to browse the collection |
| Export | `mlg export [--base-path PATH] [--cname DOMAIN] [--force]` | Exports collection to a static HTML website in `outputDir` (default `docs/`) |
| Clean | `mlg clean` | Removes the generated `outputDir` directory |
| Init | `mlg init` | Scaffolds `mlg.json` and `content/` in the current directory |
| Release | `mlg release --summary TEXT [--dry-run] [--diff]` | Records a versioned release snapshot in `metadata/`, bumps `mlg.json`, and re-exports the site |
| Version | `mlg version` | Prints installed Mathlingua version information |
| Hidden Commands | `mlg lsp [--read-only]`, `mlg debug`, `mlg extract <ID>`, `mlg report <ID...>`, `mlg whte_rbt.obj` | Editor LSP server and internal/diagnostic subcommands |

# Formulation Syntax

This file describes the formulation language exactly as it is currently implemented in the Rust code.
For a more readable overview of how formulation syntax fits into the whole
language, start with [language.md](language.md).

Intended workflow:

1. Treat this file as the editable syntax spec for the formulation language.
2. When the language should change, update this file first.
3. Then update the code in `src/frontend/formulation/` to match.

At the time this file was written, it matches these implementation files:

- `src/frontend/formulation/token.rs`
- `src/frontend/formulation/grammar.lalrpop`
- `src/frontend/formulation/parser.rs`
- `src/frontend/formulation/mod.rs`

The generated parser is built from `src/frontend/formulation/grammar.lalrpop` by the crate-root `build.rs`.

## Scope

The formulation subsystem does not have one single root grammar. It exposes several entry points:

- **`parse_expression`** — general expressions.
- **`parse_ordinary_declaration_statement`** — declarations/definitions using
  `::=`, `:=`, `is`, or a quoted spec operator (wraps
  `parse_declaration_statement(input, allow_refined_type = false)`).
- **`parse_refined_declaration_statement`** — same, but the `is` target may be a
  refined command expression (`allow_refined_type = true`).
- **`parse_hard_cast_statement`** — `<subject> is! <type>` (optionally
  `<subject> := <value> is! <type>`).
- **`parse_expression_binding`** — `<expression> := <expression>`.
- **`parse_form_or_declaration`** — forms and declarations.
- **`parse_is_or_spec`** — internal `<is-subject> is <command-type>` or
  `<subject> "op" Name` helper.
- **`parse_is_or_refined_statement_spec`** — internal variant where `is` may
  target a refined command expression.
- **`parse_is_via_statement`** — `<is-statement> via <form-or-declaration>`.
- **`parse_command_header`** — simple, infix, infix-spec, or refined command
  headers.
- **`parse_writing_alias`** — `<form-or-declaration> :~> <raw body>`.
- **`parse_expression_alias`** — `<lhs> (:=> or :->) <expression>`.
- **`parse_spec_operator_alias`** — `<placeholder-spec> :-> <target>`, where the
  target is an is-or-spec, `member_of`, a placeholder-spec, or a builtin.
- **`parse_label_header`** — dotted label header text.
- **`parse_author_header`** — `@` followed by dotted parts.
- **`parse_resource_header`** — `$` followed by dotted parts.
- **`parse_topic_header`** — `#` followed by dotted parts.

(The base `parse_declaration_statement(input, allow_refined_type)` takes a flag; the two `parse_*_declaration_statement` wrappers above are the public entry points.)

`parse_expression` is not the whole language. Command headers, aliases, and several statement-like forms are parsed separately.

## Notation

This document uses the following notation:

- `Name` means a formulation name token.
- `Placeholder` means a name ending in `_`.
- `MagneticPlaceholder` means a name ending in `__`.
- `A?` means optional.
- `A*` means zero or more.
- `A+` means one or more.
- Quoted literals like `"is"` mean exact surface text.

## Lexical Rules

### Whitespace

- Whitespace matched by `[ \t\r\n\f]+` is ignored by the lexer.

### Names

A normal formulation name is either:

- a stropped symbolic name: `` `...` `` where the inside text is non-empty and uses only operator characters from `-~!#%^&*\+=|<>/`, optionally with trailing primes (`` `*'` ``)
- an identifier-like name matching:

```text
[A-Za-z0-9](?:[A-Za-z0-9_']*[A-Za-z0-9'])?
```

Consequences:

- names may start with a digit
- names may contain `_` internally
- names may carry trailing prime marks (`'`) after an alphanumeric, so a name may end in a prime (`X'`, `X''`), including on a subscript (`x'_a'`)
- names must otherwise end in an ASCII letter or digit
- `_x`, `x_`, `x__`, and `'x` are not normal names

Examples of valid normal names:

- `x`
- `x_1`
- `X'`
- `x'_a'`
- `123`
- `` `*` ``
- `` `*+` ``
- `` `*'` ``

### Placeholders

A placeholder is an identifier-like name followed by `_`:

```text
[A-Za-z0-9](?:[A-Za-z0-9_']*[A-Za-z0-9'])?_
```

A magnetic placeholder is the same base name followed by `__`:

```text
[A-Za-z0-9](?:[A-Za-z0-9_']*[A-Za-z0-9'])?__
```

Examples:

- `x_`
- `value_`
- `x__`

### Quoted names

The lexer token `QuotedName` is restricted. It is not an arbitrary string literal. It must match:

```text
"[A-Za-z0-9](?:[A-Za-z0-9_']*[A-Za-z0-9'])?"
```

So these are valid in lexer-driven expression parsing:

- `"in"`
- `"maps_to"`

These are not valid there:

- `"less than"`
- `"x+y"`

Important implementation distinction:

- expression parsing uses this strict `QuotedName` token
- declaration statement parsing, `parse_is_or_spec`, and `parse_spec_operator_alias` do not use that token for the quoted operator
- those helper parsers accept any top-level `"..."`

So `x "less than" A` is rejected by `parse_expression`, but accepted by
`parse_declaration_statement` and `parse_is_or_spec`.

### Labels on grouped formulations

Labels use the token form:

```text
[:part(.part)*:]
```

Each `part` uses the same identifier-like rule as ordinary non-stropped names.

Important distinction:

- formulation labels like `[:a.b:]` are lexer tokens and do not support stropped symbolic parts
- structural label headers parsed by `parse_label_header` do support stropped symbolic name parts because they use the raw helper parser, not the lexer token

A label may follow a parenthesized or dot-parenthesized expression, statement,
or specification. Declaration-shaped formulations retain the wrapper in their
statement AST, while expression-shaped formulations use `ExpressionKind::Labeled`.
Consequently the same syntax is valid at top level and wherever that formulation
may be nested.

### Named operators

The lexer recognizes four infix named-operator spellings:

- plain: `|name|`
- left-colon: `:|name|`
- right-colon: `|name|:`
- both-colon: `:|name|:`

For form declarations, it also recognizes:

- prefix-form operator: `name|`
- postfix-form operator: `|name`

In the infix spellings, the content between the bars may be a dotted **member
path** whose segments are identifier-like names or operator-symbol runs — for
example `|M.*|` or `|x.y.z|`. This lets an operator name track down through a
value's fields (`x |M.*| y` is the member call `M.*(x, y)`). The prefix and
postfix form-operator spellings remain single identifier-like names.

A bracketed operator `[op]` (e.g. `[*]`) is also accepted as an infix
form-operator inside a capability declaration; the brackets are kept in the
operator's text to mark it as a placeholder whose symbol is drawn from the
definition's inputs/`Defines:`.

### Special operators

The lexer recognizes:

- multi-character operator strings made from `-~!#%^&*+=|<>/`
- single-character operators from `~!#%&<>`
- the individual punctuation tokens `+ - * / = ^` are also tokens in their own right

An operator run may also carry trailing prime marks (`'`) and/or a `_`-prefixed
subscript, so `*'`, `*''`, `*_1`, and `*'_a` are single operator tokens (a bare
`*` remains the punctuation token).

The raw helper parser also treats any non-empty string made only from `-~!#%^&*\\+=|<>/`, with optional trailing primes and subscript, as operator text.

### Reserved keywords and punctuation

The lexer has dedicated tokens for:

- `is`
- `is?`
- `is_not?`
- `via`
- `member_of`
- `satisfies`
- `::=`
- `:=`
- `...`
- `:=>`
- `:->`
- `:~>`
- `->`
- `=>`
- `@`
- `@!`
- `\`
- `\.`
- `./`
- `\:`
- `:/`
- `?:/`
- `(.`
- `.)`
- `[|`
- `|]`
- the colon-decorated arithmetic operators `:+:` `:+` `+:` `:-:` `:-` `-:` `:*:` `:*` `*:` `:^:` `:^` `^:`
- the colon-decorated special-operator forms `:op:` `:op` `op:` (used for owner-typed operator resolution)
- `(` `)` `{` `}` `[` `]` `,` `;` `:` `.` `|` `$` `?` `:?`

Because these are tokenized before ordinary names, exact spellings like `is` and `via` are effectively reserved in lexer-driven formulation parsing. Note that `is!` (the hard-cast statement) is **not** a lexer token — it is recognized by a top-level scan for ` is! ` in `parse_hard_cast_statement`, so it may be surrounded by ordinary tokens.

### Build expressions

A build applies a command type to a value at a stated abstraction level:

- `<command-type> @ <value>` — soft build (coercion), e.g. `\set@{...}`,
  `\rational@k`.
- `<command-type> @! <value>` — hard build (coercion + encoding), e.g.
  `\set@!m`.

`@`/`@!` bind the command type on the left to the primary expression on the
right. These replace the removed `value as \type` / `value as! \type` casts. A
build is also how a `Declares:` value may state its type without `is`
(`X := \set@{...}` is sugar for `... is \set`).

## Expression Grammar

### High-level precedence

From lowest precedence to highest:

1. function literal `=>` (right-associative)
2. spec and predicate forms (`is`, `is?`, `is_not?`, quoted `"op"` specs and
   predicates, infix specs `\:...:/`, `member_of`, `satisfies`, spec literals)
3. infix command `\.name./`
4. equality `=` (and `!=`) and special binary operators such as `<`, `>`
5. additive `+ -`
6. multiplicative `* /`
7. power `^`
8. named-operator forms (`|op|`, member-path `|M.*|`, colon variants)
9. unary prefix (`+`, `-`, and prefix named operators `name|`)
10. postfix named operators (`x |f`)
11. atomic / primary forms

Note that infix-command forms and named-operator forms are at **different**
levels (3 and 8): an infix command binds looser than arithmetic, while a named
operator binds tighter than arithmetic.

Associativity:

- `=` is left-associative
- `+` and `-` are left-associative
- `*` and `/` are left-associative
- `^` is right-associative
- named-operator and infix-command expressions are left-associative at their precedence level

### Grammar

```text
Expression ::= MappingExpression

MappingExpression ::=
    SpecOrPredicateExpression "=>" MappingExpression
  | SpecOrPredicateExpression

SpecOrPredicateExpression ::=
    InfixCommandExpression InfixSpec InfixCommandExpression
  | InfixCommandExpression QuotedName Name
  | InfixCommandExpression QuotedName "?" Name
  | InfixCommandExpression QuotedName CommandExpression
  | InfixCommandExpression "is" PredicateTypeExpression
  | InfixCommandExpression ("is?" | "is_not?") (CommandExpression | BuiltinTypeExpression)
  | InfixCommandExpression "member_of" InfixCommandExpression
  | InfixCommandExpression "satisfies" InfixCommandExpression
  | "?" "is" PredicateTypeExpression
  | "?" QuotedName (Name | CommandExpression)
  | InfixCommandExpression

PredicateTypeExpression ::= CommandExpression | BuiltinTypeExpression | Name
BuiltinTypeExpression ::= "\\" "\\" Chain

InfixCommandExpression ::=
    InfixCommandExpression InfixCommand EqualityExpression
  | EqualityExpression

EqualityExpression ::=
    EqualityExpression ("=" | SpecialOperator) AdditiveExpression
  | AdditiveExpression

AdditiveExpression ::=
    AdditiveExpression ("+" | "-") MultiplicativeExpression
  | MultiplicativeExpression

MultiplicativeExpression ::=
    MultiplicativeExpression ("*" | "/") PowerExpression
  | PowerExpression

PowerExpression ::=
    HighPrecedenceExpression "^" PowerExpression
  | HighPrecedenceExpression

HighPrecedenceExpression ::=
    HighPrecedenceExpression NamedOperator UnaryExpression
  | UnaryExpression

UnaryExpression ::=
    PrefixOperator UnaryExpression
  | PostfixExpression

PostfixExpression ::= PrimaryExpression PostfixNamedOperator
                    | PrimaryExpression

PrimaryExpression ::=
    LabeledExpression
  | GroupedExpression
  | FunctionExpression
  | TupleExpression
  | SetExpression
  | SubsetExpression
  | CommandExpression
  | BuiltinCommandExpression
  | MemberCall            -- Name "." Name "(" args ")"
  | MemberAccess          -- Name "." Name
  | Build                 -- CommandExpression ("@" | "@!") PrimaryExpression
  | InferredName          -- Name "?"
  | MagneticPlaceholder   -- x__
  | Placeholder           -- x_
  | Name
```

This reference does not spell out every production in full. `parse_expression`
also performs top-level scans before invoking the generated grammar for refined
predicates and spec-infix expressions, structural literal types, set expressions
whose targets use the full expression language, collection builds, contextual
commands, and variadic assignments.

The lower (spec/predicate) precedence level includes, besides `is`/`is?`:

- **Function literals** `(<param>) => <expression>` (anonymous functions), at
  the outermost, right-associative mapping level.
- **Infix specification** statements/predicates `<a> \: chain :/ <b>` and the
  `?:/` predicate form.
- **Quoted-operator specs and predicates** `<a> "op" <b>`, `<a> "op"? <b>`, and
  `<a> "op" <command>`.
- **`member_of`** and **`satisfies`** expressions.
- **Spec literals** with an implicit `?` subject: `? is T`, `? "op" name`,
  `? "op" \cmd` (values of type `\\specification`).
- **`BuiltinCommandExpression`** — the value form `\\chain{arg; arg}:tail{...}`
  with `;`-separated arguments (distinct from a builtin *type* `\\Chain`).
- A command expression may carry a **`#using{...}` / `#given{...}` context**
  suffix.

Command arguments also support surface sugar: a bare collection literal
`\foo{x_ : ...}` is read as `\foo{{x_ : ...}}`, and there are function-literal and
build-function-literal argument forms.

### Atomic forms

#### Grouped expressions

```text
GroupedExpression ::= "(" Expression ")" | "(." Expression ".)"
```

The AST records whether the grouped form used the dot-delimited spelling.

#### Labeled expressions

```text
LabeledExpression ::= GroupedExpression Label
```

Only grouped formulations may be labeled directly. In a declaration-bearing
context, the same wrapper may contain a declaration or specification such as
``(.*' := `*`.)[:operation:]`` or `(.x is \real.)[:typed:]`.

Examples:

- `(x + y)[:sum:]`
- `(. x + y .)[:normalized:]`

#### Function expressions

```text
FunctionExpression ::=
    Name "(" Expression ("," Expression)* ")"
  | Name "[|" NamedFunctionElement ("," NamedFunctionElement)* "|]"

NamedFunctionElement ::= NamedFunctionLhs (":=" | "=") SpecOrPredicateExpression
NamedFunctionLhs ::= Name | InferredParameterName | SubsetNameCall
                   | RangedMappingSelector | "..."
InferredParameterName ::= Name "?_"
RangedMappingSelector ::= Placeholder "[" Placeholder
                          "[" ("0" | "1") "..." Name "]" "]"
```

Examples:

- `f(x, y)`
- `map[| key := x, value := y |]`
- `f[|x1?_ = 1, x2?_ = 2, ... = 0|]`
- `f[|x_[i_[1...m]] = 1, ... = 0|]`

#### Tuple expressions

```text
TupleExpression ::= "(" TupleExpressionElement "," TupleExpressionElement ("," TupleExpressionElement)* ")"
TupleExpressionElement ::= SpecOrPredicateExpression | Operator
```

Important implementation detail:

- tuples must have at least two elements
- a one-element tuple is not supported

Operators may appear as tuple elements, for example `(+, x)`.

#### Set expressions

```text
SetExpression ::= "{" CollectionTarget ":" "..." "}"
                | "{" CollectionTarget ":" Expression (("," | ";") Expression)* ("|" SetPredicate)? "}"
CollectionTarget ::= SetTarget | Expression
SetPredicate ::= Expression | SetTarget ":=" Expression
```

Examples:

- `{x_ : x_ is \real}`
- `{x_ : x_ is \real | x_ = y}`
- `{(p_, q_) : ...}`
- `{\equivalence.class:of{x_}:over{R} : x_ "in" X}`

Any expression may be the produced member to the left of the collection
separator. Placeholder names within that expression are local to the collection;
ordinary names continue to refer to the surrounding scope.

#### Subset expressions

Subset expressions are limited to these exact shapes:

```text
SubsetExpression ::= Name "[" Name "]"
                   | Name "[" Name "," Name "]"
                   | Name "[" Name "[" Name "]" "]"
```

The indices are names only, not arbitrary expressions.

Examples:

- `F[A]`
- `F[A, B]`
- `F[A[B]]`

#### Command expressions

```text
CommandExpression ::= "\" Chain CurlyExpressionArgs* CommandExpressionTail* ParenExpressionArgs*

CommandExpressionTail ::= ":" Chain CurlyExpressionArgs+

CurlyExpressionArgs ::= "{" Expression ("," Expression)* "}"
ParenExpressionArgs ::= "(" Expression ("," Expression)* ")"
```

Important implementation detail:

- each tail part must have at least one `{...}` argument block
- zero or more top-level `{...}` blocks are allowed before the first tail
- zero or more trailing `(...)` blocks are allowed after all tail parts

Examples:

- `\f`
- `\function{A}{B}`
- `\function:on{A}:to{B}(x)`
- `\relation:from{A}:to{B}(x)(y)`

#### Infix commands

```text
InfixCommand ::= "\." Chain CurlyExpressionArgs* CommandExpressionTail* "./"
```

This syntax is only produced inside higher-precedence binary expressions and certain command-header contexts.

### Chains

Many command-related syntaxes use a `Chain`:

```text
Chain ::= ChainPart ("." ChainPart)*
ChainPart ::= Name | "$" Name | SpecialOperator | "="
RawChain ::= RawChainPart ("." RawChainPart)*
RawChainPart ::= Name | "$" Name | OperatorText
```

Examples:

- `function`
- `binary.op`
- `$alias`
- `<=`

Raw helper parsers used for command headers, refined commands, and built-in
spec-alias targets accept the broader `OperatorText` class for chain parts.
Lexer-driven command expressions accept `SpecialOperator` chain parts. For
single-character operator command names such as `+`, use a stropped name like
``\`+\``` in expression syntax, or the raw command-header helper spelling
where applicable.

Named operators like `|plus|` are not chain parts.

## Forms and Declarations

### Grammar

```text
FormOrDeclaration ::=
    Name
  | MappingParameter
  | FunctionFormOrDeclaration
  | TupleFormOrDeclaration
  | SetFormOrDeclaration
  | Placeholder InfixFormOperator Placeholder
  | PrefixFormOperator Placeholder
  | Placeholder PostfixFormOperator

FunctionFormOrDeclaration ::= [Name "::="] FunctionForm
FunctionForm ::= Name "(" MagneticPlaceholder ")"
               | Name "(" Placeholder ("," Placeholder)* ")"
               | Name "(" VariadicMappingParameter ")"

VariadicMappingParameter ::= (Placeholder | MagneticPlaceholder)
                             "[" Placeholder ":=" ("0" | "1") "..." Name "]"
MappingParameter ::= Name "." Placeholder
                   | Name "." Name "?" "_"
                   | Name "." Placeholder "[" Placeholder
                     "[" Placeholder ":=" ("0" | "1") "..." Name "]" "]"

TupleFormOrDeclaration ::= [Name "::="] TupleForm
TupleForm ::= "(" TupleFormElement "," TupleFormElement ("," TupleFormElement)* ")"
TupleFormElement ::= FormOrDeclaration | Operator

SetFormOrDeclaration ::= [Name "::="] SetForm
SetForm ::= "{" PlaceholderForm [":" "..."] "}"

PlaceholderForm ::= Placeholder
                  | Placeholder "(" Placeholder ("," Placeholder)* ")"
```

### Notes

- function forms support either:
  - exactly one magnetic placeholder, or
  - one or more ordinary placeholders
- a function form may instead contain one ranged variadic mapping parameter;
  `_` spreads its inputs and `__` treats them as a tuple
- a ranged variadic mapping parameter must be the only parameter, must name its
  length, and must start at 0 or 1
- mixed ordinary and magnetic placeholders are not allowed
- tuple forms also require at least two elements
- unnamed function/tuple/set forms are still represented internally as declaration variants with `name: None`

Examples:

- `x`
- `f(x_)`
- `f(x_[i_:=1...n])`
- `f(x__[i_:=1...n])`
- `f.x_`
- `f.u?_`
- `f.x_[i_[j_:=1...m]]`
- `g ::= f(x_, y_)`
- `(x_, y_)`
- `Pair ::= (x_, y_)`
- `{x_ : ...}`
- `Set ::= {x_ : ...}`
- `x_ |plus| y_`
- `neg| x_`
- `x_ |prime`

## Statement-Like Form Parsers

These are not part of `parse_expression`. They are helper parsers built in `src/frontend/formulation/parser.rs`.

### `parse_declaration_statement`

Accepted shape:

```text
DeclarationStatement ::= DeclarationBody DeclarationRelation?

DeclarationBody ::=
    IsSubject
  | IsSubject "::=" IsSubject
  | IsSubject ":=" Expression
  | IsSubject "::=" IsSubject ":=" Expression

DeclarationRelation ::=
    " is " TypeExpression
  | TopLevelQuotedOperator Expression
```

Notes:

- `::=` is the only declaration-side expansion marker
- the subject and optional `::=` expansion introduce symbols
- a chained function declaration `X ::= x(i_) ::= y_` keeps
  `X ::= x(i_)` as the function form and uses `y_` as its output; it introduces
  the alias `X`, mapping name `x`, input `i_`, and output `y_`
- a top-level `:=` introduces a value definition; its right-hand side is parsed as an expression and does not introduce new names
- when the subject is a function declaration such as `f(x_, y_)`, the `::=` expansion must be a single placeholder output such as `z_`
- nested `:=` inside expression syntax, such as named function calls, remains expression syntax

Examples:

- `G ::= (X, *, e)`
- `G ::= (X, *, e) := (a, b, c) is \foo`
- `f(x_, y_) ::= z_ := x_ + y_ is \function`
- `X ::= x(i_) ::= y_`
- `f(x_) := x_ + 1 is \real.function`
- `{x_ : ...} := {x_ : x_ is \real} is \set`
- `X ::= {x_ : ...} "in" \some.collection.of.sets`

### `parse_is_or_spec`

Accepted shapes:

```text
IsOrSpec ::= IsStatement | SubjectSpecStatement
IsStatement ::= IsSubject " is " TypeExpression
SubjectSpecStatement ::= SpecSubject TopLevelQuotedOperator Name
IsSubject ::= IsSubjectFormList | OperatorText
SpecSubject ::= FormOrDeclaration | OperatorText
IsSubjectForm ::= FormOrDeclaration | PlaceholderForm
IsSubjectFormList ::= IsSubjectForm ("," IsSubjectForm)*
TypeExpression ::=
    CommandExpression
  | BuiltinTypeExpression
  | FunctionTypeExpression
  | TupleLiteralType
  | SetLiteralType
  | SpecLiteralFunctionType
BuiltinTypeExpression ::= "\\" "\\" Chain
SpecLiteral ::= "?" "is" TypeExpression | "?" TopLevelQuotedOperator Expression
TupleLiteralType ::= "(" SpecLiteral "," SpecLiteral ("," SpecLiteral)* ")"
SetLiteralType ::= "{" (SpecLiteral | TupleLiteralType) ":" "..." "}"
SpecLiteralFunctionType ::= "(" SpecLiteral ("," SpecLiteral)* ")" "->" "(" SpecLiteral ")"
```

Notes:

- the parser looks for the exact top-level substring ` is ` with spaces around it
- the left-hand side of `is` may be a single form, a single placeholder form, a comma-separated list mixing those, or an operator
- the right-hand side of `is` must parse as a command expression, a built-in
  type expression, or a function type expression, not a general expression
- a function type has one parenthesized input spec list and one parenthesized
  output spec: `(_ "in" A, _ "in" B) -> (_ "in" C)`
- function type specs use `_` as the parameter and may be either `_ is Type` or
  `_ "operator" Target`
- structural literal types use `?` spec literals at every leaf; raw nominal
  tuple, set, mapping, and arrow types are not accepted
- `->` has one or more input specs and exactly one output spec; the declared
  input arity is preserved
- function types always use `->`; `=>` is reserved for function literals
- if no top-level ` is ` is found, the parser falls back to the quoted-operator spec form
- the quoted operator is extracted by raw scanning, so it may contain spaces or punctuation

Examples:

- `f(x_) is \function:on{A}:to{B}`
- `f(x_), y_ is \function:on{A}:to{B}`
- `f is (_ "in" A) -> (_ "in" B)`
- `(x, y) is (? is \natural, ? "in" \reals)`
- `{x : ...} is {? is \natural : ...}`
- `f is (? is \natural) -> (? "in" \naturals)`
- `f is (? is \natural, ? "in" \reals) -> (? is \real)`
- `+ is \operator`
- `x "in" A`
- `x "less than" A`

### `parse_is_or_refined_statement_spec`

Same as `parse_is_or_spec`, except:

```text
TypeExpression ::=
    CommandExpression
  | BuiltinTypeExpression
  | RefinedCommandExpression
  | FunctionTypeExpression
  | TupleLiteralType
  | SetLiteralType
  | SpecLiteralFunctionType
```

### `parse_is_via_statement`

Accepted shape:

```text
IsViaStatement ::= IsStatement " via " FormOrDeclaration
```

Notes:

- the parser looks for the exact top-level substring ` via ` with spaces around it
- the left side must be an `is` statement, not a quoted-operator spec
- the right side is a form/declaration such as `X` or `(X, Y)`

## Refined Command Syntax

Refined command syntax is implemented by custom helper parsers, not by the LALRPOP expression grammar.

### Refined command expressions

```text
RefinedCommandExpression ::=
    "\" RefinedLeft "::" RefinedTail CurlyExpressionArgs* CommandExpressionTail* ParenExpressionArgs*

RefinedLeft ::= [RawChain "."] "(" RefinedExpressionPart ("," RefinedExpressionPart)* ")"
RefinedExpressionPart ::= RawChain CommandExpressionTail*
RefinedTail ::= "[[" Name "]]" | RawChain
```

Rules:

- the whole construct must start with `\`
- a top-level `::` is required
- the left side must contain a top-level parenthesized part list
- if the optional prefix chain is present, it must end with `.`
- the part list must contain at least one part

Examples:

- `\(f)::[[g]]`
- `\prefix.(left, right:at{x})::tail{A}(x)`

### Refined command headers

Refined command headers follow the same overall idea, but use form arguments instead of expression arguments:

```text
RefinedCommandHeader ::=
    "\" RefinedHeaderLeft "::" RefinedTail CurlyHeadingArgs* CommandHeaderTail* ParenHeadingArgs*

RefinedHeaderLeft ::= [RawChain "."] "(" RefinedHeaderPart ("," RefinedHeaderPart)* ")"
RefinedHeaderPart ::= RawChain CommandHeaderTail*
RefinedTail ::= "[[" Name "]]" | RawChain

CurlyHeadingArgs ::= "{" FormOrDeclaration ("," FormOrDeclaration)* "}"
ParenHeadingArgs ::= "(" HeadingParameter ("," HeadingParameter)* ")"
HeadingParameter ::= FormOrDeclaration | Placeholder
CommandHeaderTail ::= (":" | ":?") RawChain CurlyHeadingArgs+
```

## Command Header Syntax

`parse_command_header` chooses among four cases in this order:

1. input contains top-level `\:` -> infix-spec header
2. otherwise input contains top-level `\.` -> infix command header
3. otherwise input contains top-level `::` -> refined command header
4. otherwise -> simple command header

### Simple command headers

```text
SimpleCommandHeader ::= "\" RawChain CurlyHeadingArgs* CommandHeaderTail* ParenHeadingArgs*
CommandHeaderTail ::= (":" | ":?") RawChain CurlyHeadingArgs+
```

Notes:

- each tail part must have at least one `{...}` block
- `:?` marks a command-header tail part as optional; it is accepted only in command headers, not in command expressions
- optional tail parts expand to all ordered concrete signatures that include or omit that part
- zero or more parenthesized form-argument blocks may appear at the end

Examples:

- `\function`
- `\function:on{A}:to{B}`
- `\function:on{A}:?to{B}`
- `\foo:?baz{A}:?bar{B}`
- `\function:on{A}:to{B}(f(x_))`

#### Mapping-parameter command headers

An ordinary command header may associate one mapping-form curly group with one
mapping-parameter curly group:

```text
[\integral{f(x_, y_)}:d{f.x_}]
[\integral{f(x_, y_)}:d{f.u?_, f.v?_}]
[\integral{f(x_[i_:=1...n])}:d{f.x_[i_[j_:=1...m]]}]
```

The hand-written form parser represents these selectors explicitly as exact,
arbitrary, and variadic `MappingParameterSelector` variants. Header validation
requires:

- exactly one curly group containing selectors;
- only selectors in that group, all attributed to the same mapping owner;
- exactly one different curly group containing that owner's mapping form with
  explicit parameters;
- exact selector names to occur in the mapping form;
- arbitrary selector names to be fresh rather than names of exact parameters;
- a variadic selector's name and outer index to match the associated ranged
  variadic mapping parameter.

Mapping-parameter selectors are bound by the associated mapping form. They are
allowed, but not required, as `when:` subjects; the mapping owner remains an
ordinary header parameter for `when:` validation.

Mapping selectors are currently rejected in infix, infix-spec, and refined
headers. A valid ordinary header receives a specialized signature in addition
to its general command signature:

```text
f(x_, y_) + f.x_                 -> {_(2)} + {#1}
f(x_, y_) + f.u?_                -> {_(2)} + {#?}
f(x_[i_:=1...n]) + variadic set  -> {_(*)} + {#*}
```

For example, `\integral{f(x_, y_)}:d{f.x_}` becomes
`\integral{_(2)}:d{#1}` and has general signature `\integral:d`. Multiple
specialized signatures may share the general signature, but specialized
signatures themselves remain duplicate-checked.

At a use site, command mapping-literal sugar such as
`\integral[x_, y_ is \real]{x_^2+y_^2}:d{x_}` supplies the explicit mapping
parameters and selector positions. Selector arguments must all be names bound by
that mapping literal. Resolution ranks matching candidates by fixed arity before
variadic arity, then exact ordered positions before arbitrary `#?`, and finally
variadic `#*`. Equal best ranks are an ambiguity error.

### Infix command headers

```text
InfixCommandHeader ::= HeadingParameter? "\." RawChain CurlyHeadingArgs* CommandHeaderTail* "./" HeadingParameter?
```

Notes:

- infix command headers cannot have trailing `(...)` argument blocks
- the command core must start with `\.` and end with `./`
- a left operand requires a matching right operand, and vice versa
- standalone placeholder operands are normalized to named callable parameters,
  so `n_ \.natural.+./ m_` binds `n` and `m` without requiring separate
  `when:` declarations

### Infix-spec headers

```text
InfixSpecHeader ::= FormOrDeclaration "\:" InfixSpecHeaderBody ":/" FormOrDeclaration
```

Both operands are required. The declared spelling ends in `:/`; `?:/` is only
valid for predicate expressions. The body may use either an ordinary command
chain or the same parenthesized refinement prefix used by refined command
headers.

## Alias Syntax

### Writing aliases

```text
WritingAlias ::= FormOrDeclaration ":~>" RawNonEmptyText
```

Notes:

- the body is not parsed as formulation syntax
- it is whatever trimmed text appears after the first top-level `:~>`
- the body must be non-empty

### Expression aliases

```text
ExpressionAlias ::= ExpressionAliasLhs (":=>" | ":->") Expression
ExpressionAliasLhs ::= FormOrDeclaration
                     | SimpleCommandHeader
                     | InfixCommandHeader
                     | MemberAliasLhs
MemberAliasLhs ::= Name "." Name
                 | Name "." Name "(" PlaceholderList? ")"
```

Important implementation detail:

- refined command headers are explicitly rejected on the left-hand side

### Spec-operator aliases

```text
SpecOperatorAlias ::= PlaceholderSpecStatement ":->" SpecOperatorAliasTarget
SpecOperatorAliasTarget ::= IsOrSpec
                          | MemberOfExpression
                          | PlaceholderSpecStatement
                          | "\\" "\\" RawChain
MemberOfExpression ::= InfixCommandExpression "member_of" InfixCommandExpression
PlaceholderSpecStatement ::= PlaceholderForm TopLevelQuotedOperator Name
```

As with `parse_is_or_spec`, the quoted operator is extracted by raw scanning and may contain arbitrary text.
Built-in targets use two leading backslashes, for example `\\abstract`.

## Header Parsers

These are used mostly by the structural language.

### Label headers

```text
LabelHeader ::= DottedParts
```

### Author headers

```text
AuthorHeader ::= "@" DottedParts
```

### Resource headers

```text
ResourceHeader ::= "$" DottedParts (":page{" PositiveInteger "}")?
```

### Topic headers

```text
TopicHeader ::= "#" DottedParts
```

### Dotted parts

```text
DottedParts ::= NamePart ("." NamePart)*
NamePart ::= identifier-like name | stropped symbolic name
```

Unlike expression labels, these helper parsers allow stropped symbolic parts because they use raw parsing helpers.

## Top-Level Scanning Rules Used by Helper Parsers

Several helper parsers search for delimiters only at top level. The implementation ignores delimiters that occur:

- inside `(...)`
- inside `{...}`
- inside `[...]`
- inside double quotes
- inside backticks

This top-level scanning is used for:

- ` is `
- ` via `
- `::`
- `::=`
- `:=`
- `:=>`
- `:->`
- `:~>`
- comma splitting
- dot splitting
- delimiter finding while parsing chains and refined commands

## Compact Reference Grammar

This section is intentionally dense. It is the parser-oriented reference for
the current Rust formulation implementation.

### Parser roots

```text
InputExpression ::= Expression
InputDeclarationStatement ::= DeclarationStatement
InputHardCastStatement ::= HardCastStatement
InputExpressionBinding ::= ExpressionBinding
InputFormOrDeclaration ::= FormOrDeclaration
InputIsOrSpec ::= IsOrSpec
InputIsOrRefinedStatementSpec ::= IsOrRefinedStatementSpec
InputIsViaStatement ::= IsViaStatement
InputCommandHeader ::= CommandHeader
InputWritingAlias ::= WritingAlias
InputExpressionAlias ::= ExpressionAlias
InputSpecOperatorAlias ::= SpecOperatorAlias
InputLabelHeader ::= LabelHeader
InputAuthorHeader ::= AuthorHeader
InputResourceHeader ::= ResourceHeader
InputTopicHeader ::= TopicHeader
```

### Lexical terminals

```text
IdentifierName ::= [A-Za-z0-9](?:[A-Za-z0-9_']*[A-Za-z0-9'])?
BacktickName ::= "`" OperatorText "`"
Name ::= IdentifierName | BacktickName

Placeholder ::= IdentifierName "_"
MagneticPlaceholder ::= IdentifierName "__"

QuotedName ::= "\"" IdentifierName "\""
Label ::= "[:" IdentifierName ("." IdentifierName)* ":]"

NamedOperatorPath ::= NamedOperatorPart ("." NamedOperatorPart)*
NamedOperatorPart ::= IdentifierName | OperatorText
NamedOperator ::= "|" NamedOperatorPath "|"
                | ":|" NamedOperatorPath "|"
                | "|" NamedOperatorPath "|:"
                | ":|" NamedOperatorPath "|:"

PrefixFormNamedOperator ::= IdentifierName "|"
PostfixFormNamedOperator ::= "|" IdentifierName

SpecialOperator ::= operator token described under "Special operators" above
OperatorText ::= raw helper operator text described under "Special operators" above
Ellipsis ::= "..."
```

### Operators and reusable pieces

```text
AnyOperator ::= SpecialOperator | "+" | "-" | "*" | "/" | "=" | "^"
InfixFormOperator ::= AnyOperator | NamedOperator | "[" AnyOperator "]"
PrefixFormOperator ::= AnyOperator | PrefixFormNamedOperator
PostfixFormOperator ::= AnyOperator | PostfixFormNamedOperator

PlaceholderList ::= Placeholder ("," Placeholder)*
ExpressionList ::= Expression ("," Expression)*
FormList ::= FormOrDeclaration ("," FormOrDeclaration)*

TupleExpressionElement ::= SpecOrPredicateExpression | AnyOperator
TupleFormElement ::= FormOrDeclaration | AnyOperator

SubsetNameCall ::= Name "[" (Name | Placeholder) "]"
                 | Name "[" Name "," Name "]"
                 | Name "[" Name "[" Name "]" "]"

VariadicSlice ::= Name "..."
                | Name "[" ("0" | "1") "..." Name "]"
                | Name "[" ("0" | "1") "..." Placeholder "..." Name "]"
```

### Forms and declarations

```text
FormOrDeclaration ::=
    Name
  | MappingParameter
  | FunctionFormOrDeclaration
  | TupleFormOrDeclaration
  | SetFormOrDeclaration
  | Placeholder InfixFormOperator Placeholder
  | PrefixFormOperator Placeholder
  | Placeholder PostfixFormOperator

FunctionFormOrDeclaration ::= FunctionForm
                            | Name "::=" FunctionForm

FunctionForm ::= Name "(" MagneticPlaceholder ")"
               | Name "(" PlaceholderList ")"
               | Name "(" VariadicMappingParameter ")"

VariadicMappingParameter ::= (Placeholder | MagneticPlaceholder)
                             "[" Placeholder ":=" ("0" | "1") "..." Name "]"
MappingParameter ::= Name "." Placeholder
                   | Name "." Name "?" "_"
                   | Name "." Placeholder "[" Placeholder
                     "[" Placeholder ":=" ("0" | "1") "..." Name "]" "]"

TupleFormOrDeclaration ::= TupleForm
                         | Name "::=" TupleForm

TupleForm ::= "(" TupleFormElement "," TupleFormElement ("," TupleFormElement)* ")"

SetFormOrDeclaration ::= SetForm
                       | Name "::=" SetForm

SetForm ::= "{" PlaceholderForm [":" Ellipsis] "}"

PlaceholderForm ::= Placeholder
                  | Placeholder "(" PlaceholderList ")"
```

### Expressions

```text
Expression ::= MappingExpression

MappingExpression ::=
    SpecOrPredicateExpression "=>" MappingExpression
  | SpecOrPredicateExpression

SpecOrPredicateExpression ::=
    InfixCommandExpression InfixSpec InfixCommandExpression
  | InfixCommandExpression QuotedName Name
  | InfixCommandExpression QuotedName "?" Name
  | InfixCommandExpression QuotedName CommandExpression
  | InfixCommandExpression "is" PredicateTypeExpression
  | InfixCommandExpression ("is?" | "is_not?") (CommandExpression | BuiltinTypeExpression)
  | InfixCommandExpression "member_of" InfixCommandExpression
  | InfixCommandExpression "satisfies" InfixCommandExpression
  | "?" "is" PredicateTypeExpression
  | "?" QuotedName (Name | CommandExpression)
  | InfixCommandExpression

PredicateTypeExpression ::= CommandExpression | BuiltinTypeExpression | Name
BuiltinTypeExpression ::= "\\" "\\" Chain

InfixCommandExpression ::=
    InfixCommandExpression InfixCommand EqualityExpression
  | EqualityExpression

EqualityExpression ::=
    EqualityExpression ("=" | SpecialOperator) AdditiveExpression
  | AdditiveExpression

AdditiveExpression ::=
    AdditiveExpression ("+" | "-") MultiplicativeExpression
  | MultiplicativeExpression

MultiplicativeExpression ::=
    MultiplicativeExpression ("*" | "/") PowerExpression
  | PowerExpression

PowerExpression ::=
    HighPrecedenceExpression "^" PowerExpression
  | HighPrecedenceExpression

HighPrecedenceExpression ::=
    HighPrecedenceExpression NamedOperator UnaryExpression
  | UnaryExpression

UnaryExpression ::=
    PrefixOperator UnaryExpression
  | PostfixExpression

PostfixExpression ::= PrimaryExpression PostfixNamedOperator
                    | PrimaryExpression

PrimaryExpression ::=
    GroupedExpression Label
  | GroupedExpression
  | FunctionExpression
  | TupleExpression
  | SetExpression
  | SubsetExpression
  | VariadicSlice
  | CommandExpression
  | BuiltinCommandExpression
  | MemberExpression
  | CommandExpression ("@" | "@!") PrimaryExpression
  | Name "?"
  | MagneticPlaceholder
  | Placeholder
  | Name

GroupedExpression ::= "(" Expression ")"
                    | "(." Expression ".)"

FunctionExpression ::= Name "(" ExpressionList ")"
                     | Name "[|" FunctionNamedExpressionElement ("," FunctionNamedExpressionElement)* "|]"

FunctionNamedExpressionElement ::= FunctionNamedExpressionElementLhs (":=" | "=") SpecOrPredicateExpression
FunctionNamedExpressionElementLhs ::= Name | InferredParameterName | SubsetNameCall
                                    | RangedMappingSelector | "..."
InferredParameterName ::= Name "?_"
RangedMappingSelector ::= Placeholder "[" Placeholder
                          "[" ("0" | "1") "..." Name "]" "]"

TupleExpression ::= "(" TupleExpressionElement "," TupleExpressionElement ("," TupleExpressionElement)* ")"

SetExpression ::= "{" CollectionTarget ":" "..." "}"
                | "{" CollectionTarget ":" Expression (("," | ";") Expression)* ("|" SetPredicate)? "}"
CollectionTarget ::= SetTarget | Expression
SetPredicate ::= Expression | SetTarget ":=" Expression

SubsetExpression ::= SubsetNameCall
```

### Chains and command expressions

```text
Chain ::= ChainPart ("." ChainPart)*
ChainPart ::= Name | "$" Name | SpecialOperator | "="
RawChain ::= RawChainPart ("." RawChainPart)*
RawChainPart ::= Name | "$" Name | OperatorText

CurlyExpressionArgs ::= "{" ExpressionList "}"
ParenExpressionArgs ::= "(" ExpressionList ")"

CommandExpressionTailPart ::= ":" Chain CurlyExpressionArgs+
CommandExpressionTail ::= CommandExpressionTailPart*

CommandContext ::= ("#using" | "#given") "{" (CommandContextArgument (";" CommandContextArgument)*)? "}"
CommandContextArgument ::= Name ":=" Expression | DeclarationStatement | Expression | RawText

CommandExpression ::= "\" Chain CurlyExpressionArgs* CommandExpressionTail ParenExpressionArgs* CommandContext?

BuiltinCommandArgs ::= "{" SpecOrPredicateExpression (";" SpecOrPredicateExpression)* "}"
BuiltinCommandTail ::= ":" Chain BuiltinCommandArgs
BuiltinCommandExpression ::= "\\" "\\" Chain BuiltinCommandArgs? BuiltinCommandTail*

InfixCommand ::= "\." Chain CurlyExpressionArgs* CommandExpressionTail "./"
InfixSpec ::= "\:" Chain CurlyExpressionArgs* CommandExpressionTail (":/" | "?:/")
```

`RawChain` is used by scanner-based helpers such as command headers, refined
commands, and built-in spec-alias targets. Lexer-driven command expressions use
`Chain`. Label, author, and resource headers use dotted name parts rather than
full chains.

### Scanner-based statement helpers

These forms are parsed by `src/frontend/formulation/parser.rs`, not by the LALRPOP expression grammar.

```text
IsSubject ::= IsSubjectFormList | OperatorText
SpecSubject ::= FormOrDeclaration | OperatorText
IsSubjectForm ::= FormOrDeclaration | PlaceholderForm
IsSubjectFormList ::= IsSubjectForm ("," IsSubjectForm)*
TopLevelQuotedOperator ::= a top-level double-quoted string found by raw scanning

DeclarationStatement ::= DeclarationBody DeclarationRelation?
DeclarationBody ::=
    IsSubject
  | IsSubject "::=" IsSubject
  | IsSubject ":=" Expression
  | IsSubject "::=" IsSubject ":=" Expression
DeclarationRelation ::= " is " TypeExpression | TopLevelQuotedOperator Expression

ExpressionBinding ::= Expression ":=" Expression
HardCastStatement ::= IsSubject (":=" Expression)? " is! " TypeExpression

IsStatement ::= IsSubject " is " TypeExpression
SubjectSpecStatement ::= SpecSubject TopLevelQuotedOperator Name
PlaceholderSpecStatement ::= PlaceholderForm TopLevelQuotedOperator Name

IsOrSpec ::= IsStatement | SubjectSpecStatement

IsOrRefinedStatement ::= IsSubject " is " (CommandExpression | RefinedCommandExpression)
IsOrRefinedStatementSpec ::= IsOrRefinedStatement | SubjectSpecStatement

IsViaStatement ::= IsStatement " via " FormOrDeclaration
```

### Refined command helpers

```text
RefinedTail ::= "[[" Name "]]" | RawChain

RefinedExpressionPart ::= RawChain CommandExpressionTail
RefinedHeaderPart ::= RawChain CommandHeaderTail

RefinedCommandExpression ::=
    "\" [RawChain "."] "(" RefinedExpressionPart ("," RefinedExpressionPart)* ")" "::"
    RefinedTail CurlyExpressionArgs* CommandExpressionTail ParenExpressionArgs*

RefinedCommandHeader ::=
    "\" [RawChain "."] "(" RefinedHeaderPart ("," RefinedHeaderPart)* ")" "::"
    RefinedTail CurlyHeadingArgs* CommandHeaderTail ParenHeadingArgs*
```

Notes:

- the raw helper parsers require at least one refined part
- `CommandExpressionTail` and `CommandHeaderTail` may be empty as wholes
- each individual tail part, if present, must still contain one or more `{...}` blocks

### Command headers

```text
CommandHeader ::= SimpleCommandHeader | InfixCommandHeader | InfixSpecHeader | RefinedCommandHeader

CurlyHeadingArgs ::= "{" (FormList | VariadicParameter) "}"
VariadicParameter ::= Name "..." Name?
                    | Name "[" Placeholder ":=" ("0" | "1") "..." Name? "]"
ParenHeadingArgs ::= "(" HeadingParameterList ")"
HeadingParameterList ::= HeadingParameter ("," HeadingParameter)*
HeadingParameter ::= FormOrDeclaration | Placeholder

CommandHeaderTailPart ::= (":" | ":?") RawChain CurlyHeadingArgs+
CommandHeaderTail ::= CommandHeaderTailPart*

SimpleCommandHeader ::= "\" RawChain CurlyHeadingArgs* CommandHeaderTail ParenHeadingArgs*
InfixCommandHeader ::= HeadingParameter? "\." RawChain CurlyHeadingArgs* CommandHeaderTail "./" HeadingParameter?
InfixSpecHeader ::= FormOrDeclaration "\:" InfixSpecHeaderBody ":/" FormOrDeclaration
InfixSpecHeaderBody ::= RawChain CurlyHeadingArgs* CommandHeaderTail
                      | RefinedHeaderLeft "::" RawChain CurlyHeadingArgs* CommandHeaderTail
RefinedHeaderLeft ::= [RawChain "."] "(" RefinedHeaderPart ("," RefinedHeaderPart)* ")"
```

An infix-command header must provide either both operands or neither. An
infix-spec header always requires both operands and uses `:/`; `?:/` is reserved
for predicate use sites.

### Aliases and headers

```text
WritingAlias ::= FormOrDeclaration ":~>" RawNonEmptyText

ExpressionAliasLhs ::= FormOrDeclaration
                     | SimpleCommandHeader
                     | InfixCommandHeader
                     | MemberAliasLhs

MemberAliasLhs ::= Name "." Name
                 | Name "." Name "(" PlaceholderList? ")"

ExpressionAlias ::= ExpressionAliasLhs (":=>" | ":->") Expression
SpecOperatorAlias ::= PlaceholderSpecStatement ":->" SpecOperatorAliasTarget
SpecOperatorAliasTarget ::= IsOrSpec
                          | MemberOfExpression
                          | PlaceholderSpecStatement
                          | "\\" "\\" RawChain
MemberOfExpression ::= InfixCommandExpression "member_of" InfixCommandExpression

DottedParts ::= Name ("." Name)*
LabelHeader ::= DottedParts
AuthorHeader ::= "@" DottedParts
ResourceHeader ::= "$" DottedParts (":page{" PositiveInteger "}")?
TopicHeader ::= "#" DottedParts
```

### Deliberate omissions from the current implementation

The old grammar drafts implied several forms that the current code does not accept. In particular:

- a refined command expression is not a standalone `parse_expression` atom; it
  is accepted in refined type/declaration positions, refined predicates, and
  refined spec-infix bodies
- prefix and postfix expression operators are the named-operator token forms;
  arbitrary symbolic operators remain infix except for arithmetic unary `+`/`-`
- infix commands have their own precedence level below equality and arithmetic
- expression-level `is` accepts ordinary command, built-in, function, tuple,
  set, and refined type expressions; `is?` and `is_not?` accept ordinary,
  refined, or built-in command predicates

## Current Implementation Notes and Footguns

### `parse_expression` does not parse refined command expressions

Refined command expressions are accepted through refined declaration statements
and `parse_is_or_refined_statement_spec`.

### Helper `is` statements use different subject syntax than `is` expressions

In ordinary expressions, `x is \foo{A}` requires the right-hand side to be a command expression.

In declaration statements and `parse_is_or_spec`, the right-hand side of `is`
is also a command expression, but the left-hand side is parsed with helper
syntax that accepts forms, placeholder forms, comma-separated subject lists, and
operator subjects. Refined declaration statements and
`parse_is_or_refined_statement_spec` additionally accept refined command
expressions on the right-hand side.

### Quoted operator handling is inconsistent by design in the current code

- lexer-driven expression specs require identifier-like quoted names
- raw helper parsers accept any top-level quoted operator text

This is part of the current implementation and should be preserved unless the language is intentionally changed.

### One-element tuples are not supported

Both tuple expressions and tuple forms require at least two elements.

### Subset and variadic syntax are intentionally narrow

Ordinary subset calls accept the three hard-coded shapes above; a one-index
call also accepts a placeholder index such as `x[i_]`. Variadic slices accept
only zero- or one-based ranges. A slice is rejected as an ordinary operand and
is accepted only by the broadcast operations and variadic builtins documented
in `language.md`.

### Named-operator and infix-command precedence is left-associative

Ungrouped chains like `a |f| b |g| c` are accepted and associate to the left at
the named-operator level. Infix-command chains are also left-associative at
their separate, lower precedence level.

### Tail parts require `{...}`

For both command headers and command expressions, each `:tail` part must include at least one curly argument list. Command headers may spell a tail as `:?tail` to make that tail optional at reference sites; command expressions still use plain `:tail` for whichever optional parts are present.

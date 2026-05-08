# Formulation Syntax

This file describes the formulation language exactly as it is currently implemented in the Rust code.

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

| Parser function | Accepted root syntax |
| --- | --- |
| `parse_expression` | general expressions |
| `parse_form_or_declaration` | forms and declarations |
| `parse_is_or_spec` | `<subject> is <command-type>` or `<subject> "op" Name` |
| `parse_is_or_refined_statement_spec` | same as above, but `is` may target a refined command expression |
| `parse_is_via_statement` | `<is-statement> via <tuple-form>` |
| `parse_command_header` | simple, infix, or refined command headers |
| `parse_writing_alias` | `<form-or-declaration> :~> <raw body>` |
| `parse_expression_alias` | `<lhs> :=> <expression>` |
| `parse_spec_operator_alias` | `<placeholder-spec> :-> <is-or-spec>` |
| `parse_label_header` | dotted label header text |
| `parse_author_header` | `@` followed by dotted parts |
| `parse_resource_header` | `$` followed by dotted parts |

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

- a backtick name: `` `...` `` where anything except a backtick is allowed inside
- an identifier-like name matching:

```text
[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?
```

Consequences:

- names may start with a digit
- names may contain `_` internally
- names must end in an ASCII letter or digit
- `_x`, `x_`, and `x__` are not normal names

Examples of valid normal names:

- `x`
- `x_1`
- `123`
- `` `x + y` ``

### Placeholders

A placeholder is an identifier-like name followed by `_`:

```text
[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?_
```

A magnetic placeholder is the same base name followed by `__`:

```text
[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?__
```

Examples:

- `x_`
- `value_`
- `x__`

### Quoted names

The lexer token `QuotedName` is restricted. It is not an arbitrary string literal. It must match:

```text
"[A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?"
```

So these are valid in lexer-driven expression parsing:

- `"in"`
- `"maps_to"`

These are not valid there:

- `"less than"`
- `"x+y"`

Important implementation distinction:

- expression parsing uses this strict `QuotedName` token
- `parse_is_or_spec` and `parse_spec_operator_alias` do not use that token for the quoted operator
- those helper parsers accept any top-level `"..."`

So `x "less than" A` is rejected by `parse_expression`, but accepted by `parse_is_or_spec`.

### Labels inside expressions

Expression labels use the token form:

```text
[:part(.part)*:]
```

Each `part` uses the same identifier-like rule as normal non-backtick names.

Important distinction:

- expression labels like `[:a.b:]` are lexer tokens and do not support backtick parts
- structural label headers parsed by `parse_label_header` do support backtick name parts because they use the raw helper parser, not the lexer token

### Named operators

The lexer recognizes four infix named-operator spellings:

- plain: `|name|`
- left-colon: `:|name|`
- right-colon: `|name|:`
- both-colon: `:|name|:`

For form declarations, it also recognizes:

- prefix-form operator: `name|`
- postfix-form operator: `|name`

`name` uses the identifier-like rule, not the backtick rule in all of these spellings.

### Special operators

The lexer recognizes:

- multi-character operator strings made from `-~!#%^&*+=|<>/`
- single-character operators from `~!#%&<>`
- the individual punctuation tokens `+ - * / = ^` are also tokens in their own right

The raw helper parser also treats any non-empty string made only from `-~!#%^&*\\+=|<>/` as operator text.

### Reserved keywords and punctuation

The lexer has dedicated tokens for:

- `is`
- `is?`
- `is_not?`
- `via`
- `:=`
- `:=>`
- `:->`
- `:~>`
- `\`
- `\:`
- `:/`
- `(.`
- `.)`
- `[|`
- `|]`
- `(` `)` `{` `}` `[` `]` `,` `:` `.` `|` `$`

Because these are tokenized before ordinary names, exact spellings like `is` and `via` are effectively reserved in lexer-driven formulation parsing.

## Expression Grammar

### High-level precedence

From lowest precedence to highest:

1. spec and predicate forms
2. equality `=`
3. additive `+ -`
4. multiplicative `* /`
5. power `^`
6. named-operator / infix-command forms
7. unary `+ -`
8. atomic forms

Associativity:

- `=` is left-associative
- `+` and `-` are left-associative
- `*` and `/` are left-associative
- `^` is right-associative
- named-operator and infix-command expressions are not chained by the grammar; only one such operator is accepted at that precedence level without extra grouping

### Grammar

```text
Expression ::= SpecOrPredicateExpression

SpecOrPredicateExpression ::=
    EqualityExpression QuotedName Name
  | EqualityExpression "is" EqualityExpression
  | EqualityExpression "is?" EqualityExpression
  | EqualityExpression "is_not?" EqualityExpression
  | EqualityExpression

EqualityExpression ::=
    EqualityExpression "=" AdditiveExpression
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
    PrefixExpression NamedOperator PrefixExpression
  | PrefixExpression InfixCommand PrefixExpression
  | PrefixExpression

PrefixExpression ::=
    ("+" | "-") AtomExpression
  | AtomExpression

AtomExpression ::=
    LabeledExpression
  | GroupedExpression
  | FunctionExpression
  | TupleExpression
  | SetExpression
  | SubsetExpression
  | CommandExpression
  | Name
```

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

Only grouped expressions may be labeled directly.

Examples:

- `(x + y)[:sum:]`
- `(. x + y .)[:normalized:]`

#### Function expressions

```text
FunctionExpression ::=
    Name "(" Expression ("," Expression)* ")"
  | Name "[|" NamedFunctionElement ("," NamedFunctionElement)* "|]"

NamedFunctionElement ::= NamedFunctionLhs ":=" Expression
NamedFunctionLhs ::= Name | SubsetNameCall
```

Examples:

- `f(x, y)`
- `map[| key := x, value := y |]`

#### Tuple expressions

```text
TupleExpression ::= "(" TupleExpressionElement "," TupleExpressionElement ("," TupleExpressionElement)* ")"
TupleExpressionElement ::= Expression | Operator
```

Important implementation detail:

- tuples must have at least two elements
- a one-element tuple is not supported

Operators may appear as tuple elements, for example `(+, x)`.

#### Set expressions

```text
SetExpression ::= "{" ExpressionSpec ":" PlaceholderForm ("|" Expression)? "}"
ExpressionSpec ::= EqualityExpression QuotedName Name
```

Examples:

- `{x "in" A : x_}`
- `{x "in" A : x_ | x = y}`

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
InfixCommand ::= "\:" Chain CurlyExpressionArgs* CommandExpressionTail* ":/"
```

This syntax is only produced inside higher-precedence binary expressions and certain command-header contexts.

### Chains

Many command-related syntaxes use a `Chain`:

```text
Chain ::= ChainPart ("." ChainPart)*
ChainPart ::= Name | "$" Name | OperatorText
```

Examples:

- `function`
- `binary.op`
- `$alias`
- `<=`

Named operators like `|plus|` are not chain parts.

## Forms and Declarations

### Grammar

```text
FormOrDeclaration ::=
    Name
  | FunctionFormOrDeclaration
  | TupleFormOrDeclaration
  | SetFormOrDeclaration
  | Placeholder InfixFormOperator Placeholder
  | PrefixFormOperator Placeholder
  | Placeholder PostfixFormOperator

FunctionFormOrDeclaration ::= [Name ":="] FunctionForm
FunctionForm ::= Name "(" MagneticPlaceholder ")"
               | Name "(" Placeholder ("," Placeholder)* ")"

TupleFormOrDeclaration ::= [Name ":="] TupleForm
TupleForm ::= "(" TupleFormElement "," TupleFormElement ("," TupleFormElement)* ")"
TupleFormElement ::= FormOrDeclaration | Operator

SetFormOrDeclaration ::= [Name ":="] SetForm
SetForm ::= "{" PlaceholderForm "}"

PlaceholderForm ::= Placeholder
                  | Placeholder "(" Placeholder ("," Placeholder)* ")"
```

### Notes

- function forms support either:
  - exactly one magnetic placeholder, or
  - one or more ordinary placeholders
- mixed ordinary and magnetic placeholders are not allowed
- tuple forms also require at least two elements
- unnamed function/tuple/set forms are still represented internally as declaration variants with `name: None`

Examples:

- `x`
- `f(x_)`
- `g := f(x_, y_)`
- `(x_, y_)`
- `Pair := (x_, y_)`
- `{x_}`
- `Set := {x_}`
- `x_ |plus| y_`
- `neg| x_`
- `x_ |prime`

## Statement-Like Form Parsers

These are not part of `parse_expression`. They are helper parsers built in `src/frontend/formulation/parser.rs`.

### `parse_is_or_spec`

Accepted shapes:

```text
IsOrSpec ::= IsStatement | SubjectSpecStatement
IsStatement ::= SpecSubject " is " TypeExpression
SubjectSpecStatement ::= SpecSubject TopLevelQuotedOperator Name
SpecSubject ::= FormOrDeclaration | OperatorText
TypeExpression ::= CommandExpression
```

Notes:

- the parser looks for the exact top-level substring ` is ` with spaces around it
- the right-hand side of `is` must parse as a command expression, not a general expression
- if no top-level ` is ` is found, the parser falls back to the quoted-operator spec form
- the quoted operator is extracted by raw scanning, so it may contain spaces or punctuation

Examples:

- `f(x_) is \function:on{A}:to{B}`
- `+ is \operator`
- `x "in" A`
- `x "less than" A`

### `parse_is_or_refined_statement_spec`

Same as `parse_is_or_spec`, except:

```text
TypeExpression ::= CommandExpression | RefinedCommandExpression
```

### `parse_is_via_statement`

Accepted shape:

```text
IsViaStatement ::= IsStatement " via " TupleForm
```

Notes:

- the parser looks for the exact top-level substring ` via ` with spaces around it
- the left side must be an `is` statement, not a quoted-operator spec
- the tuple on the right must be a tuple form, not a tuple expression

## Refined Command Syntax

Refined command syntax is implemented by custom helper parsers, not by the LALRPOP expression grammar.

### Refined command expressions

```text
RefinedCommandExpression ::=
    "\" RefinedLeft "::" RefinedTail CurlyExpressionArgs* CommandExpressionTail* ParenExpressionArgs*

RefinedLeft ::= [Chain "."] "(" RefinedExpressionPart ("," RefinedExpressionPart)* ")"
RefinedExpressionPart ::= Chain CommandExpressionTail*
RefinedTail ::= "[[" Name "]]" | Chain
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

RefinedHeaderLeft ::= [Chain "."] "(" RefinedHeaderPart ("," RefinedHeaderPart)* ")"
RefinedHeaderPart ::= Chain CommandHeaderTail*
RefinedTail ::= "[[" Name "]]" | Chain

CurlyHeadingArgs ::= "{" FormOrDeclaration ("," FormOrDeclaration)* "}"
ParenHeadingArgs ::= "(" FormOrDeclaration ("," FormOrDeclaration)* ")"
CommandHeaderTail ::= ":" Chain CurlyHeadingArgs+
```

## Command Header Syntax

`parse_command_header` chooses among three cases in this order:

1. input starts with `\:` -> infix command header
2. otherwise input contains top-level `::` -> refined command header
3. otherwise -> simple command header

### Simple command headers

```text
SimpleCommandHeader ::= "\" Chain CurlyHeadingArgs* CommandHeaderTail* ParenHeadingArgs*
CommandHeaderTail ::= ":" Chain CurlyHeadingArgs+
```

Notes:

- each tail part must have at least one `{...}` block
- zero or more parenthesized form-argument blocks may appear at the end

Examples:

- `\function`
- `\function:on{A}:to{B}`
- `\function:on{A}:to{B}(f(x_))`

### Infix command headers

```text
InfixCommandHeader ::= "\:" Chain CurlyHeadingArgs* CommandHeaderTail* ":/"
```

Notes:

- infix command headers cannot have trailing `(...)` argument blocks
- the spelling must start with `\:` and end with `:/`

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
ExpressionAlias ::= ExpressionAliasLhs ":=>" Expression
ExpressionAliasLhs ::= FormOrDeclaration | SimpleCommandHeader | InfixCommandHeader
```

Important implementation detail:

- refined command headers are explicitly rejected on the left-hand side

### Spec-operator aliases

```text
SpecOperatorAlias ::= PlaceholderSpecStatement ":->" IsOrSpec
PlaceholderSpecStatement ::= PlaceholderForm TopLevelQuotedOperator Name
```

As with `parse_is_or_spec`, the quoted operator is extracted by raw scanning and may contain arbitrary text.

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
ResourceHeader ::= "$" DottedParts
```

### Dotted parts

```text
DottedParts ::= NamePart ("." NamePart)*
NamePart ::= identifier-like name | backtick name
```

Unlike expression labels, these helper parsers allow backtick parts because they use raw parsing helpers.

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
- `:=>`
- `:->`
- `:~>`
- comma splitting
- dot splitting
- delimiter finding while parsing chains and refined commands

## Compact Reference Grammar

This section is intentionally dense. It is the closest thing in this file to a parser reference and is meant to play the same role that `old_docs/syntax.lark` used to play, but corrected to match the current Rust implementation.

### Parser roots

```text
InputExpression ::= Expression
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
```

### Lexical terminals

```text
IdentifierName ::= [A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?
BacktickName ::= "`" (anything except "`")* "`"
Name ::= IdentifierName | BacktickName

Placeholder ::= IdentifierName "_"
MagneticPlaceholder ::= IdentifierName "__"

QuotedName ::= "\"" IdentifierName "\""
Label ::= "[:" IdentifierName ("." IdentifierName)* ":]"

NamedOperator ::= "|" IdentifierName "|"
                | ":|" IdentifierName "|"
                | "|" IdentifierName "|:"
                | ":|" IdentifierName "|:"

PrefixFormNamedOperator ::= IdentifierName "|"
PostfixFormNamedOperator ::= "|" IdentifierName

SpecialOperator ::= token matched by (?:[-~!#%^&*+=|<>/]{2,}|[~!#%&<>])
OperatorText ::= non-empty raw string whose characters are all in -~!#%^&*\+=|<>/
```

### Operators and reusable pieces

```text
AnyOperator ::= SpecialOperator | "+" | "-" | "*" | "/" | "=" | "^"
InfixFormOperator ::= AnyOperator | NamedOperator
PrefixFormOperator ::= AnyOperator | PrefixFormNamedOperator
PostfixFormOperator ::= AnyOperator | PostfixFormNamedOperator

PlaceholderList ::= Placeholder ("," Placeholder)*
ExpressionList ::= Expression ("," Expression)*
FormList ::= FormOrDeclaration ("," FormOrDeclaration)*

TupleExpressionElement ::= Expression | AnyOperator
TupleFormElement ::= FormOrDeclaration | AnyOperator

SubsetNameCall ::= Name "[" Name "]"
                 | Name "[" Name "," Name "]"
                 | Name "[" Name "[" Name "]" "]"
```

### Forms and declarations

```text
FormOrDeclaration ::=
    Name
  | FunctionFormOrDeclaration
  | TupleFormOrDeclaration
  | SetFormOrDeclaration
  | Placeholder InfixFormOperator Placeholder
  | PrefixFormOperator Placeholder
  | Placeholder PostfixFormOperator

FunctionFormOrDeclaration ::= FunctionForm
                            | Name ":=" FunctionForm

FunctionForm ::= Name "(" MagneticPlaceholder ")"
               | Name "(" PlaceholderList ")"

TupleFormOrDeclaration ::= TupleForm
                         | Name ":=" TupleForm

TupleForm ::= "(" TupleFormElement "," TupleFormElement ("," TupleFormElement)* ")"

SetFormOrDeclaration ::= SetForm
                       | Name ":=" SetForm

SetForm ::= "{" PlaceholderForm "}"

PlaceholderForm ::= Placeholder
                  | Placeholder "(" PlaceholderList ")"
```

### Expressions

```text
Expression ::= SpecOrPredicateExpression

SpecOrPredicateExpression ::=
    EqualityExpression QuotedName Name
  | EqualityExpression "is" EqualityExpression
  | EqualityExpression "is?" EqualityExpression
  | EqualityExpression "is_not?" EqualityExpression
  | EqualityExpression

EqualityExpression ::=
    EqualityExpression "=" AdditiveExpression
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
    PrefixExpression NamedOperator PrefixExpression
  | PrefixExpression InfixCommand PrefixExpression
  | PrefixExpression

PrefixExpression ::=
    ("+" | "-") AtomExpression
  | AtomExpression

AtomExpression ::=
    GroupedExpression Label
  | GroupedExpression
  | FunctionExpression
  | TupleExpression
  | SetExpression
  | SubsetExpression
  | CommandExpression
  | Name

GroupedExpression ::= "(" Expression ")"
                    | "(." Expression ".)"

FunctionExpression ::= Name "(" ExpressionList ")"
                     | Name "[|" FunctionNamedExpressionElement ("," FunctionNamedExpressionElement)* "|]"

FunctionNamedExpressionElement ::= FunctionNamedExpressionElementLhs ":=" Expression
FunctionNamedExpressionElementLhs ::= Name | SubsetNameCall

TupleExpression ::= "(" TupleExpressionElement "," TupleExpressionElement ("," TupleExpressionElement)* ")"

SetExpression ::= "{" ExpressionSpec ":" PlaceholderForm ("|" Expression)? "}"
ExpressionSpec ::= EqualityExpression QuotedName Name

SubsetExpression ::= SubsetNameCall
```

### Chains and command expressions

```text
Chain ::= ChainPart ("." ChainPart)*
ChainPart ::= Name | "$" Name | OperatorText

CurlyExpressionArgs ::= "{" ExpressionList "}"
ParenExpressionArgs ::= "(" ExpressionList ")"

CommandExpressionTailPart ::= ":" Chain CurlyExpressionArgs+
CommandExpressionTail ::= CommandExpressionTailPart*

CommandExpression ::= "\" Chain CurlyExpressionArgs* CommandExpressionTail ParenExpressionArgs*

InfixCommand ::= "\:" Chain CurlyExpressionArgs* CommandExpressionTail ":/"
```

### Scanner-based statement helpers

These forms are parsed by `src/frontend/formulation/parser.rs`, not by the LALRPOP expression grammar.

```text
SpecSubject ::= FormOrDeclaration | OperatorText
TopLevelQuotedOperator ::= a top-level double-quoted string found by raw scanning

IsStatement ::= SpecSubject " is " CommandExpression
SubjectSpecStatement ::= SpecSubject TopLevelQuotedOperator Name
PlaceholderSpecStatement ::= PlaceholderForm TopLevelQuotedOperator Name

IsOrSpec ::= IsStatement | SubjectSpecStatement

IsOrRefinedStatement ::= SpecSubject " is " (CommandExpression | RefinedCommandExpression)
IsOrRefinedStatementSpec ::= IsOrRefinedStatement | SubjectSpecStatement

IsViaStatement ::= IsStatement " via " TupleForm
```

### Refined command helpers

```text
RefinedTail ::= "[[" Name "]]" | Chain

RefinedExpressionPart ::= Chain CommandExpressionTail
RefinedHeaderPart ::= Chain CommandHeaderTail

RefinedCommandExpression ::=
    "\" [Chain "."] "(" RefinedExpressionPart ("," RefinedExpressionPart)* ")" "::"
    RefinedTail CurlyExpressionArgs* CommandExpressionTail ParenExpressionArgs*

RefinedCommandHeader ::=
    "\" [Chain "."] "(" RefinedHeaderPart ("," RefinedHeaderPart)* ")" "::"
    RefinedTail CurlyHeadingArgs* CommandHeaderTail ParenHeadingArgs*
```

Notes:

- the raw helper parsers require at least one refined part
- `CommandExpressionTail` and `CommandHeaderTail` may be empty as wholes
- each individual tail part, if present, must still contain one or more `{...}` blocks

### Command headers

```text
CommandHeader ::= SimpleCommandHeader | InfixCommandHeader | RefinedCommandHeader

CurlyHeadingArgs ::= "{" FormList "}"
ParenHeadingArgs ::= "(" FormList ")"

CommandHeaderTailPart ::= ":" Chain CurlyHeadingArgs+
CommandHeaderTail ::= CommandHeaderTailPart*

SimpleCommandHeader ::= "\" Chain CurlyHeadingArgs* CommandHeaderTail ParenHeadingArgs*
InfixCommandHeader ::= "\:" Chain CurlyHeadingArgs* CommandHeaderTail ":/"
```

### Aliases and headers

```text
WritingAlias ::= FormOrDeclaration ":~>" RawNonEmptyText

ExpressionAliasLhs ::= FormOrDeclaration
                     | SimpleCommandHeader
                     | InfixCommandHeader

ExpressionAlias ::= ExpressionAliasLhs ":=>" Expression
SpecOperatorAlias ::= PlaceholderSpecStatement ":->" IsOrSpec

DottedParts ::= Name ("." Name)*
LabelHeader ::= DottedParts
AuthorHeader ::= "@" DottedParts
ResourceHeader ::= "$" DottedParts
```

### Deliberate omissions from the current implementation

The old grammar drafts implied several forms that the current code does not accept. In particular:

- `parse_expression` does not accept refined command expressions
- there are no general prefix or postfix non-arithmetic operator expressions
- infix command expressions are not a separate root form; they only appear through `HighPrecedenceExpression`
- predicate expressions use general equality expressions on the right-hand side, not just commands

## Current Implementation Notes and Footguns

### `parse_expression` does not parse refined command expressions

Refined command expressions are only accepted through `parse_is_or_refined_statement_spec`.

### `is` statements in helper parsers are stricter than `is` expressions

In ordinary expressions, `x is y` is parsed as a general `TypeBinary` expression whose right-hand side is any equality expression.

In `parse_is_or_spec`, the same surface `is` form requires the right-hand side to be a command expression.

### Quoted operator handling is inconsistent by design in the current code

- lexer-driven expression specs require identifier-like quoted names
- raw helper parsers accept any top-level quoted operator text

This is part of the current implementation and should be preserved unless the language is intentionally changed.

### One-element tuples are not supported

Both tuple expressions and tuple forms require at least two elements.

### Subset syntax is intentionally narrow

Only the three hard-coded name-only shapes are accepted.

### Named-operator precedence is single-step

The grammar does not allow an ungrouped chain like `a |f| b |g| c`.

### Tail parts require `{...}`

For both command headers and command expressions, each `:tail` part must include at least one curly argument list.

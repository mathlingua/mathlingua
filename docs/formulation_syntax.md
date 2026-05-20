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

| Parser function | Accepted root syntax |
| --- | --- |
| `parse_expression` | general expressions |
| `parse_expression_binding` | `<expression> := <expression>` |
| `parse_form_or_declaration` | forms and declarations |
| `parse_is_or_spec` | `<is-subject> is <command-type>` or `<subject> "op" Name` |
| `parse_is_or_refined_statement_spec` | same as above, but `is` may target a refined command expression |
| `parse_is_via_statement` | `<is-statement> via <form-or-declaration>` |
| `parse_command_header` | simple, infix, or refined command headers |
| `parse_writing_alias` | `<form-or-declaration> :~> <raw body>` |
| `parse_expression_alias` | `<lhs> :=> <expression>` |
| `parse_spec_operator_alias` | `<placeholder-spec> :-> (<is-or-spec> | builtin)` |
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

- a stropped symbolic name: `` `...` `` where the inside text is non-empty and uses only operator characters from `-~!#%^&*\+=|<>/`
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
- `` `*` ``
- `` `*+` ``

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

Each `part` uses the same identifier-like rule as ordinary non-stropped names.

Important distinction:

- expression labels like `[:a.b:]` are lexer tokens and do not support stropped symbolic parts
- structural label headers parsed by `parse_label_header` do support stropped symbolic name parts because they use the raw helper parser, not the lexer token

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
2. equality `=` and special binary operators such as `<`, `>`, `<=`, `>=`
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
- named-operator and infix-command expressions are left-associative at their precedence level

### Grammar

```text
Expression ::= SpecOrPredicateExpression

SpecOrPredicateExpression ::=
    EqualityExpression QuotedName Name
  | EqualityExpression "is" CommandExpression
  | EqualityExpression "is?" CommandExpression
  | EqualityExpression "is_not?" CommandExpression
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
  | HighPrecedenceExpression InfixCommand UnaryExpression
  | UnaryExpression

UnaryExpression ::=
    ("+" | "-") UnaryExpression
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
ChainPart ::= Name | "$" Name | SpecialOperator
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

### `parse_expression_binding`

Accepted shape:

```text
ExpressionBinding ::= Expression ":=" Expression
```

Notes:

- the parser looks for the first top-level `:=`
- both sides are trimmed and parsed with `parse_expression`
- nested `:=` inside parentheses, braces, brackets, quotes, or backticks is ignored while finding the split point

Examples:

- `A := B`
- `f(x) := y + z`

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
TypeExpression ::= CommandExpression
```

Notes:

- the parser looks for the exact top-level substring ` is ` with spaces around it
- the left-hand side of `is` may be a single form, a single placeholder form, a comma-separated list mixing those, or an operator
- the right-hand side of `is` must parse as a command expression, not a general expression
- if no top-level ` is ` is found, the parser falls back to the quoted-operator spec form
- the quoted operator is extracted by raw scanning, so it may contain spaces or punctuation

Examples:

- `f(x_) is \function:on{A}:to{B}`
- `f(x_), y_ is \function:on{A}:to{B}`
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
ParenHeadingArgs ::= "(" FormOrDeclaration ("," FormOrDeclaration)* ")"
CommandHeaderTail ::= ":" RawChain CurlyHeadingArgs+
```

## Command Header Syntax

`parse_command_header` chooses among three cases in this order:

1. input starts with `\:` -> infix command header
2. otherwise input contains top-level `::` -> refined command header
3. otherwise -> simple command header

### Simple command headers

```text
SimpleCommandHeader ::= "\" RawChain CurlyHeadingArgs* CommandHeaderTail* ParenHeadingArgs*
CommandHeaderTail ::= ":" RawChain CurlyHeadingArgs+
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
InfixCommandHeader ::= "\:" RawChain CurlyHeadingArgs* CommandHeaderTail* ":/"
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
SpecOperatorAlias ::= PlaceholderSpecStatement ":->" SpecOperatorAliasTarget
SpecOperatorAliasTarget ::= IsOrSpec | "\\" RawChain
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
ResourceHeader ::= "$" DottedParts
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
```

### Lexical terminals

```text
IdentifierName ::= [A-Za-z0-9]+(?:[A-Za-z0-9_]*[A-Za-z0-9]+)?
BacktickName ::= "`" OperatorText "`"
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
  | EqualityExpression "is" CommandExpression
  | EqualityExpression "is?" CommandExpression
  | EqualityExpression "is_not?" CommandExpression
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
  | HighPrecedenceExpression InfixCommand UnaryExpression
  | UnaryExpression

UnaryExpression ::=
    ("+" | "-") UnaryExpression
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
ChainPart ::= Name | "$" Name | SpecialOperator
RawChain ::= RawChainPart ("." RawChainPart)*
RawChainPart ::= Name | "$" Name | OperatorText

CurlyExpressionArgs ::= "{" ExpressionList "}"
ParenExpressionArgs ::= "(" ExpressionList ")"

CommandExpressionTailPart ::= ":" Chain CurlyExpressionArgs+
CommandExpressionTail ::= CommandExpressionTailPart*

CommandExpression ::= "\" Chain CurlyExpressionArgs* CommandExpressionTail ParenExpressionArgs*

InfixCommand ::= "\:" Chain CurlyExpressionArgs* CommandExpressionTail ":/"
```

`RawChain` is used by scanner-based helpers such as command headers, refined
commands, and built-in spec-alias targets. Lexer-driven command expressions use
`Chain`. Label, author, and resource headers use dotted name parts rather than
full chains.

### Scanner-based statement helpers

These forms are parsed by `src/frontend/formulation/parser.rs`, not by the LALRPOP expression grammar.

```text
ExpressionBinding ::= Expression ":=" Expression

IsSubject ::= IsSubjectFormList | OperatorText
SpecSubject ::= FormOrDeclaration | OperatorText
IsSubjectForm ::= FormOrDeclaration | PlaceholderForm
IsSubjectFormList ::= IsSubjectForm ("," IsSubjectForm)*
TopLevelQuotedOperator ::= a top-level double-quoted string found by raw scanning

IsStatement ::= IsSubject " is " CommandExpression
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
CommandHeader ::= SimpleCommandHeader | InfixCommandHeader | RefinedCommandHeader

CurlyHeadingArgs ::= "{" FormList "}"
ParenHeadingArgs ::= "(" FormList ")"

CommandHeaderTailPart ::= ":" RawChain CurlyHeadingArgs+
CommandHeaderTail ::= CommandHeaderTailPart*

SimpleCommandHeader ::= "\" RawChain CurlyHeadingArgs* CommandHeaderTail ParenHeadingArgs*
InfixCommandHeader ::= "\:" RawChain CurlyHeadingArgs* CommandHeaderTail ":/"
```

### Aliases and headers

```text
WritingAlias ::= FormOrDeclaration ":~>" RawNonEmptyText

ExpressionAliasLhs ::= FormOrDeclaration
                     | SimpleCommandHeader
                     | InfixCommandHeader

ExpressionAlias ::= ExpressionAliasLhs ":=>" Expression
SpecOperatorAlias ::= PlaceholderSpecStatement ":->" SpecOperatorAliasTarget
SpecOperatorAliasTarget ::= IsOrSpec | "\\" RawChain

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
- expression-level `is`, `is?`, and `is_not?` forms require a command expression on the right-hand side

## Current Implementation Notes and Footguns

### `parse_expression` does not parse refined command expressions

Refined command expressions are only accepted through `parse_is_or_refined_statement_spec`.

### Helper `is` statements use different subject syntax than `is` expressions

In ordinary expressions, `x is \foo{A}` requires the right-hand side to be a command expression.

In `parse_is_or_spec`, the right-hand side is also a command expression, but the left-hand side is parsed with helper syntax that accepts forms, placeholder forms, comma-separated subject lists, and operator subjects. `parse_is_or_refined_statement_spec` additionally accepts refined command expressions on the right-hand side.

### Quoted operator handling is inconsistent by design in the current code

- lexer-driven expression specs require identifier-like quoted names
- raw helper parsers accept any top-level quoted operator text

This is part of the current implementation and should be preserved unless the language is intentionally changed.

### One-element tuples are not supported

Both tuple expressions and tuple forms require at least two elements.

### Subset syntax is intentionally narrow

Only the three hard-coded name-only shapes are accepted.

### Named-operator and infix-command precedence is left-associative

Ungrouped chains like `a |f| b |g| c` are accepted and associate to the left at the named-operator/infix-command precedence level.

### Tail parts require `{...}`

For both command headers and command expressions, each `:tail` part must include at least one curly argument list.

# MathLingua Specification

## Syntax

### Shared
```
NameAssignmentItem ::=
    Name |
    OperatorName |
    Tuple |
    Sequence |
    Function |
    Set

NameAssignment ::=
    Name ":=" NameAssignmentItem

FunctionAssignment ::=
    Function ":=" Function

Assignment ::=
    NameAssignment |
    FunctionAssignment

Name ::=
    [a-zA-Z0-9'"`]+("_"[a-zA-Z0-9'"`]+)?

OperatorName ::=
    [~!@#$%^&*-+=|<>?'`"]+("_"[a-zA-Z0-9'"`]+)?

VariadicName ::=
    (Name "..."?)

List<VariadicName> ::=
    VariadicName ("," VariadicName)*

Function ::=
    Name "(" List<VariadicName> ")"

SubParamCall ::=
    Name "_" "(" List<VariadicName> ")"

SubAndRegularParamCall ::=
    Name "_" "(" List<VariadicName> ")" "(" List<VariadicName> ")"

FunctionCall ::=
    Function |
    SubParamCall |
    SubAndRegularParamCall

SubParamSequence ::=
    "{" SubParamCall "}" "_" "(" List<VariadicName> ")"

SubAndRegularParamSequence ::=
    "{" SubAndRegularParamCall "}" "_" "(" List<VariadicName> ")"

Sequence ::=
    SubParamFunctionSequence |
    SubAndRegularParamFunctionSequence

Tuple ::=
    "(" Target ("," Target)* ")"

NameOrNameAssignment ::=
    Name |
    NameAssignment

Set ::=
    "{" NameOrNameAssignment ("," NameOrNameAssignment)* "}"

Target ::=
    Assignment |
    Name |
    OperatorName |
    Tuple |
    Sequence |
    Function |
    Set
```

-----------------------------------------------------------------------------------

### Phase 1
```
Argument ::=
    Target |
    Text[.*] |
    Statement[.*]

Text[x] ::=
    "\"" x "\""    with " escaped with \"

TextBlock ::=
    "::" .* "::"   with :: escaped with {::}

Statement[x] ::=
    ("'" x "'"     without an escape for ') |
    ("`" x "`"     without an escape for `)
                   where x is any character except the surrounding
                   characters unless escaped

Id ::=
    "[" (CommandForm |
        (Name InfixCommandForm Name)) |
        (OperatorName Name) |
        (Name OperatorName) |
        (Name OperatorName Name) "]"
```

Structure:
````
[Id]
name1: arg, arg
. arg, arg
name2: ...
. name3: ...
```

-----------------------------------------------------------------------------------

### TexTalk

#### Keywords:
```
is
in
notin
as
```

Syntax:
```
SquareTargetItem ::=
    Name |
    Tuple |
    Sequence |
    Function |
    Set

List<Expression> ::=
    Expression ("," Expression)*

CommandExpression ::=
    "\" (Name ".")* Name
    ("_" "{" List<Expression>+ "}" "^" "{" List<Expression>+ "}"
    "[" (SquareTargetItem+) | (Name "...") "]")?
    ("{" List<Expression>+ "}")?
    (":" "{" List<Expression>+ "}")*
    ("(" List<Expression>+ ")")?

CommandForm ::=
    "\" (Name ".")* Name
    ("_" "{" List<VariadicName>+ "}" "^" "{" List<VariadicName>+ "}"
    "[" (SquareTargetItem+) | (Name "...") "]")?
    ("{" List<VariadicName>+ "}")?
    (":" "{" List<VariadicName>+ "}")*
    ("(" List<VariadicName>+ ")")?

InfixCommandExpression ::=
    CommandExpression "/"

InfixCommandForm ::=
    CommandForm "/"

List<Target> ::=
    Target ("," Target)*

NameOrCommand ::=
    Name |
    CommandExpression

List<NameOrCommand> ::=
    NameOrCommand ("," NameOrCommand)*

VariadicIsRhs ::=
    VariadicName |
    CommandExpression

VariadicIsExpression ::=
    VariadicTarget "is" VariadicIsRhs

IsExpression ::=
    List<Target> "is" List<NameOrCommand>

MetaIsFormItem ::=
    "statement" |
    "assignment" |
    "specification" |
    "expression" |
    "definition"

MetaIsForm ::=
    "[:" MetaIsFormItem ("," MetaIsFormItem)* ":]"

SignatureExpression ::=
    "\" (Name ".")* Name (":" Name)*

AsExpression ::=
    Expression "as" SignatureExpression

VariadicFunction ::=
    Function "..."

VariadicSequence ::=
    Sequence "..."

VariadicTarget ::=
    VariadicName |
    VariadicFunction |
    VariadicSequence

VariadicRhs ::=
    VariadicTarget |
    Expression

VariadicInExpression ::=
    VariadicTarget "in" VariadicRhs

InExpression ::=
    List<Target> "in" Expression

VariadicNotInExpression ::=
    VariadicTarget "notin" VariadicRhs

NotInExpression ::=
    List<Target> "notin" Expression

VariadicColonEqualsExpression ::=
    VariadicTarget ":=" VariadicRhs

ColonEqualsExpression ::=
    Target ":=" Expression

EqualsExpression ::=
    Expression "=" Expression

NotEqualsExpression ::=
    Expression "!=" Expression

TypeScopedInfixOperatorName ::=
    SignatureExpression "::" OperatorName "/"

TypeScopedOperatorName ::=
    SignatureExpression "::" OperatorName

MemberScopedOperatorName ::=
    "[" (Name ".")* OperatorName "]"

MemberScopedName ::=
    (Name ".")* Name

Operator ::=
    OperatorName |
    MemberScopedOperatorName |
    TypeScopedOperatorName |
    TypeScopedInfixOperatorName

InfixCommandExpression ::=
    Expression InfixCommandExpression Expression

PrefixOperatorExpression ::=
    MemberScopedOperatorName Expression

InfixOperatorExpression ::=
    Expression MemberScopedOperatorName Expression

PostfixOperatorExpression ::=
    Expression MemberScopedOperatorName

FunctionCallExpression ::=
    Name "(" List<Expression> ")"

SubParamCallExpression ::=
    Name "_" "(" List<Expression> ")"

SubAndRegularParamCallExpression ::=
    Name "_" "(" List<Expression> ")" "(" List<Expression> ")"

CallExpression ::=
    FunctionCallExpression |
    SubParamCallExpression |
    SubAndRegularParamCallExpression

TupleExpression ::=
    "(" Expression ("," Expression)* ")"

OperationExpression ::=
    PrefixOperatorExpression |
    InfixOperatorExpression |
    PostfixOperatorExpression |
    InfixCommandExpression

NameAssignmentExpression ::=
    Name ":=" Expression

FunctionAssignmentExpression ::=
    Function ":=" Expression

SetAssignmentExpression ::=
    Set ":=" Expression

SequenceAssignmentExpression ::=
    Sequence ":=" Expression

TupleAssignmentExpression ::=
    Tuple ":=" Expression

NameAssignmentAssignmentExpression ::=
    NameAssignment ":=" Expression

AssignmentExpression ::=
    NameAssignmentExpression |
    FunctionAssignmentExpression |
    SetAssignmentExpression |
    SequenceAssignmentExpression |
    TupleAssignmentExpression |
    NameAssignmentAssignmentExpression |
    OperationExpression ":=" Expression

GroupingExpression ::=
    "(" Expression ")" |
    "{" Expression "}"

Expression ::=
    Name |
    MemberScopedName |
    Tuple |
    Sequence |
    Function |
    Set |
    GroupingExpression |
    OperationExpression |
    CommandExpression |
    AsExpression |
    VariadicColonEqualsExpression |
    ColonEqualsExpression |
    EqualsExpression |
    NotEqualsExpression |
    CallExpression |
    TupleExpression |
    AssignmentExpression
```

-----------------------------------------------------------------------------------

### Phase 2
```yaml
Clause ::=
    and: |
    not: |
    or: |
    exists: |
    existsUnique: |
    forAll: |
    if: |
    iff: |
    Text[.*] |
    Statement[Expression]

Spec ::=
    Statement[IsExpression | VariadicIsExpression] |
     Statement[InExpression | VariadicInExpression]

and: Clause+

not: Clause

or: Clause+

exists: Target+
where?: Spec+
suchThat?: Clause+

existsUnique: Target+
where?: Spec+
suchThat?: Clause+

forAll: Target+
where?: Spec+
suchThat?: Clause+
then: Clause+

if: Clause+
then: Clause+

iff: Clause+
then: Clause+

generated:
from: (Name | Function)+
when?: Statement[ColonEqualsExpression]+

piecewise:
when?: Clause+
then?: Statement[ColonEqualsExpression]+
else?: Statement[ColonEqualsExpression]+

matching: Statement[ColonEqualsExpression]+

equality:
between: Target, Target
provided: (Statement[Expression InfixCommandExpression Expression] |
           Statement[Expression MemberScopedOperatorName Expression])

membership:
through: Statement

view:
as: Text[SignatureExpression]
via: Statement[Expression]
by?: Statement[CommandForm]

symbols: Name+
where: Statement[ColonEqualsExpression]+

memberSymbols: Name+
where: Statement[ColonEqualsExpression]+

MetadataItem ::=
    note: |
    author: |
    tag: |
    reference:

ProvidingItem ::=
    (view:)* |
    symbols: |
    memberSymbols: |
    equality: |
    membership:

SatisfyingItem ::=
    generated: |
    Clause |
    Spec |
    ColonEqualsExpression |
    VariadicColonEqualsExpression

ExpressingItem ::=
    piecewise: |
    match: |
    Clause |
    Spec |
    ColonEqualsExpression |
    VariadicColonEqualsExpression

[Id]
Defines: Target
with?: Assignment+
given?: Target+
when?: Spec+
suchThat?: Clause+
means?: Statement[IsExpression | VariadicIsExpression]
satisfying: SatisfyingItem+
expressing: ExpressingItem+
using?: Statement[ColonEqualsExpression]+
writing?: Text[.*]+
written: Text[.*]+
called?: Text[.*]+
Providing?: ProvidingItem+
Metadata?: MetadataItem+

note: Text[.*]

author: Text[.*]+

tag: Text[.*]+

reference: Text["@" (Name ".") Name (":page" "{" [0-9]+ "}")?
               (":offset" "{" [0-9]+ "}")?
               (":at" "{" Text[.*] "}"]+

[Id]
States:
given?: Target+
when?: Spec+
suchThat?: Clause+
that: (Clause | Spec | ColonEqualsExpression)+
using?: Statement[ColonEqualsExpression]+
written: Text[.*]+
called?: Text[.*]+
Metadata?: MetadataItem+

["@" "\" (Name ".")* Name]
Resource:
. type?: Text[.*]
. name?: Text[.*]
. author?: Text[.*]+
. homepage?: Text[.*]
. url?:  Text[.*]
. offset?: Text[.*]
Metadata?: MetadataItem+

[Id?]
Axiom: Text[.*]*
given?: Target+
where?: Spec+
suchThat?: Clause+
then: Clause+
iff?: Clause+
using?: Statement[ColonEqualsExpression]+
Metadata?: MetadataItem+

[Id?]
Conjecture: Text[.*]*
given?: Target+
where?: Spec+
suchThat?: Clause+
then: Clause+
iff?: Clause+
using?: Statement[ColonEqualsExpression]+
Metadata?: MetadataItem+

[Id?]
Theorem: Text[.*]*
given?: Target+
where?: Spec+
suchThat?: Clause+
then: Clause+
iff?: Clause+
using?: Statement[ColonEqualsExpression]+
Proof: Text[.*]+
Metadata?: MetadataItem+

["#" (Name ".")* Name]
Topic: Text[.*]*
content: Text[.*]
Metadata?: MetadataItem+

Note:
content: Text[.*]
Metadata?: MetadataItem+

SpecifyItem ::=
    zero: |
    positiveInt: |
    negativeInt: |
    positiveFloat: |
    negativeFloat:

Specify: SpecifyItem

zero:
is: Text[SignatureExpression]

positiveInt:
is: Text[SignatureExpression]

negativeInt:
is: Text[SignatureExpression]

positiveFloat:
is: Text[SignatureExpression]

negativeFloat:
is: Text[SignatureExpression]

TopLevelGroup ::=
    Defines: |
    States: |
    Axiom: |
    Conjecture: |
    Theorem: |
    Topic: |
    Resource: |
    Specify: |
    Note:

TopLevelGroupOrTextBlock ::=
    TopLevelGroup | TextBlock

List<TopLevelGroupOrTextBlock> ::=
    TopLevelGroupOrTextBlock*

Document ::=
    List<TopLevelGroupOrTextBlock>
```

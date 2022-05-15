# MathLingua Specification

## Syntax

### Shared
```
NameAssignment ::= Name ":=" (Name |
                              OperatorName |
                              Tuple |
                              Sequence |
                              Function |
                              Set)
FunctionAssignment ::= Function ":=" Function
Assignment ::= NameAssignment | FunctionAssignment

Name ::= [a-zA-Z0-9'"`]+("_"[a-zA-Z0-9'"`]+)?
OperatorName ::= [~!@#$%^&*-+=|<>?'`"]+("_"[a-zA-Z0-9'"`]+)?
NameParam ::= (Name "..."?)
List<NameParam> ::= NameParam ("," NameParam)*

RegularFunction ::= Name "(" List<NameParam> ")"
SubParamFunction ::= Name "_" "{" List<NameParam> "}"
SubAndRegularParamFunction ::= Name "_" "{" List<NameParam> "}" "(" List<NameParam> ")"
Function ::= RegularFunction |
             SubParamFunction |
             SubAndRegularParamFunction

SubParamFunctionSequence ::= "{" SubParamFunction "}" "_" ""{" List<NameParam> "}"
SubAndRegularParamFunctionSequence ::= "{" SubAndRegularParamFunction "}" "_" "{" List<NameParam> "}"
Sequence ::= SubParamFunctionSequence |
             SubAndRegularParamFunctionSequence

Tuple ::= "(" Target ("," Target)* ")"
NameOrNameAssignment ::= Name | NameAssignment
Set ::= "{" NameOrNameAssignment ("," NameOrNameAssignment)* "}"
Target ::= Assignment |
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
Argument ::= Target |
             Text[.*] |
             Statement[.*]

Text[x] ::= "\"" x "\""         with " escaped with \"
TextBlock ::= "::" .* "::"      with :: escaped with {::}
Statement[x] ::= ("'" x "'"     without an escape for ') |
                 ("`" x "`"     without an escape for `)
                                where x is any character except the surrounding
                                characters unless escaped
Id ::= "[" (CommandForm |
           (Name InfixCommandForm Name)) |
           (OperatorName Name) |
           (Name OperatorName) |
           (Name OperatorName Name) "]"

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

SquareTargetItem ::= Name |
                     Tuple |
                     Sequence |
                     Function |
                     Set
List<Expression> ::= Expression ("," Expression)*
CommandExpression ::= "\" (Name ".")* Name
                      ("_" "{" List<Expression>+ "}" "^" "{" List<Expression>+ "}"
                      "[" (SquareTargetItem+) | (Name "...") "]")?
                      ("{" List<Expression>+ "}")?
                      (":" "{" List<Expression>+ "}")*
                      ("(" List<Expression>+ ")")?
CommandForm ::= "\" (Name ".")* Name
                    ("_" "{" List<NameParam>+ "}" "^" "{" List<NameParam>+ "}"
                    "[" (SquareTargetItem+) | (Name "...") "]")?
                    ("{" List<NameParam>+ "}")?
                    (":" "{" List<NameParam>+ "}")*
                    ("(" List<NameParam>+ ")")?
InfixCommandExpression ::= CommandExpression "/"
InfixCommandForm ::= CommandForm "/"
List<Target> ::= Target ("," Target)*
NameOrCommand ::= Name | CommandExpression
List<NameOrCommand> ::= NameOrCommand ("," NameOrCommand)*
IsExpression ::= List<Target> "is" List<NameOrCommand>
MetaIsFormItem ::= "statement" | "assignment" | "specification" | "expression" | "definition"
MetaIsForm ::= "[:" MetaIsFormItem ("," MetaIsFormItem)* ":]"
SignatureExpression ::= "\" (Name ".")* Name (":" Name)*
AsExpression ::= Expression "as" SignatureExpression
InExpression ::= List<Target> "in" List<NameOrCommand>
NotInExpression ::= List<Target> "notin" <membership params>
ColonEqualsExpression ::= Target ":=" Expression
EqualsExpression ::= Expression "=" Expression
NotEqualsExpression ::= Expression "!=" Expression
TypeScopedOperatorName ::= SignatureExpression "::" OperatorName "/"
MemberScopedOperatorName ::= "[" (Name ".")* OperatorName "]"
MemberScopedName ::= (Name ".")* Name
Operator ::= OperatorName |
             MemberScopedOperatorName |
             TypeScopedOperatorName
InfixCommandExpression ::= Expression InfixCommandExpression Expression
PrefixOperatorExpression ::= MemberScopedOperatorName Expression
InfixOperatorExpression ::= Expression MemberScopedOperatorName Expression
PostfixOperatorExpression ::= Expression MemberScopedOperatorName
RegularFunctionCall ::= Name "(" List<Expression> ")"
SubParamFunctionCall ::= Name "_" "{" List<Expression> "}"
SubAndRegularParamFunctionCall ::= Name "_" "{" List<Expression> "}" "(" List<Expression> ")"
FunctionCall ::= RegularFunctionCallRegularFunctionCall |
                 SubParamFunctionCall |
                 SubAndRegularParamFunctionCall
TupleCall ::= "(" Expression ("," Expression)* ")"
OperationExpression ::= PrefixOperatorExpression |
                        InfixOperatorExpression |
                        PostfixOperatorExpression |
                        InfixCommandExpression
NameAssignmentExpression ::= Name ":=" Expression
FunctionAssignmentExpression ::= Function ":=" Expression
SetAssignmentExpression ::= Set ":=" Expression
SequenceAssignmentExpression ::= Sequence ":=" Expression
TupleAssignmentExpression ::= Tuple ":=" Expression
NameAssignmentAssignmentExpression ::= NameAssignment ":=" Expression
AssignmentExpression ::= NameAssignmentExpression |
                         FunctionAssignmentExpression |
                         SetAssignmentExpression |
                         SequenceAssignmentExpression |
                         TupleAssignmentExpression |
                         NameAssignmentAssignmentExpression |
                         OperationExpression ":=" Expression
GroupingExpression ::= "(" Expression ")" |
                       "{" Expression "}"
Expression ::= Name |
               MemberScopedName |
               Tuple |
               Sequence |
               Function |
               Set |
               GroupingExpression |
               OperationExpression |
               CommandExpression |
               AsExpression |
               ColonEqualsExpression |
               EqualsExpression |
               NotEqualsExpression |
               FunctionCall |
               TupleCall |
               AssignmentExpression
```

-----------------------------------------------------------------------------------

### Phase 2
```yaml
Clause ::= and: |
           not: |
           or: |
           exists: |
           existsUnique: |
           forAll: |
           if: |
           iff: |
           Text[.*] |
           Statement[Expression]

Spec ::= Statement[IsExpression] |
         Statement[InExpression]

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

MetadataItem ::= note: |
                 author: |
                 tag: |
                 reference:

[Id]
Defines: Target
with?: Assignment+
given?: Target+
when?: Spec+
suchThat?: Clause+
means?: Statement[IsExpression]
satisfying: (generated: | Clause+ | Spec+ | ColonEqualsExpression+)
expressing: (piecewise: | match: | Clause+ | Spec+ | ColonEqualsExpression+)
using?: Statement[ColonEqualsExpression]+
writing?: Text[.*]+
written: Text[.*]+
called?: Text[.*]+
Providing?: ((view:)* |
             symbols: |
             memberSymbols: |
             equality: |
             membership:)
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

Specify: (zero: |
          positiveInt: |
          negativeInt: |
          positiveFloat: |
          negativeFloat:)

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

TopLevelGroup ::= Defines: |
                  States: |
                  Axiom: |
                  Conjecture: |
                  Theorem: |
                  Topic: |
                  Resource: |
                  Specify: |
                  Note:

TopLevelGroupOrTextBlock ::= TopLevelGroup | TextBlock

List<TopLevelGroupOrTextBlock> ::= TopLevelGroupOrTextBlock*

Document ::= List<TopLevelGroupOrTextBlock>
```

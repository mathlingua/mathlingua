# MathLingua Language Specification

```
Name ::= Regex[[a-zA-Z0-9'"`]+("_"[a-zA-Z0-9'"`]+)?]

OperatorName ::= Regex[[~!@#$%^&*-+=|<>?'`"]+("_"[a-zA-Z0-9'"`]+)?]

NameAssignmentItem ::=
   Name |
   OperatorName |
   Tuple |
   Sequence |
   Function |
   Set

NameAssignment ::= Name ":=" NameAssignmentItem

FunctionAssignment ::= Function ":=" Function

Assignment ::=
   NameAssignment |
   FunctionAssignment

NameOrVariadicName ::=
   Name |
   VariadicName

VariadicName ::= Name ("...")?

Function ::= Name "(" NameOrVariadicName ("," NameOrVariadicName)* ")"

SubParamCall ::= Name "_" "(" NameOrVariadicName ("," NameOrVariadicName)* ")"

SubAndRegularParamCall ::= Name "_" "(" NameOrVariadicName ("," NameOrVariadicName)* ")" "(" NameOrVariadicName ("," NameOrVariadicName)* ")"

FunctionCall ::=
   Function |
   SubParamCall |
   SubAndRegularParamCall

SubParamSequence ::= "{" SubParamCall "}" "_" "(" VariadicName ("," VariadicName)* ")"

SubAndRegularParamSequence ::= "{" SubAndRegularParamCall "}" "_" "(" VariadicName ("," VariadicName)* ")"

Sequence ::=
   SubParamSequence |
   SubAndRegularParamSequence

Tuple ::= "(" Target ("," Target)* ")"

NameOrNameAssignment ::=
   Name |
   NameAssignment

Set ::= "{" NameOrNameAssignment ("," NameOrNameAssignment)* "}"

Target ::=
   Assignment |
   Name |
   OperatorName |
   Tuple |
   Sequence |
   Function |
   Set

Argument ::=
   Target |
   Text[] |
   Statement[]

Text ::= ".*" [escape=\"]

TextBlock ::= ::.*:: [escape={::}]

Statement ::= ('.*' [escape=null] | `.*` [escape=null])

InfixCommandFormCall ::= Name InfixCommandForm Name

IdPrefixOperatorCall ::= OperatorName Name

IdPostfixOperatorCall ::= Name OperatorName

IdInfixOperatorCall ::= Name OperatorName Name

IdForm ::=
   CommandForm |
   InfixCommandFormCall |
   IdPrefixOperatorCall |
   IdPostfixOperatorCall |
   IdInfixOperatorCall

Id ::= "[" IdForm "]"

SquareTargetItem ::=
   Name |
   Tuple |
   Sequence |
   Function |
   Set

SquareParams ::= ("[" SquareTargetItem ("," SquareTargetItem)* ("|" Name "...")? "]")? ("_" "{" Expression ("," Expression)* "}")? ("^" "{" Expression ("," Expression)* "}")?

NamedParameterExpression ::= ":" "{" Expression ("," Expression)* "}"

CommandExpression ::= "\" Name ("." Name)* (SquareParams)? ("{" Expression ("," Expression)* "}")? (NamedParameterExpression)* ("(" Expression ("," Expression)* ")")?

NamedParameterForm ::= ":" "Name" "{" NameOrVariadicName ("," NameOrVariadicName)* "}"

CommandForm ::= "\" Name ("." Name)* (SquareParams)? ("{" NameOrVariadicName ("," NameOrVariadicName)* "}")? (NamedParameterForm)* ("(" NameOrVariadicName ("," NameOrVariadicName)* ")")?

InfixCommandExpressionForm ::= CommandExpression "/"

InfixCommandForm ::= CommandForm "/"

NameOrCommand ::=
   Name |
   CommandExpression

VariadicIsRhs ::=
   VariadicName |
   CommandExpression

VariadicIsExpression ::= VariadicTarget 'is' VariadicIsRhs

IsExpression ::= Target ("," Target)* 'is' NameOrCommand ("," NameOrCommand)*

StatementIsFormItem ::= "statement"

AssignmentIsFormItem ::= "assignment"

SpecificationIsFormItem ::= "specification"

ExpressionIsFormItem ::= "expression"

DefinitionIsFormItem ::= "definition"

MetaIsFormItem ::=
   StatementIsFormItem |
   AssignmentIsFormItem |
   SpecificationIsFormItem |
   ExpressionIsFormItem |
   DefinitionIsFormItem

MetaIsForm ::= "[:" MetaIsFormItem ("," MetaIsFormItem)* ":]"

SignatureExpression ::= "\" Name ("." Name)* (":" Name)*

AsExpression ::= Expression 'as' SignatureExpression

VariadicFunction ::= Function "..."

VariadicSequence ::= Sequence "..."

VariadicTarget ::=
   VariadicName |
   VariadicFunction |
   VariadicSequence

VariadicRhs ::=
   VariadicTarget |
   Expression

VariadicInExpression ::= VariadicTarget "in" VariadicRhs

InExpression ::= Target ("," Target)* "in" Expression

VariadicNotInExpression ::= VariadicTarget "notin" VariadicRhs

NotInExpression ::= Target ("," Target)* "notin" Expression

VariadicColonEqualsExpression ::= VariadicTarget ":=" VariadicRhs

ColonEqualsExpression ::= Target ":=" Expression

EqualsExpression ::= Expression "=" Expression

NotEqualsExpression ::= Expression "!=" Expression

TypeScopedInfixOperatorName ::= SignatureExpression "::" OperatorName "/"

TypeScopedOperatorName ::= SignatureExpression "::" OperatorName

MemberScopedOperatorName ::= "[" (Name ".")* OperatorName "]"

MemberScopedName ::= Name ("." Name)*

Operator ::=
   OperatorName |
   MemberScopedOperatorName |
   TypeScopedOperatorName |
   TypeScopedInfixOperatorName

InfixCommandExpression ::= Expression InfixCommandExpressionForm Expression

PrefixOperatorExpression ::= MemberScopedOperatorName Expression

InfixOperatorExpression ::= Expression MemberScopedOperatorName Expression

PostfixOperatorExpression ::= Expression MemberScopedOperatorName

FunctionCallExpression ::= Name "(" Expression ("," Expression)* ")"

SubParamCallExpression ::= Name "_" "(" Expression ("," Expression)* ")"

SubAndRegularParamCallExpression ::= Name "_" "(" Expression ("," Expression)* ")" "(" Expression ("," Expression)* ")"

CallExpression ::=
   FunctionCallExpression |
   SubParamCallExpression |
   SubAndRegularParamCallExpression

TupleExpression ::= "(" Expression ("," Expression)* ")"

OperationExpression ::=
   PrefixOperatorExpression |
   InfixOperatorExpression |
   PostfixOperatorExpression |
   InfixCommandExpression

NameAssignmentExpression ::= Name ":=" Expression

FunctionAssignmentExpression ::= Function ":=" Expression

SetAssignmentExpression ::= Set ":=" Expression

SequenceAssignmentExpression ::= Sequence ":=" Expression

TupleAssignmentExpression ::= Tuple ":=" Expression

NameAssignmentAssignmentExpression ::= NameAssignment ":=" Expression

OperationAssignmentExpression ::= OperationExpression ":=" Expression

AssignmentExpression ::=
   NameAssignmentExpression |
   FunctionAssignmentExpression |
   SetAssignmentExpression |
   SequenceAssignmentExpression |
   TupleAssignmentExpression |
   NameAssignmentAssignmentExpression |
   OperationAssignmentExpression

ParenGroupingExpression ::= "(" Expression ")"

CurlyGroupingExpression ::= "{" Expression "}"

GroupingExpression ::=
   ParenGroupingExpression |
   CurlyGroupingExpression

Expression ::=
   Name |
   MemberScopedName |
   Tuple |
   Sequence |
   Function |
   FunctionCall |
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
   AssignmentExpression |
   InExpression |
   IsExpression |
   NotInExpression |
   VariadicInExpression |
   VariadicIsExpression |
   VariadicNotInExpression

Clause ::=
   and: |
   not: |
   or: |
   exists: |
   existsUnique: |
   forAll: |
   if: |
   iff: |
   Spec |
   Text[.*] |
   Statement[Expression]

Spec ::=
   Statement[IsExpression | VariadicIsExpression] |
   Statement[InExpression | VariadicInExpression]

and: (Clause)+

not: Clause

or: (Clause)+

exists: (Target)+
where?: (Spec)+
suchThat?: (Clause)+

existsUnique: (Target)+
where?: (Spec)+
suchThat?: (Clause)+

forAll: (Target)+
where?: (Spec)+
suchThat?: (Clause)+
then: (Clause)+

if: (Clause)+
then: (Clause)+

iff: (Clause)+
then: (Clause)+

NameOrFunction ::=
   Name |
   Function |
   FunctionCall

generated: 
from: (NameOrFunction)+
when?: (Statement[ColonEqualsExpression])+

piecewise: 
when?: (Clause)+
then?: (Statement[ColonEqualsExpression])+
else?: (Statement[ColonEqualsExpression])+

matching: (Statement[ColonEqualsExpression])+

ProvidedItem ::=
   Statement[Expression InfixCommandExpression Expression] |
   Statement[OperationExpression]

equality: 
between: Target Target
provided: ProvidedItem

membership: 
through: Statement[]

view: 
as: Text[SignatureExpression]
via: Statement[Expression]
by?: Statement[CommandForm]

symbols: (Name)+
where: (Statement[ColonEqualsExpression])+

symbols: (Name)+
as: Text[SignatureExpression]

memberSymbols: (Name)+
where: (Statement[ColonEqualsExpression])+

memberSymbols: (Name)+
as: Text[SignatureExpression]

MetadataItem ::=
   note: |
   author: |
   tag:

ProvidingItem ::=
   view: |
   symbols: |
   memberSymbols: |
   symbols:as: |
   memberSymbols:as: |
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
   matching: |
   Clause |
   Spec |
   ColonEqualsExpression |
   VariadicColonEqualsExpression

written: (Text[.*])+

writing?: (Text[.*])+

called?: (Text[.*])+

CodifiedItem ::=
   written: |
   writing: |
   called:

[Id]
Defines: Target
with?: (Assignment)+
given?: (Target)+
when?: (Spec)+
suchThat?: (Clause)+
means?: Statement[IsExpression | VariadicIsExpression]
satisfying: (SatisfyingItem)+
expressing: (ExpressingItem)+
Providing?: (ProvidingItem)+
Using?: (Statement[ColonEqualsExpression])+
Codified: (CodifiedItem)+
References?: (Text["@" (Name ".") Name (":page" "{" [0-9]+ "}")?
   (":offset" "{" [0-9]+ "}")?
   (":at" "{" Text[.*] "}"])+
Metadata?: (MetadataItem)+

note: Text[.*]

tag: (Text[.*])+

ThatItem ::=
   Clause |
   Spec |
   ColonEqualsExpression

[Id]
States: 
given?: (Target)+
when?: (Spec)+
suchThat?: (Clause)+
that: (ThatItem)+
Using?: Statement[ColonEqualsExpression]
Codified: (CodifiedItem)+
References?: (Text["@" (Name ".") Name (":page" "{" [0-9]+ "}")?
   (":offset" "{" [0-9]+ "}")?
   (":at" "{" Text[.*] "}"])+
Metadata?: (MetadataItem)+

ResourceName ::= "@" Name ("." Name)*

type: Text[.*]

name: Text[.*]

author: (Text[.*])+

homepage: Text[.*]

url: Text[.*]

offset: Text[.*]

ResourceItem ::=
   type: |
   name: |
   author: |
   homepage: |
   url: |
   offset:

[ResourceName]
Resource: (ResourceItem)+

[(Id)?]
Axiom: (Text[.*])*
given?: (Target)+
where?: (Spec)+
suchThat?: (Clause)+
then: (Clause)+
iff?: (Clause)+
Using?: (Statement[ColonEqualsExpression])+
References?: (Text["@" (Name ".") Name (":page" "{" [0-9]+ "}")?
   (":offset" "{" [0-9]+ "}")?
   (":at" "{" Text[.*] "}"])+
Metadata?: (MetadataItem)+

[(Id)?]
Conjecture: (Text[.*])*
given?: (Target)+
where?: (Spec)+
suchThat?: (Clause)+
then: (Clause)+
iff?: (Clause)+
Using?: (Statement[ColonEqualsExpression])+
References?: (Text["@" (Name ".") Name (":page" "{" [0-9]+ "}")?
   (":offset" "{" [0-9]+ "}")?
   (":at" "{" Text[.*] "}"])+
Metadata?: (MetadataItem)+

[(Id)?]
Theorem: (Text[.*])*
given?: (Target)+
where?: (Spec)+
suchThat?: (Clause)+
then: (Clause)+
iff?: (Clause)+
Using?: (Statement[ColonEqualsExpression])+
Proof?: (Text[.*])+
References?: (Text["@" (Name ".") Name (":page" "{" [0-9]+ "}")?
   (":offset" "{" [0-9]+ "}")?
   (":at" "{" Text[.*] "}"])+
Metadata?: (MetadataItem)+

TopicName ::= "#" Name ("." Name)*

[TopicName]
Topic: (Text[.*])*
content: Text[.*]
Metadata?: (MetadataItem)+

Note: 
content: Text[.*]
Metadata?: (MetadataItem)+

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
   TopLevelGroup |
   TextBlock

Document ::= (TopLevelGroupOrTextBlock)*

```
# MathLingua Language Specification

```
Name ::= Regex[[a-zA-Z0-9'"`]+("_"[a-zA-Z0-9'"`]+)?]

VariadicName ::= Name ("...")?

NameOrVariadicName ::=
   Name |
   VariadicName

OperatorName ::= Regex[[~!@#$%^&*-+=|<>?'`"]+("_"[a-zA-Z0-9'"`]+)?]

NameAssignmentItem ::=
   Name |
   OperatorName |
   TupleForm |
   SequenceForm |
   FunctionForm |
   SetForm

NameAssignment ::= Name ":=" NameAssignmentItem

NameOrNameAssignment ::=
   Name |
   NameAssignment

FunctionForm ::= Name "(" NameOrVariadicName ("," NameOrVariadicName)* ")"

VariadicFunctionForm ::= FunctionForm "..."

SubParamFormCall ::= Name "_" "(" NameOrVariadicName ("," NameOrVariadicName)* ")"

SubAndRegularParamFormCall ::= Name "_" "(" NameOrVariadicName ("," NameOrVariadicName)* ")" "(" NameOrVariadicName ("," NameOrVariadicName)* ")"

FunctionFormCall ::=
   FunctionForm |
   SubParamFormCall |
   SubAndRegularParamFormCall

SubParamSequenceForm ::= "{" SubParamFormCall "}" "_" "(" VariadicName ("," VariadicName)* ")"

SubAndRegularParamSequenceForm ::= "{" SubAndRegularParamFormCall "}" "_" "(" VariadicName ("," VariadicName)* ")"

SequenceForm ::=
   SubParamSequenceForm |
   SubAndRegularParamSequenceForm

TupleForm ::= "(" Target ("," Target)* ")"

SetForm ::= "{" NameOrNameAssignment ("," NameOrNameAssignment)* "}"

Target ::=
   NameAssignment |
   Name |
   OperatorName |
   TupleForm |
   SequenceForm |
   FunctionForm |
   SetForm

Argument ::=
   Target |
   Text[] |
   Formulation[]

Text ::= ".*" [escape=\"]

TextBlock ::= ::.*:: [escape={::}]

Formulation ::= ('.*' [escape=null] | `.*` [escape=null])

InfixCommandFormPart ::= CommandFormCall "/"

InfixCommandFormCall ::= Name InfixCommandFormPart Name

PrefixOperatorFormCall ::= OperatorName Name

PostfixOperatorFormCall ::= Name OperatorName

InfixOperatorFormCall ::= Name OperatorName Name

IdForm ::=
   CommandFormCall |
   InfixCommandFormCall |
   PrefixOperatorFormCall |
   PostfixOperatorFormCall |
   InfixOperatorFormCall

Id ::= "[" IdForm "]"

CommandFormCall ::= "\" Name ("." Name)* (SquareParams)? ("{" NameOrVariadicName ("," NameOrVariadicName)* "}")? (NamedParameterForm)* ("(" NameOrVariadicName ("," NameOrVariadicName)* ")")?

CommandExpressionCall ::= "\" Name ("." Name)* (SquareParams)? ("{" Expression ("," Expression)* "}")? (NamedParameterExpression)* ("(" Expression ("," Expression)* ")")?

InfixCommandExpressionPart ::= CommandExpressionCall "/"

SquareTargetItem ::=
   Name |
   TupleForm |
   SequenceForm |
   FunctionForm |
   SetForm

SquareParams ::= ("[" SquareTargetItem ("," SquareTargetItem)* ("|" Name "...")? "]")? ("_" "{" Expression ("," Expression)* "}")? ("^" "{" Expression ("," Expression)* "}")?

NamedParameterExpression ::= ":" "{" Expression ("," Expression)* "}"

NamedParameterForm ::= ":" "Name" "{" NameOrVariadicName ("," NameOrVariadicName)* "}"

NameOrCommandExpressionCall ::=
   Name |
   CommandExpressionCall

VariadicIsRhs ::=
   VariadicName |
   CommandExpressionCall

VariadicIsExpression ::= VariadicTargetForm 'is' VariadicIsRhs

IsExpression ::= Target ("," Target)* 'is' NameOrCommandExpressionCall ("," NameOrCommandExpressionCall)*

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

VariadicSequenceForm ::= SequenceForm "..."

VariadicTargetForm ::=
   VariadicName |
   VariadicFunctionForm |
   VariadicSequenceForm

VariadicRhs ::=
   VariadicTargetForm |
   Expression

VariadicInExpression ::= VariadicTargetForm "in" VariadicRhs

InExpression ::= Target ("," Target)* "in" Expression

VariadicNotInExpression ::= VariadicTargetForm "notin" VariadicRhs

NotInExpression ::= Target ("," Target)* "notin" Expression

VariadicColonEqualsExpression ::= VariadicTargetForm ":=" VariadicRhs

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

InfixCommandExpression ::= Expression InfixCommandExpressionPart Expression

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

FunctionAssignmentExpression ::= FunctionForm ":=" Expression

SetAssignmentExpression ::= SetForm ":=" Expression

SequenceAssignmentExpression ::= SequenceForm ":=" Expression

TupleAssignmentExpression ::= TupleForm ":=" Expression

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
   TupleForm |
   SequenceForm |
   FunctionForm |
   FunctionFormCall |
   SetForm |
   GroupingExpression |
   OperationExpression |
   CommandExpressionCall |
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
   Formulation[Expression]

Spec ::=
   Formulation[IsExpression | VariadicIsExpression] |
   Formulation[InExpression | VariadicInExpression]

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
   FunctionForm |
   FunctionFormCall

generated: 
from: (NameOrFunction)+
when?: (Formulation[ColonEqualsExpression])+

piecewise: 
when?: (Clause)+
then?: (Formulation[ColonEqualsExpression])+
else?: (Formulation[ColonEqualsExpression])+

matching: (Formulation[ColonEqualsExpression])+

ProvidedItem ::=
   Formulation[Expression InfixCommandExpression Expression] |
   Formulation[OperationExpression]

equality: 
between: Target Target
provided: ProvidedItem

membership: 
through: Formulation[]

view: 
as: Text[SignatureExpression]
via: Formulation[Expression]
by?: Formulation[CommandFormCall]

symbols: (Name)+
where: (Formulation[ColonEqualsExpression])+

symbols: (Name)+
as: Text[SignatureExpression]

memberSymbols: (Name)+
where: (Formulation[ColonEqualsExpression])+

memberSymbols: (Name)+
as: Text[SignatureExpression]

MetadataItem ::=
   contributor: |
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
means?: Formulation[IsExpression | VariadicIsExpression]
satisfying: (SatisfyingItem)+
expressing: (ExpressingItem)+
Providing?: (ProvidingItem)+
Using?: (Formulation[ColonEqualsExpression])+
Codified: (CodifiedItem)+
Documented?: (DocumentedItem)+
References?: (Text["@" (Name ".") Name (":page" "{" [0-9]+ "}")?
   (":offset" "{" [0-9]+ "}")?
   (":at" "{" Text[.*] "}"])+
Metadata?: (MetadataItem)+

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
Using?: Formulation[ColonEqualsExpression]
Codified: (CodifiedItem)+
Documented?: (DocumentedItem)+
References?: (Text["@" (Name ".") Name (":page" "{" [0-9]+ "}")?
   (":offset" "{" [0-9]+ "}")?
   (":at" "{" Text[.*] "}"])+
Metadata?: (MetadataItem)+

ResourceName ::= "@" Name ("." Name)*

type: Text[.*]

name: Text[.*]

author: (Text[.*])+

contributor: (Text[.*])+

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
Using?: (Formulation[ColonEqualsExpression])+
Documented?: (DocumentedItem)+
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
Using?: (Formulation[ColonEqualsExpression])+
Documented?: (DocumentedItem)+
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
Using?: (Formulation[ColonEqualsExpression])+
Proof?: (Text[.*])+
Documented?: (DocumentedItem)+
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

DocumentedItem ::=
   loosely: |
   overview: |
   motivation: |
   history: |
   examples: |
   related: |
   discovered: |
   notes:

loosely: Text[.*]

overview: Text[.*]

motivation: Text[.*]

history: Text[.*]

examples: (Text[.*])+

related: (Text[.*])+

discovered: (Text[.*])+

notes: (Text[.*])+

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

## New Specification Testbed:

```yaml

[id]
Describes:
extends?:
satisfying?:
Operations:
. infix:
  when:
  states|expresses:
  written?:
. prefix:
  when:
  states|expresses:
  written?:
. postfix:
  when:
  states|expresses:
  written?:

[id]
Declares:
means?:
satisfying?:
Operations:
. infix:
  when:
  states|expresses:
  written?:
. prefix:
  when:
  states|expresses:
  written?:
. postfix:
  when:
  states|expresses:
  written?:

[id]
States:
when:
that:

```

/*
 * Copyright 2022 Dominic Kramer
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package ast

type FormulationNodeType interface {
	MlgNodeType
	FormulationDebuggable
	FormulationNodeType()
	Start() Position
}

func (NameForm) FormulationNodeType()                               {}
func (FunctionForm) FormulationNodeType()                           {}
func (FunctionExpressionForm) FormulationNodeType()                 {}
func (TupleForm) FormulationNodeType()                              {}
func (FixedSetForm) FormulationNodeType()                           {}
func (ConditionalSetForm) FormulationNodeType()                     {}
func (ConditionalSetIdForm) FormulationNodeType()                   {}
func (FunctionCallExpression) FormulationNodeType()                 {}
func (TupleExpression) FormulationNodeType()                        {}
func (FixedSetExpression) FormulationNodeType()                     {}
func (ConditionalSetExpression) FormulationNodeType()               {}
func (CommandExpression) FormulationNodeType()                      {}
func (CommandAtExpression) FormulationNodeType()                    {}
func (PrefixOperatorCallExpression) FormulationNodeType()           {}
func (PostfixOperatorCallExpression) FormulationNodeType()          {}
func (InfixOperatorCallExpression) FormulationNodeType()            {}
func (IsExpression) FormulationNodeType()                           {}
func (ExtendsExpression) FormulationNodeType()                      {}
func (AsExpression) FormulationNodeType()                           {}
func (NameOrdinalCallExpression) FormulationNodeType()              {}
func (ChainExpression) FormulationNodeType()                        {}
func (Signature) FormulationNodeType()                              {}
func (MetaKinds) FormulationNodeType()                              {}
func (StructuralColonEqualsForm) FormulationNodeType()              {}
func (ExpressionColonEqualsItem) FormulationNodeType()              {}
func (ExpressionColonArrowItem) FormulationNodeType()               {}
func (EnclosedNonCommandOperatorTarget) FormulationNodeType()       {}
func (NonEnclosedNonCommandOperatorTarget) FormulationNodeType()    {}
func (CommandOperatorTarget) FormulationNodeType()                  {}
func (CommandId) FormulationNodeType()                              {}
func (CommandAtId) FormulationNodeType()                            {}
func (PrefixOperatorId) FormulationNodeType()                       {}
func (PostfixOperatorId) FormulationNodeType()                      {}
func (InfixOperatorId) FormulationNodeType()                        {}
func (InfixCommandOperatorId) FormulationNodeType()                 {}
func (PseudoTokenNode) FormulationNodeType()                        {}
func (PseudoExpression) FormulationNodeType()                       {}
func (MultiplexedInfixOperatorCallExpression) FormulationNodeType() {}
func (InfixOperatorForm) FormulationNodeType()                      {}
func (PrefixOperatorForm) FormulationNodeType()                     {}
func (PostfixOperatorForm) FormulationNodeType()                    {}

func (n NameForm) Start() Position                               { return n.MetaData.Start }
func (n FunctionForm) Start() Position                           { return n.MetaData.Start }
func (n FunctionExpressionForm) Start() Position                 { return n.MetaData.Start }
func (n TupleForm) Start() Position                              { return n.MetaData.Start }
func (n FixedSetForm) Start() Position                           { return n.MetaData.Start }
func (n ConditionalSetForm) Start() Position                     { return n.MetaData.Start }
func (n ConditionalSetIdForm) Start() Position                   { return n.MetaData.Start }
func (n FunctionCallExpression) Start() Position                 { return n.MetaData.Start }
func (n TupleExpression) Start() Position                        { return n.MetaData.Start }
func (n FixedSetExpression) Start() Position                     { return n.MetaData.Start }
func (n ConditionalSetExpression) Start() Position               { return n.MetaData.Start }
func (n CommandExpression) Start() Position                      { return n.MetaData.Start }
func (n CommandAtExpression) Start() Position                    { return n.MetaData.Start }
func (n PrefixOperatorCallExpression) Start() Position           { return n.MetaData.Start }
func (n PostfixOperatorCallExpression) Start() Position          { return n.MetaData.Start }
func (n InfixOperatorCallExpression) Start() Position            { return n.MetaData.Start }
func (n IsExpression) Start() Position                           { return n.MetaData.Start }
func (n ExtendsExpression) Start() Position                      { return n.MetaData.Start }
func (n AsExpression) Start() Position                           { return n.MetaData.Start }
func (n NameOrdinalCallExpression) Start() Position              { return n.MetaData.Start }
func (n ChainExpression) Start() Position                        { return n.MetaData.Start }
func (n Signature) Start() Position                              { return n.MetaData.Start }
func (n MetaKinds) Start() Position                              { return n.MetaData.Start }
func (n StructuralColonEqualsForm) Start() Position              { return n.MetaData.Start }
func (n ExpressionColonEqualsItem) Start() Position              { return n.MetaData.Start }
func (n ExpressionColonArrowItem) Start() Position               { return n.MetaData.Start }
func (n EnclosedNonCommandOperatorTarget) Start() Position       { return n.MetaData.Start }
func (n NonEnclosedNonCommandOperatorTarget) Start() Position    { return n.MetaData.Start }
func (n CommandOperatorTarget) Start() Position                  { return n.MetaData.Start }
func (n CommandId) Start() Position                              { return n.MetaData.Start }
func (n CommandAtId) Start() Position                            { return n.MetaData.Start }
func (n PrefixOperatorId) Start() Position                       { return n.MetaData.Start }
func (n PostfixOperatorId) Start() Position                      { return n.MetaData.Start }
func (n InfixOperatorId) Start() Position                        { return n.MetaData.Start }
func (n InfixCommandOperatorId) Start() Position                 { return n.MetaData.Start }
func (n PseudoTokenNode) Start() Position                        { return n.MetaData.Start }
func (n PseudoExpression) Start() Position                       { return n.MetaData.Start }
func (n MultiplexedInfixOperatorCallExpression) Start() Position { return n.MetaData.Start }
func (n InfixOperatorForm) Start() Position                      { return n.MetaData.Start }
func (n PrefixOperatorForm) Start() Position                     { return n.MetaData.Start }
func (n PostfixOperatorForm) Start() Position                    { return n.MetaData.Start }

///////////////////////// Structural Forms ///////////////////////////////////////////

type StructuralFormType interface {
	FormulationNodeType // any StructuralFormType is a NodeType
	StructuralForm()
}

func (NameForm) StructuralForm()               {}
func (FunctionForm) StructuralForm()           {}
func (FunctionExpressionForm) StructuralForm() {}
func (TupleForm) StructuralForm()              {}
func (FixedSetForm) StructuralForm()           {}
func (ConditionalSetForm) StructuralForm()     {}
func (InfixOperatorForm) StructuralForm()      {}
func (PrefixOperatorForm) StructuralForm()     {}
func (PostfixOperatorForm) StructuralForm()    {}

// x
type NameForm struct {
	Text string
	// specifies "x" vs x but Text will never contain the quotes
	IsStropped      bool
	HasQuestionMark bool
	VarArg          VarArgData
	MetaData        MetaData
}

// f(x, y)
type FunctionForm struct {
	Target   NameForm
	Params   []NameForm
	VarArg   VarArgData
	MetaData MetaData
}

// x * y
type InfixOperatorForm struct {
	Operator NameForm
	Lhs      NameForm
	Rhs      NameForm
	MetaData MetaData
}

// +y
type PrefixOperatorForm struct {
	Operator NameForm
	Param    NameForm
	MetaData MetaData
}

// x!
type PostfixOperatorForm struct {
	Operator NameForm
	Param    NameForm
	MetaData MetaData
}

// f[x, y]
type FunctionExpressionForm struct {
	Target   NameForm
	Params   []NameForm
	VarArg   VarArgData
	MetaData MetaData
}

// (x, y)
type TupleForm struct {
	Params   []StructuralFormType
	VarArg   VarArgData
	MetaData MetaData
}

// {x, y}
type FixedSetForm struct {
	Params   []StructuralFormType
	VarArg   VarArgData
	MetaData MetaData
}

// {x | ...}
type ConditionalSetForm struct {
	Target   StructuralFormType
	VarArg   VarArgData
	MetaData MetaData
}

//////////////////////////////////////////////////////////////////////

type LiteralFormType interface {
	FormulationNodeType
	LiteralFormType()
}

// [x]{x | f[x]...}
type ConditionalSetIdForm struct {
	Symbols   []StructuralFormType
	Target    StructuralFormType
	Condition FunctionExpressionForm
	MetaData  MetaData
}

func (NameForm) LiteralFormType()             {}
func (FunctionForm) LiteralFormType()         {}
func (TupleForm) LiteralFormType()            {}
func (FixedSetForm) LiteralFormType()         {}
func (ConditionalSetIdForm) LiteralFormType() {}

/////////////////////////////////////////////////////////////////////////////////////////////////

type LiteralExpressionType interface {
	FormulationNodeType
	LiteralExpressionType()
}

func (FunctionCallExpression) LiteralExpressionType()   {}
func (TupleExpression) LiteralExpressionType()          {}
func (FixedSetExpression) LiteralExpressionType()       {}
func (ConditionalSetExpression) LiteralExpressionType() {}

///////////////////////////////////////////////////////////////////////////////////

type LiteralType interface {
	LiteralFormType
	LiteralExpressionType
	LiteralType()
}

////////////////////////////////////// Pseudo Nodes ////////////////////////////

type PseudoTokenNode struct {
	Text     string
	Type     TokenType
	MetaData MetaData
}

type PseudoExpression struct {
	Children []FormulationNodeType
	MetaData MetaData
}

///////////////////////////////////// Expressions /////////////////////////////

type ExpressionType interface {
	FormulationNodeType // any ExpressionType is a NodeType
	ExpressionType()
}

func (NameForm) ExpressionType()                               {}
func (FunctionCallExpression) ExpressionType()                 {}
func (TupleExpression) ExpressionType()                        {}
func (FixedSetExpression) ExpressionType()                     {}
func (ConditionalSetExpression) ExpressionType()               {}
func (CommandExpression) ExpressionType()                      {}
func (CommandAtExpression) ExpressionType()                    {}
func (PrefixOperatorCallExpression) ExpressionType()           {}
func (PostfixOperatorCallExpression) ExpressionType()          {}
func (InfixOperatorCallExpression) ExpressionType()            {}
func (AsExpression) ExpressionType()                           {}
func (NameOrdinalCallExpression) ExpressionType()              {}
func (ChainExpression) ExpressionType()                        {}
func (PseudoTokenNode) ExpressionType()                        {}
func (PseudoExpression) ExpressionType()                       {}
func (IsExpression) ExpressionType()                           {}
func (ExtendsExpression) ExpressionType()                      {}
func (MultiplexedInfixOperatorCallExpression) ExpressionType() {}
func (ExpressionColonEqualsItem) ExpressionType()              {}
func (ExpressionColonArrowItem) ExpressionType()               {}
func (Signature) ExpressionType()                              {}

// f(x + y, z) or (f + g)(x)
type FunctionCallExpression struct {
	Target   ExpressionType
	Args     []ExpressionType
	MetaData MetaData
}

// (x + y, z)
type TupleExpression struct {
	Args     []ExpressionType
	MetaData MetaData
}

// {x + y, z}
type FixedSetExpression struct {
	Args     []ExpressionType
	MetaData MetaData
}

// [x]{(x, x+1) | x is \real ; x > 0}
type ConditionalSetExpression struct {
	Symbols    []StructuralFormType
	Target     ExpressionType
	Conditions []ExpressionType
	MetaData   MetaData
}

type NamedArg struct {
	Name     NameForm
	Args     *[]ExpressionType
	MetaData MetaData
}

// \function:on{A}:to{B}
type CommandExpression struct {
	Names      []NameForm
	SquareArgs *[]StructuralFormType
	CurlyArgs  *[]ExpressionType
	NamedArgs  *[]NamedArg
	ParenArgs  *[]ExpressionType
	MetaData   MetaData
}

// \set@[x]{x | x is \real ; x > 0}
type CommandAtExpression struct {
	Names      []NameForm
	Expression ExpressionType
	MetaData   MetaData
}

// -x
type PrefixOperatorCallExpression struct {
	Target   OperatorType
	Arg      ExpressionType
	MetaData MetaData
}

// x!
type PostfixOperatorCallExpression struct {
	Target   OperatorType
	Arg      ExpressionType
	MetaData MetaData
}

// x + y
type InfixOperatorCallExpression struct {
	Target   OperatorType
	Lhs      ExpressionType
	Rhs      ExpressionType
	MetaData MetaData
}

type MultiplexedInfixOperatorCallExpression struct {
	Target   OperatorType
	Lhs      []ExpressionType
	Rhs      []ExpressionType
	MetaData MetaData
}

// x is \y
type IsExpression struct {
	Lhs      []ExpressionType
	Rhs      []KindType
	MetaData MetaData
}

// x extends \y
type ExtendsExpression struct {
	Lhs      []ExpressionType
	Rhs      []KindType
	MetaData MetaData
}

// x as \[y]
type AsExpression struct {
	Lhs      ExpressionType
	Rhs      Signature
	MetaData MetaData
}

// NONE
// x{1}
type NameOrdinalCallExpression struct {
	Target   NameForm
	Arg      ExpressionType
	MetaData MetaData
}

// (x + y).z.a.b
type ChainExpression struct {
	Parts               []ExpressionType
	HasTrailingOperator bool
	MetaData            MetaData
}

///////////////////////////////////////////////////////////////////////////////

// \[a.b.c:x:y]::[inner label]
type Signature struct {
	MainNames       []string
	NamedGroupNames []string
	HasAtSymbol     bool
	InnerLabel      *string
	MetaData        MetaData
}

/////////////////////////////// Kinds /////////////////////////////////////////

type KindType interface {
	FormulationNodeType
	KindType()
}

// [: specification, states :]
type MetaKinds struct {
	Kinds    []string
	MetaData MetaData
}

func (NameForm) KindType()                      {} // x could refer to a type
func (CommandExpression) KindType()             {} // \function:on{A}:to{B}
func (PrefixOperatorCallExpression) KindType()  {} // *A
func (PostfixOperatorCallExpression) KindType() {} // B!
func (InfixOperatorCallExpression) KindType()   {} // A \to/ B
func (MetaKinds) KindType()                     {} // [: specification, states :]

///////////////////////////////////////////////////////////////////////////////

////////////////////////////// Colon Equals ///////////////////////////////////

type ColonEqualsType interface {
	FormulationNodeType
	ColonEqualsType()
}

func (StructuralColonEqualsForm) ColonEqualsType() {}
func (ExpressionColonEqualsItem) ColonEqualsType() {}

// X := (a, b) or f(x) := y
type StructuralColonEqualsForm struct {
	Lhs      StructuralFormType
	Rhs      StructuralFormType
	MetaData MetaData
}

// f(x) := x + 1
type ExpressionColonEqualsItem struct {
	Lhs      ExpressionType
	Rhs      ExpressionType
	MetaData MetaData
}

// x + y :=> x
type ExpressionColonArrowItem struct {
	Lhs      ExpressionType
	Rhs      ExpressionType
	MetaData MetaData
}

///////////////////////////////////////////////////////////////////////////////

///////////////////////////// Operators ///////////////////////////////////////

type OperatorType interface {
	FormulationNodeType // any OperatorType is a NodeType
	OperatorType()
}

func (EnclosedNonCommandOperatorTarget) OperatorType()    {}
func (NonEnclosedNonCommandOperatorTarget) OperatorType() {}
func (CommandOperatorTarget) OperatorType()               {}

// [x] or [x + y]
type EnclosedNonCommandOperatorTarget struct {
	Target        ExpressionType
	HasLeftColon  bool
	HasRightColon bool
	MetaData      MetaData
}

// *, ++, :+, or +:
type NonEnclosedNonCommandOperatorTarget struct {
	Text          string
	HasLeftColon  bool
	HasRightColon bool
	MetaData      MetaData
}

type CommandOperatorTarget struct {
	Command  CommandExpression
	MetaData MetaData
}

///////////////////////////////////////// Ids ///////////////////////////////////

type IdType interface {
	FormulationNodeType
	IdType()
}

func (CommandId) IdType()              {}
func (CommandAtId) IdType()            {}
func (PrefixOperatorId) IdType()       {}
func (PostfixOperatorId) IdType()      {}
func (InfixOperatorId) IdType()        {}
func (InfixCommandOperatorId) IdType() {}

type NamedParam struct {
	Name     NameForm
	Params   *[]StructuralFormType
	MetaData MetaData
}

// \function:on{A}:to{B}
type CommandId struct {
	Names        []NameForm
	SquareParams *[]StructuralFormType
	CurlyParams  *[]StructuralFormType
	NamedParams  *[]NamedParam
	ParenParams  *[]NameForm
	MetaData     MetaData
}

// \function:on{A}:to{B}/
type InfixCommandId struct {
	Names        []NameForm
	SquareParams *[]StructuralFormType
	CurlyParams  *[]StructuralFormType
	NamedParams  *[]NamedParam
	ParenParams  *[]NameForm
	MetaData     MetaData
}

// \set@[x]{f[x] | g[x]}
type CommandAtId struct {
	Names    []NameForm
	Param    LiteralFormType
	MetaData MetaData
}

// +x
type PrefixOperatorId struct {
	Operator NonEnclosedNonCommandOperatorTarget
	Param    StructuralFormType
	MetaData MetaData
}

// -x
type PostfixOperatorId struct {
	Operator NonEnclosedNonCommandOperatorTarget
	Param    StructuralFormType
	MetaData MetaData
}

type InfixOperatorId struct {
	Lhs      StructuralFormType
	Operator NonEnclosedNonCommandOperatorTarget
	Rhs      StructuralFormType
	MetaData MetaData
}

// A \subset/ B
type InfixCommandOperatorId struct {
	Lhs      StructuralFormType
	Operator InfixCommandId
	Rhs      StructuralFormType
	MetaData MetaData
}

/////////////////////////// Support structures ////////////////////////////////

type VarArgData struct {
	IsVarArg    bool
	VarArgCount *string
	MetaData    MetaData
}

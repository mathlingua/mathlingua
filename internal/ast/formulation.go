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

type Position struct {
	Offset int
	Row    int
	Column int
}

// all AST nodes are MlgNodes
type NodeType interface {
	NodeType()
	Start() Position
}

func (NameForm) NodeType()                               {}
func (FunctionForm) NodeType()                           {}
func (FunctionExpressionForm) NodeType()                 {}
func (TupleForm) NodeType()                              {}
func (FixedSetForm) NodeType()                           {}
func (ConditionalSetForm) NodeType()                     {}
func (ConditionalSetIdForm) NodeType()                   {}
func (FunctionCallExpression) NodeType()                 {}
func (TupleExpression) NodeType()                        {}
func (FixedSetExpression) NodeType()                     {}
func (ConditionalSetExpression) NodeType()               {}
func (CommandExpression) NodeType()                      {}
func (CommandAtExpression) NodeType()                    {}
func (PrefixOperatorCallExpression) NodeType()           {}
func (PostfixOperatorCallExpression) NodeType()          {}
func (InfixOperatorCallExpression) NodeType()            {}
func (IsExpression) NodeType()                           {}
func (IsNotExpression) NodeType()                        {}
func (AsExpression) NodeType()                           {}
func (NameOrdinalCallExpression) NodeType()              {}
func (ChainExpression) NodeType()                        {}
func (Signature) NodeType()                              {}
func (MetaKinds) NodeType()                              {}
func (StructuralColonEqualsForm) NodeType()              {}
func (ExpressionColonEqualsItem) NodeType()              {}
func (ExpressionColonArrowItem) NodeType()               {}
func (EnclosedNonCommandOperatorTarget) NodeType()       {}
func (NonEnclosedNonCommandOperatorTarget) NodeType()    {}
func (CommandOperatorTarget) NodeType()                  {}
func (CommandId) NodeType()                              {}
func (CommandAtId) NodeType()                            {}
func (PrefixOperatorId) NodeType()                       {}
func (PostfixOperatorId) NodeType()                      {}
func (InfixOperatorId) NodeType()                        {}
func (InfixCommandOperatorId) NodeType()                 {}
func (PseudoTokenNode) NodeType()                        {}
func (PseudoExpression) NodeType()                       {}
func (MultiplexedInfixOperatorCallExpression) NodeType() {}

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
func (n IsNotExpression) Start() Position                        { return n.MetaData.Start }
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

///////////////////////// Structural Forms ///////////////////////////////////////////

type StructuralFormType interface {
	NodeType // any StructuralFormType is a NodeType
	StructuralForm()
}

func (NameForm) StructuralForm()               {}
func (FunctionForm) StructuralForm()           {}
func (FunctionExpressionForm) StructuralForm() {}
func (TupleForm) StructuralForm()              {}
func (FixedSetForm) StructuralForm()           {}
func (ConditionalSetForm) StructuralForm()     {}

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
	NodeType
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
	NodeType
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
	Children []NodeType
	MetaData MetaData
}

///////////////////////////////////// Expressions /////////////////////////////

type ExpressionType interface {
	NodeType // any ExpressionType is a NodeType
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
func (IsNotExpression) ExpressionType()                        {}
func (MultiplexedInfixOperatorCallExpression) ExpressionType() {}
func (ExpressionColonEqualsItem) ExpressionType()              {}
func (ExpressionColonArrowItem) ExpressionType()               {}

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

type SubSupArgs struct {
	SquareArgs []StructuralFormType
	SubArgs    []ExpressionType
	SupArgs    []ExpressionType
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
	SubSupArgs *SubSupArgs
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

// x isnot \y
type IsNotExpression struct {
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
	Parts    []ExpressionType
	MetaData MetaData
}

///////////////////////////////////////////////////////////////////////////////

// \[a.b.c:x:y]
type Signature struct {
	MainNames       []string
	NamedGroupNames []string
	HasAtSymbol     bool
	MetaData        MetaData
}

/////////////////////////////// Kinds /////////////////////////////////////////

type KindType interface {
	NodeType
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
	NodeType
	ColonEqualsType()
}

func (StructuralColonEqualsForm) ColonEqualsType() {}
func (ExpressionColonEqualsItem) ColonEqualsType() {}

// X := (a, b)
type StructuralColonEqualsForm struct {
	Lhs      NameForm
	Rhs      StructuralFormType
	MetaData MetaData
}

// f(x) := x + 1
type ExpressionColonEqualsItem struct {
	Lhs      StructuralFormType
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
	NodeType // any OperatorType is a NodeType
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
	NodeType
	IdType()
}

func (CommandId) IdType()              {}
func (CommandAtId) IdType()            {}
func (PrefixOperatorId) IdType()       {}
func (PostfixOperatorId) IdType()      {}
func (InfixOperatorId) IdType()        {}
func (InfixCommandOperatorId) IdType() {}

type SubSupParams struct {
	SquareParams []StructuralFormType
	SubParams    []StructuralFormType
	SupParams    []StructuralFormType
	MetaData     MetaData
}

type NamedParam struct {
	Name     NameForm
	Params   *[]StructuralFormType
	MetaData MetaData
}

// \function:on{A}:to{B}
type CommandId struct {
	Names        []NameForm
	SubSupParams *SubSupParams
	CurlyParams  *[]StructuralFormType
	NamedParams  *[]NamedParam
	ParenParams  *[]NameForm
	MetaData     MetaData
}

// \function:on{A}:to{B}/
type InfixCommandId struct {
	Names        []NameForm
	SubSupParams *SubSupParams
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

///////////////////////////////////////////////////////////////////////////////

type FormulationType interface {
	ExpressionType
	ColonEqualsType
	FormulationType()
}

/////////////////////////// Support structures ////////////////////////////////

type VarArgData struct {
	IsVarArg    bool
	VarArgCount *string
	MetaData    MetaData
}

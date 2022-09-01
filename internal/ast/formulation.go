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
}

func (NameForm) NodeType()                      {} //
func (FunctionForm) NodeType()                  {} //
func (FunctionExpressionForm) NodeType()        {}
func (TupleForm) NodeType()                     {} //
func (FixedSetForm) NodeType()                  {}
func (ConditionalSetForm) NodeType()            {}
func (ConditionalSetIdForm) NodeType()          {}
func (FunctionCallExpression) NodeType()        {}
func (TupleExpression) NodeType()               {}
func (FixedSetExpression) NodeType()            {}
func (ConditionalSetExpression) NodeType()      {}
func (CommandExpression) NodeType()             {}
func (CommandAtExpression) NodeType()           {}
func (PrefixOperatorCallExpression) NodeType()  {}
func (PostfixOperatorCallExpression) NodeType() {}
func (InfixOperatorCallExpression) NodeType()   {}
func (IsExpression) NodeType()                  {}
func (IsNotExpression) NodeType()               {}
func (AsExpression) NodeType()                  {}
func (NameOrdinalCallExpression) NodeType()     {}
func (ChainExpression) NodeType()               {}
func (Signature) NodeType()                     {}
func (MetaKinds) NodeType()                     {}
func (StructuralColonEqualsForm) NodeType()     {}
func (ExpressionColonEqualsItem) NodeType()     {}
func (ExpressionColonEqualsIsItem) NodeType()   {}
func (NonCommandOperatorTarget) NodeType()      {}
func (CommandOperatorTarget) NodeType()         {}
func (CommandId) NodeType()                     {}
func (CommandAtId) NodeType()                   {}
func (PrefixOperatorId) NodeType()              {}
func (PostfixOperatorId) NodeType()             {}
func (InfixOperatorId) NodeType()               {}

///////////////////////// Structural Forms ///////////////////////////////////////////

type StructuralFormType interface {
	NodeType       // any StructuralFormType is a NodeType
	ExpressionType // any StructuralFormType is also an ExpressionType
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
}

// f(x, y)
type FunctionForm struct {
	Target NameForm
	Params []NameForm
	VarArg VarArgData
}

// f[x, y]
type FunctionExpressionForm struct {
	Target NameForm
	Params []NameForm
	VarArg VarArgData
}

// (x, y)
type TupleForm struct {
	Params []StructuralFormType
	VarArg VarArgData
}

// {x, y}
type FixedSetForm struct {
	Params []StructuralFormType
	VarArg VarArgData
}

// {x | ...}
type ConditionalSetForm struct {
	Target StructuralFormType
	VarArg VarArgData
}

//////////////////////////////////////////////////////////////////////

type LiteralFormType interface {
	NodeType
	LiteralFormType()
}

// [x]{x | f[x]...}
type ConditionalSetIdForm struct {
	Symbols   []NameForm
	Target    StructuralFormType
	Condition FunctionExpressionForm
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

///////////////////////////////////// Expressions /////////////////////////////

type ExpressionType interface {
	NodeType // any ExpressionType is a NodeType
	ExpressionType()
}

func (NameForm) ExpressionType()                      {}
func (FunctionForm) ExpressionType()                  {}
func (FunctionCallExpression) ExpressionType()        {}
func (TupleForm) ExpressionType()                     {}
func (TupleExpression) ExpressionType()               {}
func (FixedSetForm) ExpressionType()                  {}
func (FixedSetExpression) ExpressionType()            {}
func (CommandExpression) ExpressionType()             {}
func (CommandAtExpression) ExpressionType()           {}
func (PrefixOperatorCallExpression) ExpressionType()  {}
func (PostfixOperatorCallExpression) ExpressionType() {}
func (InfixOperatorCallExpression) ExpressionType()   {}
func (AsExpression) ExpressionType()                  {}
func (NameOrdinalCallExpression) ExpressionType()     {}
func (ChainExpression) ExpressionType()               {}

// f(x + y, z) or (f + g)(x)
type FunctionCallExpression struct {
	Target ExpressionType
	Args   []ExpressionType
}

// (x + y, z)
type TupleExpression struct {
	Args []ExpressionType
}

// {x + y, z}
type FixedSetExpression struct {
	Args []ExpressionType
}

// [x]{(x, x+1) | x is \real ; x > 0}
type ConditionalSetExpression struct {
	Symbols    []NameForm
	Target     ExpressionType
	Conditions []ExpressionType
}

// \function:on{A}:to{B}
type CommandExpression struct {
	// TODO
}

// \set@[x]{x | x is \real ; x > 0}
type CommandAtExpression struct {
	Names      []NameForm
	Expression []ExpressionType
}

// -x
type PrefixOperatorCallExpression struct {
	Target OperatorType
	Arg    ExpressionType
}

// x!
type PostfixOperatorCallExpression struct {
	Target OperatorType
	Arg    ExpressionType
}

// x + y
type InfixOperatorCallExpression struct {
	Target OperatorType
	Lhs    ExpressionType
	Rhs    ExpressionType
}

// x is \y
type IsExpression struct {
	Lhs []ExpressionType
	Rhs []KindType
}

// x isnot \y
type IsNotExpression struct {
	Lhs []ExpressionType
	Rhs []KindType
}

// x as \[y]
type AsExpression struct {
	Lhs ExpressionType
	Rhs Signature
}

// x{1}
type NameOrdinalCallExpression struct {
	Target NameForm
	Arg    ExpressionType
}

// (x + y).z.a.b
type ChainExpression struct {
	Parts []ExpressionType
}

///////////////////////////////////////////////////////////////////////////////

type Signature struct {
	MainNames       []string
	NamedGroupNames []string
	HasAtSymbol     bool
}

/////////////////////////////// Kinds /////////////////////////////////////////

type KindType interface {
	NodeType
	KindType()
}

// [: specification, states :]
type MetaKinds struct {
	Kinds []string
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

func (StructuralColonEqualsForm) ColonEqualsType()   {}
func (ExpressionColonEqualsItem) ColonEqualsType()   {}
func (ExpressionColonEqualsIsItem) ColonEqualsType() {}

// X := (a, b)
type StructuralColonEqualsForm struct {
	Lhs NameForm
	Rhs StructuralFormType
}

// f(x) := x + 1
type ExpressionColonEqualsItem struct {
	Lhs StructuralFormType
	Rhs ExpressionType
}

// f(x) := x + 1 is \function
type ExpressionColonEqualsIsItem struct {
	Lhs    StructuralFormType
	Middle ExpressionType
	Rhs    []KindType
}

///////////////////////////////////////////////////////////////////////////////

///////////////////////////// Operators ///////////////////////////////////////

type OperatorType interface {
	NodeType // any OperatorType is a NodeType
	OperatorType()
}

func (NonCommandOperatorTarget) OperatorType() {}
func (CommandOperatorTarget) OperatorType()    {}

// * or [x] or [x + y]
type NonCommandOperatorTarget struct {
	Target     ExpressionType
	IsEnclosed bool // specifies if it is enclosed in [ and ]
}

type CommandOperatorTarget struct {
	Command CommandExpression
}

///////////////////////////////////////// Ids ///////////////////////////////////

type IdType interface {
	NodeType
	IdType()
}

// \function:on{A}:to{B}
type CommandId struct {
}

// \set@[x]{f[x] | g[x]}
type CommandAtId struct {
	Names []NameForm
	Param ConditionalSetIdForm
}

// +x
type PrefixOperatorId struct {
	Operator NameForm
	Param    StructuralFormType
}

// -x
type PostfixOperatorId struct {
	Operator NameForm
	Param    StructuralFormType
}

// A \subset/ B
type InfixOperatorId struct {
	Lhs      StructuralFormType
	Operator CommandId
	Rhs      StructuralFormType
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
}

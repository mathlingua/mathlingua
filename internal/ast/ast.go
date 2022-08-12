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

// all AST nodes are MlgNodes
type NodeType interface {
	NodeType()
}

///////////////////////// Structural Forms ///////////////////////////////////////////

type StructuralFormType interface {
	NodeType       // any StructuralFormType is a NodeType
	ExpressionType // any StructuralFormType is also an ExpressionType
	StructuralForm()
}

func (n NameForm) StructuralForm()               {}
func (f FunctionForm) StructuralForm()           {}
func (f FunctionExpressionForm) StructuralForm() {}
func (f FunctionSequenceForm) StructuralForm()   {}
func (s SequenceForm) StructuralForm()           {}
func (t TupleForm) StructuralForm()              {}
func (FixedSetForm) StructuralForm()             {}
func (c ConditionalSetForm) StructuralForm()     {}

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

// {x_(i, j)}_(i, j)
type SequenceForm struct {
	Target NameForm
	Params []NameForm
	VarArg VarArgData
}

// {f_(i, j)(x, y)}_(i, j)
type FunctionSequenceForm struct {
	Target    NameForm
	SubParams []NameForm
	Params    []NameForm
	VarArg    VarArgData
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

func (n NameForm) LiteralFormType()             {}
func (f FunctionForm) LiteralFormType()         {}
func (s SequenceForm) LiteralFormType()         {}
func (t TupleForm) LiteralFormType()            {}
func (f FixedSetForm) LiteralFormType()         {}
func (c ConditionalSetIdForm) LiteralFormType() {}

/////////////////////////////////////////////////////////////////////////////////////////////////

type LiteralExpressionType interface {
	NodeType
	LiteralExpressionType()
}

func (f FunctionCallExpression) LiteralExpressionType()   {}
func (s SequenceCallExpression) LiteralExpressionType()   {}
func (t TupleExpression) LiteralExpressionType()          {}
func (f FixedSetExpression) LiteralExpressionType()       {}
func (c ConditionalSetExpression) LiteralExpressionType() {}

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

func (f FunctionCallExpression) ExpressionType()         {}
func (s SequenceCallExpression) ExpressionType()         {}
func (f FunctionSequenceCallExpression) ExpressionType() {}
func (t TupleExpression) ExpressionType()                {}
func (f FixedSetExpression) ExpressionType()             {}
func (c CommandExpression) ExpressionType()              {}
func (c CommandAtExpression) ExpressionType()            {}
func (p PrefixOperatorCallExpression) ExpressionType()   {}
func (p PostfixOperatorCallExpression) ExpressionType()  {}
func (i InfixOperatorCallExpression) ExpressionType()    {}
func (a AsExpression) ExpressionType()                   {}
func (n NameOrdinalCallExpression) ExpressionType()      {}
func (c ChainExpression) ExpressionType()                {}

// f(x + y, z) or (f + g)(x)
type FunctionCallExpression struct {
	Target ExpressionType
	Args   []ExpressionType
}

// x_(i + j, k)
type SequenceCallExpression struct {
	Target NameForm
	Args   []ExpressionType
}

// f_(i + j, k)(x + y, z)
type FunctionSequenceCallExpression struct {
	Target  NameForm
	SubArgs []ExpressionType
	Args    []ExpressionType
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

func (n NameForm) KindType()                      {} // x could refer to a type
func (c CommandExpression) KindType()             {} // \function:on{A}:to{B}
func (p PrefixOperatorCallExpression) KindType()  {} // *A
func (p PostfixOperatorCallExpression) KindType() {} // B!
func (i InfixOperatorCallExpression) KindType()   {} // A \to/ B

///////////////////////////////////////////////////////////////////////////////

////////////////////////////// Colon Equals ///////////////////////////////////

type ColonEqualsType interface {
	NodeType
	ColonEqualsType()
}

func (s StructuralColonEqualsForm) ColonEqualsType()   {}
func (e ExpressionColonEqualsItem) ColonEqualsType()   {}
func (e ExpressionColonEqualsIsItem) ColonEqualsType() {}

// X := {x_(i, j)}_(i, j)
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

func (n NonCommandOperatorTarget) OperatorType() {}
func (c CommandOperatorTarget) OperatorType()    {}

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

///////////////////////////////////////////////////////////////////////////////

type Formulation struct {
	RawText string
	Root    FormulationType
}

type IfSection struct {
	Formulations []Formulation
}

type ThenSection struct {
	Formulation []Formulation
}

type IfGroup struct {
	If   IfSection
	Then ThenSection
}

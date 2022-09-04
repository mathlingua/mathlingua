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

func (NameForm) NodeType()                            {}
func (FunctionForm) NodeType()                        {}
func (FunctionExpressionForm) NodeType()              {}
func (TupleForm) NodeType()                           {}
func (FixedSetForm) NodeType()                        {}
func (ConditionalSetForm) NodeType()                  {}
func (ConditionalSetIdForm) NodeType()                {}
func (FunctionCallExpression) NodeType()              {}
func (TupleExpression) NodeType()                     {}
func (FixedSetExpression) NodeType()                  {}
func (ConditionalSetExpression) NodeType()            {}
func (CommandExpression) NodeType()                   {}
func (CommandAtExpression) NodeType()                 {}
func (PrefixOperatorCallExpression) NodeType()        {}
func (PostfixOperatorCallExpression) NodeType()       {}
func (InfixOperatorCallExpression) NodeType()         {}
func (IsExpression) NodeType()                        {}
func (IsNotExpression) NodeType()                     {}
func (AsExpression) NodeType()                        {}
func (NameOrdinalCallExpression) NodeType()           {}
func (ChainExpression) NodeType()                     {}
func (Signature) NodeType()                           {}
func (MetaKinds) NodeType()                           {}
func (StructuralColonEqualsForm) NodeType()           {}
func (ExpressionColonEqualsItem) NodeType()           {}
func (ExpressionColonEqualsIsItem) NodeType()         {}
func (EnclosedNonCommandOperatorTarget) NodeType()    {}
func (NonEnclosedNonCommandOperatorTarget) NodeType() {}
func (CommandOperatorTarget) NodeType()               {}
func (CommandId) NodeType()                           {}
func (CommandAtId) NodeType()                         {}
func (PrefixOperatorId) NodeType()                    {}
func (PostfixOperatorId) NodeType()                   {}
func (InfixOperatorId) NodeType()                     {}
func (PseudoTokenNode) NodeType()                     {}
func (PseudoExpression) NodeType()                    {}

///////////////////////// Structural Forms ///////////////////////////////////////////

// DONE
type StructuralFormType interface {
	NodeType       // any StructuralFormType is a NodeType
	ExpressionType // any StructuralFormType is also an ExpressionType
	StructuralForm()
}

func (NameForm) StructuralForm()               {} // DONE
func (FunctionForm) StructuralForm()           {} // DONE
func (FunctionExpressionForm) StructuralForm() {} // DONE
func (TupleForm) StructuralForm()              {} // DONE
func (FixedSetForm) StructuralForm()           {} // DONE
func (ConditionalSetForm) StructuralForm()     {} // DONE

// DONE
// x
type NameForm struct {
	Text string
	// specifies "x" vs x but Text will never contain the quotes
	IsStropped      bool
	HasQuestionMark bool
	VarArg          VarArgData
}

// DONE
// f(x, y)
type FunctionForm struct {
	Target NameForm
	Params []NameForm
	VarArg VarArgData
}

// DONE
// f[x, y]
type FunctionExpressionForm struct {
	Target NameForm
	Params []NameForm
	VarArg VarArgData
}

// DONE
// (x, y)
type TupleForm struct {
	Params []StructuralFormType
	VarArg VarArgData
}

// DONE
// {x, y}
type FixedSetForm struct {
	Params []StructuralFormType
	VarArg VarArgData
}

// DONE
// {x | ...}
type ConditionalSetForm struct {
	Target StructuralFormType
	VarArg VarArgData
}

//////////////////////////////////////////////////////////////////////

// DONE
type LiteralFormType interface {
	NodeType
	LiteralFormType()
}

// DONE
// [x]{x | f[x]...}
type ConditionalSetIdForm struct {
	Symbols   []StructuralFormType
	Target    StructuralFormType
	Condition FunctionExpressionForm
}

func (NameForm) LiteralFormType()             {} // DONE
func (FunctionForm) LiteralFormType()         {} // DONE
func (TupleForm) LiteralFormType()            {} // DONE
func (FixedSetForm) LiteralFormType()         {} // DONE
func (ConditionalSetIdForm) LiteralFormType() {} // DONE

/////////////////////////////////////////////////////////////////////////////////////////////////

// DONE
type LiteralExpressionType interface {
	NodeType
	LiteralExpressionType()
}

func (FunctionCallExpression) LiteralExpressionType()   {} // DONE
func (TupleExpression) LiteralExpressionType()          {} // DONE
func (FixedSetExpression) LiteralExpressionType()       {} // DONE
func (ConditionalSetExpression) LiteralExpressionType() {} // DONE

///////////////////////////////////////////////////////////////////////////////////

type LiteralType interface {
	LiteralFormType
	LiteralExpressionType
	LiteralType()
}

////////////////////////////////////// Pseudo Nodes ////////////////////////////

type PseudoTokenNode struct {
	Type TokenType
}

type PseudoExpression struct {
	Children []NodeType
}

///////////////////////////////////// Expressions /////////////////////////////

type ExpressionType interface {
	NodeType // any ExpressionType is a NodeType
	ExpressionType()
}

func (NameForm) ExpressionType()                      {} // DONE
func (FunctionForm) ExpressionType()                  {} // DONE
func (FunctionExpressionForm) ExpressionType()        {} // DONE
func (FunctionCallExpression) ExpressionType()        {} // DONE
func (TupleForm) ExpressionType()                     {} // DONE
func (TupleExpression) ExpressionType()               {} // DONE
func (FixedSetForm) ExpressionType()                  {} // DONE
func (FixedSetExpression) ExpressionType()            {} // DONE
func (ConditionalSetForm) ExpressionType()            {} // DONE
func (CommandExpression) ExpressionType()             {} // DONE
func (CommandAtExpression) ExpressionType()           {} // DONE
func (PrefixOperatorCallExpression) ExpressionType()  {}
func (PostfixOperatorCallExpression) ExpressionType() {}
func (InfixOperatorCallExpression) ExpressionType()   {}
func (AsExpression) ExpressionType()                  {}
func (NameOrdinalCallExpression) ExpressionType()     {} // DONE
func (ChainExpression) ExpressionType()               {} // DONE
func (PseudoTokenNode) ExpressionType()               {} // DONE
func (PseudoExpression) ExpressionType()              {} // DONE

// DONE
// f(x + y, z) or (f + g)(x)
type FunctionCallExpression struct {
	Target ExpressionType
	Args   []ExpressionType
}

// DONE
// (x + y, z)
type TupleExpression struct {
	Args []ExpressionType
}

// DONE
// {x + y, z}
type FixedSetExpression struct {
	Args []ExpressionType
}

// DONE
// [x]{(x, x+1) | x is \real ; x > 0}
type ConditionalSetExpression struct {
	Symbols    []StructuralFormType
	Target     ExpressionType
	Conditions []ExpressionType
}

// DONE
type SubSupArgs struct {
	SquareArgs []StructuralFormType
	SubArgs    []ExpressionType
	SupArgs    []ExpressionType
}

// DONE
type NamedArg struct {
	Name NameForm
	Args *[]ExpressionType
}

// DONE
// \function:on{A}:to{B}
type CommandExpression struct {
	Names      []NameForm
	SubSupArgs *SubSupArgs
	CurlyArgs  *[]ExpressionType
	NamedArgs  *[]NamedArg
	ParenArgs  *[]ExpressionType
}

// DONE
// \set@[x]{x | x is \real ; x > 0}
type CommandAtExpression struct {
	Names      []NameForm
	Expression ExpressionType
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

// NONE
// x{1}
type NameOrdinalCallExpression struct {
	Target NameForm
	Arg    ExpressionType
}

// DONE
// (x + y).z.a.b
type ChainExpression struct {
	Parts []ExpressionType
}

///////////////////////////////////////////////////////////////////////////////

// \[a.b.c:x:y]
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

// DONE
type OperatorType interface {
	NodeType // any OperatorType is a NodeType
	OperatorType()
}

func (EnclosedNonCommandOperatorTarget) OperatorType()    {} // DONE
func (NonEnclosedNonCommandOperatorTarget) OperatorType() {} // DONE
func (CommandOperatorTarget) OperatorType()               {} // DONE

// DONE
// [x] or [x + y]
type EnclosedNonCommandOperatorTarget struct {
	Target ExpressionType
}

// DONE
// * or ++
type NonEnclosedNonCommandOperatorTarget struct {
	Text string
}

// DONE
type CommandOperatorTarget struct {
	Command CommandExpression
}

///////////////////////////////////////// Ids ///////////////////////////////////

type IdType interface {
	NodeType
	IdType()
}

func (CommandId) IdType()         {} // DONE
func (CommandAtId) IdType()       {} // DONE
func (PrefixOperatorId) IdType()  {}
func (PostfixOperatorId) IdType() {}
func (InfixOperatorId) IdType()   {}

// DONE
type SubSupParams struct {
	SquareParams []StructuralFormType
	SubParams    []StructuralFormType
	SupParams    []StructuralFormType
}

// DONE
type NamedParam struct {
	Name   NameForm
	Params *[]StructuralFormType
}

// DONE
// \function:on{A}:to{B}
type CommandId struct {
	Names        []NameForm
	SubSupParams *SubSupParams
	CurlyParams  *[]StructuralFormType
	NamedParams  *[]NamedParam
	ParenParams  *[]NameForm
}

// DONE
// \set@[x]{f[x] | g[x]}
type CommandAtId struct {
	Names []NameForm
	Param LiteralFormType
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

// DONE
type VarArgData struct {
	IsVarArg    bool
	VarArgCount *string
}

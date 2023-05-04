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

type VarArgData struct {
	IsVarArg            bool
	VarArgNames         []NameForm
	VarArgBounds        []NameForm
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

////////////////////////////////// Structural Forms ////////////////////////////////////////////////

type StructuralFormType interface {
	FormulationNodeType // any StructuralFormType is a NodeType
	StructuralForm()
}

func (*NameForm) StructuralForm()            {}
func (*FunctionForm) StructuralForm()        {}
func (*TupleForm) StructuralForm()           {}
func (*ConditionalSetForm) StructuralForm()  {}
func (*InfixOperatorForm) StructuralForm()   {}
func (*PrefixOperatorForm) StructuralForm()  {}
func (*PostfixOperatorForm) StructuralForm() {}

// x
type NameForm struct {
	Text string
	// specifies "x" vs x but Text will never contain the quotes
	IsStropped          bool
	HasQuestionMark     bool
	VarArg              VarArgData
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// f(x, y)
type FunctionForm struct {
	Target              NameForm
	Params              []StructuralFormType
	VarArg              VarArgData
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x * y
type InfixOperatorForm struct {
	Operator            NameForm
	Lhs                 StructuralFormType
	Rhs                 StructuralFormType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// +y
type PrefixOperatorForm struct {
	Operator            NameForm
	Param               StructuralFormType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x!
type PostfixOperatorForm struct {
	Operator            NameForm
	Param               StructuralFormType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// (x, y)
type TupleForm struct {
	Params              []StructuralFormType
	VarArg              VarArgData
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// {x | ...}
type ConditionalSetForm struct {
	Target              StructuralFormType
	VarArg              VarArgData
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type LiteralFormType interface {
	FormulationNodeType
	LiteralFormType()
}

// [x]{x | f(x)...}
type ConditionalSetIdForm struct {
	Symbols             []StructuralFormType
	Target              StructuralFormType
	Condition           FunctionForm
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

func (*NameForm) LiteralFormType()             {}
func (*FunctionForm) LiteralFormType()         {}
func (*TupleForm) LiteralFormType()            {}
func (*ConditionalSetIdForm) LiteralFormType() {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type LiteralExpressionType interface {
	FormulationNodeType
	LiteralExpressionType()
}

func (*FunctionCallExpression) LiteralExpressionType()    {}
func (*TupleExpression) LiteralExpressionType()           {}
func (*ConditionalSetExpression) LiteralExpressionType()  {}
func (*FunctionLiteralExpression) LiteralExpressionType() {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type LiteralType interface {
	LiteralFormType
	LiteralExpressionType
	LiteralType()
}

////////////////////////////////////// Pseudo Nodes ////////////////////////////////////////////////

type PseudoTokenNode struct {
	Text                string
	Type                TokenType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

type PseudoExpression struct {
	Children            []FormulationNodeType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

///////////////////////////////////// Expressions //////////////////////////////////////////////////

type ExpressionType interface {
	FormulationNodeType // any ExpressionType is a NodeType
	ExpressionType()
}

func (*NameForm) ExpressionType()                               {}
func (*FunctionCallExpression) ExpressionType()                 {}
func (*TupleExpression) ExpressionType()                        {}
func (*ConditionalSetExpression) ExpressionType()               {}
func (*CommandExpression) ExpressionType()                      {}
func (*PrefixOperatorCallExpression) ExpressionType()           {}
func (*PostfixOperatorCallExpression) ExpressionType()          {}
func (*InfixOperatorCallExpression) ExpressionType()            {}
func (*AsExpression) ExpressionType()                           {}
func (*OrdinalCallExpression) ExpressionType()                  {}
func (*ChainExpression) ExpressionType()                        {}
func (*PseudoTokenNode) ExpressionType()                        {}
func (*PseudoExpression) ExpressionType()                       {}
func (*IsExpression) ExpressionType()                           {}
func (*ExtendsExpression) ExpressionType()                      {}
func (*MultiplexedInfixOperatorCallExpression) ExpressionType() {}
func (*ExpressionColonEqualsItem) ExpressionType()              {}
func (*ExpressionColonArrowItem) ExpressionType()               {}
func (*ExpressionColonDashArrowItem) ExpressionType()           {}
func (*Signature) ExpressionType()                              {}
func (*FunctionLiteralExpression) ExpressionType()              {}

// f(x + y, z) or (f + g)(x)
type FunctionCallExpression struct {
	Target              ExpressionType
	Args                []ExpressionType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x => x + 1 or (x, y) => x + y
type FunctionLiteralExpression struct {
	Lhs                 TupleForm
	Rhs                 ExpressionType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// (x + y, z)
type TupleExpression struct {
	Args                []ExpressionType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// [x]{(x, x+1) | x is \real ; x > 0}
type ConditionalSetExpression struct {
	Symbols             []StructuralFormType
	Target              ExpressionType
	Conditions          []ExpressionType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

type NamedArg struct {
	Name                NameForm
	CurlyArg            *CurlyArg
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// \function:on{A}:to{B}
type CommandExpression struct {
	Names               []NameForm
	CurlyArg            *CurlyArg
	NamedArgs           *[]NamedArg
	ParenArgs           *[]ExpressionType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// []{} or {}
type CurlyArg struct {
	CurlyArgs           *[]ExpressionType
	Direction           *DirectionalParam
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// -x
type PrefixOperatorCallExpression struct {
	Target              OperatorType
	Arg                 ExpressionType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x!
type PostfixOperatorCallExpression struct {
	Target              OperatorType
	Arg                 ExpressionType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x + y
type InfixOperatorCallExpression struct {
	Target              OperatorType
	Lhs                 ExpressionType
	Rhs                 ExpressionType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

type MultiplexedInfixOperatorCallExpression struct {
	Target              OperatorType
	Lhs                 []ExpressionType
	Rhs                 []ExpressionType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x is \y
type IsExpression struct {
	Lhs                 []ExpressionType
	Rhs                 []KindType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x extends \y
type ExtendsExpression struct {
	Lhs                 []ExpressionType
	Rhs                 []KindType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x as \[y]
type AsExpression struct {
	Lhs                 ExpressionType
	Rhs                 Signature
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x{1}
type OrdinalCallExpression struct {
	Target              LiteralFormType
	Args                []ExpressionType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// (x + y).z.a.b
type ChainExpression struct {
	Parts               []ExpressionType
	HasTrailingOperator bool
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// \(a.b.c:x:y)::[inner label]
type Signature struct {
	MainNames           []string
	NamedGroupNames     []string
	IsInfix             bool
	InnerLabel          *string
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

/////////////////////////////// Kinds //////////////////////////////////////////////////////////////

type KindType interface {
	FormulationNodeType
	KindType()
}

// [: specification, states :]
type MetaKinds struct {
	Kinds               []string
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

func (*NameForm) KindType()                      {} // x could refer to a type
func (*CommandExpression) KindType()             {} // \function:on{A}:to{B}
func (*PrefixOperatorCallExpression) KindType()  {} // *A
func (*PostfixOperatorCallExpression) KindType() {} // B!
func (*InfixOperatorCallExpression) KindType()   {} // A \to/ B
func (*MetaKinds) KindType()                     {} // [: specification, states :]

////////////////////////////////////////////////////////////////////////////////////////////////////

////////////////////////////// Colon Equals ////////////////////////////////////////////////////////

type ColonEqualsType interface {
	FormulationNodeType
	ColonEqualsType()
}

func (StructuralColonEqualsForm) ColonEqualsType() {}
func (ExpressionColonEqualsItem) ColonEqualsType() {}

// X := (a, b) or f(x) := y
type StructuralColonEqualsForm struct {
	Lhs                 StructuralFormType
	Rhs                 StructuralFormType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// f(x) := x + 1
type ExpressionColonEqualsItem struct {
	Lhs                 ExpressionType
	Rhs                 ExpressionType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x + y :=> x
type ExpressionColonArrowItem struct {
	Lhs                 ExpressionType
	Rhs                 ExpressionType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x + y :-> x
type ExpressionColonDashArrowItem struct {
	Lhs                 ExpressionType
	Rhs                 ExpressionType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

////////////////////////////////////// Operators ///////////////////////////////////////////////////

type OperatorType interface {
	FormulationNodeType // any OperatorType is a NodeType
	OperatorType()
}

func (*EnclosedNonCommandOperatorTarget) OperatorType()    {}
func (*NonEnclosedNonCommandOperatorTarget) OperatorType() {}
func (*CommandOperatorTarget) OperatorType()               {}

// [x] or [x + y]
type EnclosedNonCommandOperatorTarget struct {
	Target              ExpressionType
	HasLeftColon        bool
	HasRightColon       bool
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// *, ++, :+, or +:
type NonEnclosedNonCommandOperatorTarget struct {
	Text                string
	HasLeftColon        bool
	HasRightColon       bool
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

type CommandOperatorTarget struct {
	Command             CommandExpression
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

////////////////////////////////////////////// Ids /////////////////////////////////////////////////

type IdType interface {
	FormulationNodeType
	IdType()
}

func (*CommandId) IdType()              {}
func (*PrefixOperatorId) IdType()       {}
func (*PostfixOperatorId) IdType()      {}
func (*InfixOperatorId) IdType()        {}
func (*InfixCommandOperatorId) IdType() {}

type NamedParam struct {
	Name                NameForm
	CurlyParam          *CurlyParam
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// \function:on{A}:to{B}
type CommandId struct {
	Names               []NameForm
	CurlyParam          *CurlyParam
	NamedParams         *[]NamedParam
	ParenParams         *[]NameForm
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// []{} or {}
type CurlyParam struct {
	SquareParams        *[]StructuralFormType
	CurlyParams         []StructuralFormType
	Direction           *DirectionalParam
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

type DirectionParamParamType interface {
	FormulationNodeType
	DirectionParamParamType()
}

func (*NameForm) DirectionParamParamType()              {}
func (*FunctionForm) DirectionParamParamType()          {}
func (*OrdinalCallExpression) DirectionParamParamType() {}

type DirectionalParam struct {
	Name                *NameForm
	SquareParams        []DirectionParamParamType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// \function:on{A}:to{B}/
type InfixCommandId struct {
	Names               []NameForm
	CurlyParam          *CurlyParam
	NamedParams         *[]NamedParam
	ParenParams         *[]NameForm
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// +x
type PrefixOperatorId struct {
	Operator            NonEnclosedNonCommandOperatorTarget
	Param               StructuralFormType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// -x
type PostfixOperatorId struct {
	Operator            NonEnclosedNonCommandOperatorTarget
	Param               StructuralFormType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

type InfixOperatorId struct {
	Lhs                 StructuralFormType
	Operator            NonEnclosedNonCommandOperatorTarget
	Rhs                 StructuralFormType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// A \subset/ B
type InfixCommandOperatorId struct {
	Lhs                 StructuralFormType
	Operator            InfixCommandId
	Rhs                 StructuralFormType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

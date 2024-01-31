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
	Params              []StructuralFormKind
	VarArg              VarArgData
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x * y
type InfixOperatorForm struct {
	Operator            NameForm
	Lhs                 StructuralFormKind
	Rhs                 StructuralFormKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// +y
type PrefixOperatorForm struct {
	Operator            NameForm
	Param               StructuralFormKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x!
type PostfixOperatorForm struct {
	Operator            NameForm
	Param               StructuralFormKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// (x, y)
type TupleForm struct {
	Params              []StructuralFormKind
	VarArg              VarArgData
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// {x | ...}
type ConditionalSetForm struct {
	Target              StructuralFormKind
	VarArg              VarArgData
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x |-> f(x) or (x, y) |-> f(x)
type FunctionLiteralForm struct {
	Lhs                 TupleForm
	Rhs                 StructuralFormKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// [x]{x | f(x)...}
type ConditionalSetIdForm struct {
	Symbols             []StructuralFormKind
	Target              StructuralFormKind
	Condition           FunctionForm
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type PseudoTokenNode struct {
	Text                string
	Type                TokenType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

type PseudoExpression struct {
	Children            []FormulationNodeKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

///////////////////////////////////// Expressions //////////////////////////////////////////////////

// f(x + y, z) or (f + g)(x)
type FunctionCallExpression struct {
	Target              ExpressionKind
	Args                []ExpressionKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
	VarArg              VarArgData
}

// x |-> x + 1 or (x, y) |-> x + y
type FunctionLiteralExpression struct {
	Lhs                 TupleForm
	Rhs                 ExpressionKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// (x + y, z) or (:x:)
type TupleExpression struct {
	Args                []ExpressionKind
	IsInvisible         bool
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// [x]{(x, x+1) | x is \real ; x > 0}
type ConditionalSetExpression struct {
	Symbols             []StructuralFormKind
	Target              ExpressionKind
	Conditions          []ExpressionKind
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
	ParenArgs           *[]ExpressionKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// []{} or {}
type CurlyArg struct {
	CurlyArgs           *[]ExpressionKind
	Direction           *DirectionalParam
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// -x
type PrefixOperatorCallExpression struct {
	Target              OperatorKind
	Arg                 ExpressionKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x!
type PostfixOperatorCallExpression struct {
	Target              OperatorKind
	Arg                 ExpressionKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x + y
type InfixOperatorCallExpression struct {
	Target              OperatorKind
	Lhs                 ExpressionKind
	Rhs                 ExpressionKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

type MultiplexedInfixOperatorCallExpression struct {
	Target              OperatorKind
	Lhs                 []ExpressionKind
	Rhs                 []ExpressionKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x is \y
type IsExpression struct {
	Lhs                 []ExpressionKind
	Rhs                 []KindKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x satisfies \y & \z
type SatisfiesExpression struct {
	Lhs                 []ExpressionKind
	Rhs                 []KindKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x extends \y
type ExtendsExpression struct {
	Lhs                 []ExpressionKind
	Rhs                 []KindKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x as \:y
type AsExpression struct {
	Lhs ExpressionKind
	// the right-hand-side is an ExpressionKind and not a TypeKind
	// to support `x as \:y` and `x as (\:a \:in:/ \:b)`
	Rhs                 ExpressionKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x[1]
type OrdinalCallExpression struct {
	Target              LiteralFormKind
	Args                []ExpressionKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// (x + y).z.a.b
type ChainExpression struct {
	Parts               []ExpressionKind
	HasTrailingOperator bool
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// \\select{statement|specification}:from{x}
type SelectFromBuiltinExpression struct {
	Kinds               []string
	Target              NameForm
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// \\map{x{i{k}}}:to{x{i{k}} + 1}:else{0}
type MapToElseBuiltinExpression struct {
	Target              OrdinalCallExpression
	To                  ExpressionKind
	Else                ExpressionKind // Note: else is optional
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// \:a.b.c:x:y::(inner.label)
type Signature struct {
	MainNames           []string
	NamedGroupNames     []string
	IsInfix             bool
	InnerLabel          *string
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

/////////////////////////////////////////// types //////////////////////////////////////////////////

// \:function:on{\:set}:to{\:set}:/
type InfixCommandType struct {
	Names               []NameForm
	CurlyTypeParam      *CurlyTypeParam
	NamedTypeParams     *[]NamedTypeParam
	ParenTypeParams     *[]ExpressionKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// \:a.b.c:x{\:a & \:b}:y{\:c}
type CommandType struct {
	Names               []NameForm
	CurlyTypeParam      *CurlyTypeParam
	NamedTypeParams     *[]NamedTypeParam
	ParenTypeParams     *[]ExpressionKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

type NamedTypeParam struct {
	Name                NameForm
	CurlyTypeParam      *CurlyTypeParam
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// {}
type CurlyTypeParam struct {
	CurlyTypeParams     *[]ExpressionKind
	TypeDirection       *DirectionalTypeParam
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

type DirectionalTypeParam struct {
	Name                *NameForm
	CurlyTypeParams     []DirectionType
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// #1 or #2
type DirectionType struct {
	Number              uint32
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

/////////////////////////////// Kinds //////////////////////////////////////////////////////////////

// \\type{\:set & \:group}
// \\type{\:set \:in:/ \:set}
type TypeMetaKind struct {
	Types               *[]TypeKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// \\formulation{expression | statement}
type FormulationMetaKind struct {
	Kinds               *[]string
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

////////////////////////////// Colon Equals ////////////////////////////////////////////////////////

// X := (a, b) or f(x) := y
type StructuralColonEqualsForm struct {
	Lhs                 StructuralFormKind
	Rhs                 StructuralFormKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// f(x) := x + 1
type ExpressionColonEqualsItem struct {
	Lhs                 ExpressionKind
	Rhs                 ExpressionKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x + y :=> x
type ExpressionColonArrowItem struct {
	Lhs                 ExpressionKind
	Rhs                 ExpressionKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// x + y :-> x; y
type ExpressionColonDashArrowItem struct {
	Lhs                 ExpressionKind
	Rhs                 []ExpressionKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

////////////////////////////////////// Operators ///////////////////////////////////////////////////

// [x] or [x + y]
type EnclosedNonCommandOperatorTarget struct {
	Target              ExpressionKind
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

// \function:on{A}:to{B}/
type InfixCommandExpression struct {
	Names               []NameForm
	CurlyArg            *CurlyArg
	NamedArgs           *[]NamedArg
	ParenArgs           *[]ExpressionKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

////////////////////////////////////////////// Ids /////////////////////////////////////////////////

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
	CurlyParams         *[]StructuralFormKind
	Direction           *DirectionalParam
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

type DirectionalParam struct {
	Name                *NameForm
	CurlyParams         []DirectionParamParamKind
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
	Param               StructuralFormKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// -x
type PostfixOperatorId struct {
	Operator            NonEnclosedNonCommandOperatorTarget
	Param               StructuralFormKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

type InfixOperatorId struct {
	Lhs                 StructuralFormKind
	Operator            NonEnclosedNonCommandOperatorTarget
	Rhs                 StructuralFormKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

// A \subset/ B
type InfixCommandOperatorId struct {
	Lhs                 StructuralFormKind
	Operator            InfixCommandId
	Rhs                 StructuralFormKind
	CommonMetaData      CommonMetaData
	FormulationMetaData FormulationMetaData
}

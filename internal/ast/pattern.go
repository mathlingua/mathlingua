/*
 * Copyright 2023 Dominic Kramer
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

type StructuralColonEqualsPattern struct {
	Lhs PatternKind
	Rhs PatternKind
}

type NameFormPattern struct {
	Text            string
	IsStropped      bool
	HasQuestionMark bool
	VarArg          VarArgPatternData
}

type SymbolFormPattern struct {
	Text       string
	IsStropped bool
	VarArg     VarArgPatternData
}

type FunctionFormPattern struct {
	Target NameFormPattern
	Params []FormPatternKind
	VarArg VarArgPatternData
}

type ExpressionFormPattern struct {
	Target NameFormPattern
	Params []FormPatternKind
	VarArg VarArgPatternData
}

type OrdinalPattern struct {
	Target LiteralFormPatternKind
	Params []NameFormPattern
}

type InfixOperatorFormPattern struct {
	Operator NameFormPattern
	Lhs      FormPatternKind
	Rhs      FormPatternKind
}

type PrefixOperatorFormPattern struct {
	Operator NameFormPattern
	Param    FormPatternKind
}

type PostfixOperatorFormPattern struct {
	Operator NameFormPattern
	Param    FormPatternKind
}

type TupleFormPattern struct {
	Params []FormPatternKind
	VarArg VarArgPatternData
}

type ConditionalSetFormPattern struct {
	Symbols       []FormPatternKind
	Target        FormPatternKind
	Specification *FunctionFormPattern
	Condition     *FunctionFormPattern
	VarArg        VarArgPatternData
}

type ConditionalSetIdFormPattern struct {
	Symbols       []FormPatternKind
	Target        FormPatternKind
	Specification *FunctionFormPattern
	Condition     *FunctionFormPattern
}

type FunctionLiteralFormPattern struct {
	Lhs TupleFormPattern
	Rhs FormPatternKind
}

type ConditionalSetExpressionPattern struct {
	Target         FormPatternKind
	Specifications []FormPatternKind
	Condition      *FormPatternKind
	VarArg         VarArgPatternData
}

type VarArgPatternData struct {
	IsVarArg     bool
	VarArgNames  []NameFormPattern
	VarArgBounds []NameFormPattern
}

type InfixCommandOperatorPattern struct {
	Lhs      FormPatternKind
	Operator InfixCommandPattern
	Rhs      FormPatternKind
}

type InfixCommandPattern struct {
	Signature   string
	Names       []NameFormPattern
	CurlyArg    *CurlyPattern
	NamedGroups *[]NamedGroupPattern
	ParenArgs   *[]NameFormPattern
}

type CommandPattern struct {
	Signature   string
	Names       []NameFormPattern
	CurlyArg    *CurlyPattern
	NamedGroups *[]NamedGroupPattern
	ParenArgs   *[]NameFormPattern
}

type CurlyPattern struct {
	SquareArgs *[]FormPatternKind
	CurlyArgs  *[]FormPatternKind
}

type ChainExpressionPattern struct {
	Parts []FormPatternKind
}

type NamedGroupPattern struct {
	Name  NameFormPattern
	Curly CurlyPattern
}

type SpecAliasPattern struct {
	Lhs  PatternKind
	Name NameFormPattern
	Rhs  PatternKind
}

type AliasPattern struct {
	Lhs PatternKind
	Rhs PatternKind
}

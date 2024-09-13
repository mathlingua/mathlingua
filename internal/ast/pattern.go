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

// A pattern describes the shape of inputs to a Defines, Describes, States
// provides, expression alias, or spec alias.
type PatternKind interface {
	PatternKind()
	GetVarArgData() VarArgPatternData
}

func (*NameFormPattern) PatternKind()                 {}
func (*FunctionFormPattern) PatternKind()             {}
func (*TupleFormPattern) PatternKind()                {}
func (*ConditionalSetExpressionPattern) PatternKind() {}
func (*ConditionalSetFormPattern) PatternKind()       {}
func (*ConditionalSetIdFormPattern) PatternKind()     {}
func (*FunctionLiteralFormPattern) PatternKind()      {}
func (*InfixOperatorFormPattern) PatternKind()        {}
func (*PrefixOperatorFormPattern) PatternKind()       {}
func (*PostfixOperatorFormPattern) PatternKind()      {}
func (*OrdinalPattern) PatternKind()                  {}

func (*StructuralColonEqualsPattern) PatternKind() {}
func (*InfixCommandOperatorPattern) PatternKind()  {}
func (*InfixCommandPattern) PatternKind()          {}
func (*CommandPattern) PatternKind()               {}
func (*NamedGroupPattern) PatternKind()            {}
func (*ChainExpressionPattern) PatternKind()       {}

func (*SpecAliasPattern) PatternKind() {}
func (*AliasPattern) PatternKind()     {}

type StructuralColonEqualsPattern struct {
	Lhs PatternKind
	Rhs PatternKind
}

type FormPatternKind interface {
	PatternKind
	FormPatternKind()
}

func (*NameFormPattern) FormPatternKind()              {}
func (*FunctionFormPattern) FormPatternKind()          {}
func (*TupleFormPattern) FormPatternKind()             {}
func (*ConditionalSetFormPattern) FormPatternKind()    {}
func (*ConditionalSetIdFormPattern) FormPatternKind()  {}
func (*FunctionLiteralFormPattern) FormPatternKind()   {}
func (*InfixOperatorFormPattern) FormPatternKind()     {}
func (*PrefixOperatorFormPattern) FormPatternKind()    {}
func (*PostfixOperatorFormPattern) FormPatternKind()   {}
func (*StructuralColonEqualsPattern) FormPatternKind() {}

type NameFormPattern struct {
	Text            string
	IsStropped      bool
	HasQuestionMark bool
	VarArg          VarArgPatternData
}

type FunctionFormPattern struct {
	Target NameFormPattern
	Params []FormPatternKind
	VarArg VarArgPatternData
}

type LiteralFormPatternKind interface {
	LiteralFormPatternKind()
}

func (*NameFormPattern) LiteralFormPatternKind()             {}
func (*FunctionFormPattern) LiteralFormPatternKind()         {}
func (*TupleFormPattern) LiteralFormPatternKind()            {}
func (*ConditionalSetFormPattern) LiteralFormPatternKind()   {}
func (*ConditionalSetIdFormPattern) LiteralFormPatternKind() {}
func (*FunctionLiteralFormPattern) LiteralFormPatternKind()  {}

type OrdinalPattern struct {
	Target LiteralFormPatternKind
	Params []NameFormPattern
}

type DirectionParamParamPatternKind interface {
	DirectionParamParamPatternKind()
}

func (*NameFormPattern) DirectionParamParamPatternKind()     {}
func (*FunctionFormPattern) DirectionParamParamPatternKind() {}
func (*OrdinalPattern) DirectionParamParamPatternKind()      {}

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
	Target        FormPatternKind
	Specification FunctionFormPattern
	Condition     *FunctionFormPattern
	VarArg        VarArgPatternData
}

type ConditionalSetIdFormPattern struct {
	Symbols       []FormPatternKind
	Target        FormPatternKind
	Specification FunctionFormPattern
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

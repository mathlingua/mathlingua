/*
 * Copyright 2024 Dominic Kramer
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
func (*SymbolFormPattern) PatternKind()               {}
func (*FunctionFormPattern) PatternKind()             {}
func (*ExpressionFormPattern) PatternKind()           {}
func (*TupleFormPattern) PatternKind()                {}
func (*ConditionalSetExpressionPattern) PatternKind() {}
func (*ConditionalSetFormPattern) PatternKind()       {}
func (*ConditionalSetIdFormPattern) PatternKind()     {}
func (*FunctionLiteralFormPattern) PatternKind()      {}
func (*InfixOperatorFormPattern) PatternKind()        {}
func (*PrefixOperatorFormPattern) PatternKind()       {}
func (*PostfixOperatorFormPattern) PatternKind()      {}
func (*OrdinalPattern) PatternKind()                  {}
func (*StructuralColonEqualsPattern) PatternKind()    {}
func (*InfixCommandOperatorPattern) PatternKind()     {}
func (*InfixCommandPattern) PatternKind()             {}
func (*CommandPattern) PatternKind()                  {}
func (*NamedGroupPattern) PatternKind()               {}
func (*ChainExpressionPattern) PatternKind()          {}
func (*SpecAliasPattern) PatternKind()                {}
func (*AliasPattern) PatternKind()                    {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type FormPatternKind interface {
	PatternKind
	FormPatternKind()
}

func (*NameFormPattern) FormPatternKind()              {}
func (*SymbolFormPattern) FormPatternKind()            {}
func (*FunctionFormPattern) FormPatternKind()          {}
func (*ExpressionFormPattern) FormPatternKind()        {}
func (*TupleFormPattern) FormPatternKind()             {}
func (*ConditionalSetFormPattern) FormPatternKind()    {}
func (*ConditionalSetIdFormPattern) FormPatternKind()  {}
func (*FunctionLiteralFormPattern) FormPatternKind()   {}
func (*InfixOperatorFormPattern) FormPatternKind()     {}
func (*PrefixOperatorFormPattern) FormPatternKind()    {}
func (*PostfixOperatorFormPattern) FormPatternKind()   {}
func (*StructuralColonEqualsPattern) FormPatternKind() {}

////////////////////////////////////////////////////////////////////////////////////////////////////

type LiteralFormPatternKind interface {
	LiteralFormPatternKind()
}

func (*NameFormPattern) LiteralFormPatternKind()             {}
func (*SymbolFormPattern) LiteralFormPatternKind()           {}
func (*FunctionFormPattern) LiteralFormPatternKind()         {}
func (*ExpressionFormPattern) LiteralFormPatternKind()       {}
func (*TupleFormPattern) LiteralFormPatternKind()            {}
func (*ConditionalSetFormPattern) LiteralFormPatternKind()   {}
func (*ConditionalSetIdFormPattern) LiteralFormPatternKind() {}
func (*FunctionLiteralFormPattern) LiteralFormPatternKind()  {}

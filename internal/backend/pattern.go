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

package backend

// A pattern describes the shape of inputs to a Defines, Describes, States
// provides, expression alias, or spec alias.
type StaticPatternType interface {
	StaticPatternType()
}

func (NameStaticPattern) StaticPatternType()           {}
func (FunctionStaticPattern) StaticPatternType()       {}
func (TupleStaticPattern) StaticPatternType()          {}
func (SetStaticPattern) StaticPatternType()            {}
func (InfixStaticPattern) StaticPatternType()          {}
func (PrefixStaticPattern) StaticPatternType()         {}
func (PostfixStaticPattern) StaticPatternType()        {}
func (CommandStaticPattern) StaticPatternType()        {}
func (MemberNameStaticPattern) StaticPatternType()     {}
func (MemberFunctionStaticPattern) StaticPatternType() {}
func (MemberInfixStaticPattern) StaticPatternType()    {}
func (MemberPrefixStaticPattern) StaticPatternType()   {}
func (MemberPostfixStaticPattern) StaticPatternType()  {}
func (SpecStaticPattern) StaticPatternType()           {}

type NameStaticPattern struct {
	Name string
}

type FunctionStaticPattern struct {
	Name   string
	Inputs []StaticPatternType
	Output StaticPatternType
}

type TupleStaticPattern struct {
	Items []StaticPatternType
}

type SetStaticPattern struct {
	Target    StaticPatternType
	Condition StaticPatternType
}

type InfixStaticPattern struct {
	Name string
	Lhs  StaticPatternType
	Rhs  StaticPatternType
}

type PrefixStaticPattern struct {
	Name string
	Arg  StaticPatternType
}

type PostfixStaticPattern struct {
	Name string
	Arg  StaticPatternType
}

type CommandStaticPattern struct {
	Signatures  string
	CurlyArgs   []StaticPatternType
	ParenArgs   []StaticPatternType
	NamedGroups []NamedGroupStaticPattern
}

type NamedGroupStaticPattern struct {
	Name string
	Args []StaticPatternType
}

type MemberNameStaticPattern struct {
	Target string
	Member NameStaticPattern
}

type MemberFunctionStaticPattern struct {
	Target string
	Member FunctionStaticPattern
}

type MemberInfixStaticPattern struct {
	Target string
	Member InfixStaticPattern
}

type MemberPrefixStaticPattern struct {
	Target string
	Member PrefixStaticPattern
}

type MemberPostfixStaticPattern struct {
	Target string
	Member PostfixStaticPattern
}

type SpecStaticPattern struct {
	Lhs  StaticPatternType
	Name string
	Rhs  StaticPatternType
}

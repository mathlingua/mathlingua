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

type IStaticPattern interface {
	IStaticPattern()
}

func (NameStaticPattern) IStaticPattern()           {}
func (FunctionStaticPattern) IStaticPattern()       {}
func (TupleStaticPattern) IStaticPattern()          {}
func (SetStaticPattern) IStaticPattern()            {}
func (InfixStaticPattern) IStaticPattern()          {}
func (PrefixStaticPattern) IStaticPattern()         {}
func (PostfixStaticPattern) IStaticPattern()        {}
func (CommandStaticPattern) IStaticPattern()        {}
func (MemberNameStaticPattern) IStaticPattern()     {}
func (MemberFunctionStaticPattern) IStaticPattern() {}
func (MemberInfixStaticPattern) IStaticPattern()    {}
func (MemberPrefixStaticPattern) IStaticPattern()   {}
func (MemberPostfixStaticPattern) IStaticPattern()  {}
func (SpecStaticPattern) IStaticPattern()           {}

type NameStaticPattern struct {
	Name string
}

type FunctionStaticPattern struct {
	Name   string
	Inputs []IStaticPattern
	Output IStaticPattern
}

type TupleStaticPattern struct {
	Items []IStaticPattern
}

type SetStaticPattern struct {
	Target    IStaticPattern
	Condition IStaticPattern
}

type InfixStaticPattern struct {
	Name string
	Lhs  IStaticPattern
	Rhs  IStaticPattern
}

type PrefixStaticPattern struct {
	Name string
	Arg  IStaticPattern
}

type PostfixStaticPattern struct {
	Name string
	Arg  IStaticPattern
}

type CommandStaticPattern struct {
	CurlyArgs   []IStaticPattern
	ParenArgs   []IStaticPattern
	NamedGroups []NamedGroupStaticPattern
}

type NamedGroupStaticPattern struct {
	Name string
	Args []IStaticPattern
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
	Lhs  IStaticPattern
	Name string
	Rhs  IStaticPattern
}

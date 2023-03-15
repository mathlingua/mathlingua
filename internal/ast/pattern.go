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

type StaticPattern interface {
	StaticPattern()
}

func (NameStaticPattern) StaticPattern()           {}
func (FunctionStaticPattern) StaticPattern()       {}
func (TupleStaticPattern) StaticPattern()          {}
func (SetStaticPattern) StaticPattern()            {}
func (InfixStaticPattern) StaticPattern()          {}
func (PrefixStaticPattern) StaticPattern()         {}
func (PostfixStaticPattern) StaticPattern()        {}
func (CommandStaticPattern) StaticPattern()        {}
func (MemberNameStaticPattern) StaticPattern()     {}
func (MemberFunctionStaticPattern) StaticPattern() {}
func (MemberInfixStaticPattern) StaticPattern()    {}
func (MemberPrefixStaticPattern) StaticPattern()   {}
func (MemberPostfixStaticPattern) StaticPattern()  {}

type NameStaticPattern struct {
	Name string
}

type FunctionStaticPattern struct {
	Name   string
	Inputs []StaticPattern
	Output StaticPattern
}

type TupleStaticPattern struct {
	Items []StaticPattern
}

type SetStaticPattern struct {
	Target    StaticPattern
	Condition StaticPattern
}

type InfixStaticPattern struct {
	Name string
	Lhs  StaticPattern
	Rhs  StaticPattern
}

type PrefixStaticPattern struct {
	Name string
	Arg  StaticPattern
}

type PostfixStaticPattern struct {
	Name string
	Arg  StaticPattern
}

type CommandStaticPattern struct {
	CurlyArgs   []StaticPattern
	ParenArgs   []StaticPattern
	NamedGroups []NamedGroupStaticPattern
}

type NamedGroupStaticPattern struct {
	Name string
	Args []StaticPattern
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

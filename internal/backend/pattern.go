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

type FormPattern interface {
	FormPattern()
}

func (NameFormPattern) FormPattern()     {}
func (FunctionFormPattern) FormPattern() {}
func (TupleFormPattern) FormPattern()    {}
func (SetFormPattern) FormPattern()      {}

type NameFormPattern struct {
	Name string
}

type FunctionFormPattern struct {
	TargetName string
	Inputs     []FormPattern
	Output     FormPattern
}

type TupleFormPattern struct {
	Items []FormPattern
}

type SetFormPattern struct {
	Target    FormPattern
	Condition FormPattern
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type InputPattern interface {
	InputPattern()
}

func (InfixInputPattern) InputPattern()   {}
func (PrefixInputPattern) InputPattern()  {}
func (PostfixInputPattern) InputPattern() {}
func (CommandInputPattern) InputPattern() {}

type InfixInputPattern struct {
	Lhs FormPattern
	Rhs FormPattern
}

type PrefixInputPattern struct {
	Arg FormPattern
}

type PostfixInputPattern struct {
	Arg FormPattern
}

type CommandInputPattern struct {
	CurlyArgs   []FormPattern
	ParenArgs   []FormPattern
	NamedGroups []NamedGroupInput
}

type NamedGroupInput struct {
	Name string
	Args []FormPattern
}

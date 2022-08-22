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

package phase4

import "mathlingua/internal/ast"

type Phase4Data struct {
	Start ast.Position
	End   ast.Position
}

type Group struct {
	Id       *string
	Sections []Section
	Data     Phase4Data
}

type Section struct {
	Name string
	Args []Argument
	Data Phase4Data
}

type Argument struct {
	IsInline bool
	Arg      ArgumentDataType
	Data     Phase4Data
}

type TextArgumentData struct {
	Text string
	Data Phase4Data
}

type FormulationArgumentData struct {
	Text string
	Data Phase4Data
}

type ArgumentTextArgumentData struct {
	Text string
	Data Phase4Data
}

type ArgumentDataType interface {
	ArgumentDataType()
}

func (Group) ArgumentDataType()                    {}
func (TextArgumentData) ArgumentDataType()         {}
func (FormulationArgumentData) ArgumentDataType()  {}
func (ArgumentTextArgumentData) ArgumentDataType() {}

type TextBlock struct {
	Text string
	Data Phase4Data
}

type Root struct {
	Nodes []TopLevelNodeType
	Data  Phase4Data
}

type TopLevelNodeType interface {
	TopLevelNodeType()
}

func (TextBlock) TopLevelNodeType() {}
func (Group) TopLevelNodeType()     {}

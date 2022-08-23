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

type Phase4MetaData struct {
	Start ast.Position
}

type Group struct {
	Id       *string
	Sections []Section
	MetaData Phase4MetaData
}

type Section struct {
	Name     string
	Args     []Argument
	MetaData Phase4MetaData
}

type Argument struct {
	IsInline bool
	Arg      ArgumentDataType
	MetaData Phase4MetaData
}

type TextArgumentData struct {
	Text     string
	MetaData Phase4MetaData
}

type FormulationArgumentData struct {
	Text     string
	MetaData Phase4MetaData
}

type ArgumentTextArgumentData struct {
	Text     string
	MetaData Phase4MetaData
}

type ArgumentDataType interface {
	ArgumentDataType()
}

func (Group) ArgumentDataType()                    {}
func (TextArgumentData) ArgumentDataType()         {}
func (FormulationArgumentData) ArgumentDataType()  {}
func (ArgumentTextArgumentData) ArgumentDataType() {}

type TextBlock struct {
	Text     string
	MetaData Phase4MetaData
}

type Root struct {
	Nodes    []TopLevelNodeType
	MetaData Phase4MetaData
}

type TopLevelNodeType interface {
	TopLevelNodeType()
}

func (TextBlock) TopLevelNodeType() {}
func (Group) TopLevelNodeType()     {}

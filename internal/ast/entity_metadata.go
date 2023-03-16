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

type DescribesMetaData struct {
	Input       StaticPatternType
	Output      StaticPatternType
	Usings      []StaticPatternType
	When        []ConstraintType
	Extends     []ConstraintType
	ExpAliases  []ExpAliasInfoType
	SpecAliases []SpecAliasInfo
	Written     []WrittenInfo
	Called      []CalledInfo
}

type DefinesMetaData struct {
	Input       StaticPatternType
	Output      StaticPatternType
	Usings      []StaticPatternType
	When        []ConstraintType
	Means       []ConstraintType
	ExpAliases  []ExpAliasInfoType
	SpecAliases []SpecAliasInfo
	Written     []WrittenInfo
	Called      []CalledInfo
}

type StatesMetaData struct {
	Input       StaticPatternType
	Output      StaticPatternType
	Usings      []StaticPatternType
	ExpAliases  []ExpAliasInfoType
	SpecAliases []SpecAliasInfo
	Written     []WrittenInfo
	Called      []CalledInfo
}
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

package ast

type Formulation[T NodeType] struct {
	RawText string
	Root    T
	Label   *string
}

type Clause = Formulation[NodeType]
type Target = Formulation[NodeType]
type Spec = Formulation[NodeType]

type AndSection struct {
	Clauses []Clause
}

type AndGroup struct {
	And AndSection
}

type NotSection struct {
	Clause Clause
}

type NotGroup struct {
	Not NotSection
}

type OrSection struct {
	Clauses []Clause
}

type OrGroup struct {
	Or OrSection
}

type ExistsSection struct {
	Targets []Target
}

type WhereSection struct {
	Specs []Spec
}

type SuchThatSection struct {
	Clauses []Clause
}

type ExistsGroup struct {
	Exists   ExistsSection
	Where    *WhereSection
	SuchThat SuchThatSection
}

type ExistsUniqueSection struct {
	Specs []Spec
}

type ExistsUniqueGroup struct {
	ExistsUnique ExistsUniqueSection
	Where        *WhereSection
	SuchThat     SuchThatSection
}

type ForAllSection struct {
	Targets []Target
}

type ThenSection struct {
	Clauses []Clause
}

type ForAllGroup struct {
	ForAll   ForAllSection
	Where    *WhereSection
	SuchThat *SuchThatSection
	Then     ThenSection
}

type IfSection struct {
	Clauses []Clause
}

type IfGroup struct {
	If   IfSection
	Then ThenSection
}

type IffSection struct {
	Clauses []Clause
}

type IffGroup struct {
	Iff  IffSection
	Then ThenSection
}

type GeneratedSection struct{}

type FromSection struct {
	Items []NodeType
}

type WhenSection struct {
	Items []NodeType
}

type GeneratedGroup struct {
	Generated GeneratedSection
	From      FromSection
	When      *WhenSection
}

type PiecewiseSection struct {
}

type ElseSection struct {
	Items []NodeType
}

type PiecewiseGroup struct {
	Piecewise PiecewiseSection
	When      *WhenSection
	Then      *ThenSection
	Else      *ElseSection
}

type MatchingSection struct {
	Items []NodeType
}

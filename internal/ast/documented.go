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

type TextItemKind interface {
	TextItemKind()
}

func (*StringItem) TextItemKind()       {}
func (*SubstitutionItem) TextItemKind() {}

type StringItem struct {
	Text string
}

type SubstitutionItem struct {
	Name       string
	NameSuffix string
	IsVarArg   bool
	Prefix     string
	Suffix     string
	Infix      string
}

type CalledSummary struct {
	RawCalled    string
	ParsedCalled []TextItemKind
	Errors       []string
}

type WrittenSummary struct {
	RawWritten    string
	ParsedWritten []TextItemKind
	Errors        []string
}

type WritingSummary struct {
	RawWritten    string
	ParsedWriting []TextItemKind
	Errors        []string
}

type DocumentedSummary struct {
	Written []WrittenSummary
	Writing []WritingSummary
	Called  []CalledSummary
}

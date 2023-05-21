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

import (
	"fmt"
	"mathlingua/internal/ast"
)

type NodeType string

const (
	RootType                     NodeType = "RootType"
	TextBlockType                NodeType = "TextBlockType"
	DocumentType                 NodeType = "DocumentType"
	GroupType                    NodeType = "GroupType"
	SectionType                  NodeType = "SectionType"
	ArgumentType                 NodeType = "ArgumentType"
	TextArgumentDataKind         NodeType = "TextArgumentDataKind"
	FormulationArgumentDataKind  NodeType = "FormulationArgumentDataKind"
	ArgumentTextArgumentDataKind NodeType = "ArgumentTextArgumentDataKind"
)

type MetaData struct {
	Start ast.Position
	Key   int
	Id    string
}

type Node interface {
	ToCode(writer ITextCodeWriter)
	Size() int
	ChildAt(index int) Node
}

type Group struct {
	Type     NodeType
	Id       *string
	Sections []Section
	MetaData MetaData
}

func (g *Group) write(indent int, writer ITextCodeWriter) {
	if g.Id != nil {
		writer.WriteId(fmt.Sprintf("[%s]", *g.Id))
		writer.WriteNewline()
	}

	for index, s := range g.Sections {
		if index > 0 {
			writer.WriteNewline()
			writer.WriteIndent(indent)
		}
		s.write(indent, writer)
	}
}

func (g *Group) ToCode(writer ITextCodeWriter) {
	g.write(0, writer)
}

func (g *Group) Size() int {
	return len(g.Sections)
}

func (g *Group) ChildAt(index int) Node {
	return &g.Sections[index]
}

func (g *Group) Start() ast.Position {
	return g.MetaData.Start
}

type Section struct {
	Type     NodeType
	Name     string
	Args     []Argument
	MetaData MetaData
}

func (s *Section) write(indent int, writer ITextCodeWriter) {
	writer.WriteHeader(fmt.Sprintf("%s:", s.Name))

	isFirstInline := true
	for _, a := range s.Args {
		if a.IsInline {
			if isFirstInline {
				writer.WriteSpace()
			} else {
				writer.WriteText(",")
				writer.WriteSpace()
			}
			a.write(indent, writer)
			isFirstInline = false
		} else {
			writer.WriteNewline()
			writer.WriteIndent(indent)
			writer.WriteDotSpace()
			a.write(indent+2, writer)
			isFirstInline = true
		}
	}
}

func (s *Section) ToCode(writer ITextCodeWriter) {
	s.write(0, writer)
}

func (s *Section) Size() int {
	return len(s.Args)
}

func (s *Section) ChildAt(index int) Node {
	return &s.Args[index]
}

type Argument struct {
	Type     NodeType
	IsInline bool
	Arg      ArgumentDataKind
	MetaData MetaData
}

func (a *Argument) write(indent int, writer ITextCodeWriter) {
	switch t := a.Arg.(type) {
	case *TextArgumentData:
		t.write(indent, writer)
	case *FormulationArgumentData:
		t.write(indent, writer)
	case *ArgumentTextArgumentData:
		t.write(indent, writer)
	case *Group:
		t.write(indent, writer)
	default:
		panic(fmt.Sprintf("Unexpected Argument type %#v", a.Arg))
	}
}

func (a *Argument) ToCode(writer ITextCodeWriter) {
	a.write(0, writer)
}

func (a *Argument) Size() int {
	return a.Arg.Size()
}

func (a *Argument) ChildAt(index int) Node {
	return a.Arg.ChildAt(index)
}

type TextArgumentData struct {
	Type     NodeType
	Text     string
	MetaData MetaData
}

func (t *TextArgumentData) write(indent int, writer ITextCodeWriter) {
	writer.WriteText(fmt.Sprintf("\"%s\"", t.Text))
}

func (t *TextArgumentData) ToCode(writer ITextCodeWriter) {
	t.write(0, writer)
}

func (t *TextArgumentData) Size() int {
	return 0
}

func (t *TextArgumentData) ChildAt(index int) Node {
	panic("Cannot get children of an TextArgumentData")
}

type FormulationArgumentData struct {
	Type     NodeType
	Text     string
	MetaData MetaData
}

func (f *FormulationArgumentData) write(indent int, writer ITextCodeWriter) {
	writer.WriteFormulation(fmt.Sprintf("'%s'", f.Text))
}

func (f *FormulationArgumentData) ToCode(writer ITextCodeWriter) {
	f.write(0, writer)
}

func (f *FormulationArgumentData) Size() int {
	return 0
}

func (f *FormulationArgumentData) ChildAt(index int) Node {
	panic("Cannot get children of an FormulationArgumentData")
}

type ArgumentTextArgumentData struct {
	Type     NodeType
	Text     string
	MetaData MetaData
}

func (a *ArgumentTextArgumentData) write(indent int, writer ITextCodeWriter) {
	writer.WriteDirect(a.Text)
}

func (a *ArgumentTextArgumentData) ToCode(writer ITextCodeWriter) {
	a.write(0, writer)
}

func (a *ArgumentTextArgumentData) Size() int {
	return 0
}

func (a *ArgumentTextArgumentData) ChildAt(index int) Node {
	panic("Cannot get children of an ArgumentTextArgumentData")
}

type ArgumentDataKind interface {
	Node
	ArgumentDataKind()
}

func (*Group) ArgumentDataKind()                    {}
func (*TextArgumentData) ArgumentDataKind()         {}
func (*FormulationArgumentData) ArgumentDataKind()  {}
func (*ArgumentTextArgumentData) ArgumentDataKind() {}

type TextBlock struct {
	Type     NodeType
	Text     string
	MetaData MetaData
}

func (t *TextBlock) write(indent int, writer ITextCodeWriter) {
	writer.WriteTextBlock(fmt.Sprintf("::%s::", t.Text))
	writer.WriteNewline()
}

func (t *TextBlock) ToCode(writer ITextCodeWriter) {
	t.write(0, writer)
}

func (t *TextBlock) Size() int {
	return 0
}

func (t *TextBlock) ChildAt(index int) Node {
	panic("Cannot get children of an TextBlock")
}

func (t *TextBlock) Start() ast.Position {
	return t.MetaData.Start
}

type Document struct {
	Type     NodeType
	Nodes    []TopLevelNodeKind
	MetaData MetaData
}

func (r *Document) write(indent int, writer ITextCodeWriter) {
	for _, node := range r.Nodes {
		node.ToCode(writer)
		writer.WriteNewline()
		writer.WriteNewline()
		writer.WriteNewline()
	}
}

func (r *Document) ToCode(writer ITextCodeWriter) {
	r.write(0, writer)
}

func (r *Document) Size() int {
	return len(r.Nodes)
}

func (r *Document) ChildAt(index int) Node {
	return r.Nodes[index]
}

type TopLevelNodeKind interface {
	Node
	TopLevelNodeKind()
	Start() ast.Position
}

func (*TextBlock) TopLevelNodeKind() {}
func (*Group) TopLevelNodeKind()     {}

type Root struct {
	Type      NodeType
	Documents map[ast.Path]Document
	MetaData  MetaData
}

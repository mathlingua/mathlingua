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
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/shared"
	"mathlingua/internal/mlglib"
)

func Parse(lexer3 shared.Lexer, tracker frontend.DiagnosticTracker) Document {
	parser := phase4Parser{
		lexer:   lexer3,
		tracker: tracker,
		keyGen:  mlglib.NewKeyGenerator(),
	}
	doc := parser.document()
	return doc
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type phase4Parser struct {
	lexer   shared.Lexer
	tracker frontend.DiagnosticTracker
	keyGen  mlglib.KeyGenerator
}

func (p *phase4Parser) appendDiagnostic(message string, position ast.Position) {
	p.tracker.Append(frontend.Diagnostic{
		Type:     frontend.Error,
		Origin:   frontend.Phase4ParserOrigin,
		Message:  message,
		Position: position,
	})
}

func (p *phase4Parser) has(tokenType ast.TokenType) bool {
	return p.lexer.HasNext() && p.lexer.Peek().Type == tokenType
}

func (p *phase4Parser) skipAheadPast(end ast.TokenType, unterminatedMessage string) {
	for p.lexer.HasNext() && !p.has(end) {
		next := p.lexer.Next()
		p.appendDiagnostic(fmt.Sprintf("Unexpected text '%s'", next.Text), next.Position)
	}

	if p.has(end) {
		p.lexer.Next() // absorb the end
	} else {
		p.appendDiagnostic(unterminatedMessage, p.lexer.Position())
	}
}

func (p *phase4Parser) document() Document {
	start := p.lexer.Position()
	nodes := make([]TopLevelNodeType, 0)
	for p.lexer.HasNext() {
		peek := p.lexer.Peek()
		if peek.Type == ast.Id {
			id := p.lexer.Next()
			if group, ok := p.group(&id.Text); ok {
				nodes = append(nodes, group)
			} else {
				p.appendDiagnostic("Expected a group to follow", id.Position)
			}
		} else if peek.Type == ast.BeginGroup {
			if group, ok := p.group(nil); ok {
				nodes = append(nodes, group)
			} else {
				p.appendDiagnostic("Expected a group", peek.Position)
			}
		} else if peek.Type == ast.TextBlock {
			textBlock := p.lexer.Next()
			nodes = append(nodes, TextBlock{
				Type: TextBlockType,
				Text: textBlock.Text,
				MetaData: MetaData{
					Start: textBlock.Position,
					Key:   p.keyGen.Next(),
				},
			})
		} else {
			// skip the unknown token
			next := p.lexer.Next()
			p.appendDiagnostic("Unexpected text", next.Position)
		}
	}
	return Document{
		Type:  DocumentType,
		Nodes: nodes,
		MetaData: MetaData{
			Start: start,
			Key:   p.keyGen.Next(),
		},
	}
}

func (p *phase4Parser) group(id *string) (Group, bool) {
	if !p.has(ast.BeginGroup) {
		return Group{}, false
	}

	begin := p.lexer.Next() // skip the BeginGroup
	sections := make([]Section, 0)
	for p.lexer.HasNext() && !p.has(ast.EndGroup) {
		if section, ok := p.section(); ok {
			sections = append(sections, section)
		} else {
			next := p.lexer.Next()
			p.appendDiagnostic("Expected a section", next.Position)
		}
	}

	p.skipAheadPast(ast.EndGroup, "Unterminated group")

	return Group{
		Type:     GroupType,
		Id:       id,
		Sections: sections,
		MetaData: MetaData{
			Start: begin.Position,
			Key:   p.keyGen.Next(),
		},
	}, true
}

func (p *phase4Parser) section() (Section, bool) {
	if !p.has(ast.BeginSection) {
		return Section{}, false
	}

	begin := p.lexer.Next() // skip the BeginSection
	var name string
	if p.has(ast.Name) {
		name = p.lexer.Next().Text
	} else {
		p.appendDiagnostic("Expected a <name>:", begin.Position)
	}

	args := make([]Argument, 0)
	for p.lexer.HasNext() && !p.has(ast.EndSection) {
		if arg, ok := p.argument(); ok {
			args = append(args, arg)
		} else {
			next := p.lexer.Next()
			p.appendDiagnostic(
				fmt.Sprintf("Expected an argument but found '%s'", next.Text), next.Position)
		}
	}

	p.skipAheadPast(ast.EndSection, "Unterminated section")
	return Section{
		Type: SectionType,
		Name: name,
		Args: args,
		MetaData: MetaData{
			Start: begin.Position,
			Key:   p.keyGen.Next(),
		},
	}, true
}

func (p *phase4Parser) argument() (Argument, bool) {
	if p.has(ast.BeginInlineArgument) {
		p.lexer.Next() // skip the BeginInlineArgument
		start := p.lexer.Position()
		var arg Argument
		found := false
		if data, ok := p.argumentData(); ok {
			found = true
			arg = Argument{
				Type:     ArgumentType,
				IsInline: true,
				Arg:      data,
				MetaData: MetaData{
					Start: start,
					Key:   p.keyGen.Next(),
				},
			}
		} else {
			p.appendDiagnostic("Expected an argument", start)
		}
		p.skipAheadPast(ast.EndInlineArgument, "Unterminated argument")
		return arg, found
	}

	if p.has(ast.BeginDotSpaceArgument) {
		p.lexer.Next() // skip the BeginDotSpaceArgument
		start := p.lexer.Position()
		var arg Argument
		found := false
		if data, ok := p.argumentData(); ok {
			found = true
			arg = Argument{
				Type:     ArgumentType,
				IsInline: false,
				Arg:      data,
				MetaData: MetaData{
					Start: start,
					Key:   p.keyGen.Next(),
				},
			}
		} else {
			p.appendDiagnostic("Expected an argument", start)
		}
		p.skipAheadPast(ast.EndDotSpaceArgument, "Unterminated argument")
		return arg, found
	}

	return Argument{}, false
}

func (p *phase4Parser) argumentData() (ArgumentDataType, bool) {
	if p.has(ast.ArgumentText) {
		arg := p.lexer.Next()
		return ArgumentTextArgumentData{
			Type: ArgumentTextArgumentDataType,
			Text: arg.Text,
			MetaData: MetaData{
				Start: arg.Position,
				Key:   p.keyGen.Next(),
			},
		}, true
	}

	if p.has(ast.FormulationTokenType) {
		arg := p.lexer.Next()
		return FormulationArgumentData{
			Type: FormulationArgumentDataType,
			Text: arg.Text,
			MetaData: MetaData{
				Start: arg.Position,
				Key:   p.keyGen.Next(),
			},
		}, true
	}

	if p.has(ast.Text) {
		arg := p.lexer.Next()
		return TextArgumentData{
			Type: TextArgumentDataType,
			Text: arg.Text,
			MetaData: MetaData{
				Start: arg.Position,
				Key:   p.keyGen.Next(),
			},
		}, true
	}

	if p.has(ast.Id) {
		id := p.lexer.Next()
		idText := id.Text
		return p.group(&idText)
	}

	return p.group(nil)
}

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
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/shared"
)

func Parse(lexer3 shared.Lexer) (Root, []frontend.Diagnostic) {
	parser := phase4Parser{
		lexer:       lexer3,
		diagnostics: make([]frontend.Diagnostic, 0),
	}
	root := parser.root()
	return root, parser.diagnostics
}

///////////////////////////////////////////////////////////////

type phase4Parser struct {
	lexer       shared.Lexer
	diagnostics []frontend.Diagnostic
}

func (p *phase4Parser) appendDiagnostic(message string, position ast.Position) {
	p.diagnostics = append(p.diagnostics, frontend.Diagnostic{
		Type:     frontend.Error,
		Origin:   frontend.Phase4ParserOrigin,
		Message:  message,
		Position: position,
	})
}

func (p *phase4Parser) has(tokenType shared.TokenType) bool {
	return p.lexer.HasNext() && p.lexer.Peek().Type == tokenType
}

func (p *phase4Parser) root() Root {
	start := p.lexer.Position()
	nodes := make([]TopLevelNodeType, 0)
	for p.lexer.HasNext() {
		peek := p.lexer.Peek()
		if peek.Type == shared.Id {
			id := p.lexer.Next()
			if group, ok := p.topLevelGroup(&id.Text); ok {
				nodes = append(nodes, group)
			} else {
				p.appendDiagnostic("Expected a group to follow", id.Position)
			}
		} else if peek.Type == shared.BeginTopLevelGroup {
			if group, ok := p.topLevelGroup(nil); ok {
				nodes = append(nodes, group)
			} else {
				p.appendDiagnostic("Expected a group", peek.Position)
			}
		} else if peek.Type == shared.TextBlock {
			textBlock := p.lexer.Next()
			nodes = append(nodes, TextBlock{
				Text: textBlock.Text,
				MetaData: Phase4MetaData{
					Start: textBlock.Position,
				},
			})
		} else {
			// skip the unknown token
			next := p.lexer.Next()
			p.appendDiagnostic("Unexpected text", next.Position)
		}
	}
	return Root{
		Nodes: nodes,
		MetaData: Phase4MetaData{
			Start: start,
		},
	}
}

func (p *phase4Parser) topLevelGroup(id *string) (Group, bool) {
	if !p.has(shared.BeginTopLevelGroup) {
		return Group{}, false
	}

	sections := make([]Section, 0)
	begin := p.lexer.Next() // skip the BeginTopLevelGroup
	for p.lexer.HasNext() && !p.has(shared.EndTopLevelGroup) {
		if section, ok := p.section(); ok {
			sections = append(sections, section)
		} else {
			next := p.lexer.Next()
			p.appendDiagnostic("Expected a section", next.Position)
		}
	}

	if p.has(shared.EndTopLevelGroup) {
		p.lexer.Next() // move past then end of the group
	} else {
		p.appendDiagnostic("Unterminated group", begin.Position)
	}

	return Group{
		Id:       id,
		Sections: sections,
		MetaData: Phase4MetaData{
			Start: begin.Position,
		},
	}, true
}

func (p *phase4Parser) argumentGroup(id *string) (Group, bool) {
	if !p.has(shared.BeginArgumentGroup) {
		return Group{}, false
	}

	sections := make([]Section, 0)
	begin := p.lexer.Next() // skip the BeginArgumentGroup
	for p.lexer.HasNext() && !p.has(shared.EndArgumentGroup) {
		if section, ok := p.section(); ok {
			sections = append(sections, section)
		} else {
			next := p.lexer.Next()
			p.appendDiagnostic("Expected a section", next.Position)
		}
	}

	if p.has(shared.EndArgumentGroup) {
		p.lexer.Next() // move past then end of the group
	} else {
		p.appendDiagnostic("Unterminated group", begin.Position)
	}

	return Group{
		Id:       id,
		Sections: sections,
		MetaData: Phase4MetaData{
			Start: begin.Position,
		},
	}, true
}

func (p *phase4Parser) section() (Section, bool) {
	return Section{}, false
}

func (p *phase4Parser) argument() (Argument, bool) {
	if p.has(shared.BeginArgumentGroup) {
		if p.has(shared.Id) {
			id := p.lexer.Next()
			idText := id.Text
			if group, ok := p.argumentGroup(&idText); ok {
				return Argument{
					IsInline: false,
					Arg:      group,
					MetaData: Phase4MetaData{
						Start: id.Position,
					},
				}, true
			}

			p.appendDiagnostic("Expected a group", id.Position)
			return Argument{}, false
		}

		if group, ok := p.argumentGroup(nil); ok {
			return Argument{
				IsInline: false,
				Arg:      group,
				MetaData: Phase4MetaData{
					Start: group.MetaData.Start,
				},
			}, true
		}

		p.appendDiagnostic("Expected a group", ast.Position{})
		return Argument{}, false
	}

	if p.has(shared.BeginInlineArgument) {
		start := p.lexer.Position()
		if arg, ok := p.argumentData(); ok {
			return Argument{
				IsInline: true,
				Arg:      arg,
				MetaData: Phase4MetaData{
					Start: start,
				},
			}, true
		}

		p.appendDiagnostic("Expected an argument", start)
		return Argument{}, false
	}

	if p.has(shared.BeginDotSpaceArgument) {
		start := p.lexer.Position()
		if arg, ok := p.argumentData(); ok {
			return Argument{
				IsInline: false,
				Arg:      arg,
				MetaData: Phase4MetaData{
					Start: start,
				},
			}, true
		}

		p.appendDiagnostic("Expected an argument", start)
		return Argument{}, false
	}

	return Argument{}, false
}

func (p *phase4Parser) argumentData() (ArgumentDataType, bool) {
	if p.has(shared.ArgumentText) {
		arg := p.lexer.Next()
		return ArgumentTextArgumentData{
			Text: arg.Text,
			MetaData: Phase4MetaData{
				Start: arg.Position,
			},
		}, true
	}

	if p.has(shared.Formulation) {
		arg := p.lexer.Next()
		return FormulationArgumentData{
			Text: arg.Text,
			MetaData: Phase4MetaData{
				Start: arg.Position,
			},
		}, true
	}

	if p.has(shared.Text) {
		arg := p.lexer.Next()
		return TextArgumentData{
			Text: arg.Text,
			MetaData: Phase4MetaData{
				Start: arg.Position,
			},
		}, true
	}

	return ArgumentTextArgumentData{}, false
}

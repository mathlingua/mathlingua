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

package formulation

import (
	"fmt"
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/shared"
	"strings"
)

func Parse(text string) (ast.NodeType, []frontend.Diagnostic, bool) {
	lexer := NewLexer(text)
	parser := formulationParser{
		lexer:       lexer,
		diagnostics: make([]frontend.Diagnostic, 0),
	}
	node, _ := parser.structuralFormType()
	return node, parser.diagnostics, len(parser.diagnostics) == 0
}

type formulationParser struct {
	lexer       shared.Lexer
	diagnostics []frontend.Diagnostic
}

func (fp *formulationParser) token(tokenType shared.TokenType) (shared.Token, bool) {
	if !fp.has(tokenType) {
		return shared.Token{}, false
	}
	return fp.lexer.Next(), true
}

func (fp *formulationParser) has(tokenType shared.TokenType) bool {
	return fp.lexer.HasNext() && fp.lexer.Peek().Type == tokenType
}

func (fp *formulationParser) hasHas(tokenType1 shared.TokenType, tokenType2 shared.TokenType) bool {
	return fp.lexer.HasNextNext() && fp.lexer.Peek().Type == tokenType1 && fp.lexer.PeekPeek().Type == tokenType2
}

func (fp *formulationParser) next() shared.Token {
	return fp.lexer.Next()
}

func (fp *formulationParser) error(message string) {
	fp.diagnostics = append(fp.diagnostics, frontend.Diagnostic{
		Type:     frontend.Error,
		Origin:   frontend.FormulationParserOrigin,
		Message:  message,
		Position: fp.lexer.Position(),
	})
}

func (fp *formulationParser) expect(tokenType shared.TokenType) (shared.Token, bool) {
	if !fp.has(tokenType) {
		fp.error(fmt.Sprintf("Expected a token of type %s", tokenType))
		return shared.Token{}, false
	}
	return fp.next(), true
}

func (fp *formulationParser) structuralFormType() (ast.StructuralFormType, bool) {
	if fun, ok := fp.functionForm(); ok {
		return fun, true
	}

	if funExp, ok := fp.functionExpressionForm(); ok {
		return funExp, true
	}

	if name, ok := fp.nameForm(); ok {
		return name, true
	}

	if tuple, ok := fp.tupleForm(); ok {
		return tuple, true
	}

	if fixedSet, ok := fp.fixedSetForm(); ok {
		return fixedSet, true
	}

	if conditionalSet, ok := fp.conditionalSetForm(); ok {
		return conditionalSet, true
	}

	return nil, false
}

func (fp *formulationParser) nameForm() (ast.NameForm, bool) {
	if !fp.has(shared.Name) {
		return ast.NameForm{}, false
	}

	name := fp.next().Text
	isStropped := false
	if strings.HasPrefix(name, "\"") && strings.HasSuffix(name, "\"") {
		isStropped = true
		name = strings.TrimPrefix(strings.TrimSuffix(name, "\""), "\"")
	}

	isVarArg := false
	var varArgCount *string = nil
	hasQuestionMark := false
	if fp.has(shared.DotDotDot) {
		fp.next() // skip the ...
		isVarArg = true

		if fp.hasHas(shared.Operator, shared.Name) && fp.lexer.Peek().Text == "#" {
			fp.next() // skip the #
			text := fp.next().Text
			varArgCount = &text
		}
	}

	if fp.has(shared.QuestionMark) {
		fp.next() // skip the ?
		hasQuestionMark = true
	}

	return ast.NameForm{
		Text:            name,
		IsStropped:      isStropped,
		HasQuestionMark: hasQuestionMark,
		VarArg: ast.VarArgData{
			IsVarArg:    isVarArg,
			VarArgCount: varArgCount,
		},
	}, true
}

func (fp *formulationParser) varArgData() (ast.VarArgData, bool) {
	if !fp.has(shared.DotDotDot) {
		return ast.VarArgData{
			IsVarArg:    false,
			VarArgCount: nil,
		}, false
	}
	fp.expect(shared.DotDotDot)
	var varArgCount *string = nil
	if fp.hasHas(shared.Operator, shared.Name) && fp.lexer.Peek().Text == "#" {
		text := fp.next().Text
		varArgCount = &text
	}
	return ast.VarArgData{
		IsVarArg:    true,
		VarArgCount: varArgCount,
	}, true
}

func (fp *formulationParser) functionForm() (ast.FunctionForm, bool) {
	if !fp.hasHas(shared.Name, shared.LParen) {
		return ast.FunctionForm{}, false
	}

	target, ok := fp.nameForm()
	if !ok {
		return ast.FunctionForm{}, false
	}

	params := make([]ast.NameForm, 0)
	fp.expect(shared.LParen)
	for fp.lexer.HasNext() {
		if fp.has(shared.RParen) {
			break
		}

		if len(params) > 0 {
			fp.expect(shared.Comma)
		}

		param, ok := fp.nameForm()
		if !ok {
			fp.error("Expected a name")
			// move past the unexpected token
			fp.lexer.Next()
		} else {
			params = append(params, param)
		}
	}
	fp.expect(shared.RParen)

	varArgData, _ := fp.varArgData()
	return ast.FunctionForm{
		Target: target,
		Params: params,
		VarArg: varArgData,
	}, true
}

func (fp *formulationParser) functionExpressionForm() (ast.FunctionExpressionForm, bool) {
	if !fp.hasHas(shared.Name, shared.LSquare) {
		return ast.FunctionExpressionForm{}, false
	}

	target, ok := fp.nameForm()
	if !ok {
		return ast.FunctionExpressionForm{}, false
	}

	params := make([]ast.NameForm, 0)
	fp.expect(shared.LSquare)
	for fp.lexer.HasNext() {
		if fp.has(shared.RSquare) {
			break
		}

		if len(params) > 0 {
			fp.expect(shared.Comma)
		}

		param, ok := fp.nameForm()
		if !ok {
			fp.error("Expected a name")
			// move past the unexpected token
			fp.lexer.Next()
		} else {
			params = append(params, param)
		}
	}
	fp.expect(shared.RSquare)

	varArgData, _ := fp.varArgData()
	return ast.FunctionExpressionForm{
		Target: target,
		Params: params,
		VarArg: varArgData,
	}, true
}

func (fp *formulationParser) tupleForm() (ast.TupleForm, bool) {
	id := fp.lexer.Snapshot()
	_, ok := fp.token(shared.LParen)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.TupleForm{}, false
	}
	params := make([]ast.StructuralFormType, 0)
	for fp.lexer.HasNext() {
		if fp.has(shared.RParen) {
			break
		}

		if len(params) > 0 {
			fp.expect(shared.Comma)
		}

		param, ok := fp.structuralFormType()
		if !ok {
			fp.lexer.RollBack(id)
			return ast.TupleForm{}, false
		}
		params = append(params, param)
	}
	fp.expect(shared.RParen)
	varArg, _ := fp.varArgData()
	fp.lexer.Commit(id)
	return ast.TupleForm{
		Params: params,
		VarArg: varArg,
	}, true
}

func (fp *formulationParser) fixedSetForm() (ast.FixedSetForm, bool) {
	id := fp.lexer.Snapshot()
	_, ok := fp.token(shared.LCurly)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.FixedSetForm{}, false
	}
	params := make([]ast.StructuralFormType, 0)
	for fp.lexer.HasNext() {
		if fp.has(shared.RParen) {
			break
		}

		if len(params) > 0 {
			fp.expect(shared.Comma)
		}

		param, ok := fp.structuralFormType()
		if !ok {
			fp.lexer.RollBack(id)
			return ast.FixedSetForm{}, false
		}
		params = append(params, param)
	}
	fp.expect(shared.RCurly)
	varArg, _ := fp.varArgData()
	fp.lexer.Commit(id)
	return ast.FixedSetForm{
		Params: params,
		VarArg: varArg,
	}, true
}

func (fp *formulationParser) conditionalSetForm() (ast.ConditionalSetForm, bool) {
	id := fp.lexer.Snapshot()
	if _, ok := fp.token(shared.LCurly); !ok {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetForm{}, false
	}

	target, ok := fp.structuralFormType()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetForm{}, false
	}

	_, ok = fp.token(shared.Bar)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetForm{}, false
	}

	_, ok = fp.token(shared.DotDotDot)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetForm{}, false
	}
	fp.expect(shared.RCurly)
	varArg, _ := fp.varArgData()
	fp.lexer.Commit(id)
	return ast.ConditionalSetForm{
		Target: target,
		VarArg: varArg,
	}, true
}

func (fp *formulationParser) expressionType() (ast.ExpressionType, bool) {
	// TODO: implement this
	return nil, false
}

func (fp *formulationParser) functionCallExpression() (ast.FunctionCallExpression, bool) {
	if !fp.hasHas(shared.Name, shared.LParen) {
		return ast.FunctionCallExpression{}, false
	}

	target, ok := fp.expressionType()
	if !ok {
		return ast.FunctionCallExpression{}, false
	}

	args := make([]ast.ExpressionType, 0)
	fp.expect(shared.LParen)
	for fp.lexer.HasNext() {
		if fp.has(shared.RParen) {
			break
		}

		if len(args) > 0 {
			fp.expect(shared.Comma)
		}

		arg, ok := fp.expressionType()
		if !ok {
			fp.error("Expected an expression")
			// move past the unexpected token
			fp.lexer.Next()
		} else {
			args = append(args, arg)
		}
	}
	fp.expect(shared.RParen)
	return ast.FunctionCallExpression{
		Target: target,
		Args:   args,
	}, true
}

func (fp *formulationParser) tupleExpression() (ast.TupleExpression, bool) {
	id := fp.lexer.Snapshot()
	_, ok := fp.token(shared.LParen)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.TupleExpression{}, false
	}
	args := make([]ast.ExpressionType, 0)
	for fp.lexer.HasNext() {
		if fp.has(shared.RParen) {
			break
		}

		if len(args) > 0 {
			fp.expect(shared.Comma)
		}

		arg, ok := fp.expressionType()
		if !ok {
			fp.lexer.RollBack(id)
			return ast.TupleExpression{}, false
		}
		args = append(args, arg)
	}
	fp.expect(shared.RParen)
	fp.lexer.Commit(id)
	return ast.TupleExpression{
		Args: args,
	}, true
}

func (fp *formulationParser) fixedSetExpression() (ast.FixedSetExpression, bool) {
	id := fp.lexer.Snapshot()
	_, ok := fp.token(shared.LCurly)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.FixedSetExpression{}, false
	}
	args := make([]ast.ExpressionType, 0)
	for fp.lexer.HasNext() {
		if fp.has(shared.RParen) {
			break
		}

		if len(args) > 0 {
			fp.expect(shared.Comma)
		}

		arg, ok := fp.structuralFormType()
		if !ok {
			fp.lexer.RollBack(id)
			return ast.FixedSetExpression{}, false
		}
		args = append(args, arg)
	}
	fp.expect(shared.RCurly)
	fp.lexer.Commit(id)
	return ast.FixedSetExpression{
		Args: args,
	}, true
}

/*
func (fp *formulationParser) chainExpression() (ast.ChainExpression, bool) {
	id := fp.lexer.Snapshot()
	parts := make([]ast.ExpressionType, 0)
	for fp.lexer.HasNext() {
		part, ok := fp.expressionType()
	}
}
*/

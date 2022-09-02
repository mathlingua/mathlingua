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

func (fp *formulationParser) token(tokenType ast.TokenType) (ast.Token, bool) {
	if !fp.has(tokenType) {
		return ast.Token{}, false
	}
	return fp.lexer.Next(), true
}

func (fp *formulationParser) has(tokenType ast.TokenType) bool {
	return fp.lexer.HasNext() && fp.lexer.Peek().Type == tokenType
}

func (fp *formulationParser) hasHas(tokenType1 ast.TokenType, tokenType2 ast.TokenType) bool {
	return fp.lexer.HasNextNext() && fp.lexer.Peek().Type == tokenType1 && fp.lexer.PeekPeek().Type == tokenType2
}

func (fp *formulationParser) next() ast.Token {
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

func (fp *formulationParser) expect(tokenType ast.TokenType) (ast.Token, bool) {
	if !fp.has(tokenType) {
		fp.error(fmt.Sprintf("Expected a token of type %s", tokenType))
		return ast.Token{}, false
	}
	return fp.next(), true
}

func (fp *formulationParser) parenArgs() ([]ast.ExpressionType, bool) {
	id := fp.lexer.Snapshot()
	_, ok := fp.token(ast.LParen)
	if !ok {
		fp.lexer.RollBack(id)
		return []ast.ExpressionType{}, false
	}
	args := make([]ast.ExpressionType, 0)
	for fp.lexer.HasNext() {
		if fp.has(ast.RParen) {
			break
		}

		if len(args) > 0 {
			fp.expect(ast.Comma)
		}

		arg, ok := fp.expressionType()
		if !ok {
			fp.lexer.RollBack(id)
			return []ast.ExpressionType{}, false
		}
		args = append(args, arg)
	}
	fp.expect(ast.RParen)
	fp.lexer.Commit(id)
	return args, true
}

func (fp *formulationParser) curlyArgs() ([]ast.ExpressionType, bool) {
	id := fp.lexer.Snapshot()
	_, ok := fp.token(ast.LCurly)
	if !ok {
		fp.lexer.RollBack(id)
		return []ast.ExpressionType{}, false
	}
	args := make([]ast.ExpressionType, 0)
	for fp.lexer.HasNext() {
		if fp.has(ast.RCurly) {
			break
		}

		if len(args) > 0 {
			fp.expect(ast.Comma)
		}

		arg, ok := fp.expressionType()
		if !ok {
			fp.lexer.RollBack(id)
			return []ast.ExpressionType{}, false
		}
		args = append(args, arg)
	}
	fp.expect(ast.RCurly)
	fp.lexer.Commit(id)
	return args, true
}

func (fp *formulationParser) parenParams() ([]ast.StructuralFormType, bool) {
	id := fp.lexer.Snapshot()
	_, ok := fp.token(ast.LParen)
	if !ok {
		fp.lexer.RollBack(id)
		return []ast.StructuralFormType{}, false
	}
	args := make([]ast.StructuralFormType, 0)
	for fp.lexer.HasNext() {
		if fp.has(ast.RParen) {
			break
		}

		if len(args) > 0 {
			fp.expect(ast.Comma)
		}

		arg, ok := fp.structuralFormType()
		if !ok {
			fp.lexer.RollBack(id)
			return []ast.StructuralFormType{}, false
		}
		args = append(args, arg)
	}
	fp.expect(ast.RParen)
	fp.lexer.Commit(id)
	return args, true
}

func (fp *formulationParser) squareParams() ([]ast.StructuralFormType, bool) {
	id := fp.lexer.Snapshot()
	_, ok := fp.token(ast.LSquare)
	if !ok {
		fp.lexer.RollBack(id)
		return []ast.StructuralFormType{}, false
	}
	args := make([]ast.StructuralFormType, 0)
	for fp.lexer.HasNext() {
		if fp.has(ast.RSquare) {
			break
		}

		if len(args) > 0 {
			fp.expect(ast.Comma)
		}

		arg, ok := fp.structuralFormType()
		if !ok {
			fp.lexer.RollBack(id)
			return []ast.StructuralFormType{}, false
		}
		args = append(args, arg)
	}
	fp.expect(ast.RSquare)
	fp.lexer.Commit(id)
	return args, true
}

func (fp *formulationParser) curlyParams() ([]ast.StructuralFormType, bool) {
	id := fp.lexer.Snapshot()
	_, ok := fp.token(ast.LCurly)
	if !ok {
		fp.lexer.RollBack(id)
		return []ast.StructuralFormType{}, false
	}
	args := make([]ast.StructuralFormType, 0)
	for fp.lexer.HasNext() {
		if fp.has(ast.RCurly) {
			break
		}

		if len(args) > 0 {
			fp.expect(ast.Comma)
		}

		arg, ok := fp.structuralFormType()
		if !ok {
			fp.lexer.RollBack(id)
			return []ast.StructuralFormType{}, false
		}
		args = append(args, arg)
	}
	fp.expect(ast.RCurly)
	fp.lexer.Commit(id)
	return args, true
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
	if !fp.has(ast.Name) {
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
	if fp.has(ast.DotDotDot) {
		fp.next() // skip the ...
		isVarArg = true

		if fp.hasHas(ast.Operator, ast.Name) && fp.lexer.Peek().Text == "#" {
			fp.next() // skip the #
			text := fp.next().Text
			varArgCount = &text
		}
	}

	if fp.has(ast.QuestionMark) {
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
	if !fp.has(ast.DotDotDot) {
		return ast.VarArgData{
			IsVarArg:    false,
			VarArgCount: nil,
		}, false
	}
	fp.expect(ast.DotDotDot)
	var varArgCount *string = nil
	if fp.hasHas(ast.Operator, ast.Name) && fp.lexer.Peek().Text == "#" {
		text := fp.next().Text
		varArgCount = &text
	}
	return ast.VarArgData{
		IsVarArg:    true,
		VarArgCount: varArgCount,
	}, true
}

func (fp *formulationParser) functionForm() (ast.FunctionForm, bool) {
	if !fp.hasHas(ast.Name, ast.LParen) {
		return ast.FunctionForm{}, false
	}

	target, ok := fp.nameForm()
	if !ok {
		return ast.FunctionForm{}, false
	}

	params := make([]ast.NameForm, 0)
	fp.expect(ast.LParen)
	for fp.lexer.HasNext() {
		if fp.has(ast.RParen) {
			break
		}

		if len(params) > 0 {
			fp.expect(ast.Comma)
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
	fp.expect(ast.RParen)

	varArgData, _ := fp.varArgData()
	return ast.FunctionForm{
		Target: target,
		Params: params,
		VarArg: varArgData,
	}, true
}

func (fp *formulationParser) functionExpressionForm() (ast.FunctionExpressionForm, bool) {
	if !fp.hasHas(ast.Name, ast.LSquare) {
		return ast.FunctionExpressionForm{}, false
	}

	target, ok := fp.nameForm()
	if !ok {
		return ast.FunctionExpressionForm{}, false
	}

	params := make([]ast.NameForm, 0)
	fp.expect(ast.LSquare)
	for fp.lexer.HasNext() {
		if fp.has(ast.RSquare) {
			break
		}

		if len(params) > 0 {
			fp.expect(ast.Comma)
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
	fp.expect(ast.RSquare)

	varArgData, _ := fp.varArgData()
	return ast.FunctionExpressionForm{
		Target: target,
		Params: params,
		VarArg: varArgData,
	}, true
}

func (fp *formulationParser) tupleForm() (ast.TupleForm, bool) {
	id := fp.lexer.Snapshot()
	_, ok := fp.token(ast.LParen)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.TupleForm{}, false
	}
	params := make([]ast.StructuralFormType, 0)
	for fp.lexer.HasNext() {
		if fp.has(ast.RParen) {
			break
		}

		if len(params) > 0 {
			fp.expect(ast.Comma)
		}

		param, ok := fp.structuralFormType()
		if !ok {
			fp.lexer.RollBack(id)
			return ast.TupleForm{}, false
		}
		params = append(params, param)
	}
	fp.expect(ast.RParen)
	varArg, _ := fp.varArgData()
	fp.lexer.Commit(id)
	return ast.TupleForm{
		Params: params,
		VarArg: varArg,
	}, true
}

func (fp *formulationParser) fixedSetForm() (ast.FixedSetForm, bool) {
	id := fp.lexer.Snapshot()
	_, ok := fp.token(ast.LCurly)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.FixedSetForm{}, false
	}
	params := make([]ast.StructuralFormType, 0)
	for fp.lexer.HasNext() {
		if fp.has(ast.RParen) {
			break
		}

		if len(params) > 0 {
			// if the input is of the form `{x |` then it is a
			// conditional set and not a fixed set
			if fp.has(ast.Bar) {
				fp.lexer.RollBack(id)
				return ast.FixedSetForm{}, false
			}
			fp.expect(ast.Comma)
		}

		param, ok := fp.structuralFormType()
		if !ok {
			fp.lexer.RollBack(id)
			return ast.FixedSetForm{}, false
		}
		params = append(params, param)
	}
	fp.expect(ast.RCurly)
	varArg, _ := fp.varArgData()
	fp.lexer.Commit(id)
	return ast.FixedSetForm{
		Params: params,
		VarArg: varArg,
	}, true
}

func (fp *formulationParser) conditionalSetForm() (ast.ConditionalSetForm, bool) {
	id := fp.lexer.Snapshot()
	if _, ok := fp.token(ast.LCurly); !ok {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetForm{}, false
	}

	target, ok := fp.structuralFormType()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetForm{}, false
	}

	_, ok = fp.token(ast.Bar)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetForm{}, false
	}

	_, ok = fp.token(ast.DotDotDot)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetForm{}, false
	}
	fp.expect(ast.RCurly)
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
	if !fp.hasHas(ast.Name, ast.LParen) {
		return ast.FunctionCallExpression{}, false
	}

	target, ok := fp.expressionType()
	if !ok {
		return ast.FunctionCallExpression{}, false
	}

	args := make([]ast.ExpressionType, 0)
	fp.expect(ast.LParen)
	for fp.lexer.HasNext() {
		if fp.has(ast.RParen) {
			break
		}

		if len(args) > 0 {
			fp.expect(ast.Comma)
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
	fp.expect(ast.RParen)
	return ast.FunctionCallExpression{
		Target: target,
		Args:   args,
	}, true
}

func (fp *formulationParser) tupleExpression() (ast.TupleExpression, bool) {
	id := fp.lexer.Snapshot()
	_, ok := fp.token(ast.LParen)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.TupleExpression{}, false
	}
	args := make([]ast.ExpressionType, 0)
	for fp.lexer.HasNext() {
		if fp.has(ast.RParen) {
			break
		}

		if len(args) > 0 {
			fp.expect(ast.Comma)
		}

		arg, ok := fp.expressionType()
		if !ok {
			fp.lexer.RollBack(id)
			return ast.TupleExpression{}, false
		}
		args = append(args, arg)
	}
	fp.expect(ast.RParen)
	fp.lexer.Commit(id)
	return ast.TupleExpression{
		Args: args,
	}, true
}

func (fp *formulationParser) fixedSetExpression() (ast.FixedSetExpression, bool) {
	id := fp.lexer.Snapshot()
	_, ok := fp.token(ast.LCurly)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.FixedSetExpression{}, false
	}
	args := make([]ast.ExpressionType, 0)
	for fp.lexer.HasNext() {
		if fp.has(ast.RParen) {
			break
		}

		if len(args) > 0 {
			// if the input is of the form `{x |` then it is a
			// conditional set and not a fixed set
			if fp.has(ast.Bar) {
				fp.lexer.RollBack(id)
				return ast.FixedSetExpression{}, false
			}
			fp.expect(ast.Comma)
		}

		arg, ok := fp.structuralFormType()
		if !ok {
			fp.lexer.RollBack(id)
			return ast.FixedSetExpression{}, false
		}
		args = append(args, arg)
	}
	fp.expect(ast.RCurly)
	fp.lexer.Commit(id)
	return ast.FixedSetExpression{
		Args: args,
	}, true
}

func (fp *formulationParser) chainExpression() (ast.ChainExpression, bool) {
	id := fp.lexer.Snapshot()
	parts := make([]ast.ExpressionType, 0)
	for fp.lexer.HasNext() {
		part, ok := fp.expressionType()
		if !ok {
			fp.lexer.RollBack(id)
			return ast.ChainExpression{}, false
		}
		parts = append(parts, part)
		if fp.has(ast.Dot) {
			fp.expect(ast.Dot)
		} else {
			break
		}
	}
	if len(parts) < 2 {
		fp.lexer.RollBack(id)
		return ast.ChainExpression{}, false
	}
	return ast.ChainExpression{
		Parts: parts,
	}, true
}

func (fp *formulationParser) nameOrdinalCallExpression() (ast.NameOrdinalCallExpression, bool) {
	id := fp.lexer.Snapshot()
	name, ok := fp.nameForm()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.NameOrdinalCallExpression{}, false
	}

	_, ok = fp.token(ast.LCurly)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.NameOrdinalCallExpression{}, false
	}

	exp, ok := fp.expressionType()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.NameOrdinalCallExpression{}, false
	}

	_, ok = fp.token(ast.RCurly)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.NameOrdinalCallExpression{}, false
	}

	fp.lexer.Commit(id)
	return ast.NameOrdinalCallExpression{
		Target: name,
		Arg:    exp,
	}, true
}

func (fp *formulationParser) pseudoToken(expectedType ast.TokenType) (ast.PseudoTokenNode, bool) {
	tok, ok := fp.token(ast.As)
	if !ok {
		return ast.PseudoTokenNode{}, false
	}
	return ast.PseudoTokenNode{
		Type: tok.Type,
	}, true
}

func (fp *formulationParser) asKeyword() (ast.PseudoTokenNode, bool) {
	return fp.pseudoToken(ast.As)
}

func (fp *formulationParser) isKeyword() (ast.PseudoTokenNode, bool) {
	return fp.pseudoToken(ast.Is)
}

func (fp *formulationParser) isNotKeyword() (ast.PseudoTokenNode, bool) {
	return fp.pseudoToken(ast.IsNot)
}

func (fp *formulationParser) isQuestionMarkKeyword() (ast.PseudoTokenNode, bool) {
	return fp.pseudoToken(ast.QuestionMark)
}

func (fp *formulationParser) operatorToken() (ast.PseudoTokenNode, bool) {
	return fp.pseudoToken(ast.Operator)
}

func (fp *formulationParser) colonEqualsToken() (ast.PseudoTokenNode, bool) {
	return fp.pseudoToken(ast.ColonEquals)
}

func (fp *formulationParser) nonEnclosedNonCommandOperatorTarget() (ast.NonEnclosedNonCommandOperatorTarget, bool) {
	if tok, ok := fp.token(ast.Operator); ok {
		return ast.NonEnclosedNonCommandOperatorTarget{
			Text: tok.Text,
		}, true
	}
	return ast.NonEnclosedNonCommandOperatorTarget{}, false
}

func (fp *formulationParser) enclosedNonCommandOperatorTarget() (ast.EnclosedNonCommandOperatorTarget, bool) {
	id := fp.lexer.Snapshot()
	if _, ok := fp.token(ast.LSquare); !ok {
		fp.lexer.RollBack(id)
		return ast.EnclosedNonCommandOperatorTarget{}, false
	}

	exp, ok := fp.expressionType()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.EnclosedNonCommandOperatorTarget{}, false
	}

	if _, ok = fp.token(ast.RSquare); !ok {
		fp.lexer.RollBack(id)
		return ast.EnclosedNonCommandOperatorTarget{}, false
	}

	fp.lexer.Commit(id)
	return ast.EnclosedNonCommandOperatorTarget{
		Target: exp,
	}, true
}

func (fp *formulationParser) namedArg() (ast.NamedArg, bool) {
	if !fp.hasHas(ast.Colon, ast.Name) {
		return ast.NamedArg{}, false
	}
	id := fp.lexer.Snapshot()
	fp.expect(ast.Colon)
	name, ok := fp.nameForm()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.NamedArg{}, false
	}
	var args *[]ast.ExpressionType = nil
	if curlyArgs, ok := fp.curlyArgs(); ok {
		args = &curlyArgs
	}
	return ast.NamedArg{
		Name: name,
		Args: args,
	}, true
}

/*
func (fp *formulationParser) subSupArgs() (ast.SubSupArgs, bool) {
	id := fp.lexer.Snapshot()
	squareParams, ok := fp.squareParams()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.SubSupArgs{}, false
	}

	subArgs := make([]ast.ExpressionType, 0)
	if fp.has(ast.Underscore) {
	}
}
*/

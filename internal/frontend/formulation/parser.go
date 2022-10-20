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
	"math"
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/shared"
	"strings"
)

func ParseExpression(text string, start ast.Position, tracker frontend.DiagnosticTracker) (ast.NodeType, bool) {
	numDiagBefore := tracker.Length()
	lexer := NewLexer(text, tracker)
	parser := formulationParser{
		lexer:   lexer,
		tracker: tracker,
		start:   start,
	}
	node, _ := parser.multiplexedExpressionType()
	parser.finalize()
	return node, tracker.Length() == numDiagBefore
}

func ParseForm(text string, start ast.Position, tracker frontend.DiagnosticTracker) (ast.NodeType, bool) {
	numDiagBefore := tracker.Length()
	lexer := NewLexer(text, tracker)
	parser := formulationParser{
		lexer:   lexer,
		tracker: tracker,
		start:   start,
	}
	node, _ := parser.form()
	parser.finalize()
	return node, tracker.Length() == numDiagBefore
}

func ParseId(text string, start ast.Position, tracker frontend.DiagnosticTracker) (ast.IdType, bool) {
	numDiagBefore := tracker.Length()
	lexer := NewLexer(text, tracker)
	parser := formulationParser{
		lexer:   lexer,
		tracker: tracker,
		start:   start,
	}
	node, _ := parser.idType()
	parser.finalize()
	return node, tracker.Length() == numDiagBefore
}

func ParseSignature(text string, start ast.Position, tracker frontend.DiagnosticTracker) (ast.Signature, bool) {
	numDiagBefore := tracker.Length()
	lexer := NewLexer(text, tracker)
	parser := formulationParser{
		lexer:   lexer,
		tracker: tracker,
		start:   start,
	}
	node, _ := parser.signature()
	parser.finalize()
	return node, tracker.Length() == numDiagBefore
}

////////////////////// utility functions ////////////////////////////////////

type formulationParser struct {
	lexer   shared.Lexer
	tracker frontend.DiagnosticTracker
	start   ast.Position
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

func (fp *formulationParser) errorAt(message string, position ast.Position) {
	fp.tracker.Append(frontend.Diagnostic{
		Type:    frontend.Error,
		Origin:  frontend.FormulationParserOrigin,
		Message: message,
		Position: ast.Position{
			Row:    fp.start.Row + position.Row,
			Column: fp.start.Column + position.Column,
			Offset: fp.start.Offset + position.Offset,
		},
	})
}

func (fp *formulationParser) error(message string) {
	position := fp.lexer.Position()
	fp.errorAt(message, position)
}

func (fp *formulationParser) finalize() {
	if fp.lexer.HasNext() {
		next := fp.lexer.Next()
		fp.error(fmt.Sprintf("Token '%s' and all of the following are unexpected", next.Text))
	}
}

func (fp *formulationParser) expect(tokenType ast.TokenType) (ast.Token, bool) {
	if !fp.has(tokenType) {
		fp.error(fmt.Sprintf("Expected a token of type %s", tokenType))
		return ast.Token{}, false
	}
	return fp.next(), true
}

////////////////////// expressions //////////////////////////////////////////

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

func (fp *formulationParser) varArgData() (ast.VarArgData, bool) {
	start := fp.lexer.Position()
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
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) literalExpressionType() (ast.LiteralExpressionType, bool) {
	if set, ok := fp.conditionalSetExpression(); ok {
		return set, ok
	}

	if fun, ok := fp.functionCallExpression(); ok {
		return fun, ok
	}

	if tup, ok := fp.tupleExpression(); ok {
		return tup, ok
	}

	if set, ok := fp.fixedSetExpression(); ok {
		return set, ok
	}

	return nil, false
}

func (fp *formulationParser) multiplexedExpressionType() (ast.ExpressionType, bool) {
	start := fp.lexer.Position()
	items := make([]ast.ExpressionType, 0)
	for fp.lexer.HasNext() {
		if len(items) > 0 {
			if fp.has(ast.Comma) {
				fp.expect(ast.Comma)
			} else {
				break
			}
		}
		arg, ok := fp.expressionType()
		if !ok {
			break
		}
		items = append(items, arg)
	}

	if len(items) == 0 {
		fp.error("Expected an expression")
		return nil, false
	}

	if len(items) == 1 {
		return items[0], true
	}

	isIndex := -1
	extendsIndex := -1

	minPrecIndexMinIndex := math.MaxInt
	minPrecIndexMaxIndex := -1
	minPrec := math.MinInt

	for i, item := range items {
		_, isOk := item.(ast.IsExpression)
		if isOk {
			if isIndex >= 0 {
				fp.error("'is' statements cannot be nested")
			}
			isIndex = i
		}

		_, extendsOk := item.(ast.ExtendsExpression)
		if extendsOk {
			if extendsIndex >= 0 {
				fp.error("'extends' statements cannot be nested")
			}
			extendsIndex = i
		}

		prec, isInfix := GetPrecedenceAndIfInfix(item)
		if isInfix {
			if prec >= minPrec {
				minPrec = prec
				minPrecIndexMinIndex = min(minPrecIndexMinIndex, i)
				minPrecIndexMaxIndex = max(minPrecIndexMaxIndex, i)
			}
		}
	}

	// if there isn't an 'is' or 'extends' statement and there
	// is at least one infix operator
	if isIndex == -1 && extendsIndex == -1 && minPrecIndexMaxIndex >= 0 {
		if minPrecIndexMinIndex != minPrecIndexMaxIndex {
			fp.error("A multiplexed operator can only be used if exactly one operator has minimum precedence")
			return nil, false
		} else {
			lhs := make([]ast.ExpressionType, 0)
			rhs := make([]ast.ExpressionType, 0)
			i := 0
			for i < minPrecIndexMinIndex {
				lhs = append(lhs, items[i])
				i++
			}
			opExp := items[minPrecIndexMinIndex].(ast.InfixOperatorCallExpression)
			lhs = append(lhs, opExp.Lhs)
			rhs = append(rhs, opExp.Rhs)
			i = minPrecIndexMinIndex + 1
			for i < len(items) {
				rhs = append(rhs, items[i])
				i++
			}
			return ast.MultiplexedInfixOperatorCallExpression{
				Target: opExp.Target,
				Lhs:    lhs,
				Rhs:    rhs,
				MetaData: ast.MetaData{
					Start: start,
				},
			}, true
		}
	}

	if isIndex >= 0 && extendsIndex >= 0 {
		fp.error("'is' and 'extends' statements cannot be used together")
		return nil, false
	} else if isIndex == -1 && extendsIndex == -1 {
		fp.error("multiple comma separated expressions is not supported in this context")
		return nil, false
	} else if isIndex >= 0 {
		lhs := make([]ast.ExpressionType, 0)
		rhs := make([]ast.KindType, 0)
		i := 0
		for i < isIndex {
			lhs = append(lhs, items[i])
			i++
		}
		isExp := items[isIndex].(ast.IsExpression)
		lhs = append(lhs, isExp.Lhs...)
		rhs = append(rhs, isExp.Rhs...)
		i = isIndex + 1
		for i < len(items) {
			rhs = append(rhs, items[i].(ast.KindType))
			i++
		}
		return ast.IsExpression{
			Lhs: lhs,
			Rhs: rhs,
			MetaData: ast.MetaData{
				Start: start,
			},
		}, true
	} else if extendsIndex >= 0 {
		lhs := make([]ast.ExpressionType, 0)
		rhs := make([]ast.KindType, 0)
		i := 0
		for i < extendsIndex {
			lhs = append(lhs, items[i])
			i++
		}
		isExp := items[extendsIndex].(ast.ExtendsExpression)
		lhs = append(lhs, isExp.Lhs...)
		rhs = append(rhs, isExp.Rhs...)
		i = extendsIndex + 1
		for i < len(items) {
			rhs = append(rhs, items[i].(ast.KindType))
			i++
		}
		return ast.ExtendsExpression{
			Lhs: lhs,
			Rhs: rhs,
			MetaData: ast.MetaData{
				Start: start,
			},
		}, true
	} else {
		panic("Unreachable code")
	}
}

func min(x, y int) int {
	if x < y {
		return x
	} else {
		return y
	}
}

func max(x, y int) int {
	if x > y {
		return x
	} else {
		return y
	}
}

func (fp *formulationParser) expressionType(additionalTerminators ...ast.TokenType) (ast.ExpressionType, bool) {
	if exp, ok := fp.pseudoExpression(additionalTerminators...); ok {
		res, consolidateOk := Consolidate(exp.Children, fp.tracker)
		if resAsExp, resAsExpOk := res.(ast.ExpressionType); resAsExpOk {
			return resAsExp, consolidateOk
		} else {
			fp.error("Expected an Expression")
			return nil, false
		}
	}
	return nil, false
}

func (fp *formulationParser) isExpressionTerminator(tokenType ast.TokenType) bool {
	return tokenType == ast.Comma ||
		tokenType == ast.RParen ||
		tokenType == ast.RSquare ||
		tokenType == ast.RCurly ||
		tokenType == ast.Semicolon
}

func (fp *formulationParser) pseudoExpression(additionalTerminators ...ast.TokenType) (ast.PseudoExpression, bool) {
	isTerminator := func(tokType ast.TokenType) bool {
		if fp.isExpressionTerminator(tokType) {
			return true
		}
		for _, t := range additionalTerminators {
			if tokType == t {
				return true
			}
		}
		return false
	}

	start := fp.lexer.Position()
	children := make([]ast.NodeType, 0)
	prevOffset := -1
	for fp.lexer.HasNext() {
		if fp.lexer.HasNext() && fp.lexer.Peek().Position.Offset == prevOffset {
			next := fp.lexer.Next()
			fp.error(fmt.Sprintf("Unexpected text '%s'", next.Text))
			prevOffset = next.Position.Offset
			continue
		}

		if fp.lexer.HasNext() {
			prevOffset = fp.lexer.Peek().Position.Offset
		}

		if isTerminator(fp.lexer.Peek().Type) {
			break
		}

		if lit, ok := fp.literalExpressionType(); ok {
			children = append(children, lit)
		} else if op, ok := fp.operatorType(); ok {
			children = append(children, op)
		} else if pseudoToken, ok := fp.pseudoTokenNode(); ok {
			children = append(children, pseudoToken)
		} else if chain, ok := fp.chainExpression(); ok {
			children = append(children, chain)
		} else if ord, ok := fp.nameOrdinalCallExpression(); ok {
			children = append(children, ord)
		} else if fun, ok := fp.functionForm(); ok {
			children = append(children, fun)
		} else if name, ok := fp.nameForm(); ok {
			children = append(children, name)
		} else if fun, ok := fp.functionExpressionForm(); ok {
			children = append(children, fun)
		} else if fun, ok := fp.functionCallExpression(); ok {
			children = append(children, fun)
		} else if tup, ok := fp.tupleForm(); ok {
			children = append(children, tup)
		} else if tup, ok := fp.tupleExpression(); ok {
			children = append(children, tup)
		} else if set, ok := fp.fixedSetForm(); ok {
			children = append(children, set)
		} else if set, ok := fp.fixedSetExpression(); ok {
			children = append(children, set)
		} else if set, ok := fp.conditionalSetForm(); ok {
			children = append(children, set)
		} else if cmd, ok := fp.commandOperatorTarget(); ok {
			children = append(children, cmd)
		} else if cmd, ok := fp.commandAtExpression(); ok {
			children = append(children, cmd)
		} else if cmd, ok := fp.commandExpression(false); ok {
			children = append(children, cmd)
		} else {
			if fp.lexer.HasNext() {
				next := fp.lexer.Next()
				fp.error(fmt.Sprintf("Unexpected text '%s'", next.Text))
			}
			break
		}
	}
	if len(children) == 0 {
		return ast.PseudoExpression{}, false
	}
	return ast.PseudoExpression{
		Children: children,
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) functionCallExpressionTarget() (ast.ExpressionType, bool) {
	if name, ok := fp.nameForm(); ok {
		return name, ok
	}

	if tup, ok := fp.tupleExpression(); ok {
		return tup, ok
	}

	return nil, false
}

func (fp *formulationParser) functionCallExpression() (ast.FunctionCallExpression, bool) {
	start := fp.lexer.Position()
	if !fp.hasHas(ast.Name, ast.LParen) {
		return ast.FunctionCallExpression{}, false
	}

	target, ok := fp.functionCallExpressionTarget()
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
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) tupleExpression() (ast.TupleExpression, bool) {
	start := fp.lexer.Position()
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
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) fixedSetExpression() (ast.FixedSetExpression, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	_, ok := fp.token(ast.LCurly)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.FixedSetExpression{}, false
	}
	args := make([]ast.ExpressionType, 0)
	for fp.lexer.HasNext() {
		if fp.has(ast.RCurly) {
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

		arg, ok := fp.expressionType()
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
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) conditionalSetExpression() (ast.ConditionalSetExpression, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	symbols, ok := fp.squareParams()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetExpression{}, false
	}

	if !fp.has(ast.LCurly) {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetExpression{}, false
	}

	fp.expect(ast.LCurly)
	target, ok := fp.expressionType(ast.Bar)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetExpression{}, false
	}

	if !fp.has(ast.Bar) {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetExpression{}, false
	}

	fp.expect(ast.Bar)
	conditions := make([]ast.ExpressionType, 0)
	for fp.lexer.HasNext() {
		condition, ok := fp.expressionType()
		if ok {
			conditions = append(conditions, condition)
		} else {
			break
		}

		if fp.has(ast.Semicolon) {
			fp.expect(ast.Semicolon)
		} else {
			break
		}
	}

	if !fp.has(ast.RCurly) {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetExpression{}, false
	}

	fp.expect(ast.RCurly)
	fp.lexer.Commit(id)
	return ast.ConditionalSetExpression{
		Symbols:    symbols,
		Target:     target,
		Conditions: conditions,
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) chainExpressionPart() (ast.ExpressionType, bool) {
	if fun, ok := fp.functionCallExpression(); ok {
		return fun, ok
	}

	if name, ok := fp.nameForm(); ok {
		return name, ok
	}

	if tuple, ok := fp.tupleExpression(); ok {
		return tuple, ok
	}

	if set, ok := fp.fixedSetExpression(); ok {
		return set, ok
	}

	if set, ok := fp.conditionalSetExpression(); ok {
		return set, ok
	}

	return nil, false
}

func (fp *formulationParser) chainExpression() (ast.ChainExpression, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	parts := make([]ast.ExpressionType, 0)
	for fp.lexer.HasNext() {
		part, ok := fp.chainExpressionPart()
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
	fp.lexer.Commit(id)
	return ast.ChainExpression{
		Parts: parts,
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) nameOrdinalCallExpression() (ast.NameOrdinalCallExpression, bool) {
	start := fp.lexer.Position()
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
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) pseudoToken(expectedType ast.TokenType) (ast.PseudoTokenNode, bool) {
	start := fp.lexer.Position()
	tok, ok := fp.token(expectedType)
	if !ok {
		return ast.PseudoTokenNode{}, false
	}
	return ast.PseudoTokenNode{
		Text: tok.Text,
		Type: tok.Type,
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) asKeyword() (ast.PseudoTokenNode, bool) {
	return fp.pseudoToken(ast.As)
}

func (fp *formulationParser) isKeyword() (ast.PseudoTokenNode, bool) {
	return fp.pseudoToken(ast.Is)
}

func (fp *formulationParser) extendsKeyword() (ast.PseudoTokenNode, bool) {
	return fp.pseudoToken(ast.Extends)
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

func (fp *formulationParser) colonArrowToken() (ast.PseudoTokenNode, bool) {
	return fp.pseudoToken(ast.ColonArrow)
}

func (fp *formulationParser) pseudoTokenNode() (ast.PseudoTokenNode, bool) {
	if as, ok := fp.asKeyword(); ok {
		return as, ok
	}

	if is, ok := fp.isKeyword(); ok {
		return is, ok
	}

	if extends, ok := fp.extendsKeyword(); ok {
		return extends, ok
	}

	if question, ok := fp.isQuestionMarkKeyword(); ok {
		return question, ok
	}

	if op, ok := fp.operatorToken(); ok {
		return op, ok
	}

	if colonEquals, ok := fp.colonEqualsToken(); ok {
		return colonEquals, ok
	}

	if colonArrow, ok := fp.colonArrowToken(); ok {
		return colonArrow, ok
	}

	return ast.PseudoTokenNode{}, false
}

func (fp *formulationParser) operatorType() (ast.OperatorType, bool) {
	if enclosed, ok := fp.enclosedNonCommandOperatorTarget(); ok {
		return enclosed, ok
	}

	if nonEnclosed, ok := fp.nonEnclosedNonCommandOperatorTarget(); ok {
		return nonEnclosed, ok
	}

	if cmd, ok := fp.commandOperatorTarget(); ok {
		return cmd, ok
	}

	return nil, false
}

func (fp *formulationParser) nonEnclosedNonCommandOperatorTarget() (ast.NonEnclosedNonCommandOperatorTarget, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	hasLeftColon := false
	if _, ok := fp.token(ast.Colon); ok {
		hasLeftColon = true
	}
	if tok, ok := fp.token(ast.Operator); ok {
		fp.lexer.Commit(id)
		hasRightColon := false
		if _, ok := fp.token(ast.Colon); ok {
			hasRightColon = true
		}
		return ast.NonEnclosedNonCommandOperatorTarget{
			Text:          tok.Text,
			HasLeftColon:  hasLeftColon,
			HasRightColon: hasRightColon,
			MetaData: ast.MetaData{
				Start: start,
			},
		}, true
	}
	fp.lexer.RollBack(id)
	return ast.NonEnclosedNonCommandOperatorTarget{}, false
}

func (fp *formulationParser) enclosedNonCommandOperatorTarget() (ast.EnclosedNonCommandOperatorTarget, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	hasLeftColon := false
	if _, ok := fp.token(ast.Colon); ok {
		hasLeftColon = true
	}
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

	hasRightColon := false
	if _, ok := fp.token(ast.Colon); ok {
		hasRightColon = true
	}

	fp.lexer.Commit(id)
	return ast.EnclosedNonCommandOperatorTarget{
		Target:        exp,
		HasLeftColon:  hasLeftColon,
		HasRightColon: hasRightColon,
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) commandOperatorTarget() (ast.CommandOperatorTarget, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	cmd, ok := fp.commandExpression(true)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.CommandOperatorTarget{}, false
	}

	if !fp.has(ast.Slash) {
		fp.lexer.RollBack(id)
		return ast.CommandOperatorTarget{}, false
	}

	fp.expect(ast.Slash)
	fp.lexer.Commit(id)
	return ast.CommandOperatorTarget{
		Command: cmd,
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) namedArg() (ast.NamedArg, bool) {
	start := fp.lexer.Position()
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
	fp.lexer.Commit(id)
	return ast.NamedArg{
		Name: name,
		Args: args,
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) subSupArgs() (ast.SubSupArgs, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	squareArgs, ok := fp.squareParams()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.SubSupArgs{}, false
	}

	subArgs := make([]ast.ExpressionType, 0)
	supArgs := make([]ast.ExpressionType, 0)

	if fp.has(ast.Underscore) {
		fp.expect(ast.Underscore)
		subArgs, _ = fp.curlyArgs()
	}

	if fp.has(ast.Caret) {
		fp.expect(ast.Caret)
		supArgs, _ = fp.curlyArgs()
	}

	fp.lexer.Commit(id)
	return ast.SubSupArgs{
		SquareArgs: squareArgs,
		SubArgs:    subArgs,
		SupArgs:    supArgs,
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) commandExpression(allowOperator bool) (ast.CommandExpression, bool) {
	start := fp.lexer.Position()
	if !fp.hasHas(ast.BackSlash, ast.Name) {
		return ast.CommandExpression{}, false
	}

	id := fp.lexer.Snapshot()
	// move past the backslash
	fp.expect(ast.BackSlash)
	names := make([]ast.NameForm, 0)
	for fp.lexer.HasNext() {
		if name, ok := fp.nameForm(); ok {
			names = append(names, name)
		} else {
			if !allowOperator {
				break
			}
			if op, ok := fp.operatorToken(); ok {
				names = append(names, ast.NameForm{
					Text:            op.Text,
					IsStropped:      false,
					HasQuestionMark: false,
					VarArg:          ast.VarArgData{},
					MetaData:        op.MetaData,
				})
			} else {
				break
			}
		}
		if fp.has(ast.Dot) {
			fp.expect(ast.Dot)
		} else {
			break
		}
	}

	if len(names) == 0 {
		fp.lexer.RollBack(id)
		return ast.CommandExpression{}, false
	}

	subSupArgs, _ := fp.subSupArgs()
	curlyArgs, _ := fp.curlyArgs()

	namedArgs := make([]ast.NamedArg, 0)
	for fp.lexer.HasNext() {
		if namedArg, ok := fp.namedArg(); ok {
			namedArgs = append(namedArgs, namedArg)
		} else {
			break
		}
	}

	parenArgs, _ := fp.parenArgs()

	fp.lexer.Commit(id)
	return ast.CommandExpression{
		Names:      names,
		SubSupArgs: &subSupArgs,
		CurlyArgs:  &curlyArgs,
		NamedArgs:  &namedArgs,
		ParenArgs:  &parenArgs,
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) commandAtExpression() (ast.CommandAtExpression, bool) {
	start := fp.lexer.Position()
	if !fp.hasHas(ast.BackSlash, ast.Name) {
		return ast.CommandAtExpression{}, false
	}

	id := fp.lexer.Snapshot()
	fp.expect(ast.BackSlash)
	names := make([]ast.NameForm, 0)
	for fp.lexer.HasNext() {
		if name, ok := fp.nameForm(); ok {
			names = append(names, name)
		} else {
			break
		}
		if fp.has(ast.Dot) {
			fp.expect(ast.Dot)
		} else {
			break
		}
	}

	if len(names) == 0 {
		fp.lexer.RollBack(id)
		return ast.CommandAtExpression{}, false
	}

	if !fp.has(ast.At) {
		fp.lexer.RollBack(id)
		return ast.CommandAtExpression{}, false
	}

	fp.expect(ast.At)
	exp, ok := fp.expressionType()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.CommandAtExpression{}, false
	}

	fp.lexer.Commit(id)
	return ast.CommandAtExpression{
		Names:      names,
		Expression: exp,
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

/////////////////////////// forms ///////////////////////////////////////

func (fp *formulationParser) form() (ast.NodeType, bool) {
	start := fp.lexer.Position()
	if fp.hasHas(ast.Name, ast.ColonEquals) {
		id := fp.lexer.Snapshot()
		name, ok := fp.nameForm()
		if !ok {
			fp.lexer.RollBack(id)
			return nil, false
		}

		fp.expect(ast.ColonEquals)
		rhs, ok := fp.structuralFormType()
		if !ok {
			fp.lexer.RollBack(id)
			fp.error("Expected an item on the righ-hand-side of :=")
			return nil, false
		}

		fp.lexer.Commit(id)
		return ast.StructuralColonEqualsForm{
			Lhs: name,
			Rhs: rhs,
			MetaData: ast.MetaData{
				Start: start,
			},
		}, true
	} else {
		return fp.structuralFormType()
	}
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

func (fp *formulationParser) nameParams() ([]ast.NameForm, bool) {
	id := fp.lexer.Snapshot()
	_, ok := fp.token(ast.LParen)
	if !ok {
		fp.lexer.RollBack(id)
		return []ast.NameForm{}, false
	}
	names := make([]ast.NameForm, 0)
	for fp.lexer.HasNext() {
		if fp.has(ast.RParen) {
			break
		}

		if len(names) > 0 {
			fp.expect(ast.Comma)
		}

		name, ok := fp.nameForm()
		if !ok {
			fp.lexer.RollBack(id)
			return []ast.NameForm{}, false
		}
		names = append(names, name)
	}
	fp.expect(ast.RParen)
	fp.lexer.Commit(id)
	return names, true
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
	start := fp.lexer.Position()
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
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) functionForm() (ast.FunctionForm, bool) {
	start := fp.lexer.Position()
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
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) functionExpressionForm() (ast.FunctionExpressionForm, bool) {
	start := fp.lexer.Position()
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
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) tupleForm() (ast.TupleForm, bool) {
	start := fp.lexer.Position()
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
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) fixedSetForm() (ast.FixedSetForm, bool) {
	start := fp.lexer.Position()
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
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) conditionalSetForm() (ast.ConditionalSetForm, bool) {
	start := fp.lexer.Position()
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
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) literalFormType() (ast.LiteralFormType, bool) {
	if name, ok := fp.nameForm(); ok {
		return name, ok
	}

	if fun, ok := fp.functionForm(); ok {
		return fun, ok
	}

	if tup, ok := fp.tupleForm(); ok {
		return tup, ok
	}

	if set, ok := fp.fixedSetForm(); ok {
		return set, ok
	}

	if set, ok := fp.conditionalSetIdForm(); ok {
		return set, ok
	}

	return nil, false
}

////////////////////////////// id forms //////////////////////////////////

func (fp *formulationParser) idType() (ast.IdType, bool) {
	if op, ok := fp.infixCommandOperatorId(); ok {
		return op, ok
	} else if op, ok := fp.infixOperatorId(); ok {
		return op, ok
	} else if op, ok := fp.prefixOperatorId(); ok {
		return op, ok
	} else if op, ok := fp.postfixOperatorId(); ok {
		return op, ok
	} else if cmd, ok := fp.commandAtId(); ok {
		return cmd, ok
	} else if cmd, ok := fp.commandId(false); ok {
		return cmd, ok
	} else {
		return nil, false
	}
}

func (fp *formulationParser) infixCommandOperatorId() (ast.InfixCommandOperatorId, bool) {
	id := fp.lexer.Snapshot()

	lhs, ok := fp.structuralFormType()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.InfixCommandOperatorId{}, false
	}

	op, ok := fp.infixCommandId()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.InfixCommandOperatorId{}, false
	}

	rhs, ok := fp.structuralFormType()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.InfixCommandOperatorId{}, false
	}

	fp.lexer.Commit(id)
	return ast.InfixCommandOperatorId{
		Lhs:      lhs,
		Operator: op,
		Rhs:      rhs,
		MetaData: op.MetaData,
	}, true
}

func (fp *formulationParser) infixOperatorId() (ast.InfixOperatorId, bool) {
	id := fp.lexer.Snapshot()

	lhs, ok := fp.structuralFormType()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.InfixOperatorId{}, false
	}

	op, ok := fp.nonEnclosedNonCommandOperatorTarget()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.InfixOperatorId{}, false
	}

	rhs, ok := fp.structuralFormType()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.InfixOperatorId{}, false
	}

	fp.lexer.Commit(id)
	return ast.InfixOperatorId{
		Lhs:      lhs,
		Operator: op,
		Rhs:      rhs,
		MetaData: op.MetaData,
	}, true
}

func (fp *formulationParser) postfixOperatorId() (ast.PostfixOperatorId, bool) {
	id := fp.lexer.Snapshot()

	param, ok := fp.structuralFormType()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.PostfixOperatorId{}, false
	}

	op, ok := fp.nonEnclosedNonCommandOperatorTarget()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.PostfixOperatorId{}, false
	}

	fp.lexer.Commit(id)
	return ast.PostfixOperatorId{
		Operator: op,
		Param:    param,
		MetaData: op.MetaData,
	}, true
}

func (fp *formulationParser) prefixOperatorId() (ast.PrefixOperatorId, bool) {
	id := fp.lexer.Snapshot()

	op, ok := fp.nonEnclosedNonCommandOperatorTarget()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.PrefixOperatorId{}, false
	}

	param, ok := fp.structuralFormType()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.PrefixOperatorId{}, false
	}

	fp.lexer.Commit(id)
	return ast.PrefixOperatorId{
		Operator: op,
		Param:    param,
		MetaData: op.MetaData,
	}, true
}

func (fp *formulationParser) conditionalSetIdForm() (ast.ConditionalSetIdForm, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	symbols, ok := fp.squareParams()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetIdForm{}, false
	}

	if !fp.has(ast.LCurly) {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetIdForm{}, false
	}

	fp.expect(ast.LCurly)
	target, ok := fp.structuralFormType()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetIdForm{}, false
	}

	if !fp.has(ast.Bar) {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetIdForm{}, false
	}

	fp.expect(ast.Bar)
	condition, ok := fp.functionExpressionForm()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetIdForm{}, false
	}

	if !fp.has(ast.RCurly) {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetIdForm{}, false
	}

	fp.expect(ast.RCurly)
	fp.lexer.Commit(id)
	return ast.ConditionalSetIdForm{
		Symbols:   symbols,
		Target:    target,
		Condition: condition,
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) namedParam() (ast.NamedParam, bool) {
	start := fp.lexer.Position()
	if !fp.hasHas(ast.Colon, ast.Name) {
		return ast.NamedParam{}, false
	}
	id := fp.lexer.Snapshot()
	fp.expect(ast.Colon)
	name, ok := fp.nameForm()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.NamedParam{}, false
	}
	var params *[]ast.StructuralFormType = nil
	if curlyParams, ok := fp.curlyParams(); ok {
		params = &curlyParams
	}
	fp.lexer.Commit(id)
	return ast.NamedParam{
		Name:   name,
		Params: params,
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) subSupParams() (ast.SubSupParams, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	squareParams, ok := fp.squareParams()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.SubSupParams{}, false
	}

	subParams := make([]ast.StructuralFormType, 0)
	supParams := make([]ast.StructuralFormType, 0)

	if fp.has(ast.Underscore) {
		fp.expect(ast.Underscore)
		subParams, _ = fp.curlyParams()
	}

	if fp.has(ast.Caret) {
		fp.expect(ast.Caret)
		supParams, _ = fp.curlyParams()
	}

	fp.lexer.Commit(id)
	return ast.SubSupParams{
		SquareParams: squareParams,
		SubParams:    subParams,
		SupParams:    supParams,
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) infixCommandId() (ast.InfixCommandId, bool) {
	id := fp.lexer.Snapshot()
	cmd, ok := fp.commandId(true)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.InfixCommandId{}, false
	}

	_, ok = fp.token(ast.Slash)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.InfixCommandId{}, false
	}

	fp.lexer.Commit(id)
	return ast.InfixCommandId{
		Names:        cmd.Names,
		SubSupParams: cmd.SubSupParams,
		CurlyParams:  cmd.CurlyParams,
		NamedParams:  cmd.NamedParams,
		ParenParams:  cmd.ParenParams,
		MetaData:     cmd.MetaData,
	}, true
}

func (fp *formulationParser) commandId(allowOperator bool) (ast.CommandId, bool) {
	start := fp.lexer.Position()
	if !fp.hasHas(ast.BackSlash, ast.Name) {
		return ast.CommandId{}, false
	}

	id := fp.lexer.Snapshot()
	fp.expect(ast.BackSlash)
	names := make([]ast.NameForm, 0)
	for fp.lexer.HasNext() {
		if name, ok := fp.nameForm(); ok {
			names = append(names, name)
		} else {
			if !allowOperator {
				break
			}
			if op, ok := fp.operatorToken(); ok {
				names = append(names, ast.NameForm{
					Text:            op.Text,
					IsStropped:      false,
					HasQuestionMark: false,
					VarArg:          ast.VarArgData{},
					MetaData:        op.MetaData,
				})
			} else {
				break
			}
		}
		if fp.has(ast.Dot) {
			fp.expect(ast.Dot)
		} else {
			break
		}
	}

	if len(names) == 0 {
		fp.lexer.RollBack(id)
		return ast.CommandId{}, false
	}

	subSupParams, _ := fp.subSupParams()
	curlyParams, _ := fp.curlyParams()

	namedParams := make([]ast.NamedParam, 0)
	for fp.lexer.HasNext() {
		if namedParam, ok := fp.namedParam(); ok {
			namedParams = append(namedParams, namedParam)
		} else {
			break
		}
	}

	parenParams, _ := fp.nameParams()

	fp.lexer.Commit(id)
	return ast.CommandId{
		Names:        names,
		SubSupParams: &subSupParams,
		CurlyParams:  &curlyParams,
		NamedParams:  &namedParams,
		ParenParams:  &parenParams,
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

func (fp *formulationParser) commandAtId() (ast.CommandAtId, bool) {
	start := fp.lexer.Position()
	if !fp.hasHas(ast.BackSlash, ast.Name) {
		return ast.CommandAtId{}, false
	}

	id := fp.lexer.Snapshot()
	fp.expect(ast.BackSlash)
	names := make([]ast.NameForm, 0)
	for fp.lexer.HasNext() {
		if name, ok := fp.nameForm(); ok {
			names = append(names, name)
		} else {
			break
		}
		if fp.has(ast.Dot) {
			fp.expect(ast.Dot)
		} else {
			break
		}
	}

	if len(names) == 0 {
		fp.lexer.RollBack(id)
		return ast.CommandAtId{}, false
	}

	if !fp.has(ast.At) {
		fp.lexer.RollBack(id)
		return ast.CommandAtId{}, false
	}

	fp.expect(ast.At)
	literal, ok := fp.literalFormType()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.CommandAtId{}, false
	}

	fp.lexer.Commit(id)
	return ast.CommandAtId{
		Names: names,
		Param: literal,
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

//////////////////////////////// signature ////////////////////////////////////////////

func (fp *formulationParser) signature() (ast.Signature, bool) {
	start := fp.lexer.Position()
	if !fp.hasHas(ast.BackSlash, ast.LSquare) {
		return ast.Signature{}, false
	}
	fp.expect(ast.BackSlash)
	fp.expect(ast.LSquare)
	mainNames := make([]string, 0)
	namedGroupNames := make([]string, 0)
	hasAtSymbol := false
	for fp.lexer.HasNext() && !fp.has(ast.LSquare) {
		if fp.has(ast.At) || fp.has(ast.Colon) {
			break
		}
		name, ok := fp.nameForm()
		if !ok {
			fp.error("Expected a name")
			break
		}
		mainNames = append(mainNames, name.Text)
	}
	if fp.has(ast.At) {
		fp.expect(ast.At)
		hasAtSymbol = true
	}
	if !hasAtSymbol {
		for fp.lexer.HasNext() && !fp.has(ast.LSquare) {
			_, ok := fp.expect(ast.Colon)
			if !ok {
				break
			}
			name, ok := fp.nameForm()
			if !ok {
				break
			}
			namedGroupNames = append(namedGroupNames, name.Text)
		}
	}
	for fp.lexer.HasNext() && !fp.has(ast.LSquare) {
		fp.error("Unexpected text")
		fp.lexer.Next()
	}
	fp.expect(ast.LSquare)
	return ast.Signature{
		MainNames:       mainNames,
		NamedGroupNames: namedGroupNames,
		HasAtSymbol:     hasAtSymbol,
		MetaData: ast.MetaData{
			Start: start,
		},
	}, true
}

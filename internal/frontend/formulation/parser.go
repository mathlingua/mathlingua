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
	"mathlingua/internal/mlglib"
	"strings"
)

func ParseExpression(
	path ast.Path,
	text string,
	start ast.Position,
	tracker *frontend.DiagnosticTracker,
	keyGen *mlglib.KeyGenerator,
) (ast.FormulationNodeKind, bool) {
	numDiagBefore := tracker.Length()
	lexer := NewLexer(path, text, tracker)
	parser := formulationParser{
		path:    path,
		lexer:   lexer,
		tracker: tracker,
		start:   start,
		keyGen:  keyGen,
	}
	node, _ := parser.multiplexedExpressionKind()
	parser.finalize()
	return node, node != nil && tracker.Length() == numDiagBefore
}

func ParseForm(
	path ast.Path,
	text string,
	start ast.Position,
	tracker *frontend.DiagnosticTracker,
	keyGen *mlglib.KeyGenerator,
) (ast.FormulationNodeKind, bool) {
	numDiagBefore := tracker.Length()
	lexer := NewLexer(path, text, tracker)
	parser := formulationParser{
		path:    path,
		lexer:   lexer,
		tracker: tracker,
		start:   start,
		keyGen:  keyGen,
	}
	node, _ := parser.structuralFormKindPossiblyWithColonEquals()
	parser.finalize()
	return node, node != nil && tracker.Length() == numDiagBefore
}

func ParseId(
	path ast.Path,
	text string,
	start ast.Position,
	tracker *frontend.DiagnosticTracker,
	keyGen *mlglib.KeyGenerator,
) (ast.IdKind, bool) {
	numDiagBefore := tracker.Length()
	lexer := NewLexer(path, text, tracker)
	parser := formulationParser{
		path:    path,
		lexer:   lexer,
		tracker: tracker,
		start:   start,
		keyGen:  keyGen,
	}
	node, _ := parser.idKind()
	parser.finalize()
	return node, node != nil && tracker.Length() == numDiagBefore
}

func ParseSignature(
	path ast.Path,
	text string,
	start ast.Position,
	tracker *frontend.DiagnosticTracker,
	keyGen *mlglib.KeyGenerator,
) (ast.Signature, bool) {
	numDiagBefore := tracker.Length()
	lexer := NewLexer(path, text, tracker)
	parser := formulationParser{
		path:    path,
		lexer:   lexer,
		tracker: tracker,
		start:   start,
		keyGen:  keyGen,
	}
	node, _ := parser.signature()
	parser.finalize()
	return node, tracker.Length() == numDiagBefore
}

//////////////////////////////// utility functions /////////////////////////////////////////////////

type formulationParser struct {
	path    ast.Path
	lexer   *frontend.Lexer
	tracker *frontend.DiagnosticTracker
	start   ast.Position
	keyGen  *mlglib.KeyGenerator
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
	return fp.lexer.HasNextNext() && fp.lexer.Peek().Type == tokenType1 &&
		fp.lexer.PeekPeek().Type == tokenType2
}

func (fp *formulationParser) next() ast.Token {
	return fp.lexer.Next()
}

func (fp *formulationParser) getShiftedPosition(position ast.Position) ast.Position {
	return ast.Position{
		Row:    fp.start.Row + position.Row,
		Column: fp.start.Column + position.Column,
		Offset: fp.start.Offset + position.Offset,
	}
}

func (fp *formulationParser) errorAt(message string, position ast.Position) {
	fp.tracker.Append(frontend.Diagnostic{
		Type:     frontend.Error,
		Path:     fp.path,
		Origin:   frontend.FormulationParserOrigin,
		Message:  message,
		Position: fp.getShiftedPosition(position),
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

///////////////////////////////////// expressions //////////////////////////////////////////////////

func (fp *formulationParser) parenArgs() (*[]ast.ExpressionKind, bool) {
	id := fp.lexer.Snapshot()
	_, ok := fp.token(ast.LParen)
	if !ok {
		fp.lexer.RollBack(id)
		return nil, false
	}
	args := make([]ast.ExpressionKind, 0)
	for fp.lexer.HasNext() {
		if fp.has(ast.RParen) {
			break
		}

		if len(args) > 0 {
			fp.expect(ast.Comma)
		}

		arg, ok := fp.expressionKind()
		if !ok {
			fp.lexer.RollBack(id)
			return nil, false
		}
		args = append(args, arg)
	}
	fp.expect(ast.RParen)
	fp.lexer.Commit(id)
	return &args, true
}

func (fp *formulationParser) curlyArgs() (*[]ast.ExpressionKind, bool) {
	id := fp.lexer.Snapshot()
	_, ok := fp.token(ast.LCurly)
	if !ok {
		fp.lexer.RollBack(id)
		return nil, false
	}
	args := make([]ast.ExpressionKind, 0)
	for fp.lexer.HasNext() {
		if fp.has(ast.RCurly) {
			break
		}

		if len(args) > 0 {
			fp.expect(ast.Comma)
		}

		arg, ok := fp.expressionKind()
		if !ok {
			fp.lexer.RollBack(id)
			return nil, false
		}
		args = append(args, arg)
	}
	fp.expect(ast.RCurly)
	fp.lexer.Commit(id)
	return &args, true
}

func (fp *formulationParser) varArgData() (ast.VarArgData, bool) {
	start := fp.lexer.Position()
	if fp.has(ast.DotDotDot) {
		fp.next() // absorb the ...
		return ast.VarArgData{
			IsVarArg:     true,
			VarArgNames:  nil,
			VarArgBounds: nil,
		}, true
	}
	if !fp.hasHas(ast.LCurly, ast.Name) && !fp.hasHas(ast.LCurly, ast.LParen) {
		return ast.VarArgData{
			IsVarArg: false,
		}, false
	}
	varArgBounds := make([]ast.NameForm, 0)
	id := fp.lexer.Snapshot()
	fp.expect(ast.LCurly)
	if fp.has(ast.Name) {
		// its of the form 'name...bound'
		varName, ok := fp.token(ast.Name)
		if !ok {
			fp.lexer.RollBack(id)
			return ast.VarArgData{
				IsVarArg: false,
			}, false
		}
		if !fp.has(ast.DotDotDot) {
			fp.lexer.RollBack(id)
			return ast.VarArgData{
				IsVarArg: false,
			}, false
		}
		fp.next() // absorb the ...
		bound, ok := fp.token(ast.Name)
		if ok {
			varArgBounds = append(varArgBounds,
				ast.NameForm{
					Text:            bound.Text,
					IsStropped:      false,
					HasQuestionMark: false,
					VarArg: ast.VarArgData{
						IsVarArg:            false,
						VarArgNames:         nil,
						VarArgBounds:        nil,
						CommonMetaData:      ast.CommonMetaData{},
						FormulationMetaData: ast.FormulationMetaData{},
					},
				})
		}
		fp.expect(ast.RCurly)
		fp.lexer.Commit(id)
		return ast.VarArgData{
			IsVarArg: true,
			VarArgNames: []ast.NameForm{
				{
					Text:            varName.Text,
					IsStropped:      false,
					HasQuestionMark: false,
					VarArg: ast.VarArgData{
						IsVarArg:            false,
						VarArgNames:         nil,
						VarArgBounds:        nil,
						CommonMetaData:      ast.CommonMetaData{},
						FormulationMetaData: ast.FormulationMetaData{},
					},
				},
			},
			VarArgBounds: varArgBounds,
			CommonMetaData: ast.CommonMetaData{
				Start: start,
				Key:   fp.keyGen.Next(),
			},
		}, true
	} else if fp.has(ast.LParen) {
		// its of the form '(name1,name2)...(bound1,bound2)'
		fp.next() // move past the (
		varNames := make([]ast.NameForm, 0)
		for fp.lexer.HasNext() {
			if fp.has(ast.RParen) {
				break
			}

			if len(varNames) > 0 {
				fp.expect(ast.Comma)
			}

			name, ok := fp.token(ast.Name)
			if !ok {
				fp.lexer.RollBack(id)
				return ast.VarArgData{}, false
			}
			varNames = append(varNames,
				ast.NameForm{
					Text:            name.Text,
					IsStropped:      false,
					HasQuestionMark: false,
					VarArg: ast.VarArgData{
						IsVarArg:            false,
						VarArgNames:         nil,
						VarArgBounds:        nil,
						CommonMetaData:      ast.CommonMetaData{},
						FormulationMetaData: ast.FormulationMetaData{},
					},
				})
		}
		fp.expect(ast.RParen)
		_, ok := fp.token(ast.DotDotDot)
		if !ok {
			fp.lexer.RollBack(id)
			return ast.VarArgData{
				IsVarArg: false,
			}, false
		}
		fp.expect(ast.LParen)
		bounds := make([]ast.NameForm, 0)
		for fp.lexer.HasNext() {
			if fp.has(ast.RParen) {
				break
			}

			if len(bounds) > 0 {
				fp.expect(ast.Comma)
			}

			name, ok := fp.token(ast.Name)
			if !ok {
				fp.lexer.RollBack(id)
				return ast.VarArgData{
					IsVarArg: false,
				}, false
			}
			bounds = append(bounds,
				ast.NameForm{
					Text:            name.Text,
					IsStropped:      false,
					HasQuestionMark: false,
					VarArg: ast.VarArgData{
						IsVarArg:            false,
						VarArgNames:         nil,
						VarArgBounds:        nil,
						CommonMetaData:      ast.CommonMetaData{},
						FormulationMetaData: ast.FormulationMetaData{},
					},
				})
		}
		fp.expect(ast.RParen)
		fp.expect(ast.RCurly)
		fp.lexer.Commit(id)
		return ast.VarArgData{
			IsVarArg:     true,
			VarArgNames:  varNames,
			VarArgBounds: bounds,
			CommonMetaData: ast.CommonMetaData{
				Start: start,
				Key:   fp.keyGen.Next(),
			},
		}, true
	} else {
		fp.lexer.RollBack(id)
		return ast.VarArgData{
			IsVarArg: false,
		}, false
	}
}

func (fp *formulationParser) literalExpressionKind() (ast.LiteralExpressionKind, bool) {
	if set, ok := fp.conditionalSetExpression(); ok {
		return &set, ok
	}

	if fun, ok := fp.expressionForm(); ok {
		return &fun, ok
	}

	if fun, ok := fp.functionCallExpression(); ok {
		return &fun, ok
	}

	if fun, ok := fp.functionLiteralExpression(); ok {
		return &fun, ok
	}

	if tup, ok := fp.tupleExpression(); ok {
		return &tup, ok
	}

	return nil, false
}

func (fp *formulationParser) functionLiteralExpression() (ast.FunctionLiteralExpression, bool) {
	id := fp.lexer.Snapshot()

	if fp.hasHas(ast.Name, ast.BarRightDashArrow) {
		name, nameOk := fp.nameForm()
		if !nameOk {
			fp.lexer.RollBack(id)
			return ast.FunctionLiteralExpression{}, false
		}
		fp.expect(ast.BarRightDashArrow)
		exp, expOk := fp.expressionKind()
		if !expOk {
			fp.lexer.RollBack(id)
			return ast.FunctionLiteralExpression{}, false
		}
		fp.lexer.Commit(id)
		return ast.FunctionLiteralExpression{
			Lhs: ast.TupleForm{
				Params: []ast.StructuralFormKind{
					&name,
				},
				VarArg: ast.VarArgData{
					IsVarArg: false,
				},
				CommonMetaData: name.CommonMetaData,
			},
			Rhs:            exp,
			CommonMetaData: name.CommonMetaData,
		}, true
	}

	tup, tupOk := fp.tupleForm()
	if !tupOk {
		fp.lexer.RollBack(id)
		return ast.FunctionLiteralExpression{}, false
	}

	_, arrowOk := fp.token(ast.BarRightDashArrow)
	if !arrowOk {
		fp.lexer.RollBack(id)
		return ast.FunctionLiteralExpression{}, false
	}

	exp, expOk := fp.expressionKind()
	if !expOk {
		fp.lexer.RollBack(id)
		return ast.FunctionLiteralExpression{}, false
	}

	fp.lexer.Commit(id)
	return ast.FunctionLiteralExpression{
		Lhs:            tup,
		Rhs:            exp,
		CommonMetaData: tup.CommonMetaData,
	}, true
}

func (fp *formulationParser) multiplexedExpressionKind() (ast.ExpressionKind, bool) {
	start := fp.lexer.Position()
	items := make([]ast.ExpressionKind, 0)
	for fp.lexer.HasNext() {
		if len(items) > 0 {
			if fp.has(ast.Comma) {
				fp.expect(ast.Comma)
			} else {
				break
			}
		}
		arg, ok := fp.expressionKind()
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

	minPrecIndexMinIndex := math.MaxInt
	minPrecIndexMaxIndex := -1
	minPrec := math.MinInt

	for i, item := range items {
		_, isOk := item.(*ast.IsExpression)
		if isOk {
			if isIndex >= 0 {
				fp.error("'is' statements cannot be nested")
			}
			isIndex = i
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

	// if there isn't an 'is' statement and there
	// is at least one infix operator
	if isIndex == -1 && minPrecIndexMaxIndex >= 0 {
		if minPrecIndexMinIndex != minPrecIndexMaxIndex {
			fp.error(
				"A multiplexed operator can only be used if exactly one operator has minimum precedence")
			return nil, false
		} else {
			lhs := make([]ast.ExpressionKind, 0)
			rhs := make([]ast.ExpressionKind, 0)
			i := 0
			for i < minPrecIndexMinIndex {
				lhs = append(lhs, items[i])
				i++
			}
			opExp := items[minPrecIndexMinIndex].(*ast.InfixOperatorCallExpression)
			lhs = append(lhs, opExp.Lhs)
			rhs = append(rhs, opExp.Rhs)
			i = minPrecIndexMinIndex + 1
			for i < len(items) {
				rhs = append(rhs, items[i])
				i++
			}
			return &ast.MultiplexedInfixOperatorCallExpression{
				Target: opExp.Target,
				Lhs:    lhs,
				Rhs:    rhs,
				CommonMetaData: ast.CommonMetaData{
					Start: fp.getShiftedPosition(start),
					Key:   fp.keyGen.Next(),
				},
			}, true
		}
	}

	if isIndex >= 0 {
		lhs := make([]ast.ExpressionKind, 0)
		rhs := make([]ast.KindKind, 0)
		i := 0
		for i < isIndex {
			lhs = append(lhs, items[i])
			i++
		}
		isExp := items[isIndex].(*ast.IsExpression)
		lhs = append(lhs, isExp.Lhs...)
		rhs = append(rhs, isExp.Rhs...)
		i = isIndex + 1
		for i < len(items) {
			rhs = append(rhs, items[i].(ast.KindKind))
			i++
		}
		if len(rhs) != 1 {
			fp.errorAt(
				fmt.Sprintf(
					"The right-hand-side of an 'is' expression must contain exactly "+
						"one item, but found %d",
					len(rhs)),
				isExp.Start())
		}
		return &ast.IsExpression{
			Lhs: lhs,
			Rhs: rhs,
			CommonMetaData: ast.CommonMetaData{
				Start: fp.getShiftedPosition(start),
				Key:   fp.keyGen.Next(),
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

func (fp *formulationParser) expressionKind(
	additionalTerminators ...ast.TokenType) (ast.ExpressionKind, bool) {
	if exp, ok := fp.pseudoExpression(additionalTerminators...); ok {
		res, consolidateOk := Consolidate(fp.path, exp.Children, fp.tracker)
		if resAsExp, resAsExpOk := res.(ast.ExpressionKind); resAsExpOk {
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
		tokenType == ast.RCurly
}

func (fp *formulationParser) pseudoExpression(
	additionalTerminators ...ast.TokenType) (ast.PseudoExpression, bool) {
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
	children := make([]ast.FormulationNodeKind, 0)
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

		if exp, ok := fp.abstractBuiltinExpression(); ok {
			children = append(children, &exp)
		} else if exp, ok := fp.specificationBuiltinExpression(); ok {
			children = append(children, &exp)
		} else if exp, ok := fp.statementBuiltinExpression(); ok {
			children = append(children, &exp)
		} else if exp, ok := fp.expressionBuiltinExpression(); ok {
			children = append(children, &exp)
		} else if exp, ok := fp.typeBuiltinExpression(); ok {
			children = append(children, &exp)
		} else if cmd, ok := fp.typeKind(true); ok {
			children = append(children, cmd)
		} else if op, ok := fp.operatorKind(); ok {
			children = append(children, op)
		} else if chain, ok := fp.chainExpression(false); ok {
			children = append(children, &chain)
		} else if lit, ok := fp.literalExpressionKind(); ok {
			children = append(children, lit)
		} else if pseudoToken, ok := fp.pseudoTokenNode(); ok {
			children = append(children, &pseudoToken)
		} else if ord, ok := fp.ordinalCallExpression(); ok {
			children = append(children, &ord)
		} else if fun, ok := fp.expressionForm(); ok {
			children = append(children, &fun)
		} else if fun, ok := fp.functionForm(); ok {
			children = append(children, &fun)
		} else if name, ok := fp.nameForm(); ok {
			children = append(children, &name)
		} else if fun, ok := fp.functionCallExpression(); ok {
			children = append(children, &fun)
		} else if tup, ok := fp.tupleForm(); ok {
			children = append(children, &tup)
		} else if set, ok := fp.conditionalSetForm(); ok {
			children = append(children, &set)
		} else if sig, ok := fp.signature(); ok {
			children = append(children, &sig)
		} else if builtin, ok := fp.mapToElseBuiltinExpression(); ok {
			children = append(children, &builtin)
		} else if cmd, ok := fp.definitionBuiltinExpression(); ok {
			children = append(children, &cmd)
		} else if cmd, ok := fp.infixCommandExpression(); ok {
			children = append(children, &cmd)
		} else if cmd, ok := fp.commandExpression(false); ok {
			children = append(children, &cmd)
		} else if grp, ok := fp.labeledGrouping(); ok {
			children = append(children, &grp)
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
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
	}, true
}

func (fp *formulationParser) kinds() []string {
	fp.expect(ast.LCurly)
	kinds := make([]string, 0)
	for fp.lexer.HasNext() {
		if fp.has(ast.RCurly) {
			break
		}

		if len(kinds) > 0 {
			fp.expect(ast.Bar)
		}

		if tok, ok := fp.token(ast.Name); ok {
			kind := tok.Text
			if kind == "specification" || kind == "statement" || kind == "expression" {
				kinds = append(kinds, kind)
			} else {
				fp.error("Expected one of 'specification', 'statement', 'expression'")
				// move past the unexpected token
				fp.lexer.Next()
			}
		} else {
			fp.error("Expected one of 'specification', 'statement', 'expression'")
			// move past the unexpected token
			fp.lexer.Next()
		}
	}
	fp.expect(ast.RCurly)
	return kinds
}

func (fp *formulationParser) mapToElseBuiltinExpression() (ast.MapToElseBuiltinExpression, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	if !fp.hasHas(ast.BackSlash, ast.BackSlash) {
		fp.lexer.RollBack(id)
		return ast.MapToElseBuiltinExpression{}, false
	}

	fp.expect(ast.BackSlash) // skip the \
	fp.expect(ast.BackSlash) // skip the \

	if !fp.has(ast.Name) || fp.lexer.Peek().Text != "map" {
		fp.lexer.RollBack(id)
		return ast.MapToElseBuiltinExpression{}, false
	}

	mapName, _ := fp.expect(ast.Name) // skip the "map" name

	if !fp.has(ast.LCurly) {
		fp.errorAt("Expected at {", mapName.Position)
		fp.lexer.Commit(id)
		return ast.MapToElseBuiltinExpression{}, true
	}

	fp.expect(ast.LCurly)
	target, ok := fp.ordinalCallExpression()
	if !ok {
		fp.errorAt("Expected an ordinal expression as an argument", mapName.Position)
		fp.lexer.Commit(id)
		return ast.MapToElseBuiltinExpression{}, true
	}
	fp.expect(ast.RCurly)

	fp.expect(ast.Colon)
	if !fp.has(ast.Name) || fp.lexer.Peek().Text != "to" {
		fp.lexer.Commit(id)
		return ast.MapToElseBuiltinExpression{}, true
	}

	toName, _ := fp.expect(ast.Name) // skip the "to" name
	fp.expect(ast.LCurly)
	to, ok := fp.expressionKind()
	if !ok {
		fp.errorAt("Expected an expression", toName.Position)
		fp.lexer.Commit(id)
		return ast.MapToElseBuiltinExpression{}, true
	}
	fp.expect(ast.RCurly)

	var elseExp ast.ExpressionKind
	if fp.hasHas(ast.Colon, ast.Name) && fp.lexer.PeekPeek().Text == "else" {
		fp.expect(ast.Colon)
		elseName, _ := fp.expect(ast.Name) // skip the else
		fp.expect(ast.LCurly)
		elseExp, ok = fp.expressionKind()
		if !ok {
			fp.errorAt("Expected an expression", elseName.Position)
			fp.lexer.Commit(id)
			return ast.MapToElseBuiltinExpression{}, true
		}
		fp.expect(ast.RCurly)
	}

	fp.lexer.Commit(id)
	return ast.MapToElseBuiltinExpression{
		Target: target,
		To:     to,
		Else:   elseExp,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
	}, true
}

func (fp *formulationParser) definitionBuiltinExpression() (ast.DefinitionBuiltinExpression, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	if !fp.hasHas(ast.BackSlash, ast.BackSlash) {
		fp.lexer.RollBack(id)
		return ast.DefinitionBuiltinExpression{}, false
	}

	fp.expect(ast.BackSlash) // skip the \
	fp.expect(ast.BackSlash) // skip the \

	if !fp.has(ast.Name) || fp.lexer.Peek().Text != "definition" {
		fp.lexer.RollBack(id)
		return ast.DefinitionBuiltinExpression{}, false
	}

	formulationName, _ := fp.expect(ast.Name) // skip the "definition" name

	if !fp.hasHas(ast.Colon, ast.Name) || fp.lexer.PeekPeek().Text != "of" {
		fp.errorAt("Expected :of{} to follow \\\\definition{}", formulationName.Position)
		fp.lexer.Commit(id)
		return ast.DefinitionBuiltinExpression{
			CommonMetaData: ast.CommonMetaData{
				Start: fp.getShiftedPosition(start),
				Key:   fp.keyGen.Next(),
			},
		}, true
	}

	fp.expect(ast.Colon)
	ofName, _ := fp.expect(ast.Name) // move past the of name

	if !fp.has(ast.LCurly) {
		fp.errorAt("Expected {", ofName.Position)
		fp.lexer.Commit(id)
		return ast.DefinitionBuiltinExpression{
			CommonMetaData: ast.CommonMetaData{
				Start: fp.getShiftedPosition(start),
				Key:   fp.keyGen.Next(),
			},
		}, true
	}

	fp.expect(ast.LCurly)
	of, ok := fp.expressionKind()
	if !ok {
		fp.errorAt("Expected a name", ofName.Position)
		fp.lexer.Commit(id)
		return ast.DefinitionBuiltinExpression{
			CommonMetaData: ast.CommonMetaData{
				Start: fp.getShiftedPosition(start),
				Key:   fp.keyGen.Next(),
			},
		}, true
	}
	fp.expect(ast.RCurly)

	var satisfies ast.ExpressionKind
	if fp.hasHas(ast.Colon, ast.Name) && fp.lexer.PeekPeek().Text == "satisfies" {
		fp.expect(ast.Colon)
		satisfiesName, _ := fp.expect(ast.Name) // move past the satisfies name

		if !fp.has(ast.LCurly) {
			fp.errorAt("Expected {", satisfiesName.Position)
			fp.lexer.Commit(id)
			return ast.DefinitionBuiltinExpression{
				CommonMetaData: ast.CommonMetaData{
					Start: fp.getShiftedPosition(start),
					Key:   fp.keyGen.Next(),
				},
			}, true
		}

		fp.expect(ast.LCurly)
		if signature, ok := fp.signature(); ok {
			satisfies = &signature
		} else {
			satisfies, ok = fp.expressionKind()
			if !ok {
				fp.errorAt("Expected an expression or signature", satisfiesName.Position)
				fp.lexer.Commit(id)
				return ast.DefinitionBuiltinExpression{
					CommonMetaData: ast.CommonMetaData{
						Start: fp.getShiftedPosition(start),
						Key:   fp.keyGen.Next(),
					},
				}, true
			}
		}

		fp.expect(ast.RCurly)
	}

	fp.lexer.Commit(id)
	return ast.DefinitionBuiltinExpression{
		Of:        of,
		Satisfies: satisfies,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
	}, true
}

func nameBuiltinExpression[T ast.FormulationNodeKind](
	fp *formulationParser,
	name string,
	getValue func(ast.CommonMetaData) T,
) (T, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	if !fp.hasHas(ast.BackSlash, ast.BackSlash) {
		fp.lexer.RollBack(id)
		return getValue(ast.CommonMetaData{}), false
	}

	fp.expect(ast.BackSlash) // skip the \
	fp.expect(ast.BackSlash) // skip the \

	if !fp.has(ast.Name) || fp.lexer.Peek().Text != name {
		fp.lexer.RollBack(id)
		return getValue(ast.CommonMetaData{}), false
	}

	fp.expect(ast.Name) // skip the name

	fp.lexer.Commit(id)
	return getValue(ast.CommonMetaData{
		Start: fp.getShiftedPosition(start),
		Key:   fp.keyGen.Next(),
	}), true
}

func (fp *formulationParser) abstractBuiltinExpression() (ast.AbstractBuiltinExpression, bool) {
	val, ok := nameBuiltinExpression(fp, "abstract",
		func(cmd ast.CommonMetaData) *ast.AbstractBuiltinExpression {
			return &ast.AbstractBuiltinExpression{}
		})
	return *val, ok
}

func (fp *formulationParser) specificationBuiltinExpression() (ast.SpecificationBuiltinExpression, bool) {
	val, ok := nameBuiltinExpression(fp, "specification",
		func(cmd ast.CommonMetaData) *ast.SpecificationBuiltinExpression {
			return &ast.SpecificationBuiltinExpression{}
		})
	return *val, ok
}

func (fp *formulationParser) statementBuiltinExpression() (ast.StatementBuiltinExpression, bool) {
	val, ok := nameBuiltinExpression(fp, "statement",
		func(cmd ast.CommonMetaData) *ast.StatementBuiltinExpression {
			return &ast.StatementBuiltinExpression{}
		})
	return *val, ok
}

func (fp *formulationParser) expressionBuiltinExpression() (ast.ExpressionBuiltinExpression, bool) {
	val, ok := nameBuiltinExpression(fp, "expression",
		func(cmd ast.CommonMetaData) *ast.ExpressionBuiltinExpression {
			return &ast.ExpressionBuiltinExpression{}
		})
	return *val, ok
}

func (fp *formulationParser) typeBuiltinExpression() (ast.TypeBuiltinExpression, bool) {
	val, ok := nameBuiltinExpression(fp, "type",
		func(cmd ast.CommonMetaData) *ast.TypeBuiltinExpression {
			return &ast.TypeBuiltinExpression{}
		})
	return *val, ok
}

func (fp *formulationParser) functionCallExpressionTarget() (ast.ExpressionKind, bool) {
	if name, ok := fp.nameForm(); ok {
		return &name, ok
	}

	if tup, ok := fp.tupleExpression(); ok {
		return &tup, ok
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

	args := make([]ast.ExpressionKind, 0)
	fp.expect(ast.LParen)
	for fp.lexer.HasNext() {
		if fp.has(ast.RParen) {
			break
		}

		if len(args) > 0 {
			fp.expect(ast.Comma)
		}

		arg, ok := fp.expressionKind()
		if !ok {
			fp.error("Expected an expression")
			// move past the unexpected token
			fp.lexer.Next()
		} else {
			args = append(args, arg)
		}
	}
	fp.expect(ast.RParen)
	varArgData, _ := fp.varArgData()
	return ast.FunctionCallExpression{
		Target: target,
		Args:   args,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
		VarArg: varArgData,
	}, true
}

func (fp *formulationParser) singleItemTupleExpression() (ast.TupleExpression, bool) {
	if tup, ok := fp.singleItemParenOrInvisibleTupleExpression(true); ok {
		return tup, true
	}
	return fp.singleItemParenOrInvisibleTupleExpression(false)
}

func (fp *formulationParser) singleItemParenOrInvisibleTupleExpression(
	isInvisible bool,
) (ast.TupleExpression, bool) {
	id := fp.lexer.Snapshot()
	tup, ok := fp.parenOrInvisibleTupleExpression(isInvisible)
	if !ok || len(tup.Args) != 1 {
		fp.lexer.RollBack(id)
		return ast.TupleExpression{}, false
	}
	fp.lexer.Commit(id)
	return tup, true
}

func (fp *formulationParser) tupleExpression() (ast.TupleExpression, bool) {
	if tup, ok := fp.parenOrInvisibleTupleExpression(true); ok {
		return tup, true
	}
	return fp.parenOrInvisibleTupleExpression(false)
}

func (fp *formulationParser) parenOrInvisibleTupleExpression(
	isInvisible bool,
) (ast.TupleExpression, bool) {
	left := ast.LParen
	right := ast.RParen

	if isInvisible {
		left = ast.LParenDot
		right = ast.DotRParen
	}

	if !fp.has(left) {
		return ast.TupleExpression{}, false
	}

	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	fp.expect(left)
	args := make([]ast.ExpressionKind, 0)
	for fp.lexer.HasNext() {
		if fp.has(right) {
			break
		}

		if len(args) > 0 {
			fp.expect(ast.Comma)
		}

		arg, ok := fp.expressionKind(ast.DotRParen)
		if !ok {
			fp.lexer.RollBack(id)
			return ast.TupleExpression{}, false
		}
		args = append(args, arg)
	}
	fp.expect(right)

	if isInvisible && len(args) != 1 {
		fp.errorAt("A (. ... .) must contain one element", start)
	}

	fp.lexer.Commit(id)
	return ast.TupleExpression{
		Args:        args,
		IsInvisible: isInvisible,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
	}, true
}

func (fp *formulationParser) labeledGrouping() (ast.LabeledGrouping, bool) {
	if !fp.has(ast.LCurlyDot) {
		return ast.LabeledGrouping{}, false
	}

	start := fp.lexer.Position()
	fp.expect(ast.LCurlyDot)

	exp, ok := fp.expressionKind(ast.DotRCurly)
	if !ok {
		fp.errorAt("Expected an expression", start)
	}

	fp.expect(ast.DotRCurly)

	label := fp.labelText()
	if len(label) == 0 {
		fp.errorAt("Expected a label", start)
	}

	return ast.LabeledGrouping{
		Arg:   exp,
		Label: label,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
	}, true
}

func (fp *formulationParser) conditionalSetExpression() (ast.ConditionalSetExpression, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	symbols, ok := fp.squareParams()
	if !ok || symbols == nil {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetExpression{}, false
	}

	if !fp.has(ast.LCurly) {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetExpression{}, false
	}

	fp.expect(ast.LCurly)
	target, ok := fp.expressionKind(ast.Bar, ast.Colon)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetExpression{}, false
	}

	if !fp.has(ast.Bar) && !fp.has(ast.Colon) {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetExpression{}, false
	}

	specifications := make([]ast.ExpressionKind, 0)
	if fp.has(ast.Colon) {
		fp.expect(ast.Colon)
		for fp.lexer.HasNext() {
			spec, ok := fp.expressionKind(ast.Semicolon, ast.Bar)
			if ok {
				specifications = append(specifications, spec)
			} else {
				break
			}

			if fp.has(ast.Semicolon) {
				fp.expect(ast.Semicolon)
			} else {
				break
			}
		}
	}

	var condition mlglib.Optional[ast.ExpressionKind]
	if fp.has(ast.Bar) {
		fp.expect(ast.Bar)
		conditionExp, ok := fp.expressionKind()
		if !ok {
			fp.lexer.RollBack(id)
			return ast.ConditionalSetExpression{}, false
		}
		condition = mlglib.Some(conditionExp)
	}

	if !fp.has(ast.RCurly) {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetExpression{}, false
	}

	fp.expect(ast.RCurly)
	fp.lexer.Commit(id)
	return ast.ConditionalSetExpression{
		Symbols:        *symbols,
		Target:         target,
		Specifications: specifications,
		Condition:      condition,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
	}, true
}

func toNameForm(tok ast.PseudoTokenNode) ast.NameForm {
	return ast.NameForm{
		Text:            tok.Text,
		IsStropped:      false,
		HasQuestionMark: false,
		VarArg: ast.VarArgData{
			IsVarArg: false,
		},
		CommonMetaData: tok.CommonMetaData,
	}
}

func (fp *formulationParser) chainName() (ast.NameForm, bool) {
	if name, ok := fp.nameForm(); ok {
		return name, ok
	}

	if tok, ok := fp.isKeyword(); ok {
		return toNameForm(tok), ok
	}

	if tok, ok := fp.asKeyword(); ok {
		return toNameForm(tok), ok
	}

	if tok, ok := fp.extendsKeyword(); ok {
		return toNameForm(tok), ok
	}

	return ast.NameForm{}, false
}

func (fp *formulationParser) chainExpressionPart() (ast.ExpressionKind, bool) {
	if fun, ok := fp.functionCallExpression(); ok {
		return &fun, ok
	}

	if name, ok := fp.chainName(); ok {
		return &name, ok
	}

	//if tuple, ok := fp.tupleExpression(); ok {
	//	return tuple, ok
	//}

	//if set, ok := fp.fixedSetExpression(); ok {
	//	return set, ok
	//}

	//if set, ok := fp.conditionalSetExpression(); ok {
	//	return set, ok
	//}

	// We want to allow expressions such as (x + y).inv
	id := fp.lexer.Snapshot()
	if tuple, ok := fp.singleItemTupleExpression(); ok {
		if len(tuple.Args) == 1 {
			fp.lexer.Commit(id)
			return &tuple, ok
		} else {
			fp.lexer.RollBack(id)
			fp.error("A tuple cannot be part of a chain expression")
			return nil, false
		}
	} else {
		fp.lexer.RollBack(id)
	}

	return nil, false
}

func (fp *formulationParser) chainExpression(
	allowTrailingOperator bool) (ast.ChainExpression, bool) {
	hasTrailingOperator := false
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	parts := make([]ast.ExpressionKind, 0)
	for fp.lexer.HasNext() {
		if allowTrailingOperator && len(parts) > 0 {
			opName, ok := fp.operatorAsNameForm()
			if ok {
				hasTrailingOperator = true
				parts = append(parts, &opName)
				break
			}
		}
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
		Parts:               parts,
		HasTrailingOperator: hasTrailingOperator,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
	}, true
}

func (fp *formulationParser) ordinalCallExpression() (ast.OrdinalCallExpression, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	name, ok := fp.nameForm()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.OrdinalCallExpression{}, false
	}

	_, ok = fp.token(ast.LCurly)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.OrdinalCallExpression{}, false
	}

	args := make([]ast.ExpressionKind, 0)
	for fp.lexer.HasNext() {
		if fp.has(ast.RCurly) {
			break
		}

		if len(args) > 0 {
			fp.expect(ast.Comma)
		}

		arg, ok := fp.expressionKind()
		if !ok {
			fp.lexer.RollBack(id)
			return ast.OrdinalCallExpression{}, false
		}
		args = append(args, arg)
	}

	if len(args) == 0 {
		fp.lexer.RollBack(id)
		return ast.OrdinalCallExpression{}, false
	}

	_, ok = fp.token(ast.RCurly)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.OrdinalCallExpression{}, false
	}

	fp.lexer.Commit(id)
	return ast.OrdinalCallExpression{
		Target: &name,
		Args:   args,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
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
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
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
	if tok, ok := fp.pseudoToken(ast.Operator); ok {
		return tok, true
	}

	// a slash can also be an operator but has its own lexer token type
	if tok, ok := fp.pseudoToken(ast.Slash); ok {
		return ast.PseudoTokenNode{
			Text:           tok.Text,
			Type:           ast.Operator,
			CommonMetaData: tok.CommonMetaData,
		}, true
	}

	return ast.PseudoTokenNode{}, false
}

func (fp *formulationParser) barRightDashArrow() (ast.PseudoTokenNode, bool) {
	return fp.pseudoToken(ast.BarRightDashArrow)
}

func (fp *formulationParser) colonEqualsToken() (ast.PseudoTokenNode, bool) {
	return fp.pseudoToken(ast.ColonEquals)
}

func (fp *formulationParser) colonEqualsColonToken() (ast.PseudoTokenNode, bool) {
	return fp.pseudoToken(ast.ColonEqualsColon)
}

func (fp *formulationParser) colonArrowToken() (ast.PseudoTokenNode, bool) {
	return fp.pseudoToken(ast.ColonArrow)
}

func (fp *formulationParser) colonDashArrowToken() (ast.PseudoTokenNode, bool) {
	return fp.pseudoToken(ast.ColonDashArrow)
}

func (fp *formulationParser) semicolonToken() (ast.PseudoTokenNode, bool) {
	return fp.pseudoToken(ast.Semicolon)
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

	if colonEqualsColon, ok := fp.colonEqualsColonToken(); ok {
		return colonEqualsColon, ok
	}

	if colonArrow, ok := fp.colonArrowToken(); ok {
		return colonArrow, ok
	}

	if colonDashArrow, ok := fp.colonDashArrowToken(); ok {
		return colonDashArrow, ok
	}

	if barRightDashArrow, ok := fp.barRightDashArrow(); ok {
		return barRightDashArrow, ok
	}

	if semicolon, ok := fp.semicolonToken(); ok {
		return semicolon, ok
	}

	return ast.PseudoTokenNode{}, false
}

func (fp *formulationParser) operatorKind() (ast.OperatorKind, bool) {
	if enclosed, ok := fp.enclosedNonCommandOperatorTarget(); ok {
		return &enclosed, ok
	}

	if nonEnclosed, ok := fp.nonEnclosedNonCommandOperatorTarget(); ok {
		return &nonEnclosed, ok
	}

	if cmd, ok := fp.infixCommandExpression(); ok {
		return &cmd, ok
	}

	return nil, false
}

func (fp *formulationParser) nonEnclosedNonCommandOperatorTarget() (
	ast.NonEnclosedNonCommandOperatorTarget, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	hasLeftColon := false
	if _, ok := fp.token(ast.Colon); ok {
		hasLeftColon = true
	}
	if tok, ok := fp.operatorToken(); ok {
		fp.lexer.Commit(id)
		hasRightColon := false
		if _, ok := fp.token(ast.Colon); ok {
			hasRightColon = true
		}
		return ast.NonEnclosedNonCommandOperatorTarget{
			Text:          tok.Text,
			HasLeftColon:  hasLeftColon,
			HasRightColon: hasRightColon,
			CommonMetaData: ast.CommonMetaData{
				Start: fp.getShiftedPosition(start),
				Key:   fp.keyGen.Next(),
			},
		}, true
	}
	fp.lexer.RollBack(id)
	return ast.NonEnclosedNonCommandOperatorTarget{}, false
}

func (fp *formulationParser) enclosedNonCommandOperatorTarget() (
	ast.EnclosedNonCommandOperatorTarget, bool) {
	if !fp.has(ast.LSquareDot) && !fp.hasHas(ast.Colon, ast.LSquareDot) {
		return ast.EnclosedNonCommandOperatorTarget{}, false
	}

	var hasLeftColon bool

	if fp.has(ast.LSquareDot) {
		hasLeftColon = false
		fp.expect(ast.LSquareDot)
	} else if fp.hasHas(ast.Colon, ast.LSquareDot) {
		hasLeftColon = true
		fp.expect(ast.Colon)
		fp.expect(ast.LSquareDot)
	}

	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()

	var target ast.ExpressionKind
	if opName, ok := fp.operatorAsNameForm(); ok {
		target = &opName
	}

	if target == nil {
		if chain, ok := fp.chainExpression(true); ok {
			target = &chain
		}
	}

	if target == nil {
		if name, ok := fp.nameForm(); ok {
			target = &name
		}
	}

	if target == nil {
		fp.lexer.RollBack(id)
		return ast.EnclosedNonCommandOperatorTarget{}, false
	}

	fp.expect(ast.DotRSquare)

	hasRightColon := false
	if _, ok := fp.token(ast.Colon); ok {
		hasRightColon = true
	}

	fp.lexer.Commit(id)
	return ast.EnclosedNonCommandOperatorTarget{
		Target:        target,
		HasLeftColon:  hasLeftColon,
		HasRightColon: hasRightColon,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
	}, true
}

func (fp *formulationParser) infixCommandExpression() (ast.InfixCommandExpression, bool) {
	if !fp.hasHas(ast.BackSlash, ast.Dot) {
		return ast.InfixCommandExpression{}, false
	}

	start := fp.lexer.Position()

	fp.expect(ast.BackSlash)
	fp.expect(ast.Dot)

	id := fp.lexer.Snapshot()
	cmd, ok := fp.commandExpressionContent(true, start)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.InfixCommandExpression{}, false
	}

	if !fp.hasHas(ast.Dot, ast.Slash) {
		fp.lexer.RollBack(id)
		return ast.InfixCommandExpression{}, false
	}

	fp.expect(ast.Dot)
	fp.expect(ast.Slash)

	fp.lexer.Commit(id)
	return ast.InfixCommandExpression{
		Names:               cmd.Names,
		CurlyArg:            cmd.CurlyArg,
		NamedArgs:           cmd.NamedArgs,
		ParenArgs:           cmd.ParenArgs,
		CommonMetaData:      cmd.CommonMetaData,
		FormulationMetaData: cmd.FormulationMetaData,
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
	var curlyArg *ast.CurlyArg
	if arg, ok := fp.curlyArg(); ok {
		curlyArg = &arg
	}
	fp.lexer.Commit(id)
	return ast.NamedArg{
		Name:     name,
		CurlyArg: curlyArg,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
	}, true
}

func (fp *formulationParser) commandExpression(allowOperator bool) (ast.CommandExpression, bool) {
	if !fp.has(ast.BackSlash) {
		return ast.CommandExpression{}, false
	}
	start := fp.lexer.Position()
	fp.expect(ast.BackSlash)
	return fp.commandExpressionContent(allowOperator, start)
}

func (fp *formulationParser) commandExpressionContent(allowOperator bool, start ast.Position) (
	ast.CommandExpression,
	bool,
) {
	id := fp.lexer.Snapshot()
	names := make([]ast.NameForm, 0)
	for fp.lexer.HasNext() {
		if name, ok := fp.chainName(); ok {
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
					CommonMetaData:  op.CommonMetaData,
				})
			} else {
				break
			}
		}
		// ./ is how infix commands are terminated
		if fp.has(ast.Dot) && !fp.hasHas(ast.Dot, ast.Slash) {
			fp.expect(ast.Dot)
		} else {
			break
		}
	}

	if len(names) == 0 {
		fp.lexer.RollBack(id)
		return ast.CommandExpression{}, false
	}

	var curlyArg *ast.CurlyArg
	if arg, ok := fp.curlyArg(); ok {
		curlyArg = &arg
	}

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
		Names:     names,
		CurlyArg:  curlyArg,
		NamedArgs: &namedArgs,
		ParenArgs: parenArgs,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
	}, true
}

/////////////////////////// forms ///////////////////////////////////////

func (fp *formulationParser) structuralColonEqualsColonFormItemKind() (
	ast.StructuralColonEqualsColonFormItemKind, bool,
) {
	if fun, ok := fp.expressionForm(); ok {
		return &fun, true
	}

	if fun, ok := fp.functionForm(); ok {
		return &fun, true
	}

	if tuple, ok := fp.tupleForm(); ok {
		return &tuple, true
	}

	return nil, false
}

func (fp *formulationParser) structuralColonEqualsColonForm() (
	ast.StructuralColonEqualsColonForm, bool,
) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	lhs, ok := fp.structuralColonEqualsColonFormItemKind()

	if !ok {
		fp.lexer.RollBack(id)
		return ast.StructuralColonEqualsColonForm{}, false
	}

	if !fp.has(ast.ColonEqualsColon) {
		fp.lexer.RollBack(id)
		return ast.StructuralColonEqualsColonForm{}, false
	}

	fp.expect(ast.ColonEqualsColon)

	rhs, ok := fp.structuralColonEqualsColonFormItemKind()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.StructuralColonEqualsColonForm{}, false
	}

	fp.lexer.Commit(id)
	return ast.StructuralColonEqualsColonForm{
		Lhs: lhs,
		Rhs: rhs,
		CommonMetaData: ast.CommonMetaData{
			Key:   fp.keyGen.Next(),
			Start: start,
		},
	}, true
}

func (fp *formulationParser) structuralFormKindWithoutColonEquals() (ast.StructuralFormKind, bool) {
	if form, ok := fp.structuralColonEqualsColonForm(); ok {
		return &form, ok
	}

	if op, ok := fp.infixOperatorForm(); ok {
		return &op, ok
	}

	if op, ok := fp.prefixOperatorForm(); ok {
		return &op, ok
	}

	if op, ok := fp.postfixOperatorForm(); ok {
		return &op, ok
	}

	if op, ok := fp.nameFunctionTupleOrSet(); ok {
		return op, ok
	}

	return nil, false
}

func (fp *formulationParser) structuralFormKindPossiblyWithColonEquals() (
	ast.StructuralFormKind, bool,
) {
	// The lint checker incorrectly reports `start` is not used even though it is.
	// nolint:typecheck
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	lhs, ok := fp.structuralFormKindWithoutColonEquals()
	if !ok {
		fp.lexer.RollBack(id)
		return nil, false
	}

	if !fp.has(ast.ColonEquals) {
		fp.lexer.Commit(id)
		return lhs, true
	}

	switch lhs.(type) {
	case *ast.NameForm:
		fp.expect(ast.ColonEquals)
		rhs, ok := fp.structuralFormKindWithoutColonEquals()
		if !ok {
			fp.lexer.RollBack(id)
			fp.error("Expected an item on the right-hand-side of :=")
			return nil, false
		}

		fp.lexer.Commit(id)
		return &ast.StructuralColonEqualsForm{
			Lhs: lhs,
			Rhs: rhs,
			CommonMetaData: ast.CommonMetaData{
				Start: fp.getShiftedPosition(start),
				Key:   fp.keyGen.Next(),
			},
		}, true
	case *ast.FunctionForm:
		fp.expect(ast.ColonEquals)
		rhs, ok := fp.structuralFormKindWithoutColonEquals()
		if !ok {
			fp.lexer.RollBack(id)
			fp.error("Expected an item on the right-hand-side of :=")
			return nil, false
		}

		fp.lexer.Commit(id)
		return &ast.StructuralColonEqualsForm{
			Lhs: lhs,
			Rhs: rhs,
			CommonMetaData: ast.CommonMetaData{
				Start: fp.getShiftedPosition(start),
				Key:   fp.keyGen.Next(),
			},
		}, true
	case *ast.StructuralColonEqualsColonForm:
		fp.expect(ast.ColonEquals)
		rhs, ok := fp.structuralFormKindWithoutColonEquals()
		if !ok {
			fp.lexer.RollBack(id)
			fp.error("Expected an item on the right-hand-side of :=")
			return nil, false
		}

		fp.lexer.Commit(id)
		return &ast.StructuralColonEqualsForm{
			Lhs: lhs,
			Rhs: rhs,
			CommonMetaData: ast.CommonMetaData{
				Start: fp.getShiftedPosition(start),
				Key:   fp.keyGen.Next(),
			},
		}, true
	default:
		fp.lexer.Commit(id)
		return lhs, true
	}
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

func (fp *formulationParser) squareParams() (*[]ast.StructuralFormKind, bool) {
	id := fp.lexer.Snapshot()
	_, ok := fp.token(ast.LSquare)
	if !ok {
		fp.lexer.RollBack(id)
		return nil, false
	}
	args := make([]ast.StructuralFormKind, 0)
	for fp.lexer.HasNext() {
		if fp.has(ast.RSquare) {
			break
		}

		if len(args) > 0 {
			fp.expect(ast.Comma)
		}

		arg, ok := fp.structuralFormKindWithoutColonEquals()
		if !ok {
			fp.lexer.RollBack(id)
			return nil, false
		}
		args = append(args, arg)
	}
	fp.expect(ast.RSquare)
	fp.lexer.Commit(id)
	return &args, true
}

func (fp *formulationParser) curlyParams() (*[]ast.StructuralFormKind, bool) {
	id := fp.lexer.Snapshot()
	_, ok := fp.token(ast.LCurly)
	if !ok {
		fp.lexer.RollBack(id)
		return nil, false
	}
	args := make([]ast.StructuralFormKind, 0)
	for fp.lexer.HasNext() {
		if fp.has(ast.RCurly) {
			break
		}

		if len(args) > 0 {
			fp.expect(ast.Comma)
		}

		arg, ok := fp.structuralFormKindPossiblyWithColonEquals()
		if !ok {
			fp.lexer.RollBack(id)
			return nil, false
		}
		args = append(args, arg)
	}
	fp.expect(ast.RCurly)
	fp.lexer.Commit(id)
	return &args, true
}

func (fp *formulationParser) nameFunctionTupleOrSet() (ast.StructuralFormKind, bool) {
	if fun, ok := fp.expressionForm(); ok {
		return &fun, true
	}

	if fun, ok := fp.functionForm(); ok {
		return &fun, true
	}

	if name, ok := fp.operatorAsNameForm(); ok {
		return &name, true
	}

	if name, ok := fp.nameForm(); ok {
		return &name, true
	}

	if tuple, ok := fp.tupleForm(); ok {
		return &tuple, true
	}

	if conditionalSet, ok := fp.conditionalSetForm(); ok {
		return &conditionalSet, true
	}

	return nil, false
}

func (fp *formulationParser) infixOperatorForm() (ast.InfixOperatorForm, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	lhs, lhsOk := fp.nameFunctionTupleOrSet()
	op, opOk := fp.operatorAsNameForm()
	rhs, rhsOk := fp.nameFunctionTupleOrSet()
	if !lhsOk || !opOk || !rhsOk {
		fp.lexer.RollBack(id)
		return ast.InfixOperatorForm{}, false
	}
	fp.lexer.Commit(id)
	return ast.InfixOperatorForm{
		Operator: op,
		Lhs:      lhs,
		Rhs:      rhs,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
	}, true
}

func (fp *formulationParser) prefixOperatorForm() (ast.PrefixOperatorForm, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	op, opOk := fp.operatorAsNameForm()
	param, paramOk := fp.nameFunctionTupleOrSet()
	if !opOk || !paramOk {
		fp.lexer.RollBack(id)
		return ast.PrefixOperatorForm{}, false
	}
	fp.lexer.Commit(id)
	return ast.PrefixOperatorForm{
		Operator: op,
		Param:    param,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
	}, true
}

func (fp *formulationParser) postfixOperatorForm() (ast.PostfixOperatorForm, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	param, paramOk := fp.nameFunctionTupleOrSet()
	op, opOk := fp.operatorAsNameForm()
	if !paramOk || !opOk {
		fp.lexer.RollBack(id)
		return ast.PostfixOperatorForm{}, false
	}
	fp.lexer.Commit(id)
	return ast.PostfixOperatorForm{
		Operator: op,
		Param:    param,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
	}, true
}

func (fp *formulationParser) operatorAsNameForm() (ast.NameForm, bool) {
	if !fp.has(ast.Operator) {
		return ast.NameForm{}, false
	}
	next := fp.next()
	return ast.NameForm{
		Text:            next.Text,
		IsStropped:      false,
		HasQuestionMark: false,
		VarArg: ast.VarArgData{
			IsVarArg: false,
		},
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(next.Position),
			Key:   fp.keyGen.Next(),
		},
	}, true
}

func (fp *formulationParser) simpleNameForm() (ast.NameForm, bool) {
	start := fp.lexer.Position()
	if !fp.has(ast.Name) {
		return ast.NameForm{}, false
	}

	name := fp.next().Text
	return ast.NameForm{
		Text:            name,
		IsStropped:      false,
		HasQuestionMark: false,
		VarArg: ast.VarArgData{
			IsVarArg: false,
		},
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
	}, true
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

	hasQuestionMark := false
	varArgData, _ := fp.varArgData()

	if fp.has(ast.QuestionMark) {
		fp.next() // skip the ?
		hasQuestionMark = true
	}

	return ast.NameForm{
		Text:            name,
		IsStropped:      isStropped,
		HasQuestionMark: hasQuestionMark,
		VarArg:          varArgData,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
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

	params := make([]ast.StructuralFormKind, 0)
	fp.expect(ast.LParen)
	for fp.lexer.HasNext() {
		if fp.has(ast.RParen) {
			break
		}

		if len(params) > 0 {
			fp.expect(ast.Comma)
		}

		param, ok := fp.structuralFormKindWithoutColonEquals()
		if !ok {
			fp.error("Expected a structural form type")
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
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
	}, true
}

func (fp *formulationParser) expressionForm() (ast.ExpressionForm, bool) {
	start := fp.lexer.Position()
	if !fp.hasHas(ast.Name, ast.LSquare) {
		return ast.ExpressionForm{}, false
	}

	target, ok := fp.nameForm()
	if !ok {
		return ast.ExpressionForm{}, false
	}

	params := make([]ast.StructuralFormKind, 0)
	fp.expect(ast.LSquare)
	for fp.lexer.HasNext() {
		if fp.has(ast.RSquare) {
			break
		}

		if len(params) > 0 {
			fp.expect(ast.Comma)
		}

		param, ok := fp.structuralFormKindWithoutColonEquals()
		if !ok {
			fp.error("Expected a structural form type")
			// move past the unexpected token
			fp.lexer.Next()
		} else {
			params = append(params, param)
		}
	}
	fp.expect(ast.RSquare)

	varArgData, _ := fp.varArgData()
	return ast.ExpressionForm{
		Target: target,
		Params: params,
		VarArg: varArgData,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
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
	params := make([]ast.StructuralFormKind, 0)
	for fp.lexer.HasNext() {
		if fp.has(ast.RParen) {
			break
		}

		if len(params) > 0 {
			_, commaOk := fp.token(ast.Comma)
			if !commaOk {
				fp.lexer.RollBack(id)
				return ast.TupleForm{}, false
			}
		}

		param, ok := fp.structuralFormKindPossiblyWithColonEquals()
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
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
	}, true
}

func (fp *formulationParser) conditionalSetForm() (ast.ConditionalSetForm, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()

	symbols, ok := fp.squareParams()
	if !ok || symbols == nil {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetForm{}, false
	}

	if _, ok := fp.token(ast.LCurly); !ok {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetForm{}, false
	}

	target, ok := fp.structuralFormKindPossiblyWithColonEquals()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetForm{}, false
	}

	if !fp.has(ast.Bar) && !fp.has(ast.Colon) {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetForm{}, false
	}

	var specification *ast.FunctionForm
	if fp.has(ast.Colon) {
		colon, _ := fp.expect(ast.Colon)
		spec, ok := fp.functionForm()
		if !ok {
			fp.lexer.RollBack(id)
			fp.errorAt("Expected a function form to follow a :", colon.Position)
			return ast.ConditionalSetForm{}, false
		}
		specification = &spec
	}

	var condition *ast.FunctionForm
	if fp.has(ast.Bar) {
		bar, _ := fp.expect(ast.Bar)
		cond, ok := fp.functionForm()
		if !ok {
			fp.lexer.RollBack(id)
			fp.errorAt("Expected a function form to follow a |", bar.Position)
			return ast.ConditionalSetForm{}, false
		}
		condition = &cond
	}

	fp.expect(ast.RCurly)
	varArg, _ := fp.varArgData()
	fp.lexer.Commit(id)
	return ast.ConditionalSetForm{
		Symbols:       *symbols,
		Target:        target,
		Specification: specification,
		Condition:     condition,
		VarArg:        varArg,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
	}, true
}

////////////////////////////////////////// id forms ////////////////////////////////////////////////

func (fp *formulationParser) idKind() (ast.IdKind, bool) {
	if op, ok := fp.infixCommandOperatorId(); ok {
		return &op, ok
	} else if op, ok := fp.infixOperatorId(); ok {
		return &op, ok
	} else if op, ok := fp.prefixOperatorId(); ok {
		return &op, ok
	} else if op, ok := fp.postfixOperatorId(); ok {
		return &op, ok
	} else if cmd, ok := fp.commandId(false); ok {
		return &cmd, ok
	} else {
		return nil, false
	}
}

func (fp *formulationParser) infixCommandOperatorId() (ast.InfixCommandOperatorId, bool) {
	id := fp.lexer.Snapshot()

	lhs, ok := fp.structuralFormKindPossiblyWithColonEquals()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.InfixCommandOperatorId{}, false
	}

	op, ok := fp.infixCommandId()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.InfixCommandOperatorId{}, false
	}

	rhs, ok := fp.structuralFormKindPossiblyWithColonEquals()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.InfixCommandOperatorId{}, false
	}

	fp.lexer.Commit(id)
	return ast.InfixCommandOperatorId{
		Lhs:            lhs,
		Operator:       op,
		Rhs:            rhs,
		CommonMetaData: op.CommonMetaData,
	}, true
}

func (fp *formulationParser) infixOperatorId() (ast.InfixOperatorId, bool) {
	id := fp.lexer.Snapshot()

	lhs, ok := fp.structuralFormKindPossiblyWithColonEquals()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.InfixOperatorId{}, false
	}

	op, ok := fp.nonEnclosedNonCommandOperatorTarget()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.InfixOperatorId{}, false
	}

	rhs, ok := fp.structuralFormKindPossiblyWithColonEquals()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.InfixOperatorId{}, false
	}

	fp.lexer.Commit(id)
	return ast.InfixOperatorId{
		Lhs:            lhs,
		Operator:       op,
		Rhs:            rhs,
		CommonMetaData: op.CommonMetaData,
	}, true
}

func (fp *formulationParser) postfixOperatorId() (ast.PostfixOperatorId, bool) {
	id := fp.lexer.Snapshot()

	param, ok := fp.structuralFormKindPossiblyWithColonEquals()
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
		Operator:       op,
		Param:          param,
		CommonMetaData: op.CommonMetaData,
	}, true
}

func (fp *formulationParser) prefixOperatorId() (ast.PrefixOperatorId, bool) {
	id := fp.lexer.Snapshot()

	op, ok := fp.nonEnclosedNonCommandOperatorTarget()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.PrefixOperatorId{}, false
	}

	param, ok := fp.structuralFormKindPossiblyWithColonEquals()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.PrefixOperatorId{}, false
	}

	fp.lexer.Commit(id)
	return ast.PrefixOperatorId{
		Operator:       op,
		Param:          param,
		CommonMetaData: op.CommonMetaData,
	}, true
}

func (fp *formulationParser) conditionalSetIdForm() (ast.ConditionalSetIdForm, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	symbols, ok := fp.squareParams()
	if !ok || symbols == nil {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetIdForm{}, false
	}

	if !fp.has(ast.LCurly) {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetIdForm{}, false
	}

	fp.expect(ast.LCurly)
	target, ok := fp.structuralFormKindPossiblyWithColonEquals()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetIdForm{}, false
	}

	if !fp.has(ast.Bar) && !fp.has(ast.Colon) {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetIdForm{}, false
	}

	var specification *ast.FunctionForm
	if fp.has(ast.Colon) {
		colon, _ := fp.expect(ast.Colon)
		spec, ok := fp.functionForm()
		if !ok {
			fp.lexer.RollBack(id)
			fp.errorAt("Expected a function form to follow a :", colon.Position)
			return ast.ConditionalSetIdForm{}, false
		}
		specification = &spec
	}

	var condition *ast.FunctionForm
	if fp.has(ast.Bar) {
		fp.expect(ast.Bar)
		cond, ok := fp.functionForm()
		if !ok {
			fp.lexer.RollBack(id)
			return ast.ConditionalSetIdForm{}, false
		}
		condition = &cond
	}

	if !fp.has(ast.RCurly) {
		fp.lexer.RollBack(id)
		return ast.ConditionalSetIdForm{}, false
	}

	fp.expect(ast.RCurly)
	fp.lexer.Commit(id)
	return ast.ConditionalSetIdForm{
		Symbols:       *symbols,
		Target:        target,
		Specification: specification,
		Condition:     condition,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
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
	name, ok := fp.simpleNameForm()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.NamedParam{}, false
	}
	var curlyParam *ast.CurlyParam
	if param, ok := fp.curlyParam(); ok {
		curlyParam = &param
	}
	fp.lexer.Commit(id)
	return ast.NamedParam{
		Name:       name,
		CurlyParam: curlyParam,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
	}, true
}

func (fp *formulationParser) curlyParam() (ast.CurlyParam, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	var curlyParams *[]ast.StructuralFormKind
	if condSet, ok := fp.conditionalSetIdForm(); ok {
		tmpCurly := make([]ast.StructuralFormKind, 0)
		tmpCurly = append(tmpCurly, &condSet)
		curlyParams = &tmpCurly
	}
	if curlyParams == nil {
		var squareArgs *[]ast.StructuralFormKind
		squarePosition := fp.lexer.Position()
		if square, squareOk := fp.squareParams(); squareOk {
			squareArgs = square
		}
		realCurlyParams, curlyOk := fp.curlyParams()
		if !curlyOk {
			fp.lexer.RollBack(id)
			return ast.CurlyParam{}, false
		}
		if squareArgs != nil {
			if realCurlyParams == nil || len(*realCurlyParams) != 1 {
				fp.errorAt("If square args are used exactly one argument must be specified", squarePosition)
			} else {
				tmpCurlyParams := make([]ast.StructuralFormKind, 0)
				first := (*realCurlyParams)[0]
				tmpCurlyParams = append(tmpCurlyParams, &ast.FunctionLiteralForm{
					Lhs: ast.TupleForm{
						Params: *squareArgs,
						CommonMetaData: ast.CommonMetaData{
							Start: squarePosition,
							Key:   fp.keyGen.Next(),
						},
					},
					Rhs: first,
					CommonMetaData: ast.CommonMetaData{
						Start: first.Start(),
						Key:   fp.keyGen.Next(),
					},
				})
				curlyParams = &tmpCurlyParams
			}
		} else {
			curlyParams = realCurlyParams
		}
	}
	fp.lexer.Commit(id)
	return ast.CurlyParam{
		CurlyParams: curlyParams,
		CommonMetaData: ast.CommonMetaData{
			Key:   fp.keyGen.Next(),
			Start: start,
		},
	}, true
}

func (fp *formulationParser) curlyArg() (ast.CurlyArg, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()
	var curlyArgs *[]ast.ExpressionKind
	if condSet, ok := fp.conditionalSetExpression(); ok {
		tmpCurly := make([]ast.ExpressionKind, 0)
		tmpCurly = append(tmpCurly, &condSet)
		curlyArgs = &tmpCurly
	}
	if curlyArgs == nil {
		var squareArgs *[]ast.StructuralFormKind
		squarePosition := fp.lexer.Position()
		if square, squareOk := fp.squareParams(); squareOk {
			squareArgs = square
		}
		realCurlyArgs, curlyOk := fp.curlyArgs()
		if !curlyOk {
			fp.lexer.RollBack(id)
			return ast.CurlyArg{}, false
		}
		if squareArgs != nil {
			if realCurlyArgs == nil || len(*realCurlyArgs) != 1 {
				fp.errorAt("If square args are used exactly one argument must be specified", squarePosition)
			} else {
				tmpCurlyArgs := make([]ast.ExpressionKind, 0)
				first := (*realCurlyArgs)[0]
				tmpCurlyArgs = append(tmpCurlyArgs, &ast.FunctionLiteralExpression{
					Lhs: ast.TupleForm{
						Params: *squareArgs,
						CommonMetaData: ast.CommonMetaData{
							Start: squarePosition,
							Key:   fp.keyGen.Next(),
						},
					},
					Rhs: first,
					CommonMetaData: ast.CommonMetaData{
						Start: first.Start(),
						Key:   fp.keyGen.Next(),
					},
				})
				curlyArgs = &tmpCurlyArgs
			}
		} else {
			curlyArgs = realCurlyArgs
		}
	}
	fp.lexer.Commit(id)
	return ast.CurlyArg{
		CurlyArgs: curlyArgs,
		CommonMetaData: ast.CommonMetaData{
			Key:   fp.keyGen.Next(),
			Start: start,
		},
	}, true
}

func (fp *formulationParser) infixCommandId() (ast.InfixCommandId, bool) {
	if !fp.hasHas(ast.BackSlash, ast.Dot) {
		return ast.InfixCommandId{}, false
	}

	start := fp.lexer.Position()

	fp.expect(ast.BackSlash)
	fp.expect(ast.Dot)

	id := fp.lexer.Snapshot()
	cmd, ok := fp.commandIdContent(true, start)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.InfixCommandId{}, false
	}

	if !fp.hasHas(ast.Dot, ast.Slash) {
		fp.lexer.RollBack(id)
		return ast.InfixCommandId{}, false
	}

	fp.expect(ast.Dot)
	fp.expect(ast.Slash)

	fp.lexer.Commit(id)

	return ast.InfixCommandId{
		Names:          cmd.Names,
		CurlyParam:     cmd.CurlyParam,
		NamedParams:    cmd.NamedParams,
		ParenParams:    cmd.ParenParams,
		CommonMetaData: cmd.CommonMetaData,
	}, true
}

func (fp *formulationParser) commandId(allowOperator bool) (ast.CommandId, bool) {
	start := fp.lexer.Position()
	if !fp.has(ast.BackSlash) {
		return ast.CommandId{}, false
	}
	fp.expect(ast.BackSlash)
	return fp.commandIdContent(allowOperator, start)
}

func (fp *formulationParser) commandIdContent(allowOperator bool, start ast.Position) (
	ast.CommandId,
	bool,
) {
	id := fp.lexer.Snapshot()
	names := make([]ast.NameForm, 0)
	for fp.lexer.HasNext() {
		if name, ok := fp.chainName(); ok {
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
					CommonMetaData:  op.CommonMetaData,
				})
			} else {
				break
			}
		}
		// ./ is how an infix command is terminated
		if fp.has(ast.Dot) && !fp.hasHas(ast.Dot, ast.Slash) {
			fp.expect(ast.Dot)
		} else {
			break
		}
	}

	if len(names) == 0 {
		fp.lexer.RollBack(id)
		return ast.CommandId{}, false
	}

	var curlyParam *ast.CurlyParam
	if param, ok := fp.curlyParam(); ok {
		curlyParam = &param
	}

	namedParams := make([]ast.NamedParam, 0)
	for fp.lexer.HasNext() {
		if namedParam, ok := fp.namedParam(); ok {
			namedParams = append(namedParams, namedParam)
		} else {
			break
		}
	}

	var parenParams *[]ast.NameForm
	if form, ok := fp.nameParams(); ok {
		parenParams = &form
	}

	fp.lexer.Commit(id)
	return ast.CommandId{
		Names:       names,
		CurlyParam:  curlyParam,
		NamedParams: &namedParams,
		ParenParams: parenParams,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
	}, true
}

////////////////////////////////////////// type ////////////////////////////////////////////////////

func (fp *formulationParser) typeKind(allowOperator bool) (ast.TypeFormKind, bool) {
	if item, ok := fp.infixCommandType(); ok {
		return &item, true
	}

	if item, ok := fp.commandType(allowOperator); ok {
		return &item, true
	}

	return nil, false
}

func (fp *formulationParser) commandType(allowOperator bool) (ast.CommandTypeForm, bool) {
	start := fp.lexer.Position()
	if !fp.hasHas(ast.BackSlash, ast.Colon) {
		return ast.CommandTypeForm{}, false
	}

	fp.expect(ast.BackSlash)
	fp.expect(ast.Colon)

	return fp.commandTypeContent(allowOperator, start)
}

func (fp *formulationParser) commandTypeContent(
	allowOperator bool,
	start ast.Position,
) (ast.CommandTypeForm, bool) {
	id := fp.lexer.Snapshot()
	names := make([]ast.NameForm, 0)
	for fp.lexer.HasNext() {
		if name, ok := fp.chainName(); ok {
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
					CommonMetaData:  op.CommonMetaData,
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
		return ast.CommandTypeForm{}, false
	}

	var curlyTypeParam *ast.CurlyTypeParam
	if param, ok := fp.curlyTypeParam(); ok {
		curlyTypeParam = &param
	}

	namedTypeParams := make([]ast.NamedTypeParam, 0)
	for fp.lexer.HasNext() {
		if namedTypeParam, ok := fp.namedTypeParam(); ok {
			namedTypeParams = append(namedTypeParams, namedTypeParam)
		} else {
			break
		}
	}

	var parenArgs *[]ast.ExpressionKind
	if form, ok := fp.parenArgs(); ok {
		parenArgs = form
	}

	fp.lexer.Commit(id)
	return ast.CommandTypeForm{
		Names:           names,
		CurlyTypeParam:  curlyTypeParam,
		NamedTypeParams: &namedTypeParams,
		ParenTypeParams: parenArgs,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
	}, true
}

func (fp *formulationParser) infixCommandType() (ast.InfixCommandTypeForm, bool) {
	if !fp.hasHas(ast.BackSlash, ast.Colon) {
		return ast.InfixCommandTypeForm{}, false
	}

	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()

	fp.expect(ast.BackSlash)
	fp.expect(ast.Colon)

	cmd, ok := fp.commandTypeContent(true, start)
	if !ok {
		fp.lexer.RollBack(id)
		return ast.InfixCommandTypeForm{}, false
	}

	if !fp.has(ast.Colon) {
		fp.lexer.RollBack(id)
		return ast.InfixCommandTypeForm{}, false
	}

	fp.expect(ast.Colon)

	// also expect a slash and report an error if it is not found
	fp.expect(ast.Slash)

	fp.lexer.Commit(id)
	return ast.InfixCommandTypeForm{
		Names:           cmd.Names,
		CurlyTypeParam:  cmd.CurlyTypeParam,
		NamedTypeParams: cmd.NamedTypeParams,
		ParenTypeParams: cmd.ParenTypeParams,
		CommonMetaData:  cmd.CommonMetaData,
	}, true
}

func (fp *formulationParser) namedTypeParam() (ast.NamedTypeParam, bool) {
	start := fp.lexer.Position()
	if !fp.hasHas(ast.Colon, ast.Name) {
		return ast.NamedTypeParam{}, false
	}
	id := fp.lexer.Snapshot()
	fp.expect(ast.Colon)
	name, ok := fp.simpleNameForm()
	if !ok {
		fp.lexer.RollBack(id)
		return ast.NamedTypeParam{}, false
	}
	var curlyTypeParam *ast.CurlyTypeParam
	if param, ok := fp.curlyTypeParam(); ok {
		curlyTypeParam = &param
	}
	fp.lexer.Commit(id)
	return ast.NamedTypeParam{
		Name:           name,
		CurlyTypeParam: curlyTypeParam,
		CommonMetaData: ast.CommonMetaData{
			Start: fp.getShiftedPosition(start),
			Key:   fp.keyGen.Next(),
		},
	}, true
}

func (fp *formulationParser) curlyTypeParam() (ast.CurlyTypeParam, bool) {
	start := fp.lexer.Position()
	id := fp.lexer.Snapshot()

	curlyArgs, curlyOk := fp.curlyArgs()
	if !curlyOk {
		fp.lexer.RollBack(id)
		return ast.CurlyTypeParam{}, false
	}

	fp.lexer.Commit(id)
	return ast.CurlyTypeParam{
		CurlyTypeParams: curlyArgs,
		CommonMetaData: ast.CommonMetaData{
			Key:   fp.keyGen.Next(),
			Start: start,
		},
	}, true
}

///////////////////////////////////////// signature ////////////////////////////////////////////////

func (fp *formulationParser) signature() (ast.Signature, bool) {
	if !fp.hasHas(ast.BackSlash, ast.Colon) {
		return ast.Signature{}, false
	}

	start := fp.lexer.Position()
	typeKind, ok := fp.typeKind(true)

	var signature *ast.Signature
	if ok {
		switch t := typeKind.(type) {
		case *ast.InfixCommandTypeForm:
			if t.CurlyTypeParam != nil {
				fp.errorAt("A signature cannot contain a {}", start)
			}
			if t.ParenTypeParams != nil {
				fp.errorAt("A signature cannot contain a ()", start)
			}
			if t.NamedTypeParams != nil {
				hasNameWithCurlyArgs := false
				for _, n := range *t.NamedTypeParams {
					if n.CurlyTypeParam != nil {
						hasNameWithCurlyArgs = true
						break
					}
				}
				if hasNameWithCurlyArgs {
					fp.errorAt("A signature cannot contain a :name{}", start)
				}
			}

			mainNames := make([]string, 0)
			for _, n := range t.Names {
				mainNames = append(mainNames, n.Text)
			}

			namedGroupNames := make([]string, 0)
			if t.NamedTypeParams != nil {
				for _, n := range *t.NamedTypeParams {
					namedGroupNames = append(namedGroupNames, n.Name.Text)
				}
			}

			signature = &ast.Signature{
				MainNames:           mainNames,
				NamedGroupNames:     namedGroupNames,
				IsInfix:             true,
				InnerLabel:          nil, // will be added below
				CommonMetaData:      *typeKind.GetCommonMetaData(),
				FormulationMetaData: *typeKind.GetFormulationMetaData(),
			}
		case *ast.CommandTypeForm:
			if t.CurlyTypeParam != nil {
				fp.errorAt("A signature cannot contain a {}", start)
			}
			if t.ParenTypeParams != nil {
				fp.errorAt("A signature cannot contain a ()", start)
			}
			if t.NamedTypeParams != nil {
				hasNameWithCurlyArgs := false
				for _, n := range *t.NamedTypeParams {
					if n.CurlyTypeParam != nil {
						hasNameWithCurlyArgs = true
						break
					}
				}
				if hasNameWithCurlyArgs {
					fp.errorAt("A signature cannot contain a :name{}", start)
				}
			}

			mainNames := make([]string, 0)
			for _, n := range t.Names {
				mainNames = append(mainNames, n.Text)
			}

			namedGroupNames := make([]string, 0)
			if t.NamedTypeParams != nil {
				for _, n := range *t.NamedTypeParams {
					namedGroupNames = append(namedGroupNames, n.Name.Text)
				}
			}

			signature = &ast.Signature{
				MainNames:           mainNames,
				NamedGroupNames:     namedGroupNames,
				IsInfix:             false,
				InnerLabel:          nil, // will be added below
				CommonMetaData:      *typeKind.GetCommonMetaData(),
				FormulationMetaData: *typeKind.GetFormulationMetaData(),
			}
		case *ast.InfixOperatorCallExpression:
			// an infix operator call expression is not a valid signature
			signature = nil
		}
	}

	if signature == nil {
		fp.errorAt("Expected a signature", start)
		return ast.Signature{}, true
	}

	if fp.hasHas(ast.Colon, ast.Colon) {
		fp.expect(ast.Colon)
		fp.expect(ast.Colon)
		innerLabel := fp.labelText()
		signature.InnerLabel = &innerLabel
	}

	return *signature, true
}

func (fp *formulationParser) labelText() string {
	innerLabel := ""
	fp.expect(ast.LParen)
	for fp.lexer.HasNext() && !fp.has(ast.RParen) {
		position := fp.lexer.Position()
		name, ok := fp.nameForm()
		if !ok {
			fp.next() // absorb the next token
			fp.errorAt("Expected a name", position)
		}
		innerLabel += name.Text
		if !fp.has(ast.RParen) {
			fp.expect(ast.Dot)
			innerLabel += "."
		}
	}
	fp.expect(ast.RParen)
	return strings.Trim(innerLabel, " ")
}

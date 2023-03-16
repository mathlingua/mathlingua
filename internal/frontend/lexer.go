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

package frontend

import (
	"fmt"
	"mathlingua/internal/ast"
	"mathlingua/internal/mlglib"
)

type LexerType interface {
	HasNext() bool
	HasNextNext() bool
	Next() ast.Token
	Peek() ast.Token
	PeekPeek() ast.Token
	Position() ast.Position
	Snapshot() int
	Commit(id int)
	RollBack(id int)
}

func NewLexer(tokens []ast.Token) LexerType {
	return &lexer{
		index:     0,
		tokens:    tokens,
		snapshots: mlglib.NewStack[snapshot](),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type snapshot struct {
	id         int
	startIndex int
}

type lexer struct {
	index     int
	tokens    []ast.Token
	snapshots mlglib.StackType[snapshot]
}

func (lex *lexer) HasNext() bool {
	return lex.index < len(lex.tokens)
}

func (lex *lexer) HasNextNext() bool {
	return lex.index+1 < len(lex.tokens)
}

func (lex *lexer) Next() ast.Token {
	peek := lex.Peek()
	lex.index++
	return peek
}

func (lex *lexer) Peek() ast.Token {
	return lex.tokens[lex.index]
}

func (lex *lexer) PeekPeek() ast.Token {
	return lex.tokens[lex.index+1]
}

func (lex *lexer) Position() ast.Position {
	if lex.HasNext() {
		return lex.Peek().Position
	} else {
		return ast.Position{
			Offset: -1,
			Row:    -1,
			Column: -1,
		}
	}
}

func (lex *lexer) Snapshot() int {
	id := 0
	if !lex.snapshots.IsEmpty() {
		id = lex.snapshots.Peek().id + 1
	}
	lex.snapshots.Push(snapshot{
		id:         id,
		startIndex: lex.index,
	})
	return id
}

func (lex *lexer) Commit(id int) {
	if lex.snapshots.IsEmpty() || lex.snapshots.Peek().id != id {
		panic(fmt.Sprintf("Lexer requested committing with id %d but expected %d",
			id, lex.snapshots.Peek().id))
	}
	lex.snapshots.Pop()
}

func (lex *lexer) RollBack(id int) {
	if lex.snapshots.IsEmpty() || lex.snapshots.Peek().id != id {
		panic(fmt.Sprintf("Lexer requested rolling back with id %d but expected %d",
			id, lex.snapshots.Peek().id))
	}
	top := lex.snapshots.Pop()
	lex.index = top.startIndex
}

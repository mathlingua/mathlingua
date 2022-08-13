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

type Char struct {
	symbol rune
	offset int
	row    int
	column int
}

func GetChars(text string) []Char {
	chars := make([]Char, 0)
	curRow := 0
	curColumn := 0
	prevPos := 0
	for pos, c := range text {
		if c == '\n' {
			curRow++
			curColumn = 0
		} else {
			curColumn += pos - prevPos
		}
		prevPos = pos
		chars = append(chars, Char{
			symbol: c,
			offset: pos,
			row:    curRow,
			column: curColumn,
		})
	}
	return chars
}

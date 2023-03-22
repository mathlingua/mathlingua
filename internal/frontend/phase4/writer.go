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

type TextCodeWriter struct {
	text string
}

func NewTextCodeWriter() *TextCodeWriter {
	return &TextCodeWriter{
		text: "",
	}
}

func (w *TextCodeWriter) WriteIndent(indent int) {
	i := 0
	for i < indent {
		i++
		w.WriteSpace()
	}
}

func (w *TextCodeWriter) write(text string) {
	w.text += text
}

func (w *TextCodeWriter) WriteHeader(header string) {
	w.write(header)
}

func (w *TextCodeWriter) WriteId(id string) {
	w.write(id)
}

func (w *TextCodeWriter) WriteSpace() {
	w.WriteText(" ")
}

func (w *TextCodeWriter) WriteDotSpace() {
	w.write(". ")
}

func (w *TextCodeWriter) WriteNewline() {
	w.write("\n")
}

func (w *TextCodeWriter) WriteText(text string) {
	w.write(text)
}

func (w *TextCodeWriter) WriteFormulation(text string) {
	w.write(text)
}

func (w *TextCodeWriter) WriteDirect(text string) {
	w.write(text)
}

func (w *TextCodeWriter) WriteTextBlock(text string) {
	w.WriteText(text)
}

func (w *TextCodeWriter) String() string {
	return w.text
}

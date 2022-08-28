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

type CodeWriter interface {
	WriteIndent(indent int)
	WriteHeader(header string)
	WriteId(id string)
	WriteSpace()
	WriteDotSpace()
	WriteNewline()
	WriteText(text string)
	WriteFormulation(text string)
	WriteDirect(text string)
	WriteTextBlock(text string)
	String() string
}

func NewTextCodeWriter() CodeWriter {
	return &textCodeWriter{
		text: "",
	}
}

func NewHtmlCodeWriter() CodeWriter {
	return &htmlCodeWriter{
		text: "",
	}
}

///////////////////////////////////////////////////////////////

type textCodeWriter struct {
	text string
}

func (w *textCodeWriter) WriteIndent(indent int) {
	i := 0
	for i < indent {
		i++
		w.text += " "
	}
}

func (w *textCodeWriter) Write(text string) {
	w.text += text
}

func (w *textCodeWriter) WriteHeader(header string) {
	w.WriteText(header)
}

func (w *textCodeWriter) WriteId(id string) {
	w.WriteText(id)
}

func (w *textCodeWriter) WriteSpace() {
	w.WriteText(" ")
}

func (w *textCodeWriter) WriteDotSpace() {
	w.WriteText(". ")
}

func (w *textCodeWriter) WriteNewline() {
	w.WriteText("\n")
}

func (w *textCodeWriter) WriteText(text string) {
	w.WriteText(text)
}

func (w *textCodeWriter) WriteFormulation(text string) {
	w.WriteText(text)
}

func (w *textCodeWriter) WriteDirect(text string) {
	w.WriteText(text)
}

func (w *textCodeWriter) WriteTextBlock(text string) {
	w.WriteText(text)
}

func (w *textCodeWriter) String() string {
	return w.text
}

/////////////////////////////////////////////////////////

type htmlCodeWriter struct {
	text string
}

func (w *htmlCodeWriter) WriteIndent(indent int) {
	i := 0
	for i < indent {
		i++
		w.text += " "
	}
}

func (w *htmlCodeWriter) Write(text string) {
	w.text += text
}

func (w *htmlCodeWriter) WriteHeader(header string) {
	w.WriteText(header)
}

func (w *htmlCodeWriter) WriteId(id string) {
	w.WriteText(id)
}

func (w *htmlCodeWriter) WriteSpace() {
	w.WriteText(" ")
}

func (w *htmlCodeWriter) WriteDotSpace() {
	w.WriteText(". ")
}

func (w *htmlCodeWriter) WriteNewline() {
	w.WriteText("\n")
}

func (w *htmlCodeWriter) WriteText(text string) {
	w.WriteText(text)
}

func (w *htmlCodeWriter) WriteFormulation(text string) {
	w.WriteText(text)
}

func (w *htmlCodeWriter) WriteDirect(text string) {
	w.WriteText(text)
}

func (w *htmlCodeWriter) WriteTextBlock(text string) {
	w.WriteText(text)
}

func (w *htmlCodeWriter) String() string {
	return w.text
}

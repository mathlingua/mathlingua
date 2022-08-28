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
	"strings"
)

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
		w.WriteSpace()
	}
}

func (w *textCodeWriter) write(text string) {
	w.text += text
}

func (w *textCodeWriter) WriteHeader(header string) {
	w.write(header)
}

func (w *textCodeWriter) WriteId(id string) {
	w.write(id)
}

func (w *textCodeWriter) WriteSpace() {
	w.WriteText(" ")
}

func (w *textCodeWriter) WriteDotSpace() {
	w.write(". ")
}

func (w *textCodeWriter) WriteNewline() {
	w.write("\n")
}

func (w *textCodeWriter) WriteText(text string) {
	w.write(text)
}

func (w *textCodeWriter) WriteFormulation(text string) {
	w.write(text)
}

func (w *textCodeWriter) WriteDirect(text string) {
	w.write(text)
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
		w.WriteSpace()
	}
}

func (w *htmlCodeWriter) write(text string) {
	w.text += text
}

func (w *htmlCodeWriter) writeElement(tag string, classname string, content string) {
	w.write(fmt.Sprintf("<%s class='%s'>%s</%s>", tag, classname, content, tag))
}

func (w *htmlCodeWriter) WriteHeader(header string) {
	w.writeElement("span", "mathlingua-header", header)
}

func (w *htmlCodeWriter) WriteId(id string) {
	w.writeElement("span", "mathlingua-id", id)
}

func (w *htmlCodeWriter) WriteSpace() {
	w.write("&nbsp;")
}

func (w *htmlCodeWriter) WriteDotSpace() {
	w.write(".&nbsp;")
}

func (w *htmlCodeWriter) WriteNewline() {
	w.write("<br/>")
}

func (w *htmlCodeWriter) WriteText(text string) {
	w.writeElement("span", "mathlingua-text", text)
}

func (w *htmlCodeWriter) WriteFormulation(text string) {
	w.writeElement("span", "mathlingua-formulation", text)
}

func (w *htmlCodeWriter) WriteDirect(text string) {
	w.write(text)
}

func (w *htmlCodeWriter) WriteTextBlock(text string) {
	w.writeElement("span", "mathlingua-text-block", strings.ReplaceAll(text, "\n", "<br/>"))
}

func (w *htmlCodeWriter) String() string {
	return w.text
}

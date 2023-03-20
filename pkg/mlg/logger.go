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

package mlg

import (
	"fmt"

	"github.com/fatih/color"
)

type ILogger interface {
	Error(text string)
	Warning(text string)
	Failure(text string)
	Success(text string)
	Log(text string)
}

func NewLogger() *Logger {
	return &Logger{}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

var boldGreenColor = color.New(color.FgGreen, color.Bold)
var boldRedColor = color.New(color.FgRed, color.Bold)
var boldYellowColor = color.New(color.FgYellow, color.Bold)

type Logger struct{}

func boldRed(text string) string {
	return boldRedColor.Sprint(text)
}

func boldGreen(text string) string {
	return boldGreenColor.Sprint(text)
}

func boldYellow(text string) string {
	return boldYellowColor.Sprint(text)
}

func (lg *Logger) Error(text string) {
	fmt.Printf("%s %s\n", boldRed("ERROR:"), text)
}

func (lg *Logger) Warning(text string) {
	fmt.Printf("%s %s\n", boldYellow("WARNING:"), text)
}

func (lg *Logger) Failure(text string) {
	fmt.Printf("%s %s\n", boldRed("FAILURE:"), text)
}

func (lg *Logger) Success(text string) {
	fmt.Printf("%s %s\n", boldGreen("SUCCESS:"), text)
}

func (lg *Logger) Log(text string) {
	fmt.Println(text)
}

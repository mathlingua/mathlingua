/*
 * Copyright 2023 Dominic Kramer
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
	"bytes"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestDiagnosticDuplicateSignature(t *testing.T) {
	runTest(t, TestCase{
		Input: `
[\a]
Defines: x
------------------------------------------
Id: "123"


[\a]
Defines: y
------------------------------------------
Id: "456"`,
		ExpectedOutput: `ERROR: test.math (9, 1)
Duplicate defined signature \[a]

FAILURE: Processed 1 file and found 1 error and 0 warnings
`,
	})
}

func TestDiagnosticUseNonExistentSignature(t *testing.T) {
	runTest(t, TestCase{
		Input: `
Theorem:
given: x
then: 'x is \a'
------------------------------------------
Id: "123"`,
		ExpectedOutput: `ERROR: test.math (4, 13)
Unrecognized signature \[a]

ERROR: test.math (4, 13)
Signature \[a] does not have a Documented:called: or Documented:written: section

FAILURE: Processed 1 file and found 2 errors and 0 warnings
`,
	})
}

func TestDiagnosticUseSignatureWithoutCalledOrWritten(t *testing.T) {
	runTest(t, TestCase{
		Input: `
[\a]
Defines: a
------------------------------------------
Id: "123"


Theorem:
given: x
then: 'x is \a'
------------------------------------------
Id: "456"`,
		ExpectedOutput: `ERROR: test.math (10, 13)
Signature \[a] does not have a Documented:called: or Documented:written: section

FAILURE: Processed 1 file and found 1 error and 0 warnings
`,
	})
}

func TestNoDiagnosticUseSignatureWithCalled(t *testing.T) {
	runTest(t, TestCase{
		Input: `
[\a]
Defines: a
Documented:
. called: "a"
------------------------------------------
Id: "123"


Theorem:
given: x
then: 'x is \a'
------------------------------------------
Id: "456"`,
		ExpectedOutput: `SUCCESS: Processed 1 file and found 0 errors and 0 warnings
`,
	})
}

func TestNoDiagnosticUseSignatureWithWritten(t *testing.T) {
	runTest(t, TestCase{
		Input: `
[\a]
Defines: a
Documented:
. written: "a"
------------------------------------------
Id: "123"


Theorem:
given: x
then: 'x is \a'
------------------------------------------
Id: "456"`,
		ExpectedOutput: `SUCCESS: Processed 1 file and found 0 errors and 0 warnings
`,
	})
}

func TestDiagnosticUseSignatureIncorrectlyExpectOneCurlyArgProvideZero(t *testing.T) {
	runTest(t, TestCase{
		Input: `
[\a{x}]
Defines: a
Documented:
. written: "a"
------------------------------------------
Id: "123"


Theorem:
given: x
then: 'x is \a'
------------------------------------------
Id: "456"`,
		ExpectedOutput: `ERROR: test.math (12, 13)
Expected a {} argument but found none

FAILURE: Processed 1 file and found 1 error and 0 warnings
`,
	})
}

func TestDiagnosticUseSignatureIncorrectlyExpectOneCurlyArgProvideTwo(t *testing.T) {
	runTest(t, TestCase{
		Input: `
[\a{x}]
Defines: a
Documented:
. written: "a"
------------------------------------------
Id: "123"


Theorem:
given: x, y, z
then: 'x is \a{y, z}'
------------------------------------------
Id: "456"`,
		ExpectedOutput: `ERROR: test.math (12, 13)
Expected 1 values but found 2: Received: y, z

FAILURE: Processed 1 file and found 1 error and 0 warnings
`,
	})
}

func TestNoDiagnosticUseSignatureCorrectlyExpectOneCurlyArgProvideOne(t *testing.T) {
	runTest(t, TestCase{
		Input: `
[\a{x}]
Defines: a
Documented:
. written: "a"
------------------------------------------
Id: "123"


Theorem:
given: x, y
then: 'x is \a{y}'
------------------------------------------
Id: "456"`,
		ExpectedOutput: `SUCCESS: Processed 1 file and found 0 errors and 0 warnings
`,
	})
}

func TestNoDiagnosticUseSignatureCorrectlyExpectTwoCurlyArgsProvideTwo(t *testing.T) {
	runTest(t, TestCase{
		Input: `
[\a{x, y}]
Defines: a
Documented:
. written: "a"
------------------------------------------
Id: "123"


Theorem:
given: x, y, z
then: 'x is \a{y, z}'
------------------------------------------
Id: "456"`,
		ExpectedOutput: `SUCCESS: Processed 1 file and found 0 errors and 0 warnings
`,
	})
}

func TestDiagnosticUseSignatureIncorrectlyExpectOneParenArgProvideZero(t *testing.T) {
	runTest(t, TestCase{
		Input: `
[\a(x)]
Defines: a
Documented:
. written: "a"
------------------------------------------
Id: "123"


Theorem:
given: x
then: 'x is \a'
------------------------------------------
Id: "456"`,
		ExpectedOutput: `ERROR: test.math (12, 13)
Expected 1 values but found 0: Received: 

FAILURE: Processed 1 file and found 1 error and 0 warnings
`,
	})
}

func TestDiagnosticUseSignatureIncorrectlyExpectOneParenArgProvideTwo(t *testing.T) {
	runTest(t, TestCase{
		Input: `
[\a(x)]
Defines: a
Documented:
. written: "a"
------------------------------------------
Id: "123"


Theorem:
given: x, y, z
then: 'x is \a(y, z)'
------------------------------------------
Id: "456"`,
		ExpectedOutput: `ERROR: test.math (12, 13)
Expected 1 values but found 2: Received: y, z

FAILURE: Processed 1 file and found 1 error and 0 warnings
`,
	})
}

func TestNoDiagnosticUseSignatureCorrectlyExpectOneParenArgProvideOne(t *testing.T) {
	runTest(t, TestCase{
		Input: `
[\a(x)]
Defines: a
Documented:
. written: "a"
------------------------------------------
Id: "123"


Theorem:
given: x, y
then: 'x is \a(y)'
------------------------------------------
Id: "456"`,
		ExpectedOutput: `SUCCESS: Processed 1 file and found 0 errors and 0 warnings
`,
	})
}

func TestNoDiagnosticUseSignatureCorrectlyExpectTwoParenArgsProvideTwo(t *testing.T) {
	runTest(t, TestCase{
		Input: `
[\a(x, y)]
Defines: a
Documented:
. written: "a"
------------------------------------------
Id: "123"


Theorem:
given: x, y, z
then: 'x is \a(y, z)'
------------------------------------------
Id: "456"`,
		ExpectedOutput: `SUCCESS: Processed 1 file and found 0 errors and 0 warnings
`,
	})
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type TestCase struct {
	Input          string
	ExpectedOutput string
}

func runTest(t *testing.T, testCase TestCase) {
	dirName, err := os.MkdirTemp("", "mlg_diagnostic_test")
	if err != nil {
		t.FailNow()
	}

	err = os.Chdir(dirName)
	if err != nil {
		t.FailNow()
	}
	defer os.RemoveAll(dirName)

	err = os.WriteFile("test.math", []byte(testCase.Input), 0644)
	if err != nil {
		t.FailNow()
	}

	var buffer bytes.Buffer

	logger := NewLogger(&buffer)
	mlg := NewMlg(logger)
	mlg.Check([]string{"."}, false, false)

	assert.Equal(t, testCase.ExpectedOutput, buffer.String())
}

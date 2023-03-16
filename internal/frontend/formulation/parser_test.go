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
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/mlglib"
	"testing"

	"github.com/stretchr/testify/assert"
)

func parse(text string) (ast.FormulationNodeType, frontend.IDiagnosticTracker) {
	tracker := frontend.NewDiagnosticTracker()
	node, _ := ParseExpression(text, ast.Position{}, tracker, mlglib.NewKeyGenerator())
	return node, tracker
}

func runTest(t *testing.T, input string, expected string) {
	doc, tracker := parse(input)
	actual := mlglib.PrettyPrint(doc)

	assert.Equal(t, expected, actual)
	assert.Equal(t, 0, len(tracker.Diagnostics()))
}

func TestIdentifier(t *testing.T) {
	// skipped since this test doesn't take into account Key values
	t.Skip()
	input := `x`
	expected := `ast.NameForm{
  Text: "x",
  IsStropped: false,
  HasQuestionMark: false,
  VarArg: ast.VarArgData{
    IsVarArg: false,
    VarArgCount: nil,
    MetaData: ast.MetaData{
      Start: ast.Position{
        Offset: 0,
        Row: 0,
        Column: 0,
      },
    },
  },
  MetaData: ast.MetaData{
    Start: ast.Position{
      Offset: 0,
      Row: 0,
      Column: 0,
    },
  },
}`
	runTest(t, input, expected)
}

func TestMultiCharIdentifier(t *testing.T) {
	// skipped since this test doesn't take into account Key values
	t.Skip()
	input := `abc`
	expected := `ast.NameForm{
  Text: "abc",
  IsStropped: false,
  HasQuestionMark: false,
  VarArg: ast.VarArgData{
    IsVarArg: false,
    VarArgCount: nil,
    MetaData: ast.MetaData{
      Start: ast.Position{
        Offset: 0,
        Row: 0,
        Column: 0,
      },
    },
  },
  MetaData: ast.MetaData{
    Start: ast.Position{
      Offset: 0,
      Row: 0,
      Column: 0,
    },
  },
}`
	runTest(t, input, expected)
}

func TestIdentifierQuestion(t *testing.T) {
	// skipped since this test doesn't take into account Key values
	t.Skip()
	input := `x?`
	expected := `ast.NameForm{
  Text: "x",
  IsStropped: false,
  HasQuestionMark: true,
  VarArg: ast.VarArgData{
    IsVarArg: false,
    VarArgCount: nil,
    MetaData: ast.MetaData{
      Start: ast.Position{
        Offset: 0,
        Row: 0,
        Column: 0,
      },
    },
  },
  MetaData: ast.MetaData{
    Start: ast.Position{
      Offset: 0,
      Row: 0,
      Column: 0,
    },
  },
}`
	runTest(t, input, expected)
}

func TestStroppedIdentifier(t *testing.T) {
	// skipped since this test doesn't take into account Key values
	t.Skip()
	input := `"ab c"`
	expected := `ast.NameForm{
  Text: "ab c",
  IsStropped: true,
  HasQuestionMark: false,
  VarArg: ast.VarArgData{
    IsVarArg: false,
    VarArgCount: nil,
    MetaData: ast.MetaData{
      Start: ast.Position{
        Offset: 0,
        Row: 0,
        Column: 0,
      },
    },
  },
  MetaData: ast.MetaData{
    Start: ast.Position{
      Offset: 0,
      Row: 0,
      Column: 0,
    },
  },
}`
	runTest(t, input, expected)
}

func TestStroppedIdentifierQuestion(t *testing.T) {
	// skipped since this test doesn't take into account Key values
	t.Skip()
	input := `"ab c"?`
	expected := `ast.NameForm{
  Text: "ab c",
  IsStropped: true,
  HasQuestionMark: true,
  VarArg: ast.VarArgData{
    IsVarArg: false,
    VarArgCount: nil,
    MetaData: ast.MetaData{
      Start: ast.Position{
        Offset: 0,
        Row: 0,
        Column: 0,
      },
    },
  },
  MetaData: ast.MetaData{
    Start: ast.Position{
      Offset: 0,
      Row: 0,
      Column: 0,
    },
  },
}`
	runTest(t, input, expected)
}

func TestVarArgIdentifier(t *testing.T) {
	// skipped since this test doesn't take into account Key values
	t.Skip()
	input := `abc...`
	expected := `ast.NameForm{
  Text: "abc",
  IsStropped: false,
  HasQuestionMark: false,
  VarArg: ast.VarArgData{
    IsVarArg: true,
    VarArgCount: nil,
    MetaData: ast.MetaData{
      Start: ast.Position{
        Offset: 0,
        Row: 0,
        Column: 0,
      },
    },
  },
  MetaData: ast.MetaData{
    Start: ast.Position{
      Offset: 0,
      Row: 0,
      Column: 0,
    },
  },
}`
	runTest(t, input, expected)
}

func TestStroppedVarArgIdentifier(t *testing.T) {
	// skipped since this test doesn't take into account Key values
	t.Skip()
	input := `"ab c"?`
	expected := `ast.NameForm{
  Text: "ab c",
  IsStropped: true,
  HasQuestionMark: true,
  VarArg: ast.VarArgData{
    IsVarArg: false,
    VarArgCount: nil,
    MetaData: ast.MetaData{
      Start: ast.Position{
        Offset: 0,
        Row: 0,
        Column: 0,
      },
    },
  },
  MetaData: ast.MetaData{
    Start: ast.Position{
      Offset: 0,
      Row: 0,
      Column: 0,
    },
  },
}`
	runTest(t, input, expected)
}

func TestStroppedVarArgIdentifierQuestion(t *testing.T) {
	// skipped since this test doesn't take into account Key values
	t.Skip()
	input := `"ab c"...?`
	expected := `ast.NameForm{
  Text: "ab c",
  IsStropped: true,
  HasQuestionMark: true,
  VarArg: ast.VarArgData{
    IsVarArg: true,
    VarArgCount: nil,
    MetaData: ast.MetaData{
      Start: ast.Position{
        Offset: 0,
        Row: 0,
        Column: 0,
      },
    },
  },
  MetaData: ast.MetaData{
    Start: ast.Position{
      Offset: 0,
      Row: 0,
      Column: 0,
    },
  },
}`
	runTest(t, input, expected)
}

func TestChainExpressionWithNames(t *testing.T) {
	// skipped since this test doesn't take into account Key values
	t.Skip()
	input := `a.b.c`
	expected := `ast.ChainExpression{
  Parts: []ast.ExpressionType{
    ast.NameForm{
      Text: "a",
      IsStropped: false,
      HasQuestionMark: false,
      VarArg: ast.VarArgData{
        IsVarArg: false,
        VarArgCount: nil,
        MetaData: ast.MetaData{
          Start: ast.Position{
            Offset: 0,
            Row: 0,
            Column: 0,
          },
        },
      },
      MetaData: ast.MetaData{
        Start: ast.Position{
          Offset: 0,
          Row: 0,
          Column: 0,
        },
      },
    },
    ast.NameForm{
      Text: "b",
      IsStropped: false,
      HasQuestionMark: false,
      VarArg: ast.VarArgData{
        IsVarArg: false,
        VarArgCount: nil,
        MetaData: ast.MetaData{
          Start: ast.Position{
            Offset: 0,
            Row: 0,
            Column: 0,
          },
        },
      },
      MetaData: ast.MetaData{
        Start: ast.Position{
          Offset: 2,
          Row: 0,
          Column: 2,
        },
      },
    },
    ast.NameForm{
      Text: "c",
      IsStropped: false,
      HasQuestionMark: false,
      VarArg: ast.VarArgData{
        IsVarArg: false,
        VarArgCount: nil,
        MetaData: ast.MetaData{
          Start: ast.Position{
            Offset: 0,
            Row: 0,
            Column: 0,
          },
        },
      },
      MetaData: ast.MetaData{
        Start: ast.Position{
          Offset: 4,
          Row: 0,
          Column: 4,
        },
      },
    },
  },
  HasTrailingOperator: false,
  MetaData: ast.MetaData{
    Start: ast.Position{
      Offset: 0,
      Row: 0,
      Column: 0,
    },
  },
}`
	runTest(t, input, expected)
}

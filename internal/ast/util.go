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

package ast

import (
	"fmt"
	"mathlingua/internal/mlglib"
)

func NoOp(val MlgNodeKind) (string, bool) {
	return "", false
}

// The lint checker incorrectly reports that this function needs a return statement.
// nolint:typecheck
func Debug(node MlgNodeKind, fn func(node MlgNodeKind) (string, bool)) string {
	switch node := node.(type) {
	case StructuralNodeKind:
		return StructuralNodeToCode(node)
	case FormulationNodeKind:
		return FormulationNodeToCode(node, fn)
	default:
		panic(fmt.Sprintf("Cannot debug a node: %s", mlglib.PrettyPrint(node)))
	}
}

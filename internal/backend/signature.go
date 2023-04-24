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

package backend

import (
	"mathlingua/internal/ast"
)

func GetSignatureStringFromTopLevel(topLevel ast.TopLevelItemType) (string, bool) {
	switch tl := topLevel.(type) {
	case *ast.DefinesGroup:
		return GetSignatureStringFromId(tl.Id)
	case *ast.DescribesGroup:
		return GetSignatureStringFromId(tl.Id)
	case *ast.StatesGroup:
		return GetSignatureStringFromId(tl.Id)
	case *ast.AxiomGroup:
		if tl.Id == nil {
			return "", false
		}
		return GetSignatureStringFromId(*tl.Id)
	case *ast.ConjectureGroup:
		if tl.Id == nil {
			return "", false
		}
		return GetSignatureStringFromId(*tl.Id)
	case *ast.TheoremGroup:
		if tl.Id == nil {
			return "", false
		}
		return GetSignatureStringFromId(*tl.Id)
	default:
		return "", false
	}
}

func GetSignatureStringFromId(id ast.IdItem) (string, bool) {
	root := id.Root
	if root == nil {
		return "", false
	}
	switch n := root.(type) {
	case *ast.CommandId:
		// \a.b.c{x, y}:a{x}:b{y}
		return GetSignatureStringFromCommandId(*n), true
	case *ast.InfixCommandOperatorId:
		// x \in/ y
		return GetSignatureStringFromInfixCommandId(*&n.Operator), true
	default:
		return "", false
	}
}

func GetSignatureStringFromCommand(cmd ast.CommandExpression) string {
	names := make([]string, 0)
	namedGroups := make([]string, 0)

	for _, n := range cmd.Names {
		names = append(names, n.Text)
	}

	for _, ng := range *cmd.NamedArgs {
		namedGroups = append(namedGroups, ng.Name.Text)
	}

	return ast.Signature{
		MainNames:       names,
		NamedGroupNames: namedGroups,
		IsInfix:         false,
	}.ToCode(noOp)
}

func GetSignatureStringFromCommandId(cmd ast.CommandId) string {
	names := make([]string, 0)
	namedGroups := make([]string, 0)

	for _, n := range cmd.Names {
		names = append(names, n.Text)
	}

	for _, ng := range *cmd.NamedParams {
		namedGroups = append(namedGroups, ng.Name.Text)
	}

	return ast.Signature{
		MainNames:       names,
		NamedGroupNames: namedGroups,
		IsInfix:         false,
	}.ToCode(noOp)
}

func GetSignatureStringFromInfixCommandId(cmd ast.InfixCommandId) string {
	names := make([]string, 0)
	namedGroups := make([]string, 0)

	for _, n := range cmd.Names {
		names = append(names, n.Text)
	}

	for _, ng := range *cmd.NamedParams {
		namedGroups = append(namedGroups, ng.Name.Text)
	}

	return ast.Signature{
		MainNames:       names,
		NamedGroupNames: namedGroups,
		IsInfix:         true,
	}.ToCode(noOp)
}

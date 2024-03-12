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

func GetUsageFromTopLevel(topLevel TopLevelItemKind) (string, bool) {
	switch tl := topLevel.(type) {
	case *DefinesGroup:
		return GetUsageFromId(tl.Id)
	case *DescribesGroup:
		return GetUsageFromId(tl.Id)
	case *StatesGroup:
		return GetUsageFromId(tl.Id)
	case *AxiomGroup:
		if tl.Id == nil {
			return "", false
		}
		return GetUsageFromId(*tl.Id)
	case *ConjectureGroup:
		if tl.Id == nil {
			return "", false
		}
		return GetUsageFromId(*tl.Id)
	case *TheoremGroup:
		if tl.Id == nil {
			return "", false
		}
		return GetUsageFromId(*tl.Id)
	case *CorollaryGroup:
		if tl.Id == nil {
			return "", false
		}
		return GetUsageFromId(*tl.Id)
	case *LemmaGroup:
		if tl.Id == nil {
			return "", false
		}
		return GetUsageFromId(*tl.Id)
	default:
		return "", false
	}
}

func GetUsageFromId(id IdItem) (string, bool) {
	root := id.Root
	if root == nil {
		return "", false
	}
	switch n := root.(type) {
	case *CommandId:
		// \a.b.c{x, y}:a{x}:b{y}
		return n.ToCode(NoOp), true
	case *InfixCommandOperatorId:
		// x \in/ y
		return n.Operator.ToCode(NoOp), true
	default:
		return "", false
	}
}

func GetSignatureStringFromTopLevel(topLevel TopLevelItemKind) (string, bool) {
	switch tl := topLevel.(type) {
	case *DefinesGroup:
		return GetSignatureStringFromId(tl.Id)
	case *DescribesGroup:
		return GetSignatureStringFromId(tl.Id)
	case *StatesGroup:
		return GetSignatureStringFromId(tl.Id)
	case *AxiomGroup:
		if tl.Id == nil {
			return "", false
		}
		return GetSignatureStringFromId(*tl.Id)
	case *ConjectureGroup:
		if tl.Id == nil {
			return "", false
		}
		return GetSignatureStringFromId(*tl.Id)
	case *TheoremGroup:
		if tl.Id == nil {
			return "", false
		}
		return GetSignatureStringFromId(*tl.Id)
	default:
		return "", false
	}
}

func GetSignatureStringFromId(id IdItem) (string, bool) {
	root := id.Root
	if root == nil {
		return "", false
	}
	switch n := root.(type) {
	case *CommandId:
		// \a.b.c{x, y}:a{x}:b{y}
		return GetSignatureStringFromCommandId(*n), true
	case *InfixCommandOperatorId:
		// x \in/ y
		return GetSignatureStringFromInfixCommandId(n.Operator), true
	default:
		return "", false
	}
}

func GetSignatureStringFromCommand(cmd CommandExpression) string {
	names := make([]string, 0)
	namedGroups := make([]string, 0)

	for _, n := range cmd.Names {
		names = append(names, n.Text)
	}

	for _, ng := range *cmd.NamedArgs {
		namedGroups = append(namedGroups, ng.Name.Text)
	}

	sig := Signature{
		MainNames:       names,
		NamedGroupNames: namedGroups,
		IsInfix:         false,
	}
	return sig.ToCode(NoOp)
}

func GetSignatureStringFromInfixCommand(cmd InfixCommandExpression) string {
	names := make([]string, 0)
	namedGroups := make([]string, 0)

	for _, n := range cmd.Names {
		names = append(names, n.Text)
	}

	for _, ng := range *cmd.NamedArgs {
		namedGroups = append(namedGroups, ng.Name.Text)
	}

	sig := Signature{
		MainNames:       names,
		NamedGroupNames: namedGroups,
		IsInfix:         true,
	}
	return sig.ToCode(NoOp)
}

func GetSignatureStringFromCommandId(cmd CommandId) string {
	names := make([]string, 0)
	namedGroups := make([]string, 0)

	for _, n := range cmd.Names {
		names = append(names, n.Text)
	}

	for _, ng := range *cmd.NamedParams {
		namedGroups = append(namedGroups, ng.Name.Text)
	}

	sig := Signature{
		MainNames:       names,
		NamedGroupNames: namedGroups,
		IsInfix:         false,
	}
	return sig.ToCode(NoOp)
}

func GetSignatureStringFromInfixCommandId(cmd InfixCommandId) string {
	names := make([]string, 0)
	namedGroups := make([]string, 0)

	for _, n := range cmd.Names {
		names = append(names, n.Text)
	}

	for _, ng := range *cmd.NamedParams {
		namedGroups = append(namedGroups, ng.Name.Text)
	}

	sig := Signature{
		MainNames:       names,
		NamedGroupNames: namedGroups,
		IsInfix:         true,
	}
	return sig.ToCode(NoOp)
}

func GetUsedSignatureStrings(node MlgNodeKind) []string {
	result := make([]string, 0)
	getUsedSignaturesImpl(node, &result)
	return result
}

func getUsedSignaturesImpl(node MlgNodeKind, signatures *[]string) {
	if node == nil {
		return
	}

	switch n := node.(type) {
	case *CommandExpression:
		*signatures = append(*signatures, GetSignatureStringFromCommand(*n))
	case *InfixCommandExpression:
		*signatures = append(*signatures, GetSignatureStringFromInfixCommand(*n))
	}

	node.ForEach(func(subNode MlgNodeKind) {
		getUsedSignaturesImpl(subNode, signatures)
	})
}

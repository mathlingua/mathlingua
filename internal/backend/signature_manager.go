/*
 * Copyright 2024 Dominic Kramer
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
	"fmt"
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/structural/phase4"
)

type SignatureManager struct {
	nodeTracker *NodeTracker
	// the tracker used to record diagnostics
	diasnosticTracker *frontend.DiagnosticTracker
	// usages of defined commands
	usages []string
}

func NewSignatureManager(
	nodeTracker *NodeTracker,
	diasnosticTracker *frontend.DiagnosticTracker,
) *SignatureManager {
	sm := SignatureManager{
		nodeTracker:       nodeTracker,
		diasnosticTracker: diasnosticTracker,
		usages:            make([]string, 0),
	}
	sm.initializeUsages()
	return &sm
}

func (ur *SignatureManager) GetUsages() []string {
	return ur.usages
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (sm *SignatureManager) initializeUsages() {
	for _, doc := range sm.nodeTracker.astRoot.Documents {
		for _, item := range doc.Items {
			usage, ok := getUsageFromTopLevel(item)
			if ok {
				sm.usages = append(sm.usages, usage)
			}
		}
	}
	sm.updateUsedSignatures()
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (sm *SignatureManager) updateUsedSignatures() {
	for _, doc := range sm.nodeTracker.astRoot.Documents {
		for _, item := range doc.Items {
			if _, ok := item.(*ast.TextBlockItem); ok {
				continue
			}

			updateAstUsedSignatures(item)
			keyToUsedSignatures := make(map[int][]string, 0)
			recordUsedSignatureStrings(item, keyToUsedSignatures)

			id, ok := GetAstMetaId(item)
			if !ok {
				// fmt.Printf("Could not get the identifier for %s\n", mlglib.PrettyPrint(item))
				continue
			}

			phase4Item, ok := sm.nodeTracker.phase4Entries[id]
			if !ok {
				// fmt.Printf("Could not get the phase4 item for id %s", id)
				continue
			}

			sm.updatePhase4UsedSignatures(phase4Item, keyToUsedSignatures)
		}
	}
}

func updateAstUsedSignatures(node ast.MlgNodeKind) {
	if node == nil {
		return
	}

	if formulation, ok := node.(*ast.Formulation[ast.FormulationNodeKind]); ok {
		formulation.FormulationMetaData.UsedSignatureStrings = getUsedSignatureStrings(formulation)
	}
	node.ForEach(updateAstUsedSignatures)
}

func recordUsedSignatureStrings(node ast.MlgNodeKind, keyToUsedSignatures map[int][]string) {
	if node == nil {
		return
	}

	if formulation, ok := node.(*ast.Formulation[ast.FormulationNodeKind]); ok {
		key := formulation.GetCommonMetaData().Key
		usedSignatures := formulation.FormulationMetaData.UsedSignatureStrings
		keyToUsedSignatures[key] = usedSignatures
	}
	node.ForEach(func(subNode ast.MlgNodeKind) {
		recordUsedSignatureStrings(subNode, keyToUsedSignatures)
	})
}

func (sm *SignatureManager) updatePhase4UsedSignatures(
	node phase4.Node,
	keyToUsedSignatures map[int][]string,
) {
	if n, ok := node.(*phase4.FormulationArgumentData); ok {
		key := n.MetaData.Key
		signatures, ok := keyToUsedSignatures[key]
		if ok {
			n.FormulationMetaData.UsedSignatureStrings = signatures
		}
	}
	for i := 0; i < node.Size(); i += 1 {
		sm.updatePhase4UsedSignatures(node.ChildAt(i), keyToUsedSignatures)
	}
}

func (sm *SignatureManager) findUsedUnknownSignatures() {
	for path, doc := range sm.nodeTracker.astRoot.Documents {
		for _, item := range doc.Items {
			findUsedUnknownSignaturesImpl(item, path, sm)
		}
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func findUsedUnknownSignaturesImpl(node ast.MlgNodeKind, path ast.Path, w *SignatureManager) {
	if node == nil {
		return
	}

	if cmd, ok := node.(*ast.CommandExpression); ok {
		sig := GetSignatureStringFromCommand(*cmd)
		if _, ok := w.nodeTracker.signaturesToIds[sig]; !ok {
			w.diasnosticTracker.Append(frontend.Diagnostic{
				Type:     frontend.Error,
				Origin:   frontend.BackendOrigin,
				Message:  fmt.Sprintf("Unrecognized signature %s", sig),
				Path:     path,
				Position: node.GetCommonMetaData().Start,
			})
		}
	} else if cmd, ok := node.(*ast.InfixCommandExpression); ok {
		sig := GetSignatureStringFromInfixCommand(*cmd)
		if _, ok := w.nodeTracker.signaturesToIds[sig]; !ok {
			w.diasnosticTracker.Append(frontend.Diagnostic{
				Type:     frontend.Error,
				Origin:   frontend.BackendOrigin,
				Message:  fmt.Sprintf("Unrecognized signature %s", sig),
				Path:     path,
				Position: node.GetCommonMetaData().Start,
			})
		}
	}
	node.ForEach(func(subNode ast.MlgNodeKind) {
		switch n := subNode.(type) {
		case *ast.ExpressionColonArrowItem:
			findUsedUnknownSignaturesImpl(n.Rhs, path, w)
		default:
			findUsedUnknownSignaturesImpl(subNode, path, w)
		}
	})
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func getUsageFromTopLevel(topLevel ast.TopLevelItemKind) (string, bool) {
	switch tl := topLevel.(type) {
	case *ast.DefinesGroup:
		return getUsageFromId(tl.Id)
	case *ast.DescribesGroup:
		return getUsageFromId(tl.Id)
	case *ast.StatesGroup:
		return getUsageFromId(tl.Id)
	case *ast.AxiomGroup:
		if tl.Id == nil {
			return "", false
		}
		return getUsageFromId(*tl.Id)
	case *ast.ConjectureGroup:
		if tl.Id == nil {
			return "", false
		}
		return getUsageFromId(*tl.Id)
	case *ast.TheoremGroup:
		if tl.Id == nil {
			return "", false
		}
		return getUsageFromId(*tl.Id)
	case *ast.CorollaryGroup:
		if tl.Id == nil {
			return "", false
		}
		return getUsageFromId(*tl.Id)
	case *ast.LemmaGroup:
		if tl.Id == nil {
			return "", false
		}
		return getUsageFromId(*tl.Id)
	default:
		return "", false
	}
}

func getUsageFromId(id ast.IdItem) (string, bool) {
	root := id.Root
	if root == nil {
		return "", false
	}
	switch n := root.(type) {
	case *ast.CommandId:
		// \a.b.c{x, y}:a{x}:b{y}
		return n.ToCode(ast.NoOp), true
	case *ast.InfixCommandOperatorId:
		// x \in/ y
		return n.Operator.ToCode(ast.NoOp), true
	default:
		return "", false
	}
}

func GetSignatureStringFromTopLevel(topLevel ast.TopLevelItemKind) (string, bool) {
	switch tl := topLevel.(type) {
	case *ast.DefinesGroup:
		return getSignatureStringFromId(tl.Id)
	case *ast.DescribesGroup:
		return getSignatureStringFromId(tl.Id)
	case *ast.StatesGroup:
		return getSignatureStringFromId(tl.Id)
	case *ast.AxiomGroup:
		if tl.Id == nil {
			return "", false
		}
		return getSignatureStringFromId(*tl.Id)
	case *ast.ConjectureGroup:
		if tl.Id == nil {
			return "", false
		}
		return getSignatureStringFromId(*tl.Id)
	case *ast.TheoremGroup:
		if tl.Id == nil {
			return "", false
		}
		return getSignatureStringFromId(*tl.Id)
	default:
		return "", false
	}
}

func getSignatureStringFromId(id ast.IdItem) (string, bool) {
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
		return GetSignatureStringFromInfixCommandId(n.Operator), true
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

	sig := ast.Signature{
		MainNames:       names,
		NamedGroupNames: namedGroups,
		IsInfix:         false,
	}
	return sig.ToCode(ast.NoOp)
}

func GetSignatureStringFromInfixCommand(cmd ast.InfixCommandExpression) string {
	names := make([]string, 0)
	namedGroups := make([]string, 0)

	for _, n := range cmd.Names {
		names = append(names, n.Text)
	}

	for _, ng := range *cmd.NamedArgs {
		namedGroups = append(namedGroups, ng.Name.Text)
	}

	sig := ast.Signature{
		MainNames:       names,
		NamedGroupNames: namedGroups,
		IsInfix:         true,
	}
	return sig.ToCode(ast.NoOp)
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

	sig := ast.Signature{
		MainNames:       names,
		NamedGroupNames: namedGroups,
		IsInfix:         false,
	}
	return sig.ToCode(ast.NoOp)
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

	sig := ast.Signature{
		MainNames:       names,
		NamedGroupNames: namedGroups,
		IsInfix:         true,
	}
	return sig.ToCode(ast.NoOp)
}

func getUsedSignatureStrings(node ast.MlgNodeKind) []string {
	result := make([]string, 0)
	getUsedSignaturesImpl(node, &result)
	return result
}

func getUsedSignaturesImpl(node ast.MlgNodeKind, signatures *[]string) {
	if node == nil {
		return
	}

	switch n := node.(type) {
	case *ast.CommandExpression:
		*signatures = append(*signatures, GetSignatureStringFromCommand(*n))
	case *ast.InfixCommandExpression:
		*signatures = append(*signatures, GetSignatureStringFromInfixCommand(*n))
	}

	node.ForEach(func(subNode ast.MlgNodeKind) {
		getUsedSignaturesImpl(subNode, signatures)
	})
}

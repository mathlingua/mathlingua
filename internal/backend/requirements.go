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
)

func CheckRequirements(
	path ast.Path,
	node ast.MlgNodeKind,
	tracker *frontend.DiagnosticTracker,
) {
	if node == nil {
		return
	}

	switch n := node.(type) {
	case *ast.AxiomGroup:
		checkAxiomGroup(path, *n, tracker)
	case *ast.ConjectureGroup:
		checkConjectureGroup(path, *n, tracker)
	case *ast.CorollaryGroup:
		checkCorollaryGroup(path, *n, tracker)
	case *ast.TheoremGroup:
		checkTheoremGroup(path, *n, tracker)
	case *ast.LemmaGroup:
		checkLemmaGroup(path, *n, tracker)
	case *ast.ProofClaimGroup:
		checkProofClaimGroup(path, *n, tracker)
	case *ast.IsExpression:
		checkIsExpression(path, *n, tracker)
	case *ast.AlsoExpression:
		checkAlsoExpression(path, *n, tracker)
	}

	node.ForEach(func(subNode ast.MlgNodeKind) {
		CheckRequirements(path, subNode, tracker)
	})
}

func checkAxiomGroup(
	path ast.Path,
	node ast.AxiomGroup,
	tracker *frontend.DiagnosticTracker,
) {
	checkResultGivenSuchThatIf(
		path,
		node.GetCommonMetaData().Start,
		node.Given,
		node.SuchThat,
		node.If,
		tracker)
}

func checkConjectureGroup(
	path ast.Path,
	node ast.ConjectureGroup,
	tracker *frontend.DiagnosticTracker,
) {
	checkResultGivenSuchThatIf(
		path,
		node.GetCommonMetaData().Start,
		node.Given,
		node.SuchThat,
		node.If,
		tracker)
}

func checkCorollaryGroup(
	path ast.Path,
	node ast.CorollaryGroup,
	tracker *frontend.DiagnosticTracker,
) {
	checkResultGivenSuchThatIf(
		path,
		node.GetCommonMetaData().Start,
		node.Given,
		node.SuchThat,
		node.If,
		tracker)
}

func checkTheoremGroup(
	path ast.Path,
	node ast.TheoremGroup,
	tracker *frontend.DiagnosticTracker,
) {
	checkResultGivenSuchThatIf(
		path,
		node.GetCommonMetaData().Start,
		node.Given,
		node.SuchThat,
		node.If,
		tracker)
}

func checkLemmaGroup(
	path ast.Path,
	node ast.LemmaGroup,
	tracker *frontend.DiagnosticTracker,
) {
	checkResultGivenSuchThatIf(
		path,
		node.GetCommonMetaData().Start,
		node.Given,
		node.SuchThat,
		node.If,
		tracker)
}

func checkProofClaimGroup(
	path ast.Path,
	node ast.ProofClaimGroup,
	tracker *frontend.DiagnosticTracker,
) {
	checkResultGivenSuchThatIf(
		path,
		node.GetCommonMetaData().Start,
		node.Given,
		node.SuchThat,
		node.If,
		tracker)
}

func checkIsExpression(
	path ast.Path,
	node ast.IsExpression,
	tracker *frontend.DiagnosticTracker,
) {
	checkIsOrAlsoExpression(path, node.GetCommonMetaData().Start, "is", node.Rhs, tracker)
}

func checkAlsoExpression(
	path ast.Path,
	node ast.AlsoExpression,
	tracker *frontend.DiagnosticTracker,
) {
	checkIsOrAlsoExpression(path, node.GetCommonMetaData().Start, "also", node.Rhs, tracker)
}

func checkIsOrAlsoExpression(
	path ast.Path,
	position ast.Position,
	isOrAlsoName string,
	rhs []ast.KindKind,
	tracker *frontend.DiagnosticTracker,
) {
	for _, item := range rhs {
		_, isName := item.(*ast.NameForm)
		_, isType := item.(*ast.TypeMetaKind)
		_, isTypeOf := item.(*ast.TypeOfBuiltinExpression)
		_, isBoolean := item.(*ast.BooleanBuiltinExpression)
		_, isTrue := item.(*ast.TrueBuiltinExpression)
		_, isFalse := item.(*ast.FalseBuiltinExpression)
		_, isCommand := item.(*ast.CommandExpression)
		_, isFormulation := item.(*ast.FormulationMetaKind)
		if !isName && !isType && !isTypeOf && !isBoolean && !isTrue && !isFalse && !isCommand && !isFormulation {
			appendError(
				path,
				position,
				fmt.Sprintf("The right-hand-side of an '%s' statement ", isOrAlsoName)+
					"can only contain a name, command, \\\\type, \\\\type:of, "+
					"\\\\boolean, \\\\true, \\\\false, or \\\\formulation",
				tracker)
		}
	}
}

func checkResultGivenSuchThatIf(
	path ast.Path,
	position ast.Position,
	givenSection *ast.GivenSection,
	suchThatSection *ast.SuchThatSection,
	ifSection *ast.IfSection,
	tracker *frontend.DiagnosticTracker,
) {
	if suchThatSection != nil && ifSection != nil {
		appendError(
			path,
			position,
			"An if: section cannot be specified if a suchThat: section is specified",
			tracker)
	}

	if givenSection != nil && ifSection != nil {
		appendError(
			path,
			position,
			"An if: section cannot be specified if a given: section is specified",
			tracker)
	}
}

func appendError(
	path ast.Path,
	potition ast.Position,
	message string,
	tracker *frontend.DiagnosticTracker,
) {
	tracker.Append(frontend.Diagnostic{
		Type:     frontend.Error,
		Origin:   frontend.BackendOrigin,
		Message:  message,
		Position: potition,
		Path:     path,
	})
}

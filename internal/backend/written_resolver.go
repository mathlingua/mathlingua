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
	"mathlingua/internal/frontend/phase4"
	"regexp"
	"strings"
	"unicode"
)

type WrittenResolver struct {
	// the tracker used to record diagnostics
	diagnosticTracker *frontend.DiagnosticTracker
	nodeTracker       *NodeTracker
	// map ids to summaries of the Documented section
	documentedSummaries map[string]ast.DocumentedSummary
	// map ids to summaries of top-level item inputs
	inputSummaries map[string]ast.InputSummary
}

func NewWrittenResolver(
	nodeTracker *NodeTracker,
	diagnosticTracker *frontend.DiagnosticTracker,
) *WrittenResolver {
	resolver := WrittenResolver{
		diagnosticTracker:   diagnosticTracker,
		nodeTracker:         nodeTracker,
		documentedSummaries: make(map[string]ast.DocumentedSummary, 0),
		inputSummaries:      make(map[string]ast.InputSummary, 0),
	}
	resolver.initializeSummaries()
	return &resolver
}

func (w *WrittenResolver) initializeSummaries() {
	for _, doc := range w.nodeTracker.astRoot.Documents {
		for _, item := range doc.Items {
			id, idOk := GetAstMetaId(item)
			if !idOk {
				continue
			}

			docSummary, docSummaryOk := GetDocumentedSummary(item, w.diagnosticTracker)
			if docSummaryOk && docSummary != nil {
				w.documentedSummaries[id] = *docSummary
			}

			inputSummary, inputSummaryOk := GetInputSummary(item, w.diagnosticTracker)
			if inputSummaryOk && inputSummary != nil {
				w.inputSummaries[id] = *inputSummary
			}
		}
	}
}

func (w *WrittenResolver) GetRenderedNode(
	path ast.Path,
	phase4Node phase4.Node,
	astNode ast.MlgNodeKind,
) phase4.Node {
	// make a copy of the input node to return
	result := phase4Node
	keyToFormulationStr := make(map[int]string, 0)
	w.formulationLikeToString(path, astNode, keyToFormulationStr)
	inlineProcessForRendering(result)
	w.updateFormulationStrings(path, result, keyToFormulationStr)
	return result
}

func (w *WrittenResolver) formulationLikeToString(
	path ast.Path,
	node ast.MlgNodeKind,
	keyToFormulationStr map[int]string,
) {
	if node == nil {
		return
	}

	if formulation, ok := node.(*ast.Formulation[ast.FormulationNodeKind]); ok {
		key := formulation.GetCommonMetaData().Key
		newText := w.formulationToWritten(path, *formulation)
		keyToFormulationStr[key] = newText
	} else if spec, ok := node.(*ast.Spec); ok {
		key := spec.GetCommonMetaData().Key
		newText := w.specToWritten(path, *spec)
		keyToFormulationStr[key] = newText
	} else if alias, ok := node.(*ast.Alias); ok {
		key := alias.GetCommonMetaData().Key
		newText := w.aliasToWritten(path, *alias)
		keyToFormulationStr[key] = newText
	} else if target, ok := node.(*ast.Target); ok {
		key := target.GetCommonMetaData().Key
		newText := w.targetToWritten(path, *target)
		keyToFormulationStr[key] = newText
	} else {
		node.ForEach(func(subNode ast.MlgNodeKind) {
			w.formulationLikeToString(path, subNode, keyToFormulationStr)
		})
	}
}

func (w *WrittenResolver) formulationToWritten(
	path ast.Path,
	node ast.Formulation[ast.FormulationNodeKind],
) string {
	return w.formulationNodeToWritten(path, node.Root)
}

func (w *WrittenResolver) specToWritten(path ast.Path, node ast.Spec) string {
	return w.formulationNodeToWritten(path, node.Root)
}

func (w *WrittenResolver) aliasToWritten(path ast.Path, node ast.Alias) string {
	return w.formulationNodeToWritten(path, node.Root)
}

func (w *WrittenResolver) targetToWritten(path ast.Path, node ast.Target) string {
	return w.formulationNodeToWritten(path, node.Root)
}

func (w *WrittenResolver) infixCommandToWritten(
	path ast.Path,
	node *ast.InfixCommandExpression,
) (string, bool) {
	sig := GetSignatureStringFromInfixCommand(*node)
	return w.toWrittenImpl(path, node, sig)
}

func (w *WrittenResolver) commandToWritten(path ast.Path, node *ast.CommandExpression) (string, bool) {
	sig := GetSignatureStringFromCommand(*node)
	return w.toWrittenImpl(path, node, sig)
}

func (w *WrittenResolver) toWrittenImpl(path ast.Path, node ast.MlgNodeKind, sig string) (string, bool) {
	found := false
	if id, ok := w.nodeTracker.signaturesToIds[sig]; ok {
		if docSummary, ok := w.documentedSummaries[id]; ok {
			if writtenItems, ok := GetResolvedWritten(docSummary); ok {
				if summaryInput, ok := w.inputSummaries[id]; ok {
					matchResult := Match(node, summaryInput.Input)
					if matchResult.MatchMakesSense && len(matchResult.Messages) == 0 {
						// nolint:ineffassign
						found = true

						nameToWritten := make(map[string]string)
						nameToWrittenPlus := make(map[string]string)
						nameToWrittenMinus := make(map[string]string)
						nameToWrittenEqual := make(map[string]string)

						varArgNameToWritten := make(map[string][]string)
						varArgNameToWrittenPlus := make(map[string][]string)
						varArgNameToWrittenMinus := make(map[string][]string)
						varArgNameToWrittenEqual := make(map[string][]string)

						for name, exp := range matchResult.Mapping {
							text := w.formulationNodeToWritten(path, exp)
							textPlus := getVarPlusQuestionMarkText(text, exp)
							textMinus := getVarMinusQuestionMarkText(text, exp)
							textEqual := getVarEqualQuestionMarkText(text)

							nameToWritten[name] = text
							nameToWrittenPlus[name] = textPlus
							nameToWrittenMinus[name] = textMinus
							nameToWrittenEqual[name] = textEqual

							varArgNameToWritten[name] = []string{text}
							varArgNameToWrittenPlus[name] = []string{textPlus}
							varArgNameToWrittenMinus[name] = []string{textMinus}
							varArgNameToWrittenEqual[name] = []string{textEqual}
						}

						for name, exps := range matchResult.VarArgMapping {
							values := make([]string, 0)
							valuesPlus := make([]string, 0)
							valuesMinus := make([]string, 0)
							valuesEqual := make([]string, 0)
							for _, exp := range exps {
								val := w.formulationNodeToWritten(path, exp)
								valPlus := getVarPlusQuestionMarkText(val, exp)
								valMinus := getVarMinusQuestionMarkText(val, exp)
								valEqual := getVarEqualQuestionMarkText(val)

								values = append(values, val)
								valuesPlus = append(valuesPlus, valPlus)
								valuesMinus = append(valuesMinus, valMinus)
								valuesEqual = append(valuesEqual, valEqual)
							}

							varArgNameToWritten[name] = values
							varArgNameToWrittenPlus[name] = valuesPlus
							varArgNameToWrittenMinus[name] = valuesMinus
							varArgNameToWrittenEqual[name] = valuesEqual

							if len(values) > 0 {
								nameToWritten[name] = values[0]
							}

							if len(valuesPlus) > 0 {
								nameToWrittenPlus[name] = valuesPlus[0]
							}

							if len(valuesMinus) > 0 {
								nameToWrittenMinus[name] = valuesMinus[0]
							}

							if len(valuesEqual) > 0 {
								nameToWrittenEqual[name] = valuesEqual[0]
							}
						}

						result := ""
						for _, item := range writtenItems {
							switch it := item.(type) {
							case *ast.StringItem:
								result += it.Text
							case *ast.SubstitutionItem:
								if it.IsVarArg {
									if it.NameSuffix == "+" {
										result += valuesToString(
											varArgNameToWrittenPlus[it.Name], it.Prefix, it.Infix, it.Suffix)
									} else if it.NameSuffix == "-" {
										result += valuesToString(
											varArgNameToWrittenMinus[it.Name], it.Prefix, it.Infix, it.Suffix)
									} else if it.NameSuffix == "=" {
										result += valuesToString(
											varArgNameToWrittenEqual[it.Name], it.Prefix, it.Infix, it.Suffix)
									} else {
										result += valuesToString(
											varArgNameToWritten[it.Name], it.Prefix, it.Infix, it.Suffix)
									}
								} else {
									if it.NameSuffix == "+" {
										result += nameToWrittenPlus[it.Name]
									} else if it.NameSuffix == "-" {
										result += nameToWrittenMinus[it.Name]
									} else if it.NameSuffix == "=" {
										result += nameToWrittenEqual[it.Name]
									} else {
										result += nameToWritten[it.Name]
									}
								}
							}
						}
						return result, true
					} else {
						if matchResult.MatchMakesSense {
							for _, message := range matchResult.Messages {
								w.diagnosticTracker.Append(frontend.Diagnostic{
									Type:     frontend.Error,
									Origin:   frontend.BackendOrigin,
									Message:  message,
									Path:     path,
									Position: node.GetCommonMetaData().Start,
								})
							}
						}
						return "", false
					}
				}
			}
		}
	}
	if !found {
		w.diagnosticTracker.Append(frontend.Diagnostic{
			Type:   frontend.Error,
			Origin: frontend.BackendOrigin,
			Message: fmt.Sprintf(
				"Signature %s does not have a Documented:called: or Documented:written: section", sig),
			Path:     path,
			Position: node.GetCommonMetaData().Start,
		})
	}
	return "", false
}

func (w *WrittenResolver) formulationNodeToWritten(path ast.Path, mlgNode ast.MlgNodeKind) string {
	if mlgNode == nil {
		return ""
	}
	customToCode := func(node ast.MlgNodeKind) (string, bool) {
		switch n := node.(type) {
		case *ast.NameForm:
			return nameToRenderedName(n.Text, n.VarArg.IsVarArg), true
		case *ast.ConditionalSetForm:
			result := "\\left \\{"
			result += w.formulationNodeToWritten(path, n.Target)
			if n.Specification != nil {
				result += "\\: : \\:"
				result += w.formulationNodeToWritten(path, n.Specification)
			}
			if n.Condition != nil {
				result += "\\: | \\:"
				result += w.formulationNodeToWritten(path, n.Condition)
			}
			result += "\\right \\}"
			return result, true
		case *ast.ConditionalSetIdForm:
			result := "\\left \\{"
			result += w.formulationNodeToWritten(path, n.Target)
			if n.Specification != nil {
				result += "\\: : \\:"
				result += w.formulationNodeToWritten(path, n.Specification)
			}
			if n.Condition != nil {
				result += "\\: | \\:"
				result += w.formulationNodeToWritten(path, n.Condition)
			}
			result += "\\right \\}"
			return result, true
		case *ast.FunctionLiteralExpression:
			result := ""
			if len(n.Lhs.Params) == 1 {
				result += w.formulationNodeToWritten(path, n.Lhs.Params[0])
			} else {
				result += w.formulationNodeToWritten(path, &n.Lhs)
			}
			result += " \\mapsto "
			result += w.formulationNodeToWritten(path, n.Rhs)
			return result, true
		case *ast.ConditionalSetExpression:
			result := "\\left \\{"
			result += w.formulationNodeToWritten(path, n.Target)
			if len(n.Specifications) > 0 {
				result += "\\: : \\:"
				for i, cond := range n.Specifications {
					if i > 0 {
						result += " ;\\: "
					}
					result += w.formulationNodeToWritten(path, cond)
				}
			}
			if condition, ok := n.Condition.Get(); ok {
				result += "\\: | \\:"
				result += w.formulationNodeToWritten(path, condition)
			}
			result += "\\right \\}"
			return result, true
		case *ast.OrdinalCallExpression:
			result := ""
			result += w.formulationNodeToWritten(path, n.Target)
			result += "_{"
			for i, arg := range n.Args {
				if i > 0 {
					result += ", "
				}
				result += w.formulationNodeToWritten(path, arg)
			}
			result += "}"
			return result, true
		case *ast.ExpressionColonArrowItem:
			result := ""
			switch lhs := n.Lhs.(type) {
			case *ast.CommandExpression:
				// if the left-hand-side is of the form \f then print it verbatim
				result += fmt.Sprintf("\\verb'%s'", lhs.ToCode(ast.NoOp))
			case *ast.InfixOperatorCallExpression:
				if _, ok := lhs.Target.(*ast.InfixCommandExpression); ok {
					// if the left-hand-side is of the form `x \in/ y` then
					// print it verbatim
					result += fmt.Sprintf("\\verb'%s'", lhs.ToCode(ast.NoOp))
				} else {
					result += w.formulationNodeToWritten(path, n.Lhs)
				}
			default:
				result += w.formulationNodeToWritten(path, n.Lhs)
			}
			result += " :\\rArr "
			result += w.formulationNodeToWritten(path, n.Rhs)
			return result, true
		case *ast.ExpressionColonDashArrowItem:
			result := ""
			result += w.formulationNodeToWritten(path, n.Lhs)
			result += " :\\rarr "
			for i, item := range n.Rhs {
				if i > 0 {
					result += "; "
				}
				result += w.formulationNodeToWritten(path, item)
			}
			return result, true
		case *ast.ExpressionColonEqualsItem:
			result := ""
			result += w.formulationNodeToWritten(path, n.Lhs)
			result += " \\coloneqq "
			result += w.formulationNodeToWritten(path, n.Rhs)
			return result, true
		case *ast.IsExpression:
			result := ""
			for i, exp := range n.Lhs {
				if i > 0 {
					result += ", "
				}
				result += w.formulationNodeToWritten(path, exp)
			}
			result += " \\textrm{ is } "
			for i, exp := range n.Rhs {
				if i > 0 {
					result += ", "
				}
				result += w.formulationNodeToWritten(path, exp)
			}
			return result, true
		case *ast.ExtendsExpression:
			result := ""
			for i, exp := range n.Lhs {
				if i > 0 {
					result += ", "
				}
				result += w.formulationNodeToWritten(path, exp)
			}
			result += " \\textrm{ extends } "
			for i, exp := range n.Rhs {
				if i > 0 {
					result += ", "
				}
				result += w.formulationNodeToWritten(path, exp)
			}
			return result, true
		case *ast.NonEnclosedNonCommandOperatorTarget:
			if n.Text == "!=" {
				return "\\neq", true
			}
			if n.Text == "&" {
				return "\\:\\&\\:", true
			}
			return n.Text, true
		case *ast.EnclosedNonCommandOperatorTarget:
			text := w.formulationNodeToWritten(path, n.Target)

			// if the text is of the form a.b.c.*
			// then render the "a.b.c" as text
			index := strings.LastIndex(text, ".")
			if index >= 0 {
				// text is of the form a.b.c.*
				head := text[0:index]
				tail := text[index+1:]
				return fmt.Sprintf("\\:[\\textrm{%s}.%s]\\:", head, tail), true
			}

			// otherwise there are no . characters and so if the inner
			// text starts with a letter (for example it could be
			// "times") then render it as a LaTeX command (for
			// example "\times")
			//
			// NOTE: this is just a hack until type checking is
			//       complete so the operator can be disambiguated
			//       and the actual rendering can be done as specified
			//       by a "written" or "writing"
			if len(text) > 0 && unicode.IsLetter(rune(text[0])) {
				return "\\" + text, true
			}

			// last it is an operator like * and so just return
			// the original text
			return text, true
		case *ast.CommandExpression:
			return w.commandToWritten(path, n)
		case *ast.InfixCommandExpression:
			return w.infixCommandToWritten(path, n)
		case *ast.AsExpression:
			return w.formulationNodeToWritten(path, n.Lhs), true
		case *ast.CommandTypeForm:
			// \:set
			noPrefix := strings.Replace(n.ToCode(ast.NoOp), "\\:", "", 1)
			return fmt.Sprintf("\\textrm{%s}", noPrefix), true
		case *ast.InfixCommandTypeForm:
			// \:in:/
			noPrefix := strings.Replace(n.ToCode(ast.NoOp), "\\:", "", 1)
			noSuffix := strings.Replace(noPrefix, ":/", "", 1)
			return fmt.Sprintf("\\textrm{ %s }", noSuffix), true
		case *ast.MapToElseBuiltinExpression:
			// \\map{x[i[k]]}:to{x[i[k]] + 1}:else{0}
			text := "\\textrm{map }"
			text += w.formulationNodeToWritten(path, &n.Target)
			text += "\\textrm{ to }"
			text += w.formulationNodeToWritten(path, n.To)
			if n.Else != nil {
				text += "\\textrm{ else }"
				text += w.formulationNodeToWritten(path, n.Else)
			}
			return text, true
		case *ast.SelectFromBuiltinExpression:
			// \\select{statement|specification}:from{x}
			text := "\\textrm{select }"
			for i, kind := range n.Kinds {
				if i > 0 {
					text += " \\text{ or }"
				}
				text += fmt.Sprintf("\\textrm{%s}", kind)
			}
			text += "\\textrm{ from }"
			text += w.formulationNodeToWritten(path, &n.Target)
			return text, true
		case *ast.AbstractBuiltinExpression:
			return "\\textrm{abstract}", true
		case *ast.SpecificationBuiltinExpression:
			return "\\textrm{specification}", true
		case *ast.StatementBuiltinExpression:
			return "\\textrm{statement}", true
		case *ast.ExpressionBuiltinExpression:
			return "\\textrm{expression}", true
		case *ast.TypeBuiltinExpression:
			return "\\textrm{type}", true
		default:
			return "", false
		}
	}

	return ast.Debug(mlgNode, customToCode)
}

func (w *WrittenResolver) updateFormulationStrings(
	path ast.Path,
	node phase4.Node,
	keyToFormulationStr map[int]string,
) {
	if arg, ok := node.(*phase4.Argument); ok {
		if argData, ok := arg.Arg.(*phase4.FormulationArgumentData); ok {
			key := argData.MetaData.Key
			if newText, ok := keyToFormulationStr[key]; ok {
				argData.Text = newText
			} else {
				w.diagnosticTracker.Append(frontend.Diagnostic{
					Type:     frontend.Warning,
					Origin:   frontend.BackendOrigin,
					Message:  fmt.Sprintf("Could not process: %s", argData.Text),
					Path:     path,
					Position: arg.MetaData.Start,
				})
			}
		} else if argData, ok := arg.Arg.(*phase4.ArgumentTextArgumentData); ok {
			// this branch is needed to handle Targets
			key := argData.MetaData.Key
			if newText, ok := keyToFormulationStr[key]; ok {
				argData.Text = newText
			}
			// else if the `keyToFormulationStr`` map doesn't contain `key` then
			// the `argData` doesn't need its text modified.  That is, it can be
			// treated as LaTeX code as is.
			// For example, f(x) can be rendered as is, but {x | ...} needs to
			// be rendered as \left \{x \: | \: \ldots \right \}
		}
	}
	size := node.Size()
	for i := 0; i < size; i++ {
		w.updateFormulationStrings(path, node.ChildAt(i), keyToFormulationStr)
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// nolint:unused
func getVarQuestionMarkText(rawWritten string, originalNode ast.MlgNodeKind) string {
	return getVarMinusQuestionMarkText(rawWritten, originalNode)
}

func getVarEqualQuestionMarkText(rawWritten string) string {
	return rawWritten
}

func getVarPlusQuestionMarkText(rawWritten string, originalNode ast.MlgNodeKind) string {
	if _, ok := originalNode.(*ast.NameForm); ok {
		return rawWritten
	}
	result := rawWritten
	if !strings.HasPrefix(result, "(") {
		result = "(" + result
	}
	if !strings.HasSuffix(result, ")") {
		result = result + ")"
	}
	return result
}

func getVarMinusQuestionMarkText(rawWritten string, originalNode ast.MlgNodeKind) string {
	isTupleExp := false
	if tup, ok := originalNode.(*ast.TupleExpression); ok && len(tup.Args) != 1 {
		isTupleExp = true
	}
	isTupleForm := false
	if tup, ok := originalNode.(*ast.TupleForm); ok && len(tup.Params) != 1 {
		isTupleForm = true
	}
	if isTupleExp || isTupleForm {
		return rawWritten
	}
	return strings.TrimSuffix(strings.TrimPrefix(rawWritten, "("), ")")
}

func valuesToString(values []string, prefix string, infix string, suffix string) string {
	// this function assumes exactly one of prefix, infix, and suffix is non-empty
	if prefix != "" {
		result := ""
		for _, v := range values {
			result += prefix
			result += v
		}
		return result
	}

	if suffix != "" {
		result := ""
		for _, v := range values {
			result += v
			result += suffix
		}
		return result
	}

	result := ""
	for i, v := range values {
		if i > 0 {
			result += infix
		}
		result += v
	}
	return result
}

func splitName(name string) []string {
	result := make([]string, 0)
	runes := []rune(name)
	i := 0
	for i < len(runes) {
		if unicode.IsLetter(runes[i]) {
			inner := make([]rune, 0)
			for i < len(runes) && unicode.IsLetter(runes[i]) {
				inner = append(inner, runes[i])
				i++
			}
			result = append(result, string(inner))
		} else {
			result = append(result, string(runes[i]))
			i++
		}
	}
	return result
}

func nameToRenderedName(name string, isVarArg bool) string {
	processed := processSpecialTokens(name)
	if !isVarArg {
		return processed
	}
	return fmt.Sprintf("{%s}...", processed)
}

func processSpecialTokens(name string) string {
	// split the name into alphabetic and non-alphabetic parts
	// and convert ` tokens to ' and greek names to the associated
	// LaTeX equivalent
	joined := ""
	for _, part := range splitName(name) {
		if isGreekLetter(part) {
			joined += fmt.Sprintf("\\%s", part)
		} else if part == "`" {
			joined += "'"
		} else {
			joined += part
		}
	}

	// convert x0 to x_{0} but don't convert 123 to 1_{23}
	digitRegex := regexp.MustCompile(`([^_0-9]+)(\d+)`)
	digitItems := digitRegex.FindStringSubmatch(joined)
	// format of items: [(full match) (group 1) (group 2)]
	if len(digitItems) == 3 && digitItems[0] == joined {
		return fmt.Sprintf("%s_{%s}", digitItems[1], digitItems[2])
	}

	// convert abc_xyz to abc_{xyz}
	underscoreRegex := regexp.MustCompile(`([^_]+)_(.+)`)
	underscoreItems := underscoreRegex.FindStringSubmatch(joined)
	// format of items: [(full match) (group 1) (group 2)]
	if len(underscoreItems) == 3 && underscoreItems[0] == joined {
		return fmt.Sprintf("%s_{%s}", underscoreItems[1], processSpecialTokens(underscoreItems[2]))
	}

	return joined
}

func isGreekLetter(name string) bool {
	if name == "varGamma" ||
		name == "varDelta" ||
		name == "varTheta" ||
		name == "varLambda" ||
		name == "varXi" ||
		name == "varPi" ||
		name == "varSigma" ||
		name == "varUpsilon" ||
		name == "varPhi" ||
		name == "varPsi" ||
		name == "varOmega" {
		return true
	}

	text := strings.ToLower(name)
	return text == "alpha" ||
		text == "beta" ||
		text == "gamma" ||
		text == "delta" ||
		text == "epsilon" ||
		text == "zeta" ||
		text == "eta" ||
		text == "theta" ||
		text == "iota" ||
		text == "kappa" ||
		text == "mu" ||
		text == "nu" ||
		text == "xi" ||
		text == "omicron" ||
		text == "pi" ||
		text == "rho" ||
		text == "sigma" ||
		text == "tau" ||
		text == "upsilon" ||
		text == "phi" ||
		text == "chi" ||
		text == "psi" ||
		text == "omega" ||
		name == "varepsilon" ||
		name == "varkappa" ||
		name == "vartheta" ||
		name == "varpi" ||
		name == "varrho" ||
		name == "varsigma" ||
		name == "varphi" ||
		name == "digamma"
}

func inlineProcessForRendering(node phase4.Node) {
	switch n := node.(type) {
	case *phase4.Argument:
		switch a := n.Arg.(type) {
		case *phase4.ArgumentTextArgumentData:
			a.Text = nameToRenderedName(a.Text, false)
		}
	}
	for i := 0; i < node.Size(); i++ {
		inlineProcessForRendering(node.ChildAt(i))
	}
}

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
	"fmt"
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/phase4"
	"mathlingua/internal/mlglib"
	"regexp"
	"strings"
	"unicode"
)

// The general approach for checking is the following:
// *** Note: At each step, a shared DiagnosticTracker is used to track any diagnostics ***
// - Find all Mathlingua files in the files/dirs requested to be checked
// - Update all Mathlingua file contents so that each top-level entries has an id
// *** At this point, all top-level entries have an id. ***
// - Parse the code in all files to generate ASTs
//   - The phase4 AST preserves the formatting of the structural language
//   - The phase5 AST has formulations expanded so all operators are disambiguated
//   - The phase4 AST has each node (in particular each formulation) have a unique
//     id.  With this, rendering of the structure is done using the phase4 AST and
//     rendering of formulations is done using the formulations specified in the
//     phase5 AST using the ids to map formulations in the phase5 AST to the
//     phase4 AST.
//   - The Root node has many Document nodes as children where each Document
//     corresponds to a file
// - Normalize each phase5 top-level entry
//   - Each target is updated so that each input and output has an explicit identifier
//     (i.e. `f(x)` to `f(x) := y` and `(a, b, c)` to `X := (a, b, c)`).
//   - All forumlations are expanded so that each alias is expanded.
// *** At this point, no aliases need to be considered and every input/output has a name ***
// - Recursively descend the Root and add a Scope to each node where, whenever a node needs to
//   record an identifier it is recorded.  For example, the Root will record anything in
//   Specify: entries, forAll: will record its introduced identifiers, etc.
// - For each Defines, Describes, and States top-level entry, create a *summary* for the
//   entry that encodes the shape and types of the inputs, outputs, constraints etc. described
//   with spec aliases in the constriants as well as the definition site Scope.
// - For each top-level entry,
//   - Expand all spec aliases to a list of `is` statements
//   - Disambiguate each operator with so it is replaced with its full unambiguous form
//   * To do this, when an `is` statement is encountered, the summary for the associated
//     type is copied.  It is then resolved, using the same approach described here, to a
//     summary that has no spec aliases.  Instead, all of the spec aliases have be replaced
//     with `is` statements.  The item in the left-had-side of the `is` will then be verified
//     if its inputs match the summary.  If not Diagnostics will be recorded in a tracker,
//     otherwise the calling scope will be updated to include the `is` expansion for the
//     identifier.  This is repeated for all spec aliases.
//
//     At this point, all scopes in the top-level item have only `is` descriptions of all
//     identifiers (i.e. none have spec aliases).  From this, the top-level item is traversed
//     from the bottom up.  For each item, either it is a name, a command call, or an operator.
//
//     If it is a name, its type is looked up in the scope.  If it is a command call, its
//     type is determined by resolving the Defines summary for the command.  If it is an
//     operator, the types of the children are used to disambiguate the operator, and the
//     operator is replaced with its disambguated form.
//
//     At this point, all nodes have a type, and all operators are disambiguated.  From
//     this, the "written as" form of each node can be determined, again bottom up and recorded
//     in the AST.
//
//     Hence each formulation has a "written as" form that will be used for rendering.  This
//     rendering is recorded to be outputed to files.
//
//     Last, any semantic checks that don't need to be verified are also checked.
//
//     The 'view' operatio n follows the same procedure except it doesn't do the last semantic
//     checks.

type WorkspacePageResponse struct {
	Path ast.Path
	Page PageResponse
}

type PathsResponse struct {
	Error string
	Paths []PathLabelPair
}

type PageResponse struct {
	Error       string
	Diagnostics []frontend.Diagnostic
	Document    phase4.Document
}

type EntryResponse struct {
	Error string
	Entry phase4.TopLevelNodeKind
}

type CheckResult struct {
	Diagnostics []frontend.Diagnostic
}

type PathLabelPair struct {
	Path  ast.Path
	Label string
	IsDir bool
}

type PathLabelContent struct {
	Path  ast.Path
	Label string
	// Content is nil if the path is a directory
	Content *string
}

type IWorkspace interface {
	DocumentCount() int
	Paths() []PathLabelPair
	GetDocumentAt(path ast.Path) (phase4.Document, []frontend.Diagnostic)
	GetEntryById(id string) (phase4.TopLevelNodeKind, error)
	GetEntryBySignature(signature string) (phase4.TopLevelNodeKind, error)
	Check() CheckResult
	GetUsages() []string
}

func NewWorkspace(contents []PathLabelContent, tracker frontend.IDiagnosticTracker) IWorkspace {
	w := workspace{
		tracker:         tracker,
		contents:        contents,
		signaturesToIds: make(map[string]string, 0),
		summaries:       make(map[string]SummaryKind, 0),
		phase4Entries:   make(map[string]phase4.TopLevelNodeKind, 0),
		topLevelEntries: make(map[string]ast.TopLevelItemKind, 0),
		usages:          make([]string, 0),
	}
	w.initialize(contents)
	return &w
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type workspace struct {
	// map paths to path contents
	contents []PathLabelContent
	// the tracker used to record diagnostics
	tracker frontend.IDiagnosticTracker
	// mapping of paths to phase4 documents
	phase4Root *phase4.Root
	// the root of the phase5 parse tree generated
	astRoot *ast.Root
	// map signatures to ids
	signaturesToIds map[string]string
	// map ids to summaries
	summaries map[string]SummaryKind
	// map ids to phase4 parses of top-level entries
	phase4Entries map[string]phase4.TopLevelNodeKind
	// map ids to phase5 top-level types
	topLevelEntries map[string]ast.TopLevelItemKind
	// usages of defined commands
	usages []string
}

func (w *workspace) DocumentCount() int {
	return len(w.contents)
}

func (w *workspace) Paths() []PathLabelPair {
	result := make([]PathLabelPair, 0)
	for _, pair := range w.contents {
		result = append(result, PathLabelPair{
			Path:  pair.Path,
			Label: pair.Label,
		})
	}
	return result
}

func (w *workspace) GetDocumentAt(path ast.Path) (phase4.Document, []frontend.Diagnostic) {
	doc := w.astRoot.Documents[path]
	phase4Doc := w.phase4Root.Documents[path]
	result := w.getRenderedNode(path, &phase4Doc, &doc)
	resultDoc, _ := result.(*phase4.Document)
	return *resultDoc, w.getDiagnosticsForPath(path)
}

func (w *workspace) GetEntryById(id string) (phase4.TopLevelNodeKind, error) {
	phase4Entry, phase4Ok := w.phase4Entries[id]
	astEntry, astOk := w.topLevelEntries[id]
	if !phase4Ok || !astOk {
		return nil, fmt.Errorf("An entry with id %s does not exist", id)
	}
	result := w.getRenderedNode(ast.ToPath(""), phase4Entry, astEntry)
	castResult, _ := result.(phase4.TopLevelNodeKind)
	return castResult, nil
}

func (w *workspace) GetEntryBySignature(signature string) (phase4.TopLevelNodeKind, error) {
	id, ok := w.signaturesToIds[signature]
	if !ok {
		return nil, fmt.Errorf("Could not determine the id for signature %s", signature)
	}
	return w.GetEntryById(id)
}

func (w *workspace) Check() CheckResult {
	w.findUsedUnknownSignatures()
	for _, pair := range w.Paths() {
		// get all of the documents to populate the tracker
		// with any rendering errors
		w.GetDocumentAt(pair.Path)
	}
	return CheckResult{
		Diagnostics: w.tracker.Diagnostics(),
	}
}

func (w *workspace) GetUsages() []string {
	return w.usages
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (w *workspace) initialize(contents []PathLabelContent) {
	w.contents = contents
	contentMap := make(map[ast.Path]string)
	for _, pair := range contents {
		if pair.Content != nil {
			contentMap[pair.Path] = *pair.Content
		}
	}
	w.phase4Root, w.astRoot = ParseRoot(contentMap, w.tracker)
	w.normalizeAst()
	w.populateScopes()
	w.initializeSignaturesToIds()
	w.initializeSummaries()
	w.initializePhase4Entries()
	w.initializeTopLevelEntries()
	w.updateUsedSignatures()
	w.initializeUsages()
}

func (w *workspace) getRenderedNode(
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

func (w *workspace) formulationLikeToString(
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
	} else {
		node.ForEach(func(subNode ast.MlgNodeKind) {
			w.formulationLikeToString(path, subNode, keyToFormulationStr)
		})
	}
}

func (w *workspace) formulationToWritten(
	path ast.Path,
	node ast.Formulation[ast.FormulationNodeKind],
) string {
	return w.formulationNodeToWritten(path, node.Root)
}

func (w *workspace) specToWritten(path ast.Path, node ast.Spec) string {
	return w.formulationNodeToWritten(path, node.Root)
}

func (w *workspace) aliasToWritten(path ast.Path, node ast.Alias) string {
	return w.formulationNodeToWritten(path, node.Root)
}

func (w *workspace) infixCommandToWritten(
	path ast.Path,
	node *ast.InfixCommandExpression,
) (string, bool) {
	sig := GetSignatureStringFromInfixCommand(*node)
	return w.toWrittenImpl(path, node, sig)
}

func (w *workspace) commandToWritten(path ast.Path, node *ast.CommandExpression) (string, bool) {
	sig := GetSignatureStringFromCommand(*node)
	return w.toWrittenImpl(path, node, sig)
}

func (w *workspace) toWrittenImpl(path ast.Path, node ast.MlgNodeKind, sig string) (string, bool) {
	found := false
	if id, ok := w.signaturesToIds[sig]; ok {
		if summary, ok := w.summaries[id]; ok {
			if writtenItems, ok := GetResolvedWritten(summary); ok {
				if summaryInput, ok := GetResolvedInput(summary); ok {
					matchResult := Match(node, summaryInput)
					if matchResult.MatchMakesSense && len(matchResult.Messages) == 0 {
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
							case *StringItem:
								result += it.Text
							case *SubstitutionItem:
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
								w.tracker.Append(frontend.Diagnostic{
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
		w.tracker.Append(frontend.Diagnostic{
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

func (w *workspace) formulationNodeToWritten(path ast.Path, mlgNode ast.MlgNodeKind) string {
	if mlgNode == nil {
		return ""
	}
	customToCode := func(node ast.MlgNodeKind) (string, bool) {
		switch n := node.(type) {
		case *ast.NameForm:
			return nameToRenderedName(n.Text, n.VarArg.IsVarArg), true
		case *ast.FunctionLiteralExpression:
			result := ""
			result += w.formulationNodeToWritten(path, &n.Lhs)
			result += " \\rArr "
			result += w.formulationNodeToWritten(path, n.Rhs)
			return result, true
		case *ast.ConditionalSetExpression:
			result := "\\left \\{"
			result += w.formulationNodeToWritten(path, n.Target)
			result += "\\: | \\:"
			for i, cond := range n.Conditions {
				if i > 0 {
					result += " ;\\: "
				}
				result += w.formulationNodeToWritten(path, cond)
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
			result += w.formulationNodeToWritten(path, n.Lhs)
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
		default:
			return "", false
		}
	}

	return ast.Debug(mlgNode, customToCode)
}

func (w *workspace) updateFormulationStrings(
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
				w.tracker.Append(frontend.Diagnostic{
					Type:     frontend.Warning,
					Origin:   frontend.BackendOrigin,
					Message:  fmt.Sprintf("Could not process: %s", argData.Text),
					Path:     path,
					Position: arg.MetaData.Start,
				})
			}
		}
	}
	size := node.Size()
	for i := 0; i < size; i++ {
		w.updateFormulationStrings(path, node.ChildAt(i), keyToFormulationStr)
	}
}

func (w *workspace) getDiagnosticsForPath(path ast.Path) []frontend.Diagnostic {
	result := make([]frontend.Diagnostic, 0)
	for _, diag := range w.tracker.Diagnostics() {
		if diag.Path == path {
			result = append(result, diag)
		}
	}
	return result
}

func (w *workspace) initializeSignaturesToIds() {
	for path, doc := range w.astRoot.Documents {
		for _, item := range doc.Items {
			sig, sigOk := GetSignatureStringFromTopLevel(item)
			id, idOk := GetAstMetaId(item)
			if sigOk && idOk {
				if _, hasSig := w.signaturesToIds[sig]; hasSig {
					w.tracker.Append(frontend.Diagnostic{
						Type:     frontend.Error,
						Origin:   frontend.BackendOrigin,
						Message:  fmt.Sprintf("Duplicate defined signature %s", sig),
						Path:     path,
						Position: item.GetCommonMetaData().Start,
					})
				} else {
					w.signaturesToIds[sig] = id
				}
			}
		}
	}
}

func (w *workspace) initializeUsages() {
	for _, doc := range w.astRoot.Documents {
		for _, item := range doc.Items {
			usage, ok := GetUsageFromTopLevel(item)
			if ok {
				w.usages = append(w.usages, usage)
			}
		}
	}
}

func (w *workspace) initializeSummaries() {
	for _, doc := range w.astRoot.Documents {
		for _, item := range doc.Items {
			id, idOk := GetAstMetaId(item)
			summary, summaryOk := Summarize(item, w.tracker)
			if idOk && summaryOk {
				w.summaries[id] = summary
			}
		}
	}
}

func (w *workspace) initializePhase4Entries() {
	for _, doc := range w.phase4Root.Documents {
		for _, item := range doc.Nodes {
			id, idOk := GetPhase4MetaId(item)
			if idOk {
				switch n := item.(type) {
				case *phase4.TextBlock:
					n.MetaData.Id = id
				case *phase4.Group:
					n.MetaData.Id = id
				}
				w.phase4Entries[id] = item
			}
		}
	}
}

func (w *workspace) initializeTopLevelEntries() {
	for _, doc := range w.astRoot.Documents {
		for _, item := range doc.Items {
			id, idOk := GetAstMetaId(item)
			if idOk {
				w.topLevelEntries[id] = item
			}
		}
	}
}

func (w *workspace) updateUsedSignatures() {
	for _, doc := range w.astRoot.Documents {
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

			phase4Item, ok := w.phase4Entries[id]
			if !ok {
				// fmt.Printf("Could not get the phase4 item for id %s", id)
				continue
			}

			w.updatePhase4UsedSignatures(phase4Item, keyToUsedSignatures)
		}
	}
}

func updateAstUsedSignatures(node ast.MlgNodeKind) {
	if node == nil {
		return
	}

	if formulation, ok := node.(*ast.Formulation[ast.FormulationNodeKind]); ok {
		formulation.FormulationMetaData.UsedSignatureStrings = GetUsedSignatureStrings(formulation)
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

func (w *workspace) updatePhase4UsedSignatures(
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
		w.updatePhase4UsedSignatures(node.ChildAt(i), keyToUsedSignatures)
	}
}

// Normalizes the given AST node in-place, which includes:
//   - update any place where an identifier is introduced to ensure it has any input and
//     output identifiers specified.  That is if `f(x)` is introduced, it is replaced with
//     something like `f(x) := y` where the output has an identifier.  Also, if `(a, b, c)`
//     is introduced, then it replaced with something like `X := (a, b, c)` where an
//     identifier `X` for the tuple itself is introduced.
//   - Any alias in formulations are expanded so that aliases are not needed anymore.
func (w *workspace) normalizeAst() {
	w.includeMissingIdentifiers()
	w.expandAliases()
}

func (w *workspace) populateScopes() {
	PopulateScopes(w.astRoot, w.tracker)
}

func (w *workspace) includeMissingIdentifiers() {
	includeMissingIdentifiersAt(w.astRoot, mlglib.NewKeyGenerator())
}

func (w *workspace) expandAliases() {
	expandAliasesAt(w.astRoot, w.summaries)
}

func (w *workspace) findUsedUnknownSignatures() {
	for path, doc := range w.astRoot.Documents {
		for _, item := range doc.Items {
			findUsedUnknownSignaturesImpl(item, path, w)
		}
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

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

// Replace `f(x)` with `f(x) := var'#'` but do not change `f(x) := y`
// Replace `(a, b)` with `var'#' := (a, b)` but do not change `X := (a, b)`
func replaceMissingIdentifier(target ast.Target, keyGen mlglib.IKeyGenerator) ast.Target {
	switch f := target.Root.(type) {
	case *ast.FunctionForm:
		return ast.Target{
			Root: &ast.StructuralColonEqualsForm{
				Lhs: f,
				Rhs: &ast.NameForm{
					Text:            fmt.Sprintf("var'%d'", keyGen.Next()),
					IsStropped:      false,
					HasQuestionMark: false,
					VarArg: ast.VarArgData{
						IsVarArg: false,
					},
				},
			},
		}
	case *ast.TupleForm:
		return ast.Target{
			Root: &ast.StructuralColonEqualsForm{
				Lhs: &ast.NameForm{
					Text:            fmt.Sprintf("var'%d'", keyGen.Next()),
					IsStropped:      false,
					HasQuestionMark: false,
					VarArg: ast.VarArgData{
						IsVarArg: false,
					},
				},
				Rhs: f,
			},
		}
	}
	return target
}

func includeMissingIdentifiersInTargets(targets []ast.Target, keyGen mlglib.IKeyGenerator) {
	for i := range targets {
		targets[i] = replaceMissingIdentifier(targets[i], keyGen)
	}
}

func includeMissingIdentifiersAt(node ast.MlgNodeKind, keyGen mlglib.IKeyGenerator) {
	if node == nil {
		return
	}

	switch n := node.(type) {
	case *ast.DefinesGroup:
		defines := &n.Defines
		defines.Defines = replaceMissingIdentifier(defines.Defines, keyGen)
		if n.Using != nil {
			includeMissingIdentifiersInTargets(n.Using.Using, keyGen)
		}
	case *ast.DescribesGroup:
		describes := &n.Describes
		describes.Describes = replaceMissingIdentifier(describes.Describes, keyGen)
		if n.Using != nil {
			includeMissingIdentifiersInTargets(n.Using.Using, keyGen)
		}
	case *ast.AxiomGroup:
		if n.Given != nil {
			includeMissingIdentifiersInTargets(n.Given.Given, keyGen)
		}
	case *ast.ConjectureGroup:
		if n.Given != nil {
			includeMissingIdentifiersInTargets(n.Given.Given, keyGen)
		}
	case *ast.TheoremGroup:
		if n.Given != nil {
			includeMissingIdentifiersInTargets(n.Given.Given, keyGen)
		}
	case *ast.CorollaryGroup:
		if n.Given != nil {
			includeMissingIdentifiersInTargets(n.Given.Given, keyGen)
		}
	case *ast.LemmaGroup:
		if n.Given != nil {
			includeMissingIdentifiersInTargets(n.Given.Given, keyGen)
		}
	case *ast.ForAllGroup:
		includeMissingIdentifiersInTargets(n.ForAll.Targets, keyGen)
	case *ast.ExistsGroup:
		includeMissingIdentifiersInTargets(n.Exists.Targets, keyGen)
	case *ast.ExistsUniqueGroup:
		includeMissingIdentifiersInTargets(n.ExistsUnique.Targets, keyGen)
	case *ast.ViewGroup:
		if n.Using != nil {
			includeMissingIdentifiersInTargets(n.Using.Using, keyGen)
		}
	}
	node.ForEach(func(subNode ast.MlgNodeKind) {
		includeMissingIdentifiersAt(subNode, keyGen)
	})
}

func expandAliasesAtWithAliases(node ast.MlgNodeKind, aliases []ExpAliasSummaryKind) {
	if node == nil {
		return
	}

	node.ForEach(func(subNode ast.MlgNodeKind) {
		for _, alias := range aliases {
			ExpandAliasInline(subNode, alias)
		}
	})
}

func expandAliasesAt(node ast.MlgNodeKind, summaries map[string]SummaryKind) {
	if node == nil {
		return
	}

	switch entry := node.(type) {
	case *ast.DefinesGroup:
		metaId, ok := GetAstMetaId(entry)
		if ok {
			summary, ok := summaries[metaId]
			if ok {
				expandAliasesAtWithAliases(node, summary.GetExpAliasSummaries())
			}
		}
	}
	node.ForEach(func(n ast.MlgNodeKind) {
		expandAliasesAt(n, summaries)
	})
}

func findUsedUnknownSignaturesImpl(node ast.MlgNodeKind, path ast.Path, w *workspace) {
	if node == nil {
		return
	}

	if cmd, ok := node.(*ast.CommandExpression); ok {
		sig := GetSignatureStringFromCommand(*cmd)
		if _, ok := w.signaturesToIds[sig]; !ok {
			w.tracker.Append(frontend.Diagnostic{
				Type:     frontend.Error,
				Origin:   frontend.BackendOrigin,
				Message:  fmt.Sprintf("Unrecognized signature %s", sig),
				Path:     path,
				Position: node.GetCommonMetaData().Start,
			})
		}
	} else if cmd, ok := node.(*ast.InfixCommandExpression); ok {
		sig := GetSignatureStringFromInfixCommand(*cmd)
		if _, ok := w.signaturesToIds[sig]; !ok {
			w.tracker.Append(frontend.Diagnostic{
				Type:     frontend.Error,
				Origin:   frontend.BackendOrigin,
				Message:  fmt.Sprintf("Unrecognized signature %s", sig),
				Path:     path,
				Position: node.GetCommonMetaData().Start,
			})
		}
	}
	node.ForEach(func(subNode ast.MlgNodeKind) {
		findUsedUnknownSignaturesImpl(subNode, path, w)
	})
}

func nameToRenderedName(name string, isVarArg bool) string {
	if isGreekLetter(name) {
		return fmt.Sprintf("\\%s", name)
	}

	reg := regexp.MustCompile(`([a-zA-Z]+)(\d+)`)
	items := reg.FindStringSubmatch(name)
	// format of items: [(full match) (group 1) (group 2)]
	if len(items) == 3 && items[0] == name {
		return fmt.Sprintf("%s_{%s}", items[1], items[2])
	}

	if isVarArg {
		return fmt.Sprintf("{%s}...", name)
	}

	return name
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

func getAllWords(node ast.MlgNodeKind) mlglib.ISet[string] {
	result := mlglib.NewSet[string]()
	getAllWordsImpl(node, result)
	return result
}

func getAllWordsImpl(node ast.MlgNodeKind, result mlglib.ISet[string]) {
	if node == nil {
		return
	}

	switch n := node.(type) {
	case *ast.TextItem:
		for _, item := range tokenize(n.RawText) {
			result.Add(item)
		}
	case *ast.TextBlockItem:
		for _, item := range tokenize(n.Text) {
			result.Add(item)
		}
	}
	node.ForEach(func(subNode ast.MlgNodeKind) {
		getAllWordsImpl(subNode, result)
	})
}

func tokenize(text string) []string {
	result := make([]string, 0)
	parts := strings.Split(text, " ")
	for _, p := range parts {
		result = append(result, strings.ToLower(p))
	}
	return result
}

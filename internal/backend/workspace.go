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
	Paths []ast.Path
}

type PageResponse struct {
	Error       string
	Diagnostics []frontend.Diagnostic
	Document    phase4.Document
}

type CheckResult struct {
	Diagnostics []frontend.Diagnostic
}

type Workspace struct {
	// map paths to path contents
	contents map[ast.Path]string
	// the tracker used to record diagnostics
	tracker *frontend.DiagnosticTracker
	// mapping of paths to phase4 documents
	phase4Root *phase4.Root
	// the root of the phase5 parse tree generated
	astRoot *ast.Root
	// map signatures to ids
	signaturesToIds map[string]string
	// map ids to summaries
	summaries map[string]SummaryType
	// map ids to phase4 parses of top-level entries
	phase4Entries map[string]phase4.TopLevelNodeType
	// map ids to phase5 top-level types
	topLevelEntries map[string]ast.TopLevelItemType
}

func NewWorkspace(contents map[ast.Path]string, tracker *frontend.DiagnosticTracker) *Workspace {
	w := Workspace{
		tracker:         tracker,
		contents:        make(map[ast.Path]string, 0),
		signaturesToIds: make(map[string]string, 0),
		summaries:       make(map[string]SummaryType, 0),
		phase4Entries:   make(map[string]phase4.TopLevelNodeType, 0),
		topLevelEntries: make(map[string]ast.TopLevelItemType, 0),
	}
	w.initialize(contents)
	return &w
}

func (w *Workspace) initialize(contents map[ast.Path]string) {
	w.contents = contents
	w.phase4Root, w.astRoot = ParseRoot(w.contents, w.tracker)
	w.normalizeAst()
	w.populateScopes()
	w.initializeSignaturesToIds()
	w.initializeSummaries()
	w.initializePhase4Entries()
	w.initializeTopLevelEntries()
}

func (w *Workspace) DocumentCount() int {
	return len(w.contents)
}

func (w *Workspace) Paths() []ast.Path {
	paths := make([]ast.Path, 0)
	for k := range w.contents {
		paths = append(paths, k)
	}
	return paths
}

func (w *Workspace) GetDocumentAt(path ast.Path) (phase4.Document, []frontend.Diagnostic) {
	doc := w.astRoot.Documents[path]
	keyToFormulationStr := make(map[int]string, 0)
	for i := range doc.Items {
		w.formulationLikeToString(path, doc.Items[i], keyToFormulationStr)
	}
	phase4Doc := w.phase4Root.Documents[path]
	w.updateFormulationStrings(path, &phase4Doc, keyToFormulationStr)
	return phase4Doc, w.getDiagnosticsForPath(path)
}

func (w *Workspace) formulationLikeToString(path ast.Path,
	node ast.MlgNodeType, keyToFormulationStr map[int]string) {
	if formulation, ok := node.(*ast.Formulation[ast.FormulationNodeType]); ok {
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
		node.ForEach(func(subNode ast.MlgNodeType) {
			w.formulationLikeToString(path, subNode, keyToFormulationStr)
		})
	}
}

func (w *Workspace) formulationToWritten(path ast.Path,
	node ast.Formulation[ast.FormulationNodeType]) string {
	return w.formulationNodeToWritten(path, node.Root)
}

func (w *Workspace) specToWritten(path ast.Path, node ast.Spec) string {
	return w.formulationNodeToWritten(path, node.Root)
}

func (w *Workspace) aliasToWritten(path ast.Path, node ast.Alias) string {
	return w.formulationNodeToWritten(path, node.Root)
}

func (w *Workspace) commandInfixToWritten(path ast.Path,
	node *ast.CommandOperatorTarget) (string, bool) {
	return w.commandToWritten(path, &node.Command)
}

func (w *Workspace) commandToWritten(path ast.Path, node *ast.CommandExpression) (string, bool) {
	sig := GetSignatureStringFromCommand(*node)
	found := false
	if id, ok := w.signaturesToIds[sig]; ok {
		if summary, ok := w.summaries[id]; ok {
			if written, ok := GetResolvedWritten(summary); ok {
				if summaryInput, ok := GetResolvedInput(summary); ok {
					matchResult := Match(node, summaryInput)
					if matchResult.MatchMakesSense && len(matchResult.Messages) == 0 {
						found = true
						nameKeysToWritten := make(map[string]string)
						for name, exp := range matchResult.Mapping {
							text := w.formulationNodeToWritten(path, exp)
							textWithoutParen := strings.TrimSuffix(strings.TrimPrefix(text, "("), ")")
							textWithParen := "(" + textWithoutParen + ")"
							nameKeysToWritten[name+"?"] = text
							nameKeysToWritten[name+"=?"] = text
							nameKeysToWritten[name+"-?"] = textWithoutParen
							nameKeysToWritten[name+"+?"] = textWithParen
						}
						result := written
						for nameKey, text := range nameKeysToWritten {
							result = strings.Replace(result, nameKey, text, -1)
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
			Type:     frontend.Error,
			Origin:   frontend.BackendOrigin,
			Message:  fmt.Sprintf("Unrecognized signature %s", sig),
			Path:     path,
			Position: node.GetCommonMetaData().Start,
		})
	}
	return "", false
}

func (w *Workspace) formulationNodeToWritten(path ast.Path, mlgNode ast.MlgNodeType) string {
	customToCode := func(node ast.MlgNodeType) (string, bool) {
		switch n := node.(type) {
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
			result += " \\coloneqq\\!> "
			result += w.formulationNodeToWritten(path, n.Rhs)
			return result, true
		case *ast.ExpressionColonDashArrowItem:
			result := ""
			result += w.formulationNodeToWritten(path, n.Lhs)
			result += " \\coloneq\\!> "
			result += w.formulationNodeToWritten(path, n.Rhs)
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
			return n.Text, true
		case *ast.EnclosedNonCommandOperatorTarget:
			text := w.formulationNodeToWritten(path, n.Target)
			if len(text) > 0 && unicode.IsLetter(rune(text[0])) {
				return "\\" + text, true
			} else {
				return text, true
			}
		case *ast.CommandExpression:
			return w.commandToWritten(path, n)
		case *ast.CommandOperatorTarget:
			return w.commandInfixToWritten(path, n)
		default:
			return "", false
		}
	}

	return ast.Debug(mlgNode, customToCode)
}

func (w *Workspace) updateFormulationStrings(path ast.Path, node phase4.Node, keyToFormulationStr map[int]string) {
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

func (w *Workspace) getDiagnosticsForPath(path ast.Path) []frontend.Diagnostic {
	result := make([]frontend.Diagnostic, 0)
	for _, diag := range w.tracker.Diagnostics() {
		if diag.Path == path {
			result = append(result, diag)
		}
	}
	return result
}

func (w *Workspace) Check() CheckResult {
	w.findUsedUnknownSignatures()
	for _, path := range w.Paths() {
		// get all of the documents to populate the tracker
		// with any rendering errors
		w.GetDocumentAt(path)
	}
	return CheckResult{
		Diagnostics: w.tracker.Diagnostics(),
	}
}

func (w *Workspace) initializeSignaturesToIds() {
	for _, doc := range w.astRoot.Documents {
		for _, item := range doc.Items {
			sig, sigOk := GetSignatureStringFromTopLevel(item)
			id, idOk := GetAstMetaId(item)
			if sigOk && idOk {
				w.signaturesToIds[sig] = id
			}
		}
	}
}

func (w *Workspace) initializeSummaries() {
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

func (w *Workspace) initializePhase4Entries() {
	for _, doc := range w.phase4Root.Documents {
		for _, item := range doc.Nodes {
			id, idOk := GetPhase4MetaId(item)
			if idOk {
				w.phase4Entries[id] = item
			}
		}
	}
}

func (w *Workspace) initializeTopLevelEntries() {
	for _, doc := range w.astRoot.Documents {
		for _, item := range doc.Items {
			id, idOk := GetAstMetaId(item)
			if idOk {
				w.topLevelEntries[id] = item
			}
		}
	}
}

// Normalizes the given AST node in-place, which includes:
//   - update any place where an identifier is introduced to ensure it has any input and
//     output identifiers specified.  That is if `f(x)` is introduced, it is replaced with
//     something like `f(x) := y` where the output has an identifier.  Also, if `(a, b, c)`
//     is introduced, then it replaced with something like `X := (a, b, c)` where an
//     identifier `X` for the tuple itself is introduced.
//   - Any alias in formulations are expanded so that aliases are not needed anymore.
func (w *Workspace) normalizeAst() {
	w.includeMissingIdentifiers()
	w.expandAliases()
}

func (w *Workspace) populateScopes() {
	PopulateScopes(w.astRoot, w.tracker)
}

// Replace `f(x)` with `f(x) := var'#'` but do not change `f(x) := y`
// Replace `(a, b)` with `var'#' := (a, b)` but do not change `X := (a, b)`
func replaceMissingIdentifier(target ast.Target, keyGen *mlglib.KeyGenerator) ast.Target {
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

func includeMissingIdentifiersInTargets(targets []ast.Target, keyGen *mlglib.KeyGenerator) {
	for i := range targets {
		targets[i] = replaceMissingIdentifier(targets[i], keyGen)
	}
}

func includeMissingIdentifiersAt(node ast.MlgNodeType, keyGen *mlglib.KeyGenerator) {
	if node == nil {
		return
	}

	switch n := node.(type) {
	case *ast.DefinesGroup:
		defines := &n.Defines
		defines.Defines = replaceMissingIdentifier(defines.Defines, keyGen)
		if n.With != nil {
			includeMissingIdentifiersInTargets(n.With.With, keyGen)
		}
		if n.Using != nil {
			includeMissingIdentifiersInTargets(n.Using.Using, keyGen)
		}
	case *ast.DescribesGroup:
		describes := &n.Describes
		describes.Describes = replaceMissingIdentifier(describes.Describes, keyGen)
		if n.With != nil {
			includeMissingIdentifiersInTargets(n.With.With, keyGen)
		}
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
	case *ast.ForAllGroup:
		includeMissingIdentifiersInTargets(n.ForAll.Targets, keyGen)
	case *ast.ExistsGroup:
		includeMissingIdentifiersInTargets(n.Exists.Targets, keyGen)
	case *ast.ExistsUniqueGroup:
		includeMissingIdentifiersInTargets(n.ExistsUnique.Targets, keyGen)
	case *ast.ConnectionGroup:
		if n.Using != nil {
			includeMissingIdentifiersInTargets(n.Using.Using, keyGen)
		}
	}
	node.ForEach(func(subNode ast.MlgNodeType) {
		includeMissingIdentifiersAt(subNode, keyGen)
	})
}

func (w *Workspace) includeMissingIdentifiers() {
	includeMissingIdentifiersAt(w.astRoot, mlglib.NewKeyGenerator())
}

func expandAliasesAtWithAliases(node ast.MlgNodeType, aliases []ExpAliasSummaryType) {
	if node == nil {
		return
	}

	node.ForEach(func(subNode ast.MlgNodeType) {
		for _, alias := range aliases {
			ExpandAliasInline(subNode, alias)
		}
	})
}

func expandAliasesAt(node ast.MlgNodeType, summaries map[string]SummaryType) {
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
	node.ForEach(func(n ast.MlgNodeType) {
		expandAliasesAt(n, summaries)
	})
}

func (w *Workspace) expandAliases() {
	expandAliasesAt(w.astRoot, w.summaries)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (w *Workspace) findUsedUnknownSignatures() {
	for path, doc := range w.astRoot.Documents {
		for _, item := range doc.Items {
			findUsedUnknownSignaturesImpl(item, path, w)
		}
	}
}

func findUsedUnknownSignaturesImpl(node ast.MlgNodeType, path ast.Path, w *Workspace) {
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
	}
	node.ForEach(func(subNode ast.MlgNodeType) {
		findUsedUnknownSignaturesImpl(subNode, path, w)
	})
}

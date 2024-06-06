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

type Workspace struct {
	// map paths to path contents
	contents []PathLabelContent
	// the tracker used to record diagnostics
	diasnosticTracker *frontend.DiagnosticTracker
	nodeTracker       NodeTracker
	writtenResolver   WrittenResolver
	signatureManager  SignatureManager
}

func NewWorkspace(
	contents []PathLabelContent,
	diasnosticTracker *frontend.DiagnosticTracker,
) *Workspace {
	nodeTracker := NewNodeTracker(contents, diasnosticTracker)
	signatureManager := NewSignatureManager(nodeTracker, diasnosticTracker)
	writtenResolver := NewWrittenResolver(nodeTracker, diasnosticTracker)

	w := Workspace{
		contents:          contents,
		diasnosticTracker: diasnosticTracker,
		nodeTracker:       *nodeTracker,
		writtenResolver:   *writtenResolver,
		signatureManager:  *signatureManager,
	}
	w.initialize(contents)
	return &w
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (w *Workspace) DocumentCount() int {
	return len(w.contents)
}

func (w *Workspace) Check() CheckResult {
	w.signatureManager.findUsedUnknownSignatures()
	for _, pair := range w.Paths() {
		// get all of the documents to populate the tracker
		// with any rendering errors
		path := pair.Path
		_, astDoc, _ := w.GetDocumentAt(path)
		CheckRequirements(pair.Path, &astDoc, w.nodeTracker.tracker)
	}
	return CheckResult{
		Diagnostics: w.diasnosticTracker.Diagnostics(),
	}
}

func (w *Workspace) GetUsages() []string {
	return w.signatureManager.GetUsages()
}

func (w *Workspace) Paths() []PathLabelPair {
	result := make([]PathLabelPair, 0)
	for _, pair := range w.contents {
		result = append(result, PathLabelPair{
			Path:  pair.Path,
			Label: pair.Label,
		})
	}
	return result
}

func (w *Workspace) GetDocumentAt(path ast.Path) (phase4.Document, ast.Document, []frontend.Diagnostic) {
	phase4Doc, astDoc := w.nodeTracker.GetDocumentAt(path)
	result := w.writtenResolver.GetRenderedNode(path, &phase4Doc, &astDoc)
	resultDoc, _ := result.(*phase4.Document)
	return *resultDoc, astDoc, w.getDiagnosticsForPath(path)
}

func (w *Workspace) GetEntryById(id string) (phase4.TopLevelNodeKind, error) {
	phase4Entry, astEntry, err := w.nodeTracker.GetEntryById(id)
	if err != nil {
		return nil, err
	}
	result := w.writtenResolver.GetRenderedNode(ast.ToPath(""), phase4Entry, astEntry)
	castResult, _ := result.(phase4.TopLevelNodeKind)
	return castResult, nil
}

func (w *Workspace) GetEntryBySignature(signature string) (phase4.TopLevelNodeKind, error) {
	id, ok := w.nodeTracker.GetIdForSignature(signature)
	if !ok {
		return nil, fmt.Errorf(fmt.Sprintf("Could not get ID for signature %s", signature))
	}
	return w.GetEntryById(id)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (w *Workspace) initialize(contents []PathLabelContent) {
	w.contents = contents
	contentMap := make(map[ast.Path]string)
	for _, pair := range contents {
		if pair.Content != nil {
			contentMap[pair.Path] = *pair.Content
		}
	}
}

func (w *Workspace) getDiagnosticsForPath(path ast.Path) []frontend.Diagnostic {
	result := make([]frontend.Diagnostic, 0)
	for _, diag := range w.diasnosticTracker.Diagnostics() {
		if diag.Path == path {
			result = append(result, diag)
		}
	}
	return result
}

////////////////////////////////////////////////////////////////////////////////////////////////////

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

func includeMissingIdentifiersAt(node ast.MlgNodeKind, keyGen *mlglib.KeyGenerator) {
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

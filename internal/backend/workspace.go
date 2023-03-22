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
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/phase4"
	"mathlingua/internal/server"
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

type ViewResult struct {
	Diagnostics map[ast.Path][]frontend.Diagnostic
	Pages       []WorkspacePageResponse
}

type WorkspacePageResponse struct {
	Path ast.Path
	Page server.PageResponse
}

type CheckResult struct {
	Diagnostics []frontend.Diagnostic
}

type Workspace struct {
	// map paths to path contents
	contents map[ast.Path]string
	// the tracker used to record diagnostics
	tracker *frontend.DiagnosticTracker
	// the root of the phase5 parse tree generated
	root ast.Root
	// map signatures to ids
	signaturesToIds map[string]string
	// map ids to summaries
	summaries map[string]SummaryType
	// map ids to phase4 parses of top-level entries
	phase4Entries map[string]phase4.Group
	// map ids to phase5 top-level types
	topLevelEntries map[string]ast.TopLevelItemType
}

func NewWorkspace(contents map[ast.Path]string) *Workspace {
	w := Workspace{
		contents:        make(map[ast.Path]string, 0),
		signaturesToIds: make(map[string]string, 0),
		summaries:       make(map[string]SummaryType, 0),
		phase4Entries:   make(map[string]phase4.Group, 0),
		topLevelEntries: make(map[string]ast.TopLevelItemType, 0),
	}
	w.initialize(contents)
	return &w
}

func (w *Workspace) initialize(contents map[ast.Path]string) {
	w.contents = contents
	w.tracker = frontend.NewDiagnosticTracker()
	w.root = ParseRoot(w.contents, w.tracker)
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

func (w *Workspace) Check() CheckResult {
	return CheckResult{
		Diagnostics: w.tracker.Diagnostics(),
	}
}

func (w *Workspace) View() ViewResult {
	return ViewResult{}
}

func (w *Workspace) initializeSignaturesToIds() {
}

func (w *Workspace) initializeSummaries() {
}

func (w *Workspace) initializePhase4Entries() {
}

func (w *Workspace) initializeTopLevelEntries() {
}

func (w *Workspace) normalizeAst() {
	Normalize(&w.root, w.tracker)
}

func (w *Workspace) populateScopes() {
	PopulateScopes(&w.root, w.tracker)
}

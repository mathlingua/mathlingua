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
	"mathlingua/internal/mlglib"
)

type NodeTracker struct {
	// the tracker used to record diagnostics
	tracker *frontend.DiagnosticTracker
	// mapping of paths to phase4 documents
	phase4Root *phase4.Root
	// the root of the phase5 parse tree generated
	astRoot *ast.Root
	// map signatures to ids
	signaturesToIds map[string]string
	// map ids to phase4 parses of top-level entries
	phase4Entries map[string]phase4.TopLevelNodeKind
	// map ids to phase5 top-level types
	topLevelEntries map[string]ast.TopLevelItemKind
}

func NewNodeTracker(contents []PathLabelContent, tracker *frontend.DiagnosticTracker) *NodeTracker {
	nt := NodeTracker{
		tracker:         tracker,
		signaturesToIds: make(map[string]string, 0),
		phase4Entries:   make(map[string]phase4.TopLevelNodeKind, 0),
		topLevelEntries: make(map[string]ast.TopLevelItemKind, 0),
	}
	nt.initialize(contents)
	return &nt
}

func (nt *NodeTracker) GetEntryById(id string) (phase4.TopLevelNodeKind, ast.TopLevelItemKind, error) {
	phase4Entry, phase4Ok := nt.phase4Entries[id]
	astEntry, astOk := nt.topLevelEntries[id]
	if !phase4Ok || !astOk {
		return nil, nil, fmt.Errorf("An entry with id %s does not exist", id)
	}
	return phase4Entry, astEntry, nil
}

func (nt *NodeTracker) GetEntryBySignature(signature string) (phase4.TopLevelNodeKind, ast.TopLevelItemKind, error) {
	id, ok := nt.signaturesToIds[signature]
	if !ok {
		return nil, nil, fmt.Errorf("Could not determine the id for signature %s", signature)
	}
	return nt.GetEntryById(id)
}

func (nt *NodeTracker) GetDocumentAt(path ast.Path) (phase4.Document, ast.Document) {
	phase4Doc := nt.phase4Root.Documents[path]
	astDoc := nt.astRoot.Documents[path]
	return phase4Doc, astDoc
}

func (nt *NodeTracker) GetIdForSignature(signature string) (string, bool) {
	id, ok := nt.signaturesToIds[signature]
	return id, ok
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func (nt *NodeTracker) initialize(contents []PathLabelContent) {
	contentMap := make(map[ast.Path]string)
	for _, pair := range contents {
		if pair.Content != nil {
			contentMap[pair.Path] = *pair.Content
		}
	}
	nt.phase4Root, nt.astRoot = ParseRoot(contentMap, nt.tracker)
	nt.normalizeAst()
	nt.initializeSignaturesToIds()
	nt.initializePhase4Entries()
	nt.initializeTopLevelEntries()
}

func (nt *NodeTracker) initializePhase4Entries() {
	for _, doc := range nt.phase4Root.Documents {
		for _, item := range doc.Nodes {
			id, idOk := GetPhase4MetaId(item)
			if idOk {
				switch n := item.(type) {
				case *phase4.TextBlock:
					n.MetaData.Id = id
				case *phase4.Group:
					n.MetaData.Id = id
				}
				nt.phase4Entries[id] = item
			}
		}
	}
}

func (w *NodeTracker) initializeSignaturesToIds() {
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

func (w *NodeTracker) initializeTopLevelEntries() {
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
func (nt *NodeTracker) normalizeAst() {
	nt.includeMissingIdentifiers()
	// TODO: expand aliases
}

func (nt *NodeTracker) populateScopes() {
}

func (nt *NodeTracker) includeMissingIdentifiers() {
	includeMissingIdentifiersAt(nt.astRoot, mlglib.NewKeyGenerator())
}

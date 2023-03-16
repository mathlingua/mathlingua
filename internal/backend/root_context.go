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
)

func NewRootContext(root *ast.Root, tracker frontend.IDiagnosticTracker) ast.IContext {
	ctx := rootContext{
		root:            root,
		signatureToNode: make(map[string]ast.MlgNodeType, 0),
	}
	ctx.initialize(tracker)
	return &ctx
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type rootContext struct {
	root            *ast.Root
	signatureToNode map[string]ast.MlgNodeType
}

func (c *rootContext) initialize(tracker frontend.IDiagnosticTracker) {
	for path, doc := range c.root.Documents {
		for _, item := range doc.Items {
			switch n := item.(type) {
			case *ast.DescribesGroup:
				c.storeSignature(n.Id, n, path, tracker)
			case *ast.DefinesGroup:
				c.storeSignature(n.Id, n, path, tracker)
			case *ast.StatesGroup:
				c.storeSignature(n.Id, n, path, tracker)
			}
		}
	}
}

func (c *rootContext) storeSignature(id ast.IdItem, n ast.MlgNodeType, path ast.Path,
	tracker frontend.IDiagnosticTracker) {
	if sig, ok := GetSignatureStringFromId(id); ok {
		if _, ok := c.signatureToNode[sig]; ok {
			tracker.Append(frontend.Diagnostic{
				Path:     path,
				Type:     frontend.Error,
				Origin:   frontend.BackendOrigin,
				Message:  fmt.Sprintf("Duplicate defined signature: %s", sig),
				Position: n.GetCommonMetaData().Start,
			})
			return
		}
		c.signatureToNode[sig] = n
		return
	}

	tracker.Append(frontend.Diagnostic{
		Path:     path,
		Type:     frontend.Error,
		Origin:   frontend.BackendOrigin,
		Message:  fmt.Sprintf("Could not determine signature"),
		Position: n.GetCommonMetaData().Start,
	})
}

func (c *rootContext) GetParent() ast.IContext {
	return nil
}

func (c *rootContext) SetParent(parent ast.IContext) {
}

func (c *rootContext) IsSubTypeOf(from string, to string) bool {
	// TODO: implement this
	return false
}

func (c *rootContext) IsViewableAs(from string, to string) bool {
	// TODO: implement this
	return false
}

func (c *rootContext) ResolveType(arg ast.ResolveTypeArg) ast.ResolvedType {
	// TODO: implement this
	return ast.ResolvedType{}
}

func (c *rootContext) ResolveTypeForm(arg ast.ResolveTypeFormArg) ast.IResolvedTypeForm {
	// TODO: implement this
	return ast.ResolvedFunctionTypeForm{}
}

func (c *rootContext) ResolveExpression(arg ast.ResolveExpressionArg) ast.ExpressionType {
	// TODO: implement this
	return nil
}

func (c *rootContext) GetWrittenAs(exp ast.ExpressionType, scope ast.IScope) {
}

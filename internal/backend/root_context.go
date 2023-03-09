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
)

func NewRootContext(root *ast.Root) ast.Context {
	ctx := rootContext{
		root:            root,
		signatureToNode: make(map[string]ast.MlgNodeType, 0),
	}
	ctx.initialize()
	return &ctx
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type rootContext struct {
	root            *ast.Root
	signatureToNode map[string]ast.MlgNodeType
}

func (c *rootContext) initialize() {
	for _, doc := range c.root.Documents {
		for _, item := range doc.Items {
			switch n := item.(type) {
			case *ast.DescribesGroup:
				// TODO: properly handle error conditions
				if sig, ok := GetSignatureStringFromId(n.Id); ok {
					c.signatureToNode[sig] = n
				} else {
					fmt.Println("Could not determine the signature for:")
					fmt.Println(ast.DebugStructuralNode(n))
				}
			case *ast.DefinesGroup:
				// TODO: properly handle error conditions
				if sig, ok := GetSignatureStringFromId(n.Id); ok {
					c.signatureToNode[sig] = n
				} else {
					fmt.Println("Could not determine the signature for:")
					fmt.Println(ast.DebugStructuralNode(n))
				}
			case *ast.StatesGroup:
				// TODO: properly handle error conditions
				if sig, ok := GetSignatureStringFromId(n.Id); ok {
					c.signatureToNode[sig] = n
				} else {
					fmt.Println("Could not determine the signature for:")
					fmt.Println(ast.DebugStructuralNode(n))
				}
			}
		}
	}
}

func (c *rootContext) GetParent() (*ast.Context, bool) {
	return nil, false
}

func (c *rootContext) SetParent(parent *ast.Context) {
}

func (c *rootContext) IsSubTypeOf(from string, to string) bool {
	// TODO: implement this
	return false
}

func (c *rootContext) IsViewableAs(from string, to string) bool {
	// TODO: implement this
	return false
}

func (c *rootContext) PopulateFromSpecAlias(nameType ast.ResolvedType,
	specName string, target ast.ExpressionType,
	scope *ast.Scope) {
}

func (c *rootContext) PopulateFromIs(is *ast.IsExpression, scope *ast.Scope) {
}

func (c *rootContext) GetWrittenAs(exp *ast.ExpressionType, scope *ast.Scope) {
}

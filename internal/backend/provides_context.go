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

import "mathlingua/internal/ast"

func NewProvidesContext(parent ast.IContext, providable ast.ProvidableNodeType) ast.IContext {
	return &providesContext{
		parent:     parent,
		providable: providable,
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type providesContext struct {
	parent     ast.IContext
	providable ast.ProvidableNodeType
}

func (c *providesContext) GetParent() ast.IContext {
	return c.parent
}

func (c *providesContext) SetParent(parent ast.IContext) {
	c.parent = parent
}

func (c *providesContext) IsSubTypeOf(from string, to string) bool {
	// TODO: implement this
	return false
}

func (c *providesContext) IsViewableAs(from string, to string) bool {
	// TODO: implement this
	return false
}

func (c *providesContext) PopulateFromSpecAlias(nameType ast.ResolvedType,
	specName string, target ast.ExpressionType,
	scope ast.IScope) {
}

func (c *providesContext) PopulateFromIs(is *ast.IsExpression, scope ast.IScope) {
}

func (c *providesContext) ResolveType(arg ast.ResolveTypeArg) ast.ResolvedType {
	// TODO: implement this
	return ast.ResolvedType{}
}

func (c *providesContext) ResolveTypeForm(arg ast.ResolveTypeFormArg) ast.IResolvedTypeForm {
	// TODO: implement this
	return ast.ResolvedFunctionTypeForm{}
}

func (c *providesContext) ResolveExpression(arg ast.ResolveExpressionArg) ast.ExpressionType {
	// TODO: implement this
	return nil
}

func (c *providesContext) GetWrittenAs(exp *ast.ExpressionType, scope ast.IScope) {
}

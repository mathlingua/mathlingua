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

func PopulateContexts(root *ast.Root) {
	rootContext := NewRootContext(root)
	root.CommonMetaData.Context = rootContext

	root.ForEach(func(subNode ast.MlgNodeType) {
		populateContextsImpl(subNode, rootContext)
	})
}

func populateContextsImpl(node ast.MlgNodeType, parentContext ast.Context) {
	switch n := node.(type) {
	case ast.ProvidableNodeType:
		providesContext := NewProvidesContext(parentContext, n)
		n.GetCommonMetaData().Context = providesContext
		n.ForEach(func(subNode ast.MlgNodeType) {
			populateContextsImpl(subNode, providesContext)
		})
	default:
		n.ForEach(func(subNode ast.MlgNodeType) {
			populateContextsImpl(subNode, parentContext)
		})
	}
}

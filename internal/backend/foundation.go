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

type IFoundation interface {
	ResolveDefinesSummary(context Context, summary DefinesSummary) DefinesSummary
	ResolveDescribesSummary(context Context, summary DescribesSummary) DescribesSummary
	ResolveStatesSummary(context Context, summary StatesSummary) StatesSummary
	// Resolve all of the identifiers in all scopes so that no identifier infos
	// contain spec aliases.  From this, all operators are resolved to be absolute
	// and written information is stored in each sub-node of the node to be used
	// to render the node.
	ResolveForRender(node ast.MlgNodeType)
}

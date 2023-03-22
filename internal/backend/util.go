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

func GetId(node ast.TopLevelItemType) (string, bool) {
	switch tl := node.(type) {
	case *ast.DefinesGroup:
		metaId := tl.MetaId
		if metaId == nil {
			return "", false
		}
		return metaId.Id.RawText, true
	case *ast.DescribesGroup:
		metaId := tl.MetaId
		if metaId == nil {
			return "", false
		}
		return metaId.Id.RawText, true
	case *ast.StatesGroup:
		metaId := tl.MetaId
		if metaId == nil {
			return "", false
		}
		return metaId.Id.RawText, true
	case *ast.AxiomGroup:
		metaId := tl.MetaId
		if metaId == nil {
			return "", false
		}
		return metaId.Id.RawText, true
	case *ast.ConjectureGroup:
		metaId := tl.MetaId
		if metaId == nil {
			return "", false
		}
		return metaId.Id.RawText, true
	case *ast.TheoremGroup:
		metaId := tl.MetaId
		if metaId == nil {
			return "", false
		}
		return metaId.Id.RawText, true
	case *ast.SpecifyGroup:
		metaId := tl.MetaId
		if metaId == nil {
			return "", false
		}
		return metaId.Id.RawText, true
	case *ast.TopicGroup:
		metaId := tl.MetaId
		if metaId == nil {
			return "", false
		}
		return metaId.Id.RawText, true
	case *ast.PersonGroup:
		metaId := tl.MetaId
		if metaId == nil {
			return "", false
		}
		return metaId.Id.RawText, true
	case *ast.ResourceGroup:
		metaId := tl.MetaId
		if metaId == nil {
			return "", false
		}
		return metaId.Id.RawText, true
	case *ast.ProofGroup:
		metaId := tl.MetaId
		if metaId == nil {
			return "", false
		}
		return metaId.Id.RawText, true
	default:
		return "", false
	}
}

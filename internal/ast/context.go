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

package ast

type Context interface {
	GetParent() (*Context, bool)
	SetParent(parent *Context)
	IsSubTypeOf(from string, to string) bool
	IsViewableAs(from string, to string) bool
	PopulateFromSpecAlias(nameType ResolvedType,
		specName string, target ExpressionType,
		scope *Scope)
	PopulateFromIs(is *IsExpression, scope *Scope)
	GetWrittenAs(exp *ExpressionType, scope *Scope)
	AddDefines(defines *DefinesGroup)
	AddDescribes(describes *DescribesGroup)
	AddStates(states *StatesSection)
}

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

type Scope interface {
	GetName() (NameInfo, bool)
}

type NameInfo struct {
	IsPlaceholder   bool
	IsNumberLiteral bool
	Type            ResolvedType
}

type GlobalScope struct {
}

type SpecifyScope struct {
}

type ForAllScope struct {
}

type ExistsScope struct {
}

type ExistsUniqueScope struct {
}

type GivenScope struct {
}

type DefinesScope struct {
}

type DescribesScope struct {
}

type StatesScope struct {
}

type TheoremScope struct {
}

type AxiomScope struct {
}

type ConjectureScope struct {
}

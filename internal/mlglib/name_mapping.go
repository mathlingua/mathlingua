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

package mlglib

type NameMapping struct {
	fromToToNames map[string]string
	toToFromNames map[string]string
}

func NewNameMapping() *NameMapping {
	return &NameMapping{
		fromToToNames: make(map[string]string, 0),
		toToFromNames: make(map[string]string, 0),
	}
}

func (nm *NameMapping) AddMapping(fromName string, toName string) {
	nm.fromToToNames[fromName] = toName
	nm.toToFromNames[toName] = fromName
}

func (nm *NameMapping) GetFromName(toName string) string {
	return nm.toToFromNames[toName]
}

func (nm *NameMapping) GetToName(fromName string) string {
	return nm.fromToToNames[fromName]
}

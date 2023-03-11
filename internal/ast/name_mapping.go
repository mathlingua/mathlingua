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

type NameMapping interface {
	AddMapping(callSiteName string, defSiteName string)
	GetCallSiteName(defSiteName string) string
	GetDefSiteName(callSiteName string) string
}

func NewNameMapping() NameMapping {
	return &nameMapping{
		callSiteToDefSiteNames: make(map[string]string, 0),
		defSiteToCallSiteNames: make(map[string]string, 0),
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

type nameMapping struct {
	callSiteToDefSiteNames map[string]string
	defSiteToCallSiteNames map[string]string
}

func (nm *nameMapping) AddMapping(callSiteName string, defSiteName string) {
	nm.callSiteToDefSiteNames[callSiteName] = defSiteName
	nm.defSiteToCallSiteNames[defSiteName] = callSiteName
}

func (nm *nameMapping) GetCallSiteName(defSiteName string) string {
	return nm.defSiteToCallSiteNames[defSiteName]
}

func (nm *nameMapping) GetDefSiteName(callSiteName string) string {
	return nm.callSiteToDefSiteNames[callSiteName]
}

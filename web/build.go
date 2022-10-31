/*
 * Copyright 2022 Dominic Kramer
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

package web

import (
	"embed"
	"io/fs"
	"net/http"
	"path"
)

//go:embed build/*
var Assets embed.FS

type AssetsFS struct{}

func (fs AssetsFS) Open(name string) (fs.File, error) {
	return Assets.Open(path.Join("build", name))
}

type AssetHandler struct{}

func (h AssetHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	http.FileServer(http.FS(AssetsFS{})).ServeHTTP(w, r)
}

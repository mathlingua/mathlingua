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

package backend

import (
	"encoding/json"
	"fmt"
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/web"
	"net/http"

	"github.com/gorilla/mux"
)

func StartServer() {
	workspace, diagnostics := NewWorkspaceFromPaths([]string{"."})
	for _, diag := range diagnostics {
		if diag.Type == frontend.Error {
			fmt.Printf("%s (%d, %d)\n%s",
				diag.Path, diag.Position.Row+1, diag.Position.Column+1,
				diag.Message)
		} else {
			fmt.Printf("%s (%d, %d)\n%s",
				diag.Path, diag.Position.Row+1, diag.Position.Column+1,
				diag.Message)
		}
	}

	router := mux.NewRouter()
	router.Use(handleCors)
	router.HandleFunc("/api/paths", func(w http.ResponseWriter, r *http.Request) {
		paths(workspace, w, r)
	}).Methods("GET")
	router.HandleFunc("/api/page", func(w http.ResponseWriter, r *http.Request) {
		page(workspace, w, r)
	}).Methods("GET")
	router.PathPrefix("/").Handler(web.AssetHandler{})

	fmt.Println("Visit http://localhost:8080 to view your documents")
	if err := http.ListenAndServe(":8080", router); err != nil {
		fmt.Println(err.Error())
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func paths(workspace *Workspace, writer http.ResponseWriter, request *http.Request) {
	setJsonContentType(writer)
	resp := PathsResponse{
		Paths: workspace.Paths(),
	}
	writeResponse(writer, &resp)
}

func page(workspace *Workspace, writer http.ResponseWriter, request *http.Request) {
	setJsonContentType(writer)
	path := request.URL.Query().Get("path")
	doc, diagnostics := workspace.GetDocumentAt(ast.ToPath(path))

	resp := PageResponse{
		Diagnostics: diagnostics,
		Document:    doc,
	}

	writeResponse(writer, &resp)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func setJsonContentType(writer http.ResponseWriter) {
	writer.Header().Set("Content-Type", "application/json")
}

func writeResponse[T any](writer http.ResponseWriter, resp *T) {
	if err := json.NewEncoder(writer).Encode(&resp); err != nil {
		fmt.Println(err)
		writer.WriteHeader(http.StatusInternalServerError)
	}
}

func handleCors(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Access-Control-Allow-Origin", "*")
		w.Header().Set("Access-Control-Allow-Methods", "GET, POST, OPTIONS, PUT")
		w.Header().Set("Access-Control-Allow-Headers", "Origin, Content-Type")

		if r.Method == "OPTIONS" {
			return
		}

		next.ServeHTTP(w, r)
	})
}
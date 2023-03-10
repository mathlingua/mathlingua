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

package server

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"mathlingua/internal/ast"
	"mathlingua/internal/frontend"
	"mathlingua/internal/frontend/phase1"
	"mathlingua/internal/frontend/phase2"
	"mathlingua/internal/frontend/phase3"
	"mathlingua/internal/frontend/phase4"
	"mathlingua/web"
	"net/http"
	"os"
	"path/filepath"
	"strings"

	"github.com/gorilla/mux"
)

func Start() {
	router := mux.NewRouter()
	router.Use(handleCors)
	router.HandleFunc("/api/paths", paths).Methods("GET")
	router.HandleFunc("/api/page", page).Methods("GET")
	router.PathPrefix("/").Handler(web.AssetHandler{})

	fmt.Println("Visit http://localhost:8080 to view your documents")
	if err := http.ListenAndServe(":8080", router); err != nil {
		fmt.Println(err.Error())
	}
}

type PathsResponse struct {
	Error string
	Paths []string
}

func paths(writer http.ResponseWriter, request *http.Request) {
	setJsonContentType(writer)
	resp := PathsResponse{
		Paths: findAllMathlinguaFiles("."),
	}
	writeResponse(writer, &resp)
}

type PageResponse struct {
	Error       string
	Diagnostics []frontend.Diagnostic
	Document    phase4.Document
}

func findAllMathlinguaFiles(dir string) []string {
	children, err := ioutil.ReadDir(dir)
	if err != nil {
		return nil
	}

	result := make([]string, 0)
	for _, child := range children {
		name := child.Name()
		if strings.HasPrefix(name, ".") {
			// skip hidden dirs
			continue
		}
		fullpath := filepath.Join(dir, name)
		if child.IsDir() {
			result = append(result, findAllMathlinguaFiles(fullpath)...)
		} else if strings.HasSuffix(child.Name(), ".math") {
			result = append(result, fullpath)
		}
	}

	return result
}

func page(writer http.ResponseWriter, request *http.Request) {
	setJsonContentType(writer)
	path := request.URL.Query().Get("path")

	bytes, err := os.ReadFile(path)
	if err != nil {
		resp := PageResponse{
			Error: err.Error(),
		}
		writeResponse(writer, &resp)
		return
	}

	input := string(bytes)
	doc, diagnostics := parse(input, ast.ToPath(path))

	resp := PageResponse{
		Diagnostics: diagnostics,
		Document:    doc,
	}

	writeResponse(writer, &resp)
}

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

func parse(text string, path ast.Path) (phase4.Document, []frontend.Diagnostic) {
	tracker := frontend.NewDiagnosticTracker()

	lexer1 := phase1.NewLexer(text, path, tracker)
	lexer2 := phase2.NewLexer(lexer1, path, tracker)
	lexer3 := phase3.NewLexer(lexer2, path, tracker)

	doc := phase4.Parse(lexer3, path, tracker)

	return doc, tracker.Diagnostics()
}

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
	"embed"
	"encoding/json"
	"fmt"
	"mathlingua/internal/ast"
	"mathlingua/internal/config"
	"mathlingua/internal/frontend"
	"mathlingua/web"
	"net/http"
	"path"
	"strings"

	"github.com/gorilla/mux"
)

func StartServer(port int, conf config.MlgConfig) {
	workspace := initWorkspace()

	router := mux.NewRouter()
	router.Use(handleCors)
	router.HandleFunc("/api/paths", func(w http.ResponseWriter, r *http.Request) {
		paths(workspace, w, r)
	}).Methods("GET")
	router.HandleFunc("/api/page", func(w http.ResponseWriter, r *http.Request) {
		page(workspace, w, r)
	}).Methods("GET")
	router.HandleFunc("/api/entry/id/{id}", func(w http.ResponseWriter, r *http.Request) {
		entryById(workspace, w, r)
	}).Methods("GET")
	router.HandleFunc("/api/entry/signature/{signature}",
		func(w http.ResponseWriter, r *http.Request) {
			entryBySignature(workspace, w, r)
		}).Methods("GET")
	router.PathPrefix("/").HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		// if the URL path cannot be determined, or corresponds to a static asset,
		// then let the default handler handle the request
		if r.URL == nil || strings.HasPrefix(r.URL.Path, "/static") {
			web.AssetHandler{}.ServeHTTP(w, r)
			return
		}

		workspace = initWorkspace()

		// otherwise fall back to responding with the contents of index.html.
		// This is needed to support client-side route handling.  Also, the
		// content of index.html is modified (title, description, etc.) based
		// on the user's configuration.
		bytes, err := embed.FS.ReadFile(web.Assets, path.Join("build", "index.html"))
		if err != nil {
			// report an error to the console, but don't fail loading the index.html
			// page without any customization
			fmt.Printf("Could not customize the docs based on the Mlg config: %s", err)
			web.AssetHandler{}.ServeHTTP(w, r)
			return
		}

		content := string(bytes)

		// set the title
		content = strings.Replace(content, "<title></title>",
			fmt.Sprintf("<title>%s</title>", conf.View.Title), 1)

		// set the description
		rawDescription := strings.ReplaceAll(conf.View.Description, "\"", "\\\"")
		descriptionHtml := fmt.Sprintf(
			"<meta name=\"description\" content=\"%s\"/>", rawDescription)
		content = strings.Replace(content,
			"<meta name=\"description\" content=\"\"/>", descriptionHtml, 1)

		// set the keywords
		rawKeywords := strings.ReplaceAll(conf.View.Keywords, "\"", "\\\"")
		keywordsHtml := fmt.Sprintf("<meta name=\"keywords\" content=\"%s\"/>", rawKeywords)
		content = strings.Replace(content,
			"<meta name=\"keywords\" content=\"\"/>", keywordsHtml, 1)

		w.WriteHeader(http.StatusOK)
		if _, err = w.Write([]byte(content)); err != nil {
			fmt.Printf("Failed to serve index.html: %s\n", err)
		}
	})

	fmt.Printf("Visit http://localhost:%d to view your documents\n", port)
	if err := http.ListenAndServe(fmt.Sprintf(":%d", port), router); err != nil {
		fmt.Println(err.Error())
	}
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func initWorkspace() IWorkspace {
	tracker := frontend.NewDiagnosticTracker()
	tracker.AddListener(func(diag frontend.Diagnostic) {
		fmt.Println(diag.String())
	})
	workspace, diagnostics := NewWorkspaceFromPaths([]string{"."}, tracker)
	for _, diag := range diagnostics {
		tracker.Append(diag)
	}
	return workspace
}

func paths(workspace IWorkspace, writer http.ResponseWriter, request *http.Request) {
	setJsonContentKind(writer)
	resp := PathsResponse{
		Paths: workspace.Paths(),
	}
	writeResponse(writer, &resp)
}

func page(workspace IWorkspace, writer http.ResponseWriter, request *http.Request) {
	setJsonContentKind(writer)
	path := request.URL.Query().Get("path")
	doc, diagnostics := workspace.GetDocumentAt(ast.ToPath(path))

	resp := PageResponse{
		Diagnostics: diagnostics,
		Document:    doc,
	}

	writeResponse(writer, &resp)
}

func entryById(workspace IWorkspace, writer http.ResponseWriter, request *http.Request) {
	setJsonContentKind(writer)

	id, ok := mux.Vars(request)["id"]
	if !ok {
		resp := EntryResponse{
			Error: "id not specified",
			Entry: nil,
		}
		writeResponse(writer, &resp)
		return
	}

	entry, err := workspace.GetEntryById(id)

	errStr := ""
	if err != nil {
		errStr = err.Error()
	}

	resp := EntryResponse{
		Error: errStr,
		Entry: entry,
	}

	writeResponse(writer, &resp)
}

func entryBySignature(workspace IWorkspace, writer http.ResponseWriter, request *http.Request) {
	setJsonContentKind(writer)

	signature, ok := mux.Vars(request)["signature"]
	if !ok {
		resp := EntryResponse{
			Error: "signature not specified",
			Entry: nil,
		}
		writeResponse(writer, &resp)
		return
	}

	entry, err := workspace.GetEntryBySignature(signature)

	errStr := ""
	if err != nil {
		errStr = err.Error()
	}

	resp := EntryResponse{
		Error: errStr,
		Entry: entry,
	}

	writeResponse(writer, &resp)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

func setJsonContentKind(writer http.ResponseWriter) {
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

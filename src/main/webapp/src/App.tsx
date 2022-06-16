import React from 'react';
import Editor, { OnMount, Monaco } from "@monaco-editor/react";

export function App() {

  const onMount: OnMount = (editor, monaco: any) => {
    registerCompletionProvider(monaco);
    registerValidator(monaco);
  };

  return (
    <Editor
       height='100vh'
       defaultLanguage='yaml'
       options={{
          lineNumbers: 'on',
          minimap: {
            enabled: false
          },
          renderIndentGuides: false
       }}
       value={''}
       onMount={onMount}
     />
  );
}

function registerCompletionProvider(monaco: any) {
  monaco.languages.registerCompletionItemProvider('yaml', {
    provideCompletionItems: (model: any, position: any, token: any) => {
      return {
        suggestions: [{
          label: 'some-label',
          kind: monaco.languages.CompletionItemKind.Text,
          insertText: 'some-text',
        }]
      };
    }
  });
}

function registerValidator(monaco: any) {
  const models = monaco.editor.getModels();
  for (const model of models) {
    const validate = () => {
      const markers = [{
        severity: monaco.MarkerSeverity.Error,
        startLineNumber: 0,
        endLineNumber: 0,
        startColumn: 0,
        endColumn: 1,
        message: 'text-error-message',
      }];
      monaco.editor.setModelMarkers(model, 'yaml', markers);
    };

    let handle: NodeJS.Timeout | null = null;
    model.onDidChangeContent(() => {
      if (handle) {
        clearTimeout(handle);
      }
      handle = setTimeout(validate, 500);
    });
    validate();
  }
}

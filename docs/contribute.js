/*
 * Copyright 2019 Google LLC
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

'use strict';

const editor = ace.edit("editor");
editor.setOptions({
  highlightActiveLine: false,
  useSoftTabs: true,
  tabSize: 2,
  showPrintMargin: false,
  showFoldWidgets: false,
  enableBasicAutocompletion: true,
  enableLiveAutocompletion: false
});
editor.session.setMode("ace/mode/yaml");

editor.on('change', () => {
    const ml = new bundle.mathlingua.common.MathLingua();
    const result = ml['parse_61zpoe$'](editor.getValue());
    if (result.errors.length === 0) {
        document.getElementById('output').innerHTML = 'Processed Successfully';
    }
    else {
        let errorText = '';
        const allMessages = new Set();
        for (const item of result.errors['array_hd7ov6$_0']) {
            const message = item['message_8yp7un$_0'];
            if (!allMessages.has(message)) {
              errorText += 'Error(' + (item.row + 1) + ', ' + (item.column + 1) + '): ' +
                message.replace(/\n/g, '<br/>') + '<br/>';
              allMessages.add(message);
            }
        }
        document.getElementById('error').innerHTML = errorText;
    }
});

const CHALK_TALK_COMPLETIONS = [
    { name: 'Result:', value: 'Result:\nAlias:\nMetadata:' },
    { name: 'Axiom:', value: 'Axiom:\nAlias:\nMetadata:' },
    { name: 'Conjecture:', value: 'Conjecture:\nAlias:\nMetadata:' },
    { name: 'Defines:', value: 'Defines:\nassuming:\nmeans:\nAlias:\nMetadata:' },
    { name: 'Refines:', value: 'Refines:\nassuming:\nmeans:\nAlias:\nMetadata:' },
    { name: 'Represents:', value: 'Represents:\nassuming:\nthat:\nAlias:\nMetadata:' },
    { name: 'for:', value: 'for:\nwhere:\nthen:' },
    { name: 'exists:', value: 'exists:\nsuchThat:' },
    { name: 'if:', value: 'if:\nthen:' },
    { name: 'iff:', value: 'iff:\nthen:' },
    { name: 'not:', value: 'not:' },
    { name: 'or:', value: 'or:' }
];

const langTools = ace.require("ace/ext/language_tools");
langTools.setCompleters();
langTools.addCompleter({
   getCompletions: (editor, session, pos, prefix, callback) => {
       if (prefix.length === 0) {
           return callback(null, []);
       }
       const column = pos.column;
       let indent = '\n';
       for (let i = 0; i < (column - prefix.length); i++) {
           indent += ' ';
       }
       callback(null, CHALK_TALK_COMPLETIONS.map(item => {
           return {
               name: item.name,
               value: item.value.replace(/\n/g, indent)
           };
       }));
   }
});

const link = document.getElementById('email');
link.onclick = function() {
    this.href = 'mailto:DominicKramer@gmail.com?subject=MathLingua%20Contribution&body=';
    this.href += encodeURIComponent('I would like to contribute the ' +
        'following to the MathLingua project:\n\n');
    this.href += encodeURIComponent(editor.getValue());
};

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

let data;
requirejs(['data'], (d) => {
  data = d.getData();
});

function search(keywordText) {
  const resultsNode = document.getElementById('results');

  const keywords = keywordText.replace(/:/g, ' ')
                              .split(' ')
                              .filter(item => item.length > 0)
                              .map(item => item.toLowerCase());

  const found = [];
  for (const item of data) {
    const itemKeywords = new Set(item.keywords);
    var foundAll = true;
    for (const kw of keywords) {
      if (!itemKeywords.has(kw)) {
        foundAll = false;
        break;
      }
    }

    if (foundAll) {
      found.push(item.text);
    }
  }

  while (resultsNode.firstChild) {
    resultsNode.removeChild(resultsNode.firstChild);
  }

  for (const text of found) {
    resultsNode.appendChild(createResultDiv(text));
  }
}

function createResultDiv(text) {
  const codeBlock = document.createElement('code');
  codeBlock.className = 'yaml';
  codeBlock.appendChild(document.createTextNode(text));

  const paddedPre = document.createElement('pre');
  paddedPre.className = 'padded';
  paddedPre.appendChild(codeBlock);

  const centeredDiv = document.createElement('div');
  centeredDiv.className = 'centered';
  centeredDiv.appendChild(paddedPre);

  hljs.highlightBlock(codeBlock);

  return centeredDiv;
}

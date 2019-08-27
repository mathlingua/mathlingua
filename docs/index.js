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

function filter(keywordText) {
  const resultsNode = document.getElementById('results');

  const keywords = keywordText.replace(/:/g, ' ')
                              .split(' ')
                              .filter(item => item.length > 0)
                              .map(item => item.toLowerCase());

  const found = [];
  for (const item of window.MATHLINGUA_DATA) {
    const itemKeywords = new Set(item.keywords);
    var foundAll = true;
    for (const kw of keywords) {
      if (!itemKeywords.has(kw)) {
        foundAll = false;
        break;
      }
    }

    if (foundAll) {
      found.push({
        text: item.text,
        href: item.href,
        mobileHref: item.mobileHref
      });
    }
  }

  while (resultsNode.firstChild) {
    resultsNode.removeChild(resultsNode.firstChild);
  }

  if (found.length === 0) {
    resultsNode.appendChild(createResultDiv('No results found', undefined, undefined));
  }
  else {
    for (const res of found) {
      resultsNode.appendChild(createResultDiv(res.text, res.href, res.mobileHref));
    }
  }
}

function isOnMobile() {
    return (typeof window.orientation !== 'undefined') ||
           (navigator.userAgent.toLowerCase().indexOf('iemobile') !== -1);
}

function createResultDiv(text, href, mobileHref) {
  const codeBlock = document.createElement('code');
  codeBlock.className = 'yaml';
  codeBlock.appendChild(document.createTextNode(text));

  const paddedPre = document.createElement('pre');
  paddedPre.className = 'padded';
  paddedPre.appendChild(codeBlock);

  const centeredDiv = document.createElement('div');
  centeredDiv.className = 'centered';
  centeredDiv.appendChild(paddedPre);

  const anchor = document.createElement('a');
  anchor.className = 'plain';
  anchor.setAttribute('target', '_');
  const onMobile = isOnMobile();
  if (!onMobile && href) {
    anchor.setAttribute('href', href);
  }
  else if (onMobile && mobileHref) {
    anchor.setAttribute('href', mobileHref);
  }
  anchor.appendChild(centeredDiv);

  hljs.highlightBlock(codeBlock);

  return anchor;
}

import './ChalkTalkReferencePanel.css';

import { ref } from '../Reference';
import { getDocItems } from './ChalkTalkReference';

export interface ChalkTalkReferencePanelProps {
  onLoad: () => void;
}

export const ChalkTalkReferencePanel = (props: ChalkTalkReferencePanelProps) => {
  props.onLoad();

  let content = `<div class='chalkTalkReferenceTitle'>Structural Language Reference</div>`;
  content += '<p><b>Note:</b> This document is still a work in progress and is not complete.</p>';

  for (const item of getDocItems()) {
    content += `<div class='chalkTalkReferenceName'>&lt;${item.name}&gt;</div>\n`;

    content += `<div class='chalkTalkReferenceHeader'>Specification</div>\n`;
    content += `<pre class='chalkTalkReferenceMono'>\n`;
    let preContent = '';
    const anyItem = item as any;
    if (anyItem.form) {
      preContent += `${ref(item.name)} ::= ${anyItem.form}\n`;
    } else if (anyItem.allowed) {
      const prefix = `${ref(item.name)} :`;
      let indent = '';
      for (let i = 0; i < prefix.length - 1; i++) {
        indent += ' ';
      }
      if (anyItem.allowed.length > 0) {
        preContent += prefix + ':= ' + anyItem.allowed[0] + '\n';
        for (let i = 1; i < anyItem.allowed.length; i++) {
          preContent += indent + '|   ' + anyItem.allowed[i] + '\n';
        }
      }
    }
    content +=
      preContent.replace(/</g, '&lt;').replace(/>/g, '&gt;') + '</pre>\n';

    if (item.example) {
      content += `<div class='chalkTalkReferenceHeader'>Examples</div>\n`;
      content += `<pre class='chalkTalkReferenceMono'>${item.example
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')}</pre>`;
    }
  }

  return (
    <div
      dangerouslySetInnerHTML={{
        __html: `<div class='chalkTalkReferenceBody'><div class='chalkTalkReferenceContent'>${content}</div></div>`,
      }}
    ></div>
  );
};

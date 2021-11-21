import './TexTalkReferencePanel.css';

import { ref } from '../Reference';
import { getDocItems } from './TexTalkReference';

export const TexTalkReferencePanel = () => {
  let content = `<div class='texTalkReferenceTitle'>Expression Language Reference</div>`;

  for (const item of getDocItems()) {
    content += `<div class='texTalkReferenceName'>&lt;${item.name}&gt;</div>\n`;

    content += `<div class='texTalkReferenceHeader'>Specification</div>\n`;
    content += `<pre class='texTalkReferenceMono'>\n`;
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
      content += `<div class='texTalkReferenceHeader'>Examples</div>\n`;
      content += `<pre class='texTalkReferenceMono'>${item.example
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')}</pre>`;
    }
  }

  return (
    <div
      dangerouslySetInnerHTML={{
        __html: `<div class='texTalkReferenceBody'><div class='texTalkReferenceContent'>${content}</div></div>`,
      }}
    ></div>
  );
};

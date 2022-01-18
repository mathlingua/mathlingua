import parse, { DOMNode, domToReact } from 'html-react-parser';
import { Element } from 'domhandler/lib/node';

import React from 'react';
import './RenderedComponent.css';
import * as api from '../../services/api';
import { connect } from 'react-redux';
import * as uuid from 'uuid';

declare const katex: any;

export interface RenderedComponentProps {
  html: string;
  onViewEntity?: (entity: api.EntityResult) => void;
}

class RenderedComponent extends React.Component<RenderedComponentProps> {
  private ref: any;

  constructor(props: RenderedComponentProps) {
    super(props);
    this.ref = React.createRef();
  }

  componentDidMount() {
    // For some reason, on initial page load LaTeX is not rendered.
    // Calling renderLatex twice is an attempt to address this.
    renderLatex(this.ref.current);
    renderLatex(this.ref.current);
  }

  componentDidUpdate() {
    // For some reason, on initial page load LaTeX is not rendered.
    // Calling renderLatex twice is an attempt to address this.
    renderLatex(this.ref.current);
    renderLatex(this.ref.current);
  }

  render() {
    const mathlinguaToggleDropdown = (id: string) => {
      const el = document.getElementById(id);
      if (el) {
        if (el.className === 'mathlingua-dropdown-menu-hidden') {
          el.className = 'mathlingua-dropdown-menu-shown';
        } else {
          el.className = 'mathlingua-dropdown-menu-hidden';
        }
      }
    };

    const viewSignature = async (signature: string, id: string) => {
      mathlinguaToggleDropdown(id);
      const entityResult = await api.getEntityWithSignature(signature);
      if (entityResult && this.props.onViewEntity) {
        this.props.onViewEntity(entityResult);
      }
    };

    async function processInnerClick(fnName: string, fnArgs: string[]) {
      if (fnName === 'mathlinguaToggleDropdown') {
        const rawId = fnArgs[0];
        if (!rawId) {
          return;
        }
        // the raw id is enclosed in either single or double quotes
        const id = rawId.substring(1, rawId.length - 1);
        mathlinguaToggleDropdown(id);
      } else if (fnName === 'mathlinguaViewSignature') {
        const rawSignature = fnArgs[0];
        const rawId = fnArgs[1];
        if (!rawSignature || !rawId) {
          return;
        }
        // the javascript code has the backslashes escaped
        // thus they need to be un-escaped to get the actual signature
        const signature = rawSignature
          .substring(1, rawSignature.length - 1)
          .replace(/\\\\/, '\\');
        const id = rawId.substring(1, rawId.length - 1);
        viewSignature(signature, id);
      }
    }

    const options = {
      replace: (node: DOMNode) => {
        if (
          !(node instanceof Element) ||
          !node.attribs ||
          !node.attribs.onclick
        ) {
          return;
        }

        const onclick = node.attribs.onclick;
        delete node.attribs.onclick;

        const className = node.attribs['class'];
        delete node.attribs['class'];

        const exec = /([a-zA-Z0-9]+)\((.*?)\)/.exec(onclick.trim());
        if (exec && exec.length === 3) {
          const fnName = exec[1];
          const fnArgs = exec[2].split(',').map((it) => it.trim());
          return (
            <span
              className={className}
              onClick={() => processInnerClick(fnName, fnArgs)}
            >
              {domToReact(node.children, options)}
            </span>
          );
        }
      },
    };
    return (
      <div ref={this.ref}>
        {parse(this.props.html.replace(/CUSTOM_SUFFIX/g, uuid.v4()), options)}
      </div>
    );
  }
}

export default connect()(RenderedComponent);

function buildMathFragment(rawText: string) {
  let text = rawText;
  if (text[0] === '"') {
    text = text.substring(1);
  }
  if (text[text.length - 1] === '"') {
    text = text.substring(0, text.length - 1);
  }
  text = text.replace(/([a-zA-Z0-9])\?\??/g, '$1');
  const fragment = document.createDocumentFragment();
  let buffer = '';
  let i = 0;
  while (i < text.length) {
    if (text[i] === '$' && text[i + 1] === '$' && text[i + 2] === '$') {
      i += 3; // skip over the $s
      fragment.appendChild(document.createTextNode(buffer));
      buffer = '';

      const span = document.createElement('span');
      let math = '';
      while (
        i < text.length &&
        !(text[i] === '$' && text[i + 1] === '$' && text[i + 2] === '$')
      ) {
        math += text[i++];
      }
      if (text[i] === '$') {
        i++; // move past the $
      }
      if (text[i] === '$') {
        i++; // move past the second $
      }
      if (text[i] === '$') {
        i++; // move past the third $
      }
      try {
        katex.render(math, span, {
          throwOnError: true,
          displayMode: false,
        });
      } catch (e) {
        span.appendChild(document.createTextNode(math));
      }
      fragment.appendChild(span);
    } else if (text[i] === '\\' && text[i + 1] === '[') {
      i += 2; // skip over \ and [
      fragment.appendChild(document.createTextNode(buffer));
      buffer = '';

      const span = document.createElement('span');
      span.className = 'display-mode';
      let math = '';
      while (i < text.length && !(text[i] === '\\' && text[i + 1] === ']')) {
        math += text[i++];
      }
      if (text[i] === '\\') {
        i++; // move past the \
      }
      if (text[i] === ']') {
        i++; // move past the ]
      }
      try {
        katex.render(math, span, {
          throwOnError: true,
          displayMode: true,
        });
      } catch {
        span.appendChild(document.createTextNode(math));
      }
      fragment.appendChild(span);
    } else if (text[i] === '\\' && text[i + 1] === '(') {
      i += 2; // skip over \ and ()
      fragment.appendChild(document.createTextNode(buffer));
      buffer = '';

      const span = document.createElement('span');
      let math = '';
      while (i < text.length && !(text[i] === '\\' && text[i + 1] === ')')) {
        math += text[i++];
      }
      if (text[i] === '\\') {
        i++; // move past the \
      }
      if (text[i] === ')') {
        i++; // move past the )
      }
      try {
        katex.render(math, span, {
          throwOnError: true,
          displayMode: false,
        });
      } catch {
        span.appendChild(document.createTextNode(math));
      }
      fragment.appendChild(span);
    } else if (text[i] === '$' && text[i + 1] === '$') {
      i += 2; // skip over $ and $
      fragment.appendChild(document.createTextNode(buffer));
      buffer = '';

      const span = document.createElement('span');
      span.className = 'display-mode';
      let math = '';
      while (i < text.length && !(text[i] === '$' && text[i + 1] === '$')) {
        math += text[i++];
      }
      if (text[i] === '$') {
        i++; // move past the $
      }
      if (text[i] === '$') {
        i++; // move past the $
      }
      try {
        katex.render(math, span, {
          throwOnError: true,
          displayMode: true,
        });
      } catch {
        span.appendChild(document.createTextNode(math));
      }
      fragment.appendChild(span);
    } else if (text[i] === '$') {
      i++; // skip over the $
      fragment.appendChild(document.createTextNode(buffer));
      buffer = '';

      const span = document.createElement('span');
      let math = '';
      while (i < text.length && text[i] !== '$') {
        math += text[i++];
      }
      if (text[i] === '$') {
        i++; // move past the $
      }
      try {
        katex.render(math, span, {
          throwOnError: true,
          displayMode: false,
        });
      } catch {
        span.appendChild(document.createTextNode(math));
      }
      fragment.appendChild(span);
    } else {
      buffer += text[i++];
    }
  }

  if (buffer.length > 0) {
    fragment.appendChild(document.createTextNode(buffer));
  }

  return fragment;
}

function renderLatex(node: any) {
  if (
    node.className &&
    node.className.indexOf &&
    node.className.indexOf('no-render') >= 0
  ) {
    return;
  }

  let isInWritten = false;
  if (node.className === 'mathlingua') {
    for (let i = 0; i < node.childNodes.length; i++) {
      const n = node.childNodes[i];
      if (
        n &&
        n.className === 'mathlingua-header' &&
        n.textContent === 'written:'
      ) {
        isInWritten = true;
        break;
      }
    }
  }

  for (let i = 0; i < node.childNodes.length; i++) {
    const child = node.childNodes[i];

    // node is an element node => nodeType === 1
    // node is an attribute node => nodeType === 2
    // node is a text node => nodeType === 3
    // node is a comment node => nodeType === 8
    if (child.nodeType === 3) {
      let text = child.textContent;
      if (text.trim()) {
        if (isInWritten) {
          // if the text is in a written: section
          // turn "some text" to $$$some text$$$
          // so the text is in math mode
          if (text[0] === '"') {
            text = text.substring(1);
          }
          if (text[text.length - 1] === '"') {
            text = text.substring(0, text.length - 1);
          }
          text = '$$$' + text + '$$$';
        }
        const fragment = buildMathFragment(text);
        i += fragment.childNodes.length - 1;
        node.replaceChild(fragment, child);
      }
    } else if (child.nodeType === 1) {
      renderLatex(child);
    }
  }
}

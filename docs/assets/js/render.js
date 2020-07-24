
function buildMathFragment(rawText) {
    var text = rawText;
    if (text[0] === '"') {
        text = text.substring(1);
    }
    if (text[text.length - 1] === '"') {
        text = text.substring(0, text.length - 1);
    }
    text = text.replace(/([a-zA-Z0-9])\?\??/g, '$1');
    const fragment = document.createDocumentFragment();
    var buffer = '';
    var i = 0;
    while (i < text.length) {
        if (text[i] === '\\' && text[i+1] === '[') {
            i += 2; // skip over \ and [
            fragment.appendChild(document.createTextNode(buffer));
            buffer = '';

            const span = document.createElement('span');
            var math = '';
            while (i < text.length &&
                !(text[i] === '\\' && text[i+1] === ']')) {
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
                    displayMode: true
                });
            } catch {
                span.appendChild(document.createTextNode(math));
            }
            fragment.appendChild(span);
        } else if (text[i] === '\\' && text[i+1] === '(') {
            i += 2; // skip over \ and ()
            fragment.appendChild(document.createTextNode(buffer));
            buffer = '';

            const span = document.createElement('span');
            var math = '';
            while (i < text.length &&
                !(text[i] === '\\' && text[i+1] === ')')) {
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
                    displayMode: true
                });
            } catch {
                span.appendChild(document.createTextNode(math));
            }
            fragment.appendChild(span);
        } else if (text[i] === '$' && text[i+1] === '$') {
            i += 2; // skip over $ and $
            fragment.appendChild(document.createTextNode(buffer));
            buffer = '';

            const span = document.createElement('span');
            var math = '';
            while (i < text.length &&
                !(text[i] === '$' && text[i+1] === '$')) {
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
                    displayMode: true
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
            var math = '';
            while (i < text.length &&
                 text[i] !== '$') {
                math += text[i++];
            }
            if (text[i] === '$') {
                i++; // move past the $
            }
            try {
                katex.render(math, span, {
                    throwOnError: true,
                    displayMode: true
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

function render(node) {
    if (node.className && node.className.indexOf('no-render') >= 0) {
        return;
    }

    /*
     * The layout for nodes corresponding to text in a "written:" section
     * looks like the following.  Determine if any child nodes of 'node'
     * that are text nodes are in a "written:" section so that the text
     * can be trated as if it is in math mode.
     *
     * <span class='mathlingua'>
     *   ...
     *   <span class='mathlingua-header'>
     *     written:
     *   </span>
     *   ...
     *   <span class='mathlingua-text'>
     *     "some text"
     *   </span>
     * </span>
     */
    let isInWritten = false;
    const parent = node.parentNode;
    if (parent && node.className === 'mathlingua-text') {
        for (let j=0; j<parent.childNodes.length; j++) {
            const sibling = parent.childNodes[j];
            if (sibling.childNodes.length > 0 &&
                sibling.childNodes[0].nodeType === 3 &&
                sibling.childNodes[0].textContent === 'written:') {
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
                    // turn "some text" to \[some text\]
                    // so the text is in math mode
                    if (text[0] === '"') {
                        text = text.substring(1);
                    }
                    if (text[text.length - 1] === '"') {
                        text = text.substring(0, text.length - 1);
                    }
                    text = '\\[' + text + '\\]';
                }
                const fragment = buildMathFragment(text);
                i += fragment.childNodes.length - 1;
                node.replaceChild(fragment, child);
            }
        } else if (child.nodeType === 1) {
            render(child);
        }
    }
}

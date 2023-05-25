import React from 'react';

import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import remarkMath from 'remark-math'
import rehypeKatex from 'rehype-katex'

import 'katex/dist/katex.min.css';

export interface MarkdownViewProps {
  text: string;
  className?: string;
}

export const MarkdownView = (props: MarkdownViewProps) => {
  // For an unknown reason, if the `'$ $' +` below is not provided, then
  // the ReactMarkdown is incorrectly rendered as a <code> in the DOM
  const text = props.text.startsWith('$') ?
                props.text :
                '$ $' + format(props.text);
  return (
    <ReactMarkdown children={text}
                   remarkPlugins={[remarkGfm, remarkMath]}
                   rehypePlugins={[rehypeKatex]}
                   className={props.className} />
  );
};

function format(text: string): string {
  return text.replaceAll(/\\\[/g, '$$$$')
             .replaceAll(/\\\]/g, '$$$$')
             .replaceAll(/\\\(/g, '$')
             .replaceAll(/\\\)/g, '$');
}

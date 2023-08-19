import React from 'react';

import styles from './MarkdownView.module.css';

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
  return (
    <ReactMarkdown children={props.text}
                   remarkPlugins={[remarkGfm, remarkMath]}
                   rehypePlugins={[rehypeKatex]}
                   className={[styles.markdown,
                               props.className].filter(name => !!name)
                                               .join(' ')} />
  );
};

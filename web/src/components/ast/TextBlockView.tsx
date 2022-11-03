import React from 'react';

import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import remarkMath from 'remark-math'
import rehypeKatex from 'rehype-katex'

import 'katex/dist/katex.min.css';

import { TextBlock } from '../../types';

export interface TextBlockViewProps {
  node: TextBlock;
}

export const TextBlockView = (props: TextBlockViewProps) => {
  return (
    <div style={styles.mathlinguaTopLevelTextBlock}>
      <ReactMarkdown children={props.node.Text}
                     remarkPlugins={[remarkGfm, remarkMath]}
                     rehypePlugins={[rehypeKatex]} />
    </div>
  );
};

const styles = {
  mathlinguaTopLevelTextBlock: {
    padding: '1ex',
    color: 'black',
  },
};
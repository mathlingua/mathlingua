import React from 'react';

import styles from './TextArgumentDataView.module.css';

import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import remarkMath from 'remark-math'
import rehypeKatex from 'rehype-katex'

import 'katex/dist/katex.min.css';

import { TextArgumentData } from '../../types';
import { LatexView } from '../../design/LatexView';

export interface TextArgumentDataViewProps {
  node: TextArgumentData;
  forceRenderAsLatex: boolean;
  preProcess?: (text: string) => string;
}

export const TextArgumentDataView = (props: TextArgumentDataViewProps) => {
  const fn = props.preProcess ? props.preProcess : (text: string) => text;
  if (props.forceRenderAsLatex) {
    return <LatexView latex={fn(props.node.Text)} color='black' />
  }

  // For an unknown reason, if the `'$ $' +` below is not provided, then
  // the ReactMarkdown is incorrectly rendered as a <code> in the DOM
  return (
    <ReactMarkdown children={fn('$ $' + props.node.Text)}
                   remarkPlugins={[remarkGfm, remarkMath]}
                   rehypePlugins={[rehypeKatex]}
                   className={styles.markdown} />
  );
};

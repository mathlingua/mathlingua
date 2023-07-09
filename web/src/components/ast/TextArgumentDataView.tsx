import React from 'react';

import styles from './TextArgumentDataView.module.css';

import { TextArgumentData } from '../../types';
import { LatexView } from '../../design/LatexView';
import { MarkdownView } from '../../design/MarkdownView';

export interface TextArgumentDataViewProps {
  node: TextArgumentData;
  showSource: boolean;
  forceRenderAsLatex: boolean;
  preProcess?: (text: string) => string;
}

export const TextArgumentDataView = (props: TextArgumentDataViewProps) => {
  if (props.showSource) {
    return <span className={styles.showSourceWrapper}>"{props.node.Text}"</span>;
  }

  const fn = props.preProcess ? props.preProcess : (text: string) => text;
  if (props.forceRenderAsLatex) {
    return <LatexView latex={fn(props.node.Text)} color='black' />
  }

  return <MarkdownView text={fn(props.node.Text)}
                       className={styles.markdown} />;
};

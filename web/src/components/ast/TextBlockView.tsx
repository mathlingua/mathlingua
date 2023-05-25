import React from 'react';

import { TextBlock } from '../../types';

import styles from './TextBlockView.module.css';
import { MarkdownView } from '../../design/MarkdownView';

export interface TextBlockViewProps {
  node: TextBlock;
}

export const TextBlockView = (props: TextBlockViewProps) => {
  return (
    <div className={styles.mathlinguaTopLevelTextBlock}>
      <MarkdownView text={props.node.Text} />
    </div>
  );
};

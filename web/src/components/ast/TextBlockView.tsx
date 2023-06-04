import React from 'react';

import { TextBlock } from '../../types';

import styles from './TextBlockView.module.css';
import { MarkdownView } from '../../design/MarkdownView';

export interface TextBlockViewProps {
  node: TextBlock;
}

export const TextBlockView = (props: TextBlockViewProps) => {
  const rawText = props.node.Text;
  const lines = rawText.split('\n');
  /*
   * If the text is of the form:
   *   some.text\nmore text...
   * then the first line of text is an id for the text block that
   * should not be displayed.
   *
   * That is, the text block is of the form:
   *
   * ::some.text
   * more text...
   * ::
   */
  const text = lines.length > 1 ? lines.slice(1).join('\n') : rawText;
  return (
    <div className={styles.mathlinguaTopLevelTextBlock}>
      <MarkdownView text={text} />
    </div>
  );
};

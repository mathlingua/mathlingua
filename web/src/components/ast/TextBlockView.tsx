import React from 'react';

import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import remarkMath from 'remark-math'
import rehypeKatex from 'rehype-katex'

import 'katex/dist/katex.min.css';

import { TextBlock } from '../../types';
import { Theme } from '../../base/theme';
import { useTheme } from '../../hooks/theme';

export interface TextBlockViewProps {
  node: TextBlock;
}

export const TextBlockView = (props: TextBlockViewProps) => {
  const theme = useTheme();
  const styles = getStyles(theme);

  return (
    <div style={styles.mathlinguaTopLevelTextBlock}>
      <ReactMarkdown children={props.node.Text}
                     remarkPlugins={[remarkGfm, remarkMath]}
                     rehypePlugins={[rehypeKatex]} />
    </div>
  );
};

function getStyles(theme: Theme) {
  return {
    mathlinguaTopLevelTextBlock: {
      padding: '1ex',
      color: theme.colors.textBlockColor,
    },
  };
}

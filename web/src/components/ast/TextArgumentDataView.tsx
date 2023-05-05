import React from 'react';
import { TextArgumentData } from '../../types';
import { LatexView } from '../LatexView';
import { Theme } from '../../base/theme';
import { useTheme } from '../../hooks/theme';

export interface TextArgumentDataViewProps {
  node: TextArgumentData;
  forceRenderAsLatex: boolean;
  preProcess?: (text: string) => string;
}

export const TextArgumentDataView = (props: TextArgumentDataViewProps) => {
  const theme = useTheme();
  const styles = getStyles(theme);

  const fn = props.preProcess ? props.preProcess : (text: string) => text;
  if (props.forceRenderAsLatex) {
    return <LatexView latex={fn(props.node.Text)} color={styles.mathlinguaTextBlock.color} />
  }

  return (
    <span style={styles.mathlinguaTextBlock}>
      {fn(props.node.Text)}
    </span>
  );
};

function getStyles(theme: Theme) {
  return {
    mathlinguaTextBlock: {
      color: theme.colors.textArgumentColor,
    }
  };
}

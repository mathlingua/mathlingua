import React from 'react';
import { TextArgumentData } from '../../types';
import { LatexView } from '../LatexView';

export interface TextArgumentDataViewProps {
  node: TextArgumentData;
  forceRenderAsLatex: boolean;
}

export const TextArgumentDataView = (props: TextArgumentDataViewProps) => {
  if (props.forceRenderAsLatex) {
    return <LatexView latex={props.node.Text} color={styles.mathlinguaTextBlock.color} />
  }

  return (
    <span style={styles.mathlinguaTextBlock}>
      {props.node.Text}
    </span>
  );
};

const styles = {
  mathlinguaTextBlock: {
    color: 'black',
  }
};

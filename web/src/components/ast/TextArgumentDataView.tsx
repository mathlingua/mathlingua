import React from 'react';

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

  return (
    <span>
      {fn(props.node.Text)}
    </span>
  );
};

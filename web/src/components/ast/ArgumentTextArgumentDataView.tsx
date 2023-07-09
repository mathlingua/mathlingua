import React from 'react';
import { ArgumentTextArgumentData } from '../../types';
import { LatexView } from '../../design/LatexView';

export interface ArgumentTextArgumentDataViewProps {
  node: ArgumentTextArgumentData;
  showSource: boolean;
  preProcess?: (text: string) => string;
}

export const ArgumentTextArgumentDataView = (props: ArgumentTextArgumentDataViewProps) => {
  if (props.showSource) {
    return <span>{props.node.Text}</span>;
  }

  const fn = props.preProcess ? props.preProcess : (text: string) => text;
  return <LatexView
    latex={fn(props.node.Text)}
    color={'black'} />;
};

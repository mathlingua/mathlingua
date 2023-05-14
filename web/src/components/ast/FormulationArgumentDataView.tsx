import React from 'react';
import { FormulationArgumentData } from '../../types';
import { LatexView } from '../../design/LatexView';

export interface FormulationArgumentDataViewProps {
  node: FormulationArgumentData;
  preProcess?: (text: string) => string;
}

export const FormulationArgumentDataView = (props: FormulationArgumentDataViewProps) => {
  const fn = props.preProcess ? props.preProcess : (text: string) => text;
  return <LatexView latex={fn(props.node.Text)} color={'black'} />;
};

import React from 'react';
import { FormulationArgumentData } from '../../types';
import { LatexView } from '../LatexView';

export interface FormulationArgumentDataViewProps {
  node: FormulationArgumentData;
}

export const FormulationArgumentDataView = (props: FormulationArgumentDataViewProps) => {
  return <LatexView latex={props.node.Text} color='black' />
};

import React from 'react';
import { ArgumentTextArgumentData } from '../../types';
import { LatexView } from '../LatexView';

export interface ArgumentTextArgumentDataViewProps {
  node: ArgumentTextArgumentData;
}

export const ArgumentTextArgumentDataView = (props: ArgumentTextArgumentDataViewProps) => {
  return <LatexView latex={props.node.Text} color={'black'} />;
};

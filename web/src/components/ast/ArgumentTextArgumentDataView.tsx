import React from 'react';
import { ArgumentTextArgumentData } from '../../types';
import { LatexView } from '../LatexView';
import { useTheme } from '../../hooks/theme';

export interface ArgumentTextArgumentDataViewProps {
  node: ArgumentTextArgumentData;
  preProcess?: (text: string) => string;
}

export const ArgumentTextArgumentDataView = (props: ArgumentTextArgumentDataViewProps) => {
  const theme = useTheme();
  const fn = props.preProcess ? props.preProcess : (text: string) => text;
  return <LatexView
    latex={fn(props.node.Text)}
    color={theme.colors.textArgumentColor} />;
};

import React from 'react';
import { ArgumentDataKind } from '../../types';
import { ArgumentTextArgumentDataView } from './ArgumentTextArgumentDataView';
import { FormulationArgumentDataView } from './FormulationArgumentDataView';
import { GroupView } from './GroupView';
import { TextArgumentDataView } from './TextArgumentDataView';

export interface ArgumentDataKindViewProps {
  node: ArgumentDataKind;
  indent: number;
  forceRenderAsLatex: boolean;
  preProcess?: (text: string) => string;
}

export const ArgumentDataKindView = (props: ArgumentDataKindViewProps) => {
  if (props.node?.Type === 'GroupType') {
    return <GroupView node={props.node as any} indent={props.indent} />;
  } else if (props.node?.Type === 'TextArgumentDataKind') {
    return <TextArgumentDataView
      node={props.node as any}
      forceRenderAsLatex={props.forceRenderAsLatex}
      preProcess={props.preProcess} />;
  } else if (props.node?.Type === 'FormulationArgumentDataKind') {
    return <FormulationArgumentDataView
      node={props.node as any}
      preProcess={props.preProcess} />;
  } else if (props.node?.Type === 'ArgumentTextArgumentDataKind') {
    return <ArgumentTextArgumentDataView
      node={props.node as any}
      preProcess={props.preProcess} />;
  } else {
    return null;
  }
};

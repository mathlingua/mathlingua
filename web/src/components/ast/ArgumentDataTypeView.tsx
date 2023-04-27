import React from 'react';
import { ArgumentDataType } from '../../types';
import { ArgumentTextArgumentDataView } from './ArgumentTextArgumentDataView';
import { FormulationArgumentDataView } from './FormulationArgumentDataView';
import { GroupView } from './GroupView';
import { TextArgumentDataView } from './TextArgumentDataView';

export interface ArgumentDataTypeViewProps {
  node: ArgumentDataType;
  indent: number;
  forceRenderAsLatex: boolean;
  preProcess?: (text: string) => string;
}

export const ArgumentDataTypeView = (props: ArgumentDataTypeViewProps) => {
  if (props.node?.Type === 'GroupType') {
    return <GroupView node={props.node as any} indent={props.indent} />;
  } else if (props.node?.Type === 'TextArgumentDataType') {
    return <TextArgumentDataView
      node={props.node as any}
      forceRenderAsLatex={props.forceRenderAsLatex}
      preProcess={props.preProcess} />;
  } else if (props.node?.Type === 'FormulationArgumentDataType') {
    return <FormulationArgumentDataView
      node={props.node as any}
      preProcess={props.preProcess} />;
  } else if (props.node?.Type === 'ArgumentTextArgumentDataType') {
    return <ArgumentTextArgumentDataView
      node={props.node as any}
      preProcess={props.preProcess} />;
  } else {
    return null;
  }
};

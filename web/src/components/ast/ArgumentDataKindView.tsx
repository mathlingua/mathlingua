import React from 'react';
import { ArgumentDataKind } from '../../types';
import { ArgumentTextArgumentDataView } from './ArgumentTextArgumentDataView';
import { FormulationArgumentDataView } from './FormulationArgumentDataView';
import { GroupView } from './GroupView';
import { TextArgumentDataView } from './TextArgumentDataView';

export interface ArgumentDataKindViewProps {
  node: ArgumentDataKind;
  showSource: boolean;
  indent: number;
  forceRenderAsLatex: boolean;
  preProcess?: (text: string) => string;
  onSelectedSignature: (signature: string) => void;
}

export const ArgumentDataKindView = (props: ArgumentDataKindViewProps) => {
  if (props.node?.Type === 'GroupType') {
    return <GroupView
      node={props.node as any}
      showSource={props.showSource}
      indent={props.indent}
      onSelectedSignature={props.onSelectedSignature} />;
  } else if (props.node?.Type === 'TextArgumentDataKind') {
    return <TextArgumentDataView
      node={props.node as any}
      showSource={props.showSource}
      forceRenderAsLatex={props.forceRenderAsLatex}
      preProcess={props.preProcess} />;
  } else if (props.node?.Type === 'FormulationArgumentDataKind') {
    return <FormulationArgumentDataView
      node={props.node as any}
      showSource={props.showSource}
      preProcess={props.preProcess}
      onSelectedSignature={props.onSelectedSignature} />;
  } else if (props.node?.Type === 'ArgumentTextArgumentDataKind') {
    return <ArgumentTextArgumentDataView
      node={props.node as any}
      showSource={props.showSource}
      preProcess={props.preProcess} />;
  } else {
    return null;
  }
};

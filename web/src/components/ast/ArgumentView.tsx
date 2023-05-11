import React from 'react';
import { Argument } from '../../types';
import { ArgumentDataKindView } from './ArgumentDataKindView';
import { Dot } from './Dot';
import { Indent } from './Indent';
import { Space } from './Space';

export interface ArgumentViewProps {
  node: Argument;
  indent: number;
  forceRenderAsLatex: boolean;
  preProcess?: (text: string) => string;
}

export const ArgumentView = (props: ArgumentViewProps) => {
  let component: React.ReactNode;
  if (props.node.IsInline) {
    component = <ArgumentDataKindView
      node={props.node.Arg}
      indent={props.indent+2}
      forceRenderAsLatex={props.forceRenderAsLatex}
      preProcess={props.preProcess} />;
  } else {
    component = (
      <>
        <Indent size={props.indent}/><Dot /><Space />
        <ArgumentDataKindView
          node={props.node.Arg}
          indent={props.indent+2}
          forceRenderAsLatex={props.forceRenderAsLatex} 
          preProcess={props.preProcess} />
      </>
    );
  }

  return component;
};

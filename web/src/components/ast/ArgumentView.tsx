import React from 'react';
import { Argument } from '../../types';
import { ArgumentDataTypeView } from './ArgumentDataTypeView';
import { Dot } from './Dot';
import { Indent } from './Indent';
import { Space } from './Space';

export interface ArgumentViewProps {
  node: Argument;
  indent: number;
}

export const ArgumentView = (props: ArgumentViewProps) => {
  let component: React.ReactNode;
  if (props.node.IsInline) {
    component = <ArgumentDataTypeView node={props.node.Arg} indent={props.indent+2} />;
  } else {
    component = (
      <>
        <Indent size={props.indent}/><Dot /><Space />
        <ArgumentDataTypeView node={props.node.Arg} indent={props.indent+2} />
      </>
    );
  }

  return component;
};

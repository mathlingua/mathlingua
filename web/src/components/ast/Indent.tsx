import React from 'react';
import { Space } from './Space';

export interface IndentProps {
  size: number;
  showSource: boolean;
}

export const Indent = (props: IndentProps) => {
  const nodes: React.ReactElement[] = [];
  for (let i=0; i<props.size; i++) {
    nodes.push(<Space key={i} showSource={props.showSource} />);
  }
  return (
    <>
      {nodes}
    </>
  );
};

import React from 'react';
import { Space } from './Space';

export interface IndentProps {
  size: number;
}

export const Indent = (props: IndentProps) => {
  const nodes: React.ReactElement[] = [];
  for (let i=0; i<props.size; i++) {
    nodes.push(<Space key={i} />);
  }
  return (
    <>
      {nodes}
    </>
  );
};

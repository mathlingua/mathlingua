import React from 'react';
import { Root } from '../../types';
import { TopLevelNodeTypeView } from './TopLevelNodeTypeView';

export interface RootViewProps {
  node: Root;
}

export const RootView = (props: RootViewProps) => {
  return (
    <>
      {
        props.node.Nodes?.map((node, index) => (
          <span key={index}>
            <TopLevelNodeTypeView node={node} />
          </span>
        ))
      }
    </>
  );
};

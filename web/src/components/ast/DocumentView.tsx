import React from 'react';
import { Document } from '../../types';
import { TopLevelNodeTypeView } from './TopLevelNodeTypeView';

export interface DocumentViewProps {
  node: Document;
  isOnSmallScreen: boolean;
}

export const DocumentView = (props: DocumentViewProps) => {
  return (
    <>
      {
        props.node.Nodes?.map((node, index) => (
          <span key={index}>
            <TopLevelNodeTypeView node={node} isOnSmallScreen={props.isOnSmallScreen} />
          </span>
        ))
      }
    </>
  );
};

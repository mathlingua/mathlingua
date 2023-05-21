import React from 'react';
import { Document } from '../../types';
import { TopLevelNodeKindView } from './TopLevelNodeKindView';

export interface DocumentViewProps {
  node: Document;
  isOnSmallScreen: boolean;
}

export const DocumentView = (props: DocumentViewProps) => {
  return (
    <>
      {
        props.node.Nodes?.map((node, index) => (
          <span id={node?.MetaData.Id ?? ''}  key={index}>
            <TopLevelNodeKindView node={node} isOnSmallScreen={props.isOnSmallScreen} />
          </span>
        ))
      }
    </>
  );
};

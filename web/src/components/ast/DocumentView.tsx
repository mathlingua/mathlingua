import React from 'react';

import { Document } from '../../types';
import { MultiTopLevelItem } from './MultiTopLevelItem';

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
            <MultiTopLevelItem node={node} isOnSmallScreen={props.isOnSmallScreen} />
          </span>
        ))
      }
    </>
  );
};

import React from 'react';
import { Group } from '../../types';
import { IdView } from './IdView';
import { Indent } from './Indent';
import { Newline } from './Newline';
import { SectionView } from './SectionView';
import { Space } from './Space';

export interface GroupViewProps {
  node: Group;
  indent: number;
}

export const GroupView = (props: GroupViewProps) => {
  return (
    <span>
      <IdView id={props.node.Id} />
      {
        props.node.Sections?.map((sec, index) => (
          <span key={index}>
            {index > 0 && <Newline />}
            {index > 0 && <Indent size={props.indent} />}
            <SectionView node={sec} indent={props.indent} />
          </span>
        ))
      }
    </span>
  );
};

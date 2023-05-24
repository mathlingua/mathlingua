import React from 'react';
import { Group } from '../../types';
import { Indent } from './Indent';
import { Newline } from './Newline';
import { SectionView } from './SectionView';

export interface GroupViewProps {
  node: Group;
  indent: number;
  onSelectedSignature: (signature: string) => void;
}

export const GroupView = (props: GroupViewProps) => {
  return (
    <span>
      {
        props.node.Sections?.map((sec, index) => (
          <span key={index}>
            {index > 0 && <Newline />}
            {index > 0 && <Indent size={props.indent} />}
            {sec.Name === 'Id' ? null : <SectionView
              node={sec}
              indent={props.indent}
              onSelectedSignature={props.onSelectedSignature} />}
          </span>
        ))
      }
    </span>
  );
};

import React from 'react';

import { Section } from '../../types';
import { ArgumentView } from './ArgumentView';
import { Comma } from './Comma';
import { Newline } from './Newline';
import { Space } from './Space';

export interface SectionViewProps {
  node: Section;
  indent: number;
}

export const SectionView = (props: SectionViewProps) => {
  return (
    <>
      <span style={styles.mathlinguaHeader}>{props.node.Name}</span>:<Space/>
      {
        props.node.Args?.map((arg, index) => (
          <span key={index}>
            {!arg.IsInline && <Newline />}
            {(arg.IsInline && props.node.Args?.[index-1]?.IsInline) &&
              <><Comma /></>}
            <ArgumentView node={arg} indent={props.indent} />
          </span>
        ))
      }
    </>
  );
};

const styles = {
  mathlinguaHeader: {
    color: '#05b',
  }
};

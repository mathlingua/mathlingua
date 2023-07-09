import React from 'react';

import styles from './GroupView.module.css';

import { Group, Section } from '../../types';
import { Indent } from './Indent';
import { Newline } from './Newline';
import { SectionView } from './SectionView';

export interface GroupViewProps {
  node: Group;
  showSource: boolean;
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
            {section(sec, props.showSource, props.indent, props.onSelectedSignature)}
          </span>
        ))
      }
    </span>
  );
};

function section(
  sec: Section,
  showSource: boolean,
  indent: number,
  onSelectedSignature: (signature: string) => void,
) {
  if (sec.Name === 'Id') {
    return null;
  }

  if (sec.Name === 'Proof') {
    // The following is needed since otherwise there is an
    // extra gap at the bottom of the group.
    return <div className={styles.proofBottomGapFix}></div>;
  }

  const secView = <SectionView
                    node={sec}
                    showSource={showSource}
                    indent={indent}
                    onSelectedSignature={onSelectedSignature} />;

  const secHasSeparator = sec.Name === 'Provides' || sec.Name === 'Documented' ||
                          sec.Name === 'References' || sec.Name === 'Aliases';
  if (!showSource && secHasSeparator) {
    return (
      <>
        <div className={styles.separator}></div>
        {secView}
      </>
    );
  }

  return secView;
}

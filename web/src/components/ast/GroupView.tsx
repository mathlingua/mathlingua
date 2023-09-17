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
  skipDocumentsSection: boolean;
  onSelectedSignature: (signature: string) => void;
}

export const GroupView = (props: GroupViewProps) => {
  return (
    <span>
      {
        props.node.Sections?.map((sec, index) => (
          section(sec, props.showSource, props.indent, props.skipDocumentsSection,
            props.onSelectedSignature, index)
        ))
      }
    </span>
  );
};

function section(
  sec: Section,
  showSource: boolean,
  indent: number,
  skipDocumentsSection: boolean,
  onSelectedSignature: (signature: string) => void,
  index: number,
) {
  if (skipDocumentsSection && sec.Name === 'Documented') {
    return null;
  }

  if (sec.Name === 'Id') {
    return null;
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
      <span key={index}>
        {index > 0 && <Newline />}
        {index > 0 && <Indent size={indent} />}
        <div className={styles.separator}></div>
        {secView}
      </span>
    );
  }

  return (
    <span key={index}>
      {index > 0 && <Newline />}
      {index > 0 && <Indent size={indent} />}
      {secView}
    </span>
  );
}

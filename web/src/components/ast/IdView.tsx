import React from 'react';

import styles from './IdView.module.css';
import { LatexView } from '../../design/LatexView';

export interface IdViewProps {
  id: string | null;
  idSrc: string | null;
  showSource: boolean;
  isLatex: boolean;
}

export const IdView = (props: IdViewProps) => {
  if (props.id === null) {
    return null;
  }

  if (props.showSource) {
    return (
      <div className={styles.idSrc}>
        [{props.idSrc ?? ''}]
      </div>
    );
  }

  if (props.isLatex) {
    return (
      <div>
        <LatexView latex={props.id} color='black' />
        <div className={styles.line} />
      </div>
    );
  }

  return (
    <div>
      {props.id}
      <div className={styles.line} />
    </div>
  );
};

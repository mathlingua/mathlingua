import React from 'react';
import { TextArgumentData } from '../../types';

export interface TextArgumentDataViewProps {
  node: TextArgumentData;
}

export const TextArgumentDataView = (props: TextArgumentDataViewProps) => {
  return (
    <span style={styles.mathlinguaTextBlock}>
      "{props.node.Text}"
    </span>
  );
};

const styles = {
  mathlinguaTextBlock: {
    color: '#386930',
  }
};

import React from 'react';
import { ArgumentTextArgumentData } from '../../types';

export interface ArgumentTextArgumentDataViewProps {
  node: ArgumentTextArgumentData;
}

export const ArgumentTextArgumentDataView = (props: ArgumentTextArgumentDataViewProps) => {
  return (
    <span style={styles.argumentText}>
      {props.node.Text}
    </span>
  );
};

const styles = {
  argumentText: {
    color: '#585858',
  },
};

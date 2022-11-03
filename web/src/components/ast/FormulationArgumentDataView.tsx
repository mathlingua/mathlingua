import React from 'react';
import { FormulationArgumentData } from '../../types';

export interface FormulationArgumentDataViewProps {
  node: FormulationArgumentData;
}

export const FormulationArgumentDataView = (props: FormulationArgumentDataViewProps) => {
  return (
    <span style={styles.mathlinguaFormulation}>
      '{props.node.Text}'
    </span>
  );
};

const styles = {
  mathlinguaFormulation: {
    color: 'darkcyan',
  }
};

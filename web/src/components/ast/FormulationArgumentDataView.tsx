import React from 'react';

import DownIcon from '@rsuite/icons/ArrowDown';

import styles from './FormulationArgumentDataView.module.css';

import { FormulationArgumentData } from '../../types';
import { LatexView } from '../../design/LatexView';

export interface FormulationArgumentDataViewProps {
  node: FormulationArgumentData;
  preProcess?: (text: string) => string;
  onSelectedSignature: (signature: string) => void;
}

export const FormulationArgumentDataView = (props: FormulationArgumentDataViewProps) => {
  const [showDropdown, setShowDropdown] = React.useState(false);

  const usedSignatures = Array.from(
    new Set(props.node.FormulationMetaData.UsedSignatureStrings ?? [])).sort();

  const fn = props.preProcess ? props.preProcess : (text: string) => text;
  const dropdownMenuCss = {
    // the `usedSignature.length > 0` check is needed so that an empty
    // dropdown is not shown if the formulation text is clicked and
    // there are not any signatures to show
    display: showDropdown && usedSignatures.length > 0 ? undefined : 'none',
  };
  return (
    <div className={styles.dropdownContainer}>
      <button className={styles.latexViewButton}
              onClick={() => setShowDropdown((shown) => !shown)}>
        <LatexView latex={fn(props.node.Text)} color={'black'} />
        {usedSignatures.length > 0 && <DownIcon className={styles.downIcon} />}
      </button>
      <div className={styles.dropdownMenu} style={dropdownMenuCss}>
        {usedSignatures.map(src => (
          <button className={styles.dropdownMenuItem}
                  key={src}
                  onClick={() => {
                    props.onSelectedSignature(src);
                    setShowDropdown(false);
                  }}>
            {src}
          </button>
        ))}
      </div>
    </div>
  );
};

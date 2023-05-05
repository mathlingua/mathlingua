import React from 'react';

import { Section } from '../../types';
import { ArgumentView } from './ArgumentView';
import { Comma } from './Comma';
import { Newline } from './Newline';
import { Space } from './Space';
import { Theme } from '../../base/theme';
import { useTheme } from '../../hooks/theme';

export interface SectionViewProps {
  node: Section;
  indent: number;
}

export const SectionView = (props: SectionViewProps) => {
  const theme = useTheme();
  const styles = getStyles(theme);

  return (
    <>
      <span style={styles.mathlinguaHeader}>{props.node.Name}</span>:<Space/>
      {
        props.node.Args?.map((arg, index) => (
          <span key={index}>
            {!arg.IsInline && <Newline />}
            {(arg.IsInline && props.node.Args?.[index-1]?.IsInline) &&
              <><Comma /></>}
            <ArgumentView
              node={arg}
              indent={props.indent}
              forceRenderAsLatex={props.node.Name === 'written' || props.node.Name == 'called'}
              preProcess={props.node.Name === 'called' ?
                (text: string) => `\\textrm\{${text}\}` : undefined} />
          </span>
        ))
      }
    </>
  );
};

function getStyles(theme: Theme) {
  return {
    mathlinguaHeader: {
      color: theme.colors.sectionHeaderColor,
    }
  };
}

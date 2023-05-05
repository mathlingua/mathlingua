import React from 'react';
import { useTheme } from '../../hooks/theme';
import { Theme } from '../../base/theme';

export interface IdViewProps {
  id: string | null;
}

export const IdView = (props: IdViewProps) => {
  const theme = useTheme();
  const styles = getStyles(theme);

  if (props.id === null) {
    return null;
  }

  return (
    <div style={styles.mathlinguaId}>
      {props.id}
      <hr style={styles.line} />
    </div>
  );
};

function getStyles(theme: Theme) {
  return {
    mathlinguaId: {
      color: 'black',
    },
    line: {
      marginTop: theme.sizeXXSmall,
      marginBottom: theme.sizeXXSmall,
      marginLeft: 0,
      marginRight: 0,
      padding: 0,
    },
  };
}

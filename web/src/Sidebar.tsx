import React from 'react';
import { Theme, useTheme } from './hooks';

export function Sidebar() {
  const theme = useTheme();
  const styles = getStyles(theme);

  return (
    <>
      <span style={styles.top}>
      </span>
      <span style={styles.bottom}>
      </span>
    </>
  );
}

function getStyles(theme: Theme) {
  return {
    top: {
      height: theme.sizeXLarge,
      width: theme.sidebarWidth,
      position: 'fixed',
      left: 0,
      top: 0,
      margin: 0,
      padding: 0,
      borderBottom: 'solid',
      borderBottomColor: theme.gray,
      borderBottomWidth: 1,
    },
    bottom: {
      height: '100%',
      width: '100%',
      position: 'fixed',
      marginTop: theme.sizeXLarge,
      marginBottom: 0,
      marginLeft: 0,
      marginRight: 0,
      padding: 0,
      overflow: 'auto',
    },
  } as const;
}

import React from 'react';

import { Theme, useTheme } from './hooks';

export interface ShellProps {
  sidebarContent: React.ReactNode;
  topbarContent: React.ReactNode;
  mainContent: React.ReactNode;
  showSidebar: boolean;
}

export function Shell(props: ShellProps) {
  const theme = useTheme();
  const styles = getStyles(theme, props.showSidebar);

  return (
    <>
      <span style={styles.topbar}>
        {props.topbarContent}
      </span>
      <span style={props.showSidebar ? styles.sidebar : styles.hidden}>
        {props.sidebarContent}
      </span>
      <span style={styles.content}>
        {props.mainContent}
      </span>
    </>
  );
}

function getStyles(theme: Theme, showSidebar: boolean) {
  const sidebarWidth = showSidebar ? theme.sidebarWidth : 0;
  return {
    sidebar: {
      position: 'fixed',
      height: '100%',
      width: sidebarWidth,
      margin: 0,
      padding: 0,
      overflow: 'auto',
      borderRight: 'solid',
      borderColor: theme.gray,
      borderWidth: 1,
    },
    content: {
      position: 'fixed',
      marginLeft: sidebarWidth,
      marginTop: theme.sizeLarge,
      marginRight: 0,
      marginBottom: 0,
      padding: 0,
      height: '100%',
      width: '100%',
      overflow: 'auto',
      background: 'white',
      borderTop: 'solid',
      borderColor: theme.gray,
      borderWidth: 1,
    },
    topbar: {
      position: 'fixed',
      marginLeft: sidebarWidth,
      padding: 0,
      marginRight: 0,
      marginTop: 0,
      marginBottom: 0,
      width: '100%',
      height: theme.sizeLarge,
    },
    hidden: {
      display: 'none',
    },
  } as const;
}

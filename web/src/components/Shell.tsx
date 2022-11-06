import React from 'react';

import { Theme, useTheme } from '../hooks/theme';

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
    <div style={styles.wrapper}>
      <div style={styles.topbar}>
        {props.topbarContent}
      </div>
      <div style={styles.leftSidebar}>
        {props.sidebarContent}
      </div>
      <div style={styles.content}>
        {props.mainContent}
      </div>
      <div style={styles.rightSidebar}>
      </div>
    </div>
  );
}

function getStyles(theme: Theme, showSidebar: boolean) {
  return {
    wrapper: {
      display: 'grid',
      gridTemplateColumns: `${theme.sidebarWidth}px auto auto ${theme.sidebarWidth}px`,
      gridTemplateAreas: `
        'topbar      topbar  topbar  topbar'
        'leftSidebar content content rightSidebar'
      `,
      overflow: 'scroll',
    },
    leftSidebar: {
      gridArea: 'leftSidebar',
      position: 'fixed',
      top: theme.sizeXLarge,
      left: 0,
      overflow: 'scroll',
      width: showSidebar ? theme.sidebarWidth : 0,
      transition: '0.5s',
      height: `calc(100% - ${theme.sizeXLarge}px)`,
      borderRight: showSidebar ? 'solid' : 'none',
      borderColor: showSidebar ? theme.gray : 'white',
      borderWidth: showSidebar ? 1 : 0,
      background: 'white',
    },
    rightSidebar: {
      gridArea: 'rightSidebar',
    },
    content: {
      gridArea: 'content',
      position: 'relative',
      overflow: 'scroll',
      marginTop: theme.sizeXLarge,
    },
    topbar: {
      gridArea: 'topbar',
      position: 'fixed',
      width: '100%',
      height: theme.sizeXLarge,
      top: 0,
      left: 0,
      borderBottom: 'solid',
      borderColor: theme.gray,
      borderWidth: 1,
      background: 'white',
      zIndex: 1, // so the topbar covers the page content
    },
    hidden: {
      display: 'none',
    },
  } as const;
}

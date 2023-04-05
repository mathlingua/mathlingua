import React from 'react';

import { Theme, useTheme } from '../hooks/theme';

export interface ShellProps {
  sidebarContent: React.ReactNode;
  topbarContent: React.ReactNode;
  mainContent: React.ReactNode;
  showSidebar: boolean;
  isOnSmallScreen: boolean;
}

export function Shell(props: ShellProps) {
  const theme = useTheme();
  const styles = getStyles(theme, props.showSidebar, props.isOnSmallScreen);

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

function getStyles(theme: Theme, showSidebar: boolean, isSmallScreen: boolean) {
  if (isSmallScreen) {
    return getSmallScreenStyles(theme, showSidebar);
  } else {
    return getLargeScreenStyles(theme, showSidebar);
  }
}

function getSmallScreenStyles(theme: Theme, showSidebar: boolean) {
  return {
    wrapper: {
      overflow: 'scroll',
      background: theme.background,
      height: '100%',
    },
    leftSidebar: {
      gridArea: 'leftSidebar',
      position: 'fixed',
      top: theme.sizeXLarge,
      left: -1,
      overflow: 'scroll',
      width: showSidebar ? '101%' : 0,
      height: `calc(100% - ${theme.sizeXLarge}px)`,
      background: theme.background,
      borderRight: 'solid',
      borderColor: theme.border,
      borderWidth: 1,
      zIndex: 2,
      transition: '0.25s',
    },
    rightSidebar: {
      display: 'none',
    },
    content: {
      position: 'relative',
      overflow: 'scroll',
      marginTop: theme.sizeXLarge,
    },
    topbar: {
      position: 'fixed',
      width: '100%',
      height: theme.sizeXLarge,
      top: 0,
      left: 0,
      background: theme.background,
      borderBottom: 'solid',
      borderWidth: 1,
      borderColor: theme.border,
      zIndex: 2,
    },
    hidden: {
      display: 'none',
    },
  } as const;
}

function getLargeScreenStyles(theme: Theme, showSidebar: boolean) {
  return {
    wrapper: {
      display: 'grid',
      gridTemplateColumns: `${theme.sidebarWidth}px auto auto ${theme.sidebarWidth}px`,
      gridTemplateAreas: `
        'topbar      topbar  topbar  topbar'
        'leftSidebar content content rightSidebar'
      `,
      overflow: 'scroll',
      background: theme.background,
      height: '100%',
    },
    leftSidebar: {
      gridArea: 'leftSidebar',
      position: 'fixed',
      top: theme.sizeXLarge,
      left: -1,
      overflow: 'scroll',
      width: showSidebar ? theme.sidebarWidth : 0,
      height: `calc(100% - ${theme.sizeXLarge}px)`,
      background: theme.background,
      borderRight: 'solid',
      borderColor: theme.border,
      borderWidth: 1,
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
      background: theme.background,
      borderBottom: 'solid',
      borderWidth: 1,
      borderColor: theme.border,
      zIndex: 2,
    },
    hidden: {
      display: 'none',
    },
  } as const;
}

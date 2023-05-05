import React from 'react';

import { useTheme } from '../hooks/theme';
import { Theme } from '../base/theme';

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
        <div style={styles.contentWrapper}>
          {props.mainContent}
        </div>
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
      zIndex: 1,
      transition: '0.25s',
      boxShadow: 'rgba(50, 50, 105, 0.15) 0px 2px 5px 0px, rgba(0, 0, 0, 0.05) 0px 1px 1px 0px',
    },
    rightSidebar: {
      display: 'none',
    },
    contentWrapper: {
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
      boxShadow: 'rgba(50, 50, 105, 0.15) 0px 2px 5px 0px, rgba(0, 0, 0, 0.05) 0px 1px 1px 0px',
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
      gridTemplateColumns: `auto auto auto`,
      gridTemplateAreas: `
        'topbar      topbar  topbar'
        'leftSidebar content rightSidebar '
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
      zIndex: 1,
      boxShadow: 'rgba(50, 50, 105, 0.15) 0px 2px 5px 0px, rgba(0, 0, 0, 0.05) 0px 1px 1px 0px',
    },
    rightSidebar: {
      gridArea: 'rightSidebar',
    },
    contentWrapper: {
      marginLeft: showSidebar ? '0' : 'auto',
      marginRight: 'auto',
      width: 'fit-content',
    },
    content: {
      gridArea: 'content',
      position: 'relative',
      overflow: 'scroll',
      marginTop: theme.sizeXLarge,
      paddingLeft: showSidebar ? theme.sidebarWidth : 0,
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
      boxShadow: 'rgba(50, 50, 105, 0.15) 0px 2px 5px 0px, rgba(0, 0, 0, 0.05) 0px 1px 1px 0px',
    },
    hidden: {
      display: 'none',
    },
  } as const;
}

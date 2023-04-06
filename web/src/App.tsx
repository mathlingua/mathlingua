import React, { useState } from 'react';

import MenuIcon from '@rsuite/icons/Menu';

import { useFetch } from 'usehooks-ts';
import { Theme, useTheme } from './hooks/theme';
import { Shell } from './components/Shell';
import { Sidebar } from './components/Sidebar';
import { PageResponse } from './types';
import { DocumentView } from './components/ast/DocumentView';

export function App() {
  const theme = useTheme();

  const [windowWidth, setWindowWidth] = useState(window.innerWidth);
  const isOnSmallScreen = determineIsOnSmallScreen(windowWidth, theme);
  const [showSidebar, setShowSidebar] = React.useState(!isOnSmallScreen);
  const [selectedPath, setSelectedPath] = React.useState(undefined as string | undefined);

  window.addEventListener('resize', () => {
    const newWidth = window.innerWidth;
    setWindowWidth(newWidth);
    const isOnSmallScreen = determineIsOnSmallScreen(newWidth, theme);
    if (showSidebar && isOnSmallScreen) {
      setShowSidebar(false);
    }
  });

  const styles = getStyles(theme, isOnSmallScreen);

  const [activePath, setActivePath] = React.useState<string>('');
  const { data } = useFetch<PageResponse>(`/api/page?path=${encodeURIComponent(activePath)}`);

  const topbar = (
    <button style={styles.menuButton}
            onClick={() => setShowSidebar(!showSidebar)}>
      <MenuIcon />
    </button>
  );

  const sidebar = (
    <div style={styles.sidebar}>
      <div style={styles.outline}>
        Outline
      </div>
      <Sidebar
        selectedPath={selectedPath}
        onSelect={(path, isInit) => {
          setSelectedPath(path);
          if (path.endsWith('.math')) {
            setActivePath(path);
            if (!isInit && isOnSmallScreen) {
              setShowSidebar(false);
            }
          }
        }}/>
    </div>
  );

  const mainContent = (
    <div style={styles.content}>
      {data?.Document && (
        <div style={styles.page}>
          <DocumentView node={data?.Document} isOnSmallScreen={isOnSmallScreen} />
        </div>
      )}
    </div>
  );

  return (
    <Shell
      showSidebar={showSidebar}
      topbarContent={topbar}
      sidebarContent={sidebar}
      mainContent={mainContent}
      isOnSmallScreen={isOnSmallScreen} />
  );
}

function getStyles(theme: Theme, isOnSmallScreen: boolean) {
  return {
    sidebar: {
      width: 'max-content',
      height: 'max-content',
    },
    content: {
      height: 'max-content',
      overflow: 'scroll',
      marginLeft: 'auto',
      marginRight: 'auto',
      width: isOnSmallScreen ? '100%' : theme.mainWidth,
    },
    menuButton: {
      background: 'none',
      border: 'none',
      margin: theme.sizeXSmall,
    },
    page: {
      background: 'white',
      paddingLeft: isOnSmallScreen ? '1ex' : '4em',
      paddingRight: isOnSmallScreen ? '1ex' : '4em',
      paddingTop: isOnSmallScreen ? '1ex' : '1em',
      paddingBottom: isOnSmallScreen ? '1ex' : '1em',
      margin: '1ex',
    },
    outline: {
      fontWeight: 'bold',
      paddingTop: theme.sizeXSmall,
      paddingLeft: theme.sizeSmall,
      fontSize: '110%',
    },
  };
}

export function determineIsOnSmallScreen(windowWidth: number, theme: Theme) {
  const maxWidth = theme.sidebarWidth *1.25;
  return windowWidth <= maxWidth;
}

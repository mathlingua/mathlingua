import React, { useState } from 'react';

import MenuIcon from '@rsuite/icons/Menu';

import { useFetch } from 'usehooks-ts';
import { Theme, useTheme } from './hooks/theme';
import { Shell } from './components/Shell';
import { Sidebar } from './components/Sidebar';
import { PageResponse } from './types';
import { RootView } from './components/ast/RootView';

export function App() {
  const theme = useTheme();

  const [screenWidth, setScreenWidth] = useState(window.screen.availWidth);
  const isOnSmallScreen = determineIsOnSmallScreen(screenWidth, theme);
  const [showSidebar, setShowSidebar] = React.useState(!isOnSmallScreen);

  window.addEventListener('resize', () => {
    setScreenWidth(window.screen.availWidth);
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
      <Sidebar onSelect={(path) => {
        if (path.endsWith('.math')) {
          setActivePath(path);
          if (isOnSmallScreen) {
            setShowSidebar(false);
          }
        }
      }}/>
    </div>
  );

  const mainContent = (
    <div style={styles.content}>
      {data?.Root && (
        <div style={styles.page}>
          <RootView node={data?.Root} isOnSmallScreen={isOnSmallScreen} />
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
      boxShadow: '0 1px 5px rgba(0,0,0,.2)',
      paddingLeft: isOnSmallScreen ? '2em' : '4em',
      paddingRight: isOnSmallScreen ? '2em' : '4em',
      paddingTop: isOnSmallScreen ? '2em' : '2em',
      paddingBottom: isOnSmallScreen ? '2em' : '2em',
      margin: '1ex',
      borderRadius: 2,
    },
  };
}

export function determineIsOnSmallScreen(screenWidth: number, theme: Theme) {
  const maxWidth = theme.sidebarWidth*2 + theme.mainWidth;
  return screenWidth <= maxWidth;
}

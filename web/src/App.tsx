import React from 'react';

import MenuIcon from '@rsuite/icons/Menu';

import { useFetch } from 'usehooks-ts';
import { Theme, useTheme } from './hooks/theme';
import { Shell } from './components/Shell';
import { Sidebar } from './components/Sidebar';
import { PageResponse } from './types';
import { RootView } from './components/ast/RootView';

export function App() {
  const theme = useTheme();

  const [showSidebar, setShowSidebar] = React.useState(true);
  const styles = getStyles(theme);

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
        }
      }}/>
    </div>
  );

  const mainContent = (
    <div style={styles.content}>
      {data?.Root && <RootView node={data?.Root} />}
    </div>
  );

  return (
    <div>
      <Shell
        showSidebar={showSidebar}
        topbarContent={topbar}
        sidebarContent={sidebar}
        mainContent={mainContent} />
    </div>
  );
}

function getStyles(theme: Theme) {
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
      width: '800px',
    },
    menuButton: {
      background: 'none',
      border: 'none',
      margin: theme.sizeXSmall,
    },
  };
}

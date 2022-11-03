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
  const styles = getStyles(theme, showSidebar);

  const [activePath, setActivePath] = React.useState<string>('');
  const { data } = useFetch<PageResponse>(`/api/page?path=${encodeURIComponent(activePath)}`);

  const topbar = (
    <button style={styles.menuButton}
            onClick={() => setShowSidebar(!showSidebar)}>
      <MenuIcon />
    </button>
  );

  const sidebar = <Sidebar onSelect={(path) => {
    if (path.endsWith('.math')) {
      setActivePath(path);
    }
  }}/>;

  const mainContent = (
    <div style={styles.content}>
      {data?.Root && <RootView node={data?.Root} />}
    </div>
  );

  return (
    <Shell
      showSidebar={showSidebar}
      topbarContent={topbar}
      sidebarContent={sidebar}
      mainContent={mainContent} />
  );
}

function getStyles(theme: Theme, sidebarVisible: boolean) {
  return {
    content: {
      margin: theme.sizeSmall,
      padding: theme.sizeXXLarge,
      maxWidth: '800px',
      marginLeft: sidebarVisible ? theme.sizeXXLarge : 'auto',
      marginRight: 'auto',
    },
    menuButton: {
      background: 'none',
      border: 'none',
      margin: theme.sizeXXSmall,
    },
  };
}

import React from 'react';

import MenuIcon from '@rsuite/icons/Menu';

import { useFetch } from 'usehooks-ts';
import { Theme, useTheme } from './hooks';
import { Shell } from './Shell';
import { Sidebar } from './Sidebar';

export function App() {
  const theme = useTheme();
  const styles = getStyles(theme);

  const [showSidebar, setShowSidebar] = React.useState(true);

  const [activePath, setActivePath] = React.useState<string>('');
  const { data } = useFetch(`/api/page?path=${encodeURIComponent(activePath)}`);

  const topbar = (
    <button style={styles.menuButton}
            onClick={() => setShowSidebar(!showSidebar)}>
      <MenuIcon />
    </button>
  );

  const sidebar = <Sidebar onSelect={setActivePath}/>;

  const mainContent = (
    <div style={styles.content}
         dangerouslySetInnerHTML={{
           __html: (data as any)?.Html ?? ''
         }}>
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

function getStyles(theme: Theme) {
  return {
    content: {
      margin: theme.sizeSmall,
      paddingBottom: theme.sizeXXLarge,
    },
    menuButton: {
      background: 'none',
      border: 'none',
      margin: theme.sizeXXSmall,
    },
  };
}

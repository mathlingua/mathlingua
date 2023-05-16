import React from 'react';

import styles from './MainPage.module.css';

import MenuIcon from '@rsuite/icons/Menu';
import { useFetch } from 'usehooks-ts';
import { PageResponse } from '../types';
import { Sidebar } from '../components/Sidebar';
import { DocumentView } from '../components/ast/DocumentView';
import { Button } from '../design/Button';

export function MainPage() {
  const [windowWidth, setWindowWidth] = React.useState(window.innerWidth);
  const isOnSmallScreen = determineIsOnSmallScreen(windowWidth);
  const [open, setOpen] = React.useState(!isOnSmallScreen);

  window.addEventListener('resize', () => {
    const newWidth = window.innerWidth;
    setWindowWidth(newWidth);
  });

  const [activePath, setActivePath] = React.useState<string>('');
  const { data } = useFetch<PageResponse>(`/api/page?path=${encodeURIComponent(activePath)}`);

  const sidebar = (
    <div className={styles.sidebar}>
      <Sidebar
        onSelect={(pathItem) => {
          if (pathItem.path.endsWith('.math')) {
            setActivePath(pathItem.path);
            if (isOnSmallScreen) {
              setOpen(false);
            }
          }
        }}/>
    </div>
  );

  const mainContent = (
    <div className={styles.mainContent}>
      {data?.Document && (
        <div className={styles.page}>
          <DocumentView node={data?.Document} isOnSmallScreen={isOnSmallScreen} />
        </div>
      )}
    </div>
  );


  const leftStyle = {
    display: open ? 'block' : 'none',
  };

  return (
    <div className={styles.outerWrapper}>
      <header className={styles.header}>
        <Button flat
                onClick={() => {
          setOpen(open => !open);
        }}>
          <MenuIcon style={{
            color: 'black',
            marginLeft: 10,
            marginRight: 0,
            marginTop: 0,
            marginBottom: 0,
            padding: 0,
          }} />
        </Button>
      </header>
      <div className={styles.contentWrapper}>
        <main className={styles.content}>
          <nav className={styles.left} style={leftStyle}>
            {sidebar}
          </nav>
          <div className={styles.center}>
            {mainContent}
          </div>
        </main>
      </div>
      <footer className={styles.footer}>
      </footer>
    </div>
  );
}

function determineIsOnSmallScreen(windowWidth: number) {
  return windowWidth <= 450;
}

import React from 'react';

import styles from './MainPage.module.css';

import MenuIcon from '@rsuite/icons/Menu';
import GearIcon from '@rsuite/icons/Gear';
import LeftIcon from '@rsuite/icons/ArrowLeftLine';
import RightIcon from '@rsuite/icons/ArrowRightLine';

import { useFetch } from 'usehooks-ts';
import { PageResponse, PathsResponse } from '../types';
import { Sidebar } from '../components/Sidebar';
import { DocumentView } from '../components/ast/DocumentView';
import { Button } from '../design/Button';
import { Link, useLocation, useNavigate } from 'react-router-dom';
import { getNextTheme, useTheme } from '../hooks/useTheme';

export function MainPage() {
  const {setTheme} = useTheme();

  const navigate = useNavigate();
  const location = useLocation();

  const isOnSmallScreen = determineIsOnSmallScreen(window.innerWidth);
  const [open, setOpen] = React.useState(!isOnSmallScreen);

  React.useEffect(() => {
    const listener = () => {
      if (open && determineIsOnSmallScreen(window.innerWidth)) {
        setOpen(false);
      }
    };

    window.addEventListener('resize', listener);
    return () => window.removeEventListener('resize', listener);
    // open should not be in the dependency array since the effect
    // should only be run when the component first renders
    // eslint-disable-next-line
  }, []);

  const pathname = location.pathname;
  const trimmedPathname = pathname.startsWith('/') ? pathname.substring(1) : pathname;

  const { data: activePathData } = useFetch<PageResponse>(
    `/api/page?path=${encodeURIComponent(trimmedPathname)}`);
  const { data: pathsData } = useFetch<PathsResponse>('/api/paths');

  const allPaths = pathsData?.Paths?.filter(path => path.Path.endsWith('.math')) ?? [];
  const activeIndex = allPaths.findIndex((path) => path.Path === trimmedPathname);
  const prevItem = (activeIndex > 0) ? allPaths[activeIndex - 1] : null;
  const nextItem = (activeIndex < allPaths.length - 1) ? allPaths[activeIndex + 1] : null;

  React.useEffect(() => {
    const first = pathsData?.Paths?.[0];
    if (location.pathname === "/" && first !== undefined) {
      navigate(first.Path);
    }
  }, [pathsData, location.pathname, navigate]);

  const sidebar = (
    <div className={styles.sidebar}>
      <Sidebar
        selectedPath={trimmedPathname}
        allPaths={pathsData?.Paths ?? null}
        onSelect={(pathItem) => {
          if (pathItem.endsWith('.math') && isOnSmallScreen) {
            setOpen(false);
          }
        }}/>
    </div>
  );

  const mainContent = (
    <div className={styles.mainContent}>
      {activePathData?.Document && (
        <div className={styles.page}>
          <DocumentView node={activePathData?.Document} isOnSmallScreen={isOnSmallScreen} />
        </div>
      )}
      {(prevItem !== null || nextItem !== null) &&
        (
          <div className={styles.navigationWrapper}>
            {prevItem?.Path ?
              (
                <Link to={prevItem.Path}>
                  <Button flat className={styles.navigationButtonLeft}><LeftIcon /></Button>
                </Link>
               ) : (
                <Button flat className={styles.navigationButtonLeftDisabled}><LeftIcon /></Button>
               )}
            {nextItem?.Path ?
              (
                <Link to={nextItem.Path}>
                  <Button flat className={styles.navigationButtonRight}><RightIcon /></Button>
                </Link>
              ) : (
                <Button flat className={styles.navigationButtonRightDisabled}><RightIcon /></Button>
              )}
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
          setOpen(oldOpen => !oldOpen);
        }}>
          <MenuIcon className={styles.headerIcon} />
        </Button>
        <GearIcon className={styles.headerIcon}
                  onClick={() => {
                    setTheme(theme => getNextTheme(theme));
                  }}/>
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
  return windowWidth <= 864;
}

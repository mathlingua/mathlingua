import { useLocation } from 'react-router';
import { Page } from '../page/Page';
import { SidePanel } from '../side-panel/SidePanel';
import styles from './ContentPanel.module.css';

export interface HashLocation {
  viewedPath: string;
  targetId: string;
}

function getHashLocation(location: {
  pathname: string;
  hash: string;
}): HashLocation {
  return {
    viewedPath: location.pathname.replace(/^\//, ''),
    targetId: location.hash.replace(/^#/, ''),
  };
}

export const ContentPanel = () => {
  const hashLocation = getHashLocation(useLocation());
  return (
    <div className={styles.contentPane}>
      <SidePanel viewedPath={hashLocation.viewedPath} />
      <Page
        viewedPath={hashLocation.viewedPath}
        targetId={hashLocation.targetId}
      />
    </div>
  );
};

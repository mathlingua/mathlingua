import { useLocation } from 'react-router';
import { Page } from '../page/Page';
import { SidePanel } from '../side-panel/SidePanel';
import styles from './ContentPanel.module.css';

export const ContentPanel = () => {
  const viewedPath = useLocation().pathname.replace(/^\//, '');
  return (
    <div className={styles.contentPane}>
      <SidePanel viewedPath={viewedPath} />
      <Page viewedPath={viewedPath} />
    </div>
  );
};

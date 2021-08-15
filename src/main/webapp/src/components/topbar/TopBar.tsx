import styles from './TopBar.module.css';

import { SidePanelButton } from '../side-panel-button/SidePanelButton';
import { SearchPanel } from '../search-panel/SearchPanel';
import { GitHubIcon } from '../github-icon/GitHubIcon';

export const TopBar = () => {
  return (
    <div className={styles.topbar}>
      <SidePanelButton />
      <SearchPanel />
      <GitHubIcon />
    </div>
  );
};

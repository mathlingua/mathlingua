import styles from './TopBar.module.css';

import { SidePanelButton } from '../side-panel-button/SidePanelButton';
import { SearchPanel } from '../search-panel/SearchPanel';
import { GitHubIcon } from '../github-icon/GitHubIcon';
import { isStatic } from '../../services/api';
import { EditModeButton } from '../edit-mode-button/EditModeButton';
import { HelpButton } from '../help-button/HelpButton';

export const TopBar = () => {
  return (
    <div className={styles.topbar}>
      <SidePanelButton />
      <SearchPanel />
      {isStatic() ? null : <EditModeButton />}
      <GitHubIcon />
      <HelpButton />
    </div>
  );
};

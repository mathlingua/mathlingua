import styles from './GitHubIcon.module.css';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faGithub } from '@fortawesome/free-brands-svg-icons';

export const GitHubIcon = () => {
  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        padding: 0,
        marginLeft: 0,
        marginRight: 0,
        marginTop: 0,
        marginBottom: 'auto',
      }}
    >
      <a href="https://github.com/DominicKramer/codex">
        <FontAwesomeIcon className={styles.githubIcon} icon={faGithub} />
      </a>
    </div>
  );
};

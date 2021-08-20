import styles from './GitHubIcon.module.css';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faGithub } from '@fortawesome/free-brands-svg-icons';

export const GitHubIcon = () => {
  return (
    <a href="https://github.com/DominicKramer/codex">
      <FontAwesomeIcon className={styles.githubIcon} icon={faGithub} />
    </a>
  );
};

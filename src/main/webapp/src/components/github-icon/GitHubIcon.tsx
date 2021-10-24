import styles from './GitHubIcon.module.css';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faGithub } from '@fortawesome/free-brands-svg-icons';
import { useEffect, useState } from 'react';
import { getGitHubUrl } from '../../services/api';

export const GitHubIcon = () => {
  const [url, setUrl] = useState('' as string | undefined);

  useEffect(() => {
    getGitHubUrl().then((url) => setUrl(url));
  }, []);

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
      {url ? (
        <a href={url} target="_blank">
          <FontAwesomeIcon className={styles.githubIcon} icon={faGithub} />
        </a>
      ) : (
        <FontAwesomeIcon className={styles.githubIcon} icon={faGithub} />
      )}
    </div>
  );
};

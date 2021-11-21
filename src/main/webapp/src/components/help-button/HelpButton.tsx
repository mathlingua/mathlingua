import styles from './HelpButton.module.css';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faQuestionCircle } from '@fortawesome/free-solid-svg-icons';

export const HelpButton = () => {
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
      <a href="#/help" target="_blank">
        <FontAwesomeIcon
          className={styles.helpIcon}
          icon={faQuestionCircle}
          style={{
            filter: 'drop-shadow(0.45px 0.45px 0px rgba(0, 0, 0, 0.2))',
          }}
        />
      </a>
    </div>
  );
};

import { useState } from 'react';
import RenderedComponent from '../rendered-component/RenderedComponent';
import styles from './TopLevelEntry.module.css';

import { EntityResult } from '../../services/api';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faTimes, faShare } from '@fortawesome/free-solid-svg-icons';
import { Link } from 'react-router-dom';

export interface TopLevelEntryProps {
  id: string;
  relativePath: string;
  showOpenButton: boolean;
  showCloseButton: boolean;
  rawHtml: string;
  renderedHtml: string;
  onViewEntity?: (entity: EntityResult) => void;
  onEntityClosed?: (id: string) => void;
}

export const TopLevelEntry = (props: TopLevelEntryProps) => {
  const [showRendered, setShowRendered] = useState(true);

  return (
    <div id={props.id} className={styles.mathlinguaTopLevel}>
      {props.showCloseButton ? (
        <button
          className={styles.mathlinguaCloseButton}
          onClick={() => {
            if (props.onEntityClosed) {
              props.onEntityClosed(props.id);
            }
          }}
          data-test-id="close-single-top-level-entry"
        >
          <FontAwesomeIcon icon={faTimes} />
        </button>
      ) : null}
      <span className={styles.mathlinguaIconPane}>
        {props.showOpenButton ? (
          <Link
            to={`/${props.relativePath}#${props.id}`}
            className={styles.mathlinguaOpenIcon}
          >
            <FontAwesomeIcon icon={faShare} />
          </Link>
        ) : null}
        <button
          className={styles.mathlinguaFlipIcon}
          onClick={() => {
            setShowRendered(!showRendered);
          }}
        >
          •
        </button>
      </span>
      {showRendered ? (
        <RenderedComponent
          onViewEntity={props.onViewEntity}
          html={props.renderedHtml}
        />
      ) : (
        <RenderedComponent
          onViewEntity={props.onViewEntity}
          html={props.rawHtml}
        />
      )}
    </div>
  );
};

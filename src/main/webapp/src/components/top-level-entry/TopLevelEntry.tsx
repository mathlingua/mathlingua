import { useState } from 'react';
import RenderedComponent from '../rendered-component/RenderedComponent';
import styles from './TopLevelEntry.module.css';

import { EntityResult } from '../../services/api';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faTimes } from '@fortawesome/free-solid-svg-icons';

export interface TopLevelEntryProps {
  id: string;
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
        >
          <FontAwesomeIcon icon={faTimes} />
        </button>
      ) : null}
      <button
        className={styles.mathlinguaFlipIcon}
        onClick={() => {
          setShowRendered(!showRendered);
        }}
      >
        •
      </button>
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

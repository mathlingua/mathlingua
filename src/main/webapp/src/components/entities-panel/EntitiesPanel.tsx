import styles from './EntitiesPanel.module.css';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faTimes } from '@fortawesome/free-solid-svg-icons';

export interface EntryPane {
  id: string;
  // the JSX.Element that is a <TopLevelEntry/>
  topLevelEntry: JSX.Element;
}

export interface EntitiesPanelProps {
  entityPanes: EntryPane[];
  onCloseAll?: () => void;
}

export const EntitiesPanel = (props: EntitiesPanelProps) => {
  return (
    <div
      className={styles.entitiesPanel}
      style={
        props.entityPanes.length > 0
          ? { display: 'block' }
          : { display: 'none' }
      }
    >
      <div>
        <button
          className={styles.closeAllButton}
          onClick={() => {
            if (props.onCloseAll) {
              props.onCloseAll();
            }
          }}
          data-test-id="close-all-entities"
        >
          <FontAwesomeIcon icon={faTimes} />
        </button>
        {props.entityPanes.map((pane) => (
          <span key={pane.id} className={styles.entitiesPanelItem}>
            {pane.topLevelEntry}
          </span>
        ))}
      </div>
    </div>
  );
};

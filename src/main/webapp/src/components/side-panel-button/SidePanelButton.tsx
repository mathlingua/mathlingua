import styles from './SidePanelButton.module.css';

import {
  selectSidePanelVisible,
  sidePanelVisibilityChanged,
} from '../../store/sidePanelVisibleSlice';
import { useAppDispatch, useAppSelector } from '../../support/hooks';

import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { faBars } from '@fortawesome/free-solid-svg-icons';

export const SidePanelButton = () => {
  const visible = useAppSelector(selectSidePanelVisible);
  const dispatch = useAppDispatch();

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
      <button
        className={styles.sidePanelButton}
        onClick={() => {
          dispatch(sidePanelVisibilityChanged(!visible));
        }}
      >
        <FontAwesomeIcon icon={faBars} />
      </button>
    </div>
  );
};

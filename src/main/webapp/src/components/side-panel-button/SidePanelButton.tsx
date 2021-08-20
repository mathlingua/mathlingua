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
    <button
      className={styles.sidePanelButton}
      onClick={() => {
        dispatch(sidePanelVisibilityChanged(!visible));
      }}
    >
      <FontAwesomeIcon icon={faBars} />
    </button>
  );
};

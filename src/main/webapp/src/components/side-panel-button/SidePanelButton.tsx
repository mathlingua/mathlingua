import styles from './SidePanelButton.module.css';

import {
  selectSidePanelVisible,
  sidePanelVisibilityChanged,
} from '../../store/sidePanelVisibleSlice';
import { useAppDispatch, useAppSelector } from '../../support/hooks';

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
      &#x2630;
    </button>
  );
};

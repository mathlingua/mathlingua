import {
  isEditModeUpdated,
  selectIsEditMode,
} from '../../store/isEditModeSlice';
import { sidePanelVisibilityChanged } from '../../store/sidePanelVisibleSlice';
import { useAppDispatch, useAppSelector } from '../../support/hooks';
import styles from './EditModeButton.module.css';

export const EditModeButton = () => {
  const dispatch = useAppDispatch();
  const isEditMode = useAppSelector(selectIsEditMode);

  return (
    <button
      className={styles.editModeButton}
      onClick={() => {
        const newEditMode = !isEditMode;
        dispatch(isEditModeUpdated(newEditMode));
        if (newEditMode) {
          dispatch(sidePanelVisibilityChanged(false));
        }
      }}
    >
      {isEditMode ? 'View' : 'Edit'}
    </button>
  );
};
